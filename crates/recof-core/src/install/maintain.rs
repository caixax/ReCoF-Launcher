//! Verify and repair, move, and uninstall.

use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::install::copy::{copy_tree, Cancel, CopyOutcome, CopyProgress};
use crate::install::state::InstalledState;
use crate::journal::{self, OperationKind, Plan, PlannedWrite, Progress as JournalProgress};
use crate::manifest::{safe_relative_path, ManifestFile};
use crate::space::TreeSize;
use crate::store::StoredVersion;
use crate::{Error, Result};

/// What one file turned out to be during a verify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyEntry {
    Ok { path: String },
    WrongContent { path: String, detail: String },
    Missing { path: String },
}

impl VerifyEntry {
    pub fn path(&self) -> &str {
        match self {
            VerifyEntry::Ok { path }
            | VerifyEntry::WrongContent { path, .. }
            | VerifyEntry::Missing { path } => path,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, VerifyEntry::Ok { .. })
    }
}

/// The result of checking a copy against the manifest it was installed from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairReport {
    pub entries: Vec<VerifyEntry>,
}

impl RepairReport {
    /// Whether everything matched.
    pub fn is_intact(&self) -> bool {
        self.entries.iter().all(VerifyEntry::is_ok)
    }

    /// The files that need writing again.
    pub fn broken(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|entry| !entry.is_ok())
            .map(VerifyEntry::path)
            .collect()
    }
}

/// Hashes every file the installed manifest names.
///
/// Uses the manifest kept inside the copy, so this works with no network
/// and no version store.
pub fn verify(install_root: &Path, progress: &mut dyn FnMut(usize, usize)) -> Result<RepairReport> {
    let state = InstalledState::load(install_root)?.ok_or_else(|| Error::Journal {
        detail: format!("{} is not a ReCoF install", install_root.display()),
    })?;

    let files = &state.manifest.files;
    let mut entries = Vec::with_capacity(files.len());

    for (index, file) in files.iter().enumerate() {
        progress(index, files.len());
        entries.push(check_one(install_root, file)?);
    }
    progress(files.len(), files.len());

    Ok(RepairReport { entries })
}

fn check_one(install_root: &Path, file: &ManifestFile) -> Result<VerifyEntry> {
    let Some(relative) = safe_relative_path(&file.path) else {
        return Ok(VerifyEntry::WrongContent {
            path: file.path.clone(),
            detail: "the manifest names a path outside the install".to_owned(),
        });
    };
    let full = install_root.join(relative);
    if !full.exists() {
        return Ok(VerifyEntry::Missing {
            path: file.path.clone(),
        });
    }
    match crate::manifest::verify_installed_file(file, &full)? {
        Ok(()) => Ok(VerifyEntry::Ok {
            path: file.path.clone(),
        }),
        Err(problem) => Ok(VerifyEntry::WrongContent {
            path: file.path.clone(),
            detail: problem.to_string(),
        }),
    }
}

/// Writes the files a verify found wrong, taking them from the version
/// store.
///
/// Repair goes through the journal like every other write, so an
/// interrupted repair is recovered the same way.
pub fn repair(
    install_root: &Path,
    release: &StoredVersion,
    broken: &[&str],
    now: &str,
    id: &str,
    progress: &mut dyn FnMut(JournalProgress),
) -> Result<usize> {
    if broken.is_empty() {
        return Ok(0);
    }

    let mut writes = Vec::with_capacity(broken.len());
    for path in broken {
        let file = release.manifest.file(path).ok_or_else(|| Error::Journal {
            detail: format!("{path} is not in the release being repaired from"),
        })?;
        let source_name = file.copy_of.as_deref().unwrap_or(&file.path);
        let relative = safe_relative_path(source_name).ok_or_else(|| Error::Journal {
            detail: format!("{source_name:?} is not a usable path"),
        })?;
        writes.push(PlannedWrite {
            path: file.path.clone(),
            source: release.files_dir().join(relative),
            sha256: file.sha256.clone(),
            size: file.size,
        });
    }

    let plan = Plan {
        operation: OperationKind::Repair,
        version: release.manifest.version.clone(),
        previous_version: None,
        writes,
        deletes: Vec::new(),
    };
    let applied = journal::apply(install_root, &plan, now, id, progress, None)?;
    Ok(applied.files_written)
}

