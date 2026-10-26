//! The three levels of verification, and the path rules that go with them.
//!
//! 1. The Ed25519 signature over the exact bytes of the manifest, against a
//!    key compiled into this executable. Nothing else is trusted until this
//!    passes, because the GitHub asset digest only proves a file is what
//!    GitHub holds, not who put it there.
//! 2. The SHA-256 of the archive, against the manifest and against the
//!    `digest` field GitHub reports for the asset.
//! 3. The SHA-256 of every file once it has been extracted.
//!
//! Any failure means nothing is installed.

use std::path::{Component, Path, PathBuf};

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::consts::EXPECTED_BUILDID;
use crate::hashes::{digests_equal, sha256_bytes, sha256_file_with_size};
use crate::manifest::keys::{keys_for, TrustedKey};
use crate::manifest::schema::{Manifest, Product, SCHEMA_VERSION};
use crate::version::Version;

/// Everything that can make a release untrustworthy.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VerifyError {
    #[error("this build trusts no signing keys")]
    NoTrustedKeys,

    #[error("the signature file is not valid: {0}")]
    MalformedSignature(String),

    #[error("the signature names key {0:?}, which this build does not trust")]
    UnknownKey(String),

    #[error("key {key:?} is not allowed to sign {product}")]
    KeyNotAllowed { key: String, product: &'static str },

    #[error("the signature does not match the manifest")]
    BadSignature,

    #[error("the manifest is not valid JSON: {0}")]
    MalformedManifest(String),

    #[error("manifest schema {found}, this launcher understands {understood}")]
    UnsupportedSchema { found: u32, understood: u32 },

    #[error("the manifest describes {found} but {expected} was expected")]
    WrongProduct {
        found: &'static str,
        expected: &'static str,
    },

    // The versions are kept as text: this error exists to be shown, and a
    // parsed Version here would make every VerifyError large enough to be
    // worth boxing.
    #[error("this release needs launcher {needed} or newer, this is {running}")]
    LauncherTooOld { needed: String, running: String },

    #[error("the release targets build {found:?}, the install is build {expected}")]
    WrongBuild { found: u64, expected: u64 },

    #[error("the manifest lists no files")]
    NoFiles,

    #[error("{0:?} is not a path a release may write")]
    UnsafePath(String),

    #[error("{path:?} is listed twice")]
    DuplicatePath { path: String },

    #[error("{path:?} is a copy of {target:?}, which the manifest does not list")]
    DanglingCopy { path: String, target: String },

    #[error("{path:?} is a copy of itself")]
    SelfCopy { path: String },

    #[error("{path:?} has a malformed SHA-256")]
    MalformedDigest { path: String },

    #[error("{what}: expected sha256 {expected}, got {found}")]
    DigestMismatch {
        what: String,
        expected: String,
        found: String,
    },

    #[error("{what}: expected {expected} bytes, got {found}")]
    SizeMismatch {
        what: String,
        expected: u64,
        found: u64,
    },

    #[error("the GitHub asset digest {found:?} does not match the manifest")]
    AssetDigestMismatch { found: String },
}

impl VerifyError {
    /// Fluent message id for the sentence the user sees.
    pub fn message_id(&self) -> &'static str {
        match self {
            VerifyError::NoTrustedKeys => "err-no-trusted-keys",
            VerifyError::MalformedSignature(_)
            | VerifyError::UnknownKey(_)
            | VerifyError::KeyNotAllowed { .. }
            | VerifyError::BadSignature => "err-bad-signature",
            VerifyError::MalformedManifest(_)
            | VerifyError::NoFiles
            | VerifyError::UnsafePath(_)
            | VerifyError::DuplicatePath { .. }
            | VerifyError::DanglingCopy { .. }
            | VerifyError::SelfCopy { .. }
            | VerifyError::MalformedDigest { .. }
            | VerifyError::WrongProduct { .. } => "err-bad-manifest",
            VerifyError::UnsupportedSchema { .. } | VerifyError::LauncherTooOld { .. } => {
                "err-launcher-too-old"
            }
            VerifyError::WrongBuild { .. } => "err-wrong-build",
            VerifyError::DigestMismatch { .. }
            | VerifyError::SizeMismatch { .. }
            | VerifyError::AssetDigestMismatch { .. } => "err-hash-mismatch",
        }
    }
}

