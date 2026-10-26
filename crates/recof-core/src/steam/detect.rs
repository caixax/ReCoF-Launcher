//! Locating the Steam libraries and the Cry of Fear install inside them.
//!
//! The chain is the one Steam itself uses:
//!
//! 1. `HKCU\Software\Valve\Steam\SteamPath` gives the Steam folder.
//! 2. `steamapps\libraryfolders.vdf` lists every library.
//! 3. `steamapps\appmanifest_223710.acf` in a library means the game is
//!    installed there, and gives `installdir` and `buildid`.
//!
//! The pure half (parse this text, produce these paths) is separated from
//! the IO half so the tests can run against fixtures instead of a real
//! Steam install.

use std::path::{Path, PathBuf};

use recof_platform::{read_string, RegRoot};

use crate::consts::{EXPECTED_BUILDID, GAME_DIR, GAME_EXE, STEAM_APP_ID};
use crate::error::IoResultExt;
use crate::kv;
use crate::{Error, Result};

const STEAM_KEY: &str = r"Software\Valve\Steam";
const STEAM_KEY_WOW: &str = r"SOFTWARE\WOW6432Node\Valve\Steam";

/// How the install was found, which decides how much the launcher trusts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionSource {
    /// Found through Steam's own files.
    Steam,
    /// Picked by the user with the folder dialog.
    Manual,
}

/// Where Steam is and which libraries it knows about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamLayout {
    /// The Steam client folder, which is also a library.
    pub steam_path: PathBuf,
    /// Every library, the Steam folder first, in the order Steam lists them.
    pub libraries: Vec<PathBuf>,
}

/// A Cry of Fear installation, retail or a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInstall {
    /// The folder that holds `cof.exe`.
    pub root: PathBuf,
    /// The library the install sits in, when it came from Steam.
    pub library: Option<PathBuf>,
    /// `installdir` from the app manifest.
    pub installdir: Option<String>,
    /// `buildid` from the app manifest.
    pub buildid: Option<u64>,
    /// `SizeOnDisk` from the app manifest, which is Steam's own figure and
    /// not a substitute for measuring the tree.
    pub size_on_disk: Option<u64>,
    /// `StateFlags` from the app manifest. 4 means fully installed.
    pub state_flags: Option<u64>,
    pub source: DetectionSource,
}

impl GameInstall {
    /// Whether the build matches the one ReCoF targets.
    pub fn buildid_matches(&self) -> bool {
        self.buildid == Some(EXPECTED_BUILDID)
    }

    /// The mod directory, `<root>/cryoffear`.
    pub fn game_dir(&self) -> PathBuf {
        self.root.join(GAME_DIR)
    }

    /// The engine launcher, `<root>/cof.exe`.
    pub fn game_exe(&self) -> PathBuf {
        self.root.join(GAME_EXE)
    }
}

/// The Steam folder, from the registry.
///
/// `HKCU` first, which is what the running user's client wrote, then the
/// 32 bit `HKLM` key that the installer writes.
pub fn steam_path() -> Result<Option<PathBuf>> {
    if let Some(path) = read_string(RegRoot::CurrentUser, STEAM_KEY, "SteamPath")? {
        if !path.is_empty() {
            return Ok(Some(normalize(&path)));
        }
    }
    for key in [STEAM_KEY_WOW, STEAM_KEY] {
        if let Some(path) = read_string(RegRoot::LocalMachine, key, "InstallPath")? {
            if !path.is_empty() {
                return Ok(Some(normalize(&path)));
            }
        }
    }
    Ok(None)
}

/// Steam writes the path with forward slashes and in lower case. Both are
/// fine for the file APIs, but a path shown to the user should look like a
/// Windows path.
fn normalize(path: &str) -> PathBuf {
    PathBuf::from(path.replace('/', "\\"))
}

/// Every library Steam knows about, the Steam folder first.
pub fn libraries(steam_path: &Path) -> Result<SteamLayout> {
    let vdf = steam_path.join("steamapps").join("libraryfolders.vdf");
    let text = match std::fs::read_to_string(&vdf) {
        Ok(text) => text,
        // No file means a client that has never been started, or a layout
        // the launcher does not know. The Steam folder itself is still a
        // library, so carry on with just that one.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(Error::io(&vdf, error)),
    };

    let mut libraries = vec![steam_path.to_path_buf()];
    for found in libraries_from_vdf(&text) {
        if !libraries.iter().any(|known| same_path(known, &found)) {
            libraries.push(found);
        }
    }
    Ok(SteamLayout {
        steam_path: steam_path.to_path_buf(),
        libraries,
    })
}

