//! Windows backend.

mod disk;
mod paths;
mod registry;

pub use disk::{cluster_size, disk_space, volume_root};
pub use paths::{desktop_dir, local_app_data, protected_dirs, start_menu_programs_dir};
pub use registry::{read_string, read_u32, RegRoot};

/// Name of the platform, for logs and the bug report.
pub const PLATFORM_NAME: &str = "windows";
