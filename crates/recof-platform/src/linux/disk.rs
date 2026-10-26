//! Free space on Linux.
//!
//! `statvfs` through libc would be the real answer. Pulling in libc for a
//! backend that is not in scope is not worth it, so this reports the
//! operation as unsupported and the caller shows the space check as
//! unavailable rather than claiming the disk is full.

use std::path::Path;

use crate::{DiskSpace, PlatformError, Result};

pub fn disk_space(_path: &Path) -> Result<DiskSpace> {
    Err(PlatformError::Unsupported("free disk space"))
}

pub fn volume_root(_path: &Path) -> Result<String> {
    Err(PlatformError::Unsupported("volume root"))
}

pub fn cluster_size(_path: &Path) -> Result<u64> {
    // The ext4 and btrfs default, and close enough for an estimate.
    Ok(4096)
}
