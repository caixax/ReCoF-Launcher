//! Measuring the copy and checking it fits.
//!
//! The rule is that an install either has room for everything it will need
//! or does not start. "Everything" is more than the copy of the game: the
//! version store and the save backups live under `%LOCALAPPDATA%`, which
//! may well be on a different volume from the copy, so both are checked.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use recof_platform::DiskSpace;

use crate::{Error, Result};

/// Room kept for the version store: a handful of ReCoF releases, each a few
/// megabytes of rebuilt binaries, kept so the user can roll back.
pub const RESERVE_VERSION_STORE: u64 = 256 * 1024 * 1024;

/// Room kept for save backups. A Cry of Fear save set is small; this allows
/// for many of them plus the automatic one taken before each update.
pub const RESERVE_BACKUPS: u64 = 512 * 1024 * 1024;

/// Room left free on the volume regardless, so filling the disk to the last
/// byte is never the outcome of an install.
pub const HEADROOM: u64 = 1024 * 1024 * 1024;

/// What a directory tree weighs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeSize {
    pub files: u64,
    pub directories: u64,
    /// Sum of the file sizes.
    pub bytes: u64,
    /// Sum of the file sizes rounded up to whole allocation units, which is
    /// what the copy will actually take.
    pub bytes_on_disk: u64,
}

/// Progress of a measurement, so a slow disk does not look like a hang.
pub type SizeProgress<'a> = &'a mut dyn FnMut(&TreeSize);

/// A cancel flag shared with the UI thread.
pub type Cancel = Arc<AtomicBool>;

/// Measuring was stopped on request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

/// Measures a tree, rounding every file up to the allocation unit of the
/// volume it will be copied to.
///
/// Symbolic links and junctions are not followed: a library that points
/// somewhere else must not make the estimate loop or double count.
pub fn measure_tree(
    root: &Path,
    cluster_size: u64,
    cancel: &Cancel,
    progress: SizeProgress<'_>,
) -> Result<std::result::Result<TreeSize, Cancelled>> {
    let mut total = TreeSize::default();
    let mut stack = vec![root.to_path_buf()];
    let mut since_report = 0u64;

    while let Some(directory) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return Ok(Err(Cancelled));
        }

        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            // A folder that vanished or cannot be read is reported rather
            // than silently making the estimate too small.
            Err(error) => return Err(Error::io(&directory, error)),
        };

        for entry in entries {
            let entry = entry.map_err(|error| Error::io(&directory, error))?;
            let kind = entry
                .file_type()
                .map_err(|error| Error::io(entry.path(), error))?;

            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                total.directories += 1;
                stack.push(entry.path());
                continue;
            }

            let size = entry
                .metadata()
                .map_err(|error| Error::io(entry.path(), error))?
                .len();
            total.files += 1;
            total.bytes += size;
            total.bytes_on_disk += round_up(size, cluster_size);

            since_report += 1;
            if since_report >= 512 {
                since_report = 0;
                progress(&total);
            }
        }
    }

    progress(&total);
    Ok(Ok(total))
}

/// Rounds a file size up to a whole number of allocation units.
pub fn round_up(size: u64, cluster_size: u64) -> u64 {
    if cluster_size == 0 {
        return size;
    }
    // An empty file still occupies a directory entry but no cluster.
    if size == 0 {
        return 0;
    }
    size.div_ceil(cluster_size) * cluster_size
}

/// Everything the install needs and everything the disks have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpacePlan {
    /// What the game copy will take.
    pub copy: TreeSize,
    /// Where the copy is going.
    pub destination: PathBuf,
    pub destination_space: DiskSpace,
    /// Where the version store, backups and logs go.
    pub app_data: PathBuf,
    pub app_data_space: DiskSpace,
    /// Whether both land on the same volume, in which case the two
    /// requirements add up instead of being checked separately.
    pub same_volume: bool,
}

/// A volume that does not have room, and by how much.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortfall {
    pub volume: String,
    pub needed: u64,
    pub available: u64,
}

impl Shortfall {
    pub fn missing(&self) -> u64 {
        self.needed.saturating_sub(self.available)
    }
}

