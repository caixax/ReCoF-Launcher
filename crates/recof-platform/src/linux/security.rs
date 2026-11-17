//! Windows security features that can stop the game from starting.
//!
//! Neither Smart App Control nor Microsoft Defender exists here.

use std::path::{Path, PathBuf};

use crate::{PlatformError, Result};

/// Always false: there is no Smart App Control outside Windows.
pub fn smart_app_control_enforcing() -> Result<bool> {
    Ok(false)
}

/// Always false: there is no Microsoft Defender to ask.
pub fn defender_realtime_on() -> Result<bool> {
    Ok(false)
}

/// Always false, so nothing is ever offered for exclusion.
pub fn defender_exclusion_allowed(_path: &Path) -> bool {
    false
}

pub fn add_defender_exclusions(_paths: &[PathBuf]) -> Result<()> {
    Err(PlatformError::Unsupported("Windows Security exclusions"))
}
