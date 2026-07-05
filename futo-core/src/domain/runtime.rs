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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_installation(runtime: &str, version: &str) -> Installation {
        Installation {
            runtime: RuntimeName(runtime.into()),
            version: Version(version.into()),
            status: InstallStatus::Installed,
            path: format!("C:\\runtimes\\{}\\{}", runtime, version),
            version_dir: format!("C:\\runtimes\\{}\\{}", runtime, version),
            installed_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn empty_state_has_no_installations() {
        let state = DaemonState::new();
        assert!(state.installations.is_empty());
        assert!(state.active.is_empty());
        assert!(state.pids.is_empty());
    }

    #[test]
    fn find_installation_by_runtime_and_version() {
        let mut state = DaemonState::new();
        state
            .installations
            .push(make_installation("nodejs", "20.0.0"));
        state
            .installations
            .push(make_installation("nodejs", "22.0.0"));
        state
            .installations
            .push(make_installation("python", "3.12.0"));

        let found =
            state.find_installation(&RuntimeName("nodejs".into()), &Version("20.0.0".into()));
        assert!(found.is_some());
        assert_eq!(found.unwrap().version, Version("20.0.0".into()));

        let not_found =
            state.find_installation(&RuntimeName("nodejs".into()), &Version("21.0.0".into()));
        assert!(not_found.is_none());
    }

    #[test]
    fn find_installation_mut_modifies_in_place() {
        let mut state = DaemonState::new();
        state
            .installations
            .push(make_installation("rust", "1.80.0"));

        let inst = state
            .find_installation_mut(&RuntimeName("rust".into()), &Version("1.80.0".into()))
            .unwrap();
        inst.status = InstallStatus::Active;

        assert_eq!(state.installations[0].status, InstallStatus::Active);
    }

    #[test]
    fn list_installed_all_and_filtered() {
        let mut state = DaemonState::new();
        state.installations.push(make_installation("php", "8.3.0"));
        state.installations.push(make_installation("php", "8.4.0"));
        state.installations.push(make_installation("deno", "2.0.0"));

        let all = state.list_installed(None);
        assert_eq!(all.len(), 3);

        let php_only = state.list_installed(Some(&RuntimeName("php".into())));
        assert_eq!(php_only.len(), 2);
    }

    #[test]
    fn set_get_remove_active() {
        let mut state = DaemonState::new();
        assert!(state.active_version(&RuntimeName("php".into())).is_none());

        state.set_active(&RuntimeName("php".into()), &Version("8.4.0".into()));
        assert_eq!(
            state.active_version(&RuntimeName("php".into())),
            Some("8.4.0")
        );

        state.remove_active(&RuntimeName("php".into()));
        assert!(state.active_version(&RuntimeName("php".into())).is_none());
    }

    #[test]
    fn set_get_remove_pid() {
        let mut state = DaemonState::new();
        assert!(state.get_pid(&RuntimeName("mariadb".into())).is_none());

        state.set_pid(&RuntimeName("mariadb".into()), 9042);
        assert_eq!(state.get_pid(&RuntimeName("mariadb".into())), Some(9042));

        state.remove_pid(&RuntimeName("mariadb".into()));
        assert!(state.get_pid(&RuntimeName("mariadb".into())).is_none());
    }

    #[test]
    fn daemon_state_json_roundtrip() {
        let mut state = DaemonState::new();
        state
            .installations
            .push(make_installation("python", "3.13.0"));
        state.set_active(&RuntimeName("python".into()), &Version("3.13.0".into()));
        state.set_pid(&RuntimeName("postgresql".into()), 5432);

        let json = serde_json::to_string_pretty(&state).unwrap();
        let restored: DaemonState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.installations.len(), 1);
        assert_eq!(
            restored.installations[0].runtime,
            RuntimeName("python".into())
        );
        assert_eq!(restored.active.get("python").unwrap(), "3.13.0");
        assert_eq!(restored.pids.get("postgresql"), Some(&5432));
    }

    #[test]
    fn old_state_without_pids_field_deserializes() {
        let old_json = r#"{
            "installations": [],
            "active": {}
        }"#;
        let state: DaemonState = serde_json::from_str(old_json).unwrap();
        assert!(state.pids.is_empty());
        assert!(state.installations.is_empty());
    }

    #[test]
    fn installation_without_version_dir_defaults_to_empty() {
        let old_json = r#"{
            "runtime": "php",
            "version": "8.3.0",
            "status": "Installed",
            "path": "/opt/php/8.3.0",
            "installed_at": "2026-01-01T00:00:00Z"
        }"#;
        let inst: Installation = serde_json::from_str(old_json).unwrap();
        assert_eq!(inst.version_dir, "");
    }

    #[test]
    fn installation_with_version_dir_deserializes() {
        let json = r#"{
            "runtime": "nodejs",
            "version": "22.0.0",
            "status": "Active",
            "path": "/opt/nodejs/22.0.0",
            "version_dir": "/opt/nodejs/22.0.0",
            "installed_at": "2026-01-01T00:00:00Z"
        }"#;
        let inst: Installation = serde_json::from_str(json).unwrap();
        assert_eq!(inst.version_dir, "/opt/nodejs/22.0.0");
        assert_eq!(inst.status, InstallStatus::Active);
    }

    #[test]
    fn runtime_name_display() {
        assert_eq!(RuntimeName("python".into()).to_string(), "python");
    }

    #[test]
    fn runtime_name_from_str() {
        let r: RuntimeName = "deno".into();
        assert_eq!(r, RuntimeName("deno".into()));
    }

    #[test]
    fn version_display() {
        assert_eq!(Version("3.13.3".into()).to_string(), "3.13.3");
    }

    #[test]
    fn version_from_str() {
        let v: Version = "2.3.8".into();
        assert_eq!(v, Version("2.3.8".into()));
    }

    #[test]
    fn install_status_error_variant() {
        let err = InstallStatus::Error("disk full".into());
        let json = serde_json::to_string(&err).unwrap();
        let restored: InstallStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, InstallStatus::Error("disk full".into()));
    }

    #[test]
    fn daemon_state_default_equals_new() {
        let d1 = DaemonState::default();
        let d2 = DaemonState::new();
        assert!(d1.installations.is_empty());
        assert!(d2.installations.is_empty());
    }

    #[test]
    fn overwrite_active_version() {
        let mut state = DaemonState::new();
        state.set_active(&RuntimeName("php".into()), &Version("8.3.0".into()));
        state.set_active(&RuntimeName("php".into()), &Version("8.4.0".into()));
        assert_eq!(
            state.active_version(&RuntimeName("php".into())),
            Some("8.4.0")
        );
    }

    #[test]
    fn overwrite_pid() {
        let mut state = DaemonState::new();
        state.set_pid(&RuntimeName("postgresql".into()), 5432);
        state.set_pid(&RuntimeName("postgresql".into()), 5433);
        assert_eq!(state.get_pid(&RuntimeName("postgresql".into())), Some(5433));
    }
}
