//! Building the command line `cof.exe` is started with.
//!
//! The base arguments are the ones retail `CoFLaunchApp.exe` passes, taken
//! from `src/coflaunchapp/Form1.cs` in the ReCoF repository:
//!
//! ```text
//! -basedir cryoffear -game cryoffear -hw -heapsize 1536000 -no3dmenu
//! ```
//!
//! then the window and resolution flags, then `-widescreen` for a mode
//! wider than four by three.
//!
//! Two differences from retail, both deliberate:
//!
//! - Retail always passes `-no3dmenu` and passes it a second time when its
//!   checkbox is ticked. Here it is a single flag, present unless the user
//!   turns the 3D menu on, which is the same behaviour with one argument
//!   instead of two.
//! - Retail writes the resolution into
//!   `HKCU\Software\Valve\Cry of Fear\Settings`. This launcher never
//!   writes to the registry; the values are passed on the command line and
//!   kept in the launcher's own settings.

use crate::consts::BASE_LAUNCH_ARGS;
use crate::settings::LaunchSettings;

/// A cvar the release told the launcher about, with the value to pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CvarArg {
    pub name: String,
    pub value: String,
}

/// Builds the full argument list.
///
/// `cvars` are passed as `+name value`, which is how the engine takes a
/// console variable on the command line.
pub fn build(settings: &LaunchSettings, cvars: &[CvarArg]) -> Vec<String> {
    let mut args: Vec<String> = BASE_LAUNCH_ARGS
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();

    if !settings.three_d_menu {
        args.push("-no3dmenu".to_owned());
    }

    if settings.windowed {
        args.push("-windowed".to_owned());
    } else {
        args.push("-fullscreen".to_owned());
    }

    if let (Some(width), Some(height)) = (settings.width, settings.height) {
        if settings.widescreen {
            args.push("-widescreen".to_owned());
        }
        args.push("-w".to_owned());
        args.push(width.to_string());
        args.push("-h".to_owned());
        args.push(height.to_string());
    } else if settings.widescreen {
        args.push("-widescreen".to_owned());
    }

    for cvar in cvars {
        args.push(format!("+{}", cvar.name));
        args.push(cvar.value.clone());
    }

    args.extend(split_extra(&settings.extra_args));
    args
}

/// Splits the free text argument box the way a shell would, so a path in
/// quotes stays one argument.
pub fn split_extra(text: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_content = false;

    for character in text.chars() {
        match character {
            '"' => {
                in_quotes = !in_quotes;
                has_content = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has_content {
                    args.push(std::mem::take(&mut current));
                    has_content = false;
                }
            }
            c => {
                current.push(c);
                has_content = true;
            }
        }
    }
    if has_content {
        args.push(current);
    }
    args
}

/// The command line as a single string, for showing the user what will
/// run. Not used to start anything: the process is given the list.
pub fn preview(exe: &str, args: &[String]) -> String {
    let mut line = quote_if_needed(exe);
    for arg in args {
        line.push(' ');
        line.push_str(&quote_if_needed(arg));
    }
    line
}

