//! Finishing or undoing an interrupted operation.
//!
//! Called at every start, before anything else touches the install. The
//! decision is made from the journal alone: before the point of no return
//! nothing in the game tree was touched, so the staging folder is thrown
//! away; after it, everything needed is already on disk, so the renames are
//! finished.

use std::path::Path;

use crate::error::IoResultExt;
use crate::journal::apply::{commit_all, Progress};
use crate::journal::state::{staging_dir, Journal, Phase};
use crate::version::Version;
use crate::Result;

/// What recovery did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recovery {
    /// There was no unfinished operation.
    Nothing,
    /// An operation that had not reached the point of no return was undone.
    /// The install is exactly as it was before it started.
    RolledBack {
        operation: crate::journal::OperationKind,
        version: Version,
    },
    /// An operation past the point of no return was finished.
    RolledForward {
        operation: crate::journal::OperationKind,
        version: Version,
        /// Steps that still had to be put in place.
        completed: usize,
    },
}

impl Recovery {
    /// Whether anything had to be done, which is what decides if the user
    /// is told about it.
    pub fn happened(&self) -> bool {
        !matches!(self, Recovery::Nothing)
    }

    /// Fluent message id for the notice shown after recovery.
    pub fn message_id(&self) -> Option<&'static str> {
        match self {
            Recovery::Nothing => None,
            Recovery::RolledBack { .. } => Some("recovery-rolled-back"),
            Recovery::RolledForward { .. } => Some("recovery-rolled-forward"),
        }
    }
}

/// Recovers an install, if it needs it.
pub fn recover(install_root: &Path, progress: &mut dyn FnMut(Progress)) -> Result<Recovery> {
    let Some(mut journal) = Journal::load(install_root)? else {
        return Ok(Recovery::Nothing);
    };

    log::warn!(
        "unfinished {:?} of version {} from {}, phase {:?}",
        journal.operation,
        journal.version,
        journal.started,
        journal.phase
    );

    if journal.is_committing() {
        let pending = journal
            .steps
            .iter()
            .filter(|step| step.state != crate::journal::StepState::Committed)
            .count();

        // Finishing uses the same code the original run used, so there is
        // no second implementation that could disagree with the first.
        commit_all(install_root, &mut journal, progress, &mut None)?;

        journal.phase = Phase::Done;
        journal.save(install_root)?;

        let outcome = Recovery::RolledForward {
            operation: journal.operation,
            version: journal.version.clone(),
            completed: pending,
        };
        Journal::clear(install_root)?;
        progress(Progress::Finished);
        return Ok(outcome);
    }

    // Before the point of no return: nothing in the game tree was touched,
    // so throwing the staging folder away is the whole rollback.
    //
    // Backups are checked too. They should be empty here, but deleting a
    // directory that exists and should not is safer than assuming.
    restore_any_backups(install_root, &journal)?;
    crate::journal::state::remove_dir_if_present(&staging_dir(install_root))?;

    let outcome = Recovery::RolledBack {
        operation: journal.operation,
        version: journal.version.clone(),
    };
    Journal::clear(install_root)?;
    progress(Progress::Finished);
    Ok(outcome)
}

/// Puts back any file that was moved aside.
///
/// Reached only before the point of no return, where there should be
/// nothing to put back. If a backup is there anyway, something unexpected
/// happened and restoring it is the conservative answer.
fn restore_any_backups(install_root: &Path, journal: &Journal) -> Result<()> {
    for step in &journal.steps {
        let backup = step.backup_path(install_root);
        if !backup.exists() {
            continue;
        }
        let target = step.target_path(install_root);
        log::warn!(
            "restoring {} from a backup left before the point of no return",
            step.path
        );
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).at(parent)?;
        }
        std::fs::rename(&backup, &target).at(&target)?;
    }
    Ok(())
}
