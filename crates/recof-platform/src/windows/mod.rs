//! Windows backend.

mod disk;
mod display;
mod paths;
mod process;
mod registry;

pub use disk::{cluster_size, disk_space, volume_root};
pub use display::{current_display_mode, display_modes};
pub use paths::{desktop_dir, local_app_data, protected_dirs, start_menu_programs_dir};
pub use process::{is_running_by_name, is_steam_running, open_in_shell, spawn};
pub use registry::{read_string, read_u32, RegRoot};

/// Name of the platform, for logs and the bug report.
pub const PLATFORM_NAME: &str = "windows";
