//! Release manifests: what they say, and whether to believe them.

pub mod keys;
pub mod schema;
pub mod verify;

pub use keys::{TrustedKey, TRUSTED_KEYS};
pub use schema::{
    Archive, Channel, Manifest, ManifestFile, Product, LAUNCHER_MANIFEST_NAME, MANIFEST_NAME,
    SCHEMA_VERSION, SIGNATURE_SUFFIX,
};
pub use verify::{
    check_asset_digest, check_manifest, check_target_build, safe_relative_path, verify_archive,
    verify_extracted_bytes, verify_installed_file, verify_manifest, SignatureFile,
    VerifiedManifest, VerifyError, VerifyResult,
};
