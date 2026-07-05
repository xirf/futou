#![cfg(windows)]

mod pipe_client;

use clap::{Parser, Subcommand};
use futou_ipc::messages::{
    ActivateParams, CatalogueListResult, DeactivateParams, InstallParams, ProgressParams,
    RuntimeListResult, UninstallParams,
};
use pipe_client::PipeClient;

const PIPE_NAME: &str = "futou";

#[derive(Parser)]
#[command(name = "futou", about = "futou Environment Manager", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed runtimes
    List,
    /// Install a runtime version
    Install { runtime: String, version: String },
    /// Uninstall a runtime version
    Uninstall { runtime: String, version: String },
    /// Activate a runtime version
    Use { runtime: String, version: String },
    /// Deactivate a runtime
    Deactivate { runtime: String },
    /// List available runtimes in catalogue
    Catalogue,
    /// Show daemon status
    Status,
    /// Refresh the catalogue
    Refresh,
}

async fn connect() -> Result<PipeClient, String> {
    PipeClient::connect(PIPE_NAME).await
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List => cmd_list().await,
        Commands::Install { runtime, version } => cmd_install(&runtime, &version).await,
        Commands::Uninstall { runtime, version } => cmd_uninstall(&runtime, &version).await,
        Commands::Use { runtime, version } => cmd_use(&runtime, &version).await,
        Commands::Deactivate { runtime } => cmd_deactivate(&runtime).await,
        Commands::Catalogue => cmd_catalogue().await,
        Commands::Status => cmd_status().await,
        Commands::Refresh => cmd_refresh().await,
    };

    match result {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn cmd_list() -> Result<String, String> {
    let mut client = connect().await?;
    let result = client.send_request("runtime.list", None).await?;
    let list: RuntimeListResult = serde_json::from_value(result).map_err(|e| e.to_string())?;

    if list.installed.is_empty() {
        return Ok("No runtimes installed.".to_string());
    }

    let mut output = String::from("Installed runtimes:\n");
    for r in &list.installed {
        output.push_str(&format!("  {} {} [{}]\n", r.runtime, r.version, r.status));
    }
    Ok(output)
}

async fn cmd_install(runtime: &str, version: &str) -> Result<String, String> {
    let mut client = connect().await?;
    let mut notif_rx = client.notification_receiver();

    let params = serde_json::to_value(InstallParams {
        runtime: runtime.to_string(),
        version: version.to_string(),
    })
    .unwrap();

    let send = client.send_request("runtime.install", Some(params));
    let recv = notif_rx.recv();

    tokio::pin!(send);
    tokio::pin!(recv);

    loop {
        tokio::select! {
            result = &mut send => {
                let _ = result?;
                return Ok(format!("{} {} installed successfully", runtime, version));
            }
            notif = &mut recv => {
                if let Ok(notification) = notif {
                    if let Some(params) = notification.params {
                        if let Ok(p) = serde_json::from_value::<ProgressParams>(params) {
                            print!("\r{} {:.1}%", p.message, p.progress * 100.0);
                            use std::io::Write;
                            std::io::stdout().flush().ok();
                        }
                    }
                }
            }
        }
    }
}

async fn cmd_uninstall(runtime: &str, version: &str) -> Result<String, String> {
    eprintln!(
        "Warning: This will delete all data for {} {}!",
        runtime, version
    );
    eprint!("Type 'yes' to confirm: ");
    use std::io::Write;
    std::io::stderr().flush().ok();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    if input.trim() != "yes" {
        return Err("Cancelled".to_string());
    }

    let mut client = connect().await?;
    let params = serde_json::to_value(UninstallParams {
        runtime: runtime.to_string(),
        version: version.to_string(),
    })
    .unwrap();
    client
        .send_request("runtime.uninstall", Some(params))
        .await?;
    Ok(format!("{} {} uninstalled", runtime, version))
}

async fn cmd_use(runtime: &str, version: &str) -> Result<String, String> {
    let mut client = connect().await?;
    let params = serde_json::to_value(ActivateParams {
        runtime: runtime.to_string(),
        version: version.to_string(),
    })
    .unwrap();
    client
        .send_request("runtime.activate", Some(params))
        .await?;
    Ok(format!("{} {} is now active", runtime, version))
}

async fn cmd_deactivate(runtime: &str) -> Result<String, String> {
    let mut client = connect().await?;
    let params = serde_json::to_value(DeactivateParams {
        runtime: runtime.to_string(),
    })
    .unwrap();
    client
        .send_request("runtime.deactivate", Some(params))
        .await?;
    Ok(format!("{} deactivated", runtime))
}

async fn cmd_catalogue() -> Result<String, String> {
    let mut client = connect().await?;
    let result = client.send_request("catalogue.list", None).await?;
    let cat: CatalogueListResult = serde_json::from_value(result).map_err(|e| e.to_string())?;

    if cat.runtimes.is_empty() {
        return Ok("Catalogue is empty or unavailable.".to_string());
    }

    let mut output = String::from("Available runtimes:\n");
    for r in &cat.runtimes {
        let versions = r.versions.join(", ");
        output.push_str(&format!(
            "  {} ({}): {}\n",
            r.display_name, r.name, versions
        ));
    }
    Ok(output)
}

async fn cmd_status() -> Result<String, String> {
    let mut client = connect().await?;
    let result = client.send_request("daemon.status", None).await?;
    Ok(format!(
        "Daemon status: {}",
        serde_json::to_string_pretty(&result).unwrap()
    ))
}

async fn cmd_refresh() -> Result<String, String> {
    let mut client = connect().await?;
    client.send_request("catalogue.refresh", None).await?;
    Ok("Catalogue refreshed.".to_string())
}
