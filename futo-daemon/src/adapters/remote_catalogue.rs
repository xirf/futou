use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use futou_core::ports::catalogue_source::{CatalogueError, CatalogueSource, VersionUrls};
use futou_ipc::catalogue::{CatalogueManifest, CATALOGUE_SCHEMA_VERSION};
use tokio::io::AsyncWriteExt;

/// Loads only Ed25519-signed catalogue snapshots from remote, cache, or bundle storage.
pub struct RemoteCatalogueSource {
    remote_url: String,
    cache_path: PathBuf,
    bundle_path: PathBuf,
    public_key: [u8; 32],
    client: reqwest::Client,
}

impl RemoteCatalogueSource {
    /// Creates a catalogue source using a pinned Ed25519 public key.
    pub fn new(
        remote_url: String,
        cache_dir: PathBuf,
        bundle_dir: PathBuf,
        public_key: [u8; 32],
    ) -> Self {
        Self {
            remote_url,
            cache_path: cache_dir.join("cache.json"),
            bundle_path: bundle_dir.join("bundle.json"),
            public_key,
            client: reqwest::Client::new(),
        }
    }

    async fn try_cache(&self) -> Option<CatalogueManifest> {
        self.load_signed_file(&self.cache_path).await.ok()
    }

    async fn try_bundle(&self) -> Result<CatalogueManifest, CatalogueError> {
        self.load_signed_file(&self.bundle_path)
            .await
            .map_err(|error| {
                CatalogueError::Network(format!("No trusted catalogue available: {error}"))
            })
    }

    async fn load_signed_file(
        &self,
        manifest_path: &Path,
    ) -> Result<CatalogueManifest, CatalogueError> {
        let manifest = tokio::fs::read(manifest_path)
            .await
            .map_err(|error| CatalogueError::Network(error.to_string()))?;
        let signature = tokio::fs::read_to_string(signature_path(manifest_path))
            .await
            .map_err(|error| CatalogueError::Network(error.to_string()))?;
        self.verify_manifest(&manifest, &signature)
    }

    async fn fetch_and_cache(&self) -> Result<CatalogueManifest, CatalogueError> {
        let (bytes, signature, manifest) =
            tokio::time::timeout(Duration::from_secs(8), self.fetch_remote())
                .await
                .map_err(|_| CatalogueError::Network("remote request timed out".into()))??;

        self.write_cache(&bytes, signature.as_bytes()).await?;
        Ok(manifest)
    }

    async fn write_cache(&self, manifest: &[u8], signature: &[u8]) -> Result<(), CatalogueError> {
        let parent = self
            .cache_path
            .parent()
            .ok_or_else(|| CatalogueError::Network("invalid catalogue cache path".into()))?;
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| CatalogueError::Network(error.to_string()))?;

        write_atomic(&self.cache_path, manifest).await?;
        write_atomic(&signature_path(&self.cache_path), signature).await
    }

    async fn load_manifest(&self, force_remote: bool) -> Result<CatalogueManifest, CatalogueError> {
        if !force_remote {
            if let Some(cached) = self.try_cache().await {
                return Ok(cached);
            }
        }

        match self.fetch_and_cache().await {
            Ok(manifest) => Ok(manifest),
            Err(_) if force_remote => match self.try_cache().await {
                Some(cached) => Ok(cached),
                None => self.try_bundle().await,
            },
            Err(_) => self.try_bundle().await,
        }
    }

    async fn fetch_remote(&self) -> Result<(Vec<u8>, String, CatalogueManifest), CatalogueError> {
        let (manifest_response, signature_response) = tokio::try_join!(
            self.client.get(&self.remote_url).send(),
            self.client.get(format!("{}.sig", self.remote_url)).send()
        )
        .map_err(|error| CatalogueError::Network(error.to_string()))?;

        let manifest_response = manifest_response
            .error_for_status()
            .map_err(|error| CatalogueError::Network(error.to_string()))?;
        let signature_response = signature_response
            .error_for_status()
            .map_err(|error| CatalogueError::Network(error.to_string()))?;
        let bytes = manifest_response
            .bytes()
            .await
            .map_err(|error| CatalogueError::Network(error.to_string()))?
            .to_vec();
        let signature = signature_response
            .text()
            .await
            .map_err(|error| CatalogueError::Network(error.to_string()))?;
        let manifest = self.verify_manifest(&bytes, &signature)?;
        Ok((bytes, signature, manifest))
    }

    fn verify_manifest(
        &self,
        bytes: &[u8],
        signature_hex: &str,
    ) -> Result<CatalogueManifest, CatalogueError> {
        let verifying_key = VerifyingKey::from_bytes(&self.public_key)
            .map_err(|error| CatalogueError::Network(format!("Invalid catalogue key: {error}")))?;
        let signature_bytes = decode_signature(signature_hex)?;
        let signature = Signature::from_slice(&signature_bytes).map_err(|error| {
            CatalogueError::Network(format!("Invalid catalogue signature: {error}"))
        })?;
        verifying_key.verify(bytes, &signature).map_err(|_| {
            CatalogueError::Network("Catalogue signature verification failed".into())
        })?;

        let manifest: CatalogueManifest = serde_json::from_slice(bytes)
            .map_err(|error| CatalogueError::Network(format!("Catalogue parse error: {error}")))?;
        validate_manifest(&manifest)?;
        Ok(manifest)
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
        let artifact = version_entry
            .artifacts
            .get(current_platform()?)
            .ok_or(CatalogueError::NoPlatformMatch)?;

        Ok(VersionUrls {
            url: artifact.url.clone(),
            checksum: artifact.sha256.clone(),
            archive_type: version_entry.archive_type.clone(),
            bin_dir: version_entry.bin_dir.clone(),
        })
    }
}

