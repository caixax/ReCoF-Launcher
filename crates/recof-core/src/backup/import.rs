//! Bringing progress over from the Steam copy.
//!
//! One direction only. Files are read out of the Steam installation and
//! written into the ReCoF copy, and never the other way round, so there
//! is no path through this module that can write inside the Steam folder.
//! The function that copies takes the vanilla path as a read only source
//! and the ReCoF path as the only destination, and the test asserts the
//! Steam tree is unchanged afterwards.

use std::path::{Path, PathBuf};

use crate::backup::saves::SAVE_SET;
use crate::error::IoResultExt;
use crate::hashes::sha256_file;
use crate::{Error, Result};

/// One file that would be brought over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntry {
    /// Path relative to the install root, with forward slashes.
    pub relative: String,
    pub bytes: u64,
    /// Whether the ReCoF copy already has a file here.
    pub overwrites: bool,
    /// Whether the two files are already identical, in which case copying
    /// it changes nothing.
    pub identical: bool,
}

/// What an import would do, worked out before anything is written.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportPlan {
    pub entries: Vec<ImportEntry>,
}

impl ImportPlan {
    /// Whether there is anything at all to bring over.
    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(|entry| entry.identical)
    }

    /// Files that would replace something the user already has.
    pub fn overwrites(&self) -> Vec<&ImportEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.overwrites && !entry.identical)
            .collect()
    }

    /// How many files would actually be written.
    pub fn to_write(&self) -> usize {
        self.entries.iter().filter(|entry| !entry.identical).count()
    }

    pub fn total_bytes(&self) -> u64 {
        self.entries
            .iter()
            .filter(|entry| !entry.identical)
            .map(|entry| entry.bytes)
            .sum()
    }
}

/// What an import did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub copied: usize,
    pub skipped_identical: usize,
}

/// Works out what would be brought over, without writing anything.
pub fn plan(vanilla: &Path, recof: &Path) -> Result<ImportPlan> {
    let mut entries = Vec::new();

    for item in SAVE_SET {
        let source = vanilla.join(item.replace('/', std::path::MAIN_SEPARATOR_STR));
        if source.is_dir() {
            walk(&source, item, recof, &mut entries)?;
        } else if source.is_file() {
            entries.push(entry_for(item, &source, recof)?);
        }
    }

    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(ImportPlan { entries })
}

fn walk(
    directory: &Path,
    prefix: &str,
    recof: &Path,
    entries: &mut Vec<ImportEntry>,
) -> Result<()> {
    let listing = std::fs::read_dir(directory).at(directory)?;
    for item in listing {
        let item = item.at(directory)?;
        let kind = item.file_type().at(item.path())?;
        if kind.is_symlink() {
            continue;
        }
        let name = item.file_name().to_string_lossy().into_owned();
        let relative = format!("{prefix}/{name}");

        if kind.is_dir() {
            walk(&item.path(), &relative, recof, entries)?;
        } else {
            entries.push(entry_for(&relative, &item.path(), recof)?);
        }
    }
    Ok(())
}

fn entry_for(relative: &str, source: &Path, recof: &Path) -> Result<ImportEntry> {
    let bytes = std::fs::metadata(source).at(source)?.len();
    let target = recof.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let overwrites = target.is_file();

    // Comparing by hash rather than by size keeps the "this will replace
    // your files" warning honest: a file that is already the same is not
    // a replacement.
    let identical = if overwrites {
        match (sha256_file(source), sha256_file(&target)) {
            (Ok(left), Ok(right)) => left == right,
            _ => false,
        }
    } else {
        false
    };

    Ok(ImportEntry {
        relative: relative.to_owned(),
        bytes,
        overwrites,
        identical,
    })
}

/// Copies the planned files from the Steam copy into the ReCoF copy.
///
/// `vanilla` is opened read only. Nothing in this function writes,
/// renames or deletes anything under it.
pub fn import_from_vanilla(
    vanilla: &Path,
    recof: &Path,
    plan: &ImportPlan,
    progress: &mut dyn FnMut(usize, usize),
) -> Result<ImportReport> {
    if crate::paths::is_inside(recof, vanilla) {
        return Err(Error::Journal {
            detail: "the destination is inside the Steam installation".to_owned(),
        });
    }

    let total = plan.to_write();
    let mut copied = 0usize;
    let mut skipped = 0usize;

    for entry in &plan.entries {
        if entry.identical {
            skipped += 1;
            continue;
        }

        let safe =
            crate::manifest::safe_relative_path(&entry.relative).ok_or_else(|| Error::Journal {
                detail: format!("{:?} is not a path an import may write", entry.relative),
            })?;
        let source = vanilla.join(&safe);
        let target = recof.join(&safe);

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).at(parent)?;
        }
        std::fs::copy(&source, &target).map_err(|error| Error::io(&target, error))?;

        copied += 1;
        progress(copied, total);
    }

    log::info!(
        "imported {copied} files from {} into {}",
        vanilla.display(),
        recof.display()
    );
    Ok(ImportReport {
        copied,
        skipped_identical: skipped,
    })
}

