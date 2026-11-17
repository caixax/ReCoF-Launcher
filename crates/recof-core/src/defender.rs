//! Windows Security exclusions for the launcher and the ReCoF copy.
//!
//! ReCoF and the launcher are not signed yet, so Microsoft Defender can
//! take a rebuilt DLL or the launcher itself for something it is not, and
//! block or delete it. The launcher offers to add its own folders to the
//! exclusions, and only those: its program folder (or just its exe when it
//! sits in a folder shared with other things), its data folder and the
//! copy. Never the Steam folder, a library's `steamapps`, the Steam copy of
//! the game, or anything that holds one of them.

use std::path::{Path, PathBuf};

use crate::consts::APP_DIR_NAME;
use crate::paths::{comparable, is_inside};
use crate::settings::DefenderSettings;
use crate::steam::SteamLayout;
use crate::Result;

/// What the folders to exclude are worked out from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    /// The running launcher.
    pub launcher_exe: Option<PathBuf>,
    /// `%LOCALAPPDATA%\ReCoF Launcher`.
    pub app_data: PathBuf,
    /// The ReCoF copy, when there is one.
    pub install_root: Option<PathBuf>,
    /// Folders that must neither be excluded nor sit inside an exclusion:
    /// Steam and its libraries' `steamapps`, and the Steam copy of the game.
    pub keep_out: Vec<PathBuf>,
}

/// The Steam folders an exclusion must stay clear of.
pub fn steam_folders(layout: &SteamLayout) -> Vec<PathBuf> {
    std::iter::once(layout.steam_path.clone())
        .chain(
            layout
                .libraries
                .iter()
                .map(|library| library.join("steamapps")),
        )
        .collect()
}

/// What stands for the launcher in the exclusions.
///
/// Its folder when that is the one the installer made, named after the
/// launcher. Anywhere else, the portable exe may sit in Downloads or on
/// the desktop, so only the exe itself is excluded: it is the whole
/// portable launcher, and the folder around it is none of its business.
pub fn launcher_target(exe: &Path) -> PathBuf {
    match exe.parent() {
        Some(folder)
            if folder
                .file_name()
                .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(APP_DIR_NAME)) =>
        {
            folder.to_path_buf()
        }
        _ => exe.to_path_buf(),
    }
}

/// The folders to exclude, in the order they are shown.
///
/// A folder inside another one on the list is left out, since the outer
/// one covers it. A folder too broad for the platform to accept (a drive,
/// a top level folder not named after ReCoF) or touching Steam is left
/// out too, and logged.
pub fn folders(scope: &Scope) -> Vec<PathBuf> {
    let candidates = scope
        .launcher_exe
        .as_deref()
        .map(launcher_target)
        .into_iter()
        .chain(std::iter::once(scope.app_data.clone()))
        .chain(scope.install_root.clone());

    let mut kept: Vec<PathBuf> = Vec::new();
    for candidate in candidates {
        if !recof_platform::defender_exclusion_allowed(&candidate) {
            log::info!(
                "{} is too broad to exclude, leaving it out",
                candidate.display()
            );
            continue;
        }
        if scope
            .keep_out
            .iter()
            .any(|steam| is_inside(&candidate, steam) || is_inside(steam, &candidate))
        {
            log::info!(
                "{} touches a Steam folder, leaving it out",
                candidate.display()
            );
            continue;
        }
        if kept.iter().any(|folder| is_inside(&candidate, folder)) {
            continue;
        }
        kept.retain(|folder| !is_inside(folder, &candidate));
        kept.push(candidate);
    }
    kept
}

/// The folders not yet excluded, as far as the launcher knows.
pub fn missing(folders: &[PathBuf], remembered: &DefenderSettings) -> Vec<PathBuf> {
    folders
        .iter()
        .filter(|folder| !remembered.excluded.contains(&comparable(folder)))
        .cloned()
        .collect()
}

/// Records folders as excluded, so the offer is not made for them again.
pub fn remember(remembered: &mut DefenderSettings, added: &[PathBuf]) {
    remembered
        .excluded
        .extend(added.iter().map(|path| comparable(path)));
    remembered.excluded.sort();
    remembered.excluded.dedup();
}

/// Whether Defender's real-time protection is on. A machine that cannot
/// say (another antivirus, Defender turned off) answers no.
pub fn realtime_on() -> bool {
    match recof_platform::defender_realtime_on() {
        Ok(on) => on,
        Err(error) => {
            log::info!("Defender's status could not be read, taking it as off: {error}");
            false
        }
    }
}

