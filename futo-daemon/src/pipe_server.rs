use std::sync::Arc;

use futou_ipc::messages::{RpcRequest, RpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tracing::{error, info};

use crate::handler::handle_request;
use crate::AppContext;

pub async fn run_pipe_server(ctx: Arc<tokio::sync::RwLock<AppContext>>, pipe_name: &str) {
    let mut shutdown_rx = {
        let lock = ctx.read().await;
        lock.shutdown_tx.subscribe()
    };

    let path = format!(r"\\.\pipe\{}", pipe_name);
    info!("Waiting for client connections on {}", path);

    // Create the first instance (establishes the pipe name)
    let server = create_pipe_instance(&path, true).await;
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        serve_client(server, ctx_clone).await;
    });

    // Create additional instances for concurrent clients
    for _ in 0..3 {
        if let Ok(server) = ServerOptions::new()
            .first_pipe_instance(false)
            .create(&path)
        {
            let ctx_clone = ctx.clone();
            tokio::spawn(async move {
                serve_client(server, ctx_clone).await;
            });
        } else {
            // Stagger creation so the first instance has time to register
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            if let Ok(server) = ServerOptions::new()
                .first_pipe_instance(false)
                .create(&path)
            {
                let ctx_clone = ctx.clone();
                tokio::spawn(async move {
                    serve_client(server, ctx_clone).await;
                });
            }
        }
    }

    // Replenish instances as clients disconnect
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Pipe server shutting down");
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
        }

        let _ = ServerOptions::new()
            .first_pipe_instance(false)
            .create(&path)
            .map(|server| {
                let ctx_clone = ctx.clone();
                tokio::spawn(async move {
                    serve_client(server, ctx_clone).await;
                });
            });
    }
}

async fn create_pipe_instance(path: &str, first: bool) -> NamedPipeServer {
    loop {
        match ServerOptions::new().first_pipe_instance(first).create(path) {
            Ok(server) => return server,
            Err(e) => {
                error!(
                    "Failed to create pipe instance (first={}): {}. Retrying in 1s...",
                    first, e
                );
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn serve_client(server: NamedPipeServer, ctx: Arc<tokio::sync::RwLock<AppContext>>) {
    if let Err(e) = server.connect().await {
        error!("Client connection failed: {}", e);
        return;
    }

    info!("Client connected");
    handle_requests(server, ctx).await;
}

async fn handle_requests(server: NamedPipeServer, ctx: Arc<tokio::sync::RwLock<AppContext>>) {
    let (reader, mut writer) = tokio::io::split(server);
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                info!("Client disconnected");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let request: RpcRequest = match serde_json::from_str(trimmed) {
                    Ok(req) => req,
                    Err(e) => {
                        let error_resp =
                            RpcResponse::error(0, -32700, format!("Parse error: {}", e));
                        let json = serde_json::to_string(&error_resp).unwrap_or_default();
                        let _ = writer.write_all(json.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                        continue;
                    }
                };

                match handle_request(&ctx, &request).await {
                    Ok(response) => {
                        let json = serde_json::to_string(&response).unwrap_or_default();
                        let _ = writer.write_all(json.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                    }
                    Err(response) => {
                        let json = serde_json::to_string(&response).unwrap_or_default();
                        let _ = writer.write_all(json.as_bytes()).await;
                        let _ = writer.write_all(b"\n").await;
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }
}