/// Pulls the library paths out of a `libraryfolders.vdf`, handling both the
/// current format (one block per library with a `path` key) and the older
/// flat one (a numbered key whose value is the path).
///
/// A file that does not parse yields no libraries rather than an error: the
/// Steam folder is still usable and the user can always pick the folder by
/// hand.
pub fn libraries_from_vdf(text: &str) -> Vec<PathBuf> {
    let Ok(root) = kv::parse(text) else {
        log::warn!("libraryfolders.vdf did not parse, ignoring it");
        return Vec::new();
    };

    // The root key is "libraryfolders" in both formats, but be forgiving.
    let folders = root
        .get_object("libraryfolders")
        .or_else(|| root.iter().find_map(|(_, value)| value.as_object()));
    let Some(folders) = folders else {
        return Vec::new();
    };

    let mut found = Vec::new();
    for (key, value) in folders.iter() {
        // Only numbered entries are libraries. The rest are bookkeeping.
        if key.parse::<u32>().is_err() {
            continue;
        }
        let path = match value {
            kv::Value::String(path) => Some(path.clone()),
            kv::Value::Object(block) => block.get_str("path").map(str::to_owned),
        };
        if let Some(path) = path.filter(|path| !path.trim().is_empty()) {
            found.push(normalize(&path));
        }
    }
    found
}

/// Looks for Cry of Fear in every library and returns the first install
/// found, or `None` when it is installed nowhere.
pub fn find_game() -> Result<Option<GameInstall>> {
    let Some(steam) = steam_path()? else {
        log::info!("Steam is not in the registry");
        return Ok(None);
    };
    let layout = libraries(&steam)?;
    find_game_in(&layout)
}

/// The same search over a layout that is already known.
pub fn find_game_in(layout: &SteamLayout) -> Result<Option<GameInstall>> {
    for library in &layout.libraries {
        match game_in_library(library) {
            Ok(Some(install)) => return Ok(Some(install)),
            Ok(None) => {}
            // One unreadable library must not hide the game in another one.
            Err(error) => log::warn!("library {}: {error}", library.display()),
        }
    }
    Ok(None)
}

/// Reads the app manifest in one library, if it is there.
pub fn game_in_library(library: &Path) -> Result<Option<GameInstall>> {
    let steamapps = library.join("steamapps");
    let manifest = steamapps.join(format!("appmanifest_{STEAM_APP_ID}.acf"));
    let text = match std::fs::read_to_string(&manifest) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io(&manifest, error)),
    };

    let Some(mut install) = install_from_manifest(&text) else {
        log::warn!("{} did not parse as an app manifest", manifest.display());
        return Ok(None);
    };

    let installdir = install.installdir.clone().unwrap_or_default();
    install.root = steamapps.join("common").join(&installdir);
    install.library = Some(library.to_path_buf());

    // A manifest without the files is a download that never finished.
    if !install.game_exe().is_file() {
        log::warn!(
            "{} names {} but there is no {GAME_EXE} there",
            manifest.display(),
            install.root.display()
        );
        return Ok(None);
    }
    Ok(Some(install))
}

/// Parses an `appmanifest_223710.acf`. The returned `root` is empty: only
/// the caller knows which library the file came from.
pub fn install_from_manifest(text: &str) -> Option<GameInstall> {
    let root = kv::parse(text).ok()?;
    let state = root
        .get_object("AppState")
        .or_else(|| root.iter().find_map(|(_, value)| value.as_object()))?;

    Some(GameInstall {
        root: PathBuf::new(),
        library: None,
        installdir: state.get_str("installdir").map(str::to_owned),
        buildid: state.get_u64("buildid"),
        size_on_disk: state.get_u64("SizeOnDisk"),
        state_flags: state.get_u64("StateFlags"),
        source: DetectionSource::Steam,
    })
}

/// Checks a folder the user picked by hand.
///
/// The folder has to hold `cof.exe` and a `cryoffear` directory. No app
/// manifest is read, so there is no buildid: the hash check is what tells
/// the user whether this is retail 1.6a.
pub fn inspect_folder(root: &Path) -> Result<GameInstall> {
    let exe = root.join(GAME_EXE);
    if !exe.is_file() {
        return Err(Error::NoGameExe(root.to_path_buf()));
    }
    if !root.join(GAME_DIR).is_dir() {
        return Err(Error::NoGameDir(root.to_path_buf()));
    }

    // An install picked by hand may still sit inside a Steam library, in
    // which case the manifest two levels up still applies.
    let from_manifest = root
        .parent()
        .and_then(Path::parent)
        .map(|steamapps| steamapps.join(format!("appmanifest_{STEAM_APP_ID}.acf")))
        .filter(|manifest| manifest.is_file())
        .and_then(|manifest| std::fs::read_to_string(manifest).ok())
        .and_then(|text| install_from_manifest(&text));

    let mut install = from_manifest.unwrap_or(GameInstall {
        root: PathBuf::new(),
        library: None,
        installdir: None,
        buildid: None,
        size_on_disk: None,
        state_flags: None,
        source: DetectionSource::Manual,
    });
    install.root = root.to_path_buf();
    install.source = DetectionSource::Manual;
    Ok(install)
}