fn quote_if_needed(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_owned();
    }
    if arg.chars().any(char::is_whitespace) || arg.contains('"') {
        format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
        arg.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> LaunchSettings {
        LaunchSettings::default()
    }

    #[test]
    fn the_base_arguments_match_the_retail_launcher() {
        let args = build(&settings(), &[]);
        let joined = args.join(" ");
        assert!(
            joined.starts_with("-basedir cryoffear -game cryoffear -hw -heapsize 1536000"),
            "{joined}"
        );
    }

    #[test]
    fn the_3d_menu_is_off_by_default_as_retail_has_it() {
        let args = build(&settings(), &[]);
        assert_eq!(args.iter().filter(|arg| *arg == "-no3dmenu").count(), 1);

        let mut with_menu = settings();
        with_menu.three_d_menu = true;
        let args = build(&with_menu, &[]);
        assert!(!args.contains(&"-no3dmenu".to_owned()));
    }

    #[test]
    fn window_mode_is_always_stated_one_way_or_the_other() {
        let args = build(&settings(), &[]);
        assert!(args.contains(&"-fullscreen".to_owned()));
        assert!(!args.contains(&"-windowed".to_owned()));

        let mut windowed = settings();
        windowed.windowed = true;
        let args = build(&windowed, &[]);
        assert!(args.contains(&"-windowed".to_owned()));
        assert!(!args.contains(&"-fullscreen".to_owned()));
    }

    #[test]
    fn a_resolution_becomes_w_and_h() {
        let mut chosen = settings();
        chosen.width = Some(1920);
        chosen.height = Some(1080);
        chosen.widescreen = true;

        let args = build(&chosen, &[]);
        let joined = args.join(" ");
        assert!(joined.contains("-widescreen -w 1920 -h 1080"), "{joined}");
    }

    #[test]
    fn no_resolution_means_the_game_decides() {
        let args = build(&settings(), &[]);
        assert!(!args.contains(&"-w".to_owned()));
        assert!(!args.contains(&"-h".to_owned()));
    }

    #[test]
    fn widescreen_can_be_asked_for_without_a_resolution() {
        let mut wide = settings();
        wide.widescreen = true;
        assert!(build(&wide, &[]).contains(&"-widescreen".to_owned()));
    }

    #[test]
    fn cvars_are_passed_with_a_leading_plus_and_a_separate_value() {
        let cvars = [
            CvarArg {
                name: "rc_fov".to_owned(),
                value: "100".to_owned(),
            },
            CvarArg {
                name: "rc_fix_flashlight".to_owned(),
                value: "1".to_owned(),
            },
        ];
        let args = build(&settings(), &cvars);
        let joined = args.join(" ");
        assert!(joined.contains("+rc_fov 100"), "{joined}");
        assert!(joined.contains("+rc_fix_flashlight 1"), "{joined}");
    }

    #[test]
    fn extra_arguments_come_last_so_they_can_override() {
        let mut with_extra = settings();
        with_extra.extra_args = "-dev -console".to_owned();
        let args = build(&with_extra, &[]);
        assert_eq!(&args[args.len() - 2..], ["-dev", "-console"]);
    }

    #[test]
    fn quoted_extra_arguments_stay_in_one_piece() {
        assert_eq!(
            split_extra(r#"-game "mi mod" -dev"#),
            vec!["-game", "mi mod", "-dev"]
        );
        assert_eq!(split_extra("   "), Vec::<String>::new());
        assert_eq!(split_extra(""), Vec::<String>::new());
        assert_eq!(split_extra("  -a   -b  "), vec!["-a", "-b"]);
        // An empty quoted argument is still an argument.
        assert_eq!(split_extra(r#"-a "" -b"#), vec!["-a", "", "-b"]);
    }

    #[test]
    fn the_preview_quotes_what_a_shell_would() {
        let args = vec![
            "-basedir".to_owned(),
            "cryoffear".to_owned(),
            "mi carpeta".to_owned(),
        ];
        assert_eq!(
            preview(r"D:\ReCoF\cof.exe", &args),
            r#"D:\ReCoF\cof.exe -basedir cryoffear "mi carpeta""#
        );
        assert_eq!(
            preview(r"D:\Mis Juegos\cof.exe", &[]),
            r#""D:\Mis Juegos\cof.exe""#
        );
    }

    #[test]
    fn nothing_in_the_arguments_is_ever_the_registry() {
        // The launcher passes values rather than writing them, so no
        // argument may name a registry path.
        let mut everything = settings();
        everything.width = Some(1280);
        everything.height = Some(720);
        everything.widescreen = true;
        everything.windowed = true;
        everything.three_d_menu = true;
        everything.extra_args = "-dev".to_owned();

        for arg in build(&everything, &[]) {
            assert!(!arg.contains("HKCU"), "{arg}");
            assert!(!arg.contains("Software\\Valve"), "{arg}");
        }
    }
}