pub type VerifyResult<T> = std::result::Result<T, VerifyError>;

/// The contents of a `.sig` file.
///
/// JSON rather than raw bytes so the key id travels with the signature,
/// which is what makes rotating a key possible without guessing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureFile {
    /// Always `ed25519` for now.
    pub alg: String,
    pub key_id: String,
    /// Standard base64 of the 64 byte signature.
    pub sig: String,
}

/// A manifest whose signature has been checked.
///
/// The only way to build one is through [`verify_manifest`], so a function
/// taking this type cannot be handed an unverified manifest by mistake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedManifest {
    manifest: Manifest,
    /// The key that signed it, for the log and the about screen.
    pub signed_by: String,
}

impl VerifiedManifest {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn into_manifest(self) -> Manifest {
        self.manifest
    }
}

/// Builds a verified manifest without a signature, for tests in this crate
/// that exercise code downstream of verification.
#[cfg(test)]
pub(crate) fn verified_for_tests(manifest: Manifest) -> VerifiedManifest {
    VerifiedManifest {
        manifest,
        signed_by: "test".to_owned(),
    }
}

impl std::ops::Deref for VerifiedManifest {
    type Target = Manifest;

    fn deref(&self) -> &Self::Target {
        &self.manifest
    }
}

/// Level one: check the signature, then the manifest itself.
///
/// `manifest_bytes` must be the file exactly as it was downloaded. Parsing
/// and re-serialising it before this point would change the bytes the
/// signature covers.
pub fn verify_manifest(
    manifest_bytes: &[u8],
    signature_bytes: &[u8],
    expected_product: Product,
    running_launcher: &Version,
) -> VerifyResult<VerifiedManifest> {
    verify_manifest_with_keys(
        manifest_bytes,
        signature_bytes,
        expected_product,
        running_launcher,
        &keys_for(expected_product).copied().collect::<Vec<_>>(),
    )
}

/// The same check against an explicit key list, which is how the tests
/// exercise it without a key compiled into the build.
pub fn verify_manifest_with_keys(
    manifest_bytes: &[u8],
    signature_bytes: &[u8],
    expected_product: Product,
    running_launcher: &Version,
    keys: &[TrustedKey],
) -> VerifyResult<VerifiedManifest> {
    if keys.is_empty() {
        return Err(VerifyError::NoTrustedKeys);
    }

    let signature: SignatureFile = serde_json::from_slice(signature_bytes)
        .map_err(|error| VerifyError::MalformedSignature(error.to_string()))?;
    if signature.alg != "ed25519" {
        return Err(VerifyError::MalformedSignature(format!(
            "unknown algorithm {:?}",
            signature.alg
        )));
    }

    let raw = base64_decode(&signature.sig)
        .ok_or_else(|| VerifyError::MalformedSignature("the signature is not base64".to_owned()))?;
    let raw: [u8; 64] = raw.try_into().map_err(|bytes: Vec<u8>| {
        VerifyError::MalformedSignature(format!("{} bytes, expected 64", bytes.len()))
    })?;
    let parsed_signature = Signature::from_bytes(&raw);

    let key = keys
        .iter()
        .find(|key| key.id == signature.key_id)
        .ok_or_else(|| VerifyError::UnknownKey(signature.key_id.clone()))?;
    if !key.products.contains(&expected_product) {
        return Err(VerifyError::KeyNotAllowed {
            key: key.id.to_owned(),
            product: expected_product.as_str(),
        });
    }

    let verifying = VerifyingKey::from_bytes(&key.bytes)
        .map_err(|error| VerifyError::MalformedSignature(error.to_string()))?;
    // verify_strict rejects small order and otherwise degenerate keys, so a
    // signature cannot be made to verify against a crafted public key.
    verifying
        .verify_strict(manifest_bytes, &parsed_signature)
        .map_err(|_| VerifyError::BadSignature)?;

    let manifest: Manifest = serde_json::from_slice(manifest_bytes)
        .map_err(|error| VerifyError::MalformedManifest(error.to_string()))?;

    check_manifest(&manifest, expected_product, running_launcher)?;

    Ok(VerifiedManifest {
        manifest,
        signed_by: key.id.to_owned(),
    })
}

