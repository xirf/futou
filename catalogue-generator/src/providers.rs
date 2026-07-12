use anyhow::{bail, Context, Result};
use futou_ipc::catalogue::{
    ArtifactEntry, CatalogueEntry, CatalogueManifest, HashMethod, TrustLevel, VersionEntry,
    CATALOGUE_SCHEMA_VERSION,
};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;

const PHP_INDEX: &str = "https://downloads.php.net/~windows/releases/releases.json";
const NODE_INDEX: &str = "https://nodejs.org/dist/index.json";
const MARIADB_INDEX: &str = "https://downloads.mariadb.org/rest-api/mariadb/";
const DENO_RELEASES: &str = "https://api.github.com/repos/denoland/deno/releases?per_page=10";

pub fn discover() -> Result<CatalogueManifest> {
    let client = Client::builder()
        .user_agent("futou-catalogue-generator")
        .build()?;
    let php: Value = client.get(PHP_INDEX).send()?.error_for_status()?.json()?;
    let node: Value = client.get(NODE_INDEX).send()?.error_for_status()?.json()?;
    let mariadb: Value = client
        .get(MARIADB_INDEX)
        .send()?
        .error_for_status()?
        .json()?;
    let deno: Value = client
        .get(DENO_RELEASES)
        .send()?
        .error_for_status()?
        .json()?;
    let mut runtimes = HashMap::new();
    runtimes.insert("php".into(), parse_php(&php)?);
    runtimes.insert("nodejs".into(), discover_node(&client, &node)?);
    runtimes.insert("mariadb".into(), discover_mariadb(&client, &mariadb)?);
    runtimes.insert("deno".into(), parse_deno(&deno)?);
    merge_pinned(&mut runtimes)?;
    Ok(CatalogueManifest {
        schema_version: CATALOGUE_SCHEMA_VERSION,
        generated_at: chrono::Utc::now().to_rfc3339(),
        runtimes,
    })
}

fn merge_pinned(runtimes: &mut HashMap<String, CatalogueEntry>) -> Result<()> {
    let pinned: CatalogueManifest = serde_json::from_str(include_str!("../providers-pinned.json"))
        .context("parse providers-pinned.json")?;
    if pinned.schema_version != CATALOGUE_SCHEMA_VERSION {
        bail!("pinned providers use an unsupported schema");
    }
    for (name, entry) in pinned.runtimes {
        if runtimes.insert(name.clone(), entry).is_some() {
            bail!("pinned provider duplicates {name}");
        }
    }
    Ok(())
}

fn discover_mariadb(client: &Client, index: &Value) -> Result<CatalogueEntry> {
    let majors = index
        .get("major_releases")
        .and_then(Value::as_array)
        .context("MariaDB index missing major_releases")?;
    let mut versions = HashMap::new();
    for major in majors
        .iter()
        .filter(|release| release.get("release_status").and_then(Value::as_str) == Some("Stable"))
        .take(2)
    {
        let id = major
            .get("release_id")
            .and_then(Value::as_str)
            .context("MariaDB major missing release_id")?;
        let provenance = format!("https://downloads.mariadb.org/rest-api/mariadb/{id}/");
        let release: Value = client.get(&provenance).send()?.error_for_status()?.json()?;
        let (version, artifact) = parse_mariadb_release(&release)?;
        versions.insert(
            version.clone(),
            artifact_release(
                artifact.0,
                &artifact.1,
                provenance,
                Some(&format!("mariadb-{version}-winx64/bin")),
            ),
        );
    }
    if versions.is_empty() {
        bail!("MariaDB index contains no stable Windows ZIP");
    }
    Ok(entry(
        "MariaDB",
        "MariaDB database server",
        "MariaDB Foundation",
        "https://mariadb.org",
        versions,
    ))
}

fn parse_mariadb_release(value: &Value) -> Result<(String, (String, String))> {
    let releases = value
        .get("releases")
        .and_then(Value::as_object)
        .context("MariaDB response missing releases")?;
    let (version, file) = releases
        .iter()
        .filter_map(|(version, release)| {
            release
                .get("files")?
                .as_array()?
                .iter()
                .find(|file| {
                    file.get("file_name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| {
                            name.ends_with("winx64.zip") && !name.contains("debugsymbols")
                        })
                })
                .map(|file| (version, file))
        })
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .context("MariaDB response contains no winx64 ZIP")?;
    let url = file
        .get("file_download_url")
        .and_then(Value::as_str)
        .context("MariaDB file missing download URL")?;
    let sha = file
        .pointer("/checksum/sha256sum")
        .and_then(Value::as_str)
        .context("MariaDB file missing SHA-256")?;
    Ok((
        version.clone(),
        (url.replacen("http://", "https://", 1), sha.into()),
    ))
}