/// Moves a copy to another folder, possibly on another disk.
///
/// A rename is tried first, which is instant within a volume. Across
/// volumes it falls back to copying and only removes the original once the
/// copy is complete, so an interruption leaves the original intact.
pub fn move_install(
    from: &Path,
    to: &Path,
    measured: TreeSize,
    cancel: &Cancel,
    progress: &mut dyn FnMut(&CopyProgress),
) -> Result<MoveOutcome> {
    if to.exists() {
        let is_empty = std::fs::read_dir(to)
            .map_err(|error| Error::io(to, error))?
            .next()
            .is_none();
        if !is_empty {
            return Err(Error::Journal {
                detail: format!("{} is not empty", to.display()),
            });
        }
    }

    // An unfinished operation must not be carried to a new location where
    // its recorded paths no longer mean anything.
    if journal::Journal::load(from)?.is_some() {
        return Err(Error::Journal {
            detail: "the install has an unfinished operation; recover it before moving".to_owned(),
        });
    }

    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).at(parent)?;
    }

    // Within a volume this is a metadata change and cannot half happen.
    if to.exists() {
        // An empty folder in the way would make rename fail on Windows.
        let _ = std::fs::remove_dir(to);
    }
    if std::fs::rename(from, to).is_ok() {
        return Ok(MoveOutcome::Renamed);
    }

    let outcome = copy_tree(
        from,
        to,
        measured,
        cancel,
        progress,
        &crate::install::copy::skip_nothing,
    )?;
    if outcome == CopyOutcome::Cancelled {
        crate::fsutil::remove_dir_if_present(to)?;
        return Ok(MoveOutcome::Cancelled);
    }

    // Only now, with everything at the new location, does the old one go.
    crate::fsutil::remove_dir_if_present(from)?;
    Ok(MoveOutcome::Copied)
}

/// How a move was carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOutcome {
    /// Same volume: instant.
    Renamed,
    /// Across volumes: copied, then the original removed.
    Copied,
    /// Stopped on request, with the original untouched.
    Cancelled,
}

/// What an uninstall should remove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UninstallOptions {
    /// Remove the stored releases of ReCoF as well.
    pub remove_version_store: bool,
    /// Remove the save backups too. Off by default: a backup is the only
    /// copy of somebody's progress.
    pub remove_backups: bool,
}

impl Default for UninstallOptions {
    fn default() -> Self {
        UninstallOptions {
            remove_version_store: true,
            remove_backups: false,
        }
    }
}

