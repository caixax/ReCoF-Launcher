//! Desktop entries on Linux.
//!
//! A `.desktop` file is the equivalent and is straightforward, but the
//! rest of the Linux port is not in scope, so this reports the operation
//! as unsupported rather than writing a file nothing else here manages.

use std::path::Path;

use crate::{PlatformError, Result};

#[derive(Debug, Clone)]
pub struct Shortcut<'a> {
    pub link_path: &'a Path,
    pub target: &'a Path,
    pub working_dir: &'a Path,
    pub arguments: &'a str,
    pub description: &'a str,
}

pub fn create_shortcut(_shortcut: &Shortcut<'_>) -> Result<()> {
    Err(PlatformError::Unsupported("creating a shortcut"))
}
