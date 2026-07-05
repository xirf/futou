use std::path::PathBuf;
use std::time::Duration;

use futou_core::ports::catalogue_source::{CatalogueError, CatalogueSource, VersionUrls};
use futou_ipc::catalogue::CatalogueManifest;

pub struct RemoteCatalogueSource {
    remote_url: String,
    cache_path: PathBuf,
    bundle_path: PathBuf,
    client: reqwest::Client,
}

impl RemoteCatalogueSource {
    pub fn new(remote_url: String, cache_dir: PathBuf, bundle_dir: PathBuf) -> Self {
        Self {
            remote_url,
            cache_path: cache_dir.join("cache.json"),
            bundle_path: bundle_dir.join("bundle.json"),
            client: reqwest::Client::new(),
        }
    }

    async fn try_cache(&self) -> Option<CatalogueManifest> {
        let cached = tokio::fs::read_to_string(&self.cache_path).await.ok()?;
        serde_json::from_str(&cached).ok()
    }

    async fn try_bundle(&self) -> Result<CatalogueManifest, CatalogueError> {
        let bundled = tokio::fs::read_to_string(&self.bundle_path)
            .await
            .map_err(|_| CatalogueError::Network("No catalogue available".to_string()))?;
        serde_json::from_str(&bundled)
            .map_err(|e| CatalogueError::Network(format!("Bundle parse error: {}", e)))
    }

    async fn fetch_and_cache(&self) -> Result<CatalogueManifest, CatalogueError> {
        match tokio::time::timeout(Duration::from_secs(8), self.fetch_remote()).await {
            Ok(Ok(manifest)) => {
                if let Ok(json) = serde_json::to_string_pretty(&manifest) {
                    let _ = tokio::fs::create_dir_all(self.cache_path.parent().unwrap()).await;
                    let _ = tokio::fs::write(&self.cache_path, &json).await;
                }
                Ok(manifest)
            }
            _ => Err(CatalogueError::Network("remote unavailable".into())),
        }
    }

    async fn load_manifest(&self, force_remote: bool) -> Result<CatalogueManifest, CatalogueError> {
        if !force_remote {
            if let Some(cached) = self.try_cache().await {
                return Ok(cached);
            }
        }

        match self.fetch_and_cache().await {
            Ok(manifest) => Ok(manifest),
            Err(_) => {
                if force_remote {
                    if let Some(cached) = self.try_cache().await {
                        return Ok(cached);
                    }
                }
                self.try_bundle().await
            }
        }
    }

    async fn fetch_remote(&self) -> Result<CatalogueManifest, CatalogueError> {
        let resp = self
            .client
            .get(&self.remote_url)
            .send()
            .await
            .map_err(|e| CatalogueError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CatalogueError::Network(format!("HTTP {}", resp.status())));
        }

        resp.json::<CatalogueManifest>()
            .await
            .map_err(|e| CatalogueError::Network(e.to_string()))
    }
}

#[async_trait::async_trait]
impl CatalogueSource for RemoteCatalogueSource {
    async fn fetch(&self) -> Result<CatalogueManifest, CatalogueError> {
        self.load_manifest(false).await
    }

    async fn refresh(&self) -> Result<CatalogueManifest, CatalogueError> {
        self.load_manifest(true).await
    }

    async fn fetch_version_urls(
        &self,
        runtime: &str,
        version: &str,
    ) -> Result<VersionUrls, CatalogueError> {
        let manifest = self.load_manifest(false).await?;

        let entry = manifest
            .runtimes
            .get(runtime)
            .ok_or_else(|| CatalogueError::RuntimeNotFound(runtime.to_string()))?;

        let version_entry =
            entry
                .versions
                .get(version)
                .ok_or_else(|| CatalogueError::VersionNotFound {
                    runtime: runtime.to_string(),
                    version: version.to_string(),
                })?;

        let platform = if cfg!(target_os = "windows") {
            "windows-amd64"
        } else if cfg!(target_os = "linux") {
            "linux-amd64"
        } else if cfg!(target_os = "macos") {
            "darwin-arm64"
        } else {
            return Err(CatalogueError::NoPlatformMatch);
        };

        let url = version_entry
            .url
            .get(platform)
            .or_else(|| version_entry.url.values().next())
            .ok_or(CatalogueError::NoPlatformMatch)?
            .clone();

        let checksum = version_entry
            .checksum
            .get(platform)
            .cloned()
            .unwrap_or_default();

        Ok(VersionUrls {
            url: url.clone(),
            checksum,
            archive_type: version_entry.archive_type.clone(),
            bin_dir: version_entry.bin_dir.clone(),
        })
    }
}
