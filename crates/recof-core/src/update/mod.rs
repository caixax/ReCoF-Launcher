//! Checking for, fetching and applying releases.
//!
//! Both products go through the same path: a zip, a signed manifest and
//! its signature. The launcher's own release is a zip holding one
//! executable rather than a bare exe, so there is one download, one
//! verification and one extraction code path instead of two.
//!
//! Order matters and is not negotiable:
//!
//! 1. Read the release list. Nothing here is trusted.
//! 2. Download the manifest, its signature and the archive.
//! 3. Check the signature over the manifest bytes exactly as downloaded.
//! 4. Check the archive hash against the manifest, and against GitHub's
//!    own digest for the asset when it reports one.
//! 5. Extract into the version store, checking every file.
//! 6. Only then apply, through the journal.

pub mod selfupdate;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::manifest::{
    self, Channel, Product, VerifiedManifest, LAUNCHER_MANIFEST_NAME, MANIFEST_NAME,
    SIGNATURE_SUFFIX,
};
use crate::net::{self, DownloadOutcome, DownloadProgress, GithubClient, NetError, Release};
use crate::store::{StoredVersion, VersionStore};
use crate::version::Version;
use crate::{Error, Result};

/// Largest manifest or signature the launcher will read. Both are small
/// JSON documents.
const MAX_METADATA_BYTES: u64 = 1024 * 1024;

/// What a check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Available {
    pub product: Product,
    pub release: Release,
    pub version: Version,
    /// What is installed now, if anything.
    pub installed: Option<Version>,
    /// The release notes, for the changelog shown before updating.
    pub changelog: String,
}

impl Available {
    /// Whether this is worth offering.
    pub fn is_newer(&self) -> bool {
        match &self.installed {
            Some(installed) => self.version > *installed,
            None => true,
        }
    }
}

/// The manifest name a product publishes.
pub fn manifest_name(product: Product) -> &'static str {
    match product {
        Product::Recof => MANIFEST_NAME,
        Product::RecofLauncher => LAUNCHER_MANIFEST_NAME,
    }
}

/// Looks for something newer on a channel.
pub fn check(
    client: &GithubClient,
    repo: &str,
    product: Product,
    channel: Channel,
    installed: Option<Version>,
) -> std::result::Result<Option<Available>, NetError> {
    let Some(release) = client.newest(repo, channel)? else {
        return Ok(None);
    };
    let Some(version) = release.version() else {
        log::warn!("{repo}: tag {:?} is not a version", release.tag_name);
        return Ok(None);
    };

    let changelog = release.body.clone().unwrap_or_default();
    Ok(Some(Available {
        product,
        release,
        version,
        installed,
        changelog,
    }))
}

/// The three files a release is made of, once they are on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub archive: PathBuf,
    pub manifest_bytes: Vec<u8>,
    pub signature_bytes: Vec<u8>,
    /// GitHub's digest for the archive, when it reported one.
    pub asset_digest: Option<String>,
}

/// How far fetching has got.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchProgress {
    Metadata,
    Archive(DownloadProgress),
    Verifying,
    Extracting { done: usize, total: usize },
}

/// Downloads the manifest, the signature and the archive.
///
/// Returns `None` when the download was cancelled.
pub fn fetch(
    release: &Release,
    product: Product,
    directory: &Path,
    cancel: &Arc<AtomicBool>,
    progress: &mut dyn FnMut(FetchProgress),
) -> Result<Option<Fetched>> {
    progress(FetchProgress::Metadata);

    let manifest_file = manifest_name(product);
    let signature_file = format!("{manifest_file}{SIGNATURE_SUFFIX}");

    let manifest_asset = release
        .asset(manifest_file)
        .ok_or_else(|| Error::MissingAsset(manifest_file.to_owned()))?;
    let signature_asset = release
        .asset(&signature_file)
        .ok_or_else(|| Error::MissingAsset(signature_file.clone()))?;
    let archive_asset = release
        .asset_ending_with(".zip")
        .ok_or_else(|| Error::MissingAsset("the release archive".to_owned()))?;

    let manifest_bytes = fetch_small(
        &manifest_asset.browser_download_url,
        directory,
        manifest_file,
        cancel,
    )?;
    let Some(manifest_bytes) = manifest_bytes else {
        return Ok(None);
    };

    let signature_bytes = fetch_small(
        &signature_asset.browser_download_url,
        directory,
        &signature_file,
        cancel,
    )?;
    let Some(signature_bytes) = signature_bytes else {
        return Ok(None);
    };

    let outcome = net::download(
        &archive_asset.browser_download_url,
        directory,
        &archive_asset.name,
        archive_asset.size,
        cancel,
        &mut |state| progress(FetchProgress::Archive(state)),
    )
    .map_err(Error::Net)?;

    let archive = match outcome {
        DownloadOutcome::Downloaded { path, .. } => path,
        DownloadOutcome::Cancelled => return Ok(None),
    };

    Ok(Some(Fetched {
        archive,
        manifest_bytes,
        signature_bytes,
        asset_digest: archive_asset.digest.clone(),
    }))
}

