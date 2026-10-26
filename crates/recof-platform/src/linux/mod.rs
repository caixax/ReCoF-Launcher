//! Linux backend.
//!
//! Native Linux and Steam Deck support is out of scope for now. The backend
//! exists so the workspace keeps compiling for a non Windows host and so the
//! shape of the platform boundary is checked by the compiler rather than by
//! good intentions. Functions with an obvious XDG answer are implemented;
//! the rest return [`PlatformError::Unsupported`].

mod paths;

pub use paths::{desktop_dir, local_app_data, start_menu_programs_dir};

pub const PLATFORM_NAME: &str = "linux";
