use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use sha2::Digest;
use tracing::info;

use crate::domain::runtime::{InstallStatus, Installation, RuntimeName, Version};
use crate::ports::catalogue_source::CatalogueSource;
use crate::ports::downloader::Downloader;
use crate::ports::extractor::Extractor;
use crate::ports::runtime_repo::RuntimeRepository;
use crate::ports::shim_manager::ShimManager;

static INSTALL_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
/// Installs and removes checksum-pinned runtime archives.
pub struct InstallService {
    downloader: Arc<dyn Downloader>,
    extractor: Arc<dyn Extractor>,
    repository: Arc<dyn RuntimeRepository>,
    catalogue: Arc<dyn CatalogueSource>,
    shim_manager: Arc<dyn ShimManager>,
    runtimes_dir: PathBuf,
}

impl InstallService {
    /// Creates an install service using the supplied platform adapters.
    pub fn new(
        downloader: Arc<dyn Downloader>,
        extractor: Arc<dyn Extractor>,
        repository: Arc<dyn RuntimeRepository>,
        catalogue: Arc<dyn CatalogueSource>,
        shim_manager: Arc<dyn ShimManager>,
        runtimes_dir: PathBuf,
    ) -> Self {
        Self {
            downloader,
            extractor,
            repository,
            catalogue,
            shim_manager,
            runtimes_dir,
        }
    }

