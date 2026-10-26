//! Finding Steam and the Cry of Fear installation.
//!
//! [`detect`] only ever reads: nothing in it, or anywhere reachable from
//! it, writes inside a Steam library.
//!
//! [`shortcuts`] is the one exception in the whole project. Adding ReCoF
//! to Steam as a non Steam game means writing `shortcuts.vdf` under
//! `userdata`, which is inside the Steam folder. It is opt in, it refuses
//! while Steam is running, and it copies the file first. Nothing else
//! here writes anywhere near a Steam library, and nothing at all writes
//! inside the Cry of Fear installation.

pub mod detect;
pub mod shortcuts;

pub use detect::{
    find_game, inspect_folder, libraries, libraries_from_vdf, steam_path, DetectionSource,
    GameInstall, SteamLayout,
};
