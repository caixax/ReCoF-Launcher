//! Running a plan through the journal.

use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::hashes::{digests_equal, sha256_file};
use crate::journal::state::{backup_dir, staging_dir, Action, Journal, Phase, Step, StepState};
use crate::manifest::safe_relative_path;
use crate::version::Version;
use crate::{Error, Result};

/// One file the plan will write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedWrite {
    /// Where it goes, relative to the install root, with forward slashes.
    pub path: String,
    /// Where the content comes from: a file already verified in the version
    /// store. Several entries may name the same source, which is how
    /// `sw.dll` is written as a copy of `hw.dll`.
    pub source: PathBuf,
    /// What the content must hash to once it is in place.
    pub sha256: String,
    pub size: u64,
}

/// A whole operation, before anything has been written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub operation: crate::journal::OperationKind,
    pub version: Version,
    pub previous_version: Option<Version>,
    pub writes: Vec<PlannedWrite>,
    /// Files to remove, relative to the install root. Used when a new
    /// release drops a file an older one installed.
    pub deletes: Vec<String>,
}

/// Where the apply is, so the UI can show it without guessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Staging {
        done: usize,
        total: usize,
        path: String,
    },
    Committing {
        done: usize,
        total: usize,
        path: String,
    },
    Finished,
}

/// What an apply ended up doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub version: Version,
    pub files_written: usize,
    pub files_deleted: usize,
}

/// A hook the tests use to stop an apply at an exact moment.
///
/// Production code passes `None`. Nothing else in the crate may use it, so
/// the real path and the tested path are the same code.
pub type Interrupt<'a> = Option<&'a mut dyn FnMut(&str, &Journal) -> bool>;

/// Carries out a plan, or stops at the first failure with the journal left
/// in a state the next start can recover from.
pub fn apply(
    install_root: &Path,
    plan: &Plan,
    now: &str,
    id: &str,
    progress: &mut dyn FnMut(Progress),
    mut interrupt: Interrupt<'_>,
) -> Result<Applied> {
    if let Some(existing) = Journal::load(install_root)? {
        return Err(Error::Journal {
            detail: format!(
                "an unfinished {:?} from {} is still recorded; recover it first",
                existing.operation, existing.started
            ),
        });
    }

    let steps = plan_steps(install_root, plan)?;
    let total = steps.len();
    if total == 0 {
        return Ok(Applied {
            version: plan.version.clone(),
            files_written: 0,
            files_deleted: 0,
        });
    }

    let mut journal = Journal::new(
        id.to_owned(),
        plan.operation,
        plan.version.clone(),
        plan.previous_version.clone(),
        now.to_owned(),
        steps,
    );

    // Start clean: a previous run that was recovered may have left nothing,
    // but an aborted test run might have.
    crate::journal::state::remove_dir_if_present(&staging_dir(install_root))?;
    crate::journal::state::remove_dir_if_present(&backup_dir(install_root))?;

    journal.save(install_root)?;
    stop_here(&mut interrupt, "after-journal-created", &journal)?;

    stage_all(install_root, plan, &mut journal, progress, &mut interrupt)?;

    // The point of no return. Everything is on disk and verified; from here
    // the operation is finished rather than undone.
    journal.phase = Phase::Committing;
    journal.save(install_root)?;
    stop_here(&mut interrupt, "after-phase-committing", &journal)?;

    commit_all(install_root, &mut journal, progress, &mut interrupt)?;

    journal.phase = Phase::Done;
    journal.save(install_root)?;
    stop_here(&mut interrupt, "after-phase-done", &journal)?;

    Journal::clear(install_root)?;
    progress(Progress::Finished);

    Ok(Applied {
        version: plan.version.clone(),
        files_written: journal
            .steps
            .iter()
            .filter(|step| matches!(step.action, Action::Write { .. }))
            .count(),
        files_deleted: journal
            .steps
            .iter()
            .filter(|step| step.action == Action::Delete)
            .count(),
    })
}

/// Turns a plan into journal steps, refusing anything that would write
/// outside the install.
fn plan_steps(install_root: &Path, plan: &Plan) -> Result<Vec<Step>> {
    let mut steps = Vec::with_capacity(plan.writes.len() + plan.deletes.len());

    for write in &plan.writes {
        let relative = safe_relative_path(&write.path).ok_or_else(|| Error::Journal {
            detail: format!("{:?} is not a path a plan may write", write.path),
        })?;
        let target = install_root.join(&relative);
        steps.push(Step {
            index: steps.len(),
            path: write.path.replace('\\', "/"),
            action: Action::Write {
                sha256: write.sha256.clone(),
                size: write.size,
            },
            state: StepState::Planned,
            had_previous: target.exists(),
        });
    }

    for path in &plan.deletes {
        let relative = safe_relative_path(path).ok_or_else(|| Error::Journal {
            detail: format!("{path:?} is not a path a plan may delete"),
        })?;
        let target = install_root.join(&relative);
        // Nothing to do for a file that is not there.
        if !target.exists() {
            continue;
        }
        steps.push(Step {
            index: steps.len(),
            path: path.replace('\\', "/"),
            action: Action::Delete,
            state: StepState::Planned,
            had_previous: true,
        });
    }

    Ok(steps)
}

