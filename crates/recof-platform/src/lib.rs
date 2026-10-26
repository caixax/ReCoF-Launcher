//! The platform boundary.
//!
//! Every registry read, display mode query, free space check, shortcut and
//! process start in ReCoF Launcher goes through this crate. Nothing else in
//! the workspace may use `windows` or `#[cfg(windows)]` for behaviour, which
//! is what keeps a later Linux port from turning into a rewrite.
//!
//! Each backend exposes the same functions. The Linux backend compiles and
//! returns [`PlatformError::Unsupported`] where there is nothing to do yet.

mod error;
mod types;

pub use error::PlatformError;
pub use types::{DiskSpace, DisplayMode};

pub type Result<T> = std::result::Result<T, PlatformError>;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod backend;

#[cfg(not(windows))]
#[path = "linux/mod.rs"]
mod backend;

pub use backend::*;
