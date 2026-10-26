//! Backing up and restoring the player's progress.
//!
//! A backup is a zip with a manifest inside listing the SHA-256 of every
//! file it holds. After writing it, the archive is opened again and every
//! entry is read back and hashed: a backup that cannot be read is worse
//! than no backup, because it is trusted and is not there.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::consts::SAVE_PATHS;
use crate::error::IoResultExt;
use crate::hashes::{digests_equal, sha256_bytes};
use crate::{Error, Result};

/// Format version of the backup manifest.
pub const BACKUP_SCHEMA: u32 = 1;

/// Name of the manifest inside the archive. The leading dot keeps it out
/// of the way of anything that is actually a save.
const BACKUP_MANIFEST: &str = ".recof-backup.json";

/// What is backed up, relative to the root of an install.
///
/// Source: `docs/feasibility/launcher.md`, "Back up": the save folder,
/// the endings and unlockables, and the configuration files.
pub const SAVE_SET: &[&str] = SAVE_PATHS;

/// Largest a single save file may be before the backup refuses it. Saves
/// are a few megabytes; this stops a mistyped path from zipping a game.
const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

/// Why a backup was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackupKind {
    /// The user pressed the button.
    Manual,
    /// Taken automatically before an update.
    BeforeUpdate,
    /// Taken automatically before importing over existing saves.
    BeforeImport,
    /// Exported on the way out of an uninstall.
    Export,
}

impl BackupKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BackupKind::Manual => "manual",
            BackupKind::BeforeUpdate => "before-update",
            BackupKind::BeforeImport => "before-import",
            BackupKind::Export => "export",
        }
    }
}

/// The manifest stored inside a backup archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    pub schema: u32,
    pub created: String,
    pub kind: BackupKind,
    /// The ReCoF version that was installed when this was taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recof_version: Option<String>,
    /// Where the files came from, for the record.
    pub source: String,
    /// Path inside the archive to SHA-256 of its contents.
    pub files: BTreeMap<String, String>,
}

impl BackupManifest {
    pub fn total_files(&self) -> usize {
        self.files.len()
    }
}

/// A backup sitting in the backups folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backup {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub manifest: BackupManifest,
}

/// Creates a backup of an install's saves.
///
/// Returns `None` when there is nothing to back up, which is the normal
/// answer for a copy nobody has played yet.
pub fn create(
    install_root: &Path,
    backups_dir: &Path,
    kind: BackupKind,
    now: &str,
    stamp: &str,
    recof_version: Option<String>,
) -> Result<Option<Backup>> {
    let gathered = gather(install_root)?;
    if gathered.is_empty() {
        log::info!("{} has nothing to back up", install_root.display());
        return Ok(None);
    }

    std::fs::create_dir_all(backups_dir).at(backups_dir)?;
    let name = format!("saves-{stamp}-{}.zip", kind.as_str());
    let path = backups_dir.join(&name);
    let temporary = backups_dir.join(format!("{name}.part"));

    let mut manifest = BackupManifest {
        schema: BACKUP_SCHEMA,
        created: now.to_owned(),
        kind,
        recof_version,
        source: install_root.display().to_string(),
        files: BTreeMap::new(),
    };
    for (relative, bytes) in &gathered {
        manifest.files.insert(relative.clone(), sha256_bytes(bytes));
    }

    {
        let file = std::fs::File::create(&temporary).at(&temporary)?;
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for (relative, bytes) in &gathered {
            writer
                .start_file(relative, options)
                .map_err(|error| zip_error(&temporary, error))?;
            writer.write_all(bytes).at(&temporary)?;
        }

        let manifest_json =
            serde_json::to_vec_pretty(&manifest).map_err(|error| Error::Journal {
                detail: format!("the backup manifest could not be written: {error}"),
            })?;
        writer
            .start_file(BACKUP_MANIFEST, options)
            .map_err(|error| zip_error(&temporary, error))?;
        writer.write_all(&manifest_json).at(&temporary)?;

        let mut file = writer
            .finish()
            .map_err(|error| zip_error(&temporary, error))?;
        file.flush().at(&temporary)?;
        file.sync_all().at(&temporary)?;
    }

    // Read it back before calling it a backup.
    verify_archive(&temporary, &manifest)?;

    std::fs::rename(&temporary, &path).at(&path)?;
    let size = std::fs::metadata(&path).at(&path)?.len();

    log::info!(
        "backed up {} files to {}",
        manifest.files.len(),
        path.display()
    );
    Ok(Some(Backup {
        path,
        name,
        size,
        manifest,
    }))
}