/// Everything about a manifest that can be judged without touching a file.
pub fn check_manifest(
    manifest: &Manifest,
    expected_product: Product,
    running_launcher: &Version,
) -> VerifyResult<()> {
    if manifest.schema != SCHEMA_VERSION {
        return Err(VerifyError::UnsupportedSchema {
            found: manifest.schema,
            understood: SCHEMA_VERSION,
        });
    }
    if manifest.product != expected_product {
        return Err(VerifyError::WrongProduct {
            found: manifest.product.as_str(),
            expected: expected_product.as_str(),
        });
    }
    if manifest.min_launcher > *running_launcher {
        return Err(VerifyError::LauncherTooOld {
            needed: manifest.min_launcher.to_string(),
            running: running_launcher.to_string(),
        });
    }
    if manifest.files.is_empty() {
        return Err(VerifyError::NoFiles);
    }

    let mut seen = std::collections::HashSet::new();
    for file in &manifest.files {
        safe_relative_path(&file.path).ok_or_else(|| VerifyError::UnsafePath(file.path.clone()))?;
        // Compared without case: Windows would treat hw.dll and HW.DLL as
        // one file, so a manifest listing both is contradicting itself.
        if !seen.insert(crate::manifest::schema::normalize_relative(&file.path)) {
            return Err(VerifyError::DuplicatePath {
                path: file.path.clone(),
            });
        }
        if !is_hex_digest(&file.sha256) {
            return Err(VerifyError::MalformedDigest {
                path: file.path.clone(),
            });
        }
    }

    for file in &manifest.files {
        let Some(target) = &file.copy_of else {
            continue;
        };
        if crate::manifest::schema::normalize_relative(target)
            == crate::manifest::schema::normalize_relative(&file.path)
        {
            return Err(VerifyError::SelfCopy {
                path: file.path.clone(),
            });
        }
        let source = manifest
            .file(target)
            .ok_or_else(|| VerifyError::DanglingCopy {
                path: file.path.clone(),
                target: target.clone(),
            })?;
        // A copy of a copy would need ordering rules nothing needs yet.
        if source.is_copy() {
            return Err(VerifyError::DanglingCopy {
                path: file.path.clone(),
                target: target.clone(),
            });
        }
    }

    if !is_hex_digest(&manifest.archive.sha256) {
        return Err(VerifyError::MalformedDigest {
            path: manifest.archive.name.clone(),
        });
    }

    Ok(())
}

/// Checks the buildid a game release was made for against the install.
///
/// Separate from [`check_manifest`] because the launcher knows the build
/// only after it has found the game.
pub fn check_target_build(manifest: &Manifest, installed_buildid: Option<u64>) -> VerifyResult<()> {
    let (Some(target), Some(installed)) = (manifest.target_buildid, installed_buildid) else {
        return Ok(());
    };
    if target != installed {
        return Err(VerifyError::WrongBuild {
            found: target,
            expected: installed,
        });
    }
    Ok(())
}

/// The buildid every current ReCoF release targets, for messages.
pub const fn expected_buildid() -> u64 {
    EXPECTED_BUILDID
}

/// Level two: the archive on disk is the one the manifest describes.
pub fn verify_archive(manifest: &Manifest, archive: &Path) -> crate::Result<VerifyResult<()>> {
    let (digest, size) = sha256_file_with_size(archive)?;
    Ok(compare(
        &manifest.archive.name,
        &manifest.archive.sha256,
        manifest.archive.size,
        &digest,
        size,
    ))
}

