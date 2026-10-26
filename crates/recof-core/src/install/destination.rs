//! Choosing and checking where the copy goes.
//!
//! The rules are short and none of them bends: never inside the Steam
//! folder, never inside a `steamapps` directory, never in Program Files or
//! the Windows folder, never on top of the vanilla install, and never on
//! top of somebody else's files.

use std::path::{Path, PathBuf};

use crate::install::state::InstalledState;
use crate::paths::is_inside;
use crate::steam::SteamLayout;
use crate::{Error, Result};

/// Folder name used for the suggested destination.
pub const DEFAULT_FOLDER_NAME: &str = "ReCoF";

/// What is already at a destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationState {
    /// Nothing is there yet.
    Missing,
    /// The folder exists and holds nothing.
    Empty,
    /// A ReCoF copy this launcher made.
    ExistingInstall(Box<InstalledState>),
    /// A folder with files in it that are not a ReCoF install.
    NotEmpty { entries: usize },
}

/// Why a destination cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DestinationProblem {
    #[error("{0} is inside the Steam folder")]
    InsideSteam(PathBuf),

    #[error("{0} is inside a Steam library's steamapps folder")]
    InsideSteamApps(PathBuf),

    #[error("{0} is inside a protected system folder")]
    Protected(PathBuf),

    #[error("{0} is the vanilla installation")]
    IsVanilla(PathBuf),

    #[error("{0} is the root of a drive")]
    DriveRoot(PathBuf),

    #[error("{path} already holds {entries} files that are not a ReCoF install")]
    NotEmpty { path: PathBuf, entries: usize },

    #[error("{0} is not a usable path")]
    Unusable(PathBuf),
}

impl DestinationProblem {
    /// Fluent message id for the sentence the user sees.
    pub fn message_id(&self) -> &'static str {
        match self {
            DestinationProblem::InsideSteam(_) => "dest-inside-steam",
            DestinationProblem::InsideSteamApps(_) => "dest-inside-steamapps",
            DestinationProblem::Protected(_) => "dest-protected",
            DestinationProblem::IsVanilla(_) => "dest-is-vanilla",
            DestinationProblem::DriveRoot(_) => "dest-drive-root",
            DestinationProblem::NotEmpty { .. } => "dest-not-empty",
            DestinationProblem::Unusable(_) => "dest-unusable",
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            DestinationProblem::InsideSteam(path)
            | DestinationProblem::InsideSteamApps(path)
            | DestinationProblem::Protected(path)
            | DestinationProblem::IsVanilla(path)
            | DestinationProblem::DriveRoot(path)
            | DestinationProblem::NotEmpty { path, .. }
            | DestinationProblem::Unusable(path) => path,
        }
    }
}

/// Looks at what is already at a destination.
pub fn inspect(destination: &Path) -> Result<DestinationState> {
    let entries = match std::fs::read_dir(destination) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DestinationState::Missing)
        }
        Err(error) => return Err(Error::io(destination, error)),
    };

    let count = entries.count();
    if count == 0 {
        return Ok(DestinationState::Empty);
    }
    if let Some(state) = InstalledState::load(destination)? {
        return Ok(DestinationState::ExistingInstall(Box::new(state)));
    }
    Ok(DestinationState::NotEmpty { entries: count })
}

/// Checks a destination against every rule.
///
/// `vanilla` is the install the copy would be made from, which must never
/// be the destination.
pub fn validate(
    destination: &Path,
    layout: Option<&SteamLayout>,
    vanilla: Option<&Path>,
) -> std::result::Result<(), DestinationProblem> {
    if destination.as_os_str().is_empty() || !destination.is_absolute() {
        return Err(DestinationProblem::Unusable(destination.to_path_buf()));
    }
    // A drive root would make uninstalling mean deleting the whole drive.
    if destination.parent().is_none() {
        return Err(DestinationProblem::DriveRoot(destination.to_path_buf()));
    }
    if let Some(parent) = destination.parent() {
        if parent.as_os_str().is_empty() {
            return Err(DestinationProblem::DriveRoot(destination.to_path_buf()));
        }
    }

    for protected in recof_platform::protected_dirs() {
        if is_inside(destination, &protected) {
            return Err(DestinationProblem::Protected(destination.to_path_buf()));
        }
    }

    if let Some(layout) = layout {
        if is_inside(destination, &layout.steam_path) {
            return Err(DestinationProblem::InsideSteam(destination.to_path_buf()));
        }
        for library in &layout.libraries {
            if is_inside(destination, &library.join("steamapps")) {
                return Err(DestinationProblem::InsideSteamApps(
                    destination.to_path_buf(),
                ));
            }
        }
    }

    if let Some(vanilla) = vanilla {
        if is_inside(destination, vanilla) || is_inside(vanilla, destination) {
            return Err(DestinationProblem::IsVanilla(destination.to_path_buf()));
        }
    }

    match inspect(destination) {
        Ok(DestinationState::NotEmpty { entries }) => Err(DestinationProblem::NotEmpty {
            path: destination.to_path_buf(),
            entries,
        }),
        Ok(_) => Ok(()),
        // A folder that cannot even be read is not somewhere to install.
        Err(_) => Err(DestinationProblem::Unusable(destination.to_path_buf())),
    }
}