/// Collects the files to back up, as path and contents.
fn gather(install_root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut found = Vec::new();
    for entry in SAVE_SET {
        let path = install_root.join(entry.replace('/', std::path::MAIN_SEPARATOR_STR));
        if path.is_dir() {
            collect_dir(&path, entry, &mut found)?;
        } else if path.is_file() {
            found.push(((*entry).to_owned(), read_bounded(&path)?));
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(found)
}

fn collect_dir(directory: &Path, prefix: &str, found: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    let entries = std::fs::read_dir(directory).at(directory)?;
    for entry in entries {
        let entry = entry.at(directory)?;
        let kind = entry.file_type().at(entry.path())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = format!("{prefix}/{name}");

        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            collect_dir(&entry.path(), &relative, found)?;
        } else {
            found.push((relative, read_bounded(&entry.path())?));
        }
    }
    Ok(())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    let size = std::fs::metadata(path).at(path)?.len();
    if size > MAX_FILE_BYTES {
        return Err(Error::Journal {
            detail: format!("{} is {size} bytes, too large to back up", path.display()),
        });
    }
    std::fs::read(path).at(path)
}

/// Opens an archive and checks every file against the manifest.
fn verify_archive(path: &Path, expected: &BackupManifest) -> Result<()> {
    let file = std::fs::File::open(path).at(path)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|error| zip_error(path, error))?;

    for (relative, digest) in &expected.files {
        let mut entry = archive.by_name(relative).map_err(|error| Error::Journal {
            detail: format!("{relative} is missing from the backup just written: {error}"),
        })?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| Error::Journal {
                detail: format!("{relative} could not be read back: {error}"),
            })?;

        if !digests_equal(&sha256_bytes(&bytes), digest) {
            return Err(Error::Journal {
                detail: format!("{relative} does not read back as what was written"),
            });
        }
    }
    Ok(())
}

/// Reads the manifest out of a backup archive.
pub fn read_manifest(path: &Path) -> Result<BackupManifest> {
    let file = std::fs::File::open(path).at(path)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|error| zip_error(path, error))?;
    let mut entry = archive
        .by_name(BACKUP_MANIFEST)
        .map_err(|error| Error::Journal {
            detail: format!("{} has no backup manifest: {error}", path.display()),
        })?;

    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).at(path)?;
    let manifest: BackupManifest =
        serde_json::from_slice(&bytes).map_err(|error| Error::Journal {
            detail: format!("{} has an unreadable manifest: {error}", path.display()),
        })?;

    if manifest.schema != BACKUP_SCHEMA {
        return Err(Error::Journal {
            detail: format!(
                "{} uses backup schema {}, this launcher understands {BACKUP_SCHEMA}",
                path.display(),
                manifest.schema
            ),
        });
    }
    Ok(manifest)
}

/// Every backup in the folder, newest first.
///
/// A file that is not a readable backup is skipped rather than reported:
/// the folder belongs to the user and may hold anything.
pub fn list(backups_dir: &Path) -> Result<Vec<Backup>> {
    let entries = match std::fs::read_dir(backups_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::io(backups_dir, error)),
    };

    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.at(backups_dir)?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("zip") {
            continue;
        }
        let Ok(manifest) = read_manifest(&path) else {
            log::debug!("{} is not a backup", path.display());
            continue;
        };
        found.push(Backup {
            name: entry.file_name().to_string_lossy().into_owned(),
            size: entry.metadata().map(|data| data.len()).unwrap_or(0),
            path,
            manifest,
        });
    }

    // The name carries the timestamp, so sorting by it sorts by date.
    found.sort_by(|left, right| right.name.cmp(&left.name));
    Ok(found)
}

/// What a restore did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreReport {
    pub files: usize,
}

