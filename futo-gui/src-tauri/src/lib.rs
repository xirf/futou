use futou_ipc::messages::RpcRequest;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ClientOptions;

fn data_dir() -> std::path::PathBuf {
    let base = std::env::var("APPDATA")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(base).join(".futou")
}

fn settings_path() -> std::path::PathBuf {
    data_dir().join("settings.json")
}

fn read_settings() -> serde_json::Value {
    let path = settings_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str(&content) {
                return v;
            }
        }
    }
    serde_json::json!({
        "install_dir": data_dir().to_string_lossy(),
    })
}

fn write_settings(v: &serde_json::Value) -> Result<(), String> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(
        settings_path(),
        serde_json::to_string_pretty(v).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn configured_install_dir() -> std::path::PathBuf {
    read_settings()["install_dir"]
        .as_str()
        .filter(|path| !path.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(data_dir)
}

fn start_daemon() -> Result<(), String> {
    let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    else {
        return Err("Cannot resolve GUI executable directory".to_string());
    };

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
            cmd.arg("--data-dir").arg(configured_install_dir());
            cmd.spawn()
                .map_err(|e| format!("Failed to start daemon: {}", e))?;
            return Ok(());
        }
    }
    Err("Daemon executable not found beside GUI".to_string())
}

#[tauri::command]
async fn daemon_status() -> Result<String, String> {
    send_rpc("daemon.status", None).await
}

#[tauri::command]
fn daemon_start() -> Result<(), String> {
    start_daemon()
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
async fn runtime_start(
    runtime: String,
    version: String,
    document_root: Option<String>,
) -> Result<String, String> {
    let mut params = serde_json::json!({ "runtime": runtime, "version": version });
    if let Some(dr) = document_root {
        params["document_root"] = serde_json::Value::String(dr);
    }
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
        "apache" => return check_config(&dir.join("data").join("conf"), "httpd.conf"),
        "nginx" => return check_config(&dir.join("data").join("conf"), "nginx.conf"),
        _ => return Ok(None),
    };
    for c in candidates {
        let p = dir.join(c);
        if p.exists() {
            return Ok(Some(p.to_string_lossy().to_string()));
        }
    }
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

fn check_config(dir: &std::path::Path, name: &str) -> Result<Option<String>, String> {
    let p = dir.join(name);
    if p.exists() {
        Ok(Some(p.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
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

// ── Settings commands ──

#[tauri::command]
fn get_config() -> Result<String, String> {
    Ok(read_settings().to_string())
}

#[tauri::command]
fn set_install_location(path: String) -> Result<(), String> {
    let mut settings = read_settings();
    settings["install_dir"] = serde_json::Value::String(path);
    write_settings(&settings)
}

#[tauri::command]
fn get_autostart() -> Result<bool, String> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "FutouDaemon",
        ])
        .output()
        .map_err(|e| format!("Failed to query registry: {}", e))?;
    Ok(output.status.success())
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe_dir = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .map(|d| d.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let daemon = exe_dir.join("futou-daemon.exe");
        let target = if daemon.exists() {
            daemon
        } else {
            exe_dir.join("futou-daemon-x86_64-pc-windows-msvc.exe")
        };
        let value = format!(
            "\"{}\" --data-dir \"{}\"",
            target.to_string_lossy(),
            configured_install_dir().to_string_lossy()
        );

        std::process::Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "FutouDaemon",
                "/t",
                "REG_SZ",
                "/d",
                &value,
                "/f",
            ])
            .output()
            .map_err(|e| format!("Failed to set autostart: {}", e))?;
    } else {
        std::process::Command::new("reg")
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "FutouDaemon",
                "/f",
            ])
            .output()
            .map_err(|e| format!("Failed to remove autostart: {}", e))?;
    }
    Ok(())
}

// ── RPC helper ──

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

    client
        .write_all(json.as_bytes())
        .await
        .map_err(|e| format!("Write error: {}", e))?;

    let (reader, _writer) = tokio::io::split(client);
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    tokio::time::timeout(std::time::Duration::from_secs(30), buf_reader.read_line(&mut line))
        .await
        .map_err(|_| "Request timed out (30s)".to_string())?
        .map_err(|e| format!("Read error: {}", e))?;

    Ok(line.trim().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let _ = app.handle().plugin(tauri_plugin_dialog::init());
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
            get_config,
            set_install_location,
            get_autostart,
            set_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
