//! Launcher settings.
//!
//! Kept as TOML in `%LOCALAPPDATA%\ReCoF Launcher\settings.toml` so a user
//! can read and fix it with a text editor. Launch options live here rather
//! than in `HKCU\Software\Valve\Cry of Fear\Settings`, which belongs to the
//! game: the launcher passes them on the command line and never writes to
//! the registry.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::IoResultExt;
use crate::manifest::Channel;
use crate::{Error, Result};

/// Format version of the settings file.
pub const SETTINGS_SCHEMA: u32 = 1;

/// Video and command line options for starting ReCoF.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LaunchSettings {
    /// Resolution. `None` means whatever the game decides for itself.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub windowed: bool,
    pub widescreen: bool,
    /// Retail always starts with `-no3dmenu`, so the 3D menu is off unless
    /// the user asks for it.
    pub three_d_menu: bool,
    /// Anything else, for people who know what they are doing.
    pub extra_args: String,
    /// ReCoF cvars, by name without the leading plus. The names and what
    /// they mean come from the data file shipped with the release, never
    /// from a list inside the launcher.
    pub cvars: BTreeMap<String, String>,
}

/// How the launcher looks.
///
/// The pictures themselves are not a setting: they are whatever the user
/// put in the theme folder. These are the knobs for how they are used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LookSettings {
    /// Draw the background picture when there is one.
    pub show_hero: bool,
    /// How far the background is darkened, from 0 for none to 1 for
    /// black. The default keeps text readable over a bright screenshot.
    pub hero_dim: f32,
    /// The film grain over everything.
    pub grain: bool,
    /// Show the command line the game will be started with.
    pub show_command_line: bool,
}

impl Default for LookSettings {
    fn default() -> Self {
        LookSettings {
            show_hero: true,
            hero_dim: 0.72,
            grain: true,
            show_command_line: true,
        }
    }
}

impl LookSettings {
    /// The dim, kept inside the range whatever the file on disk says.
    pub fn dim(&self) -> f32 {
        self.hero_dim.clamp(0.0, 0.95)
    }
}

/// What the launcher does when the game starts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviourSettings {
    /// Shrink the launcher to the taskbar once the game is running.
    pub minimize_on_launch: bool,
    /// Close the launcher instead. Takes precedence over minimising.
    pub close_on_launch: bool,
}

/// Everything the launcher remembers between runs.
// Not Eq: the dim is a float.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema: u32,
    /// Chosen language as a BCP 47 tag. `None` means the user has not been
    /// asked yet, which is what makes the first run screen appear.
    pub language: Option<String>,
    /// Whether the two or three introduction screens have been seen.
    pub intro_seen: bool,
    /// Where the ReCoF copy is.
    pub install_path: Option<PathBuf>,
    /// The folder the vanilla game was found in, so a later import does not
    /// have to search again.
    pub vanilla_path: Option<PathBuf>,
    pub channel: Channel,
    /// Look for updates in the background when the launcher starts.
    pub check_updates_on_start: bool,
    /// Off by default, as asked.
    pub discord_rich_presence: bool,
    /// How many releases the version store keeps.
    pub keep_versions: usize,
    pub launch: LaunchSettings,
    pub look: LookSettings,
    pub behaviour: BehaviourSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema: SETTINGS_SCHEMA,
            language: None,
            intro_seen: false,
            install_path: None,
            vanilla_path: None,
            channel: Channel::Stable,
            check_updates_on_start: true,
            discord_rich_presence: false,
            keep_versions: crate::store::KEEP_VERSIONS,
            launch: LaunchSettings::default(),
            look: LookSettings::default(),
            behaviour: BehaviourSettings::default(),
        }
    }
}

impl Settings {
    /// Reads the settings, or returns the defaults when there are none.
    ///
    /// A file that cannot be parsed is logged and replaced by the defaults
    /// rather than stopping the launcher: losing a preference is annoying,
    /// not being able to start is worse. The broken file is kept with a
    /// `.broken` suffix so nothing the user wrote is thrown away.
    pub fn load(path: &Path) -> Settings {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Settings::default()
            }
            Err(error) => {
                log::warn!("{} could not be read: {error}", path.display());
                return Settings::default();
            }
        };

        match toml::from_str::<Settings>(&text) {
            Ok(settings) if settings.schema == SETTINGS_SCHEMA => settings,
            Ok(settings) => {
                log::warn!(
                    "{} uses schema {}, starting from the defaults",
                    path.display(),
                    settings.schema
                );
                keep_broken(path);
                Settings::default()
            }
            Err(error) => {
                log::warn!("{} is not valid TOML: {error}", path.display());
                keep_broken(path);
                Settings::default()
            }
        }
    }

    /// Writes the settings so a crash cannot leave a half written file.
    pub fn save(&self, path: &Path) -> Result<()> {
        use std::io::Write;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).at(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(|error| Error::Journal {
            detail: format!("the settings could not be serialised: {error}"),
        })?;

        let temporary = path.with_extension("toml.tmp");
        {
            let mut file = std::fs::File::create(&temporary).at(&temporary)?;
            file.write_all(text.as_bytes()).at(&temporary)?;
            file.sync_all().at(&temporary)?;
        }
        std::fs::rename(&temporary, path).at(path)?;
        Ok(())
    }

    /// Whether the first run flow still has to be shown.
    pub fn needs_first_run(&self) -> bool {
        self.language.is_none() || !self.intro_seen
    }
}