/// Puts a backup back into an install.
///
/// Every file is checked against the backup's own manifest as it comes
/// out, so a damaged archive replaces nothing.
pub fn restore(archive: &Path, install_root: &Path) -> Result<RestoreReport> {
    let manifest = read_manifest(archive)?;

    let file = std::fs::File::open(archive).at(archive)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|error| zip_error(archive, error))?;

    // Everything is read and checked before anything is written, so a
    // restore cannot stop halfway through with some saves replaced.
    let mut staged = Vec::with_capacity(manifest.files.len());
    for (relative, digest) in &manifest.files {
        let safe = crate::manifest::safe_relative_path(relative).ok_or_else(|| Error::Journal {
            detail: format!("{relative:?} is not a path a backup may restore"),
        })?;

        let mut entry = zip.by_name(relative).map_err(|error| Error::Journal {
            detail: format!("{relative} is missing from the backup: {error}"),
        })?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| Error::Journal {
                detail: format!("{relative} could not be read: {error}"),
            })?;

        if !digests_equal(&sha256_bytes(&bytes), digest) {
            return Err(Error::Journal {
                detail: format!("{relative} in the backup does not match its own manifest"),
            });
        }
        staged.push((safe, bytes));
    }

    for (relative, bytes) in &staged {
        let target = install_root.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).at(parent)?;
        }
        std::fs::write(&target, bytes).at(&target)?;
    }

    log::info!(
        "restored {} files into {}",
        staged.len(),
        install_root.display()
    );
    Ok(RestoreReport {
        files: staged.len(),
    })
}