/// Reads the directory to confirm it exists, so a path on a disconnected
/// drive is reported as missing rather than silently treated as empty.
pub fn directory_exists(path: &Path) -> Result<bool> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).at(path),
    }
}

/// Compares two paths without caring about case or trailing separators,
/// which is how Windows compares them.
fn same_path(left: &Path, right: &Path) -> bool {
    fn key(path: &Path) -> String {
        path.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .to_lowercase()
            .replace('/', "\\")
    }
    key(left) == key(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODERN_VDF: &str = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"apps" { "228980" "62939497" }
	}
	"1"
	{
		"path"		"I:\\SteamLibrary"
		"apps" { "223710" "4695601334" }
	}
	"2"
	{
		"path"		"D:\\Librería Steam"
		"apps" { }
	}
}
"#;

    #[test]
    fn reads_every_library_from_the_modern_format() {
        let found = libraries_from_vdf(MODERN_VDF);
        assert_eq!(
            found,
            vec![
                PathBuf::from(r"C:\Program Files (x86)\Steam"),
                PathBuf::from(r"I:\SteamLibrary"),
                PathBuf::from(r"D:\Librería Steam"),
            ]
        );
    }

    #[test]
    fn reads_every_library_from_the_old_format() {
        let text = r#"
"LibraryFolders"
{
	"TimeNextStatsReport"		"1234567890"
	"ContentStatsID"		"-123"
	"1"		"D:\\SteamLibrary"
	"2"		"E:\\Games\\Steam"
}
"#;
        assert_eq!(
            libraries_from_vdf(text),
            vec![
                PathBuf::from(r"D:\SteamLibrary"),
                PathBuf::from(r"E:\Games\Steam"),
            ]
        );
    }

    #[test]
    fn bookkeeping_keys_are_not_libraries() {
        let text = r#"
"libraryfolders"
{
	"TimeNextStatsReport"	"1"
	"ContentStatsID"	"2"
	"0" { "path" "C:\\Steam" }
}
"#;
        assert_eq!(libraries_from_vdf(text), vec![PathBuf::from(r"C:\Steam")]);
    }

    #[test]
    fn an_empty_path_is_skipped() {
        let text = r#""libraryfolders" { "0" { "path" "" } "1" { "path" "D:\\S" } }"#;
        assert_eq!(libraries_from_vdf(text), vec![PathBuf::from(r"D:\S")]);
    }

    #[test]
    fn a_broken_file_yields_no_libraries_instead_of_failing() {
        assert!(libraries_from_vdf(r#""libraryfolders" { "0" "#).is_empty());
        assert!(libraries_from_vdf("").is_empty());
    }

    #[test]
    fn forward_slashes_become_backslashes() {
        let text = r#""libraryfolders" { "0" { "path" "d:/games/steam" } }"#;
        assert_eq!(
            libraries_from_vdf(text),
            vec![PathBuf::from(r"d:\games\steam")]
        );
    }

    #[test]
    fn parses_a_real_shaped_app_manifest() {
        let text = r#"
"AppState"
{
	"appid"		"223710"
	"name"		"Cry of Fear"
	"StateFlags"		"4"
	"installdir"		"Cry of Fear"
	"SizeOnDisk"		"4695601334"
	"buildid"		"5821301"
}
"#;
        let install = install_from_manifest(text).expect("a well formed manifest");
        assert_eq!(install.installdir.as_deref(), Some("Cry of Fear"));
        assert_eq!(install.buildid, Some(EXPECTED_BUILDID));
        assert_eq!(install.size_on_disk, Some(4_695_601_334));
        assert_eq!(install.state_flags, Some(4));
        assert!(install.buildid_matches());
    }

    #[test]
    fn a_different_buildid_is_reported_not_hidden() {
        let text = r#""AppState" { "installdir" "Cry of Fear" "buildid" "9999999" }"#;
        let install = install_from_manifest(text).unwrap();
        assert_eq!(install.buildid, Some(9_999_999));
        assert!(!install.buildid_matches());
    }

    #[test]
    fn a_manifest_without_a_buildid_is_still_usable() {
        let text = r#""AppState" { "installdir" "Cry of Fear" }"#;
        let install = install_from_manifest(text).unwrap();
        assert_eq!(install.buildid, None);
        assert!(!install.buildid_matches());
    }

    #[test]
    fn path_comparison_ignores_case_and_trailing_separators() {
        assert!(same_path(
            Path::new(r"C:\Program Files (x86)\Steam"),
            Path::new(r"c:\program files (x86)\steam\")
        ));
        assert!(same_path(Path::new("c:/steam"), Path::new(r"C:\Steam")));
        assert!(!same_path(Path::new(r"C:\Steam"), Path::new(r"D:\Steam")));
    }
}
