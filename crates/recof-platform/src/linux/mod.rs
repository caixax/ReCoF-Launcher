//! Linux backend.
//!
//! Native Linux and Steam Deck support is out of scope for now. The backend
//! exists so the workspace keeps compiling for a non Windows host and so the
//! shape of the platform boundary is checked by the compiler rather than by
//! good intentions. Functions with an obvious answer are implemented; the
//! rest return [`crate::PlatformError::Unsupported`].

mod disk;
mod display;
mod paths;
mod process;
mod registry;
mod shortcut;

pub use disk::{cluster_size, disk_space, volume_root};
pub use display::{current_display_mode, display_modes};
pub use paths::{desktop_dir, local_app_data, protected_dirs, start_menu_programs_dir};
pub use process::{is_running_by_name, is_steam_running, open_in_shell, spawn};
pub use registry::{read_string, read_u32, RegRoot};
pub use shortcut::{create_shortcut, Shortcut};

pub const PLATFORM_NAME: &str = "linux";
