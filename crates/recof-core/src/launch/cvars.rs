//! The ReCoF options, as the release describes them.
//!
//! The launcher ships no list of cvars. A release carries a data file,
//! named by the `cvars` entry of its manifest, and the options screen is
//! built from that. A release that adds an option therefore works with a
//! launcher that predates it, which is the whole point.
//!
//! Labels come from the launcher's translations when a key exists for
//! them, and from the English text in the data file when it does not, so a
//! new option is readable rather than blank.
//!
//! What a value can hold is limited by the engine, not by taste. It splits
//! its command line at every `+` and `-` to find the commands, drops any
//! byte outside printable ASCII, and its console parser ends a token at
//! punctuation and a line at `//`. So a value may hold letters, digits,
//! spaces, dots and underscores, and nothing else, which also means no
//! negative numbers.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::launch::args::CvarArg;
use crate::{Error, Result};

/// Format version of the cvar data file.
pub const CVARS_SCHEMA: u32 = 1;

/// What kind of control an option gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CvarKind {
    /// A switch, passed as 1 or 0.
    Bool,
    /// A whole number, with optional bounds.
    Int,
    /// A number with a fractional part.
    Float,
    /// Free text, for anything else. Spaces are allowed: the value is
    /// passed quoted.
    Text,
    /// A launch option rather than a console variable: `-name` when on,
    /// nothing when off. For things like `-condebug` that the engine reads
    /// once at start.
    Flag,
}

/// One option a release offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cvar {
    /// The console variable, including its `rc_` prefix. For a flag, the
    /// option without its leading dash.
    pub name: String,
    #[serde(rename = "type")]
    pub kind: CvarKind,
    /// The value the game uses when the launcher passes nothing.
    pub default: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Groups options under a heading. Free text; unknown groups are shown
    /// in the order they first appear.
    #[serde(default)]
    pub group: String,
    /// Fluent message id for the label and its description, when the
    /// launcher happens to ship one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// English label, used when the launcher has no key for this option.
    pub label_en: String,
    #[serde(default)]
    pub desc_en: String,
}

impl Cvar {
    /// Whether a value is acceptable for this option.
    pub fn accepts(&self, value: &str) -> bool {
        if !value.bytes().all(is_passable) {
            return false;
        }
        match self.kind {
            CvarKind::Bool | CvarKind::Flag => matches!(value, "0" | "1"),
            CvarKind::Int => match value.parse::<i64>() {
                Ok(number) => self.in_range(number as f64),
                Err(_) => false,
            },
            CvarKind::Float => match value.parse::<f64>() {
                Ok(number) => number.is_finite() && self.in_range(number),
                Err(_) => false,
            },
            CvarKind::Text => true,
        }
    }

    /// The value to show for this option.
    ///
    /// What the user chose in the launcher wins. Otherwise a console
    /// variable the game saved in `config.cfg` shows what the game will
    /// really use, and only then the default. A flag is never saved by the
    /// game, so it goes straight to the default.
    pub fn current<'a>(
        &'a self,
        chosen: &'a BTreeMap<String, String>,
        saved: &'a BTreeMap<String, String>,
    ) -> &'a str {
        if let Some(value) = chosen.get(&self.name).filter(|value| self.accepts(value)) {
            return value;
        }
        if self.kind != CvarKind::Flag {
            if let Some(value) = saved
                .get(&self.name.to_lowercase())
                .filter(|value| self.accepts(value))
            {
                return value;
            }
        }
        &self.default
    }

    fn in_range(&self, value: f64) -> bool {
        self.min.map(|min| value >= min).unwrap_or(true)
            && self.max.map(|max| value <= max).unwrap_or(true)
    }
}

/// The whole data file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CvarFile {
    pub schema: u32,
    pub cvars: Vec<Cvar>,
}

