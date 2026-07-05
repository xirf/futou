use futou_core::ports::downloader::{DownloadError, Downloader};
use std::path::Path;

pub struct NullDownloader;

#[async_trait::async_trait]
impl Downloader for NullDownloader {
    async fn download(
        &self,
        _url: &str,
        _dest: &Path,
        _progress: Box<dyn Fn(f64, String) + Send + Sync>,
    ) -> Result<(), DownloadError> {
        Err(DownloadError::Io("aria2c not installed".to_string()))
    }
}
