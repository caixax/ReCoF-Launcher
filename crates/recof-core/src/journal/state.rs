//! The journal file itself: what it records and how it is written.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::consts::STATE_DIR;
use crate::error::IoResultExt;
use crate::version::Version;
use crate::{Error, Result};

/// Format version of the journal. A journal from a newer launcher is left
/// alone rather than half understood.
pub const JOURNAL_SCHEMA: u32 = 1;

/// Name of the journal inside `.recof`.
const JOURNAL_FILE: &str = "journal.json";

/// Where staged content waits before being renamed into place.
const STAGING_DIR: &str = "staging";

/// Where a replaced file is kept until the operation finishes.
const BACKUP_DIR: &str = "backup";

/// What the operation was for, so the UI and the log can say so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    Install,
    Update,
    Rollback,
    Repair,
}

/// How far the operation got. The order of these matters: everything up to
/// and including [`Phase::Staged`] is undone, everything from
/// [`Phase::Committing`] on is finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    /// Steps are being written into staging.
    Preparing,
    /// Every step is staged and verified. Still nothing touched.
    Staged,
    /// The point of no return has been passed: files are being renamed into
    /// place.
    Committing,
    /// Every step is in place. Only cleanup is left.
    Done,
}

/// What happens to one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Action {
    /// Put this content at this path.
    Write {
        /// Lower case hex SHA-256 of the content that must end up there.
        sha256: String,
        size: u64,
    },
    /// Remove the file at this path.
    Delete,
}

/// How far one step got.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepState {
    /// Recorded, nothing written yet.
    Planned,
    /// The content is in staging and its hash has been checked.
    Staged,
    /// The file is in place and the old one, if any, is in backup.
    Committed,
}

/// One file in the operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// Position in the plan, and the name used under `staging` and
    /// `backup`.
    pub index: usize,
    /// Path relative to the install root, with forward slashes.
    pub path: String,
    pub action: Action,
    pub state: StepState,
    /// Whether a file was already there, which decides whether there is a
    /// backup to restore.
    #[serde(default)]
    pub had_previous: bool,
}

impl Step {
    /// The staged content for this step.
    pub fn staged_path(&self, install_root: &Path) -> PathBuf {
        staging_dir(install_root).join(format!("{:05}.new", self.index))
    }

    /// Where the file that was there before is kept.
    pub fn backup_path(&self, install_root: &Path) -> PathBuf {
        backup_dir(install_root).join(format!("{:05}.old", self.index))
    }

    /// The real location in the install.
    pub fn target_path(&self, install_root: &Path) -> PathBuf {
        install_root.join(self.path.replace('/', std::path::MAIN_SEPARATOR_STR))
    }
}

/// An operation in progress, or the record of one that was interrupted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journal {
    pub schema: u32,
    /// Identifies this operation in the log.
    pub id: String,
    pub operation: OperationKind,
    /// The version the install is moving to.
    pub version: Version,
    /// The version it was at, so a rollback knows where it came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_version: Option<Version>,
    pub started: String,
    pub phase: Phase,
    pub steps: Vec<Step>,
}

impl Journal {
    pub fn new(
        id: String,
        operation: OperationKind,
        version: Version,
        previous_version: Option<Version>,
        started: String,
        steps: Vec<Step>,
    ) -> Self {
        Journal {
            schema: JOURNAL_SCHEMA,
            id,
            operation,
            version,
            previous_version,
            started,
            phase: Phase::Preparing,
            steps,
        }
    }

    /// Whether the point of no return has been passed.
    pub fn is_committing(&self) -> bool {
        self.phase >= Phase::Committing
    }

    /// Whether every step reached at least the given state.
    pub fn all_steps_at_least(&self, state: StepState) -> bool {
        self.steps.iter().all(|step| step.state >= state)
    }

