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
    /// Free text, for anything else.
    Text,
}

/// One option a release offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cvar {
    /// The console variable, including its `rc_` prefix.
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
        match self.kind {
            CvarKind::Bool => matches!(value, "0" | "1"),
            CvarKind::Int => match value.parse::<i64>() {
                Ok(number) => self.in_range(number as f64),
                Err(_) => false,
            },
            CvarKind::Float => match value.parse::<f64>() {
                Ok(number) => number.is_finite() && self.in_range(number),
                Err(_) => false,
            },
            // A value with whitespace would become two arguments.
            CvarKind::Text => !value.chars().any(char::is_whitespace),
        }
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

    /// Turns the saved values into arguments.
    ///
    /// A value equal to the default is left out: the command line should
    /// say what the user changed and nothing more. A value the file does
    /// not describe, left over from an older release, is dropped.
    pub fn to_args(&self, chosen: &std::collections::BTreeMap<String, String>) -> Vec<CvarArg> {
        let mut args = Vec::new();
        for cvar in &self.cvars {
            let Some(value) = chosen.get(&cvar.name) else {
                continue;
            };
            if value == &cvar.default || !cvar.accepts(value) {
                continue;
            }
            args.push(CvarArg {
                name: cvar.name.clone(),
                value: value.clone(),
            });
        }
        args
    }
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
    use std::collections::BTreeMap;

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

    #[test]
    fn a_text_value_with_a_space_is_refused() {
        let cvar = Cvar {
            name: "rc_name".to_owned(),
            kind: CvarKind::Text,
            default: "simon".to_owned(),
            min: None,
            max: None,
            group: String::new(),
            key: None,
            label_en: "Name".to_owned(),
            desc_en: String::new(),
        };
        assert!(cvar.accepts("simon"));
        assert!(!cvar.accepts("two words"));
    }

    #[test]
    fn only_changed_values_reach_the_command_line() {
        let file = file();
        let mut chosen = BTreeMap::new();
        chosen.insert("rc_fov".to_owned(), "90".to_owned());
        assert!(file.to_args(&chosen).is_empty(), "the default is left out");

        chosen.insert("rc_fov".to_owned(), "110".to_owned());
        let args = file.to_args(&chosen);
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "rc_fov");
        assert_eq!(args[0].value, "110");
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
