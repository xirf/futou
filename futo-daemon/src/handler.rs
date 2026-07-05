use std::sync::Arc;

use futou_core::domain::runtime::{RuntimeName, Version};
use futou_ipc::error_codes;
use futou_ipc::messages::{
    ActivateParams, ActivateResult, ActiveResult, CatalogueListResult, CatalogueRefreshResult,
    DaemonStatusResult, DeactivateParams, DeactivateResult, InstallParams, InstallProgress,
    InstallStartedResult, InstallStatusParams, InstalledRuntime, LogEntry, LogsParams, LogsResult,
    RpcRequest, RpcResponse, RuntimeListResult, ShutdownResult, StartServerParams,
    StartServerResult, StopServerParams, StopServerResult, UninstallParams, UninstallResult,
};
use tracing::{error, info};

use crate::AppContext;

pub async fn handle_request(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let method = request.method.as_str();
    let id = request.id;

    info!("RPC call: {} (id={})", method, id);

    let result = match method {
        "catalogue.list" => handle_catalogue_list(ctx, id).await,
        "catalogue.refresh" => handle_catalogue_refresh(ctx, id).await,
        "runtime.install" => handle_runtime_install(ctx, id, request).await,
        "runtime.install.status" => handle_runtime_install_status(ctx, id, request).await,
        "runtime.uninstall" => handle_runtime_uninstall(ctx, id, request).await,
        "runtime.list" => handle_runtime_list(ctx, id).await,
        "runtime.activate" => handle_runtime_activate(ctx, id, request).await,
        "runtime.deactivate" => handle_runtime_deactivate(ctx, id, request).await,
        "runtime.start" => handle_runtime_start(ctx, id, request).await,
        "runtime.stop" => handle_runtime_stop(ctx, id, request).await,
        "runtime.logs" => handle_runtime_logs(ctx, id, request).await,
        "runtime.active" => handle_runtime_active(ctx, id).await,
        "daemon.status" => handle_daemon_status(ctx, id).await,
        "daemon.shutdown" => handle_daemon_shutdown(ctx, id).await,
        _ => Err(RpcResponse::error(
            id,
            error_codes::METHOD_NOT_FOUND,
            format!("Unknown method: {}", method),
        )),
    };

    match &result {
        Ok(_res) => info!("RPC success: {} (id={})", method, id),
        Err(err) => error!("RPC error: {} (id={}): {:?}", method, id, err),
    }

    result
}

async fn handle_catalogue_list(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
) -> Result<RpcResponse, RpcResponse> {
    let lock = ctx.read().await;
    match lock.catalogue_service.list_runtimes().await {
        Ok(runtimes) => Ok(RpcResponse::success(
            id,
            serde_json::to_value(CatalogueListResult { runtimes }).unwrap(),
        )),
        Err(e) => Err(RpcResponse::error(
            id,
            error_codes::CATALOGUE_UNAVAILABLE,
            e.to_string(),
        )),
    }
}

async fn handle_catalogue_refresh(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
) -> Result<RpcResponse, RpcResponse> {
    let lock = ctx.read().await;
    match lock.catalogue_service.refresh().await {
        Ok(updated) => Ok(RpcResponse::success(
            id,
            serde_json::to_value(CatalogueRefreshResult { updated }).unwrap(),
        )),
        Err(e) => Err(RpcResponse::error(
            id,
            error_codes::CATALOGUE_UNAVAILABLE,
            e.to_string(),
        )),
    }
}

async fn handle_runtime_install(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: InstallParams = serde_json::from_value(request.params.clone().unwrap_or_default())
        .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    let task_id = format!("{}-{}", params.runtime, params.version);

    let (install_service, progress_map) = {
        let lock = ctx.read().await;
        (lock.install_service.clone(), lock.download_progress.clone())
    };

    {
        let mut map = progress_map.write().unwrap();
        map.insert(
            task_id.clone(),
            InstallProgress {
                task_id: task_id.clone(),
                stage: "starting".to_string(),
                progress: 0,
                message: "Starting install...".to_string(),
            },
        );
    }

    let runtime = RuntimeName(params.runtime);
    let version = Version(params.version);
    let pm = progress_map.clone();
    let tid = task_id.clone();

    tokio::spawn(async move {
        let tid_outer = tid.clone();
        let result = install_service
            .install(&runtime, &version, move |pct, msg| {
                let stage = if pct < 0.05 {
                    "looking_up"
                } else if pct < 0.70 {
                    "downloading"
                } else if pct < 0.80 {
                    "verifying"
                } else if pct < 0.95 {
                    "extracting"
                } else {
                    "registering"
                };
                let mut map = pm.write().unwrap();
                map.insert(
                    tid_outer.clone(),
                    InstallProgress {
                        task_id: tid_outer.clone(),
                        stage: stage.to_string(),
                        progress: (pct * 100.0) as u64,
                        message: msg,
                    },
                );
            })
            .await;

        let mut map = progress_map.write().unwrap();
        match result {
            Ok(inst) => {
                map.insert(
                    tid.clone(),
                    InstallProgress {
                        task_id: tid.clone(),
                        stage: "completed".to_string(),
                        progress: 100,
                        message: format!("Installed {} {}", inst.runtime, inst.version),
                    },
                );
            }
            Err(e) => {
                map.insert(
                    tid.clone(),
                    InstallProgress {
                        task_id: tid.clone(),
                        stage: "failed".to_string(),
                        progress: 0,
                        message: e.to_string(),
                    },
                );
            }
        }
    });

    Ok(RpcResponse::success(
        id,
        serde_json::to_value(InstallStartedResult { task_id }).unwrap(),
    ))
}

