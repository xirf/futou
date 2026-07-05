use std::path::PathBuf;
use std::sync::Arc;

use sha2::Digest;
use tracing::info;

use crate::domain::runtime::{InstallStatus, Installation, RuntimeName, Version};
use crate::ports::catalogue_source::CatalogueSource;
use crate::ports::downloader::Downloader;
use crate::ports::extractor::Extractor;
use crate::ports::runtime_repo::RuntimeRepository;

#[derive(Clone)]
pub struct InstallService {
    downloader: Arc<dyn Downloader>,
    extractor: Arc<dyn Extractor>,
    repository: Arc<dyn RuntimeRepository>,
    catalogue: Arc<dyn CatalogueSource>,
    runtimes_dir: PathBuf,
}

impl InstallService {
    pub fn new(
        downloader: Arc<dyn Downloader>,
        extractor: Arc<dyn Extractor>,
        repository: Arc<dyn RuntimeRepository>,
        catalogue: Arc<dyn CatalogueSource>,
        runtimes_dir: PathBuf,
    ) -> Self {
        Self {
            downloader,
            extractor,
            repository,
            catalogue,
            runtimes_dir,
        }
    }

    pub async fn install(
        &self,
        runtime: &RuntimeName,
        version: &Version,
        progress: impl Fn(f64, String) + Send + Sync + 'static,
    ) -> Result<Installation, InstallError> {
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

        let version_dir = self.runtimes_dir.join(&runtime.0).join(&version.0);
        let archive_path = self
            .runtimes_dir
            .join(&runtime.0)
            .join(format!("{}-{}.tmp", runtime, version));

        if let Some(parent) = archive_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| InstallError::Io(e.to_string()))?;
        }

        progress(0.05, "Downloading".to_string());
        let progress_dl = progress.clone();
        self.downloader
            .download(
                &version_urls.url,
                &archive_path,
                Box::new(move |pct, msg| {
                    let calculated = 0.05 + pct * 0.65;
                    progress_dl(calculated, msg);
                }),
            )
            .await?;

        progress(0.70, "Verifying checksum".to_string());
        if !version_urls.checksum.is_empty() {
            verify_checksum(&archive_path, &version_urls.checksum)
                .map_err(|e| InstallError::Verification(e.to_string()))?;
        }

        progress(0.80, "Extracting".to_string());
        std::fs::create_dir_all(&version_dir).map_err(|e| InstallError::Io(e.to_string()))?;
        self.extractor
            .extract(&archive_path, &version_dir, &version_urls.archive_type)
            .await?;

        let bin_dir = version_urls
            .bin_dir
            .map(|d| version_dir.join(d))
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

        self.repository
            .add_installation(installation.clone())
            .await?;

        let _ = std::fs::remove_file(&archive_path);

        progress(1.0, "Done".to_string());
        Ok(installation)
    }

    pub async fn uninstall(
        &self,
        runtime: &RuntimeName,
        version: &Version,
    ) -> Result<(), InstallError> {
        let version_dir = self.runtimes_dir.join(&runtime.0).join(&version.0);
        if version_dir.exists() {
            std::fs::remove_dir_all(&version_dir).map_err(|e| InstallError::Io(e.to_string()))?;
        }
        self.repository
            .remove_installation(runtime, version)
            .await?;
        Ok(())
    }
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
pub enum InstallError {
    #[error("Already installed")]
    AlreadyInstalled,
    #[error("Catalogue error: {0}")]
    Catalogue(#[from] crate::ports::catalogue_source::CatalogueError),
    #[error("Download error: {0}")]
    Download(#[from] crate::ports::downloader::DownloadError),
    #[error("Extraction error: {0}")]
    Extract(#[from] crate::ports::extractor::ExtractError),
    #[error("Verification failed: {0}")]
    Verification(String),
    #[error("Repository error: {0}")]
    Repository(#[from] crate::ports::runtime_repo::RepositoryError),
    #[error("IO error: {0}")]
    Io(String),
}
