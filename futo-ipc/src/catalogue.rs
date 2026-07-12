use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current signed catalogue schema version.
pub const CATALOGUE_SCHEMA_VERSION: u32 = 2;

/// A signed snapshot of runtime releases.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogueManifest {
    /// Schema version used to reject incompatible snapshots.
    pub schema_version: u32,
    /// RFC 3339 timestamp set by the catalogue generator.
    pub generated_at: String,
    /// Runtime entries keyed by their stable command-line name.
    pub runtimes: HashMap<String, CatalogueEntry>,
}

/// Metadata and releases for one runtime.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatalogueEntry {
    /// Human-readable runtime name.
    pub display_name: String,
    /// Short runtime description.
    pub description: String,
    /// Organization or distribution source that provides the artifacts.
    pub provider: String,
    /// HTTPS project or provider homepage.
    pub homepage: String,
    /// Whether artifacts come from an official or disclosed third-party source.
    pub trust_level: TrustLevel,
    /// Releases keyed by their exact version.
    pub versions: HashMap<String, VersionEntry>,
}

/// Describes the relationship between a runtime and its artifact provider.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// The project or its designated publisher distributes the artifact.
    Official,
    /// A disclosed independent distributor provides the artifact.
    ThirdParty,
}

/// Downloadable artifacts for one runtime version.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionEntry {
    /// Archive format understood by the extractor.
    pub archive_type: String,
    /// Optional relative directory containing runtime executables.
    pub bin_dir: Option<String>,
    /// Platform artifacts keyed by Futou platform identifier.
    pub artifacts: HashMap<String, ArtifactEntry>,
}

/// A pinned artifact and the source of its digest.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactEntry {
    /// HTTPS download URL.
    pub url: String,
    /// Lowercase SHA-256 digest of the downloaded bytes.
    pub sha256: String,
    /// Page or sidecar file from which this artifact was discovered.
    pub provenance_url: String,
    /// How the digest was obtained.
    pub hash_method: HashMethod,
}

/// Records whether a digest was publisher-supplied or computed during ingestion.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HashMethod {
    /// The upstream publisher supplied the digest.
    Publisher,
    /// The catalogue pipeline downloaded and hashed the artifact.
    CiComputed,
}

/// Persisted daemon configuration shared by frontends.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    /// Optional daemon data directory override.
    pub data_dir: Option<String>,
    /// Optional legacy daemon port setting.
    pub daemon_port: Option<u16>,
}
