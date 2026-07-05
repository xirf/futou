use std::path::{Path, PathBuf};
use std::process::Command as SyncCommand;

use futou_core::ports::process_manager::{ProcessError, ProcessManager};
use tracing::{info, warn};

pub struct WindowsProcessManager;

fn find_exe(bin_dir: &Path, names: &[&str]) -> Option<PathBuf> {
    for name in names {
        let p = bin_dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn data_initialized(data_dir: &Path) -> bool {
    data_dir.exists() && data_dir.join("mysql").exists()
        || data_dir.join("ibdata1").exists()
        || data_dir.join("PG_VERSION").exists()
}

impl WindowsProcessManager {
    fn default_port(runtime: &str) -> Option<u16> {
        match runtime {
            "mariadb" | "mysql" => Some(3306),
            "postgresql" => Some(5432),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl ProcessManager for WindowsProcessManager {
    async fn start_server(
        &self,
        runtime: &str,
        bin_dir: &Path,
        data_dir: &Path,
    ) -> Result<u32, ProcessError> {
        let port = Self::default_port(runtime)
            .ok_or_else(|| ProcessError::NotServer(runtime.to_string()))?;

        match runtime {
            "mariadb" | "mysql" => {
                let exe = find_exe(bin_dir, &["mariadbd.exe", "mysqld.exe"]).ok_or_else(|| {
                    ProcessError::Io(format!("mariadbd not found in {:?}", bin_dir))
                })?;

                info!(
                    "Starting MariaDB: {:?} --datadir={:?} --port={}",
                    exe, data_dir, port
                );
                let child = tokio::process::Command::new(&exe)
                    .arg(format!("--datadir={}", data_dir.display()))
                    .arg(format!("--port={}", port))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map_err(|e| ProcessError::Io(format!("spawn mariadbd: {}", e)))?;

                let pid = child.id().unwrap_or(0);
                Ok(pid)
            }
            "postgresql" => {
                let pg_ctl = bin_dir.join("pg_ctl.exe");
                if !pg_ctl.exists() {
                    return Err(ProcessError::Io(format!(
                        "pg_ctl not found in {:?}",
                        bin_dir
                    )));
                }

                info!(
                    "Starting PostgreSQL: {:?} -D {:?} -p {}",
                    pg_ctl, data_dir, port
                );
                let child = tokio::process::Command::new(&pg_ctl)
                    .arg("start")
                    .arg("-D")
                    .arg(data_dir)
                    .arg("-l")
                    .arg(data_dir.join("pg.log"))
                    .arg("-o")
                    .arg(format!("-p {}", port))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map_err(|e| ProcessError::Io(format!("spawn pg_ctl: {}", e)))?;

                let pid = child.id().unwrap_or(0);
                Ok(pid)
            }
            _ => Err(ProcessError::NotServer(runtime.to_string())),
        }
    }

    async fn stop_server(&self, pid: u32) -> Result<(), ProcessError> {
        if pid == 0 {
            return Ok(());
        }
        info!("Stopping process PID {}", pid);
        let output = SyncCommand::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .output()
            .map_err(|e| ProcessError::Io(format!("taskkill: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("taskkill PID {} returned non-zero: {}", pid, stderr.trim());
        }
        Ok(())
    }

    async fn init_data_dir(
        &self,
        runtime: &str,
        bin_dir: &Path,
        data_dir: &Path,
    ) -> Result<(), ProcessError> {
        if data_initialized(data_dir) {
            info!("Data dir {:?} already initialized, skipping", data_dir);
            return Ok(());
        }

        let _ = std::fs::create_dir_all(data_dir);

        match runtime {
            "mariadb" | "mysql" => {
                let init_exe =
                    find_exe(bin_dir, &["mariadb-install-db.exe", "mysql_install_db.exe"])
                        .ok_or_else(|| {
                            ProcessError::Io(format!("mysql_install_db not found in {:?}", bin_dir))
                        })?;

                info!(
                    "Initializing MariaDB data dir: {:?} --datadir={:?}",
                    init_exe, data_dir
                );
                let output = tokio::process::Command::new(&init_exe)
                    .arg(format!("--datadir={}", data_dir.display()))
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| ProcessError::Io(format!("init: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(ProcessError::ExitError(format!(
                        "mysql_install_db: {}",
                        stderr
                    )));
                }
            }
            "postgresql" => {
                let initdb = bin_dir.join("initdb.exe");
                if !initdb.exists() {
                    return Err(ProcessError::Io(format!(
                        "initdb not found in {:?}",
                        bin_dir
                    )));
                }

                info!(
                    "Initializing PostgreSQL data dir: {:?} -D {:?}",
                    initdb, data_dir
                );
                let output = tokio::process::Command::new(&initdb)
                    .arg("-D")
                    .arg(data_dir)
                    .arg("--no-locale")
                    .arg("-E")
                    .arg("UTF8")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .await
                    .map_err(|e| ProcessError::Io(format!("initdb: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(ProcessError::ExitError(format!("initdb: {}", stderr)));
                }
            }
            _ => return Err(ProcessError::NotServer(runtime.to_string())),
        }

        info!("Data dir initialized: {:?}", data_dir);
        Ok(())
    }
}