fn parse_deno(releases: &Value) -> Result<CatalogueEntry> {
    let releases = releases
        .as_array()
        .context("Deno releases must be an array")?;
    let mut versions = HashMap::new();
    for release in releases
        .iter()
        .filter(|release| release.get("prerelease") == Some(&Value::Bool(false)))
        .take(2)
    {
        let tag = release
            .get("tag_name")
            .and_then(Value::as_str)
            .context("Deno release missing tag")?;
        let assets = release
            .get("assets")
            .and_then(Value::as_array)
            .context("Deno release missing assets")?;
        let asset = assets
            .iter()
            .find(|asset| {
                asset.get("name").and_then(Value::as_str) == Some("deno-x86_64-pc-windows-msvc.zip")
            })
            .context("Deno release missing Windows x64 ZIP")?;
        let url = asset
            .get("browser_download_url")
            .and_then(Value::as_str)
            .context("Deno asset missing URL")?;
        let digest = asset
            .get("digest")
            .and_then(Value::as_str)
            .and_then(|value| value.strip_prefix("sha256:"))
            .context("Deno asset missing SHA-256 digest")?;
        let provenance = release
            .get("html_url")
            .and_then(Value::as_str)
            .context("Deno release missing provenance URL")?;
        versions.insert(
            tag.trim_start_matches('v').into(),
            artifact_release(url.into(), digest, provenance, None),
        );
    }
    if versions.is_empty() {
        bail!("Deno releases contain no stable Windows ZIP with digest");
    }
    Ok(entry(
        "Deno",
        "JavaScript and TypeScript runtime",
        "Deno Land Inc.",
        "https://deno.com",
        versions,
    ))
}

fn parse_php(index: &Value) -> Result<CatalogueEntry> {
    let families = index.as_object().context("PHP index must be an object")?;
    let mut discovered: Vec<(String, VersionEntry)> = Vec::new();
    for family in families.values() {
        let version = family
            .get("version")
            .and_then(Value::as_str)
            .context("PHP release missing version")?;
        let builds = family
            .as_object()
            .context("PHP release must be an object")?;
        let zip = builds
            .iter()
            .find_map(|(name, value)| {
                (name.contains("nts") && name.ends_with("x64"))
                    .then(|| value.get("zip"))
                    .flatten()
            })
            .context("PHP release missing NTS x64 ZIP")?;
        let path = zip
            .get("path")
            .and_then(Value::as_str)
            .context("PHP ZIP missing path")?;
        let sha256 = zip
            .get("sha256")
            .and_then(Value::as_str)
            .context("PHP ZIP missing sha256")?;
        discovered.push((
            version.into(),
            artifact_release(
                format!("https://downloads.php.net/~windows/releases/{path}"),
                sha256,
                PHP_INDEX,
                None,
            ),
        ));
    }
    discovered.sort_by_key(|(version, _)| std::cmp::Reverse(version_key(version)));
    let versions: HashMap<_, _> = discovered.into_iter().take(4).collect();
    if versions.is_empty() {
        bail!("PHP index contains no releases");
    }
    Ok(entry(
        "PHP",
        "PHP: Hypertext Preprocessor",
        "PHP",
        "https://www.php.net",
        versions,
    ))
}

fn version_key(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse().unwrap_or_default())
        .collect()
}

fn discover_node(client: &Client, index: &Value) -> Result<CatalogueEntry> {
    let releases = index.as_array().context("Node index must be an array")?;
    let mut selected = Vec::new();
    for release in releases {
        let version = release
            .get("version")
            .and_then(Value::as_str)
            .context("Node release missing version")?;
        let is_lts = release.get("lts").is_some_and(|value| value.is_string());
        if selected.is_empty() || (is_lts && !selected.iter().any(|(_, lts)| *lts)) {
            selected.push((version, is_lts));
        }
        if selected.len() == 2 {
            break;
        }
    }
    let mut versions = HashMap::new();
    for (tag, _) in selected {
        let filename = format!("node-{tag}-win-x64.zip");
        let sums_url = format!("https://nodejs.org/dist/{tag}/SHASUMS256.txt");
        let sums = client.get(&sums_url).send()?.error_for_status()?.text()?;
        let sha = parse_node_sums(&sums, &filename)?;
        versions.insert(
            tag.trim_start_matches('v').into(),
            artifact_release(
                format!("https://nodejs.org/dist/{tag}/{filename}"),
                sha,
                sums_url,
                Some(filename.trim_end_matches(".zip")),
            ),
        );
    }
    Ok(entry(
        "Node.js",
        "JavaScript runtime built on V8",
        "OpenJS Foundation",
        "https://nodejs.org",
        versions,
    ))
}

