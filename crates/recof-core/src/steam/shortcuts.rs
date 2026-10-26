//! Adding ReCoF to Steam as a non Steam game.
//!
//! This is the one documented exception to the rule that the launcher
//! writes only in the ReCoF copy and in `%LOCALAPPDATA%`, so it is opt
//! in, it refuses to run while Steam is open, and it copies the original
//! `shortcuts.vdf` before touching it.
//!
//! Steam rewrites this file when it exits, so an edit made while Steam is
//! running is thrown away. Refusing is the only way to keep the promise
//! the button makes.

use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::kv::binary::{self, BinMap, BinValue};
use crate::{Error, Result};

/// Why the shortcut cannot be added.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShortcutProblem {
    #[error("Steam is running")]
    SteamRunning,
    #[error("no Steam user profile was found")]
    NoUserProfile,
    #[error("the shortcuts file could not be read: {0}")]
    Unreadable(String),
}

impl ShortcutProblem {
    pub fn message_id(&self) -> &'static str {
        match self {
            ShortcutProblem::SteamRunning => "shortcut-steam-needs-steam-closed",
            ShortcutProblem::NoUserProfile => "err-registry",
            ShortcutProblem::Unreadable(_) => "err-io",
        }
    }
}

/// A Steam user whose shortcuts file could be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamUser {
    /// The numeric folder name under `userdata`.
    pub id: String,
    /// `<steam>/userdata/<id>/config/shortcuts.vdf`.
    pub shortcuts: PathBuf,
}

/// Every Steam user profile that has a config folder.
///
/// A machine with one account has one; a shared machine has several, and
/// the user picks.
pub fn users(steam_path: &Path) -> Result<Vec<SteamUser>> {
    let userdata = steam_path.join("userdata");
    let entries = match std::fs::read_dir(&userdata) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::io(&userdata, error)),
    };

    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.at(&userdata)?;
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        // Steam uses the account id as the folder name, and "0" for the
        // anonymous profile, which is not a real user.
        if id == "0" || id.parse::<u64>().is_err() {
            continue;
        }
        let config = entry.path().join("config");
        if !config.is_dir() {
            continue;
        }
        found.push(SteamUser {
            id,
            shortcuts: config.join("shortcuts.vdf"),
        });
    }

    found.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(found)
}

/// What the launcher adds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutEntry {
    pub app_name: String,
    /// Full path to the executable. Steam expects it quoted.
    pub exe: PathBuf,
    pub start_dir: PathBuf,
    pub launch_options: String,
    pub icon: String,
}

impl ShortcutEntry {
    /// Builds the entry Steam stores.
    pub fn to_map(&self, app_id: i32) -> BinMap {
        let mut map = BinMap::new();
        map.push("appid", BinValue::Int(app_id));
        map.push("AppName", BinValue::String(self.app_name.clone()));
        map.push(
            "Exe",
            BinValue::String(quote(&self.exe.display().to_string())),
        );
        map.push(
            "StartDir",
            BinValue::String(quote(&self.start_dir.display().to_string())),
        );
        map.push("icon", BinValue::String(self.icon.clone()));
        map.push("ShortcutPath", BinValue::String(String::new()));
        map.push(
            "LaunchOptions",
            BinValue::String(self.launch_options.clone()),
        );
        map.push("IsHidden", BinValue::Int(0));
        map.push("AllowDesktopConfig", BinValue::Int(1));
        map.push("AllowOverlay", BinValue::Int(1));
        map.push("OpenVR", BinValue::Int(0));
        map.push("Devkit", BinValue::Int(0));
        map.push("DevkitGameID", BinValue::String(String::new()));
        map.push("DevkitOverrideAppID", BinValue::Int(0));
        map.push("LastPlayTime", BinValue::Int(0));
        map.push("tags", BinValue::Map(BinMap::new()));
        map
    }
}

fn quote(path: &str) -> String {
    format!("\"{path}\"")
}

/// The app id Steam gives a non Steam shortcut.
///
/// Steam derives it from the executable path and the name with a CRC32,
/// which is what keeps grid images attached to the right entry. The same
/// inputs therefore always produce the same id, so adding the shortcut
/// twice updates it rather than making a second one.
pub fn app_id(exe: &Path, app_name: &str) -> i32 {
    let input = format!("{}{}", quote(&exe.display().to_string()), app_name);
    let crc = crc32(input.as_bytes());
    // Steam sets the top bit and treats the value as signed.
    ((crc | 0x8000_0000) as i32).to_owned()
}