    /// Downloads, verifies, stages, and atomically registers one runtime version.
    pub async fn install(
        &self,
        runtime: &RuntimeName,
        version: &Version,
        progress: impl Fn(f64, String) + Send + Sync + 'static,
    ) -> Result<Installation, InstallError> {
        validate_windows_component(&runtime.0)?;
        validate_windows_component(&version.0)?;

        if self
            .repository
            .load()
            .await?
            .find_installation(runtime, version)
            .is_some()
        {
            return Err(InstallError::AlreadyInstalled);
        }

        let progress = Arc::new(progress);

        progress(0.0, "Looking up version in catalogue".to_string());
        let version_urls = self
            .catalogue
            .fetch_version_urls(&runtime.0, &version.0)
            .await?;

        validate_sha256(&version_urls.checksum)?;
        if !version_urls.url.starts_with("https://") {
            return Err(InstallError::UnsafeUrl(version_urls.url));
        }
        if let Some(bin_dir) = version_urls.bin_dir.as_deref() {
            validate_relative_path(Path::new(bin_dir))?;
        }

        let version_dir = self.runtimes_dir.join(&runtime.0).join(&version.0);
        if version_dir.exists() {
            return Err(InstallError::AlreadyInstalled);
        }
        let nonce = INSTALL_COUNTER.fetch_add(1, Ordering::Relaxed);
        let staging_dir = self.runtimes_dir.join(&runtime.0).join(format!(
            ".{}.installing-{}-{nonce}",
            version,
            std::process::id()
        ));
        let archive_path = self
            .runtimes_dir
            .join(&runtime.0)
            .join(format!(".{runtime}-{version}-{nonce}.download"));

        if let Some(parent) = archive_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| InstallError::Io(e.to_string()))?;
        }

        progress(0.05, "Downloading".to_string());
        let progress_dl = progress.clone();
        if let Err(error) = self
            .downloader
            .download(
                &version_urls.url,
                &archive_path,
                Box::new(move |pct, msg| {
                    let calculated = 0.05 + pct * 0.65;
                    progress_dl(calculated, msg);
                }),
            )
            .await
        {
            let _ = std::fs::remove_file(&archive_path);
            return Err(error.into());
        }

        progress(0.70, "Verifying checksum".to_string());
        if let Err(error) = verify_checksum(&archive_path, &version_urls.checksum) {
            let _ = std::fs::remove_file(&archive_path);
            return Err(InstallError::Verification(error));
        }

        progress(0.80, "Extracting".to_string());
        std::fs::create_dir(&staging_dir).map_err(|error| {
            let _ = std::fs::remove_file(&archive_path);
            InstallError::Io(error.to_string())
        })?;
        if let Err(error) = self
            .extractor
            .extract(&archive_path, &staging_dir, &version_urls.archive_type)
            .await
        {
            let _ = std::fs::remove_file(&archive_path);
            let _ = std::fs::remove_dir_all(&staging_dir);
            return Err(error.into());
        }

        let staged_bin_dir = version_urls
            .bin_dir
            .as_deref()
            .map(|directory| staging_dir.join(directory))
            .unwrap_or_else(|| staging_dir.clone());
        let staging_root = std::fs::canonicalize(&staging_dir).map_err(|error| {
            let _ = std::fs::remove_file(&archive_path);
            let _ = std::fs::remove_dir_all(&staging_dir);
            InstallError::Io(error.to_string())
        })?;
        let canonical_bin = std::fs::canonicalize(&staged_bin_dir).map_err(|error| {
            let _ = std::fs::remove_file(&archive_path);
            let _ = std::fs::remove_dir_all(&staging_dir);
            InstallError::Io(error.to_string())
        })?;
        if !canonical_bin.starts_with(&staging_root) {
            let _ = std::fs::remove_dir_all(&staging_dir);
            let _ = std::fs::remove_file(&archive_path);
            return Err(InstallError::UnsafePath(
                canonical_bin.display().to_string(),
            ));
        }
        std::fs::rename(&staging_dir, &version_dir).map_err(|error| {
            let _ = std::fs::remove_file(&archive_path);
            let _ = std::fs::remove_dir_all(&staging_dir);
            InstallError::Io(error.to_string())
        })?;
        let bin_dir = version_urls
            .bin_dir
            .map(|directory| version_dir.join(directory))
            .unwrap_or_else(|| version_dir.clone());
        info!(
            "install: runtime={} version={} bin_dir={:?}",
            runtime, version, bin_dir
        );

        progress(0.95, "Registering installation".to_string());
        let installation = Installation {
            runtime: runtime.clone(),
            version: version.clone(),
            status: InstallStatus::Installed,
            path: bin_dir.to_string_lossy().to_string(),
            version_dir: version_dir.to_string_lossy().to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
        };

        if let Err(error) = self.repository.add_installation(installation.clone()).await {
            let _ = std::fs::remove_dir_all(&version_dir);
            let _ = std::fs::remove_file(&archive_path);
            return match error {
                crate::ports::runtime_repo::RepositoryError::AlreadyExists { .. } => {
                    Err(InstallError::AlreadyInstalled)
                }
                other => Err(other.into()),
            };
        }

        let _ = std::fs::remove_file(&archive_path);

        progress(1.0, "Done".to_string());
        Ok(installation)
    }

    /// Removes an installation only when its recorded path matches its runtime identity.
    pub async fn uninstall(
        &self,
        runtime: &RuntimeName,
        version: &Version,
    ) -> Result<(), InstallError> {
        validate_windows_component(&runtime.0)?;
        validate_windows_component(&version.0)?;

        let state = self.repository.load().await?;
        let installation = state
            .find_installation(runtime, version)
            .ok_or(InstallError::NotInstalled)?;
        if installation.version_dir.is_empty() {
            return Err(InstallError::UnsafePath(
                "installation has no recorded version directory".into(),
            ));
        }

        let version_dir = PathBuf::from(&installation.version_dir);
        let expected_dir = self.runtimes_dir.join(&runtime.0).join(&version.0);
        let root = std::fs::canonicalize(&self.runtimes_dir)
            .map_err(|e| InstallError::Io(e.to_string()))?;
        let target =
            std::fs::canonicalize(&version_dir).map_err(|e| InstallError::Io(e.to_string()))?;
        let expected =
            std::fs::canonicalize(expected_dir).map_err(|e| InstallError::Io(e.to_string()))?;
        if target == root || target != expected || !target.starts_with(&root) {
            return Err(InstallError::UnsafePath(target.display().to_string()));
        }

        std::fs::remove_dir_all(&target).map_err(|e| InstallError::Io(e.to_string()))?;
        self.repository
            .remove_installation(runtime, version)
            .await?;
        if state.active_version(runtime) == Some(version.0.as_str()) {
            self.shim_manager.remove_shims(&runtime.0).await?;
        }
        Ok(())
    }
}

fn validate_windows_component(value: &str) -> Result<(), InstallError> {
    let path = Path::new(value);
    let is_single_component = matches!(
        (path.components().next(), path.components().nth(1)),
        (Some(Component::Normal(_)), None)
    );
    let trimmed = value.trim_end_matches([' ', '.']);
    let stem = trimmed.split('.').next().unwrap_or_default();
    let is_reserved = matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );
    if value.is_empty()
        || !is_single_component
        || trimmed != value
        || value.chars().any(|c| c < ' ' || "<>:\"/\\|?*".contains(c))
        || is_reserved
    {
        return Err(InstallError::InvalidIdentifier(value.into()));
    }
    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<(), InstallError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(InstallError::UnsafePath(path.display().to_string()));
    }
    for component in path.components() {
        validate_windows_component(&component.as_os_str().to_string_lossy())?;
    }
    Ok(())
}

