mod adapters;
mod composition_root;
mod handler;
mod operation_log;
mod pipe_server;
mod tray_manager;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::composition_root::AppContext;

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()))
        })
        .join(".futou")
}

fn resolve_data_dir<I>(args: I) -> PathBuf
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--data-dir" {
            if let Some(path) = args.next().filter(|path| !path.trim().is_empty()) {
                return PathBuf::from(path);
            }
        }
    }
    default_data_dir()
}

fn load_or_create_config(data_dir: &Path) -> PathBuf {
    let config_path = data_dir.join("config.toml");
    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let default_config = format!(
            r#"data_dir = "{}"
"#,
            data_dir.to_string_lossy().replace('\\', "\\\\")
        );
        let _ = std::fs::write(&config_path, default_config);
    }
    config_path
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let data_dir = resolve_data_dir(std::env::args().skip(1));
    let _config_path = load_or_create_config(&data_dir);

    info!("futou daemon starting. Data directory: {:?}", data_dir);

    let ctx = match composition_root::build_composition_root(&data_dir).await {
        Ok(ctx) => Arc::new(tokio::sync::RwLock::new(ctx)),
        Err(e) => {
            error!("Failed to initialize: {}", e);
            return;
        }
    };

    let shutdown_tx = {
        let lock = ctx.read().await;
        lock.shutdown_tx.clone()
    };

    let ctx_pipe = ctx.clone();
    let pipe_handle = tokio::spawn(async move {
        pipe_server::run_pipe_server(ctx_pipe, "futou").await;
    });

    let tx = shutdown_tx.clone();
    std::thread::spawn(move || {
        tray_manager::run_tray(tx);
    });

    let mut shutdown_rx = shutdown_tx.subscribe();
    shutdown_rx.recv().await.ok();
    info!("Shutdown signal received, stopping daemon");

    pipe_handle.abort();

    {
        let lock = ctx.read().await;
        info!("Shutting down aria2");
        lock.downloader.shutdown().await;
    }

    info!("Daemon stopped");
}

#[cfg(test)]
mod tests {
    use super::resolve_data_dir;
    use std::path::PathBuf;

    #[test]
    fn resolves_configured_data_directory() {
        let args = ["--data-dir".to_string(), r"D:\SDK".to_string()];

        assert_eq!(resolve_data_dir(args), PathBuf::from(r"D:\SDK"));
    }
}
