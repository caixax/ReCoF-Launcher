//! Finding Steam and the Cry of Fear installation.
//!
//! Everything here reads. Nothing in this module, or anywhere reachable from
//! it, writes inside a Steam library.

pub mod detect;

pub use detect::{
    find_game, inspect_folder, libraries, libraries_from_vdf, steam_path, DetectionSource,
    GameInstall, SteamLayout,
};