/// Phase one: write everything into staging and check it.
fn stage_all(
    install_root: &Path,
    plan: &Plan,
    journal: &mut Journal,
    progress: &mut dyn FnMut(Progress),
    interrupt: &mut Interrupt<'_>,
) -> Result<()> {
    let staging = staging_dir(install_root);
    std::fs::create_dir_all(&staging).at(&staging)?;

    let total = journal.steps.len();
    for index in 0..total {
        let path = journal.steps[index].path.clone();
        progress(Progress::Staging {
            done: index,
            total,
            path: path.clone(),
        });

        if let Action::Write { sha256, size } = journal.steps[index].action.clone() {
            let source = &plan.writes[index].source;
            let staged = journal.steps[index].staged_path(install_root);

            copy_and_sync(source, &staged)?;

            // Verified in staging, not after it is in place, so a bad file
            // never reaches the game tree at all.
            let written = std::fs::metadata(&staged).at(&staged)?.len();
            if written != size {
                return Err(Error::Journal {
                    detail: format!("{path}: staged {written} bytes, the manifest says {size}"),
                });
            }
            let digest = sha256_file(&staged)?;
            if !digests_equal(&digest, &sha256) {
                return Err(Error::Journal {
                    detail: format!("{path}: staged content hashes to {digest}, expected {sha256}"),
                });
            }
            stop_here(interrupt, &format!("staged-content-{index}"), journal)?;
        }

        journal.steps[index].state = StepState::Staged;
        journal.save(install_root)?;
        stop_here(interrupt, &format!("staged-recorded-{index}"), journal)?;
    }

    journal.phase = Phase::Staged;
    journal.save(install_root)?;
    Ok(())
}

/// Phase two: put every staged file in place.
pub(crate) fn commit_all(
    install_root: &Path,
    journal: &mut Journal,
    progress: &mut dyn FnMut(Progress),
    interrupt: &mut Interrupt<'_>,
) -> Result<()> {
    let backups = backup_dir(install_root);
    std::fs::create_dir_all(&backups).at(&backups)?;

    let total = journal.steps.len();
    for index in 0..total {
        if journal.steps[index].state == StepState::Committed {
            continue;
        }
        let step = journal.steps[index].clone();
        progress(Progress::Committing {
            done: index,
            total,
            path: step.path.clone(),
        });

        let target = step.target_path(install_root);
        let backup = step.backup_path(install_root);
        let staged = step.staged_path(install_root);

        // Every step has to survive being run again, because recovery
        // restarts the last step it cannot prove finished. Two rules make
        // that safe: a backup is written once and never overwritten, since
        // it holds the only copy of the file that was there; and a step
        // whose work is already visible on disk is recognised instead of
        // being redone.
        if already_done(&step, &target, &staged)? {
            journal.steps[index].state = StepState::Committed;
            journal.save(install_root)?;
            continue;
        }

        if target.exists() && !backup.exists() {
            if let Some(parent) = backup.parent() {
                std::fs::create_dir_all(parent).at(parent)?;
            }
            std::fs::rename(&target, &backup).at(&target)?;
        }
        stop_here(interrupt, &format!("moved-aside-{index}"), journal)?;

        match step.action {
            Action::Write { .. } => {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).at(parent)?;
                }
                std::fs::rename(&staged, &target).at(&target)?;
            }
            Action::Delete => {
                // The backup already holds it, so anything still at the
                // target is a leftover from a repeated run.
                if target.exists() {
                    std::fs::remove_file(&target).at(&target)?;
                }
            }
        }
        stop_here(interrupt, &format!("renamed-into-place-{index}"), journal)?;

        journal.steps[index].state = StepState::Committed;
        journal.save(install_root)?;
        stop_here(interrupt, &format!("committed-recorded-{index}"), journal)?;
    }

    Ok(())
}

/// Whether a step's work is already on disk from an earlier attempt.
///
/// This is what makes recovery safe to run on a step that was interrupted
/// between putting the file in place and recording that it had. The answer
/// is taken from the disk, not from the journal, because the journal is
/// exactly the thing that was not written.
fn already_done(step: &Step, target: &Path, staged: &Path) -> Result<bool> {
    match &step.action {
        Action::Write { sha256, .. } => {
            // While the staged file is still there, the rename has not
            // happened: a rename removes the source.
            if staged.exists() {
                return Ok(false);
            }
            if !target.exists() {
                return Err(Error::Journal {
                    detail: format!(
                        "{}: neither the staged file nor the target is there, \
                         so the operation cannot be finished",
                        step.path
                    ),
                });
            }
            // The staged file is gone and something is at the target. It is
            // the new content only if it hashes to what was promised.
            let digest = sha256_file(target)?;
            if digests_equal(&digest, sha256) {
                Ok(true)
            } else {
                Err(Error::Journal {
                    detail: format!(
                        "{}: the staged content is gone and what is in place \
                         hashes to {digest}, not {sha256}",
                        step.path
                    ),
                })
            }
        }
        // A delete is finished exactly when the file is not there.
        Action::Delete => Ok(!target.exists()),
    }
}

/// Copies a file and makes sure the bytes have reached the disk, so a power
/// cut cannot leave a staged file that looks complete but is not.
fn copy_and_sync(source: &Path, destination: &Path) -> Result<()> {
    use std::io::Write;

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).at(parent)?;
    }

    let mut input = std::fs::File::open(source).at(source)?;
    let mut output = std::fs::File::create(destination).at(destination)?;
    std::io::copy(&mut input, &mut output).at(destination)?;
    output.flush().at(destination)?;
    output.sync_all().at(destination)?;
    Ok(())
}

/// Lets a test stop the apply at a named point, as a crash would.
fn stop_here(interrupt: &mut Interrupt<'_>, point: &str, journal: &Journal) -> Result<()> {
    if let Some(hook) = interrupt.as_deref_mut() {
        if hook(point, journal) {
            return Err(Error::Interrupted {
                point: point.to_owned(),
            });
        }
    }
    Ok(())
}
