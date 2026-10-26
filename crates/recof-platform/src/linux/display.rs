//! Display modes on Linux.
//!
//! X11 and Wayland both have an answer; neither is in scope yet. An empty
//! list makes the resolution picker fall back to "as the game decides",
//! which is a usable default rather than a failure.

use crate::{DisplayMode, Result};

pub fn display_modes() -> Result<Vec<DisplayMode>> {
    Ok(Vec::new())
}

pub fn current_display_mode() -> Result<Option<DisplayMode>> {
    Ok(None)
}