impl CvarFile {
    /// Reads and checks a cvar file from a stored release.
    pub fn load(path: &std::path::Path) -> Result<CvarFile> {
        let bytes = std::fs::read(path).map_err(|error| Error::io(path, error))?;
        let file: CvarFile = serde_json::from_slice(&bytes).map_err(|error| Error::Journal {
            detail: format!("{} is not a cvar file: {error}", path.display()),
        })?;

        if file.schema != CVARS_SCHEMA {
            return Err(Error::Journal {
                detail: format!(
                    "{} uses cvar schema {}, this launcher understands {CVARS_SCHEMA}",
                    path.display(),
                    file.schema
                ),
            });
        }
        file.check()?;
        Ok(file)
    }

    /// Rejects a file that would produce a broken command line.
    pub fn check(&self) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for cvar in &self.cvars {
            if !is_valid_cvar_name(&cvar.name) {
                return Err(Error::Journal {
                    detail: format!("{:?} is not a usable cvar name", cvar.name),
                });
            }
            if !seen.insert(cvar.name.to_lowercase()) {
                return Err(Error::Journal {
                    detail: format!("{} is listed twice", cvar.name),
                });
            }
            if !cvar.accepts(&cvar.default) {
                return Err(Error::Journal {
                    detail: format!(
                        "the default {:?} is not valid for {}",
                        cvar.default, cvar.name
                    ),
                });
            }
        }
        Ok(())
    }

    /// The groups in the order they first appear.
    pub fn groups(&self) -> Vec<&str> {
        let mut groups = Vec::new();
        for cvar in &self.cvars {
            if !groups.contains(&cvar.group.as_str()) {
                groups.push(cvar.group.as_str());
            }
        }
        groups
    }

    /// Turns the values the user chose into arguments.
    ///
    /// Every console variable the user set in the launcher is passed, even
    /// at its default. Many are saved by the game in `config.cfg`, so a
    /// value left out would not mean the default: it would mean whatever
    /// the console last set. One the user never touched is left out, and
    /// the game keeps its own. A flag is passed when it is on, whether the
    /// user turned it on or it is on by default. A value the file does not
    /// describe, left over from an older release, is dropped.
    ///
    /// The same list goes to [`write_saved`] before the game starts, for
    /// the variables `config.cfg` would otherwise undo.
    pub fn to_args(&self, chosen: &BTreeMap<String, String>) -> Vec<CvarArg> {
        let mut args = Vec::new();
        for cvar in &self.cvars {
            let value = chosen.get(&cvar.name).filter(|value| cvar.accepts(value));
            match cvar.kind {
                CvarKind::Flag => {
                    if value.unwrap_or(&cvar.default) == "1" {
                        args.push(CvarArg::Flag {
                            name: cvar.name.clone(),
                        });
                    }
                }
                _ => {
                    if let Some(value) = value {
                        args.push(CvarArg::Set {
                            name: cvar.name.clone(),
                            value: value.clone(),
                        });
                    }
                }
            }
        }
        args
    }

    /// Every option at its default, as if the user had chosen each one.
    pub fn defaults(&self) -> BTreeMap<String, String> {
        self.cvars
            .iter()
            .map(|cvar| (cvar.name.clone(), cvar.default.clone()))
            .collect()
    }
}

/// Whether a byte can travel in a value, as the module notes explain.
fn is_passable(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'.' | b'_')
}

/// The console variables the game saved in a copy, by lower case name.
///
/// Read from `cryoffear/config.cfg`, which the game writes on exit. A
/// missing or unreadable file is no values at all rather than an error:
/// it only means the game has not been run yet.
pub fn saved_values(install_root: &Path) -> BTreeMap<String, String> {
    let path = install_root
        .join(crate::consts::GAME_DIR)
        .join("config.cfg");
    match std::fs::read(&path) {
        Ok(bytes) => parse_config(&String::from_utf8_lossy(&bytes)),
        Err(_) => BTreeMap::new(),
    }
}

