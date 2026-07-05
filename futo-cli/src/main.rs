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