impl SpacePlan {
    /// Works out what is needed and what is there.
    pub fn new(copy: TreeSize, destination: &Path, app_data: &Path) -> Result<Self> {
        let destination_space = recof_platform::disk_space(destination)?;
        let app_data_space = recof_platform::disk_space(app_data)?;
        let same_volume = destination_space
            .volume
            .eq_ignore_ascii_case(&app_data_space.volume);

        Ok(SpacePlan {
            copy,
            destination: destination.to_path_buf(),
            destination_space,
            app_data: app_data.to_path_buf(),
            app_data_space,
            same_volume,
        })
    }

    /// What the version store and the backups need together.
    pub fn app_data_needed(&self) -> u64 {
        RESERVE_VERSION_STORE + RESERVE_BACKUPS
    }

    /// What the destination volume needs.
    pub fn destination_needed(&self) -> u64 {
        if self.same_volume {
            self.copy.bytes_on_disk + self.app_data_needed()
        } else {
            self.copy.bytes_on_disk
        }
    }

    /// The volumes that do not have room, if any.
    pub fn shortfalls(&self) -> Vec<Shortfall> {
        let mut found = Vec::new();

        if !self
            .destination_space
            .fits(self.destination_needed(), HEADROOM)
        {
            found.push(Shortfall {
                volume: self.destination_space.volume.clone(),
                needed: self.destination_needed() + HEADROOM,
                available: self.destination_space.available,
            });
        }

        if !self.same_volume && !self.app_data_space.fits(self.app_data_needed(), HEADROOM) {
            found.push(Shortfall {
                volume: self.app_data_space.volume.clone(),
                needed: self.app_data_needed() + HEADROOM,
                available: self.app_data_space.available,
            });
        }

        found
    }

    /// Whether the install may go ahead.
    pub fn fits(&self) -> bool {
        self.shortfalls().is_empty()
    }
}

