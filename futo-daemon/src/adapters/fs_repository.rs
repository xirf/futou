use std::path::{Path, PathBuf};

use futou_core::domain::runtime::{DaemonState, Installation, RuntimeName, Version};
use futou_core::ports::runtime_repo::{RepositoryError, RuntimeRepository};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::warn;

#[cfg(windows)]
async fn replace_file(source: PathBuf, destination: PathBuf) -> std::io::Result<()> {
    tokio::task::spawn_blocking(move || {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        };

        let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        // SAFETY: both pointers reference NUL-terminated buffers alive for the duration of the call.
        if unsafe {
            MoveFileExW(
                source.as_ptr(),
                destination.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    })
    .await
    .map_err(std::io::Error::other)?
}

#[cfg(not(windows))]
async fn replace_file(source: PathBuf, destination: PathBuf) -> std::io::Result<()> {
    tokio::fs::rename(source, destination).await
}

/// Stores daemon state in an atomically replaced JSON file.
pub struct FsRepository {
    path: PathBuf,
    state: Mutex<Option<DaemonState>>,
}

impl FsRepository {
    /// Creates a repository backed by `path`.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            state: Mutex::new(None),
        }
    }

    fn backup_path(&self) -> PathBuf {
        self.path.with_extension("json.bak")
    }

    fn temporary_path(&self) -> PathBuf {
        self.path.with_extension("json.tmp")
    }

    fn backup_temporary_path(&self) -> PathBuf {
        self.path.with_extension("json.bak.tmp")
    }

    async fn read_optional(path: &Path) -> Result<Option<String>, RepositoryError> {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(Some(content)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(RepositoryError::Io(error.to_string())),
        }
    }

    async fn recover_backup(
        &self,
        primary_error: Option<&RepositoryError>,
    ) -> Result<Option<DaemonState>, RepositoryError> {
        let Some(content) = Self::read_optional(&self.backup_path()).await? else {
            return Ok(None);
        };
        let state = serde_json::from_str(&content).map_err(|backup_error| {
            RepositoryError::Serialization(match primary_error {
                Some(primary_error) => format!(
                    "primary state invalid ({primary_error}); backup invalid ({backup_error})"
                ),
                None => format!("backup state invalid ({backup_error})"),
            })
        })?;
        Ok(Some(state))
    }

    async fn load_from_disk(&self) -> Result<DaemonState, RepositoryError> {
        let Some(content) = Self::read_optional(&self.path).await? else {
            if let Some(state) = self.recover_backup(None).await? {
                warn!(path = %self.path.display(), "recovering missing state from backup");
                self.persist(&state).await?;
                return Ok(state);
            }
            return Ok(DaemonState::new());
        };

        match serde_json::from_str(&content)
            .map_err(|error| RepositoryError::Serialization(error.to_string()))
        {
            Ok(state) => Ok(state),
            Err(primary_error) => match self.recover_backup(Some(&primary_error)).await? {
                Some(state) => {
                    warn!(path = %self.path.display(), "recovering invalid state from backup");
                    self.persist(&state).await?;
                    Ok(state)
                }
                None => Err(primary_error),
            },
        }
    }

    async fn persist(&self, state: &DaemonState) -> Result<(), RepositoryError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| RepositoryError::Io(error.to_string()))?;
        }

        let content = serde_json::to_vec_pretty(state)
            .map_err(|error| RepositoryError::Serialization(error.to_string()))?;
        let temporary_path = self.temporary_path();
        let write_result = async {
            let mut temporary = tokio::fs::File::create(&temporary_path).await?;
            temporary.write_all(&content).await?;
            temporary.sync_all().await
        }
        .await;
        if let Err(error) = write_result {
            let _ = tokio::fs::remove_file(&temporary_path).await;
            return Err(RepositoryError::Io(error.to_string()));
        }

        let has_valid_primary = matches!(
            Self::read_optional(&self.path).await?,
            Some(content) if serde_json::from_str::<DaemonState>(&content).is_ok()
        );
        if has_valid_primary {
            let backup_temporary_path = self.backup_temporary_path();
            if let Err(error) = tokio::fs::copy(&self.path, &backup_temporary_path).await {
                let _ = tokio::fs::remove_file(backup_temporary_path).await;
                let _ = tokio::fs::remove_file(&temporary_path).await;
                return Err(RepositoryError::Io(error.to_string()));
            }
            let backup = match tokio::fs::OpenOptions::new()
                .write(true)
                .open(&backup_temporary_path)
                .await
            {
                Ok(backup) => backup,
                Err(error) => {
                    let _ = tokio::fs::remove_file(backup_temporary_path).await;
                    let _ = tokio::fs::remove_file(&temporary_path).await;
                    return Err(RepositoryError::Io(error.to_string()));
                }
            };
            if let Err(error) = backup.sync_all().await {
                let _ = tokio::fs::remove_file(backup_temporary_path).await;
                let _ = tokio::fs::remove_file(&temporary_path).await;
                return Err(RepositoryError::Io(error.to_string()));
            }
            if let Err(error) =
                replace_file(backup_temporary_path.clone(), self.backup_path()).await
            {
                let _ = tokio::fs::remove_file(backup_temporary_path).await;
                let _ = tokio::fs::remove_file(&temporary_path).await;
                return Err(RepositoryError::Io(error.to_string()));
            }
        }
        if let Err(error) = replace_file(temporary_path.clone(), self.path.clone()).await {
            let _ = tokio::fs::remove_file(temporary_path).await;
            return Err(RepositoryError::Io(error.to_string()));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl RuntimeRepository for FsRepository {
    async fn load(&self) -> Result<DaemonState, RepositoryError> {
        let mut state = self.state.lock().await;
        if state.is_none() {
            *state = Some(self.load_from_disk().await?);
        }
        Ok(state.as_ref().expect("state initialized").clone())
    }

    async fn update_state(
        &self,
        update: Box<
            dyn for<'state> FnOnce(&'state mut DaemonState) -> Result<(), RepositoryError> + Send,
        >,
    ) -> Result<DaemonState, RepositoryError> {
        let mut cached = self.state.lock().await;
        let mut next = match cached.as_ref() {
            Some(state) => state.clone(),
            None => self.load_from_disk().await?,
        };
        update(&mut next)?;
        self.persist(&next).await?;
        *cached = Some(next.clone());
        Ok(next)
    }

    async fn add_installation(&self, installation: Installation) -> Result<(), RepositoryError> {
        self.update_state(Box::new(move |state| {
            if state
                .find_installation(&installation.runtime, &installation.version)
                .is_some()
            {
                return Err(RepositoryError::AlreadyExists {
                    runtime: installation.runtime.to_string(),
                    version: installation.version.to_string(),
                });
            }
            state.installations.push(installation);
            Ok(())
        }))
        .await
        .map(drop)
    }

    async fn update_status(
        &self,
        runtime: &RuntimeName,
        version: &Version,
        status: &str,
    ) -> Result<(), RepositoryError> {
        let runtime = runtime.clone();
        let version = version.clone();
        let status = status.to_owned();
        self.update_state(Box::new(move |state| {
            let installation =
                state
                    .find_installation_mut(&runtime, &version)
                    .ok_or_else(|| RepositoryError::NotFound {
                        runtime: runtime.to_string(),
                        version: version.to_string(),
                    })?;
            installation.status = match status.as_str() {
                "active" => futou_core::domain::runtime::InstallStatus::Active,
                "installed" => futou_core::domain::runtime::InstallStatus::Installed,
                _ => futou_core::domain::runtime::InstallStatus::Error(status),
            };
            Ok(())
        }))
        .await
        .map(drop)
    }

    async fn remove_installation(
        &self,
        runtime: &RuntimeName,
        version: &Version,
    ) -> Result<(), RepositoryError> {
        let runtime = runtime.clone();
        let version = version.clone();
        self.update_state(Box::new(move |state| {
            let previous_len = state.installations.len();
            state.installations.retain(|installation| {
                installation.runtime != runtime || installation.version != version
            });
            if state.installations.len() == previous_len {
                return Err(RepositoryError::NotFound {
                    runtime: runtime.to_string(),
                    version: version.to_string(),
                });
            }
            if state.active_version(&runtime) == Some(version.0.as_str()) {
                state.remove_active(&runtime);
            }
            Ok(())
        }))
        .await
        .map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futou_core::domain::runtime::InstallStatus;
    use uuid::Uuid;

    fn test_path() -> PathBuf {
        std::env::temp_dir().join(format!("futou-state-{}.json", Uuid::new_v4()))
    }

    fn installation(version: &str) -> Installation {
        Installation {
            runtime: RuntimeName("nodejs".into()),
            version: Version(version.into()),
            status: InstallStatus::Installed,
            path: format!(r"C:\runtimes\nodejs\{version}"),
            version_dir: format!(r"C:\runtimes\nodejs\{version}"),
            installed_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn concurrent_updates_do_not_lose_installations() {
        let path = test_path();
        let repository = std::sync::Arc::new(FsRepository::new(path.clone()));
        let mut tasks = Vec::new();
        for version in 0..20 {
            let repository = repository.clone();
            tasks.push(tokio::spawn(async move {
                repository
                    .add_installation(installation(&version.to_string()))
                    .await
                    .unwrap();
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }

        assert_eq!(repository.load().await.unwrap().installations.len(), 20);
        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_file(path.with_extension("json.bak")).await;
    }

    #[tokio::test]
    async fn corrupt_primary_recovers_valid_backup() {
        let path = test_path();
        let backup = path.with_extension("json.bak");
        tokio::fs::write(&path, "not json").await.unwrap();
        let mut state = DaemonState::new();
        state.installations.push(installation("22"));
        tokio::fs::write(&backup, serde_json::to_vec(&state).unwrap())
            .await
            .unwrap();

        let repository = FsRepository::new(path.clone());
        assert_eq!(repository.load().await.unwrap().installations.len(), 1);
        let recovered = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(serde_json::from_str::<DaemonState>(&recovered).is_ok());
        let _ = tokio::fs::remove_file(path).await;
        let _ = tokio::fs::remove_file(backup).await;
    }

    #[tokio::test]
    async fn interrupted_temporary_file_does_not_replace_primary() {
        let path = test_path();
        let mut state = DaemonState::new();
        state.installations.push(installation("20"));
        tokio::fs::write(&path, serde_json::to_vec(&state).unwrap())
            .await
            .unwrap();
        tokio::fs::write(path.with_extension("json.tmp"), "partial")
            .await
            .unwrap();

        let repository = FsRepository::new(path.clone());
        assert_eq!(repository.load().await.unwrap().installations.len(), 1);
        let _ = tokio::fs::remove_file(path).await;
        let _ = tokio::fs::remove_file(repository.temporary_path()).await;
    }

    #[tokio::test]
    async fn unreadable_primary_is_not_treated_as_empty_state() {
        let path = test_path();
        tokio::fs::create_dir(&path).await.unwrap();

        let repository = FsRepository::new(path.clone());
        assert!(matches!(
            repository.load().await,
            Err(RepositoryError::Io(_))
        ));
        tokio::fs::remove_dir(path).await.unwrap();
    }

    #[tokio::test]
    async fn duplicate_installation_is_rejected_atomically() {
        let path = test_path();
        let repository = FsRepository::new(path.clone());
        repository
            .add_installation(installation("22"))
            .await
            .unwrap();

        assert!(matches!(
            repository.add_installation(installation("22")).await,
            Err(RepositoryError::AlreadyExists { .. })
        ));
        assert_eq!(repository.load().await.unwrap().installations.len(), 1);
        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_file(path.with_extension("json.bak")).await;
    }

    #[tokio::test]
    async fn removing_active_installation_clears_active_version() {
        let path = test_path();
        let repository = FsRepository::new(path.clone());
        repository
            .add_installation(installation("22"))
            .await
            .unwrap();
        repository
            .update_state(Box::new(|state| {
                state.set_active(&RuntimeName("nodejs".into()), &Version("22".into()));
                Ok(())
            }))
            .await
            .unwrap();

        repository
            .remove_installation(&RuntimeName("nodejs".into()), &Version("22".into()))
            .await
            .unwrap();
        assert!(repository
            .load()
            .await
            .unwrap()
            .active_version(&RuntimeName("nodejs".into()))
            .is_none());
        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_file(path.with_extension("json.bak")).await;
    }
}