async fn handle_runtime_install_status(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: InstallStatusParams =
        serde_json::from_value(request.params.clone().unwrap_or_default())
            .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    let progress_map = {
        let lock = ctx.read().await;
        lock.download_progress.clone()
    };

    let progress = {
        let map = progress_map.read().unwrap();
        map.get(&params.task_id).cloned()
    };

    match progress {
        Some(p) => Ok(RpcResponse::success(id, serde_json::to_value(p).unwrap())),
        None => Err(RpcResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            format!("No such task: {}", params.task_id),
        )),
    }
}

async fn handle_runtime_uninstall(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: UninstallParams =
        serde_json::from_value(request.params.clone().unwrap_or_default())
            .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    let lock = ctx.read().await;
    let runtime = RuntimeName(params.runtime);
    let version = Version(params.version);

    match lock.install_service.uninstall(&runtime, &version).await {
        Ok(_) => {
            let result = UninstallResult {
                runtime: runtime.to_string(),
                version: version.to_string(),
                status: "uninstalled".to_string(),
            };
            Ok(RpcResponse::success(
                id,
                serde_json::to_value(result).unwrap(),
            ))
        }
        Err(e) => Err(RpcResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}

async fn handle_runtime_list(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
) -> Result<RpcResponse, RpcResponse> {
    let lock = ctx.read().await;
    match lock.env_service.status().await {
        Ok(status) => {
            let installed: Vec<InstalledRuntime> = status
                .installed
                .into_iter()
                .map(|e| InstalledRuntime {
                    runtime: e.runtime,
                    version: e.version,
                    status: e.status.to_lowercase(),
                    path: e.path,
                    version_dir: Some(e.version_dir),
                    process: e.process,
                })
                .collect();
            info!(
                "runtime.list returns {} installed runtimes",
                installed.len()
            );
            Ok(RpcResponse::success(
                id,
                serde_json::to_value(RuntimeListResult { installed }).unwrap(),
            ))
        }
        Err(e) => {
            error!("runtime.list failed: {}", e);
            Err(RpcResponse::error(
                id,
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            ))
        }
    }
}

async fn handle_runtime_activate(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: ActivateParams = serde_json::from_value(request.params.clone().unwrap_or_default())
        .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    info!("runtime.activate: {} {}", params.runtime, params.version);

    let lock = ctx.read().await;
    let runtime_name = params.runtime.clone();
    let version_str = params.version.clone();
    let runtime = RuntimeName(params.runtime);
    let version = Version(params.version);

    match lock.activation_service.activate(&runtime, &version).await {
        Ok(_) => {
            info!("runtime.activate success: {} {}", runtime, version);
            lock.operation_log.push(
                &runtime_name,
                "info",
                format!("{} {} added to PATH", runtime_name, version_str),
            );
            let result = ActivateResult {
                runtime: runtime.to_string(),
                version: version.to_string(),
                status: "active".to_string(),
            };
            Ok(RpcResponse::success(
                id,
                serde_json::to_value(result).unwrap(),
            ))
        }
        Err(e) => {
            error!("runtime.activate failed: {} {}: {}", runtime, version, e);
            lock.operation_log.push(
                &runtime_name,
                "error",
                format!("Failed to activate {} {}: {}", runtime_name, version_str, e),
            );
            Err(RpcResponse::error(
                id,
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            ))
        }
    }
}

async fn handle_runtime_deactivate(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: DeactivateParams =
        serde_json::from_value(request.params.clone().unwrap_or_default())
            .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    info!("runtime.deactivate: {}", params.runtime);

    let lock = ctx.read().await;
    let runtime_name = params.runtime.clone();
    let runtime = RuntimeName(params.runtime);

    match lock.activation_service.deactivate(&runtime).await {
        Ok(_) => {
            info!("runtime.deactivate success: {}", runtime);
            lock.operation_log.push(
                &runtime_name,
                "info",
                format!("{} removed from PATH", runtime_name),
            );
            let result = DeactivateResult {
                runtime: runtime.to_string(),
                status: "deactivated".to_string(),
            };
            Ok(RpcResponse::success(
                id,
                serde_json::to_value(result).unwrap(),
            ))
        }
        Err(e) => {
            error!("runtime.deactivate failed: {}: {}", runtime, e);
            lock.operation_log.push(
                &runtime_name,
                "error",
                format!("Failed to deactivate {}: {}", runtime_name, e),
            );
            Err(RpcResponse::error(
                id,
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            ))
        }
    }
}

async fn handle_runtime_start(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: StartServerParams =
        serde_json::from_value(request.params.clone().unwrap_or_default())
            .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    info!("runtime.start: {} {}", params.runtime, params.version);

    let lock = ctx.read().await;
    let runtime_name = params.runtime.clone();
    let version_str = params.version.clone();
    let runtime = RuntimeName(params.runtime);
    let version = Version(params.version);

    match lock
        .activation_service
        .start_process(&runtime, &version)
        .await
    {
        Ok(pid) => {
            info!("runtime.start success: {} {} pid={}", runtime, version, pid);
            lock.operation_log.push(
                &runtime_name,
                "info",
                format!(
                    "{} {} server started (pid {})",
                    runtime_name, version_str, pid
                ),
            );
            Ok(RpcResponse::success(
                id,
                serde_json::to_value(StartServerResult {
                    runtime: runtime.to_string(),
                    version: version.to_string(),
                    pid,
                })
                .unwrap(),
            ))
        }
        Err(e) => {
            error!("runtime.start failed: {} {}: {}", runtime, version, e);
            lock.operation_log.push(
                &runtime_name,
                "error",
                format!("Failed to start {} {}: {}", runtime_name, version_str, e),
            );
            Err(RpcResponse::error(
                id,
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            ))
        }
    }
}

async fn handle_runtime_stop(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: StopServerParams =
        serde_json::from_value(request.params.clone().unwrap_or_default())
            .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    info!("runtime.stop: {}", params.runtime);

    let lock = ctx.read().await;
    let runtime_name = params.runtime.clone();
    let runtime = RuntimeName(params.runtime);

    match lock.activation_service.stop_process(&runtime).await {
        Ok(()) => {
            info!("runtime.stop success: {}", runtime);
            lock.operation_log.push(
                &runtime_name,
                "info",
                format!("{} server stopped", runtime_name),
            );
            Ok(RpcResponse::success(
                id,
                serde_json::to_value(StopServerResult {
                    runtime: runtime.to_string(),
                    status: "stopped".to_string(),
                })
                .unwrap(),
            ))
        }
        Err(e) => {
            error!("runtime.stop failed: {}: {}", runtime, e);
            lock.operation_log.push(
                &runtime_name,
                "error",
                format!("Failed to stop {}: {}", runtime_name, e),
            );
            Err(RpcResponse::error(
                id,
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            ))
        }
    }
}

async fn handle_runtime_logs(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
    request: &RpcRequest,
) -> Result<RpcResponse, RpcResponse> {
    let params: LogsParams = serde_json::from_value(request.params.clone().unwrap_or_default())
        .map_err(|e| RpcResponse::error(id, error_codes::INVALID_PARAMS, e.to_string()))?;

    let lock = ctx.read().await;
    let entries: Vec<LogEntry> = lock
        .operation_log
        .for_runtime(&params.runtime)
        .into_iter()
        .map(|e| LogEntry {
            timestamp: e.timestamp,
            runtime: e.runtime,
            level: e.level,
            message: e.message,
        })
        .collect();

    Ok(RpcResponse::success(
        id,
        serde_json::to_value(LogsResult { entries }).unwrap(),
    ))
}

async fn handle_runtime_active(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
) -> Result<RpcResponse, RpcResponse> {
    let lock = ctx.read().await;
    match lock.env_service.status().await {
        Ok(status) => Ok(RpcResponse::success(
            id,
            serde_json::to_value(ActiveResult {
                active: status.active,
            })
            .unwrap(),
        )),
        Err(e) => Err(RpcResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}

async fn handle_daemon_status(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
) -> Result<RpcResponse, RpcResponse> {
    let lock = ctx.read().await;
    let active_tasks = {
        let map = lock.download_progress.read().unwrap();
        map.values()
            .filter(|p| p.stage != "completed" && p.stage != "failed")
            .count() as u32
    };
    let result = DaemonStatusResult {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: lock.started_at.elapsed().as_secs(),
        aria2_running: lock.aria2_available,
        active_tasks,
    };
    Ok(RpcResponse::success(
        id,
        serde_json::to_value(result).unwrap(),
    ))
}

async fn handle_daemon_shutdown(
    ctx: &Arc<tokio::sync::RwLock<AppContext>>,
    id: u64,
) -> Result<RpcResponse, RpcResponse> {
    let lock = ctx.read().await;
    let _ = lock.shutdown_tx.send(());
    let result = ShutdownResult {
        status: "shutting_down".to_string(),
    };
    Ok(RpcResponse::success(
        id,
        serde_json::to_value(result).unwrap(),
    ))
}
