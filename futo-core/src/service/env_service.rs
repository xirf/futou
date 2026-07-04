use std::sync::Arc;

use crate::ports::runtime_repo::RuntimeRepository;

pub struct EnvService {
    repository: Arc<dyn RuntimeRepository>,
}

impl EnvService {
    pub fn new(repository: Arc<dyn RuntimeRepository>) -> Self {
        Self { repository }
    }

    pub async fn status(&self) -> Result<EnvStatus, EnvServiceError> {
        let state = self.repository.load().await?;
        let installed: Vec<InstalledEntry> = state.installations.iter().map(|i| {
            let process = state.get_pid(&i.runtime).map(|_| "running".to_string());
            InstalledEntry {
                runtime: i.runtime.to_string(),
                version: i.version.to_string(),
                status: format!("{:?}", i.status),
                path: i.path.clone(),
                version_dir: i.version_dir.clone(),
                process,
            }
        }).collect();

        let active = state.active.clone();

        Ok(EnvStatus { installed, active })
    }

    pub async fn installed_runtimes(&self) -> Result<Vec<String>, EnvServiceError> {
        let state = self.repository.load().await?;
        let mut names: Vec<String> = state.installations.iter().map(|i| i.runtime.to_string()).collect();
        names.sort();
        names.dedup();
        Ok(names)
    }
}

pub struct EnvStatus {
    pub installed: Vec<InstalledEntry>,
    pub active: std::collections::HashMap<String, String>,
}

pub struct InstalledEntry {
    pub runtime: String,
    pub version: String,
    pub status: String,
    pub path: String,
    pub version_dir: String,
    pub process: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum EnvServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] crate::ports::runtime_repo::RepositoryError),
}
