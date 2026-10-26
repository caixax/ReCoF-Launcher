//! Windows backend.

mod paths;

pub use paths::{desktop_dir, local_app_data, start_menu_programs_dir};

/// Name of the platform, for logs and the bug report.
pub const PLATFORM_NAME: &str = "windows";