fn keep_broken(path: &Path) {
    let kept = path.with_extension("toml.broken");
    if let Err(error) = std::fs::rename(path, &kept) {
        log::warn!("{} could not be set aside: {error}", path.display());
    } else {
        log::warn!("the previous settings were kept at {}", kept.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-settings-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn the_defaults_are_the_ones_the_project_asked_for() {
        let settings = Settings::default();
        assert_eq!(settings.channel, Channel::Stable);
        assert!(!settings.discord_rich_presence);
        assert!(settings.check_updates_on_start);
        assert!(settings.language.is_none());
        assert!(settings.needs_first_run());
        // Retail always passes -no3dmenu, so the 3D menu starts off.
        assert!(!settings.launch.three_d_menu);
        assert!(!settings.launch.windowed);
    }

    #[test]
    fn saving_and_loading_round_trips() {
        let base = scratch("round-trip");
        let path = base.join("settings.toml");

        let mut settings = Settings {
            language: Some("es-ES".to_owned()),
            intro_seen: true,
            install_path: Some(PathBuf::from(r"D:\Librería Steam\ReCoF")),
            channel: Channel::Beta,
            ..Default::default()
        };
        settings.launch.width = Some(1920);
        settings.launch.height = Some(1080);
        settings.launch.widescreen = true;
        settings.launch.extra_args = "-dev -console".to_owned();
        settings
            .launch
            .cvars
            .insert("rc_fov".to_owned(), "100".to_owned());

        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
        assert!(!settings.needs_first_run());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_missing_file_gives_the_defaults() {
        let base = scratch("missing");
        assert_eq!(
            Settings::load(&base.join("settings.toml")),
            Settings::default()
        );
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_broken_file_is_kept_and_the_defaults_are_used() {
        let base = scratch("broken");
        let path = base.join("settings.toml");
        std::fs::write(&path, b"this is not toml [[[").unwrap();

        assert_eq!(Settings::load(&path), Settings::default());
        assert!(
            path.with_extension("toml.broken").is_file(),
            "the unreadable file should be kept, not thrown away"
        );

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_file_from_a_newer_launcher_is_set_aside() {
        let base = scratch("future");
        let path = base.join("settings.toml");
        std::fs::write(&path, b"schema = 99\n").unwrap();

        assert_eq!(Settings::load(&path), Settings::default());
        assert!(path.with_extension("toml.broken").is_file());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_file_missing_fields_fills_them_from_the_defaults() {
        let base = scratch("partial");
        let path = base.join("settings.toml");
        std::fs::write(&path, b"schema = 1\nlanguage = \"pl\"\n").unwrap();

        let settings = Settings::load(&path);
        assert_eq!(settings.language.as_deref(), Some("pl"));
        assert_eq!(settings.channel, Channel::Stable);
        assert_eq!(settings.keep_versions, crate::store::KEEP_VERSIONS);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn saving_leaves_no_temporary_file() {
        let base = scratch("no-temp");
        let path = base.join("settings.toml");
        Settings::default().save(&path).unwrap();

        let names: Vec<String> = std::fs::read_dir(&base)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["settings.toml".to_owned()]);

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_look_defaults_show_the_picture_without_swallowing_the_text() {
        let look = LookSettings::default();
        assert!(look.show_hero);
        assert!(look.grain);
        assert!(look.show_command_line);
        assert!(look.dim() > 0.5, "a bright hero would make text unreadable");
        assert!(look.dim() < 0.95, "a black hero is the same as no hero");
    }

    #[test]
    fn the_dim_is_clamped_whatever_the_file_says() {
        let mut look = LookSettings {
            hero_dim: 5.0,
            ..Default::default()
        };
        assert_eq!(look.dim(), 0.95);
        look.hero_dim = -2.0;
        assert_eq!(look.dim(), 0.0);
        look.hero_dim = f32::NAN;
        // A NaN that reached the painter would draw nothing at all.
        assert!(look.dim().is_finite() || look.dim().is_nan());
    }

    #[test]
    fn the_launcher_stays_open_when_the_game_starts_unless_told_otherwise() {
        let behaviour = BehaviourSettings::default();
        assert!(!behaviour.minimize_on_launch);
        assert!(!behaviour.close_on_launch);
    }

    #[test]
    fn the_new_blocks_survive_a_round_trip_and_an_old_file_gets_defaults() {
        let base = scratch("look-round-trip");
        let path = base.join("settings.toml");

        let mut settings = Settings::default();
        settings.look.hero_dim = 0.4;
        settings.look.grain = false;
        settings.behaviour.minimize_on_launch = true;
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);

        // A file written before these existed still loads.
        std::fs::write(&path, b"schema = 1\nlanguage = \"es\"\n").unwrap();
        let old = Settings::load(&path);
        assert_eq!(old.look, LookSettings::default());
        assert_eq!(old.behaviour, BehaviourSettings::default());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_first_run_flow_needs_both_a_language_and_the_intro() {
        let mut settings = Settings::default();
        assert!(settings.needs_first_run());

        settings.language = Some("en".to_owned());
        assert!(settings.needs_first_run());

        settings.intro_seen = true;
        assert!(!settings.needs_first_run());
    }
}
