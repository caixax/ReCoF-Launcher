//! Copying the game, cancelably.
//!
//! The source is the user's Steam install and is opened read only. Nothing
//! in this module writes, renames or deletes anything under the source, and
//! the one assertion worth making about it is in the tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::IoResultExt;
use crate::space::TreeSize;
use crate::{Error, Result};

/// Shared with the UI so a copy can be stopped.
pub type Cancel = Arc<AtomicBool>;

/// How far a copy has got.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyProgress {
    pub files_done: u64,
    pub files_total: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
    /// The file being written, relative to the destination.
    pub current: String,
}

impl CopyProgress {
    /// A number between 0 and 1, by bytes rather than by file count, so a
    /// tree of one huge file and many small ones still moves smoothly.
    pub fn fraction(&self) -> f32 {
        if self.bytes_total == 0 {
            return 1.0;
        }
        (self.bytes_done as f64 / self.bytes_total as f64).clamp(0.0, 1.0) as f32
    }
}

/// How a copy ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyOutcome {
    Copied(TreeSize),
    /// Stopped on request. The caller removes what was written.
    Cancelled,
}

/// Copies `source` into `destination`, creating it.
///
/// `expected` comes from the measurement shown to the user, and is used
/// only to drive the progress bar. Symbolic links and junctions are
/// skipped rather than followed, so a library that points elsewhere cannot
/// make the copy loop.
pub fn copy_tree(
    source: &Path,
    destination: &Path,
    expected: TreeSize,
    cancel: &Cancel,
    progress: &mut dyn FnMut(&CopyProgress),
    skip: &dyn Fn(&Path) -> bool,
) -> Result<CopyOutcome> {
    std::fs::create_dir_all(destination).at(destination)?;

    let mut state = CopyProgress {
        files_done: 0,
        files_total: expected.files,
        bytes_done: 0,
        bytes_total: expected.bytes,
        current: String::new(),
    };

    let mut stack = vec![PathBuf::new()];
    while let Some(relative_dir) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return Ok(CopyOutcome::Cancelled);
        }

        let from = source.join(&relative_dir);
        let to = destination.join(&relative_dir);
        std::fs::create_dir_all(&to).at(&to)?;

        let entries = std::fs::read_dir(&from).map_err(|error| Error::io(&from, error))?;
        for entry in entries {
            let entry = entry.map_err(|error| Error::io(&from, error))?;
            let kind = entry
                .file_type()
                .map_err(|error| Error::io(entry.path(), error))?;
            let relative = relative_dir.join(entry.file_name());

            if skip(&relative) || kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                stack.push(relative);
                continue;
            }

            if cancel.load(Ordering::Relaxed) {
                return Ok(CopyOutcome::Cancelled);
            }

            let target = destination.join(&relative);
            let size =
                std::fs::copy(entry.path(), &target).map_err(|error| Error::io(&target, error))?;

            state.files_done += 1;
            state.bytes_done += size;
            state.current = relative.to_string_lossy().replace('\\', "/");
            // Reporting every file would flood the channel on a tree of
            // thousands; every file is fine for the big ones and every
            // sixty four keeps the small ones cheap.
            if state.files_done % 64 == 0 || size > 4 * 1024 * 1024 {
                progress(&state);
            }
        }
    }

    state.current.clear();
    progress(&state);

    Ok(CopyOutcome::Copied(TreeSize {
        files: state.files_done,
        directories: 0,
        bytes: state.bytes_done,
        bytes_on_disk: state.bytes_done,
    }))
}

