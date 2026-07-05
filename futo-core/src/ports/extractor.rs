#[async_trait::async_trait]
pub trait Extractor: Send + Sync {
    async fn extract(
        &self,
        archive: &std::path::Path,
        dest: &std::path::Path,
        archive_type: &str,
    ) -> Result<(), ExtractError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("Unsupported archive type: {0}")]
    UnsupportedType(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Extraction failed: {0}")]
    Other(String),
}
