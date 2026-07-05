#[async_trait::async_trait]
pub trait ShimManager: Send + Sync {
    async fn create_shims(
        &self,
        runtime: &str,
        version: &str,
        bin_dir: &std::path::Path,
    ) -> Result<(), ShimError>;
    async fn remove_shims(&self, runtime: &str) -> Result<(), ShimError>;
    async fn shim_dir(&self) -> Result<std::path::PathBuf, ShimError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ShimError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Symlink creation failed: {0}")]
    SymlinkFailed(String),
}
