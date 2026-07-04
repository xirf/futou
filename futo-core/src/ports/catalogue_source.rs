use futou_ipc::catalogue::CatalogueManifest;

#[async_trait::async_trait]
pub trait CatalogueSource: Send + Sync {
    async fn fetch(&self) -> Result<CatalogueManifest, CatalogueError>;
    async fn fetch_version_urls(&self, runtime: &str, version: &str) -> Result<VersionUrls, CatalogueError>;
}

pub struct VersionUrls {
    pub url: String,
    pub checksum: String,
    pub archive_type: String,
    pub bin_dir: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogueError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Runtime {0} not found in catalogue")]
    RuntimeNotFound(String),
    #[error("Version {version} of {runtime} not found")]
    VersionNotFound { runtime: String, version: String },
    #[error("No download URL for current platform")]
    NoPlatformMatch,
}
