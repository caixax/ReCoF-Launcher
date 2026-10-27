//! Constants the whole launcher agrees on.
//!
//! Every value here was checked against the ReCoF repository; the comment
//! names the file it came from.

/// Steam app id of Cry of Fear.
pub const STEAM_APP_ID: u32 = 223_710;

/// Content depot of Cry of Fear (`appmanifest_223710.acf`).
pub const STEAM_DEPOT_ID: u32 = 223_711;

/// The buildid of retail 1.6a, unchanged since at least 2022-03-31.
/// Source: `docs/feasibility/launcher.md`, "Finding the install".
pub const EXPECTED_BUILDID: u64 = 5_821_301;

/// Retail version string the whole project targets.
pub const RETAIL_VERSION: &str = "1.6a";

/// Name of the game's mod directory inside the install.
pub const GAME_DIR: &str = "cryoffear";

/// The engine launcher executable, at the root of the install.
pub const GAME_EXE: &str = "cof.exe";

/// Directory the launcher creates inside the ReCoF copy for its own state.
pub const STATE_DIR: &str = ".recof";

/// Directory name under `%LOCALAPPDATA%` for settings, logs, backups, cache
/// and the version store.
pub const APP_DIR_NAME: &str = "ReCoF Launcher";

/// Product name shown to the user.
pub const APP_DISPLAY_NAME: &str = "ReCoF Launcher";

/// Repository that publishes ReCoF releases.
pub const RECOF_REPO: &str = "caixax/ReCoF";

/// Repository that publishes ReCoF Launcher releases.
pub const LAUNCHER_REPO: &str = "caixax/ReCoF-Launcher";

/// Version of this launcher, from Cargo.
pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Discord application id for Rich Presence.
///
/// Empty until somebody creates the application at
/// <https://discord.com/developers/applications> and gives the launcher
/// its id. While it is empty the presence reports itself as unavailable
/// and the setting says so, rather than silently doing nothing.
///
/// An application id is public: it is handed to every Discord client
/// that sees the presence, and it is not a secret or a token. It can be
/// set here, or passed as `RECOF_DISCORD_APP_ID` at build time so a
/// fork does not have to edit this file. The build script watches that
/// variable, so changing it rebuilds.
pub const DISCORD_APP_ID: &str = match option_env!("RECOF_DISCORD_APP_ID") {
    Some(id) => id,
    None => "",
};

/// Steam URL that starts retail Cry of Fear.
pub const VANILLA_LAUNCH_URL: &str = "steam://rungameid/223710";

/// Base arguments retail `CoFLaunchApp.exe` passes to `cof.exe`.
/// Source: `src/coflaunchapp/Form1.cs`, `button1_Click`.
pub const BASE_LAUNCH_ARGS: &[&str] = &[
    "-basedir",
    "cryoffear",
    "-game",
    "cryoffear",
    "-hw",
    "-heapsize",
    "1536000",
];

/// Files the launcher backs up as "the player's progress and settings".
/// Paths are relative to the root of an install.
/// Source: `docs/feasibility/launcher.md`, "Back up".
pub const SAVE_PATHS: &[&str] = &[
    "cryoffear/SAVE",
    "cryoffear/scriptsettings.dat",
    "cryoffear/alt.dat",
    "cryoffear/config.cfg",
    "cryoffear/autoexec.cfg",
    "cryoffear/user.scr",
];
