//! Starting programs on Linux.

use std::path::Path;

use crate::{PlatformError, Result};

pub fn is_steam_running() -> Result<bool> {
    Err(PlatformError::Unsupported("checking whether Steam runs"))
}

pub fn is_running_by_name(_name: &str) -> Result<bool> {
    Err(PlatformError::Unsupported("looking for a process by name"))
}

pub fn spawn(program: &Path, args: &[String], working_dir: &Path) -> Result<u32> {
    let child = std::process::Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .spawn()
        .map_err(|error| PlatformError::Api {
            api: "spawn",
            detail: format!("{}: {error}", program.display()),
        })?;
    Ok(child.id())
}

pub fn open_in_shell(target: &str) -> Result<()> {
    std::process::Command::new("xdg-open")
        .arg(target)
        .spawn()
        .map_err(|error| PlatformError::Api {
            api: "xdg-open",
            detail: format!("{target}: {error}"),
        })?;
    Ok(())
}
