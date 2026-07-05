use std::path::Path;

#[async_trait::async_trait]
pub trait ProcessManager: Send + Sync {
    async fn start_server(
        &self,
        runtime: &str,
        bin_dir: &Path,
        data_dir: &Path,
        document_root: Option<&Path>,
    ) -> Result<u32, ProcessError>;
    /// Kill a process by PID.
    async fn stop_server(&self, pid: u32) -> Result<(), ProcessError>;
    /// Run one-time data directory initialization.
    async fn init_data_dir(
        &self,
        runtime: &str,
        bin_dir: &Path,
        data_dir: &Path,
    ) -> Result<(), ProcessError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("not a server runtime: {0}")]
    NotServer(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("process exited with error: {0}")]
    ExitError(String),
    #[error("timeout")]
    Timeout,
}