    /// Writes the journal so that a crash leaves either the old file or the
    /// new one, never a truncated mixture.
    ///
    /// The temporary file is flushed to the disk before the rename, so the
    /// rename cannot land before the content it points at.
    pub fn save(&self, install_root: &Path) -> Result<()> {
        let directory = state_dir(install_root);
        std::fs::create_dir_all(&directory).at(&directory)?;

        let final_path = journal_path(install_root);
        let temporary = directory.join(format!("{JOURNAL_FILE}.tmp"));

        let json = serde_json::to_vec_pretty(self).map_err(|error| Error::Journal {
            detail: format!("the journal could not be serialised: {error}"),
        })?;

        {
            use std::io::Write;
            let mut file = std::fs::File::create(&temporary).at(&temporary)?;
            file.write_all(&json).at(&temporary)?;
            file.sync_all().at(&temporary)?;
        }

        std::fs::rename(&temporary, &final_path).at(&final_path)?;
        Ok(())
    }

    /// Reads an unfinished journal, if there is one.
    ///
    /// A journal that cannot be parsed is an error rather than a shrug: the
    /// launcher must not start touching files when it does not know what a
    /// previous run was in the middle of.
    pub fn load(install_root: &Path) -> Result<Option<Journal>> {
        let path = journal_path(install_root);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(Error::io(&path, error)),
        };

        let journal: Journal = serde_json::from_slice(&bytes).map_err(|error| Error::Journal {
            detail: format!(
                "{} is not a journal this launcher understands: {error}",
                path.display()
            ),
        })?;

        if journal.schema != JOURNAL_SCHEMA {
            return Err(Error::Journal {
                detail: format!(
                    "journal schema {}, this launcher understands {JOURNAL_SCHEMA}",
                    journal.schema
                ),
            });
        }
        Ok(Some(journal))
    }

    /// Removes the journal and everything it was using.
    ///
    /// Called only when the operation has either fully finished or been
    /// fully undone, so there is nothing left to recover from.
    pub fn clear(install_root: &Path) -> Result<()> {
        remove_dir_if_present(&staging_dir(install_root))?;
        remove_dir_if_present(&backup_dir(install_root))?;
        let path = journal_path(install_root);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(Error::io(&path, error)),
        }
    }
}

/// `<install>/.recof`.
pub fn state_dir(install_root: &Path) -> PathBuf {
    install_root.join(STATE_DIR)
}

/// `<install>/.recof/journal.json`.
pub fn journal_path(install_root: &Path) -> PathBuf {
    state_dir(install_root).join(JOURNAL_FILE)
}

/// `<install>/.recof/staging`.
pub fn staging_dir(install_root: &Path) -> PathBuf {
    state_dir(install_root).join(STAGING_DIR)
}

/// `<install>/.recof/backup`.
pub fn backup_dir(install_root: &Path) -> PathBuf {
    state_dir(install_root).join(BACKUP_DIR)
}

pub(crate) use crate::fsutil::remove_dir_if_present;

#[cfg(test)]
mod tests {
    use super::*;

    fn journal() -> Journal {
        Journal::new(
            "20261026T120000-abcd".to_owned(),
            OperationKind::Update,
            Version::parse("0.4.0").unwrap(),
            Some(Version::parse("0.3.0").unwrap()),
            "2026-10-26T12:00:00Z".to_owned(),
            vec![Step {
                index: 0,
                path: "cryoffear/cl_dlls/hl.dll".to_owned(),
                action: Action::Write {
                    sha256: "a".repeat(64),
                    size: 10,
                },
                state: StepState::Planned,
                had_previous: true,
            }],
        )
    }

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-journal-state").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn phases_are_ordered_so_the_point_of_no_return_is_a_comparison() {
        assert!(Phase::Preparing < Phase::Staged);
        assert!(Phase::Staged < Phase::Committing);
        assert!(Phase::Committing < Phase::Done);

