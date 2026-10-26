//! The version store.
//!
//! A verified release is extracted once into
//! `%LOCALAPPDATA%\ReCoF Launcher\versions\<product>\<version>` and stays
//! there, so rolling back to it later needs no network and no second
//! download. The last few are kept and older ones are pruned.
//!
//! Extraction is driven by the signed manifest, never by what the archive
//! happens to contain: an entry the manifest does not list is not written,
//! and an entry whose content does not hash to what the manifest promises
//! stops the whole thing.

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::journal::{OperationKind, Plan, PlannedWrite};
use crate::manifest::{safe_relative_path, Manifest, Product, VerifiedManifest, VerifyError};
use crate::manifest::{MANIFEST_NAME, SIGNATURE_SUFFIX};
use crate::version::Version;
use crate::{Error, Result};

/// How many versions of a product are kept before the oldest is pruned.
pub const KEEP_VERSIONS: usize = 3;

/// Largest file a release may contain. A release is a handful of rebuilt
/// binaries, the biggest of which is under 3 MB, so this is generous and
/// still stops a crafted archive from filling the disk during extraction.
const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

/// Directory name holding the extracted files inside a stored version.
const FILES_DIR: &str = "files";

/// One release sitting in the store, ready to be applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredVersion {
    pub product: Product,
    pub version: Version,
    /// `<store>/<product>/<version>`.
    pub dir: PathBuf,
    pub manifest: Manifest,
}

impl StoredVersion {
    /// Where the extracted files are.
    pub fn files_dir(&self) -> PathBuf {
        self.dir.join(FILES_DIR)
    }

    /// The copy of the manifest kept next to them.
    pub fn manifest_path(&self) -> PathBuf {
        self.dir.join(MANIFEST_NAME)
    }

    /// Turns this release into a plan the journal can carry out.
    ///
    /// `currently_installed` is the manifest of what is in the copy now, if
    /// anything. Files it installed that this release no longer lists are
    /// added as deletes, so an update does not leave orphans behind.
    pub fn plan(
        &self,
        operation: OperationKind,
        currently_installed: Option<&Manifest>,
    ) -> Result<Plan> {
        let mut writes = Vec::with_capacity(self.manifest.files.len());
        for file in &self.manifest.files {
            // A copy takes its content from the entry it names, which is
            // how sw.dll is written from the single hw.dll in the archive.
            let source_path = file.copy_of.as_deref().unwrap_or(&file.path);
            let relative = safe_relative_path(source_path)
                .ok_or_else(|| Error::from(VerifyError::UnsafePath(source_path.to_owned())))?;
            let source = self.files_dir().join(relative);
            if !source.is_file() {
                return Err(Error::Journal {
                    detail: format!(
                        "{} is missing from the version store at {}",
                        file.path,
                        source.display()
                    ),
                });
            }
            writes.push(PlannedWrite {
                path: file.path.clone(),
                source,
                sha256: file.sha256.clone(),
                size: file.size,
            });
        }

        let deletes = match currently_installed {
            Some(previous) => previous
                .files
                .iter()
                .filter(|old| self.manifest.file(&old.path).is_none())
                .map(|old| old.path.clone())
                .collect(),
            None => Vec::new(),
        };

        Ok(Plan {
            operation,
            version: self.manifest.version.clone(),
            previous_version: currently_installed.map(|manifest| manifest.version.clone()),
            writes,
            deletes,
        })
    }
}

/// The folder holding every stored release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionStore {
    root: PathBuf,
}

impl VersionStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        VersionStore { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn product_dir(&self, product: Product) -> PathBuf {
        self.root.join(product.as_str())
    }

    /// Where a given version lives, whether or not it is there yet.
    pub fn version_dir(&self, product: Product, version: &Version) -> PathBuf {
        // A version string is digits, dots, hyphens and plus signs, all of
        // which are fine in a directory name, but a plus is replaced so a
        // build tag cannot produce an awkward path.
        self.product_dir(product)
            .join(version.to_string().replace('+', "_"))
    }

