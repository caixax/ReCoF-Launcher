//! Starting the game.
//!
//! ReCoF is started by running the copy's own `cof.exe` directly, with the
//! working directory at the root of the copy. Retail Cry of Fear is left
//! to Steam.

pub mod args;
pub mod cvars;

use std::path::Path;

pub use args::{build, preview, CvarArg, MAX_COMMAND_LINE};
pub use cvars::{saved_values, Cvar, CvarFile, CvarKind};

use crate::consts::{GAME_EXE, VANILLA_LAUNCH_URL};
use crate::settings::LaunchSettings;
use crate::{Error, Result};

/// Name of the game process, for noticing that it is already running.
pub const GAME_PROCESS: &str = "cof.exe";

/// Why the game cannot be started, or a warning about starting it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchProblem {
    /// Steam is not running. The game works through Steam while it runs,
    /// so this is a warning rather than a refusal.
    SteamNotRunning,
    /// The game is already open.
    AlreadyRunning,
    /// There is no cof.exe where there should be.
    MissingExe,
}

impl LaunchProblem {
    pub fn message_id(&self) -> &'static str {
        match self {
            LaunchProblem::SteamNotRunning => "launch-steam-not-running",
            LaunchProblem::AlreadyRunning => "launch-already-running",
            LaunchProblem::MissingExe => "err-no-game-exe",
        }
    }

    /// Whether the user may go ahead anyway.
    pub fn is_only_a_warning(&self) -> bool {
        matches!(self, LaunchProblem::SteamNotRunning)
    }
}

/// Everything worth saying before the play button is pressed.
pub fn check(install_root: &Path) -> Vec<LaunchProblem> {
    let mut problems = Vec::new();

    if !install_root.join(GAME_EXE).is_file() {
        problems.push(LaunchProblem::MissingExe);
    }
    // A platform that cannot answer is not a reason to refuse.
    if matches!(recof_platform::is_steam_running(), Ok(false)) {
        problems.push(LaunchProblem::SteamNotRunning);
    }
    if matches!(recof_platform::is_running_by_name(GAME_PROCESS), Ok(true)) {
        problems.push(LaunchProblem::AlreadyRunning);
    }
    problems
}

/// Starts ReCoF from a copy.
///
/// The working directory is the root of the copy, which is what the engine
/// expects: `-basedir cryoffear` is relative to it.
pub fn play_recof(
    install_root: &Path,
    settings: &LaunchSettings,
    cvars: &[CvarArg],
) -> Result<u32> {
    let exe = install_root.join(GAME_EXE);
    if !exe.is_file() {
        return Err(Error::NoGameExe(install_root.to_path_buf()));
    }

    let arguments = build(settings, cvars);
    let line = preview(&exe.display().to_string(), &arguments);
    if line.len() > MAX_COMMAND_LINE {
        return Err(Error::CommandLineTooLong { length: line.len() });
    }
    // The engine reads config.cfg after the command line, so an option the
    // game saved there goes in there too. Failing to write it is not a
    // reason to refuse to play.
    match cvars::write_saved(install_root, cvars) {
        Ok(0) => {}
        Ok(count) => log::info!("{count} options in config.cfg set to the launcher's values"),
        Err(error) => log::warn!("config.cfg was not updated, saved options may win: {error}"),
    }
    log::info!("starting {line}");
    Ok(recof_platform::spawn(&exe, &arguments, install_root)?)
}

/// Starts retail Cry of Fear through Steam.
pub fn play_vanilla() -> Result<()> {
    log::info!("opening {VANILLA_LAUNCH_URL}");
    Ok(recof_platform::open_in_shell(VANILLA_LAUNCH_URL)?)
}

/// Opens the Steam client.
pub fn open_steam() -> Result<()> {
    Ok(recof_platform::open_in_shell("steam://open/main")?)
}

/// The resolutions to offer, from the display rather than from a list.
pub fn available_resolutions() -> Vec<recof_platform::DisplayMode> {
    recof_platform::display_modes().unwrap_or_else(|error| {
        log::warn!("the display modes could not be read: {error}");
        Vec::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_without_the_game_is_refused() {
        let empty = std::env::temp_dir().join("recof-launch-empty");
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).unwrap();

        let problems = check(&empty);
        assert!(problems.contains(&LaunchProblem::MissingExe));
        assert!(!LaunchProblem::MissingExe.is_only_a_warning());

        let outcome = play_recof(&empty, &LaunchSettings::default(), &[]);
        assert!(matches!(outcome, Err(Error::NoGameExe(_))));

        std::fs::remove_dir_all(&empty).unwrap();
    }

    #[test]
    fn a_command_line_the_engine_would_overrun_on_is_refused() {
        let copy = std::env::temp_dir().join("recof-launch-long");
        let _ = std::fs::remove_dir_all(&copy);
        std::fs::create_dir_all(&copy).unwrap();
        std::fs::write(copy.join(GAME_EXE), b"not really the game").unwrap();

        let settings = LaunchSettings {
            extra_args: "-x ".repeat(400),
            ..LaunchSettings::default()
        };
        let outcome = play_recof(&copy, &settings, &[]);
        assert!(
            matches!(outcome, Err(Error::CommandLineTooLong { length }) if length > MAX_COMMAND_LINE),
            "{outcome:?}"
        );

        std::fs::remove_dir_all(&copy).unwrap();
    }

    #[test]
    fn steam_not_running_is_a_warning_and_not_a_refusal() {
        assert!(LaunchProblem::SteamNotRunning.is_only_a_warning());
        assert!(!LaunchProblem::AlreadyRunning.is_only_a_warning());
    }

    #[test]
    fn every_problem_has_a_message() {
        for problem in [
            LaunchProblem::SteamNotRunning,
            LaunchProblem::AlreadyRunning,
            LaunchProblem::MissingExe,
        ] {
            assert!(
                problem.message_id().starts_with("launch-")
                    || problem.message_id().starts_with("err-")
            );
        }
    }

    #[test]
    fn asking_the_display_for_modes_never_fails() {
        // On a machine with no display this is empty, which the options
        // screen handles as "whatever the game decides".
        let _ = available_resolutions();
    }
}
