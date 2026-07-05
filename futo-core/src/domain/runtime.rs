use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeName(pub String);

impl From<&str> for RuntimeName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for RuntimeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version(pub String);

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallStatus {
    Pending,
    Downloading,
    Verifying,
    Extracting,
    Installed,
    Active,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub runtime: RuntimeName,
    pub version: Version,
    pub status: InstallStatus,
    pub path: String,
    #[serde(default)]
    pub version_dir: String,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub installations: Vec<Installation>,
    pub active: HashMap<String, String>,
    #[serde(default)]
    pub pids: HashMap<String, u32>,
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonState {
    pub fn new() -> Self {
        Self {
            installations: Vec::new(),
            active: HashMap::new(),
            pids: HashMap::new(),
        }
    }

    pub fn find_installation(
        &self,
        runtime: &RuntimeName,
        version: &Version,
    ) -> Option<&Installation> {
        self.installations
            .iter()
            .find(|i| i.runtime == *runtime && i.version == *version)
    }

    pub fn find_installation_mut(
        &mut self,
        runtime: &RuntimeName,
        version: &Version,
    ) -> Option<&mut Installation> {
        self.installations
            .iter_mut()
            .find(|i| i.runtime == *runtime && i.version == *version)
    }

    pub fn list_installed(&self, runtime: Option<&RuntimeName>) -> Vec<&Installation> {
        match runtime {
            Some(name) => self
                .installations
                .iter()
                .filter(|i| i.runtime == *name)
                .collect(),
            None => self.installations.iter().collect(),
        }
    }

    pub fn active_version(&self, runtime: &RuntimeName) -> Option<&str> {
        self.active.get(&runtime.0).map(|s| s.as_str())
    }

    pub fn set_active(&mut self, runtime: &RuntimeName, version: &Version) {
        self.active.insert(runtime.0.clone(), version.0.clone());
    }

    pub fn remove_active(&mut self, runtime: &RuntimeName) {
        self.active.remove(&runtime.0);
    }

    pub fn set_pid(&mut self, runtime: &RuntimeName, pid: u32) {
        self.pids.insert(runtime.0.clone(), pid);
    }

    pub fn get_pid(&self, runtime: &RuntimeName) -> Option<u32> {
        self.pids.get(&runtime.0).copied()
    }

    pub fn remove_pid(&mut self, runtime: &RuntimeName) {
        self.pids.remove(&runtime.0);
    }
}
