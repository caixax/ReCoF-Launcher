//! XDG equivalents of the Windows known folders.

use std::path::PathBuf;

use crate::{PlatformError, Result};

/// `$XDG_DATA_HOME`, falling back to `~/.local/share`.
pub fn local_app_data() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var_os("HOME").ok_or(PlatformError::Unsupported("HOME is not set"))?;
    Ok(PathBuf::from(home).join(".local").join("share"))
}

pub fn desktop_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or(PlatformError::Unsupported("HOME is not set"))?;
    Ok(PathBuf::from(home).join("Desktop"))
}

pub fn start_menu_programs_dir() -> Result<PathBuf> {
    Ok(local_app_data()?.join("applications"))
}

/// Directories an install must never be placed in.
pub fn protected_dirs() -> Vec<PathBuf> {
    ["/usr", "/bin", "/sbin", "/etc", "/boot"]
        .iter()
        .map(PathBuf::from)
        .collect()
}
