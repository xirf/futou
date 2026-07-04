use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcMessage {
    Request(RpcRequest),
    Response(RpcResponse),
    Notification(RpcNotification),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressParams {
    pub task_id: String,
    pub stage: String,
    pub progress: f64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallParams {
    pub runtime: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallResult {
    pub runtime: String,
    pub version: String,
    pub status: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStartedResult {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallStatusParams {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub task_id: String,
    pub stage: String,
    pub progress: u64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UninstallParams {
    pub runtime: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UninstallResult {
    pub runtime: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateParams {
    pub runtime: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateResult {
    pub runtime: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeactivateParams {
    pub runtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeactivateResult {
    pub runtime: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartServerParams {
    pub runtime: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartServerResult {
    pub runtime: String,
    pub version: String,
    pub pid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopServerParams {
    pub runtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopServerResult {
    pub runtime: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsParams {
    pub runtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub runtime: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsResult {
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeListResult {
    pub installed: Vec<InstalledRuntime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledRuntime {
    pub runtime: String,
    pub version: String,
    pub status: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveResult {
    pub active: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatusResult {
    pub version: String,
    pub uptime_secs: u64,
    pub aria2_running: bool,
    pub active_tasks: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShutdownResult {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogueListResult {
    pub runtimes: Vec<CatalogueRuntime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogueRuntime {
    pub name: String,
    pub display_name: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogueRefreshResult {
    pub updated: bool,
}

impl RpcResponse {
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: u64, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn error_with_data(id: u64, code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: Some(data),
            }),
        }
    }
}

impl RpcNotification {
    pub fn progress(task_id: impl Into<String>, stage: impl Into<String>, progress: f64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "progress".to_string(),
            params: Some(serde_json::json!(ProgressParams {
                task_id: task_id.into(),
                stage: stage.into(),
                progress,
                message: message.into(),
            })),
        }
    }
}