/// Puts chosen values into the lines of the copy's `config.cfg` that
/// already name them, and returns how many lines changed.
///
/// The engine runs the command line (`stuffcmds`, at the end of
/// `valve.rc`) before it runs `config.cfg`, so a variable the game saved
/// there undoes the `+name value` it was started with. Writing the value
/// into that line makes the launcher's choice the one read last. A
/// variable the file does not name yet is left to the command line, which
/// works for it because nothing runs afterwards to undo it. Every other
/// byte of the file is kept as it was, and the new file replaces the old
/// one with a single rename, so a failure leaves the old one whole.
pub fn write_saved(install_root: &Path, args: &[CvarArg]) -> Result<usize> {
    let path = install_root
        .join(crate::consts::GAME_DIR)
        .join("config.cfg");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(Error::io(&path, error)),
    };

    let (rewritten, changed) = rewrite_config(&bytes, args);
    if changed == 0 {
        return Ok(0);
    }
    let temporary = path.with_extension("cfg.recof-new");
    std::fs::write(&temporary, &rewritten).map_err(|error| Error::io(&temporary, error))?;
    std::fs::rename(&temporary, &path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        Error::io(&path, error)
    })?;
    Ok(changed)
}

/// The rewriting itself, on bytes, since the file may hold a player name
/// or a bind that is not UTF-8.
fn rewrite_config(bytes: &[u8], args: &[CvarArg]) -> (Vec<u8>, usize) {
    let chosen: BTreeMap<String, &str> = args
        .iter()
        .filter_map(|arg| match arg {
            CvarArg::Set { name, value } => Some((name.to_ascii_lowercase(), value.as_str())),
            CvarArg::Flag { .. } => None,
        })
        .collect();

    let mut out = Vec::with_capacity(bytes.len() + 64);
    let mut changed = 0;
    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        let body_len = line
            .iter()
            .rposition(|byte| !matches!(byte, b'\r' | b'\n'))
            .map_or(0, |last| last + 1);
        let (body, ending) = line.split_at(body_len);

        let content = body.trim_ascii_start();
        let name_len = content
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(content.len());
        let name = &content[..name_len];
        let value = std::str::from_utf8(name)
            .ok()
            .and_then(|name| chosen.get(&name.to_ascii_lowercase()));

        match value {
            Some(value) => {
                let mut replaced = name.to_vec();
                replaced.extend_from_slice(format!(" \"{value}\"").as_bytes());
                if replaced != body {
                    changed += 1;
                }
                out.extend_from_slice(&replaced);
                out.extend_from_slice(ending);
            }
            None => out.extend_from_slice(line),
        }
    }
    (out, changed)
}

/// Reads `name value` and `name "value"` lines. Anything else, such as a
/// `bind` with two arguments, still yields its first one, which no option
/// is named after.
fn parse_config(text: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let Some((name, rest)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let rest = rest.trim_start();
        let value = match rest.strip_prefix('"') {
            Some(quoted) => quoted.split('"').next().unwrap_or(""),
            None => rest.split_whitespace().next().unwrap_or(""),
        };
        values.insert(name.to_lowercase(), value.to_owned());
    }
    values
}