fn parse_node_sums<'a>(sums: &'a str, filename: &str) -> Result<&'a str> {
    sums.lines()
        .find_map(|line| {
            line.split_once("  ")
                .filter(|(_, name)| *name == filename)
                .map(|(sum, _)| sum)
        })
        .context("Node checksum entry missing")
}

fn artifact_release(
    url: String,
    sha: &str,
    provenance: impl Into<String>,
    bin_dir: Option<&str>,
) -> VersionEntry {
    VersionEntry {
        archive_type: "zip".into(),
        bin_dir: bin_dir.map(str::to_owned),
        artifacts: HashMap::from([(
            "windows-amd64".into(),
            ArtifactEntry {
                url,
                sha256: sha.into(),
                provenance_url: provenance.into(),
                hash_method: HashMethod::Publisher,
            },
        )]),
    }
}

fn entry(
    name: &str,
    description: &str,
    provider: &str,
    homepage: &str,
    versions: HashMap<String, VersionEntry>,
) -> CatalogueEntry {
    CatalogueEntry {
        display_name: name.into(),
        description: description.into(),
        provider: provider.into(),
        homepage: homepage.into(),
        trust_level: TrustLevel::Official,
        versions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_php_release_fixture() {
        let fixture = serde_json::json!({"8.4":{"version":"8.4.1","nts-vs17-x64":{"zip":{"path":"php-8.4.1.zip","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}}});
        let entry = parse_php(&fixture).unwrap();
        assert!(entry.versions.contains_key("8.4.1"));
    }

    #[test]
    fn php_keeps_only_the_four_newest_families() {
        let mut fixture = serde_json::Map::new();
        for version in ["7.4.33", "8.0.30", "8.1.34", "8.2.32", "8.3.32", "8.4.23"] {
            fixture.insert(version.into(), serde_json::json!({"version":version,"nts-vs17-x64":{"zip":{"path":format!("php-{version}.zip"),"sha256":"a".repeat(64)}}}));
        }
        let entry = parse_php(&Value::Object(fixture)).unwrap();
        assert_eq!(entry.versions.len(), 4);
        assert!(!entry.versions.contains_key("7.4.33"));
        assert!(!entry.versions.contains_key("8.0.30"));
    }

    #[test]
    fn parses_node_checksum_fixture() {
        let sums = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  node-v22.1.0-win-x64.zip\n";
        assert_eq!(
            parse_node_sums(sums, "node-v22.1.0-win-x64.zip").unwrap(),
            "a".repeat(64)
        );
        assert!(parse_node_sums(sums, "missing.zip").is_err());
    }

    #[test]
    fn parses_mariadb_release_fixture() {
        let fixture = serde_json::json!({"releases":{"11.4.5":{"files":[{"file_name":"mariadb-11.4.5-winx64.zip","file_download_url":"https://archive.mariadb.org/mariadb.zip","checksum":{"sha256sum":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}]}}});
        let (version, (_, sha)) = parse_mariadb_release(&fixture).unwrap();
        assert_eq!(version, "11.4.5");
        assert_eq!(sha.len(), 64);
    }

    #[test]
    fn parses_deno_release_fixture() {
        let fixture = serde_json::json!([{"tag_name":"v2.3.8","prerelease":false,"html_url":"https://github.com/denoland/deno/releases/tag/v2.3.8","assets":[{"name":"deno-x86_64-pc-windows-msvc.zip","browser_download_url":"https://github.com/denoland/deno/releases/download/v2.3.8/deno.zip","digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}]);
        assert!(parse_deno(&fixture).unwrap().versions.contains_key("2.3.8"));
    }

    #[test]
    fn merges_pinned_providers() {
        let mut runtimes = HashMap::new();
        merge_pinned(&mut runtimes).unwrap();
        assert!(runtimes.contains_key("postgresql"));
        assert!(runtimes.contains_key("nginx"));
        assert!(!runtimes.contains_key("apache"));
        let manifest = CatalogueManifest {
            schema_version: CATALOGUE_SCHEMA_VERSION,
            generated_at: "2026-07-11T00:00:00Z".into(),
            runtimes,
        };
        crate::validate(&manifest).unwrap();
    }
}
