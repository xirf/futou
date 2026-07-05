use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogueManifest {
    pub runtimes: HashMap<String, CatalogueEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogueEntry {
    pub display_name: String,
    pub description: String,
    pub versions: HashMap<String, VersionEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionEntry {
    pub url: HashMap<String, String>,
    pub checksum: HashMap<String, String>,
    pub archive_type: String,
    pub bin_dir: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BundledCatalogue {
    pub runtimes: HashMap<String, BundledRuntimeEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BundledRuntimeEntry {
    pub display_name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub default_version: String,
    pub version_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub data_dir: Option<String>,
    pub daemon_port: Option<u16>,
}
