use std::sync::Arc;

use futou_ipc::messages::CatalogueRuntime;

use crate::ports::catalogue_source::CatalogueSource;

pub struct CatalogueService {
    source: Arc<dyn CatalogueSource>,
}

impl CatalogueService {
    pub fn new(source: Arc<dyn CatalogueSource>) -> Self {
        Self { source }
    }

    pub async fn list_runtimes(&self) -> Result<Vec<CatalogueRuntime>, CatalogueServiceError> {
        let manifest = self.source.fetch().await?;
        let mut runtimes: Vec<CatalogueRuntime> = manifest
            .runtimes
            .into_iter()
            .map(|(name, entry)| {
                let mut versions: Vec<String> = entry.versions.keys().cloned().collect();
                versions.sort_by(|a, b| {
                    let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
                    let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
                    for i in 0..std::cmp::max(3, std::cmp::max(a_parts.len(), b_parts.len())) {
                        let av = a_parts.get(i).copied().unwrap_or(0);
                        let bv = b_parts.get(i).copied().unwrap_or(0);
                        if bv != av {
                            return bv.cmp(&av);
                        }
                    }
                    std::cmp::Ordering::Equal
                });
                CatalogueRuntime {
                    name,
                    display_name: entry.display_name,
                    versions,
                }
            })
            .collect();
        runtimes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(runtimes)
    }

    pub async fn refresh(&self) -> Result<bool, CatalogueServiceError> {
        let _ = self.source.fetch().await?;
        Ok(true)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogueServiceError {
    #[error("Catalogue error: {0}")]
    Source(#[from] crate::ports::catalogue_source::CatalogueError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::catalogue_source::{CatalogueError, CatalogueSource, VersionUrls};
    use futou_ipc::catalogue::{CatalogueEntry, CatalogueManifest, VersionEntry};
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockCatalogue {
        manifest: CatalogueManifest,
    }

    #[async_trait::async_trait]
    impl CatalogueSource for MockCatalogue {
        async fn fetch(&self) -> Result<CatalogueManifest, CatalogueError> {
            Ok(self.manifest.clone())
        }

        async fn fetch_version_urls(
            &self,
            _runtime: &str,
            _version: &str,
        ) -> Result<VersionUrls, CatalogueError> {
            unimplemented!()
        }
    }

    fn version_entry(archive_type: &str) -> VersionEntry {
        VersionEntry {
            url: HashMap::new(),
            checksum: HashMap::new(),
            archive_type: archive_type.into(),
            bin_dir: None,
        }
    }

    fn catalogue_with_versions(versions: &[&str]) -> CatalogueManifest {
        let mut vmap = HashMap::new();
        for v in versions {
            vmap.insert(v.to_string(), version_entry("zip"));
        }
        CatalogueManifest {
            runtimes: HashMap::from([(
                "test".into(),
                CatalogueEntry {
                    display_name: "Test Runtime".into(),
                    description: "A test runtime".into(),
                    versions: vmap,
                },
            )]),
        }
    }

    #[tokio::test]
    async fn empty_catalogue_returns_empty_list() {
        let source = Arc::new(MockCatalogue {
            manifest: CatalogueManifest {
                runtimes: HashMap::new(),
            },
        });
        let svc = CatalogueService::new(source);
        let runtimes = svc.list_runtimes().await.unwrap();
        assert!(runtimes.is_empty());
    }

    #[tokio::test]
    async fn versions_sorted_descending() {
        let source = Arc::new(MockCatalogue {
            manifest: catalogue_with_versions(&["1.0.0", "2.0.0", "1.5.0", "2.1.3", "2.1.10"]),
        });
        let svc = CatalogueService::new(source);
        let runtimes = svc.list_runtimes().await.unwrap();
        assert_eq!(runtimes.len(), 1);
        assert_eq!(
            runtimes[0].versions,
            vec!["2.1.10", "2.1.3", "2.0.0", "1.5.0", "1.0.0"]
        );
    }

    #[tokio::test]
    async fn runtimes_sorted_by_name() {
        let runtimes = HashMap::from([
            (
                "zzz".into(),
                CatalogueEntry {
                    display_name: "Z Runtime".into(),
                    description: "".into(),
                    versions: HashMap::from([("1.0.0".into(), version_entry("zip"))]),
                },
            ),
            (
                "aaa".into(),
                CatalogueEntry {
                    display_name: "A Runtime".into(),
                    description: "".into(),
                    versions: HashMap::from([("2.0.0".into(), version_entry("zip"))]),
                },
            ),
        ]);
        let source = Arc::new(MockCatalogue {
            manifest: CatalogueManifest { runtimes },
        });
        let svc = CatalogueService::new(source);
        let result = svc.list_runtimes().await.unwrap();
        assert_eq!(result[0].name, "aaa");
        assert_eq!(result[1].name, "zzz");
    }

    #[tokio::test]
    async fn version_sort_handles_two_segment_versions() {
        let source = Arc::new(MockCatalogue {
            manifest: catalogue_with_versions(&["2.3", "2.10", "1.9", "2.3.1"]),
        });
        let svc = CatalogueService::new(source);
        let runtimes = svc.list_runtimes().await.unwrap();
        assert_eq!(runtimes[0].versions[0], "2.10");
        assert!(runtimes[0].versions[1] == "2.3.1" || runtimes[0].versions[2] == "2.3.1");
    }

    #[tokio::test]
    async fn refresh_returns_true() {
        let source = Arc::new(MockCatalogue {
            manifest: catalogue_with_versions(&["1.0.0"]),
        });
        let svc = CatalogueService::new(source);
        assert!(svc.refresh().await.unwrap());
    }
}
