use futou_ipc::messages::{RpcNotification, RpcRequest};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::sync::broadcast;

pub struct PipeClient {
    writer: tokio::io::WriteHalf<tokio::net::windows::named_pipe::NamedPipeClient>,
    reader: BufReader<tokio::io::ReadHalf<tokio::net::windows::named_pipe::NamedPipeClient>>,
    next_id: u64,
    notification_tx: broadcast::Sender<RpcNotification>,
}

impl PipeClient {
    pub async fn connect(pipe_name: &str) -> Result<Self, String> {
        let path = format!(r"\\.\pipe\{}", pipe_name);
        let client = ClientOptions::new()
            .open(&path)
            .map_err(|e| format!("Failed to connect to daemon: {}. Is the daemon running?", e))?;

        let (reader, writer) = tokio::io::split(client);
        let (notification_tx, _) = broadcast::channel(256);

        Ok(Self {
            writer,
            reader: BufReader::new(reader),
            next_id: 1,
            notification_tx,
        })
    }

    pub fn notification_receiver(&self) -> broadcast::Receiver<RpcNotification> {
        self.notification_tx.subscribe()
    }

    pub async fn send_request(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let mut json = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        json.push('\n');

        self.writer
            .write_all(json.as_bytes())
            .await
            .map_err(|e| format!("Write error: {}", e))?;

        let mut line = String::new();
        loop {
            line.clear();
            let n = self
                .reader
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Read error: {}", e))?;

            if n == 0 {
                return Err("Connection closed by daemon".to_string());
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let msg: serde_json::Value =
                serde_json::from_str(trimmed).map_err(|e| format!("Parse error: {}", e))?;

            if msg.get("method").and_then(|m| m.as_str()) == Some("progress") {
                if let Ok(notification) = serde_json::from_value::<RpcNotification>(msg) {
                    let _ = self.notification_tx.send(notification);
                }
                continue;
            }

            if msg.get("id").and_then(|i| i.as_u64()) == Some(id) {
                if let Some(error) = msg.get("error") {
                    let msg = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(msg.to_string());
                }
                return Ok(msg
                    .get("result")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null));
            }
        }
    }
}