/// Suggests where to install.
///
/// First choice is a `ReCoF` folder beside a Steam library's `steamapps`,
/// which keeps the copy on the same disk as the game it was made from.
/// Libraries that are the Steam folder itself, or that sit in Program
/// Files, are skipped, and if none is usable the user profile is the
/// fallback.
pub fn suggest(
    layout: Option<&SteamLayout>,
    vanilla: Option<&Path>,
    home: &Path,
) -> Option<PathBuf> {
    if let Some(layout) = layout {
        // The library holding the game first: the copy then lands on the
        // same disk, which is usually where the room is.
        let mut candidates: Vec<&PathBuf> = Vec::new();
        if let Some(vanilla) = vanilla {
            for library in &layout.libraries {
                if is_inside(vanilla, library) {
                    candidates.push(library);
                }
            }
        }
        for library in &layout.libraries {
            if !candidates.contains(&library) {
                candidates.push(library);
            }
        }

        for library in candidates {
            let candidate = library.join(DEFAULT_FOLDER_NAME);
            if validate(&candidate, Some(layout), vanilla).is_ok() {
                return Some(candidate);
            }
        }
    }

    let fallback = home.join("Games").join(DEFAULT_FOLDER_NAME);
    if validate(&fallback, layout, vanilla).is_ok() {
        return Some(fallback);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join("recof-destination-tests")
            .join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn layout(steam: &Path, extra: &[&Path]) -> SteamLayout {
        let mut libraries = vec![steam.to_path_buf()];
        libraries.extend(extra.iter().map(|path| path.to_path_buf()));
        SteamLayout {
            steam_path: steam.to_path_buf(),
            libraries,
        }
    }

    #[test]
    fn a_folder_inside_the_steam_client_is_refused() {
        let steam = Path::new(r"C:\Steam");
        let problem = validate(
            Path::new(r"C:\Steam\ReCoF"),
            Some(&layout(steam, &[])),
            None,
        )
        .unwrap_err();
        assert!(matches!(problem, DestinationProblem::InsideSteam(_)));
        assert_eq!(problem.message_id(), "dest-inside-steam");
    }

    #[test]
    fn a_folder_inside_steamapps_is_refused() {
        let steam = Path::new(r"C:\Steam");
        let library = Path::new(r"I:\SteamLibrary");
        let problem = validate(
            Path::new(r"I:\SteamLibrary\steamapps\common\ReCoF"),
            Some(&layout(steam, &[library])),
            None,
        )
        .unwrap_err();
        assert!(matches!(problem, DestinationProblem::InsideSteamApps(_)));
    }

    #[test]
    fn a_folder_beside_steamapps_in_a_library_is_fine() {
        let steam = Path::new(r"C:\Steam");
        let library = Path::new(r"I:\SteamLibrary");
        assert!(validate(
            Path::new(r"I:\SteamLibrary\ReCoF"),
            Some(&layout(steam, &[library])),
            None,
        )
        .is_ok());
    }

    #[test]
    fn program_files_and_the_windows_folder_are_refused() {
        for protected in recof_platform::protected_dirs() {
            let candidate = protected.join("ReCoF");
            let problem = validate(&candidate, None, None).unwrap_err();
            assert!(
                matches!(problem, DestinationProblem::Protected(_)),
                "{} gave {problem:?}",
                candidate.display()
            );
        }
    }

    #[test]
    fn the_vanilla_install_itself_is_refused_both_ways() {
        let vanilla = Path::new(r"I:\SteamLibrary\steamapps\common\Cry of Fear");
        assert!(matches!(
            validate(vanilla, None, Some(vanilla)).unwrap_err(),
            DestinationProblem::IsVanilla(_)
        ));
        assert!(matches!(
            validate(
                Path::new(r"I:\SteamLibrary\steamapps\common\Cry of Fear\copy"),
                None,
                Some(vanilla),
            )
            .unwrap_err(),
            DestinationProblem::IsVanilla(_)
        ));
        // A parent of the vanilla install would swallow it too.
        assert!(matches!(
            validate(Path::new(r"I:\SteamLibrary"), None, Some(vanilla)).unwrap_err(),
            DestinationProblem::IsVanilla(_)
        ));
    }

    #[test]
    fn a_drive_root_is_refused() {
        assert!(matches!(
            validate(Path::new(r"D:\"), None, None).unwrap_err(),
            DestinationProblem::DriveRoot(_)
        ));
    }

    #[test]
    fn a_relative_path_is_refused() {
        assert!(matches!(
            validate(Path::new(r"ReCoF"), None, None).unwrap_err(),
            DestinationProblem::Unusable(_)
        ));
        assert!(matches!(
            validate(Path::new(""), None, None).unwrap_err(),
            DestinationProblem::Unusable(_)
        ));
    }

    #[test]
    fn a_folder_with_other_files_in_it_is_refused() {
        let base = scratch("not-empty");
        let destination = base.join("Mis cosas");
        std::fs::create_dir_all(&destination).unwrap();
        std::fs::write(destination.join("importante.txt"), b"no me borres").unwrap();

        let problem = validate(&destination, None, None).unwrap_err();
        assert!(matches!(problem, DestinationProblem::NotEmpty { .. }));
        assert_eq!(problem.message_id(), "dest-not-empty");

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_missing_or_empty_folder_is_fine() {
        let base = scratch("empty-ok");
        assert!(validate(&base.join("not there yet"), None, None).is_ok());

        let empty = base.join("vacía");
        std::fs::create_dir_all(&empty).unwrap();
        assert!(validate(&empty, None, None).is_ok());

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn inspecting_tells_the_three_cases_apart() {
        let base = scratch("inspect");
        assert_eq!(
            inspect(&base.join("missing")).unwrap(),
            DestinationState::Missing
        );

        let empty = base.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        assert_eq!(inspect(&empty).unwrap(), DestinationState::Empty);

        let other = base.join("other");
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("a.txt"), b"x").unwrap();
        assert!(matches!(
            inspect(&other).unwrap(),
            DestinationState::NotEmpty { entries: 1 }
        ));

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn the_suggestion_prefers_the_library_holding_the_game() {
        let steam = PathBuf::from(r"C:\Steam");
        let library = PathBuf::from(r"I:\SteamLibrary");
        let other = PathBuf::from(r"E:\Games");
        let vanilla = library.join(r"steamapps\common\Cry of Fear");

        let layout = SteamLayout {
            steam_path: steam.clone(),
            libraries: vec![steam, other, library.clone()],
        };
        let suggested = suggest(Some(&layout), Some(&vanilla), Path::new(r"C:\Users\x"));
        assert_eq!(suggested, Some(library.join("ReCoF")));
    }

    #[test]
    fn the_suggestion_skips_a_library_that_is_the_steam_folder() {
        // The common case on a single drive machine: the only library is
        // the Steam folder itself, which is also inside Program Files.
        let steam = recof_platform::protected_dirs()
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"))
            .join("Steam");
        let layout = SteamLayout {
            steam_path: steam.clone(),
            libraries: vec![steam.clone()],
        };
        let vanilla = steam.join(r"steamapps\common\Cry of Fear");
        let home = PathBuf::from(r"C:\Users\x");

        let suggested = suggest(Some(&layout), Some(&vanilla), &home);
        assert_eq!(suggested, Some(home.join("Games").join("ReCoF")));
    }

    #[test]
    fn with_no_steam_at_all_the_user_profile_is_suggested() {
        let home = PathBuf::from(r"D:\Users\x");
        assert_eq!(
            suggest(None, None, &home),
            Some(home.join("Games").join("ReCoF"))
        );
    }
}
