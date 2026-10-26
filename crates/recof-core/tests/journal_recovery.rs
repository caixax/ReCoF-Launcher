//! The journal promise, tested by breaking it on purpose.
//!
//! An apply is interrupted at every single point it can be interrupted at,
//! recovery is run, and the install is checked to be either exactly what it
//! was or exactly what it was becoming. Never a mixture, and never with a
//! journal left behind.

use std::path::{Path, PathBuf};

use recof_core::hashes::sha256_bytes;
use recof_core::journal::{
    apply, journal_path, recover, Journal, OperationKind, Phase, Plan, PlannedWrite, Progress,
    Recovery, StepState,
};
use recof_core::version::Version;

/// Content of the install before the operation.
const BEFORE: &[(&str, &[u8])] = &[
    ("cof.exe", b"old launcher"),
    ("hw.dll", b"old engine"),
    ("sw.dll", b"old engine"),
    ("cryoffear/cl_dlls/hl.dll", b"old server"),
    ("cryoffear/cl_dlls/client.dll", b"old client"),
    // A file the new release drops.
    ("obsolete.dll", b"to be removed"),
    // A file nothing in the plan touches, which must survive untouched.
    ("cryoffear/config.cfg", b"the player settings"),
];

/// Content the operation installs.
const AFTER: &[(&str, &[u8])] = &[
    ("cof.exe", b"new launcher"),
    ("hw.dll", b"new engine build"),
    ("sw.dll", b"new engine build"),
    ("cryoffear/cl_dlls/hl.dll", b"new server"),
    ("cryoffear/cl_dlls/client.dll", b"new client"),
    // A file the release adds, in a directory that does not exist yet.
    ("cryoffear/cl_dlls/GameUI.dll", b"brand new file"),
];

const DELETES: &[&str] = &["obsolete.dll"];

struct Fixture {
    root: PathBuf,
    store: PathBuf,
}

impl Fixture {
    /// A fresh install and version store. The names carry a space and a non
    /// ASCII character because real Steam libraries do.
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir()
            .join("recof journal tests ñ")
            .join(name);
        let _ = std::fs::remove_dir_all(&base);

        let root = base.join("Mi copia de ReCoF");
        let store = base.join("version store");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&store).unwrap();

        for (path, content) in BEFORE {
            write(&root.join(path), content);
        }
        for (path, content) in AFTER {
            // The store keeps one file per content, flat, as the extractor
            // leaves it.
            write(&store.join(flat_name(path)), content);
        }

        Fixture { root, store }
    }

    fn plan(&self) -> Plan {
        Plan {
            operation: OperationKind::Update,
            version: Version::parse("0.4.0").unwrap(),
            previous_version: Some(Version::parse("0.3.0").unwrap()),
            writes: AFTER
                .iter()
                .map(|(path, content)| PlannedWrite {
                    path: (*path).to_owned(),
                    source: self.store.join(flat_name(path)),
                    sha256: sha256_bytes(content),
                    size: content.len() as u64,
                })
                .collect(),
            deletes: DELETES.iter().map(|path| (*path).to_owned()).collect(),
        }
    }

    fn assert_is_before(&self, context: &str) {
        for (path, content) in BEFORE {
            assert_eq!(
                read(&self.root.join(path)).as_deref(),
                Some(*content),
                "{context}: {path} should still hold the old content"
            );
        }
        assert!(
            !self.root.join("cryoffear/cl_dlls/GameUI.dll").exists(),
            "{context}: the new file should not exist after a rollback"
        );
    }

    fn assert_is_after(&self, context: &str) {
        for (path, content) in AFTER {
            assert_eq!(
                read(&self.root.join(path)).as_deref(),
                Some(*content),
                "{context}: {path} should hold the new content"
            );
        }
        for path in DELETES {
            assert!(
                !self.root.join(path).exists(),
                "{context}: {path} should have been deleted"
            );
        }
        // A file outside the plan is never touched either way.
        assert_eq!(
            read(&self.root.join("cryoffear/config.cfg")).as_deref(),
            Some(&b"the player settings"[..]),
            "{context}: an untouched file was changed"
        );
    }

    fn assert_no_journal_left(&self, context: &str) {
        assert!(
            !journal_path(&self.root).exists(),
            "{context}: a journal was left behind"
        );
        assert!(
            !self.root.join(".recof/staging").exists(),
            "{context}: the staging folder was left behind"
        );
        assert!(
            !self.root.join(".recof/backup").exists(),
            "{context}: the backup folder was left behind"
        );
    }
}