/// Where the launcher would export saves to, for the uninstall flow.
pub fn export_name(stamp: &str) -> PathBuf {
    PathBuf::from(format!("recof-saves-{stamp}.zip"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-import-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn write(path: &Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn make_vanilla(root: &Path) {
        write(
            &root.join("cryoffear/SAVE/saveinfo1.cof"),
            b"the steam progress",
        );
        write(&root.join("cryoffear/SAVE/ap 01.sav"), b"a save");
        write(&root.join("cryoffear/scriptsettings.dat"), b"endings");
        write(&root.join("cryoffear/config.cfg"), b"bind w +forward");
        // Not progress, so it is never imported.
        write(&root.join("cryoffear/cl_dlls/hl.dll"), b"a binary");
    }

    /// Every file under a tree with its contents, for comparing.
    fn fingerprint(root: &Path) -> Vec<(String, Vec<u8>)> {
        let mut found = Vec::new();
        for entry in walkdir::WalkDir::new(root).sort_by_file_name() {
            let entry = entry.unwrap();
            if !entry.file_type().is_file() {
                continue;
            }
            found.push((
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                std::fs::read(entry.path()).unwrap(),
            ));
        }
        found
    }

    #[test]
    fn the_plan_lists_only_progress_files() {
        let base = scratch("plan");
        let vanilla = base.join("Cry of Fear");
        let recof = base.join("ReCoF");
        make_vanilla(&vanilla);
        std::fs::create_dir_all(&recof).unwrap();

        let plan = plan(&vanilla, &recof).unwrap();
        let names: Vec<&str> = plan
            .entries
            .iter()
            .map(|entry| entry.relative.as_str())
            .collect();

        assert_eq!(
            names,
            [
                "cryoffear/SAVE/ap 01.sav",
                "cryoffear/SAVE/saveinfo1.cof",
                "cryoffear/config.cfg",
                "cryoffear/scriptsettings.dat",
            ]
        );
        assert!(plan.overwrites().is_empty());
        assert_eq!(plan.to_write(), 4);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn importing_copies_the_files_and_never_touches_the_steam_copy() {
        let base = scratch("import");
        let vanilla = base.join("Cry of Fear");
        let recof = base.join("ReCoF");
        make_vanilla(&vanilla);
        std::fs::create_dir_all(&recof).unwrap();

        let before = fingerprint(&vanilla);
        let plan = plan(&vanilla, &recof).unwrap();
        let report = import_from_vanilla(&vanilla, &recof, &plan, &mut |_, _| {}).unwrap();

        assert_eq!(report.copied, 4);
        assert_eq!(report.skipped_identical, 0);
        assert_eq!(
            std::fs::read(recof.join("cryoffear/SAVE/saveinfo1.cof")).unwrap(),
            b"the steam progress"
        );
        assert!(!recof.join("cryoffear/cl_dlls/hl.dll").exists());

        assert_eq!(
            fingerprint(&vanilla),
            before,
            "the Steam installation was modified"
        );

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn files_that_would_be_replaced_are_reported_before_anything_is_written() {
        let base = scratch("overwrite");
        let vanilla = base.join("Cry of Fear");
        let recof = base.join("ReCoF");
        make_vanilla(&vanilla);
        write(
            &recof.join("cryoffear/SAVE/saveinfo1.cof"),
            b"progress made in ReCoF",
        );

        let plan = plan(&vanilla, &recof).unwrap();
        let overwrites = plan.overwrites();
        assert_eq!(overwrites.len(), 1);
        assert_eq!(overwrites[0].relative, "cryoffear/SAVE/saveinfo1.cof");

        // Making the plan wrote nothing.
        assert_eq!(
            std::fs::read(recof.join("cryoffear/SAVE/saveinfo1.cof")).unwrap(),
            b"progress made in ReCoF"
        );

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_file_that_is_already_identical_is_not_called_a_replacement() {
        let base = scratch("identical");
        let vanilla = base.join("Cry of Fear");
        let recof = base.join("ReCoF");
        make_vanilla(&vanilla);
        write(&recof.join("cryoffear/config.cfg"), b"bind w +forward");

        let plan = plan(&vanilla, &recof).unwrap();
        assert!(plan.overwrites().is_empty());
        assert_eq!(plan.to_write(), 3);

        let report = import_from_vanilla(&vanilla, &recof, &plan, &mut |_, _| {}).unwrap();
        assert_eq!(report.copied, 3);
        assert_eq!(report.skipped_identical, 1);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_vanilla_install_with_no_saves_gives_an_empty_plan() {
        let base = scratch("nothing");
        let vanilla = base.join("Cry of Fear");
        std::fs::create_dir_all(vanilla.join("cryoffear")).unwrap();
        let recof = base.join("ReCoF");
        std::fs::create_dir_all(&recof).unwrap();

        let plan = plan(&vanilla, &recof).unwrap();
        assert!(plan.is_empty());
        assert_eq!(plan.to_write(), 0);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn importing_into_the_steam_folder_is_refused() {
        let base = scratch("into-steam");
        let vanilla = base.join("Cry of Fear");
        make_vanilla(&vanilla);
        let inside = vanilla.join("copy");
        std::fs::create_dir_all(&inside).unwrap();

        let plan = plan(&vanilla, &inside).unwrap();
        let outcome = import_from_vanilla(&vanilla, &inside, &plan, &mut |_, _| {});
        assert!(outcome.is_err(), "nothing may write inside the Steam copy");

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn progress_counts_up_to_the_number_of_files_written() {
        let base = scratch("progress");
        let vanilla = base.join("Cry of Fear");
        let recof = base.join("ReCoF");
        make_vanilla(&vanilla);
        std::fs::create_dir_all(&recof).unwrap();

        let plan = plan(&vanilla, &recof).unwrap();
        let mut seen = Vec::new();
        import_from_vanilla(&vanilla, &recof, &plan, &mut |done, total| {
            seen.push((done, total))
        })
        .unwrap();

        assert_eq!(seen.len(), 4);
        assert_eq!(seen.last(), Some(&(4, 4)));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_export_name_carries_the_timestamp() {
        assert_eq!(
            export_name("20261026-120000"),
            PathBuf::from("recof-saves-20261026-120000.zip")
        );
    }
}