/// CRC-32, the IEEE polynomial, which is the one Steam uses here.
fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (index, slot) in table.iter_mut().enumerate() {
        let mut value = index as u32;
        for _ in 0..8 {
            value = if value & 1 != 0 {
                0xedb8_8320 ^ (value >> 1)
            } else {
                value >> 1
            };
        }
        *slot = value;
    }

    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc = table[((crc ^ u32::from(*byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    crc ^ 0xffff_ffff
}

/// Adds or updates a shortcut in a user's `shortcuts.vdf`.
///
/// A copy of the original is written next to it first, with the given
/// stamp in its name, so the change can always be undone by hand.
pub fn add(
    user: &SteamUser,
    entry: &ShortcutEntry,
    stamp: &str,
) -> std::result::Result<PathBuf, ShortcutProblem> {
    if matches!(recof_platform::is_steam_running(), Ok(true)) {
        return Err(ShortcutProblem::SteamRunning);
    }

    let bytes = match std::fs::read(&user.shortcuts) {
        Ok(bytes) => bytes,
        // A user who has never added a non Steam game has no file yet.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(ShortcutProblem::Unreadable(error.to_string())),
    };

    let mut root = if bytes.is_empty() {
        let mut root = BinMap::new();
        root.push("shortcuts", BinValue::Map(BinMap::new()));
        root
    } else {
        binary::parse(&bytes).map_err(|error| ShortcutProblem::Unreadable(error.to_string()))?
    };

    // The backup is written before anything is changed, and only when
    // there was a file to back up.
    let backup = user
        .shortcuts
        .with_file_name(format!("shortcuts.vdf.recof-{stamp}.bak"));
    if !bytes.is_empty() {
        std::fs::write(&backup, &bytes)
            .map_err(|error| ShortcutProblem::Unreadable(error.to_string()))?;
    }

    let id = app_id(&entry.exe, &entry.app_name);
    let updated = with_shortcut(&root, entry, id);
    root = updated;

    let out = binary::write(&root);
    // Written beside the real file and renamed, so a crash cannot leave
    // Steam with a half written shortcuts file.
    let temporary = user.shortcuts.with_extension("vdf.recof-tmp");
    std::fs::write(&temporary, &out)
        .map_err(|error| ShortcutProblem::Unreadable(error.to_string()))?;
    std::fs::rename(&temporary, &user.shortcuts)
        .map_err(|error| ShortcutProblem::Unreadable(error.to_string()))?;

    log::info!(
        "added {} to {} as app id {id}",
        entry.app_name,
        user.shortcuts.display()
    );
    Ok(backup)
}

/// Returns the document with the shortcut added, or replaced when one
/// with the same app id is already there.
fn with_shortcut(root: &BinMap, entry: &ShortcutEntry, id: i32) -> BinMap {
    let existing = root.get_map("shortcuts").cloned().unwrap_or_default();

    let mut shortcuts = BinMap::new();
    let mut replaced = false;
    for (key, value) in existing.iter() {
        let is_ours = value
            .as_map()
            .and_then(|map| map.get_int("appid"))
            .map(|found| found == id)
            .unwrap_or(false);
        if is_ours {
            shortcuts.push(key, BinValue::Map(entry.to_map(id)));
            replaced = true;
        } else {
            shortcuts.push(key, value.clone());
        }
    }
    if !replaced {
        let index = existing.next_index();
        shortcuts.push(index, BinValue::Map(entry.to_map(id)));
    }

    let mut out = BinMap::new();
    out.push("shortcuts", BinValue::Map(shortcuts));
    // Anything else the file held, which there normally is not, is kept.
    for (key, value) in root.iter() {
        if !key.eq_ignore_ascii_case("shortcuts") {
            out.push(key, value.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-shortcut-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn entry() -> ShortcutEntry {
        ShortcutEntry {
            app_name: "ReCoF".to_owned(),
            exe: PathBuf::from(r"D:\Librería Steam\ReCoF\cof.exe"),
            start_dir: PathBuf::from(r"D:\Librería Steam\ReCoF"),
            launch_options: "-basedir cryoffear -game cryoffear".to_owned(),
            icon: String::new(),
        }
    }

    #[test]
    fn the_crc_matches_the_known_check_value() {
        // The standard CRC-32 check: "123456789" hashes to 0xCBF43926.
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn the_app_id_is_stable_for_the_same_inputs() {
        let first = app_id(&entry().exe, &entry().app_name);
        let second = app_id(&entry().exe, &entry().app_name);
        assert_eq!(first, second, "the same shortcut must keep its id");
        assert!(first < 0, "Steam sets the top bit, so the id is negative");

        let other = app_id(Path::new(r"D:\Otra\cof.exe"), "ReCoF");
        assert_ne!(first, other);
    }

    #[test]
    fn the_entry_has_the_fields_steam_expects_and_quotes_the_paths() {
        let map = entry().to_map(-5);
        assert_eq!(map.get_int("appid"), Some(-5));
        assert_eq!(map.get_str("AppName"), Some("ReCoF"));
        assert_eq!(
            map.get_str("Exe"),
            Some("\"D:\\Librería Steam\\ReCoF\\cof.exe\"")
        );
        assert_eq!(
            map.get_str("StartDir"),
            Some("\"D:\\Librería Steam\\ReCoF\"")
        );
        assert_eq!(map.get_int("AllowOverlay"), Some(1));
        assert!(map.get_map("tags").is_some());
    }

    #[test]
    fn adding_to_an_empty_document_creates_the_first_entry() {
        let root = BinMap::new();
        let id = app_id(&entry().exe, &entry().app_name);
        let updated = with_shortcut(&root, &entry(), id);

        let shortcuts = updated.get_map("shortcuts").unwrap();
        assert_eq!(shortcuts.len(), 1);
        assert_eq!(shortcuts.get_map("0").unwrap().get_int("appid"), Some(id));
    }

    #[test]
    fn adding_keeps_the_shortcuts_that_were_already_there() {
        let mut other = BinMap::new();
        other.push("appid", BinValue::Int(-42));
        other.push("AppName", BinValue::String("Something else".to_owned()));

        let mut shortcuts = BinMap::new();
        shortcuts.push("0", BinValue::Map(other));
        let mut root = BinMap::new();
        root.push("shortcuts", BinValue::Map(shortcuts));

        let id = app_id(&entry().exe, &entry().app_name);
        let updated = with_shortcut(&root, &entry(), id);
        let shortcuts = updated.get_map("shortcuts").unwrap();

        assert_eq!(shortcuts.len(), 2);
        assert_eq!(shortcuts.get_map("0").unwrap().get_int("appid"), Some(-42));
        assert_eq!(shortcuts.get_map("1").unwrap().get_int("appid"), Some(id));
    }

    #[test]
    fn adding_the_same_shortcut_twice_updates_it_rather_than_duplicating() {
        let id = app_id(&entry().exe, &entry().app_name);
        let once = with_shortcut(&BinMap::new(), &entry(), id);
        let twice = with_shortcut(&once, &entry(), id);

        assert_eq!(twice.get_map("shortcuts").unwrap().len(), 1);

        // And an entry whose options changed is replaced in place.
        let mut changed = entry();
        changed.launch_options = "-dev".to_owned();
        let updated = with_shortcut(&once, &changed, id);
        assert_eq!(
            updated
                .get_map("shortcuts")
                .unwrap()
                .get_map("0")
                .unwrap()
                .get_str("LaunchOptions"),
            Some("-dev")
        );
    }

    #[test]
    fn a_user_profile_is_found_and_the_anonymous_one_is_skipped() {
        let base = scratch("users");
        let steam = base.join("Steam");
        for id in ["0", "123456789", "987654321", "not-a-number"] {
            std::fs::create_dir_all(steam.join("userdata").join(id).join("config")).unwrap();
        }
        // A profile with no config folder is not usable.
        std::fs::create_dir_all(steam.join("userdata").join("555")).unwrap();

        let found = users(&steam).unwrap();
        let ids: Vec<&str> = found.iter().map(|user| user.id.as_str()).collect();
        assert_eq!(ids, ["123456789", "987654321"]);
        assert!(found[0].shortcuts.ends_with("config/shortcuts.vdf"));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_steam_folder_with_no_userdata_gives_no_users() {
        let base = scratch("no-userdata");
        assert!(users(&base).unwrap().is_empty());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn writing_keeps_a_copy_of_the_original_and_the_file_still_parses() {
        let base = scratch("write");
        let config = base.join("Steam/userdata/123456789/config");
        std::fs::create_dir_all(&config).unwrap();

        // An existing file with somebody else's shortcut in it.
        let mut existing = BinMap::new();
        let mut theirs = BinMap::new();
        theirs.push("appid", BinValue::Int(-42));
        theirs.push("AppName", BinValue::String("Theirs".to_owned()));
        let mut shortcuts = BinMap::new();
        shortcuts.push("0", BinValue::Map(theirs));
        existing.push("shortcuts", BinValue::Map(shortcuts));

        let path = config.join("shortcuts.vdf");
        let original = binary::write(&existing);
        std::fs::write(&path, &original).unwrap();

        let user = SteamUser {
            id: "123456789".to_owned(),
            shortcuts: path.clone(),
        };

        // Steam is almost certainly not running under the test runner,
        // but if it is, the right behaviour is to refuse and that is
        // what the other test covers.
        match add(&user, &entry(), "20261026-120000") {
            Ok(backup) => {
                assert_eq!(std::fs::read(&backup).unwrap(), original);
                let written = binary::parse(&std::fs::read(&path).unwrap()).unwrap();
                let shortcuts = written.get_map("shortcuts").unwrap();
                assert_eq!(shortcuts.len(), 2);
                assert_eq!(
                    shortcuts.get_map("0").unwrap().get_str("AppName"),
                    Some("Theirs")
                );
                assert_eq!(
                    shortcuts.get_map("1").unwrap().get_str("AppName"),
                    Some("ReCoF")
                );
                // No temporary file left behind.
                assert!(!config.join("shortcuts.vdf.recof-tmp").exists());
            }
            Err(ShortcutProblem::SteamRunning) => {
                // Refused, and nothing was written.
                assert_eq!(std::fs::read(&path).unwrap(), original);
            }
            Err(other) => panic!("unexpected failure: {other}"),
        }

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn every_problem_has_a_message() {
        for problem in [
            ShortcutProblem::SteamRunning,
            ShortcutProblem::NoUserProfile,
            ShortcutProblem::Unreadable("x".to_owned()),
        ] {
            assert!(!problem.message_id().is_empty());
            assert!(!problem.to_string().is_empty());
        }
    }
}