fn validate_sha256(checksum: &str) -> Result<(), InstallError> {
    let checksum = checksum.strip_prefix("sha256:").unwrap_or(checksum);
    if checksum.len() != 64
        || !checksum
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(InstallError::InvalidChecksum);
    }
    Ok(())
}

fn verify_checksum(path: &std::path::Path, expected: &str) -> Result<(), String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        use sha2::Digest;
        hasher.update(&buffer[..n]);
    }
    let hash = format!("{:x}", hasher.finalize());

    let expected_clean = expected.strip_prefix("sha256:").unwrap_or(expected);
    if hash != expected_clean {
        return Err(format!("expected {}, got {}", expected_clean, hash));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
/// Describes a rejected or failed runtime installation operation.
pub enum InstallError {
    #[error("Already installed")]
    AlreadyInstalled,
    #[error("Runtime is not installed")]
    NotInstalled,
    #[error("Invalid runtime or version identifier: {0}")]
    InvalidIdentifier(String),
    #[error("Unsafe path: {0}")]
    UnsafePath(String),
    #[error("Catalogue URL must use HTTPS: {0}")]
    UnsafeUrl(String),
    #[error("Catalogue entry has no valid SHA-256 checksum")]
    InvalidChecksum,
    #[error("Catalogue error: {0}")]
    Catalogue(#[from] crate::ports::catalogue_source::CatalogueError),
    #[error("Download error: {0}")]
    Download(#[from] crate::ports::downloader::DownloadError),
    #[error("Extraction error: {0}")]
    Extract(#[from] crate::ports::extractor::ExtractError),
    #[error("Shim error: {0}")]
    Shim(#[from] crate::ports::shim_manager::ShimError),
    #[error("Verification failed: {0}")]
    Verification(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::ports::runtime_repo::RepositoryError),
    #[error("IO error: {0}")]
    Io(String),
}

#[cfg(test)]
mod tests {
    use super::{
        validate_relative_path, validate_sha256, validate_windows_component, verify_checksum,
    };
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn write_temp(content: &[u8]) -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir();
        let path = dir.join(format!("futou_test_checksum_{}_{}", std::process::id(), n));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content).unwrap();
        path
    }

    #[test]
    fn verify_matching_checksum_passes() {
        // SHA-256 of "hello world\n"
        let content = b"hello world\n";
        let expected = "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447";
        let path = write_temp(content);
        assert!(verify_checksum(&path, expected).is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_wrong_checksum_fails() {
        let content = b"hello world\n";
        let path = write_temp(content);
        let result = verify_checksum(
            &path,
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_strips_sha256_prefix() {
        let content = b"hello world\n";
        let expected_with_prefix =
            "sha256:a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447";
        let path = write_temp(content);
        assert!(verify_checksum(&path, expected_with_prefix).is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_empty_file() {
        // SHA-256 of empty
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let path = write_temp(b"");
        assert!(verify_checksum(&path, expected).is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_nonexistent_file_errors() {
        let result = verify_checksum(std::path::Path::new("nonexistent_file_xyz.abc"), "abc");
        assert!(result.is_err());
    }

    #[test]
    fn accepts_safe_identifiers_and_nested_bin_directories() {
        assert!(validate_windows_component("nodejs").is_ok());
        assert!(validate_windows_component("22.0.0-rc.1").is_ok());
        assert!(validate_relative_path(std::path::Path::new("node-v22/bin")).is_ok());
    }

    #[test]
    fn rejects_unsafe_windows_identifiers() {
        for value in [
            "",
            ".",
            "..",
            "../escape",
            r"..\escape",
            r"C:\escape",
            "CON",
            "name.",
        ] {
            assert!(
                validate_windows_component(value).is_err(),
                "accepted {value:?}"
            );
        }
    }

    #[test]
    fn rejects_unsafe_catalogue_paths() {
        for value in ["", "../bin", r"C:\bin", r"safe\..\bin"] {
            assert!(validate_relative_path(std::path::Path::new(value)).is_err());
        }
    }

    #[test]
    fn requires_a_sha256_checksum() {
        assert!(validate_sha256("").is_err());
        assert!(validate_sha256("abc").is_err());
        assert!(validate_sha256(&"a".repeat(64)).is_ok());
        assert!(validate_sha256(&format!("sha256:{}", "a".repeat(64))).is_ok());
        assert!(validate_sha256(&"A".repeat(64)).is_err());
    }
}