fn zip_error(path: &Path, error: zip::result::ZipError) -> Error {
    Error::Journal {
        detail: format!("{}: {error}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-backup-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// An install with saves in it, including a name with a space and one
    /// with non ASCII.
    fn make_install(root: &Path) {
        let files: &[(&str, &[u8])] = &[
            ("cryoffear/SAVE/saveinfo1.cof", b"the player progress"),
            ("cryoffear/SAVE/ap 01.sav", b"a save with a space"),
            ("cryoffear/SAVE/partida ñ.sav", b"una partida"),
            ("cryoffear/scriptsettings.dat", b"endings"),
            ("cryoffear/alt.dat", b"unlockables"),
            ("cryoffear/config.cfg", b"bind w +forward"),
            // Not in the save set, so it must not be backed up.
            ("cryoffear/cl_dlls/client.dll", b"a game binary"),
        ];
        for (path, content) in files {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, content).unwrap();
        }
    }

    #[test]
    fn a_backup_holds_the_saves_and_nothing_else() {
        let base = scratch("create");
        let install = base.join("ReCoF");
        make_install(&install);

        let backup = create(
            &install,
            &base.join("backups"),
            BackupKind::Manual,
            "2026-10-26T12:00:00Z",
            "20261026-120000",
            Some("0.3.0".to_owned()),
        )
        .unwrap()
        .expect("there are saves to back up");

        assert_eq!(backup.manifest.total_files(), 6);
        assert!(backup
            .manifest
            .files
            .contains_key("cryoffear/SAVE/saveinfo1.cof"));
        assert!(backup
            .manifest
            .files
            .contains_key("cryoffear/SAVE/partida ñ.sav"));
        assert!(backup.manifest.files.contains_key("cryoffear/config.cfg"));
        assert!(
            !backup
                .manifest
                .files
                .contains_key("cryoffear/cl_dlls/client.dll"),
            "a game binary is not progress"
        );
        assert_eq!(backup.manifest.recof_version.as_deref(), Some("0.3.0"));
        assert!(backup.path.is_file());
        assert!(backup.size > 0);
        // The temporary name is gone.
        assert!(!base
            .join("backups")
            .join(format!("{}.part", backup.name))
            .exists());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn an_install_with_no_saves_produces_no_backup() {
        let base = scratch("nothing");
        let install = base.join("ReCoF");
        std::fs::create_dir_all(install.join("cryoffear")).unwrap();

        let backup = create(
            &install,
            &base.join("backups"),
            BackupKind::Manual,
            "2026-10-26T12:00:00Z",
            "20261026-120000",
            None,
        )
        .unwrap();
        assert_eq!(backup, None);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_backup_round_trips_through_a_restore() {
        let base = scratch("round-trip");
        let install = base.join("ReCoF");
        make_install(&install);

        let backup = create(
            &install,
            &base.join("backups"),
            BackupKind::BeforeUpdate,
            "2026-10-26T12:00:00Z",
            "20261026-120000",
            None,
        )
        .unwrap()
        .unwrap();

        // The player plays on and then wants the old saves back.
        std::fs::write(install.join("cryoffear/SAVE/saveinfo1.cof"), b"later").unwrap();
        std::fs::remove_file(install.join("cryoffear/alt.dat")).unwrap();

        let report = restore(&backup.path, &install).unwrap();
        assert_eq!(report.files, 6);
        assert_eq!(
            std::fs::read(install.join("cryoffear/SAVE/saveinfo1.cof")).unwrap(),
            b"the player progress"
        );
        assert_eq!(
            std::fs::read(install.join("cryoffear/alt.dat")).unwrap(),
            b"unlockables"
        );
        assert_eq!(
            std::fs::read(install.join("cryoffear/SAVE/partida ñ.sav")).unwrap(),
            "una partida".as_bytes()
        );

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_restore_into_an_empty_folder_recreates_the_structure() {
        let base = scratch("restore-empty");
        let install = base.join("ReCoF");
        make_install(&install);
        let backup = create(
            &install,
            &base.join("backups"),
            BackupKind::Export,
            "2026-10-26T12:00:00Z",
            "20261026-120000",
            None,
        )
        .unwrap()
        .unwrap();

        let elsewhere = base.join("Otra copia");
        std::fs::create_dir_all(&elsewhere).unwrap();
        restore(&backup.path, &elsewhere).unwrap();
        assert!(elsewhere.join("cryoffear/SAVE/saveinfo1.cof").is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_damaged_backup_restores_nothing_at_all() {
        let base = scratch("damaged");
        let install = base.join("ReCoF");
        make_install(&install);
        let backup = create(
            &install,
            &base.join("backups"),
            BackupKind::Manual,
            "2026-10-26T12:00:00Z",
            "20261026-120000",
            None,
        )
        .unwrap()
        .unwrap();

        // Flip a byte in the middle of the archive.
        let mut bytes = std::fs::read(&backup.path).unwrap();
        let middle = bytes.len() / 2;
        bytes[middle] ^= 0xff;
        std::fs::write(&backup.path, &bytes).unwrap();

        let before = std::fs::read(install.join("cryoffear/config.cfg")).unwrap();
        std::fs::write(install.join("cryoffear/config.cfg"), b"changed since").unwrap();

        // Whatever the damage did, the restore must not half apply.
        if restore(&backup.path, &install).is_ok() {
            // If it still verified, every file has to be the original.
            assert_eq!(
                std::fs::read(install.join("cryoffear/config.cfg")).unwrap(),
                before
            );
        } else {
            assert_eq!(
                std::fs::read(install.join("cryoffear/config.cfg")).unwrap(),
                b"changed since",
                "a refused restore must change nothing"
            );
        }

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn listing_shows_the_newest_first_and_skips_what_is_not_a_backup() {
        let base = scratch("listing");
        let install = base.join("ReCoF");
        let backups = base.join("backups");
        make_install(&install);

        for stamp in ["20261024-100000", "20261026-120000", "20261025-090000"] {
            create(
                &install,
                &backups,
                BackupKind::Manual,
                "2026-10-26T12:00:00Z",
                stamp,
                None,
            )
            .unwrap();
        }
        std::fs::write(backups.join("not-a-backup.zip"), b"random bytes").unwrap();
        std::fs::write(backups.join("notes.txt"), b"my notes").unwrap();

        let found = list(&backups).unwrap();
        assert_eq!(found.len(), 3);
        assert!(found[0].name.contains("20261026"));
        assert!(found[2].name.contains("20261024"));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn listing_a_folder_that_is_not_there_is_empty_rather_than_an_error() {
        let base = scratch("no-folder");
        assert!(list(&base.join("never created")).unwrap().is_empty());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn every_kind_has_a_name_that_goes_in_a_file_name() {
        for kind in [
            BackupKind::Manual,
            BackupKind::BeforeUpdate,
            BackupKind::BeforeImport,
            BackupKind::Export,
        ] {
            let name = kind.as_str();
            assert!(!name.is_empty());
            assert!(name
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-'));
        }
    }

    #[test]
    fn the_save_set_is_the_one_the_study_names() {
        assert!(SAVE_SET.contains(&"cryoffear/SAVE"));
        assert!(SAVE_SET.contains(&"cryoffear/scriptsettings.dat"));
        assert!(SAVE_SET.contains(&"cryoffear/alt.dat"));
        assert!(SAVE_SET.contains(&"cryoffear/config.cfg"));
        assert!(SAVE_SET.contains(&"cryoffear/autoexec.cfg"));
        assert!(SAVE_SET.contains(&"cryoffear/user.scr"));
    }
}