fn current_platform() -> Result<&'static str, CatalogueError> {
    if cfg!(target_os = "windows") {
        Ok("windows-amd64")
    } else if cfg!(target_os = "linux") {
        Ok("linux-amd64")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("darwin-arm64")
    } else {
        Err(CatalogueError::NoPlatformMatch)
    }
}

fn validate_manifest(manifest: &CatalogueManifest) -> Result<(), CatalogueError> {
    if manifest.schema_version != CATALOGUE_SCHEMA_VERSION {
        return Err(CatalogueError::Network(format!(
            "Unsupported catalogue schema {}",
            manifest.schema_version
        )));
    }
    if manifest.generated_at.is_empty() {
        return Err(CatalogueError::Network(
            "Catalogue generated_at is empty".into(),
        ));
    }

    for (runtime, entry) in &manifest.runtimes {
        if runtime.is_empty()
            || entry.provider.is_empty()
            || entry.display_name.is_empty()
            || entry.versions.is_empty()
        {
            return Err(CatalogueError::Network(
                "Catalogue runtime metadata is incomplete".into(),
            ));
        }
        require_https(&entry.homepage)?;
        for (version_name, version) in &entry.versions {
            if version_name.is_empty() || version.artifacts.is_empty() {
                return Err(CatalogueError::Network(
                    "Catalogue version metadata is incomplete".into(),
                ));
            }
            if !matches!(version.archive_type.as_str(), "zip" | "tar.gz") {
                return Err(CatalogueError::Network("Unsupported archive type".into()));
            }
            if version
                .bin_dir
                .as_deref()
                .is_some_and(is_unsafe_relative_path)
            {
                return Err(CatalogueError::Network("Unsafe catalogue bin_dir".into()));
            }
            for artifact in version.artifacts.values() {
                require_https(&artifact.url)?;
                require_https(&artifact.provenance_url)?;
                if artifact.sha256.len() != 64
                    || !artifact
                        .sha256
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err(CatalogueError::Network("Invalid artifact SHA-256".into()));
                }
            }
        }
    }
    Ok(())
}

fn require_https(value: &str) -> Result<(), CatalogueError> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| CatalogueError::Network("Invalid catalogue URL".into()))?;
    if url.scheme() != "https" {
        return Err(CatalogueError::Network(
            "Catalogue URLs must use HTTPS".into(),
        ));
    }
    Ok(())
}

fn is_unsafe_relative_path(value: &str) -> bool {
    value.is_empty()
        || value.contains(':')
        || value
            .replace('\\', "/")
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
        || Path::new(value)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
}

fn decode_signature(value: &str) -> Result<Vec<u8>, CatalogueError> {
    let value = value.trim();
    if value.len() != 128 {
        return Err(CatalogueError::Network(
            "Catalogue signature must be 64-byte hex".into(),
        ));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).expect("hex bytes are ASCII-sized");
            u8::from_str_radix(pair, 16)
                .map_err(|_| CatalogueError::Network("Invalid catalogue signature hex".into()))
        })
        .collect()
}

fn signature_path(manifest_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sig", manifest_path.display()))
}

async fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), CatalogueError> {
    let temporary = PathBuf::from(format!("{}.{}.tmp", path.display(), uuid::Uuid::new_v4()));
    let mut file = tokio::fs::File::create(&temporary)
        .await
        .map_err(|error| CatalogueError::Network(error.to_string()))?;
    file.write_all(contents)
        .await
        .map_err(|error| CatalogueError::Network(error.to_string()))?;
    file.sync_all()
        .await
        .map_err(|error| CatalogueError::Network(error.to_string()))?;
    drop(file);
    replace_file(temporary, path.to_owned()).await
}

#[cfg(windows)]
async fn replace_file(source: PathBuf, destination: PathBuf) -> Result<(), CatalogueError> {
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
        let result = unsafe {
            MoveFileExW(
                source.as_ptr(),
                destination.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if result == 0 {
            return Err(CatalogueError::Network(
                std::io::Error::last_os_error().to_string(),
            ));
        }
        Ok(())
    })
    .await
    .map_err(|error| CatalogueError::Network(error.to_string()))?
}

#[cfg(not(windows))]
async fn replace_file(source: PathBuf, destination: PathBuf) -> Result<(), CatalogueError> {
    tokio::fs::rename(source, destination)
        .await
        .map_err(|error| CatalogueError::Network(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use futou_ipc::catalogue::{
        ArtifactEntry, CatalogueEntry, HashMethod, TrustLevel, VersionEntry,
    };
    use std::collections::HashMap;

    fn signed_manifest() -> (RemoteCatalogueSource, Vec<u8>, String) {
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let source = RemoteCatalogueSource::new(
            "https://example.com/catalogue.json".into(),
            PathBuf::new(),
            PathBuf::new(),
            signing_key.verifying_key().to_bytes(),
        );
        let manifest = CatalogueManifest {
            schema_version: CATALOGUE_SCHEMA_VERSION,
            generated_at: "2026-07-11T00:00:00Z".into(),
            runtimes: HashMap::from([(
                "nodejs".into(),
                CatalogueEntry {
                    display_name: "Node.js".into(),
                    description: "JavaScript runtime".into(),
                    provider: "OpenJS Foundation".into(),
                    homepage: "https://nodejs.org".into(),
                    trust_level: TrustLevel::Official,
                    versions: HashMap::from([(
                        "24.0.0".into(),
                        VersionEntry {
                            archive_type: "zip".into(),
                            bin_dir: Some("node-v24-win-x64".into()),
                            artifacts: HashMap::from([(
                                "windows-amd64".into(),
                                ArtifactEntry {
                                    url: "https://nodejs.org/node.zip".into(),
                                    sha256: "a".repeat(64),
                                    provenance_url: "https://nodejs.org/checksums".into(),
                                    hash_method: HashMethod::Publisher,
                                },
                            )]),
                        },
                    )]),
                },
            )]),
        };
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let signature = signing_key.sign(&bytes);
        let signature_hex = signature
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        (source, bytes, signature_hex)
    }

    #[test]
    fn verifies_signed_manifest_and_rejects_tampering() {
        let (source, bytes, signature) = signed_manifest();
        assert!(source.verify_manifest(&bytes, &signature).is_ok());

        let mut tampered = bytes;
        tampered[0] ^= 1;
        assert!(source.verify_manifest(&tampered, &signature).is_err());
    }

    #[test]
    fn rejects_unsafe_or_incomplete_artifacts() {
        let (source, bytes, signature) = signed_manifest();
        let mut manifest: CatalogueManifest = serde_json::from_slice(&bytes).unwrap();
        manifest
            .runtimes
            .get_mut("nodejs")
            .unwrap()
            .versions
            .get_mut("24.0.0")
            .unwrap()
            .bin_dir = Some("../outside".into());
        let unsafe_bytes = serde_json::to_vec(&manifest).unwrap();
        assert!(source.verify_manifest(&unsafe_bytes, &signature).is_err());

        {
            let version = manifest
                .runtimes
                .get_mut("nodejs")
                .unwrap()
                .versions
                .get_mut("24.0.0")
                .unwrap();
            version.bin_dir = None;
            version
                .artifacts
                .get_mut("windows-amd64")
                .unwrap()
                .sha256
                .clear();
        }
        assert!(validate_manifest(&manifest).is_err());

        manifest
            .runtimes
            .get_mut("nodejs")
            .unwrap()
            .versions
            .get_mut("24.0.0")
            .unwrap()
            .artifacts
            .get_mut("windows-amd64")
            .unwrap()
            .sha256 = "A".repeat(64);
        assert!(validate_manifest(&manifest).is_err());
    }
}
