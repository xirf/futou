#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    async fn download(
        &self,
        url: &str,
        dest: &std::path::Path,
        progress: Box<dyn Fn(f64, String) + Send + Sync>,
    ) -> Result<(), DownloadError>;
    async fn shutdown(&self) {}
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Download cancelled")]
    Cancelled,
}
