use crate::domain::runtime::{DaemonState, Installation, RuntimeName, Version};

/// Persists and transactionally updates daemon runtime state.
#[async_trait::async_trait]
pub trait RuntimeRepository: Send + Sync {
    /// Loads the latest committed daemon state.
    async fn load(&self) -> Result<DaemonState, RepositoryError>;

    /// Applies one serialized mutation and returns the committed state.
    async fn update_state(
        &self,
        update: Box<
            dyn for<'state> FnOnce(&'state mut DaemonState) -> Result<(), RepositoryError> + Send,
        >,
    ) -> Result<DaemonState, RepositoryError>;

    /// Adds an installation without overwriting concurrent state changes.
    async fn add_installation(&self, installation: Installation) -> Result<(), RepositoryError>;

    /// Updates an installation status without overwriting concurrent state changes.
    async fn update_status(
        &self,
        runtime: &RuntimeName,
        version: &Version,
        status: &str,
    ) -> Result<(), RepositoryError>;

    /// Removes one installation or returns [`RepositoryError::NotFound`].
    async fn remove_installation(
        &self,
        runtime: &RuntimeName,
        version: &Version,
    ) -> Result<(), RepositoryError>;
}

/// Describes a state loading, serialization, or persistence failure.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found: {runtime} {version}")]
    NotFound { runtime: String, version: String },
    #[error("Already exists: {runtime} {version}")]
    AlreadyExists { runtime: String, version: String },
}
