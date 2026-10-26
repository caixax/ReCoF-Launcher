//! The recoverable journal.
//!
//! Every write of a game file goes through here. The rule the project
//! cannot break is that no operation may leave a half finished
//! installation, so the work is split into two halves with a single point
//! of no return between them:
//!
//! 1. **Staging.** Every new file is written into `.recof/staging`, hashed,
//!    and only then recorded as staged. Nothing in the game tree has been
//!    touched, so an interruption anywhere here is undone by deleting the
//!    staging folder.
//! 2. **Committing.** Once, and only once, every file is staged, the
//!    journal moves to the committing phase. From then on each file is put
//!    in place with two renames: the old file is moved aside into
//!    `.recof/backup`, and the staged file is renamed onto the target. An
//!    interruption here is finished on the next start, because everything
//!    needed is already on disk.
//!
//! So an unfinished journal is rolled back before the point of no return
//! and rolled forward after it, and either way the install ends up at
//! exactly one version.
//!
//! Renames are used rather than copies because `.recof` lives inside the
//! install, so source and target are always on the same volume, where a
//! rename is a metadata operation that either happened or did not.

mod apply;
mod recover;
mod state;

pub use apply::{apply, Applied, Interrupt, Plan, PlannedWrite, Progress};
pub use recover::{recover, Recovery};
pub use state::{
    journal_path, staging_dir, Action, Journal, OperationKind, Phase, Step, StepState,
    JOURNAL_SCHEMA,
};
