use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use futou_core::ports::catalogue_source::CatalogueSource;
use futou_core::ports::downloader::Downloader;
use futou_core::ports::extractor::Extractor;
use futou_core::ports::path_manager::PathManager;
use futou_core::ports::process_manager::ProcessManager;
use futou_core::ports::runtime_repo::RuntimeRepository;
use futou_core::ports::shim_manager::ShimManager;
use futou_core::service::activation_service::ActivationService;
use futou_core::service::catalogue_service::CatalogueService;
use futou_core::service::env_service::EnvService;
use futou_core::service::install_service::InstallService;
use sha2::{Digest, Sha256};

use crate::adapters::aria2_downloader::Aria2Downloader;
use crate::adapters::async_extractor::AsyncExtractor;
use crate::adapters::fs_repository::FsRepository;
use crate::adapters::null_downloader::NullDownloader;
use crate::adapters::path_manager_win::WindowsPathManager;
use crate::adapters::process_manager_win::WindowsProcessManager;
use crate::adapters::remote_catalogue::RemoteCatalogueSource;
use crate::adapters::shim_manager_win::WindowsShimManager;
use crate::operation_log::OperationLog;
use std::env;
use tracing::warn;

fn catalogue_public_key() -> anyhow::Result<[u8; 32]> {
    let hex = include_str!("../resources/catalogue-public-key.hex").trim();
    anyhow::ensure!(hex.len() == 64, "catalogue public key must be 32-byte hex");
    let mut key = [0u8; 32];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        key[index] = u8::from_str_radix(std::str::from_utf8(pair)?, 16)?;
    }
    Ok(key)
}

pub struct AppContext {
    pub catalogue_service: CatalogueService,
    pub install_service: InstallService,
    pub activation_service: ActivationService,
    pub env_service: EnvService,
    pub downloader: Arc<dyn Downloader>,
    pub shutdown_tx: tokio::sync::broadcast::Sender<()>,
    pub download_progress: std::sync::Arc<
        std::sync::RwLock<std::collections::HashMap<String, futou_ipc::messages::InstallProgress>>,
    >,
    pub operation_log: std::sync::Arc<OperationLog>,
    pub started_at: Instant,
    pub aria2_available: bool,
}

async fn resolve_aria2c(data_dir: &Path) -> PathBuf {
    const ARIA2_SHA256: &str = "67d015301eef0b612191212d564c5bb0a14b5b9c4796b76454276a4d28d9b288";
    let bundled = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("aria2c")))
        .filter(|p| p.exists());

    if let Some(p) = bundled {
        warn!("aria2c bundled with binary at {:?}", p);
        return p;
    }

    if let Ok(p) = which::which("aria2c") {
        warn!("aria2c found in PATH at {:?}", p);
        return PathBuf::from("aria2c");
    }

    let data_aria2 = data_dir.join("aria2").join("aria2c.exe");
    if data_aria2.exists() {
        warn!("aria2c found in data dir at {:?}", data_aria2);
        return data_aria2;
    }

    warn!("aria2c not found anywhere. Downloading from GitHub...");
    let download_url = "https://github.com/aria2/aria2/releases/download/release-1.37.0/aria2-1.37.0-win-64bit-build1.zip";
    let zip_path = data_dir.join("aria2").join("aria2.zip");

    let _ = tokio::fs::create_dir_all(data_dir.join("aria2")).await;
    match download_aria2(download_url, &zip_path, ARIA2_SHA256).await {
        Ok(()) => {
            if extract_zip(&zip_path, &data_dir.join("aria2")).is_ok() {
                let _ = tokio::fs::remove_file(&zip_path).await;
                let exe = find_aria2_exe(&data_dir.join("aria2"));
                if let Some(p) = exe {
                    let _ = tokio::fs::copy(&p, &data_aria2).await;
                    warn!("aria2c downloaded to {:?}", data_aria2);
                    return data_aria2;
                }
            }
        }
        Err(e) => warn!("Failed to download aria2c: {}", e),
    }

    PathBuf::from("aria2c")
}

async fn download_aria2(url: &str, dest: &Path, expected_sha256: &str) -> Result<(), String> {
    let resp = reqwest::get(url)
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| format!("HTTP error: {}", e))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Read error: {}", e))?;
    let actual_sha256 = format!("{:x}", Sha256::digest(&bytes));
    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "aria2 checksum mismatch: expected {expected_sha256}, got {actual_sha256}"
        ));
    }
    tokio::fs::write(dest, &bytes)
        .await
        .map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

