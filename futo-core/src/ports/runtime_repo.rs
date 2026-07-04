use crate::domain::runtime::{DaemonState, RuntimeName, Version, Installation};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait RuntimeRepository: Send + Sync {
    async fn load(&self) -> Result<DaemonState, RepositoryError>;
    async fn save(&self, state: &DaemonState) -> Result<(), RepositoryError>;
    async fn add_installation(&self, installation: Installation) -> Result<(), RepositoryError>;
    async fn update_status(&self, runtime: &RuntimeName, version: &Version, status: &str) -> Result<(), RepositoryError>;
    async fn remove_installation(&self, runtime: &RuntimeName, version: &Version) -> Result<(), RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found: {runtime} {version}")]
    NotFound { runtime: String, version: String },
}

pub type SharedRepository = Arc<dyn RuntimeRepository>;