fn flat_name(path: &str) -> String {
    path.replace('/', "_")
}

fn write(path: &Path, content: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn read(path: &Path) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

fn no_progress(_: Progress) {}

/// Every point the apply can be interrupted at, in the order they happen.
fn interruption_points(fixture: &Fixture) -> Vec<String> {
    let mut points = Vec::new();
    let mut hook = |point: &str, _: &Journal| {
        points.push(point.to_owned());
        false
    };
    apply(
        &fixture.root,
        &fixture.plan(),
        "2026-10-26T12:00:00Z",
        "test-run",
        &mut no_progress,
        Some(&mut hook),
    )
    .expect("an uninterrupted apply succeeds");
    points
}

#[test]
fn an_uninterrupted_apply_installs_everything_and_leaves_nothing_behind() {
    let fixture = Fixture::new("clean-run");
    let applied = apply(
        &fixture.root,
        &fixture.plan(),
        "2026-10-26T12:00:00Z",
        "test-run",
        &mut no_progress,
        None,
    )
    .expect("the apply succeeds");

    assert_eq!(applied.version, Version::parse("0.4.0").unwrap());
    assert_eq!(applied.files_written, AFTER.len());
    assert_eq!(applied.files_deleted, DELETES.len());

    fixture.assert_is_after("clean run");
    fixture.assert_no_journal_left("clean run");
}

#[test]
fn recovery_leaves_the_install_whole_whatever_moment_it_was_interrupted_at() {
    let points = interruption_points(&Fixture::new("collect-points"));
    assert!(
        points.len() > 20,
        "expected an interruption point per step, got {}: {points:?}",
        points.len()
    );

    // Before this point nothing in the game tree has been touched.
    let commit_at = points
        .iter()
        .position(|point| point == "after-phase-committing")
        .expect("the point of no return is one of the points");

    for (index, point) in points.iter().enumerate() {
        let fixture = Fixture::new(&format!("interrupt-{index:03}"));
        let context = format!("interrupted at {point:?}");

        let outcome = apply(
            &fixture.root,
            &fixture.plan(),
            "2026-10-26T12:00:00Z",
            "test-run",
            &mut no_progress,
            Some(&mut |seen: &str, _: &Journal| seen == point),
        );
        assert!(
            matches!(outcome, Err(recof_core::Error::Interrupted { .. })),
            "{context}: the apply should have stopped, got {outcome:?}"
        );

        // Exactly what happens on the next start.
        let recovery = recover(&fixture.root, &mut no_progress)
            .unwrap_or_else(|error| panic!("{context}: recovery failed: {error}"));

        if index < commit_at {
            assert!(
                matches!(recovery, Recovery::RolledBack { .. }),
                "{context}: expected a rollback, got {recovery:?}"
            );
            fixture.assert_is_before(&context);
        } else {
            assert!(
                matches!(recovery, Recovery::RolledForward { .. }),
                "{context}: expected a roll forward, got {recovery:?}"
            );
            fixture.assert_is_after(&context);
        }
        fixture.assert_no_journal_left(&context);

        // Recovering again has nothing to do.
        assert_eq!(
            recover(&fixture.root, &mut no_progress).unwrap(),
            Recovery::Nothing,
            "{context}: a second recovery should find nothing"
        );
    }
}

#[test]
fn recovery_on_a_clean_install_does_nothing() {
    let fixture = Fixture::new("clean-recovery");
    let recovery = recover(&fixture.root, &mut no_progress).unwrap();
    assert_eq!(recovery, Recovery::Nothing);
    assert!(!recovery.happened());
    assert!(recovery.message_id().is_none());
    fixture.assert_is_before("clean recovery");
}

#[test]
fn an_apply_refuses_to_start_on_top_of_an_unfinished_one() {
    let fixture = Fixture::new("already-running");
    let _ = apply(
        &fixture.root,
        &fixture.plan(),
        "2026-10-26T12:00:00Z",
        "first",
        &mut no_progress,
        Some(&mut |point: &str, _: &Journal| point == "staged-recorded-1"),
    );
    assert!(journal_path(&fixture.root).exists());

    let second = apply(
        &fixture.root,
        &fixture.plan(),
        "2026-10-26T12:05:00Z",
        "second",
        &mut no_progress,
        None,
    );
    assert!(
        matches!(second, Err(recof_core::Error::Journal { .. })),
        "a second apply should refuse, got {second:?}"
    );

    // And the first one is still recoverable.
    recover(&fixture.root, &mut no_progress).unwrap();
    fixture.assert_is_before("after refusing a second apply");
}

#[test]
fn content_that_does_not_match_the_manifest_never_reaches_the_game_tree() {
    let fixture = Fixture::new("bad-content");
    let mut plan = fixture.plan();
    // The store holds something other than what the plan promises.
    plan.writes[2].sha256 = sha256_bytes(b"not what is in the store");

    let outcome = apply(
        &fixture.root,
        &fixture.plan_with(plan.writes.clone(), plan.deletes.clone()),
        "2026-10-26T12:00:00Z",
        "bad",
        &mut no_progress,
        None,
    );
    assert!(
        matches!(outcome, Err(recof_core::Error::Journal { .. })),
        "a hash mismatch must stop the apply, got {outcome:?}"
    );

    // The tree is untouched, and recovery confirms it.
    fixture.assert_is_before("hash mismatch");
    assert!(matches!(
        recover(&fixture.root, &mut no_progress).unwrap(),
        Recovery::RolledBack { .. }
    ));
    fixture.assert_is_before("hash mismatch, after recovery");
    fixture.assert_no_journal_left("hash mismatch");
}

#[test]
fn a_plan_that_writes_outside_the_install_is_refused() {
    let fixture = Fixture::new("escape");
    for bad in [
        "../outside.dll",
        "C:/Windows/System32/evil.dll",
        "a/../../b",
    ] {
        let mut plan = fixture.plan();
        plan.writes[0].path = bad.to_owned();

        let outcome = apply(
            &fixture.root,
            &plan,
            "2026-10-26T12:00:00Z",
            "escape",
            &mut no_progress,
            None,
        );
        assert!(
            matches!(outcome, Err(recof_core::Error::Journal { .. })),
            "{bad:?} should be refused, got {outcome:?}"
        );
        assert!(!journal_path(&fixture.root).exists(), "{bad:?}");
    }
    fixture.assert_is_before("escape attempt");
}

#[test]
fn the_journal_records_the_phase_it_reached() {
    let fixture = Fixture::new("phases");

    // Stopped while staging: nothing committed.
    let _ = apply(
        &fixture.root,
        &fixture.plan(),
        "2026-10-26T12:00:00Z",
        "run",
        &mut no_progress,
        Some(&mut |point: &str, _: &Journal| point == "staged-recorded-2"),
    );
    let journal = Journal::load(&fixture.root).unwrap().unwrap();
    assert_eq!(journal.phase, Phase::Preparing);
    assert!(!journal.is_committing());
    assert_eq!(journal.steps[0].state, StepState::Staged);
    assert_eq!(journal.steps[5].state, StepState::Planned);
    assert_eq!(journal.version, Version::parse("0.4.0").unwrap());
    assert_eq!(
        journal.previous_version,
        Some(Version::parse("0.3.0").unwrap())
    );

    recover(&fixture.root, &mut no_progress).unwrap();
    fixture.assert_is_before("phases");
}

#[test]
fn a_journal_that_cannot_be_read_stops_the_launcher_rather_than_guessing() {
    let fixture = Fixture::new("corrupt-journal");
    std::fs::create_dir_all(fixture.root.join(".recof")).unwrap();
    std::fs::write(journal_path(&fixture.root), b"not json at all").unwrap();

    assert!(recover(&fixture.root, &mut no_progress).is_err());
    fixture.assert_is_before("corrupt journal");
}

#[test]
fn progress_is_reported_for_every_file() {
    let fixture = Fixture::new("progress");
    let mut staging = 0usize;
    let mut committing = 0usize;
    let mut finished = 0usize;

    apply(
        &fixture.root,
        &fixture.plan(),
        "2026-10-26T12:00:00Z",
        "run",
        &mut |progress| match progress {
            Progress::Staging { .. } => staging += 1,
            Progress::Committing { .. } => committing += 1,
            Progress::Finished => finished += 1,
        },
        None,
    )
    .unwrap();

    let total = AFTER.len() + DELETES.len();
    assert_eq!(staging, total);
    assert_eq!(committing, total);
    assert_eq!(finished, 1);
}

impl Fixture {
    /// Builds a plan from parts, for the tests that change one field.
    fn plan_with(&self, writes: Vec<PlannedWrite>, deletes: Vec<String>) -> Plan {
        Plan {
            operation: OperationKind::Update,
            version: Version::parse("0.4.0").unwrap(),
            previous_version: Some(Version::parse("0.3.0").unwrap()),
            writes,
            deletes,
        }
    }
}
