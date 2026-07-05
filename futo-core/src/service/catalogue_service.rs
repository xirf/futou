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
