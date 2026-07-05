use std::path::{Path, PathBuf};

use futou_core::ports::shim_manager::{ShimError, ShimManager};

pub struct WindowsShimManager {
    shim_dir: PathBuf,
}

const EXECUTABLE_EXTENSIONS: &[&str] = &["exe", "bat", "cmd", "com"];

fn is_executable(name: &std::ffi::OsStr) -> bool {
    let name = name.to_string_lossy();
    let lower = name.to_lowercase();
    EXECUTABLE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{}", ext)) || lower == *ext)
}

impl WindowsShimManager {
    pub fn new(shim_dir: PathBuf) -> Self {
        Self { shim_dir }
    }

    fn create_bat_shim(shim_path: &Path, target: &Path) -> Result<(), ShimError> {
        let target_exe = target.to_string_lossy().replace('/', "\\");
        let bat_content = format!("@echo off\r\n\"{}\" %*\r\n", target_exe);
        std::fs::write(shim_path, bat_content).map_err(|e| ShimError::Io(e.to_string()))
    }
}

#[async_trait::async_trait]
impl ShimManager for WindowsShimManager {
    async fn create_shims(
        &self,
        _runtime: &str,
        _version: &str,
        bin_dir: &Path,
    ) -> Result<(), ShimError> {
        std::fs::create_dir_all(&self.shim_dir).map_err(|e| ShimError::Io(e.to_string()))?;

        let mut reader = tokio::fs::read_dir(bin_dir)
            .await
            .map_err(|e| ShimError::Io(e.to_string()))?;

        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|e| ShimError::Io(e.to_string()))?
        {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let filename = entry.file_name();
            if !is_executable(&filename) {
                continue;
            }

            let bat_path = self.shim_dir.join(&filename).with_extension("bat");
            let _ = std::fs::remove_file(&bat_path);
            Self::create_bat_shim(&bat_path, &path)?;
        }

        Ok(())
    }

    async fn remove_shims(&self, _runtime: &str) -> Result<(), ShimError> {
        let mut reader = tokio::fs::read_dir(&self.shim_dir)
            .await
            .map_err(|e| ShimError::Io(e.to_string()))?;

        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|e| ShimError::Io(e.to_string()))?
        {
            let path = entry.path();
            if path.is_file() || path.is_symlink() {
                let _ = std::fs::remove_file(&path);
            }
        }

        Ok(())
    }

    async fn shim_dir(&self) -> Result<PathBuf, ShimError> {
        Ok(self.shim_dir.clone())
    }
}