fn extract_zip(zip_path: &PathBuf, out_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(path) = entry.enclosed_name() else {
            continue;
        };
        let target = out_dir.join(path);

        if entry.is_dir() {
            let _ = std::fs::create_dir_all(&target);
        } else {
            let _ = std::fs::create_dir_all(target.parent().unwrap());
            let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn find_aria2_exe(dir: &PathBuf) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let inner = entry.path().join("aria2c.exe");
            if inner.exists() {
                return Some(inner);
            }
            // recurse one level
            if let Some(found) = find_aria2_exe(&entry.path()) {
                return Some(found);
            }
        }
        if entry.file_name().to_string_lossy() == "aria2c.exe" {
            return Some(entry.path());
        }
    }
    None
}

async fn cleanup_old_aria2(pid_path: &PathBuf) {
    if let Ok(pid_str) = tokio::fs::read_to_string(pid_path).await {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output();
        }
    }
    let _ = std::fs::remove_file(pid_path);
}

pub async fn build_composition_root(data_dir: &Path) -> anyhow::Result<AppContext> {
    let runtimes_dir = data_dir.join("runtimes");
    let download_dir = data_dir.join("aria2").join("downloads");
    let cache_dir = data_dir.join("catalogue");
    let shim_dir = data_dir.join("shims");

    tokio::fs::create_dir_all(&runtimes_dir).await?;
    tokio::fs::create_dir_all(&download_dir).await?;
    tokio::fs::create_dir_all(&cache_dir).await?;
    tokio::fs::create_dir_all(&shim_dir).await?;

    let state_path = data_dir.join("state.json");
    let repository = Arc::new(FsRepository::new(state_path));

    let bundle_path = data_dir.join("catalogue").join("bundle.json");
    let bundle_signature_path = data_dir.join("catalogue").join("bundle.json.sig");
    for (path, contents) in [
        (
            &bundle_path,
            include_bytes!("../resources/bundle.json").as_slice(),
        ),
        (
            &bundle_signature_path,
            include_bytes!("../resources/bundle.json.sig").as_slice(),
        ),
    ] {
        if !matches!(tokio::fs::read(path).await.as_deref(), Ok(existing) if existing == contents) {
            tokio::fs::write(path, contents).await?;
        }
    }

    let remote_url =
        "https://raw.githubusercontent.com/xirf/futou/master/catalogue.json".to_string();
    let catalogue_source: Arc<dyn CatalogueSource> = Arc::new(RemoteCatalogueSource::new(
        remote_url,
        cache_dir,
        data_dir.join("catalogue"),
        catalogue_public_key()?,
    ));

    let aria2_path = resolve_aria2c(data_dir).await;

    let aria2_pid_path = data_dir.join("aria2").join("aria2.pid");
    cleanup_old_aria2(&aria2_pid_path).await;

    let mut aria2_available = false;
    let downloader: Arc<dyn Downloader> =
        match Aria2Downloader::spawn(&aria2_path, &download_dir, &aria2_pid_path).await {
            Ok(d) => {
                warn!("aria2 RPC downloader ready (path: {:?})", aria2_path);
                aria2_available = true;
                Arc::new(d)
            }
            Err(e) => {
                warn!(
                    "aria2c not found ({}). Install aria2 to enable downloads.",
                    e
                );
                Arc::new(NullDownloader)
            }
        };

    let extractor: Arc<dyn Extractor> = Arc::new(AsyncExtractor);

    let shim_manager: Arc<dyn ShimManager> = Arc::new(WindowsShimManager::new(shim_dir));
    let path_manager: Arc<dyn PathManager> = Arc::new(WindowsPathManager::new());
    let process_manager: Arc<dyn ProcessManager> = Arc::new(WindowsProcessManager);

    let catalogue_service = CatalogueService::new(catalogue_source.clone());
    let install_service = InstallService::new(
        downloader.clone(),
        extractor,
        repository.clone(),
        catalogue_source,
        shim_manager.clone(),
        runtimes_dir,
    );
    let activation_service = ActivationService::new(
        repository.clone(),
        shim_manager,
        path_manager,
        process_manager,
    );

    // Kill orphaned server processes from a previous daemon run
    if let Ok(state) = repository.load().await {
        for pid in state.pids.values() {
            let _ = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output();
        }
        if !state.pids.is_empty() {
            let _ = repository
                .update_state(Box::new(|state| {
                    state.pids.clear();
                    Ok(())
                }))
                .await;
        }
    }

    let env_service = EnvService::new(repository);

    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

    let operation_log = std::sync::Arc::new(OperationLog::new());
    if !aria2_available {
        operation_log.push(
            "system",
            "error",
            "aria2c not found — installs will fail until aria2 is installed".into(),
        );
    }

    Ok(AppContext {
        catalogue_service,
        install_service,
        activation_service,
        env_service,
        downloader,
        shutdown_tx,
        download_progress: std::sync::Arc::new(std::sync::RwLock::new(
            std::collections::HashMap::new(),
        )),
        operation_log,
        started_at: Instant::now(),
        aria2_available,
    })
}