    /// Extracts a verified release into the store.
    ///
    /// The archive must already have passed [`crate::manifest::verify_archive`].
    /// Every file is extracted to a temporary folder and only then moved
    /// into place, so an interrupted extraction never looks like a complete
    /// stored version.
    pub fn store(
        &self,
        verified: &VerifiedManifest,
        manifest_bytes: &[u8],
        signature_bytes: &[u8],
        archive: &Path,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<StoredVersion> {
        let manifest = verified.manifest();
        let product = manifest.product;
        let final_dir = self.version_dir(product, &manifest.version);
        let staging = final_dir.with_extension("incomplete");

        crate::fsutil::remove_dir_if_present(&staging)?;
        std::fs::create_dir_all(staging.join(FILES_DIR)).at(&staging)?;

        let file = std::fs::File::open(archive).at(archive)?;
        let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|error| {
            Error::Journal {
                detail: format!("{} is not a usable zip: {error}", archive.display()),
            }
        })?;

        // Only entries the manifest lists are written, and only once each.
        let real_files: Vec<_> = manifest
            .files
            .iter()
            .filter(|file| !file.is_copy())
            .collect();
        let total = real_files.len();

        for (index, entry) in real_files.iter().enumerate() {
            progress(index, total);

            if entry.size > MAX_FILE_BYTES {
                return Err(Error::Journal {
                    detail: format!("{} is {} bytes, which is too large", entry.path, entry.size),
                });
            }

            let relative = safe_relative_path(&entry.path)
                .ok_or_else(|| Error::from(VerifyError::UnsafePath(entry.path.clone())))?;

            let mut contents = Vec::with_capacity(entry.size as usize);
            {
                let mut inside = zip.by_name(&entry.path).map_err(|error| Error::Journal {
                    detail: format!("{} is not in the archive: {error}", entry.path),
                })?;
                // Bounded by the manifest size plus one byte, so a zip bomb
                // cannot be read into memory before the check below.
                inside
                    .by_ref()
                    .take(entry.size + 1)
                    .read_to_end(&mut contents)
                    .map_err(|error| Error::Journal {
                        detail: format!("{} could not be read: {error}", entry.path),
                    })?;
            }

            crate::manifest::verify_extracted_bytes(entry, &contents)?;

            let destination = staging.join(FILES_DIR).join(&relative);
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).at(parent)?;
            }
            std::fs::write(&destination, &contents).at(&destination)?;
        }
        progress(total, total);

        // Keep the manifest and its signature with the files, so a stored
        // version can be checked again later without the network.
        std::fs::write(staging.join(MANIFEST_NAME), manifest_bytes)
            .at(staging.join(MANIFEST_NAME))?;
        let signature_name = format!("{MANIFEST_NAME}{SIGNATURE_SUFFIX}");
        std::fs::write(staging.join(&signature_name), signature_bytes)
            .at(staging.join(&signature_name))?;

        crate::fsutil::remove_dir_if_present(&final_dir)?;
        if let Some(parent) = final_dir.parent() {
            std::fs::create_dir_all(parent).at(parent)?;
        }
        std::fs::rename(&staging, &final_dir).at(&final_dir)?;

        Ok(StoredVersion {
            product,
            version: manifest.version.clone(),
            dir: final_dir,
            manifest: manifest.clone(),
        })
    }

    /// Every stored version of a product, oldest first.
    ///
    /// A folder that does not hold a readable manifest is ignored rather
    /// than reported: it is a leftover, not a problem the user can act on.
    pub fn list(&self, product: Product) -> Result<Vec<StoredVersion>> {
        let directory = self.product_dir(product);
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(Error::io(&directory, error)),
        };

        let mut found = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| Error::io(&directory, error))?;
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            let dir = entry.path();
            let Ok(bytes) = std::fs::read(dir.join(MANIFEST_NAME)) else {
                continue;
            };
            let Ok(manifest) = serde_json::from_slice::<Manifest>(&bytes) else {
                log::warn!("{} holds an unreadable manifest", dir.display());
                continue;
            };
            if manifest.product != product {
                continue;
            }
            found.push(StoredVersion {
                product,
                version: manifest.version.clone(),
                dir,
                manifest,
            });
        }

        found.sort_by(|left, right| left.version.cmp(&right.version));
        Ok(found)
    }

    /// One stored version, if it is there.
    pub fn get(&self, product: Product, version: &Version) -> Result<Option<StoredVersion>> {
        Ok(self
            .list(product)?
            .into_iter()
            .find(|stored| stored.version == *version))
    }

    /// Deletes the oldest versions beyond `keep`, never touching the one
    /// that is currently installed.
    pub fn prune(
        &self,
        product: Product,
        keep: usize,
        installed: Option<&Version>,
    ) -> Result<Vec<Version>> {
        let stored = self.list(product)?;
        if stored.len() <= keep {
            return Ok(Vec::new());
        }

        let mut removed = Vec::new();
        let surplus = stored.len() - keep;
        for candidate in stored.iter().take(surplus) {
            if Some(&candidate.version) == installed {
                continue;
            }
            std::fs::remove_dir_all(&candidate.dir).at(&candidate.dir)?;
            removed.push(candidate.version.clone());
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashes::sha256_bytes;
    use crate::manifest::schema::{Archive, Channel, ManifestFile, SCHEMA_VERSION};
    use std::io::Write;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-store-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// A zip holding the given entries.
    fn make_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = std::fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }

    fn manifest_for(version: &str, entries: &[(&str, &[u8])], archive: &Path) -> Manifest {
        let archive_bytes = std::fs::read(archive).unwrap();
        Manifest {
            schema: SCHEMA_VERSION,
            product: Product::Recof,
            version: Version::parse(version).unwrap(),
            channel: Channel::Stable,
            released: "2026-10-26T12:00:00Z".to_owned(),
            target_buildid: Some(crate::consts::EXPECTED_BUILDID),
            min_launcher: Version::parse("0.1.0").unwrap(),
            notes_url: None,
            archive: Archive {
                name: archive.file_name().unwrap().to_string_lossy().into_owned(),
                size: archive_bytes.len() as u64,
                sha256: sha256_bytes(&archive_bytes),
            },
            files: entries
                .iter()
                .map(|(path, content)| ManifestFile {
                    path: (*path).to_owned(),
                    size: content.len() as u64,
                    sha256: sha256_bytes(content),
                    copy_of: None,
                })
                .collect(),
            data: Default::default(),
        }
    }

    /// Builds a VerifiedManifest without going through the signature path,
    /// which is tested on its own in the manifest module.
    fn as_verified(manifest: Manifest) -> (VerifiedManifest, Vec<u8>) {
        let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        (crate::manifest::verify::verified_for_tests(manifest), bytes)
    }

    const ENTRIES: &[(&str, &[u8])] = &[
        ("hw.dll", b"engine bytes"),
        ("cryoffear/cl_dlls/hl.dll", b"server bytes"),
    ];

    #[test]
    fn storing_extracts_only_what_the_manifest_lists() {
        let base = scratch("extract");
        let archive = base.join("recof-0.3.0.zip");
        // The archive also holds a file nobody asked for.
        let mut entries = ENTRIES.to_vec();
        entries.push(("not-in-the-manifest.dll", b"should be ignored"));
        make_zip(&archive, &entries);

        let (verified, bytes) = as_verified(manifest_for("0.3.0", ENTRIES, &archive));
        let store = VersionStore::new(base.join("versions"));
        let stored = store
            .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
            .unwrap();

        assert_eq!(stored.version, Version::parse("0.3.0").unwrap());
        assert_eq!(
            std::fs::read(stored.files_dir().join("hw.dll")).unwrap(),
            b"engine bytes"
        );
        assert_eq!(
            std::fs::read(stored.files_dir().join("cryoffear/cl_dlls/hl.dll")).unwrap(),
            b"server bytes"
        );
        assert!(!stored.files_dir().join("not-in-the-manifest.dll").exists());
        assert!(stored.manifest_path().is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_file_whose_content_does_not_match_stops_the_extraction() {
        let base = scratch("bad-content");
        let archive = base.join("recof-0.3.0.zip");
        make_zip(&archive, &[("hw.dll", b"something else entirely")]);

        let (verified, bytes) = as_verified(manifest_for("0.3.0", ENTRIES, &archive));
        let store = VersionStore::new(base.join("versions"));
        let outcome = store.store(&verified, &bytes, b"{}", &archive, &mut |_, _| {});

        assert!(outcome.is_err(), "{outcome:?}");
        // Nothing half extracted is left where list() would find it.
        assert!(store.list(Product::Recof).unwrap().is_empty());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_manifest_entry_missing_from_the_archive_is_an_error() {
        let base = scratch("missing-entry");
        let archive = base.join("recof-0.3.0.zip");
        make_zip(&archive, &[("hw.dll", b"engine bytes")]);

        let (verified, bytes) = as_verified(manifest_for("0.3.0", ENTRIES, &archive));
        let store = VersionStore::new(base.join("versions"));
        assert!(store
            .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
            .is_err());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn listing_is_ordered_oldest_first() {
        let base = scratch("listing");
        let store = VersionStore::new(base.join("versions"));

        for version in ["0.2.0", "1.0.0", "0.10.0", "1.0.0-beta.1"] {
            let archive = base.join(format!("recof-{version}.zip"));
            make_zip(&archive, ENTRIES);
            let (verified, bytes) = as_verified(manifest_for(version, ENTRIES, &archive));
            store
                .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
                .unwrap();
        }

        let listed: Vec<String> = store
            .list(Product::Recof)
            .unwrap()
            .iter()
            .map(|stored| stored.version.to_string())
            .collect();
        assert_eq!(listed, ["0.2.0", "0.10.0", "1.0.0-beta.1", "1.0.0"]);

        assert!(store
            .get(Product::Recof, &Version::parse("0.10.0").unwrap())
            .unwrap()
            .is_some());
        assert!(store
            .get(Product::Recof, &Version::parse("9.9.9").unwrap())
            .unwrap()
            .is_none());
        assert!(store.list(Product::RecofLauncher).unwrap().is_empty());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn pruning_keeps_the_newest_and_never_removes_what_is_installed() {
        let base = scratch("pruning");
        let store = VersionStore::new(base.join("versions"));
        for version in ["0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0"] {
            let archive = base.join(format!("recof-{version}.zip"));
            make_zip(&archive, ENTRIES);
            let (verified, bytes) = as_verified(manifest_for(version, ENTRIES, &archive));
            store
                .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
                .unwrap();
        }

        // The oldest is the one in use, so it stays even though it is the
        // first candidate for removal.
        let installed = Version::parse("0.1.0").unwrap();
        let removed = store.prune(Product::Recof, 3, Some(&installed)).unwrap();
        assert_eq!(removed, vec![Version::parse("0.2.0").unwrap()]);

        let left: Vec<String> = store
            .list(Product::Recof)
            .unwrap()
            .iter()
            .map(|stored| stored.version.to_string())
            .collect();
        assert_eq!(left, ["0.1.0", "0.3.0", "0.4.0", "0.5.0"]);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn pruning_an_almost_empty_store_does_nothing() {
        let base = scratch("prune-empty");
        let store = VersionStore::new(base.join("versions"));
        assert!(store.prune(Product::Recof, 3, None).unwrap().is_empty());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_plan_points_at_the_stored_files_and_handles_copies() {
        let base = scratch("planning");
        let archive = base.join("recof-0.3.0.zip");
        make_zip(&archive, ENTRIES);

        let mut manifest = manifest_for("0.3.0", ENTRIES, &archive);
        manifest.files.push(ManifestFile {
            path: "sw.dll".to_owned(),
            size: ENTRIES[0].1.len() as u64,
            sha256: sha256_bytes(ENTRIES[0].1),
            copy_of: Some("hw.dll".to_owned()),
        });

        let (verified, bytes) = as_verified(manifest);
        let store = VersionStore::new(base.join("versions"));
        let stored = store
            .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
            .unwrap();

        let plan = stored.plan(OperationKind::Install, None).unwrap();
        assert_eq!(plan.writes.len(), 3);
        // sw.dll takes its bytes from the single stored hw.dll.
        assert_eq!(plan.writes[2].path, "sw.dll");
        assert_eq!(plan.writes[2].source, stored.files_dir().join("hw.dll"));
        assert!(plan.deletes.is_empty());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_plan_deletes_files_the_previous_release_installed() {
        let base = scratch("orphans");
        let archive = base.join("recof-0.4.0.zip");
        make_zip(&archive, ENTRIES);
        let (verified, bytes) = as_verified(manifest_for("0.4.0", ENTRIES, &archive));
        let store = VersionStore::new(base.join("versions"));
        let stored = store
            .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
            .unwrap();

        let mut previous = manifest_for("0.3.0", ENTRIES, &archive);
        previous.files.push(ManifestFile {
            path: "dropped.dll".to_owned(),
            size: 1,
            sha256: sha256_bytes(b"x"),
            copy_of: None,
        });

        let plan = stored.plan(OperationKind::Update, Some(&previous)).unwrap();
        assert_eq!(plan.deletes, vec!["dropped.dll".to_owned()]);
        assert_eq!(
            plan.previous_version,
            Some(Version::parse("0.3.0").unwrap())
        );

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn storing_the_same_version_again_replaces_it() {
        let base = scratch("replace");
        let store = VersionStore::new(base.join("versions"));

        let archive = base.join("a.zip");
        make_zip(&archive, ENTRIES);
        let (verified, bytes) = as_verified(manifest_for("0.3.0", ENTRIES, &archive));
        store
            .store(&verified, &bytes, b"{}", &archive, &mut |_, _| {})
            .unwrap();

        let second: &[(&str, &[u8])] = &[("hw.dll", b"rebuilt engine")];
        let archive2 = base.join("b.zip");
        make_zip(&archive2, second);
        let (verified2, bytes2) = as_verified(manifest_for("0.3.0", second, &archive2));
        let stored = store
            .store(&verified2, &bytes2, b"{}", &archive2, &mut |_, _| {})
            .unwrap();

        assert_eq!(store.list(Product::Recof).unwrap().len(), 1);
        assert_eq!(
            std::fs::read(stored.files_dir().join("hw.dll")).unwrap(),
            b"rebuilt engine"
        );
        assert!(!stored.files_dir().join("cryoffear/cl_dlls/hl.dll").exists());

        std::fs::remove_dir_all(&base).unwrap();
    }
}
