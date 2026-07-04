use futou_ipc::messages::RpcRequest;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ClientOptions;

fn start_daemon() {
    let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    else { return };

    let names = [
        "futou-daemon.exe",
        "futou-daemon-x86_64-pc-windows-msvc.exe",
    ];
    for name in &names {
        let path = dir.join(name);
        if path.exists() {
            let mut cmd = std::process::Command::new(&path);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000);
            }
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
            let _ = cmd.spawn();
            return;
        }
    }
}

#[tauri::command]
async fn daemon_status() -> Result<String, String> {
    send_rpc("daemon.status", None).await
}

#[tauri::command]
fn daemon_start() -> Result<(), String> {
    start_daemon();
    Ok(())
}

#[tauri::command]
async fn daemon_shutdown() -> Result<String, String> {
    send_rpc("daemon.shutdown", None).await
}

#[tauri::command]
async fn runtime_list() -> Result<String, String> {
    send_rpc("runtime.list", None).await
}

#[tauri::command]
async fn runtime_install(runtime: String, version: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime, "version": version });
    send_rpc("runtime.install", Some(params)).await
}

#[tauri::command]
async fn runtime_uninstall(runtime: String, version: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime, "version": version });
    send_rpc("runtime.uninstall", Some(params)).await
}

#[tauri::command]
async fn runtime_activate(runtime: String, version: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime, "version": version });
    send_rpc("runtime.activate", Some(params)).await
}

#[tauri::command]
async fn runtime_deactivate(runtime: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime });
    send_rpc("runtime.deactivate", Some(params)).await
}

#[tauri::command]
async fn runtime_start(runtime: String, version: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime, "version": version });
    send_rpc("runtime.start", Some(params)).await
}

#[tauri::command]
async fn runtime_stop(runtime: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime });
    send_rpc("runtime.stop", Some(params)).await
}

#[tauri::command]
async fn runtime_logs(runtime: String) -> Result<String, String> {
    let params = serde_json::json!({ "runtime": runtime });
    send_rpc("runtime.logs", Some(params)).await
}

#[tauri::command]
async fn open_dir(path: String) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn()
        .map_err(|e| format!("Failed to open: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn find_config(runtime: String, version_dir: String) -> Result<Option<String>, String> {
    let dir = std::path::Path::new(&version_dir);
    let candidates: &[&str] = match runtime.as_str() {
        "php" => &["php.ini", "php.ini-development"],
        "mariadb" | "mysql" => &["my.ini", "my.cnf"],
        "postgresql" => &["postgresql.conf"],
        _ => return Ok(None),
    };
    for c in candidates {
        let p = dir.join(c);
        if p.exists() {
            return Ok(Some(p.to_string_lossy().to_string()));
        }
    }
    // For postgres, also check data dir
    if runtime == "postgresql" {
        for c in candidates {
            let p = dir.join("data").join(c);
            if p.exists() {
                return Ok(Some(p.to_string_lossy().to_string()));
            }
        }
    }
    Ok(None)
}

#[tauri::command]
async fn catalogue_list() -> Result<String, String> {
    send_rpc("catalogue.list", None).await
}

#[tauri::command]
async fn runtime_install_status(task_id: String) -> Result<String, String> {
    let params = serde_json::json!({ "task_id": task_id });
    send_rpc("runtime.install.status", Some(params)).await
}

async fn send_rpc(method: &str, params: Option<serde_json::Value>) -> Result<String, String> {
    let path = r"\\.\pipe\futou";
    let mut client = ClientOptions::new()
        .open(path)
        .map_err(|e| format!("Cannot connect to daemon: {}", e))?;

    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: method.to_string(),
        params,
    };

    let mut json = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    json.push('\n');

    client.write_all(json.as_bytes()).await
        .map_err(|e| format!("Write error: {}", e))?;

    let (reader, _writer) = tokio::io::split(client);
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    buf_reader.read_line(&mut line).await
        .map_err(|e| format!("Read error: {}", e))?;

    Ok(line.trim().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            start_daemon();
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            daemon_status,
            daemon_start,
            daemon_shutdown,
            runtime_list,
            runtime_install,
            runtime_install_status,
            runtime_uninstall,
            runtime_activate,
            runtime_deactivate,
            runtime_start,
            runtime_stop,
            runtime_logs,
            open_dir,
            open_file,
            find_config,
            catalogue_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
