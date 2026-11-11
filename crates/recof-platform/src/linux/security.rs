//! Windows security features that can stop the game from starting.
//!
//! Smart App Control does not exist here.

use crate::Result;

/// Always false: there is no Smart App Control outside Windows.
pub fn smart_app_control_enforcing() -> Result<bool> {
    Ok(false)
}