/// Also level two: GitHub's own digest for the asset agrees.
///
/// GitHub reports it as `sha256:<hex>`. A missing digest is not a failure,
/// because the manifest signature is what actually establishes trust; this
/// catches a file that was swapped between GitHub and here.
pub fn check_asset_digest(manifest: &Manifest, asset_digest: Option<&str>) -> VerifyResult<()> {
    let Some(digest) = asset_digest else {
        return Ok(());
    };
    let hex = digest.strip_prefix("sha256:").unwrap_or(digest).trim();
    if hex.is_empty() {
        return Ok(());
    }
    if !digests_equal(hex, &manifest.archive.sha256) {
        return Err(VerifyError::AssetDigestMismatch {
            found: digest.to_owned(),
        });
    }
    Ok(())
}

/// Level three: one extracted file is what the manifest says.
pub fn verify_extracted_bytes(
    file: &crate::manifest::schema::ManifestFile,
    bytes: &[u8],
) -> VerifyResult<()> {
    compare(
        &file.path,
        &file.sha256,
        file.size,
        &sha256_bytes(bytes),
        bytes.len() as u64,
    )
}

/// Level three for a file already on disk, used by verify and repair.
pub fn verify_installed_file(
    file: &crate::manifest::schema::ManifestFile,
    path: &Path,
) -> crate::Result<VerifyResult<()>> {
    let (digest, size) = sha256_file_with_size(path)?;
    Ok(compare(&file.path, &file.sha256, file.size, &digest, size))
}

fn compare(
    what: &str,
    expected_digest: &str,
    expected_size: u64,
    found_digest: &str,
    found_size: u64,
) -> VerifyResult<()> {
    if found_size != expected_size {
        return Err(VerifyError::SizeMismatch {
            what: what.to_owned(),
            expected: expected_size,
            found: found_size,
        });
    }
    if !digests_equal(found_digest, expected_digest) {
        return Err(VerifyError::DigestMismatch {
            what: what.to_owned(),
            expected: expected_digest.to_owned(),
            found: found_digest.to_owned(),
        });
    }
    Ok(())
}

/// Whether a string is a lower or upper case hex SHA-256.
pub fn is_hex_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Turns a manifest path into a relative path that cannot escape the
/// install, or refuses it.
///
/// This is what stops a signed but hostile manifest, or a zip entry, from
/// writing outside the copy. Refused: absolute paths, drive letters, UNC
/// paths, `..` anywhere, and Windows device names such as `CON` or `AUX`,
/// which open a device rather than a file.
pub fn safe_relative_path(path: &str) -> Option<PathBuf> {
    if path.is_empty() || path.len() > 1024 {
        return None;
    }
    // A backslash is a separator on Windows and a legal file name character
    // elsewhere. Treating it as a separator everywhere is the safe reading.
    let unified = path.replace('\\', "/");
    if unified.starts_with('/') || unified.contains("//") {
        return None;
    }
    // "C:" or "C:/..." and alternate data streams.
    if unified.contains(':') {
        return None;
    }

    let mut result = PathBuf::new();
    for part in unified.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return None;
        }
        if part.chars().any(|character| {
            character.is_control() || matches!(character, '<' | '>' | '"' | '|' | '?' | '*')
        }) {
            return None;
        }
        // Windows strips trailing dots and spaces, so "a " and "a" would be
        // the same file while comparing as different paths.
        if part.ends_with('.') || part.ends_with(' ') || part.starts_with(' ') {
            return None;
        }
        if is_device_name(part) {
            return None;
        }
        result.push(part);
    }

    // Belt and braces: whatever the string said, the parsed path must be
    // made of plain components only.
    if result
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    if result.as_os_str().is_empty() {
        return None;
    }
    Some(result)
}

