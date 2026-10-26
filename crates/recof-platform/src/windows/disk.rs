//! Free space, asked of the volume rather than guessed.

use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDiskFreeSpaceW, GetVolumePathNameW,
};

use crate::{DiskSpace, PlatformError, Result};

/// Free and total bytes on the volume that holds `path`.
///
/// The path does not have to exist yet: the volume is resolved with
/// `GetVolumePathNameW` first, which works on a path that is only planned.
/// The free figure is the quota aware one, so a disk quota is respected.
pub fn disk_space(path: &Path) -> Result<DiskSpace> {
    let root = volume_root(path)?;
    let wide_root = wide(&root);

    let mut available_to_caller: u64 = 0;
    let mut total: u64 = 0;
    let mut total_free: u64 = 0;

    // SAFETY: the string is NUL terminated and outlives the call, and the
    // three out pointers are valid for the duration.
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide_root.as_ptr()),
            Some(&mut available_to_caller),
            Some(&mut total),
            Some(&mut total_free),
        )
    }
    .map_err(|error| PlatformError::Api {
        api: "GetDiskFreeSpaceExW",
        detail: format!("{root}: {error}"),
    })?;

    Ok(DiskSpace {
        volume: root,
        available: available_to_caller,
        total,
    })
}

/// The mount point that holds `path`, for example `D:\` or
/// `C:\mounted\volume\`.
pub fn volume_root(path: &Path) -> Result<String> {
    let wide_path = wide(&path.to_string_lossy());
    let mut buffer = [0u16; 260];

    // SAFETY: both buffers are valid and the length is passed by the binding.
    unsafe { GetVolumePathNameW(PCWSTR(wide_path.as_ptr()), &mut buffer) }.map_err(|error| {
        PlatformError::Api {
            api: "GetVolumePathNameW",
            detail: format!("{}: {error}", path.display()),
        }
    })?;

    let length = buffer.iter().position(|unit| *unit == 0).unwrap_or(0);
    Ok(String::from_utf16_lossy(&buffer[..length]))
}

/// Allocation unit of the volume that holds `path`, in bytes.
///
/// A file always takes up a whole number of clusters, so a tree of many
/// small files costs more than the sum of its sizes. The install estimate
/// rounds every file up by this.
pub fn cluster_size(path: &Path) -> Result<u64> {
    let root = volume_root(path)?;
    let wide_root = wide(&root);

    let mut sectors_per_cluster: u32 = 0;
    let mut bytes_per_sector: u32 = 0;

    // SAFETY: the string is NUL terminated and the out pointers are valid.
    unsafe {
        GetDiskFreeSpaceW(
            PCWSTR(wide_root.as_ptr()),
            Some(&mut sectors_per_cluster),
            Some(&mut bytes_per_sector),
            None,
            None,
        )
    }
    .map_err(|error| PlatformError::Api {
        api: "GetDiskFreeSpaceW",
        detail: format!("{root}: {error}"),
    })?;

    let cluster = u64::from(sectors_per_cluster) * u64::from(bytes_per_sector);
    // A volume reporting zero would turn the estimate into a division by
    // zero later. 4 KiB is the NTFS default.
    Ok(if cluster == 0 { 4096 } else { cluster })
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_space_for_the_temp_directory() {
        let space = disk_space(&std::env::temp_dir()).expect("temp is on a real volume");
        assert!(space.total > 0);
        assert!(space.available <= space.total);
        assert!(space.volume.ends_with('\\'));
    }

    #[test]
    fn works_for_a_path_that_does_not_exist_yet() {
        let planned = std::env::temp_dir().join("recof-does-not-exist-yet/ReCoF");
        let space = disk_space(&planned).expect("the volume is resolvable without the path");
        assert!(space.total > 0);
    }

    #[test]
    fn cluster_size_is_a_sane_power_of_two() {
        let cluster = cluster_size(&std::env::temp_dir()).expect("temp is on a real volume");
        assert!(cluster >= 512, "{cluster}");
        assert!(cluster.is_power_of_two(), "{cluster}");
    }

    #[test]
    fn a_path_with_spaces_and_non_ascii_resolves() {
        let planned = std::env::temp_dir().join("Librería Steam/Mi ReCoF/juego");
        assert!(disk_space(&planned).is_ok());
    }
}
