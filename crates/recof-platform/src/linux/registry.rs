//! There is no registry on Linux.
//!
//! The Steam paths the Windows backend reads from the registry live in
//! `~/.steam/steam` and friends instead. Finding them is part of the Linux
//! port, which is not in scope, so every read answers "not there" rather
//! than failing: callers already handle a missing Steam install.

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegRoot {
    CurrentUser,
    LocalMachine,
}

pub fn read_string(_root: RegRoot, _subkey: &str, _value: &str) -> Result<Option<String>> {
    Ok(None)
}

pub fn read_u32(_root: RegRoot, _subkey: &str, _value: &str) -> Result<Option<u32>> {
    Ok(None)
}
