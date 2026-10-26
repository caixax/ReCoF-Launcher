//! Everything ReCoF Launcher does that is not drawing pixels.
//!
//! This crate holds no UI and reaches the operating system only through
//! [`recof_platform`], so the logic can be tested on any host and a later
//! Linux port stays a matter of filling in that one crate.

pub mod consts;
pub mod error;
pub mod hashes;
pub mod kv;
pub mod space;
pub mod steam;

pub use error::{Error, Result};
