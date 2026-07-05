use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use futou_core::ports::downloader::{DownloadError, Downloader};
use tokio::sync::RwLock;
use tracing::warn;

pub struct Aria2Downloader {
    rpc_url: String,
    rpc_secret: String,
    client: reqwest::Client,
    aria2_process: Arc<RwLock<Option<tokio::process::Child>>>,
    download_dir: PathBuf,
    pid_path: PathBuf,
}

impl Aria2Downloader {
    pub async fn spawn(
        aria2c_path: &Path,
        download_dir: &Path,
        pid_path: &Path,
    ) -> Result<Self, DownloadError> {
        let secret = uuid::Uuid::new_v4().to_string();
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0")
                .map_err(|e| DownloadError::Io(e.to_string()))?;
            let port = listener
                .local_addr()
                .map_err(|e| DownloadError::Io(e.to_string()))?
                .port();
            drop(listener);
            port
        };

        let child = tokio::process::Command::new(aria2c_path)
            .args([
                "--enable-rpc",
                "--rpc-listen-all=false",
                &format!("--rpc-listen-port={}", port),
                "--rpc-secret",
                &secret,
                "--dir",
                download_dir.to_str().unwrap_or("."),
                "--console-log-level=error",
                "--summary-interval=0",
                "--continue=true",
                "--max-connection-per-server=4",
                "--split=4",
                "--min-split-size=1M",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| DownloadError::Io(e.to_string()))?;

        let pid = child
            .id()
            .ok_or(DownloadError::Io("No PID assigned".to_string()))?;
        if let Some(parent) = pid_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(pid_path, pid.to_string());

        let rpc_url = format!("http://127.0.0.1:{}/jsonrpc", port);

        let downloader = Self {
            rpc_url,
            rpc_secret: secret,
            client: reqwest::Client::new(),
            aria2_process: Arc::new(RwLock::new(Some(child))),
            download_dir: download_dir.to_path_buf(),
            pid_path: pid_path.to_path_buf(),
        };

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(downloader)
    }

    async fn rpc_call(
        &self,
        method: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, DownloadError> {
        let mut all_params = vec![serde_json::Value::String(format!(
            "token:{}",
            self.rpc_secret
        ))];
        all_params.extend(params);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "futou",
            "method": method,
            "params": all_params,
        });

        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DownloadError::Http(e.to_string()))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| DownloadError::Http(e.to_string()))?;

        if json.get("error").is_some() {
            warn!("aria2 RPC error for {}: {:?}", method, json);
        }

        Ok(json)
    }
}

#[async_trait::async_trait]
impl Downloader for Aria2Downloader {
    async fn shutdown(&self) {
        let _ = self.rpc_call("aria2.shutdown", vec![]).await;
        if let Some(mut child) = self.aria2_process.write().await.take() {
            let _ = child.wait().await;
        }
        let _ = std::fs::remove_file(&self.pid_path);
    }

    async fn download(
        &self,
        url: &str,
        dest: &Path,
        progress: Box<dyn Fn(f64, String) + Send + Sync>,
    ) -> Result<(), DownloadError> {
        let filename = dest
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download.tmp");

        let result = self
            .rpc_call(
                "aria2.addUri",
                vec![
                    serde_json::json!([url]),
                    serde_json::json!({
                        "out": filename,
                        "allow-overwrite": true,
                        "auto-file-renaming": false,
                    }),
                ],
            )
            .await?;

        let gid = result
            .get("result")
            .and_then(|r| r.as_str())
            .ok_or_else(|| DownloadError::Http("No GID returned".to_string()))?
            .to_string();

        loop {
            let status = self
                .rpc_call("aria2.tellStatus", vec![serde_json::json!(gid)])
                .await?;

            let status_str = status["result"]["status"].as_str().unwrap_or("unknown");
            let completed: f64 = status["result"]["completedLength"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            let total: f64 = status["result"]["totalLength"]
                .as_str()
                .unwrap_or("1")
                .parse()
                .unwrap_or(1.0);

            match status_str {
                "complete" => {
                    progress(1.0, "Download complete".to_string());
                    let aria2_path = self.download_dir.join(filename);
                    if aria2_path.as_path() != dest {
                        tokio::fs::rename(&aria2_path, dest)
                            .await
                            .map_err(|e| DownloadError::Io(e.to_string()))?;
                    }
                    return Ok(());
                }
                "error" => {
                    let msg = status["result"]["errorMessage"]
                        .as_str()
                        .unwrap_or("unknown");
                    return Err(DownloadError::Http(format!("Download failed: {}", msg)));
                }
                "removed" => {
                    return Err(DownloadError::Cancelled);
                }
                _ => {
                    let pct = if total > 0.0 { completed / total } else { 0.0 };
                    progress(pct, format!("{:.1}%", pct * 100.0));
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
}
