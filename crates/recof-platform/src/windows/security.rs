//! Windows security features that can stop the game from starting.

use super::registry::{read_u32, RegRoot};
use crate::Result;

/// Where Windows keeps the state of Smart App Control.
const CI_POLICY_KEY: &str = r"SYSTEM\CurrentControlSet\Control\CI\Policy";
const SAC_STATE_VALUE: &str = "VerifiedAndReputablePolicyState";
/// "On (enforcement)" in Microsoft's documentation; 0 is off and 2 is
/// evaluation, which blocks nothing.
const SAC_ENFORCING: u32 = 1;

/// Whether Smart App Control is on and blocking code that is neither
/// signed nor known to Microsoft.
///
/// ReCoF's DLLs are new builds without a signature, so with this on Windows
/// refuses to load them and cof.exe stops with a "Bad Image" box
/// (status 0xc0e90002). A machine without the value (Windows 10, or
/// Windows 11 that never had it) answers false.
pub fn smart_app_control_enforcing() -> Result<bool> {
    Ok(read_u32(RegRoot::LocalMachine, CI_POLICY_KEY, SAC_STATE_VALUE)? == Some(SAC_ENFORCING))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asking_about_smart_app_control_never_fails() {
        // Windows 10 and machines that never had it have no value, which
        // reads as off rather than as an error.
        assert!(smart_app_control_enforcing().is_ok());
    }
}