/// Formats a byte count the way the UI shows it, in binary units.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(volume: &str, available: u64) -> DiskSpace {
        DiskSpace {
            volume: volume.to_owned(),
            available,
            total: available * 2,
        }
    }

    fn plan(copy_bytes: u64, destination: DiskSpace, app_data: DiskSpace) -> SpacePlan {
        let same_volume = destination.volume.eq_ignore_ascii_case(&app_data.volume);
        SpacePlan {
            copy: TreeSize {
                files: 1,
                directories: 0,
                bytes: copy_bytes,
                bytes_on_disk: copy_bytes,
            },
            destination: PathBuf::from("D:\\ReCoF"),
            destination_space: destination,
            app_data: PathBuf::from("C:\\Users\\x\\AppData\\Local"),
            app_data_space: app_data,
            same_volume,
        }
    }

    #[test]
    fn rounding_matches_how_a_filesystem_allocates() {
        assert_eq!(round_up(0, 4096), 0);
        assert_eq!(round_up(1, 4096), 4096);
        assert_eq!(round_up(4096, 4096), 4096);
        assert_eq!(round_up(4097, 4096), 8192);
        assert_eq!(round_up(12345, 4096), 16384);
        // A volume that reported nothing must not divide by zero.
        assert_eq!(round_up(1234, 0), 1234);
    }

    #[test]
    fn two_volumes_are_checked_separately() {
        let copy = 5 * 1024 * 1024 * 1024;
        let plan = plan(
            copy,
            space("D:\\", copy + HEADROOM),
            space("C:\\", RESERVE_VERSION_STORE + RESERVE_BACKUPS + HEADROOM),
        );
        assert!(!plan.same_volume);
        assert_eq!(plan.destination_needed(), copy);
        assert!(plan.fits(), "{:?}", plan.shortfalls());
    }

    #[test]
    fn one_volume_has_to_hold_both() {
        let copy = 5 * 1024 * 1024 * 1024;
        // Exactly enough for the copy, but not for the store and backups.
        let available = copy + HEADROOM;
        let plan = plan(copy, space("C:\\", available), space("C:\\", available));
        assert!(plan.same_volume);
        assert_eq!(
            plan.destination_needed(),
            copy + RESERVE_VERSION_STORE + RESERVE_BACKUPS
        );
        assert!(!plan.fits());

        let shortfalls = plan.shortfalls();
        assert_eq!(shortfalls.len(), 1);
        assert_eq!(shortfalls[0].volume, "C:\\");
        assert_eq!(
            shortfalls[0].missing(),
            RESERVE_VERSION_STORE + RESERVE_BACKUPS
        );
    }

    #[test]
    fn headroom_is_never_spent() {
        let copy = 1000u64;
        // Enough for the copy and the reserves, but nothing left over.
        let available = copy + RESERVE_VERSION_STORE + RESERVE_BACKUPS;
        let plan = plan(copy, space("C:\\", available), space("C:\\", available));
        assert!(!plan.fits());

        let plan = plan2(copy, available + HEADROOM);
        assert!(plan.fits());
    }

    fn plan2(copy_bytes: u64, available: u64) -> SpacePlan {
        plan(
            copy_bytes,
            space("C:\\", available),
            space("C:\\", available),
        )
    }

    #[test]
    fn a_full_appdata_volume_is_reported_on_its_own() {
        let copy = 1024u64;
        let plan = plan(copy, space("D:\\", u64::MAX / 4), space("C:\\", 1024));
        let shortfalls = plan.shortfalls();
        assert_eq!(shortfalls.len(), 1);
        assert_eq!(shortfalls[0].volume, "C:\\");
    }

    #[test]
    fn both_volumes_can_be_short_at_once() {
        let plan = plan(u64::MAX / 4, space("D:\\", 1), space("C:\\", 1));
        assert_eq!(plan.shortfalls().len(), 2);
    }

    #[test]
    fn measuring_counts_files_directories_and_clusters() {
        let root = std::env::temp_dir().join("recof-measure-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("sub directorio/ñ")).unwrap();
        std::fs::write(root.join("a.bin"), vec![0u8; 100]).unwrap();
        std::fs::write(root.join("sub directorio/b.bin"), vec![0u8; 4097]).unwrap();
        std::fs::write(root.join("sub directorio/ñ/c.bin"), Vec::new()).unwrap();

        let cancel: Cancel = Arc::new(AtomicBool::new(false));
        let mut seen = 0usize;
        let size = measure_tree(&root, 4096, &cancel, &mut |_| seen += 1)
            .unwrap()
            .unwrap();

        assert_eq!(size.files, 3);
        assert_eq!(size.directories, 2);
        assert_eq!(size.bytes, 100 + 4097);
        assert_eq!(size.bytes_on_disk, 4096 + 8192);
        assert!(seen >= 1, "progress is reported at least once");

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn measuring_an_empty_tree_is_zero() {
        let root = std::env::temp_dir().join("recof-measure-empty");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let cancel: Cancel = Arc::new(AtomicBool::new(false));
        let size = measure_tree(&root, 4096, &cancel, &mut |_| {})
            .unwrap()
            .unwrap();
        assert_eq!(size, TreeSize::default());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn cancelling_stops_the_measurement() {
        let root = std::env::temp_dir().join("recof-measure-cancel");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("a.bin"), vec![0u8; 10]).unwrap();

        let cancel: Cancel = Arc::new(AtomicBool::new(true));
        let outcome = measure_tree(&root, 4096, &cancel, &mut |_| {}).unwrap();
        assert_eq!(outcome, Err(Cancelled));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_root_is_an_error_not_a_zero_estimate() {
        let cancel: Cancel = Arc::new(AtomicBool::new(false));
        let missing = std::env::temp_dir().join("recof-measure-missing-xyz");
        let _ = std::fs::remove_dir_all(&missing);
        assert!(measure_tree(&missing, 4096, &cancel, &mut |_| {}).is_err());
    }

    #[test]
    fn byte_formatting_reads_the_way_a_user_expects() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(4_695_601_334), "4.4 GiB");
        assert_eq!(format_bytes(150 * 1024 * 1024), "150 MiB");
    }
}