        let mut journal = journal();
        assert!(!journal.is_committing());
        journal.phase = Phase::Staged;
        assert!(!journal.is_committing());
        journal.phase = Phase::Committing;
        assert!(journal.is_committing());
        journal.phase = Phase::Done;
        assert!(journal.is_committing());
    }

    #[test]
    fn step_states_are_ordered_too() {
        assert!(StepState::Planned < StepState::Staged);
        assert!(StepState::Staged < StepState::Committed);
    }

    #[test]
    fn saving_and_loading_round_trips() {
        let root = scratch("round-trip");
        let original = journal();
        original.save(&root).unwrap();

        let loaded = Journal::load(&root)
            .unwrap()
            .expect("a journal was written");
        assert_eq!(loaded, original);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn no_journal_is_not_an_error() {
        let root = scratch("no-journal");
        assert_eq!(Journal::load(&root).unwrap(), None);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn saving_leaves_no_temporary_file_behind() {
        let root = scratch("no-temp");
        journal().save(&root).unwrap();

        let leftovers: Vec<_> = std::fs::read_dir(state_dir(&root))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(leftovers, vec!["journal.json".to_owned()]);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn saving_twice_replaces_the_previous_journal() {
        let root = scratch("overwrite");
        journal().save(&root).unwrap();

        let mut second = journal();
        second.phase = Phase::Committing;
        second.save(&root).unwrap();

        assert_eq!(
            Journal::load(&root).unwrap().unwrap().phase,
            Phase::Committing
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_corrupt_journal_is_an_error_not_a_silent_fresh_start() {
        let root = scratch("corrupt");
        std::fs::create_dir_all(state_dir(&root)).unwrap();
        std::fs::write(journal_path(&root), b"{ this is not json").unwrap();

        assert!(Journal::load(&root).is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_journal_from_a_newer_launcher_is_refused() {
        let root = scratch("future-schema");
        let mut journal = journal();
        journal.schema = JOURNAL_SCHEMA + 1;
        journal.save(&root).unwrap();

        assert!(Journal::load(&root).is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn clearing_removes_the_journal_and_its_working_folders() {
        let root = scratch("clear");
        journal().save(&root).unwrap();
        std::fs::create_dir_all(staging_dir(&root)).unwrap();
        std::fs::create_dir_all(backup_dir(&root)).unwrap();
        std::fs::write(staging_dir(&root).join("00000.new"), b"x").unwrap();

        Journal::clear(&root).unwrap();

        assert!(!journal_path(&root).exists());
        assert!(!staging_dir(&root).exists());
        assert!(!backup_dir(&root).exists());
        // Clearing again is not an error.
        Journal::clear(&root).unwrap();

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn step_paths_land_where_they_should() {
        let root = PathBuf::from("D:\\ReCoF");
        let step = journal().steps[0].clone();

        assert_eq!(
            step.target_path(&root),
            root.join("cryoffear").join("cl_dlls").join("hl.dll")
        );
        assert_eq!(
            step.staged_path(&root),
            root.join(".recof").join("staging").join("00000.new")
        );
        assert_eq!(
            step.backup_path(&root),
            root.join(".recof").join("backup").join("00000.old")
        );
    }

    #[test]
    fn all_steps_at_least_reports_the_weakest_step() {
        let mut journal = journal();
        journal.steps.push(Step {
            index: 1,
            path: "hw.dll".to_owned(),
            action: Action::Delete,
            state: StepState::Staged,
            had_previous: true,
        });

        assert!(journal.all_steps_at_least(StepState::Planned));
        assert!(!journal.all_steps_at_least(StepState::Staged));

        journal.steps[0].state = StepState::Committed;
        assert!(journal.all_steps_at_least(StepState::Staged));
    }

    #[test]
    fn a_path_with_spaces_and_non_ascii_round_trips() {
        let root = scratch("Librería ñ");
        let mut original = journal();
        original.steps[0].path = "cryoffear/mapas ñ/档案.bin".to_owned();
        original.save(&root).unwrap();

        let loaded = Journal::load(&root).unwrap().unwrap();
        assert_eq!(loaded.steps[0].path, "cryoffear/mapas ñ/档案.bin");

        std::fs::remove_dir_all(&root).unwrap();
    }
}
