use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use futou_core::ports::path_manager::{PathError, PathManager};
use winreg::enums::*;
use winreg::RegKey;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub struct WindowsPathManager;

impl WindowsPathManager {
    pub fn new() -> Self {
        Self
    }

    fn current_path() -> Result<Vec<String>, PathError> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| PathError::Registry(e.to_string()))?;

        let value: String = env.get_value("PATH").unwrap_or_default();
        Ok(value.split(';').map(|s| s.to_string()).collect())
    }

    fn set_path(paths: &[String]) -> Result<(), PathError> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| PathError::Registry(e.to_string()))?;

        let new_path = paths.join(";");
        env.set_value("PATH", &new_path)
            .map_err(|e| PathError::Registry(e.to_string()))?;

        let wide: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                wide.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                5000,
                std::ptr::null_mut(),
            );
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl PathManager for WindowsPathManager {
    async fn add_to_path(&self, dir: &Path) -> Result<(), PathError> {
        let dir_str = dir.to_string_lossy().replace('/', "\\");
        let mut paths = Self::current_path()?;

        if !paths.iter().any(|p| p == &dir_str) {
            paths.push(dir_str);
            Self::set_path(&paths)?;
        }

        Ok(())
    }

    async fn remove_from_path(&self, dir: &Path) -> Result<(), PathError> {
        let dir_str = dir.to_string_lossy().replace('/', "\\");
        let mut paths = Self::current_path()?;
        paths.retain(|p| p != &dir_str);
        Self::set_path(&paths)
    }

    async fn is_in_path(&self, dir: &Path) -> Result<bool, PathError> {
        let dir_str = dir.to_string_lossy().replace('/', "\\");
        let paths = Self::current_path()?;
        Ok(paths.iter().any(|p| p == &dir_str))
    }
}
