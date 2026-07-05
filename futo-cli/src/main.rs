#![cfg(windows)]

mod pipe_client;

use clap::{Parser, Subcommand};
use futou_ipc::messages::{
    ActivateParams, CatalogueListResult, DeactivateParams, InstallParams, RuntimeListResult,
    UninstallParams,
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
    /// Activate a runtime (add to PATH)
    Use { runtime: String, version: String },
    /// Deactivate a runtime (remove from PATH)
    Deactivate { runtime: String },
    /// Start a server process (database or web server)
    Start {
        runtime: String,
        #[arg(short, long)]
        version: Option<String>,
        #[arg(short = 'd', long)]
        document_root: Option<String>,
    },
    /// Stop a server process
    Stop { runtime: String },
    /// Show operation logs for a runtime
    Logs { runtime: String },
    /// Show currently active runtimes
    Active,
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
        Commands::Start {
            runtime,
            version,
            document_root,
        } => cmd_start(&runtime, version.as_deref(), document_root.as_deref()).await,
        Commands::Stop { runtime } => cmd_stop(&runtime).await,
        Commands::Logs { runtime } => cmd_logs(&runtime).await,
        Commands::Active => cmd_active().await,
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
        let running = r.process.as_ref().map(|_| " [running]").unwrap_or("");
        output.push_str(&format!(
            "  {} {} [{}{}]\n",
            r.runtime, r.version, r.status, running
        ));
    }
    Ok(output)
}

async fn cmd_install(runtime: &str, version: &str) -> Result<String, String> {
    let mut client = connect().await?;
    let params = serde_json::to_value(InstallParams {
        runtime: runtime.to_string(),
        version: version.to_string(),
    })
    .unwrap();

    let result = client.send_request("runtime.install", Some(params)).await?;
    let task_id = result
        .get("task_id")
        .and_then(|t| t.as_str())
        .ok_or("No task_id in response")?
        .to_string();

    loop {
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;

        let status_params = serde_json::json!({ "task_id": task_id });
        let status = client
            .send_request("runtime.install.status", Some(status_params))
            .await?;

        let stage = status.get("stage").and_then(|s| s.as_str()).unwrap_or("");
        let progress = status.get("progress").and_then(|p| p.as_u64()).unwrap_or(0);
        let message = status.get("message").and_then(|m| m.as_str()).unwrap_or("");

        print!("\r{} {:.0}%", message, progress);
        use std::io::Write;
        std::io::stdout().flush().ok();

        match stage {
            "completed" => {
                println!();
                return Ok(format!("{} {} installed successfully", runtime, version));
            }
            "failed" => {
                println!();
                return Err(message.to_string());
            }
            _ => {}
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

async fn resolve_version(runtime: &str, version: Option<&str>) -> Result<String, String> {
    match version {
        Some(v) => Ok(v.to_string()),
        None => {
            let mut client = connect().await?;
            let result = client.send_request("runtime.list", None).await?;
            let list: RuntimeListResult =
                serde_json::from_value(result).map_err(|e| e.to_string())?;
            list.installed
                .iter()
                .find(|r| r.runtime == runtime)
                .map(|r| r.version.clone())
                .ok_or_else(|| format!("{} is not installed", runtime))
        }
    }
}

async fn cmd_start(
    runtime: &str,
    version: Option<&str>,
    document_root: Option<&str>,
) -> Result<String, String> {
    let version = resolve_version(runtime, version).await?;

    if document_root.is_none() {
        let is_web = matches!(runtime, "apache" | "nginx");
        if is_web {
            eprint!("Document root: ");
            use std::io::Write;
            std::io::stderr().flush().ok();
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| e.to_string())?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                return Err("Document root is required for web servers".to_string());
            }
            return Box::pin(cmd_start(runtime, Some(&version), Some(&trimmed))).await;
        }
    }

    let mut client = connect().await?;
    let mut params = serde_json::json!({
        "runtime": runtime,
        "version": version,
    });
    if let Some(dr) = document_root {
        params["document_root"] = serde_json::Value::String(dr.to_string());
    }
    let result = client.send_request("runtime.start", Some(params)).await?;
    let pid = result.get("pid").and_then(|p| p.as_u64()).unwrap_or(0);
    Ok(format!("{} {} started (pid {})", runtime, version, pid))
}

async fn cmd_stop(runtime: &str) -> Result<String, String> {
    let mut client = connect().await?;
    let params = serde_json::json!({ "runtime": runtime });
    client.send_request("runtime.stop", Some(params)).await?;
    Ok(format!("{} stopped", runtime))
}

async fn cmd_logs(runtime: &str) -> Result<String, String> {
    let mut client = connect().await?;
    let params = serde_json::json!({ "runtime": runtime });
    let result = client.send_request("runtime.logs", Some(params)).await?;
    let entries = result
        .get("entries")
        .and_then(|e| e.as_array())
        .ok_or("No log entries")?;

    if entries.is_empty() {
        return Ok(format!("No logs for {}", runtime));
    }

    let mut output = String::new();
    for entry in entries {
        let ts = entry
            .get("timestamp")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let level = entry.get("level").and_then(|l| l.as_str()).unwrap_or("");
        let msg = entry.get("message").and_then(|m| m.as_str()).unwrap_or("");
        output.push_str(&format!("[{}] {:5} {}\n", ts, level.to_uppercase(), msg));
    }
    Ok(output)
}

async fn cmd_active() -> Result<String, String> {
    let mut client = connect().await?;
    let result = client.send_request("runtime.active", None).await?;
    let active = result.get("active").and_then(|a| a.as_object());

    match active {
        Some(map) if !map.is_empty() => {
            let mut output = String::from("Active runtimes:\n");
            for (runtime, version) in map {
                output.push_str(&format!(
                    "  {} -> {}\n",
                    runtime,
                    version.as_str().unwrap_or("?")
                ));
            }
            Ok(output)
        }
        _ => Ok("No active runtimes.".to_string()),
    }
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