/// A cvar name has to be a bare word: anything else would need quoting on
/// the command line and would not be a console variable anyway.
pub fn is_valid_cvar_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && name.as_bytes()[0].is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> CvarFile {
        CvarFile {
            schema: CVARS_SCHEMA,
            cvars: vec![
                Cvar {
                    name: "rc_fov".to_owned(),
                    kind: CvarKind::Int,
                    default: "90".to_owned(),
                    min: Some(70.0),
                    max: Some(120.0),
                    group: "video".to_owned(),
                    key: Some("cvar-rc-fov".to_owned()),
                    label_en: "Field of view".to_owned(),
                    desc_en: "How much you can see at once.".to_owned(),
                },
                Cvar {
                    name: "rc_fix_flashlight".to_owned(),
                    kind: CvarKind::Bool,
                    default: "1".to_owned(),
                    min: None,
                    max: None,
                    group: "fixes".to_owned(),
                    key: None,
                    label_en: "Fix the flashlight".to_owned(),
                    desc_en: String::new(),
                },
            ],
        }
    }

    #[test]
    fn round_trips_through_json() {
        let original = file();
        let json = serde_json::to_string_pretty(&original).unwrap();
        assert_eq!(serde_json::from_str::<CvarFile>(&json).unwrap(), original);
    }

    #[test]
    fn the_json_uses_the_documented_names() {
        let json = serde_json::to_value(file()).unwrap();
        assert_eq!(json["schema"], 1);
        assert_eq!(json["cvars"][0]["name"], "rc_fov");
        assert_eq!(json["cvars"][0]["type"], "int");
        assert_eq!(json["cvars"][0]["default"], "90");
        assert_eq!(json["cvars"][0]["label_en"], "Field of view");
        assert_eq!(json["cvars"][1]["type"], "bool");
    }

    #[test]
    fn a_minimal_entry_parses() {
        let json = r#"{
            "schema": 1,
            "cvars": [
                { "name": "rc_new_thing", "type": "bool", "default": "0",
                  "label_en": "Something new" }
            ]
        }"#;
        let file: CvarFile = serde_json::from_str(json).unwrap();
        assert!(file.check().is_ok());
        assert_eq!(file.cvars[0].key, None);
        assert_eq!(file.cvars[0].group, "");
    }

    #[test]
    fn values_are_checked_against_their_kind_and_bounds() {
        let file = file();
        let fov = &file.cvars[0];
        assert!(fov.accepts("90"));
        assert!(fov.accepts("70"));
        assert!(fov.accepts("120"));
        assert!(!fov.accepts("69"));
        assert!(!fov.accepts("121"));
        assert!(!fov.accepts("ninety"));
        assert!(!fov.accepts("90.5"));

        let switch = &file.cvars[1];
        assert!(switch.accepts("0"));
        assert!(switch.accepts("1"));
        assert!(!switch.accepts("true"));
        assert!(!switch.accepts("2"));
    }

    fn option(name: &str, kind: CvarKind, default: &str) -> Cvar {
        Cvar {
            name: name.to_owned(),
            kind,
            default: default.to_owned(),
            min: None,
            max: None,
            group: String::new(),
            key: None,
            label_en: name.to_owned(),
            desc_en: String::new(),
        }
    }

    fn set(name: &str, value: &str) -> CvarArg {
        CvarArg::Set {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }

    #[test]
    fn a_text_value_may_have_spaces_but_nothing_the_engine_splits_on() {
        let cvar = option("rc_speedometer_pos", CvarKind::Text, ".5 .9");
        for good in ["simon", "two words", ".5 .9", "255 255 255", "a_b", ""] {
            assert!(cvar.accepts(good), "{good:?} should pass");
        }
        // The engine cuts a command at + and -, ends a token at
        // punctuation and a line at //, and drops anything outside ASCII.
        for bad in [
            "a-b",
            "a+b",
            "a;b",
            "\"q\"",
            "x//y",
            "a,b",
            "caf\u{e9}",
            "a\nb",
            "a\tb",
        ] {
            assert!(!cvar.accepts(bad), "{bad:?} should be refused");
        }
    }

    #[test]
    fn a_negative_number_cannot_be_passed() {
        let mut cvar = option("rc_offset", CvarKind::Int, "0");
        cvar.min = Some(-10.0);
        assert!(cvar.accepts("5"));
        assert!(!cvar.accepts("-5"));

        let float = option("rc_scale", CvarKind::Float, "0.5");
        assert!(float.accepts("1.25"));
        assert!(!float.accepts("-1.25"));
        assert!(!float.accepts("1e-3"));
    }

    #[test]
    fn every_chosen_value_is_passed_even_at_its_default() {
        // The game saves most options in config.cfg, so leaving the
        // default out would leave whatever the console set last.
        let file = file();
        assert!(
            file.to_args(&BTreeMap::new()).is_empty(),
            "untouched stays out"
        );

        let mut chosen = BTreeMap::new();
        chosen.insert("rc_fov".to_owned(), "90".to_owned());
        assert_eq!(file.to_args(&chosen), vec![set("rc_fov", "90")]);

        chosen.insert("rc_fov".to_owned(), "110".to_owned());
        chosen.insert("rc_fix_flashlight".to_owned(), "0".to_owned());
        assert_eq!(
            file.to_args(&chosen),
            vec![set("rc_fov", "110"), set("rc_fix_flashlight", "0")]
        );
    }

    #[test]
    fn a_flag_is_passed_when_it_is_on_and_only_then() {
        let file = CvarFile {
            schema: CVARS_SCHEMA,
            cvars: vec![
                option("condebug", CvarKind::Flag, "0"),
                option("rc_on_by_default", CvarKind::Flag, "1"),
            ],
        };
        let flag = |name: &str| CvarArg::Flag {
            name: name.to_owned(),
        };

        assert_eq!(
            file.to_args(&BTreeMap::new()),
            vec![flag("rc_on_by_default")]
        );

        let mut chosen = BTreeMap::new();
        chosen.insert("condebug".to_owned(), "1".to_owned());
        chosen.insert("rc_on_by_default".to_owned(), "0".to_owned());
        assert_eq!(file.to_args(&chosen), vec![flag("condebug")]);

        assert!(!file.cvars[0].accepts("2"));
    }

    #[test]
    fn the_value_shown_is_the_choice_then_the_game_config_then_the_default() {
        let cvar = option("rc_rawinput", CvarKind::Bool, "1");
        let mut chosen = BTreeMap::new();
        let mut saved = BTreeMap::new();
        assert_eq!(cvar.current(&chosen, &saved), "1");

        saved.insert("rc_rawinput".to_owned(), "0".to_owned());
        assert_eq!(cvar.current(&chosen, &saved), "0", "the game's own value");

        chosen.insert("rc_rawinput".to_owned(), "1".to_owned());
        assert_eq!(cvar.current(&chosen, &saved), "1", "the launcher wins");

        // A saved value the option would not accept is ignored.
        let other = option("rc_speedometer", CvarKind::Bool, "0");
        saved.insert("rc_speedometer".to_owned(), "yes".to_owned());
        assert_eq!(other.current(&BTreeMap::new(), &saved), "0");

        // A flag is never in config.cfg, whatever the file says.
        let flag = option("condebug", CvarKind::Flag, "0");
        saved.insert("condebug".to_owned(), "1".to_owned());
        assert_eq!(flag.current(&BTreeMap::new(), &saved), "0");
    }

    #[test]
    fn the_game_config_is_read_the_way_the_game_writes_it() {
        let text =
            "// This file is overwritten whenever you change your user settings in the game.\n\
                    bind \"w\" \"+forward\"\n\
                    rc_rawinput \"0\"\n\
                    RC_Speedometer_Pos \".5 .9\"\n\
                    sensitivity 2.5\n\
                    \n\
                    unbindall\n";
        let values = parse_config(text);
        assert_eq!(values.get("rc_rawinput").map(String::as_str), Some("0"));
        assert_eq!(
            values.get("rc_speedometer_pos").map(String::as_str),
            Some(".5 .9")
        );
        assert_eq!(values.get("sensitivity").map(String::as_str), Some("2.5"));
        assert!(!values.contains_key("unbindall"));

        let missing = std::env::temp_dir().join("recof-no-such-copy");
        assert!(saved_values(&missing).is_empty());
    }

    #[test]
    fn a_chosen_value_replaces_the_line_the_game_saved() {
        let config = b"// written by the game\r\n\
                       bind \"w\" \"+forward\"\r\n\
                       RC_RawInput \"0\"\r\n\
                       rc_speedometer_jump \"1\"\r\n\
                       name \"Sim\xf3n\"\r\n\
                       rc_speedometer \"0\"";
        let args = [
            set("rc_rawinput", "1"),
            set("rc_speedometer", "1"),
            set("rc_hlmovement", "1"),
            CvarArg::Flag {
                name: "condebug".to_owned(),
            },
        ];
        let (rewritten, changed) = rewrite_config(config, &args);
        assert_eq!(changed, 2);
        assert_eq!(
            rewritten,
            b"// written by the game\r\n\
              bind \"w\" \"+forward\"\r\n\
              RC_RawInput \"1\"\r\n\
              rc_speedometer_jump \"1\"\r\n\
              name \"Sim\xf3n\"\r\n\
              rc_speedometer \"1\""
                .to_vec(),
            "only the named lines change; a longer name sharing the prefix, a \
             non UTF-8 byte and the line ends are kept"
        );

        let (_, unchanged) = rewrite_config(&rewritten, &args);
        assert_eq!(unchanged, 0, "a value already there is not a change");
    }

    #[test]
    fn writing_the_saved_values_touches_only_a_config_that_needs_it() {
        let copy = std::env::temp_dir().join("recof-cvars-write-saved");
        let _ = std::fs::remove_dir_all(&copy);
        let config = copy.join(crate::consts::GAME_DIR).join("config.cfg");

        // No config yet: the command line does the work.
        assert_eq!(write_saved(&copy, &[set("rc_rawinput", "0")]).unwrap(), 0);

        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(&config, "rc_rawinput \"1\"\n").unwrap();
        assert_eq!(write_saved(&copy, &[set("rc_rawinput", "0")]).unwrap(), 1);
        assert_eq!(
            std::fs::read_to_string(&config).unwrap(),
            "rc_rawinput \"0\"\n"
        );
        assert!(!config.with_extension("cfg.recof-new").exists());

        std::fs::remove_dir_all(&copy).unwrap();
    }

    #[test]
    fn defaults_cover_every_option() {
        let defaults = file().defaults();
        assert_eq!(defaults.get("rc_fov").map(String::as_str), Some("90"));
        assert_eq!(
            defaults.get("rc_fix_flashlight").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn a_value_from_a_release_that_no_longer_has_it_is_dropped() {
        let file = file();
        let mut chosen = BTreeMap::new();
        chosen.insert("rc_removed_option".to_owned(), "1".to_owned());
        assert!(file.to_args(&chosen).is_empty());
    }

    #[test]
    fn an_out_of_range_value_is_dropped_rather_than_passed() {
        let file = file();
        let mut chosen = BTreeMap::new();
        chosen.insert("rc_fov".to_owned(), "999".to_owned());
        assert!(file.to_args(&chosen).is_empty());
    }

    #[test]
    fn a_hostile_cvar_name_is_refused() {
        for name in [
            "",
            "rc fov",
            "-rc_fov",
            "+rc_fov",
            "rc_fov;format c:",
            "1rc",
            "rc\"fov",
            &"a".repeat(65),
        ] {
            assert!(!is_valid_cvar_name(name), "{name:?} should be refused");
        }
        assert!(is_valid_cvar_name("rc_fov"));
        assert!(is_valid_cvar_name("rc_fix_x2"));
    }

    #[test]
    fn a_file_with_a_bad_name_or_a_duplicate_is_refused() {
        let mut broken = file();
        broken.cvars[0].name = "rc fov".to_owned();
        assert!(broken.check().is_err());

        let mut duplicate = file();
        duplicate.cvars[1].name = "RC_FOV".to_owned();
        assert!(duplicate.check().is_err());

        let mut bad_default = file();
        bad_default.cvars[0].default = "999".to_owned();
        assert!(bad_default.check().is_err());
    }

    #[test]
    fn groups_keep_the_order_they_appear_in() {
        assert_eq!(file().groups(), vec!["video", "fixes"]);
    }

    #[test]
    fn loading_from_disk_checks_the_schema() {
        let dir = std::env::temp_dir().join("recof-cvars-tests");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let good = dir.join("recof-cvars.json");
        std::fs::write(&good, serde_json::to_vec_pretty(&file()).unwrap()).unwrap();
        assert_eq!(CvarFile::load(&good).unwrap(), file());

        let mut future = file();
        future.schema = CVARS_SCHEMA + 1;
        let bad = dir.join("future.json");
        std::fs::write(&bad, serde_json::to_vec_pretty(&future).unwrap()).unwrap();
        assert!(CvarFile::load(&bad).is_err());

        let garbage = dir.join("garbage.json");
        std::fs::write(&garbage, b"not json").unwrap();
        assert!(CvarFile::load(&garbage).is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
