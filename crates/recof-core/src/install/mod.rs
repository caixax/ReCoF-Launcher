//! Installing, repairing, moving and removing a ReCoF copy.

pub mod copy;
pub mod destination;
pub mod maintain;
pub mod state;

pub use copy::{copy_tree, Cancel, CopyOutcome, CopyProgress};
pub use destination::{DestinationProblem, DestinationState};
pub use maintain::{move_install, uninstall, verify, RepairReport, UninstallOptions, VerifyEntry};
pub use state::{InstalledState, STATE_SCHEMA};

use std::path::Path;

use crate::consts::STATE_DIR;
use crate::journal::{self, OperationKind, Progress as JournalProgress};
use crate::space::TreeSize;
use crate::store::StoredVersion;
use crate::version::Version;
use crate::{Error, Result};

/// Where an install has got to, for the progress screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStage {
    /// Copying the base game out of the Steam install.
    Copying(CopyProgress),
    /// Writing the ReCoF files through the journal.
    Applying(JournalProgress),
    Finished,
}

/// Everything an install needs to know.
#[derive(Debug, Clone)]
pub struct InstallRequest<'a> {
    /// The Steam install the copy is made from. Opened read only.
    pub source: &'a Path,
    /// Steam buildid of that source, when it is known.
    pub source_buildid: Option<u64>,
    /// Where the copy goes.
    pub destination: &'a Path,
    /// The size measured and shown to the user, for the progress bar.
    pub measured: TreeSize,
    /// The release to apply on top of the copy.
    pub release: &'a StoredVersion,
    /// RFC 3339 timestamp for the records.
    pub now: &'a str,
    /// Identifier for this operation in the log and the journal.
    pub id: &'a str,
}

/// How an install ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed(Box<InstalledState>),
    /// Stopped on request. Everything that was written has been removed.
    Cancelled,
}

/// Copies the game and applies a release on top of it.
///
/// The destination must already have passed
/// [`destination::validate`]. If the copy is cancelled, everything written
/// is removed, but only when this call created the folder: a destination
/// the user already had is never deleted.
pub fn install(
    request: InstallRequest<'_>,
    cancel: &Cancel,
    progress: &mut dyn FnMut(InstallStage),
) -> Result<InstallOutcome> {
    let destination = request.destination;
    let created_by_us = !destination.exists();

    let outcome = copy_tree(
        request.source,
        destination,
        request.measured,
        cancel,
        &mut |state| progress(InstallStage::Copying(state.clone())),
        // The source is a Steam install and has no .recof, but a reinstall
        // over an emptied copy might, and the launcher's own state must
        // never be copied from somewhere else.
        &|relative| relative.starts_with(STATE_DIR),
    )?;

    if outcome == CopyOutcome::Cancelled {
        clean_up_after_cancel(destination, created_by_us)?;
        return Ok(InstallOutcome::Cancelled);
    }

    let plan = request.release.plan(OperationKind::Install, None)?;
    let applied = journal::apply(
        destination,
        &plan,
        request.now,
        request.id,
        &mut |state| progress(InstallStage::Applying(state)),
        None,
    );

    let applied = match applied {
        Ok(applied) => applied,
        Err(error) => {
            // The journal has already undone or recorded whatever it was in
            // the middle of. What is left here is a copy of the base game
            // with no ReCoF applied, which is removed if this call made it.
            if created_by_us {
                log::warn!("apply failed, removing the copy that was just made");
                let _ = journal::Journal::clear(destination);
                crate::fsutil::remove_dir_if_present(destination)?;
            }
            return Err(error);
        }
    };

    let state = InstalledState {
        schema: state::STATE_SCHEMA,
        version: applied.version,
        channel: request.release.manifest.channel,
        installed_at: request.now.to_owned(),
        launcher_version: crate::consts::LAUNCHER_VERSION.to_owned(),
        source_path: request.source.display().to_string(),
        source_buildid: request.source_buildid,
        manifest: request.release.manifest.clone(),
    };
    state.save(destination)?;

    progress(InstallStage::Finished);
    Ok(InstallOutcome::Installed(Box::new(state)))
}

/// Applies a release on top of a copy that already exists.
///
/// Used by update and by rollback. The copy of the base game is not
/// touched; only the files the manifest names are written.
pub fn apply_release(
    install_root: &Path,
    release: &StoredVersion,
    operation: OperationKind,
    now: &str,
    id: &str,
    progress: &mut dyn FnMut(JournalProgress),
) -> Result<InstalledState> {
    let current = InstalledState::load(install_root)?;
    let plan = release.plan(operation, current.as_ref().map(|state| &state.manifest))?;
    let applied = journal::apply(install_root, &plan, now, id, progress, None)?;

    let state = InstalledState {
        schema: state::STATE_SCHEMA,
        version: applied.version,
        channel: release.manifest.channel,
        installed_at: now.to_owned(),
        launcher_version: crate::consts::LAUNCHER_VERSION.to_owned(),
        source_path: current
            .as_ref()
            .map(|state| state.source_path.clone())
            .unwrap_or_default(),
        source_buildid: current.as_ref().and_then(|state| state.source_buildid),
        manifest: release.manifest.clone(),
    };
    state.save(install_root)?;
    Ok(state)
}

/// Finishes or undoes an interrupted operation in a copy.
///
/// Called before anything else touches the install.
pub fn recover(install_root: &Path) -> Result<journal::Recovery> {
    journal::recover(install_root, &mut |_| {})
}

/// The version in a copy, or nothing if the folder is not a ReCoF install.
pub fn installed_version(install_root: &Path) -> Result<Option<Version>> {
    Ok(InstalledState::load(install_root)?.map(|state| state.version))
}

fn clean_up_after_cancel(destination: &Path, created_by_us: bool) -> Result<()> {
    if created_by_us {
        log::info!("install cancelled, removing {}", destination.display());
        crate::fsutil::remove_dir_if_present(destination)?;
        return Ok(());
    }

    // The folder was there and empty before this started, so remove what
    // was written into it but leave the folder itself.
    log::info!(
        "install cancelled, emptying {} but keeping the folder",
        destination.display()
    );
    let entries = std::fs::read_dir(destination).map_err(|error| Error::io(destination, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| Error::io(destination, error))?;
        let path = entry.path();
        if path.is_dir() {
            crate::fsutil::remove_dir_if_present(&path)?;
        } else {
            crate::fsutil::remove_file_if_present(&path)?;
        }
    }
    Ok(())
}
