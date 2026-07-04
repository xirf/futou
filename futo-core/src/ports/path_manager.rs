#[async_trait::async_trait]
pub trait PathManager: Send + Sync {
    async fn add_to_path(&self, dir: &std::path::Path) -> Result<(), PathError>;
    async fn remove_from_path(&self, dir: &std::path::Path) -> Result<(), PathError>;
    async fn is_in_path(&self, dir: &std::path::Path) -> Result<bool, PathError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("Registry error: {0}")]
    Registry(String),
    #[error("IO error: {0}")]
    Io(String),
}
