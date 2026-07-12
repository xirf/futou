use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use futou_ipc::catalogue::{CatalogueManifest, CATALOGUE_SCHEMA_VERSION};
use rand_core::OsRng;
use std::{env, fs, path::Path};

mod providers;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, path] if command == "validate" => validate_file(Path::new(path)),
        [command, output] if command == "generate" => generate(output),
        [command, input, signature] if command == "sign" => sign(input, signature),
        [command, input, signature, public_key] if command == "verify" => {
            verify(input, signature, public_key)
        }
        [command, public_key] if command == "keygen" => keygen(public_key),
        _ => bail!("usage: catalogue-generator <validate FILE | generate OUTPUT | sign INPUT SIGNATURE | verify INPUT SIGNATURE PUBLIC_KEY | keygen PUBLIC_KEY>"),
    }
}

fn keygen(public_key: &str) -> Result<()> {
    let key = SigningKey::generate(&mut OsRng);
    fs::write(public_key, encode_hex(key.verifying_key().as_bytes()))
        .with_context(|| format!("write {public_key}"))?;
    println!("{}", encode_hex(&key.to_bytes()));
    Ok(())
}

fn read_manifest(path: &Path) -> Result<CatalogueManifest> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn validate_file(path: &Path) -> Result<()> {
    validate(&read_manifest(path)?)
}

fn validate(manifest: &CatalogueManifest) -> Result<()> {
    if manifest.schema_version != CATALOGUE_SCHEMA_VERSION || manifest.runtimes.is_empty() {
        bail!("unsupported or empty catalogue");
    }
    for (runtime, entry) in &manifest.runtimes {
        if runtime.is_empty()
            || entry.versions.is_empty()
            || !entry.homepage.starts_with("https://")
        {
            bail!("runtime {runtime:?} has incomplete metadata");
        }
        for (version, release) in &entry.versions {
            if release.archive_type != "zip" {
                bail!("{runtime} {version}: only zip is supported");
            }
            let artifact = release
                .artifacts
                .get("windows-amd64")
                .context("missing windows-amd64 artifact")?;
            if !artifact.url.starts_with("https://")
                || !artifact.provenance_url.starts_with("https://")
            {
                bail!("{runtime} {version}: artifact and provenance must use HTTPS");
            }
            if artifact.sha256.len() != 64
                || !artifact.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                bail!("{runtime} {version}: sha256 must be a 64-character hex digest");
            }
            if let Some(path) = &release.bin_dir {
                validate_relative_path(runtime, version, path)?;
            }
        }
    }
    Ok(())
}

fn validate_relative_path(runtime: &str, version: &str, path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with(['/', '\\'])
        || path.contains(':')
        || path
            .split(['/', '\\'])
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        bail!("{runtime} {version}: unsafe bin_dir {path:?}");
    }
    Ok(())
}

fn generate(output: &str) -> Result<()> {
    let manifest = providers::discover()?;
    validate(&manifest)?;
    fs::write(output, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write {output}"))
}

fn sign(input: &str, signature: &str) -> Result<()> {
    let secret = decode_hex::<32>(
        &env::var("CATALOGUE_SIGNING_KEY").context("CATALOGUE_SIGNING_KEY is not set")?,
    )?;
    let signature_bytes = SigningKey::from_bytes(&secret)
        .sign(&fs::read(input)?)
        .to_bytes();
    fs::write(signature, encode_hex(&signature_bytes)).with_context(|| format!("write {signature}"))
}

fn verify(input: &str, signature: &str, public_key: &str) -> Result<()> {
    let public_key = decode_hex::<32>(fs::read_to_string(public_key)?.trim())?;
    let signature = decode_hex::<64>(fs::read_to_string(signature)?.trim())?;
    VerifyingKey::from_bytes(&public_key)?
        .verify(&fs::read(input)?, &Signature::from_bytes(&signature))
        .context("catalogue signature verification failed")
}

fn decode_hex<const N: usize>(input: &str) -> Result<[u8; N]> {
    if input.len() != N * 2 {
        bail!("expected {} hexadecimal characters", N * 2);
    }
    let mut bytes = [0; N];
    for (index, pair) in input.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = u8::from_str_radix(std::str::from_utf8(pair)?, 16)?;
    }
    Ok(bytes)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futou_ipc::catalogue::{
        ArtifactEntry, CatalogueEntry, HashMethod, TrustLevel, VersionEntry,
    };
    use std::collections::HashMap;

    fn manifest(checksum: &str, bin_dir: Option<&str>) -> CatalogueManifest {
        CatalogueManifest {
            schema_version: 2,
            generated_at: "2026-07-11T00:00:00Z".into(),
            runtimes: HashMap::from([(
                "nodejs".into(),
                CatalogueEntry {
                    display_name: "Node.js".into(),
                    description: "JavaScript runtime".into(),
                    provider: "Node.js".into(),
                    homepage: "https://nodejs.org".into(),
                    trust_level: TrustLevel::Official,
                    versions: HashMap::from([(
                        "22.0.0".into(),
                        VersionEntry {
                            archive_type: "zip".into(),
                            bin_dir: bin_dir.map(str::to_owned),
                            artifacts: HashMap::from([(
                                "windows-amd64".into(),
                                ArtifactEntry {
                                    url: "https://example.test/node.zip".into(),
                                    sha256: checksum.into(),
                                    provenance_url: "https://example.test/checksums".into(),
                                    hash_method: HashMethod::Publisher,
                                },
                            )]),
                        },
                    )]),
                },
            )]),
        }
    }

    #[test]
    fn accepts_complete_windows_artifact() {
        assert!(validate(&manifest(&"a".repeat(64), Some("node/bin"))).is_ok());
    }

    #[test]
    fn rejects_missing_digest_and_traversal() {
        assert!(validate(&manifest("", None)).is_err());
        assert!(validate(&manifest(&"a".repeat(64), Some("../bin"))).is_err());
    }
}