/// Adds the folders to the exclusions, behind one administrator prompt.
pub fn add(folders: &[PathBuf]) -> Result<()> {
    for folder in folders {
        log::info!(
            "adding {} to the Windows Security exclusions",
            folder.display()
        );
    }
    recof_platform::add_defender_exclusions(folders)?;
    log::info!("Windows Security accepted the exclusions");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(exe: &str, install: Option<&str>) -> Scope {
        Scope {
            launcher_exe: Some(PathBuf::from(exe)),
            app_data: PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
            install_root: install.map(PathBuf::from),
            keep_out: vec![
                PathBuf::from(r"C:\Program Files (x86)\Steam"),
                PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps"),
                PathBuf::from(r"D:\SteamLibrary\steamapps"),
            ],
        }
    }

    #[test]
    fn the_installed_launcher_is_excluded_by_its_folder() {
        let found = folders(&scope(
            r"C:\Users\x\AppData\Local\Programs\ReCoF Launcher\ReCoF-Launcher.exe",
            Some(r"D:\SteamLibrary\ReCoF"),
        ));
        assert_eq!(
            found,
            vec![
                PathBuf::from(r"C:\Users\x\AppData\Local\Programs\ReCoF Launcher"),
                PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
                PathBuf::from(r"D:\SteamLibrary\ReCoF"),
            ]
        );
    }

    #[test]
    fn a_portable_launcher_in_downloads_is_excluded_by_its_exe_only() {
        let found = folders(&scope(r"C:\Users\x\Downloads\ReCoF-Launcher.exe", None));
        assert_eq!(
            found,
            vec![
                PathBuf::from(r"C:\Users\x\Downloads\ReCoF-Launcher.exe"),
                PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
            ]
        );
    }

    #[test]
    fn nothing_inside_steam_or_holding_it_is_ever_excluded() {
        let mut steam_side = scope(
            r"C:\Program Files (x86)\Steam\ReCoF Launcher\ReCoF-Launcher.exe",
            Some(r"D:\SteamLibrary\steamapps\common\Cry of Fear"),
        );
        steam_side.app_data = PathBuf::from(r"C:\Program Files (x86)\Steam\x\y");
        assert!(folders(&steam_side).is_empty());

        // A folder that holds a library's steamapps is refused as well.
        let holding = scope(
            r"D:\SteamLibrary\x\ReCoF-Launcher.exe",
            Some(r"D:\SteamLibrary"),
        );
        let found = folders(&holding);
        assert!(!found.contains(&PathBuf::from(r"D:\SteamLibrary")));
        assert!(found.contains(&PathBuf::from(r"D:\SteamLibrary\x\ReCoF-Launcher.exe")));
    }

    #[test]
    fn a_drive_root_or_top_level_folder_is_never_offered() {
        for broad in [r"E:\", r"E:\Games"] {
            let found = folders(&scope(
                r"C:\Users\x\Desktop\ReCoF-Launcher.exe",
                Some(broad),
            ));
            assert_eq!(
                found,
                vec![
                    PathBuf::from(r"C:\Users\x\Desktop\ReCoF-Launcher.exe"),
                    PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
                ],
                "{broad} should be left out"
            );
        }
    }

    #[test]
    fn a_top_level_folder_named_after_recof_is_offered() {
        let found = folders(&scope(
            r"C:\ReCoF Launcher\ReCoF-Launcher.exe",
            Some(r"H:\ReCoF\"),
        ));
        assert_eq!(
            found,
            vec![
                PathBuf::from(r"C:\ReCoF Launcher"),
                PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
                PathBuf::from(r"H:\ReCoF\"),
            ]
        );
    }

    #[test]
    fn a_folder_inside_another_is_covered_by_it() {
        let found = folders(&scope(
            r"D:\Games\ReCoF\ReCoF-Launcher.exe",
            Some(r"D:\Games\ReCoF"),
        ));
        assert_eq!(
            found,
            vec![
                PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
                PathBuf::from(r"D:\Games\ReCoF"),
            ]
        );
    }

    #[test]
    fn remembered_folders_are_not_offered_again_whatever_their_case() {
        let wanted = folders(&scope(
            r"C:\Users\x\Downloads\ReCoF-Launcher.exe",
            Some(r"D:\Games\ReCoF"),
        ));
        let mut remembered = DefenderSettings::default();
        assert_eq!(missing(&wanted, &remembered), wanted);

        remember(
            &mut remembered,
            &[
                PathBuf::from(r"c:\users\X\appdata\local\recof launcher\"),
                PathBuf::from(r"C:\Users\x\Downloads\ReCoF-Launcher.exe"),
            ],
        );
        assert_eq!(
            missing(&wanted, &remembered),
            vec![PathBuf::from(r"D:\Games\ReCoF")]
        );

        // Remembering the same folder twice keeps one entry.
        remember(&mut remembered, &[PathBuf::from(r"D:\Games\ReCoF")]);
        remember(&mut remembered, &[PathBuf::from(r"d:/games/recof")]);
        assert_eq!(remembered.excluded.len(), 3);
        assert!(missing(&wanted, &remembered).is_empty());
    }

    #[test]
    fn the_steam_folders_are_the_client_and_every_steamapps() {
        let layout = SteamLayout {
            steam_path: PathBuf::from(r"C:\Steam"),
            libraries: vec![
                PathBuf::from(r"C:\Steam"),
                PathBuf::from(r"D:\SteamLibrary"),
            ],
        };
        assert_eq!(
            steam_folders(&layout),
            vec![
                PathBuf::from(r"C:\Steam"),
                PathBuf::from(r"C:\Steam\steamapps"),
                PathBuf::from(r"D:\SteamLibrary\steamapps"),
            ]
        );
    }
}
