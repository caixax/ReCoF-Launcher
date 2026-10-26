//! Where the launcher keeps its own things.
//!
//! Outside the ReCoF copy, the launcher writes in exactly one place:
//! `%LOCALAPPDATA%\ReCoF Launcher`. Every path below hangs off that, and
//! nothing in the crate builds one by hand.

use std::path::{Path, PathBuf};

use crate::consts::{APP_DIR_NAME, STATE_DIR};
use crate::error::IoResultExt;
use crate::Result;

/// The launcher's own folder and everything under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    /// The real location, `%LOCALAPPDATA%\ReCoF Launcher`.
    pub fn discover() -> Result<Self> {
        let root = recof_platform::local_app_data()?.join(APP_DIR_NAME);
        Ok(AppPaths { root })
    }

    /// An arbitrary root, for tests.
    pub fn at(root: impl Into<PathBuf>) -> Self {
        AppPaths { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Settings, as TOML so a user can read and fix it by hand.
    pub fn settings_file(&self) -> PathBuf {
        self.root.join("settings.toml")
    }

    /// Rotating logs.
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Save backups, one zip each.
    pub fn backups_dir(&self) -> PathBuf {
        self.root.join("backups")
    }

    /// Extracted releases, kept so a rollback is possible.
    pub fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    /// Downloads in progress and cached GitHub replies.
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// Where a download is written before it is verified.
    pub fn downloads_dir(&self) -> PathBuf {
        self.cache_dir().join("downloads")
    }

    /// Creates every directory the launcher writes into.
    pub fn create_all(&self) -> Result<()> {
        for directory in [
            self.root.clone(),
            self.logs_dir(),
            self.backups_dir(),
            self.versions_dir(),
            self.cache_dir(),
            self.downloads_dir(),
        ] {
            std::fs::create_dir_all(&directory).at(&directory)?;
        }
        Ok(())
    }
}

/// `<install>/.recof/installed.json`, the record of what is in a copy.
pub fn installed_state_file(install_root: &Path) -> PathBuf {
    install_root.join(STATE_DIR).join("installed.json")
}

/// Whether a path is inside, or is, another one.
///
/// Compared without case and with separators unified, which is how Windows
/// compares them. Used to refuse installing into the Steam folder and to
/// make sure nothing writes outside where it should.
pub fn is_inside(path: &Path, ancestor: &Path) -> bool {
    let path = normalize_for_comparison(path);
    let ancestor = normalize_for_comparison(ancestor);
    if path == ancestor {
        return true;
    }
    path.starts_with(&format!("{ancestor}\\"))
}

fn normalize_for_comparison(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_hangs_off_one_folder() {
        let paths = AppPaths::at(r"C:\Users\x\AppData\Local\ReCoF Launcher");
        for path in [
            paths.settings_file(),
            paths.logs_dir(),
            paths.backups_dir(),
            paths.versions_dir(),
            paths.cache_dir(),
            paths.downloads_dir(),
        ] {
            assert!(
                is_inside(&path, paths.root()),
                "{} escapes the app folder",
                path.display()
            );
        }
    }

    #[test]
    fn the_real_folder_is_under_local_app_data() {
        let paths = AppPaths::discover().expect("LOCALAPPDATA is always there on Windows");
        assert!(paths.root().ends_with(APP_DIR_NAME));
    }

    #[test]
    fn create_all_makes_every_directory() {
        let root = std::env::temp_dir().join("recof-paths-test ñ");
        let _ = std::fs::remove_dir_all(&root);
        let paths = AppPaths::at(&root);
        paths.create_all().unwrap();

        assert!(paths.logs_dir().is_dir());
        assert!(paths.backups_dir().is_dir());
        assert!(paths.versions_dir().is_dir());
        assert!(paths.downloads_dir().is_dir());
        // Running it twice is fine.
        paths.create_all().unwrap();

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn containment_ignores_case_and_separator_style() {
        let steam = Path::new(r"C:\Program Files (x86)\Steam");
        assert!(is_inside(
            Path::new(r"c:\program files (x86)\steam\steamapps\common\ReCoF"),
            steam
        ));
        assert!(is_inside(
            Path::new("C:/Program Files (x86)/Steam/x"),
            steam
        ));
        assert!(is_inside(steam, steam));
        assert!(is_inside(
            Path::new(r"C:\Program Files (x86)\Steam\"),
            steam
        ));
    }

    #[test]
    fn a_sibling_with_a_shared_prefix_is_not_inside() {
        let steam = Path::new(r"C:\Steam");
        assert!(!is_inside(Path::new(r"C:\SteamLibrary"), steam));
        assert!(!is_inside(Path::new(r"C:\Steam2\game"), steam));
        assert!(!is_inside(Path::new(r"D:\Steam\game"), steam));
    }

    #[test]
    fn the_installed_state_lives_in_the_copy() {
        let install = Path::new(r"D:\Games\ReCoF");
        let state = installed_state_file(install);
        assert_eq!(state, install.join(".recof").join("installed.json"));
        assert!(is_inside(&state, install));
    }
}