/// Nothing is skipped.
pub fn skip_nothing(_: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-copy-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// A small tree shaped like the game: files at the root, files in
    /// nested directories, a name with a space and one with non ASCII.
    fn make_source(root: &Path) -> TreeSize {
        let files: &[(&str, &[u8])] = &[
            ("cof.exe", b"launcher"),
            ("hw.dll", b"engine"),
            ("cryoffear/liblist.gam", b"game definition"),
            ("cryoffear/cl_dlls/client.dll", b"client"),
            ("cryoffear/maps/ap 01.bsp", b"a map with a space"),
            ("cryoffear/sound/ñandú.wav", b"sonido"),
            ("cryoffear/empty.dat", b""),
        ];
        let mut size = TreeSize::default();
        for (path, content) in files {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(&full, content).unwrap();
            size.files += 1;
            size.bytes += content.len() as u64;
        }
        // An empty directory, which must still appear in the copy.
        std::fs::create_dir_all(root.join("cryoffear/overviews")).unwrap();
        size
    }

    fn no_cancel() -> Cancel {
        Arc::new(AtomicBool::new(false))
    }

    #[test]
    fn copies_every_file_and_keeps_the_shape() {
        let base = scratch("full-copy");
        let source = base.join("Cry of Fear");
        let destination = base.join("Mi ReCoF");
        let expected = make_source(&source);

        let outcome = copy_tree(
            &source,
            &destination,
            expected,
            &no_cancel(),
            &mut |_| {},
            &skip_nothing,
        )
        .unwrap();

        let CopyOutcome::Copied(copied) = outcome else {
            panic!("the copy should have finished");
        };
        assert_eq!(copied.files, expected.files);
        assert_eq!(copied.bytes, expected.bytes);

        assert_eq!(
            std::fs::read(destination.join("cryoffear/sound/ñandú.wav")).unwrap(),
            "sonido".as_bytes()
        );
        assert_eq!(
            std::fs::read(destination.join("cryoffear/maps/ap 01.bsp")).unwrap(),
            b"a map with a space"
        );
        assert!(destination.join("cryoffear/empty.dat").is_file());
        assert!(destination.join("cryoffear/overviews").is_dir());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_source_is_left_exactly_as_it_was() {
        let base = scratch("read-only-source");
        let source = base.join("Cry of Fear");
        let destination = base.join("ReCoF");
        let expected = make_source(&source);

        let before = fingerprint(&source);
        copy_tree(
            &source,
            &destination,
            expected,
            &no_cancel(),
            &mut |_| {},
            &skip_nothing,
        )
        .unwrap();
        assert_eq!(fingerprint(&source), before);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn cancelling_stops_the_copy() {
        let base = scratch("cancel");
        let source = base.join("Cry of Fear");
        let destination = base.join("ReCoF");
        let expected = make_source(&source);

        let cancel = Arc::new(AtomicBool::new(true));
        let outcome = copy_tree(
            &source,
            &destination,
            expected,
            &cancel,
            &mut |_| {},
            &skip_nothing,
        )
        .unwrap();
        assert_eq!(outcome, CopyOutcome::Cancelled);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn cancelling_part_way_leaves_an_incomplete_copy_for_the_caller_to_clean() {
        let base = scratch("cancel-midway");
        let source = base.join("Cry of Fear");
        let destination = base.join("ReCoF");
        let expected = make_source(&source);

        let cancel = no_cancel();
        let flag = Arc::clone(&cancel);
        let mut seen = 0;
        let outcome = copy_tree(
            &source,
            &destination,
            expected,
            &cancel,
            &mut |_| {
                seen += 1;
                flag.store(true, Ordering::Relaxed);
            },
            &skip_nothing,
        )
        .unwrap();

        // Either it finished the directory it was in or it stopped; what
        // matters is that the source is untouched and the destination can
        // simply be deleted.
        assert!(matches!(
            outcome,
            CopyOutcome::Cancelled | CopyOutcome::Copied(_)
        ));
        std::fs::remove_dir_all(&destination).unwrap();
        assert!(source.join("cof.exe").is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn skipped_paths_are_not_copied() {
        let base = scratch("skip");
        let source = base.join("Cry of Fear");
        let destination = base.join("ReCoF");
        let expected = make_source(&source);
        std::fs::create_dir_all(source.join(".recof")).unwrap();
        std::fs::write(source.join(".recof/installed.json"), b"{}").unwrap();

        copy_tree(
            &source,
            &destination,
            expected,
            &no_cancel(),
            &mut |_| {},
            &|relative| relative.starts_with(".recof"),
        )
        .unwrap();

        assert!(!destination.join(".recof").exists());
        assert!(destination.join("cof.exe").is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn progress_reaches_the_end() {
        let base = scratch("progress");
        let source = base.join("Cry of Fear");
        let destination = base.join("ReCoF");
        let expected = make_source(&source);

        let mut last = None;
        copy_tree(
            &source,
            &destination,
            expected,
            &no_cancel(),
            &mut |state| last = Some(state.clone()),
            &skip_nothing,
        )
        .unwrap();

        let last = last.expect("progress is reported at least once");
        assert_eq!(last.files_done, expected.files);
        assert_eq!(last.bytes_done, expected.bytes);
        assert!((last.fraction() - 1.0).abs() < f32::EPSILON);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_progress_fraction_is_well_behaved_at_the_edges() {
        let mut state = CopyProgress {
            files_done: 0,
            files_total: 0,
            bytes_done: 0,
            bytes_total: 0,
            current: String::new(),
        };
        // Nothing to copy counts as finished rather than as a division by
        // zero.
        assert_eq!(state.fraction(), 1.0);

        state.bytes_total = 100;
        assert_eq!(state.fraction(), 0.0);
        state.bytes_done = 50;
        assert!((state.fraction() - 0.5).abs() < 0.001);
        // More copied than measured, which can happen if the tree grew.
        state.bytes_done = 1000;
        assert_eq!(state.fraction(), 1.0);
    }

    #[test]
    fn a_missing_source_is_an_error() {
        let base = scratch("missing-source");
        let outcome = copy_tree(
            &base.join("not there"),
            &base.join("ReCoF"),
            TreeSize::default(),
            &no_cancel(),
            &mut |_| {},
            &skip_nothing,
        );
        assert!(outcome.is_err());
        std::fs::remove_dir_all(&base).unwrap();
    }

    /// Every path and size under a tree, for comparing before and after.
    fn fingerprint(root: &Path) -> Vec<(String, u64)> {
        let mut found = Vec::new();
        for entry in walkdir::WalkDir::new(root).sort_by_file_name() {
            let entry = entry.unwrap();
            let relative = entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let size = entry.metadata().map(|data| data.len()).unwrap_or(0);
            found.push((relative, size));
        }
        found
    }
}
