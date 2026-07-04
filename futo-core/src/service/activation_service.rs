use std::path::PathBuf;
use std::sync::Arc;

use tracing::info;
use crate::domain::runtime::{InstallStatus, RuntimeName, Version};
use crate::ports::path_manager::PathManager;
use crate::ports::process_manager::ProcessManager;
use crate::ports::runtime_repo::RuntimeRepository;
use crate::ports::shim_manager::ShimManager;

pub struct ActivationService {
    repository: Arc<dyn RuntimeRepository>,
    shim_manager: Arc<dyn ShimManager>,
    path_manager: Arc<dyn PathManager>,
    process_manager: Arc<dyn ProcessManager>,
}

impl ActivationService {
    pub fn new(
        repository: Arc<dyn RuntimeRepository>,
        shim_manager: Arc<dyn ShimManager>,
        path_manager: Arc<dyn PathManager>,
        process_manager: Arc<dyn ProcessManager>,
    ) -> Self {
        Self { repository, shim_manager, path_manager, process_manager }
    }

    pub async fn activate(&self, runtime: &RuntimeName, version: &Version) -> Result<(), ActivationError> {
        let mut state = self.repository.load().await?;

        let installation = state
            .find_installation(runtime, version)
            .ok_or_else(|| ActivationError::NotInstalled(runtime.to_string(), version.to_string()))?
            .clone();

        let bin_dir = PathBuf::from(&installation.path);
        info!("activate: runtime={} version={} bin_dir={:?} exists={}", runtime, version, bin_dir, bin_dir.exists());
        self.shim_manager.create_shims(&runtime.0, &version.0, &bin_dir).await?;

        state.set_active(runtime, version);
        if let Some(inst) = state.find_installation_mut(runtime, version) {
            inst.status = InstallStatus::Active;
        }
        self.repository.save(&state).await?;

        if !self.path_manager.is_in_path(&self.shim_manager.shim_dir().await?).await? {
            self.path_manager.add_to_path(&self.shim_manager.shim_dir().await?).await?;
        }

        Ok(())
    }

    pub async fn deactivate(&self, runtime: &RuntimeName) -> Result<(), ActivationError> {
        let mut state = self.repository.load().await?;

        self.shim_manager.remove_shims(&runtime.0).await?;

        if let Some(version_str) = state.active_version(runtime).map(|s| s.to_string()) {
            let version = Version(version_str);
            if let Some(inst) = state.find_installation_mut(runtime, &version) {
                inst.status = InstallStatus::Installed;
            }
        }

        state.remove_active(runtime);
        self.repository.save(&state).await?;

        Ok(())
    }

    pub async fn start_process(&self, runtime: &RuntimeName, version: &Version) -> Result<u32, ActivationError> {
        let state = self.repository.load().await?;

        let installation = state
            .find_installation(runtime, version)
            .ok_or_else(|| ActivationError::NotInstalled(runtime.to_string(), version.to_string()))?
            .clone();

        let bin_dir = PathBuf::from(&installation.path);
        let data_dir = PathBuf::from(&installation.version_dir).join("data");

        self.process_manager.init_data_dir(&runtime.0, &bin_dir, &data_dir).await
            .map_err(|e| ActivationError::Process(e.to_string()))?;

        let pid = self.process_manager.start_server(&runtime.0, &bin_dir, &data_dir).await
            .map_err(|e| ActivationError::Process(e.to_string()))?;

        let mut state = self.repository.load().await?;
        state.set_pid(runtime, pid);
        self.repository.save(&state).await?;

        Ok(pid)
    }

    pub async fn stop_process(&self, runtime: &RuntimeName) -> Result<(), ActivationError> {
        let state = self.repository.load().await?;

        if let Some(pid) = state.get_pid(runtime) {
            self.process_manager.stop_server(pid).await
                .map_err(|e| ActivationError::Process(e.to_string()))?;

            let mut state = self.repository.load().await?;
            state.remove_pid(runtime);
            self.repository.save(&state).await?;
        }

        Ok(())
    }

    pub async fn use_version(&self, runtime: &RuntimeName, version: &Version) -> Result<(), ActivationError> {
        self.activate(runtime, version).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ActivationError {
    #[error("{0} {1} is not installed")]
    NotInstalled(String, String),
    #[error("Shim error: {0}")]
    Shim(#[from] crate::ports::shim_manager::ShimError),
    #[error("Path error: {0}")]
    Path(#[from] crate::ports::path_manager::PathError),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::ports::runtime_repo::RepositoryError),
    #[error("IO error: {0}")]
    Io(String),
}
