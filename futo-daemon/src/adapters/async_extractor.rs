use std::path::Path;

use futou_core::ports::extractor::{ExtractError, Extractor};

pub struct AsyncExtractor;

#[async_trait::async_trait]
impl Extractor for AsyncExtractor {
    async fn extract(&self, archive: &Path, dest: &Path, archive_type: &str) -> Result<(), ExtractError> {
        tokio::task::spawn_blocking({
            let archive = archive.to_path_buf();
            let dest = dest.to_path_buf();
            let archive_type = archive_type.to_string();
            move || {
                match archive_type.as_str() {
                    "zip" => extract_zip(&archive, &dest),
                    "tar.gz" | "tgz" => extract_tar_gz(&archive, &dest),
                    other => Err(ExtractError::UnsupportedType(other.to_string())),
                }
            }
        }).await
            .map_err(|e| ExtractError::Other(e.to_string()))?
    }
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), ExtractError> {
    let file = std::fs::File::open(archive)
        .map_err(|e| ExtractError::Io(e.to_string()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| ExtractError::Other(e.to_string()))?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)
            .map_err(|e| ExtractError::Other(e.to_string()))?;
        let entry_path = entry.mangled_name();
        let full_path = dest.join(&entry_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&full_path)
                .map_err(|e| ExtractError::Io(e.to_string()))?;
        } else {
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ExtractError::Io(e.to_string()))?;
            }
            let mut out = std::fs::File::create(&full_path)
                .map_err(|e| ExtractError::Io(e.to_string()))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| ExtractError::Io(e.to_string()))?;
        }
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), ExtractError> {
    let file = std::fs::File::open(archive)
        .map_err(|e| ExtractError::Io(e.to_string()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    tar.unpack(dest)
        .map_err(|e| ExtractError::Io(e.to_string()))?;
    Ok(())
}
