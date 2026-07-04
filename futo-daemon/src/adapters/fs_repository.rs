use std::path::PathBuf;
use std::sync::Arc;

use futou_core::domain::runtime::{DaemonState, Installation, RuntimeName, Version};
use futou_core::ports::runtime_repo::{RepositoryError, RuntimeRepository};
use tokio::sync::RwLock;

pub struct FsRepository {
    path: PathBuf,
    cache: Arc<RwLock<Option<DaemonState>>>,
}

impl FsRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path, cache: Arc::new(RwLock::new(None)) }
    }
}

#[async_trait::async_trait]
impl RuntimeRepository for FsRepository {
    async fn load(&self) -> Result<DaemonState, RepositoryError> {
        {
            let cached = self.cache.read().await;
            if let Some(ref state) = *cached {
                return Ok(state.clone());
            }
        }

        if !self.path.exists() {
            let state = DaemonState::new();
            self.cache.write().await.replace(state.clone());
            return Ok(state);
        }

        let content = tokio::fs::read_to_string(&self.path).await
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        let state: DaemonState = serde_json::from_str(&content)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
        self.cache.write().await.replace(state.clone());
        Ok(state)
    }

    async fn save(&self, state: &DaemonState) -> Result<(), RepositoryError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| RepositoryError::Io(e.to_string()))?;
        }
        let content = serde_json::to_string_pretty(state)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
        tokio::fs::write(&self.path, &content).await
            .map_err(|e| RepositoryError::Io(e.to_string()))?;
        self.cache.write().await.replace(state.clone());
        Ok(())
    }

    async fn add_installation(&self, installation: Installation) -> Result<(), RepositoryError> {
        let mut state = self.load().await?;
        state.installations.push(installation);
        self.save(&state).await
    }

    async fn update_status(&self, runtime: &RuntimeName, version: &Version, status: &str) -> Result<(), RepositoryError> {
        let mut state = self.load().await?;
        if let Some(inst) = state.find_installation_mut(runtime, version) {
            inst.status = futou_core::domain::runtime::InstallStatus::Installed;
            // map string status to enum
            inst.status = match status {
                "active" => futou_core::domain::runtime::InstallStatus::Active,
                "installed" => futou_core::domain::runtime::InstallStatus::Installed,
                _ => futou_core::domain::runtime::InstallStatus::Error(status.to_string()),
            };
        }
        self.save(&state).await
    }

    async fn remove_installation(&self, runtime: &RuntimeName, version: &Version) -> Result<(), RepositoryError> {
        let mut state = self.load().await?;
        state.installations.retain(|i| !(i.runtime == *runtime && i.version == *version));
        self.save(&state).await
    }
}
