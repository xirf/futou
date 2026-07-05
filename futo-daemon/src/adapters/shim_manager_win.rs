use std::collections::HashMap;
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

fn manifest_path(shim_dir: &Path) -> PathBuf {
    shim_dir.join("shims.json")
}

fn load_manifest(shim_dir: &Path) -> HashMap<String, Vec<String>> {
    let path = manifest_path(shim_dir);
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_manifest(shim_dir: &Path, manifest: &HashMap<String, Vec<String>>) {
    let path = manifest_path(shim_dir);
    let _ = std::fs::create_dir_all(shim_dir);
    if let Ok(json) = serde_json::to_string_pretty(manifest) {
        let _ = std::fs::write(path, json);
    }
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
        runtime: &str,
        version: &str,
        bin_dir: &Path,
    ) -> Result<(), ShimError> {
        std::fs::create_dir_all(&self.shim_dir).map_err(|e| ShimError::Io(e.to_string()))?;

        let key = format!("{}/{}", runtime, version);
        let mut manifest = load_manifest(&self.shim_dir);

        if let Some(old_files) = manifest.remove(runtime) {
            for f in &old_files {
                let _ = std::fs::remove_file(self.shim_dir.join(f));
            }
        }

        let mut new_files = Vec::new();
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

            let bat_name = format!("{}.bat", filename.to_string_lossy());
            let bat_path = self.shim_dir.join(&bat_name);
            Self::create_bat_shim(&bat_path, &path)?;
            new_files.push(bat_name);
        }

        manifest.insert(key, new_files);
        save_manifest(&self.shim_dir, &manifest);

        Ok(())
    }

    async fn remove_shims(&self, runtime: &str) -> Result<(), ShimError> {
        let mut manifest = load_manifest(&self.shim_dir);

        for (key, files) in &manifest {
            if key.starts_with(&format!("{}/", runtime)) || key == runtime {
                for f in files {
                    let _ = std::fs::remove_file(self.shim_dir.join(f));
                }
            }
        }
        manifest.retain(|k, _| !k.starts_with(&format!("{}/", runtime)) && k != runtime);
        save_manifest(&self.shim_dir, &manifest);

        Ok(())
    }

    async fn shim_dir(&self) -> Result<PathBuf, ShimError> {
        Ok(self.shim_dir.clone())
    }
}