/// Reserved MS-DOS device names, with or without an extension.
fn is_device_name(part: &str) -> bool {
    const DEVICES: &[&str] = &[
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    let stem = part.split('.').next().unwrap_or(part).to_lowercase();
    DEVICES.contains(&stem.as_str())
}

/// Standard base64 without a dependency on a particular crate version.
fn base64_decode(text: &str) -> Option<Vec<u8>> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.decode(text.trim()).ok()
}

/// Standard base64, for the signing side.
pub fn base64_encode(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    STANDARD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::{Archive, Channel, ManifestFile};
    use ed25519_dalek::{Signer, SigningKey};

    /// A deterministic key pair, so the tests do not need randomness.
    fn test_key(seed: u8) -> (SigningKey, TrustedKey, &'static str) {
        let signing = SigningKey::from_bytes(&[seed; 32]);
        let id: &'static str = Box::leak(format!("test-key-{seed}").into_boxed_str());
        let trusted = TrustedKey {
            id,
            bytes: signing.verifying_key().to_bytes(),
            products: &[Product::Recof, Product::RecofLauncher],
        };
        (signing, trusted, id)
    }

    fn manifest() -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: Product::Recof,
            version: Version::parse("0.3.0").unwrap(),
            channel: Channel::Stable,
            released: "2026-10-26T12:00:00Z".to_owned(),
            target_buildid: Some(EXPECTED_BUILDID),
            min_launcher: Version::parse("0.1.0").unwrap(),
            notes_url: None,
            archive: Archive {
                name: "recof-0.3.0.zip".to_owned(),
                size: 10,
                sha256: sha256_bytes(b"0123456789"),
            },
            files: vec![ManifestFile {
                path: "hw.dll".to_owned(),
                size: 4,
                sha256: sha256_bytes(b"halo"),
                copy_of: None,
            }],
            data: Default::default(),
        }
    }

    fn sign(manifest: &Manifest, key: &SigningKey, key_id: &str) -> (Vec<u8>, Vec<u8>) {
        let bytes = serde_json::to_vec_pretty(manifest).unwrap();
        let signature = key.sign(&bytes);
        let file = SignatureFile {
            alg: "ed25519".to_owned(),
            key_id: key_id.to_owned(),
            sig: base64_encode(&signature.to_bytes()),
        };
        (bytes, serde_json::to_vec(&file).unwrap())
    }

    fn running() -> Version {
        Version::parse("0.3.0").unwrap()
    }

    #[test]
    fn a_correctly_signed_manifest_is_accepted() {
        let (signing, trusted, id) = test_key(1);
        let (bytes, signature) = sign(&manifest(), &signing, id);

        let verified =
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted])
                .expect("a good signature verifies");
        assert_eq!(verified.signed_by, id);
        assert_eq!(verified.version, Version::parse("0.3.0").unwrap());
    }

    #[test]
    fn a_build_with_no_keys_installs_nothing() {
        let (signing, _, id) = test_key(1);
        let (bytes, signature) = sign(&manifest(), &signing, id);
        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[]),
            Err(VerifyError::NoTrustedKeys)
        );
    }

    #[test]
    fn one_changed_byte_in_the_manifest_breaks_the_signature() {
        let (signing, trusted, id) = test_key(1);
        let (bytes, signature) = sign(&manifest(), &signing, id);

        for position in [0, bytes.len() / 2, bytes.len() - 1] {
            let mut tampered = bytes.clone();
            tampered[position] ^= 0x01;
            let outcome = verify_manifest_with_keys(
                &tampered,
                &signature,
                Product::Recof,
                &running(),
                &[trusted],
            );
            assert!(
                matches!(
                    outcome,
                    Err(VerifyError::BadSignature) | Err(VerifyError::MalformedManifest(_))
                ),
                "byte {position}: {outcome:?}"
            );
        }
    }

    #[test]
    fn a_manifest_resigned_with_another_key_is_refused() {
        let (_, trusted, _) = test_key(1);
        let (other_signing, _, other_id) = test_key(2);
        let (bytes, signature) = sign(&manifest(), &other_signing, other_id);

        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted]),
            Err(VerifyError::UnknownKey(other_id.to_owned()))
        );
    }

    #[test]
    fn a_signature_claiming_a_trusted_key_id_still_has_to_verify() {
        let (_, trusted, id) = test_key(1);
        let (other_signing, _, _) = test_key(2);
        // Signed with key 2 but labelled as key 1.
        let (bytes, signature) = sign(&manifest(), &other_signing, id);

        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted]),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn a_key_may_be_limited_to_one_product() {
        let (signing, mut trusted, id) = test_key(1);
        trusted.products = &[Product::RecofLauncher];
        let (bytes, signature) = sign(&manifest(), &signing, id);

        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted]),
            Err(VerifyError::KeyNotAllowed {
                key: id.to_owned(),
                product: "recof",
            })
        );
    }

    #[test]
    fn a_launcher_manifest_cannot_pass_as_a_game_manifest() {
        let (signing, trusted, id) = test_key(1);
        let mut manifest = manifest();
        manifest.product = Product::RecofLauncher;
        let (bytes, signature) = sign(&manifest, &signing, id);

        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted]),
            Err(VerifyError::WrongProduct {
                found: "recof-launcher",
                expected: "recof",
            })
        );
    }

    #[test]
    fn a_malformed_signature_file_is_refused_without_panicking() {
        let (_, trusted, id) = test_key(1);
        let bytes = serde_json::to_vec_pretty(&manifest()).unwrap();

        for bad in [
            br#"not json"#.to_vec(),
            br#"{}"#.to_vec(),
            serde_json::to_vec(&SignatureFile {
                alg: "rsa".to_owned(),
                key_id: id.to_owned(),
                sig: "AAAA".to_owned(),
            })
            .unwrap(),
            serde_json::to_vec(&SignatureFile {
                alg: "ed25519".to_owned(),
                key_id: id.to_owned(),
                sig: "not base64 at all !!".to_owned(),
            })
            .unwrap(),
            serde_json::to_vec(&SignatureFile {
                alg: "ed25519".to_owned(),
                key_id: id.to_owned(),
                sig: base64_encode(&[0u8; 10]),
            })
            .unwrap(),
        ] {
            let outcome =
                verify_manifest_with_keys(&bytes, &bad, Product::Recof, &running(), &[trusted]);
            assert!(
                matches!(outcome, Err(VerifyError::MalformedSignature(_))),
                "{outcome:?}"
            );
        }
    }

    #[test]
    fn an_all_zero_signature_does_not_verify() {
        let (_, trusted, id) = test_key(1);
        let bytes = serde_json::to_vec_pretty(&manifest()).unwrap();
        let signature = serde_json::to_vec(&SignatureFile {
            alg: "ed25519".to_owned(),
            key_id: id.to_owned(),
            sig: base64_encode(&[0u8; 64]),
        })
        .unwrap();

        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted]),
            Err(VerifyError::BadSignature)
        );
    }

    #[test]
    fn a_release_needing_a_newer_launcher_is_refused() {
        let (signing, trusted, id) = test_key(1);
        let mut manifest = manifest();
        manifest.min_launcher = Version::parse("9.0.0").unwrap();
        let (bytes, signature) = sign(&manifest, &signing, id);

        assert_eq!(
            verify_manifest_with_keys(&bytes, &signature, Product::Recof, &running(), &[trusted]),
            Err(VerifyError::LauncherTooOld {
                needed: "9.0.0".to_owned(),
                running: running().to_string(),
            })
        );
    }

    #[test]
    fn a_manifest_from_a_future_schema_is_refused() {
        let mut manifest = manifest();
        manifest.schema = SCHEMA_VERSION + 1;
        assert_eq!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::UnsupportedSchema {
                found: SCHEMA_VERSION + 1,
                understood: SCHEMA_VERSION,
            })
        );
    }

    #[test]
    fn a_manifest_that_writes_outside_the_copy_is_refused() {
        for path in [
            "../evil.dll",
            "..\\evil.dll",
            "a/../../evil.dll",
            "/etc/passwd",
            "C:/Windows/System32/evil.dll",
            "C:\\Windows\\evil.dll",
            "\\\\server\\share\\evil.dll",
            "cryoffear/../../evil.dll",
            "con",
            "cryoffear/nul.dll",
            "trailing.",
            "trailing ",
            " leading",
            "has\u{0}null",
            "",
        ] {
            let mut manifest = manifest();
            manifest.files[0].path = path.to_owned();
            let outcome = check_manifest(&manifest, Product::Recof, &running());
            assert!(
                matches!(outcome, Err(VerifyError::UnsafePath(_))),
                "{path:?} should be refused, got {outcome:?}"
            );
        }
    }

    #[test]
    fn ordinary_paths_are_accepted() {
        for path in [
            "hw.dll",
            "cryoffear/cl_dlls/client.dll",
            "cryoffear\\cl_dlls\\hl.dll",
            "a/b/c/d/e.bin",
            "nombre con espacios.dll",
            "ñandú/档案.dll",
            "version.txt",
        ] {
            assert!(safe_relative_path(path).is_some(), "{path:?}");
        }
    }

    #[test]
    fn the_same_file_cannot_be_listed_twice() {
        let mut manifest = manifest();
        let mut duplicate = manifest.files[0].clone();
        duplicate.path = "HW.DLL".to_owned();
        manifest.files.push(duplicate);

        assert!(matches!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::DuplicatePath { .. })
        ));
    }

    #[test]
    fn a_copy_must_name_a_real_entry() {
        let mut manifest = manifest();
        manifest.files.push(ManifestFile {
            path: "sw.dll".to_owned(),
            size: 4,
            sha256: sha256_bytes(b"halo"),
            copy_of: Some("missing.dll".to_owned()),
        });
        assert!(matches!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::DanglingCopy { .. })
        ));
    }

    #[test]
    fn a_copy_of_itself_is_refused() {
        let mut manifest = manifest();
        manifest.files[0].copy_of = Some("hw.dll".to_owned());
        assert!(matches!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::SelfCopy { .. })
        ));
    }

    #[test]
    fn a_chain_of_copies_is_refused() {
        let mut manifest = manifest();
        manifest.files.push(ManifestFile {
            path: "sw.dll".to_owned(),
            size: 4,
            sha256: sha256_bytes(b"halo"),
            copy_of: Some("hw.dll".to_owned()),
        });
        manifest.files.push(ManifestFile {
            path: "tw.dll".to_owned(),
            size: 4,
            sha256: sha256_bytes(b"halo"),
            copy_of: Some("sw.dll".to_owned()),
        });
        assert!(matches!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::DanglingCopy { .. })
        ));
    }

    #[test]
    fn a_manifest_with_no_files_is_refused() {
        let mut manifest = manifest();
        manifest.files.clear();
        assert_eq!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::NoFiles)
        );
    }

    #[test]
    fn a_malformed_digest_is_refused() {
        let mut manifest = manifest();
        manifest.files[0].sha256 = "not a hash".to_owned();
        assert!(matches!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::MalformedDigest { .. })
        ));

        let mut manifest = super::tests::manifest();
        manifest.archive.sha256 = "zz".repeat(32);
        assert!(matches!(
            check_manifest(&manifest, Product::Recof, &running()),
            Err(VerifyError::MalformedDigest { .. })
        ));
    }

    #[test]
    fn the_target_build_is_checked_against_the_install() {
        let manifest = manifest();
        assert!(check_target_build(&manifest, Some(EXPECTED_BUILDID)).is_ok());
        // An install whose build is unknown, picked by hand, does not block.
        assert!(check_target_build(&manifest, None).is_ok());
        assert_eq!(
            check_target_build(&manifest, Some(1234)),
            Err(VerifyError::WrongBuild {
                found: EXPECTED_BUILDID,
                expected: 1234,
            })
        );
        assert_eq!(expected_buildid(), EXPECTED_BUILDID);
    }

    #[test]
    fn an_extracted_file_is_checked_by_size_and_hash() {
        let file = ManifestFile {
            path: "hw.dll".to_owned(),
            size: 4,
            sha256: sha256_bytes(b"halo"),
            copy_of: None,
        };
        assert!(verify_extracted_bytes(&file, b"halo").is_ok());

        assert!(matches!(
            verify_extracted_bytes(&file, b"halp"),
            Err(VerifyError::DigestMismatch { .. })
        ));
        assert!(matches!(
            verify_extracted_bytes(&file, b"halooo"),
            Err(VerifyError::SizeMismatch { .. })
        ));
    }

    #[test]
    fn the_github_asset_digest_is_compared_when_present() {
        let manifest = manifest();
        let good = format!("sha256:{}", manifest.archive.sha256);
        assert!(check_asset_digest(&manifest, Some(&good)).is_ok());
        assert!(check_asset_digest(&manifest, Some(&manifest.archive.sha256)).is_ok());
        // GitHub not reporting one is not a reason to refuse: the manifest
        // signature is what establishes trust.
        assert!(check_asset_digest(&manifest, None).is_ok());
        assert!(check_asset_digest(&manifest, Some("")).is_ok());

        assert!(matches!(
            check_asset_digest(&manifest, Some("sha256:deadbeef")),
            Err(VerifyError::AssetDigestMismatch { .. })
        ));
    }

    #[test]
    fn the_archive_on_disk_is_checked() {
        let dir = std::env::temp_dir().join("recof-verify-archive");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("recof-0.3.0.zip");
        std::fs::write(&path, b"0123456789").unwrap();

        let manifest = manifest();
        assert!(verify_archive(&manifest, &path).unwrap().is_ok());

        std::fs::write(&path, b"0123456788").unwrap();
        assert!(matches!(
            verify_archive(&manifest, &path).unwrap(),
            Err(VerifyError::DigestMismatch { .. })
        ));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn every_error_has_a_message_id() {
        let errors = [
            VerifyError::NoTrustedKeys,
            VerifyError::MalformedSignature("x".to_owned()),
            VerifyError::UnknownKey("x".to_owned()),
            VerifyError::KeyNotAllowed {
                key: "x".to_owned(),
                product: "recof",
            },
            VerifyError::BadSignature,
            VerifyError::MalformedManifest("x".to_owned()),
            VerifyError::UnsupportedSchema {
                found: 2,
                understood: 1,
            },
            VerifyError::WrongProduct {
                found: "a",
                expected: "b",
            },
            VerifyError::LauncherTooOld {
                needed: running().to_string(),
                running: running().to_string(),
            },
            VerifyError::WrongBuild {
                found: 1,
                expected: 2,
            },
            VerifyError::NoFiles,
            VerifyError::UnsafePath("x".to_owned()),
            VerifyError::DuplicatePath {
                path: "x".to_owned(),
            },
            VerifyError::DanglingCopy {
                path: "x".to_owned(),
                target: "y".to_owned(),
            },
            VerifyError::SelfCopy {
                path: "x".to_owned(),
            },
            VerifyError::MalformedDigest {
                path: "x".to_owned(),
            },
            VerifyError::DigestMismatch {
                what: "x".to_owned(),
                expected: "a".to_owned(),
                found: "b".to_owned(),
            },
            VerifyError::SizeMismatch {
                what: "x".to_owned(),
                expected: 1,
                found: 2,
            },
            VerifyError::AssetDigestMismatch {
                found: "x".to_owned(),
            },
        ];
        for error in errors {
            assert!(error.message_id().starts_with("err-"), "{error}");
            assert!(!error.to_string().is_empty());
        }
    }
}