/// Removes a copy, and optionally the store and the backups.
///
/// Refuses anything that is not a ReCoF install, so a mistyped path cannot
/// take a folder of somebody's own files with it.
pub fn uninstall(
    install_root: &Path,
    paths: &crate::paths::AppPaths,
    options: UninstallOptions,
) -> Result<Vec<PathBuf>> {
    if InstalledState::load(install_root)?.is_none() {
        return Err(Error::Journal {
            detail: format!(
                "{} does not hold a ReCoF install, so it will not be removed",
                install_root.display()
            ),
        });
    }

    let mut removed = vec![install_root.to_path_buf()];
    crate::fsutil::remove_dir_if_present(install_root)?;

    if options.remove_version_store {
        crate::fsutil::remove_dir_if_present(&paths.versions_dir())?;
        removed.push(paths.versions_dir());
    }
    if options.remove_backups {
        crate::fsutil::remove_dir_if_present(&paths.backups_dir())?;
        removed.push(paths.backups_dir());
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashes::sha256_bytes;
    use crate::manifest::schema::{Archive, Channel, Manifest, Product, SCHEMA_VERSION};
    use crate::version::Version;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-maintain-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    const FILES: &[(&str, &[u8])] = &[
        ("hw.dll", b"engine bytes"),
        ("cryoffear/cl_dlls/hl.dll", b"server bytes"),
    ];

    fn manifest() -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: Product::Recof,
            version: Version::parse("0.3.0").unwrap(),
            channel: Channel::Stable,
            released: "2026-10-26T12:00:00Z".to_owned(),
            target_buildid: Some(crate::consts::EXPECTED_BUILDID),
            min_launcher: Version::parse("0.1.0").unwrap(),
            notes_url: None,
            archive: Archive {
                name: "recof-0.3.0.zip".to_owned(),
                size: 1,
                sha256: sha256_bytes(b"x"),
            },
            files: FILES
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

    /// An install holding exactly what the manifest describes.
    fn make_install(root: &Path) {
        for (path, content) in FILES {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, content).unwrap();
        }
        InstalledState {
            schema: crate::install::state::STATE_SCHEMA,
            version: Version::parse("0.3.0").unwrap(),
            channel: Channel::Stable,
            installed_at: "2026-10-26T12:00:00Z".to_owned(),
            launcher_version: "0.1.0".to_owned(),
            source_path: "somewhere".to_owned(),
            source_buildid: Some(crate::consts::EXPECTED_BUILDID),
            manifest: manifest(),
        }
        .save(root)
        .unwrap();
    }

    #[test]
    fn verify_passes_on_an_untouched_install() {
        let base = scratch("verify-ok");
        let root = base.join("ReCoF");
        make_install(&root);

        let report = verify(&root, &mut |_, _| {}).unwrap();
        assert!(report.is_intact());
        assert!(report.broken().is_empty());
        assert_eq!(report.entries.len(), FILES.len());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn verify_finds_a_changed_and_a_missing_file() {
        let base = scratch("verify-broken");
        let root = base.join("ReCoF");
        make_install(&root);

        std::fs::write(root.join("hw.dll"), b"tampered with").unwrap();
        std::fs::remove_file(root.join("cryoffear/cl_dlls/hl.dll")).unwrap();

        let report = verify(&root, &mut |_, _| {}).unwrap();
        assert!(!report.is_intact());
        assert_eq!(report.broken(), vec!["hw.dll", "cryoffear/cl_dlls/hl.dll"]);
        assert!(matches!(
            report.entries[0],
            VerifyEntry::WrongContent { .. }
        ));
        assert!(matches!(report.entries[1], VerifyEntry::Missing { .. }));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn verify_on_a_folder_that_is_not_an_install_is_an_error() {
        let base = scratch("verify-not-install");
        assert!(verify(&base, &mut |_, _| {}).is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_move_within_a_volume_is_a_rename() {
        let base = scratch("move-same-volume");
        let from = base.join("ReCoF");
        let to = base.join("ReCoF movido ñ");
        make_install(&from);

        let cancel: Cancel = Arc::new(AtomicBool::new(false));
        let outcome = move_install(&from, &to, TreeSize::default(), &cancel, &mut |_| {}).unwrap();

        assert_eq!(outcome, MoveOutcome::Renamed);
        assert!(!from.exists());
        assert_eq!(std::fs::read(to.join("hw.dll")).unwrap(), b"engine bytes");
        assert!(InstalledState::load(&to).unwrap().is_some());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_move_into_a_folder_with_files_in_it_is_refused() {
        let base = scratch("move-occupied");
        let from = base.join("ReCoF");
        let to = base.join("ocupado");
        make_install(&from);
        std::fs::create_dir_all(&to).unwrap();
        std::fs::write(to.join("mio.txt"), b"no").unwrap();

        assert!(move_install(
            &from,
            &to,
            TreeSize::default(),
            &Arc::new(AtomicBool::new(false)),
            &mut |_| {}
        )
        .is_err());
        assert!(from.join("hw.dll").is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_move_with_an_unfinished_operation_is_refused() {
        let base = scratch("move-unfinished");
        let from = base.join("ReCoF");
        make_install(&from);

        crate::journal::Journal::new(
            "x".to_owned(),
            OperationKind::Update,
            Version::parse("0.4.0").unwrap(),
            None,
            "2026-10-26T12:00:00Z".to_owned(),
            Vec::new(),
        )
        .save(&from)
        .unwrap();

        assert!(move_install(
            &from,
            &base.join("destino"),
            TreeSize::default(),
            &Arc::new(AtomicBool::new(false)),
            &mut |_| {}
        )
        .is_err());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn uninstall_removes_the_copy_and_the_store_but_keeps_backups() {
        let base = scratch("uninstall");
        let root = base.join("ReCoF");
        make_install(&root);

        let paths = crate::paths::AppPaths::at(base.join("appdata"));
        paths.create_all().unwrap();
        std::fs::write(paths.backups_dir().join("saves.zip"), b"progress").unwrap();
        std::fs::create_dir_all(paths.versions_dir().join("recof/0.3.0")).unwrap();

        let removed = uninstall(&root, &paths, UninstallOptions::default()).unwrap();

        assert!(!root.exists());
        assert!(!paths.versions_dir().exists());
        assert!(paths.backups_dir().join("saves.zip").is_file());
        assert_eq!(removed.len(), 2);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn uninstall_can_be_asked_to_remove_the_backups_too() {
        let base = scratch("uninstall-all");
        let root = base.join("ReCoF");
        make_install(&root);

        let paths = crate::paths::AppPaths::at(base.join("appdata"));
        paths.create_all().unwrap();
        std::fs::write(paths.backups_dir().join("saves.zip"), b"progress").unwrap();

        uninstall(
            &root,
            &paths,
            UninstallOptions {
                remove_version_store: true,
                remove_backups: true,
            },
        )
        .unwrap();
        assert!(!paths.backups_dir().exists());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn uninstall_refuses_a_folder_that_is_not_a_recof_install() {
        let base = scratch("uninstall-refuses");
        let other = base.join("Documentos importantes");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("tesis.docx"), b"years of work").unwrap();

        let paths = crate::paths::AppPaths::at(base.join("appdata"));
        assert!(uninstall(&other, &paths, UninstallOptions::default()).is_err());
        assert!(other.join("tesis.docx").is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }
}
