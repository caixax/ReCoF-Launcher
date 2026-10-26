//! Linux backend.
//!
//! Native Linux and Steam Deck support is out of scope for now. The backend
//! exists so the workspace keeps compiling for a non Windows host and so the
//! shape of the platform boundary is checked by the compiler rather than by
//! good intentions. Functions with an obvious answer are implemented; the
//! rest return [`crate::PlatformError::Unsupported`].

mod disk;
mod paths;
mod registry;

pub use disk::{cluster_size, disk_space, volume_root};
pub use paths::{desktop_dir, local_app_data, start_menu_programs_dir};
pub use registry::{read_string, read_u32, RegRoot};

pub const PLATFORM_NAME: &str = "linux";