/// Downloads a small file and returns its bytes.
fn fetch_small(
    url: &str,
    directory: &Path,
    name: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<Option<Vec<u8>>> {
    let outcome =
        net::download(url, directory, name, 0, cancel, &mut |_| {}).map_err(Error::Net)?;
    match outcome {
        DownloadOutcome::Downloaded { path, bytes } => {
            if bytes > MAX_METADATA_BYTES {
                return Err(Error::Net(NetError::TooLarge {
                    size: bytes,
                    limit: MAX_METADATA_BYTES,
                }));
            }
            Ok(Some(
                std::fs::read(&path).map_err(|error| Error::io(&path, error))?,
            ))
        }
        DownloadOutcome::Cancelled => Ok(None),
    }
}

/// Checks everything and puts the release in the store.
///
/// This is the only way a downloaded release becomes usable, and it
/// refuses at the first thing that does not add up.
pub fn verify_and_store(
    fetched: &Fetched,
    product: Product,
    running_launcher: &Version,
    installed_buildid: Option<u64>,
    store: &VersionStore,
    progress: &mut dyn FnMut(FetchProgress),
) -> Result<StoredVersion> {
    progress(FetchProgress::Verifying);

    // One: the signature over the manifest exactly as it was downloaded.
    let verified: VerifiedManifest = manifest::verify_manifest(
        &fetched.manifest_bytes,
        &fetched.signature_bytes,
        product,
        running_launcher,
    )?;
    log::info!(
        "manifest for {} {} is signed by {}",
        product.as_str(),
        verified.version,
        verified.signed_by
    );

    // A game release built for another Steam build is refused here, where
    // the install is known, rather than inside the manifest check.
    if product == Product::Recof {
        manifest::check_target_build(verified.manifest(), installed_buildid)?;
    }

    // Two: the archive, against the manifest and against GitHub.
    manifest::verify_archive(verified.manifest(), &fetched.archive)??;
    manifest::check_asset_digest(verified.manifest(), fetched.asset_digest.as_deref())?;

    // Three: every file, as it comes out of the archive.
    let stored = store.store(
        &verified,
        &fetched.manifest_bytes,
        &fetched.signature_bytes,
        &fetched.archive,
        &mut |done, total| progress(FetchProgress::Extracting { done, total }),
    )?;

    // The archive has served its purpose and the store holds the files.
    let _ = std::fs::remove_file(&fetched.archive);

    Ok(stored)
}

/// Installs from files the user already has, for testers working before
/// anything is published.
///
/// The manifest and its signature must sit next to the archive, under the
/// names a release uses. Verification is exactly the same: a local file
/// is not more trusted than a downloaded one.
pub fn from_local_archive(
    archive: &Path,
    product: Product,
    running_launcher: &Version,
    installed_buildid: Option<u64>,
    store: &VersionStore,
    progress: &mut dyn FnMut(FetchProgress),
) -> Result<StoredVersion> {
    let directory = archive.parent().unwrap_or(Path::new("."));
    let manifest_file = manifest_name(product);
    let manifest_path = directory.join(manifest_file);
    let signature_path = directory.join(format!("{manifest_file}{SIGNATURE_SUFFIX}"));

    if !manifest_path.is_file() {
        return Err(Error::MissingAsset(manifest_file.to_owned()));
    }
    if !signature_path.is_file() {
        return Err(Error::MissingAsset(format!(
            "{manifest_file}{SIGNATURE_SUFFIX}"
        )));
    }

    let fetched = Fetched {
        archive: archive.to_path_buf(),
        manifest_bytes: std::fs::read(&manifest_path)
            .map_err(|error| Error::io(&manifest_path, error))?,
        signature_bytes: std::fs::read(&signature_path)
            .map_err(|error| Error::io(&signature_path, error))?,
        // There is no GitHub asset, so there is no second opinion to
        // compare against. The signature is what matters.
        asset_digest: None,
    };

    // A local archive is the user's own file and is not deleted for them,
    // so the store copies out of it and leaves it alone.
    progress(FetchProgress::Verifying);
    let verified = manifest::verify_manifest(
        &fetched.manifest_bytes,
        &fetched.signature_bytes,
        product,
        running_launcher,
    )?;
    if product == Product::Recof {
        manifest::check_target_build(verified.manifest(), installed_buildid)?;
    }
    manifest::verify_archive(verified.manifest(), &fetched.archive)??;

    store.store(
        &verified,
        &fetched.manifest_bytes,
        &fetched.signature_bytes,
        &fetched.archive,
        &mut |done, total| progress(FetchProgress::Extracting { done, total }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Asset;

    fn release(tag: &str) -> Release {
        Release {
            tag_name: tag.to_owned(),
            name: None,
            body: Some("notes".to_owned()),
            draft: false,
            prerelease: false,
            published_at: None,
            html_url: None,
            assets: vec![
                Asset {
                    name: "recof-0.3.0.zip".to_owned(),
                    size: 10,
                    browser_download_url: "https://example.invalid/a.zip".to_owned(),
                    digest: None,
                },
                Asset {
                    name: MANIFEST_NAME.to_owned(),
                    size: 10,
                    browser_download_url: "https://example.invalid/m.json".to_owned(),
                    digest: None,
                },
                Asset {
                    name: format!("{MANIFEST_NAME}{SIGNATURE_SUFFIX}"),
                    size: 10,
                    browser_download_url: "https://example.invalid/m.sig".to_owned(),
                    digest: None,
                },
            ],
        }
    }

    #[test]
    fn each_product_looks_for_its_own_manifest() {
        assert_eq!(manifest_name(Product::Recof), "recof-manifest.json");
        assert_eq!(
            manifest_name(Product::RecofLauncher),
            "recof-launcher-manifest.json"
        );
    }

    #[test]
    fn something_is_newer_only_when_it_actually_is() {
        let base = Available {
            product: Product::Recof,
            release: release("v0.3.0"),
            version: Version::parse("0.3.0").unwrap(),
            installed: None,
            changelog: String::new(),
        };
        assert!(base.is_newer(), "nothing installed means anything is newer");

        let mut same = base.clone();
        same.installed = Some(Version::parse("0.3.0").unwrap());
        assert!(!same.is_newer());

        let mut older = base.clone();
        older.installed = Some(Version::parse("0.4.0").unwrap());
        assert!(!older.is_newer());

        let mut newer = base;
        newer.installed = Some(Version::parse("0.2.0").unwrap());
        assert!(newer.is_newer());
    }

    #[test]
    fn a_release_missing_the_manifest_is_refused_before_anything_downloads() {
        let dir = std::env::temp_dir().join("recof-update-missing-asset");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut without = release("v0.3.0");
        without.assets.retain(|asset| asset.name.ends_with(".zip"));

        let outcome = fetch(
            &without,
            Product::Recof,
            &dir,
            &Arc::new(AtomicBool::new(false)),
            &mut |_| {},
        );
        assert!(
            matches!(outcome, Err(Error::MissingAsset(_))),
            "{outcome:?}"
        );
        // Nothing was written, because nothing was attempted.
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_local_archive_without_its_manifest_is_refused() {
        let dir = std::env::temp_dir().join("recof-update-local");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let archive = dir.join("recof-0.3.0.zip");
        std::fs::write(&archive, b"not really a zip").unwrap();

        let store = VersionStore::new(dir.join("versions"));
        let outcome = from_local_archive(
            &archive,
            Product::Recof,
            &Version::parse("0.1.0").unwrap(),
            None,
            &store,
            &mut |_| {},
        );
        assert!(
            matches!(outcome, Err(Error::MissingAsset(_))),
            "{outcome:?}"
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_local_archive_with_no_signing_key_installs_nothing() {
        let dir = std::env::temp_dir().join("recof-update-local-unsigned");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let archive = dir.join("recof-0.3.0.zip");
        std::fs::write(&archive, b"pretend archive").unwrap();
        std::fs::write(dir.join(MANIFEST_NAME), b"{}").unwrap();
        std::fs::write(
            dir.join(format!("{MANIFEST_NAME}{SIGNATURE_SUFFIX}")),
            br#"{"alg":"ed25519","key_id":"x","sig":"AAAA"}"#,
        )
        .unwrap();

        let store = VersionStore::new(dir.join("versions"));
        let outcome = from_local_archive(
            &archive,
            Product::Recof,
            &Version::parse("0.1.0").unwrap(),
            None,
            &store,
            &mut |_| {},
        );
        // This build trusts no keys yet, so the right answer is to refuse.
        assert!(outcome.is_err(), "an unverifiable archive must not install");
        assert!(store.list(Product::Recof).unwrap().is_empty());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
