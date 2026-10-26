//! Replacing the launcher with a newer one.
//!
//! The new executable goes through exactly the same checks as a game
//! release: a signed manifest, the archive hash, and the hash of the file
//! that comes out. Only then is it swapped in, with `self-replace`, which
//! renames the running executable aside and moves the new one into its
//! place, the one trick Windows allows on a file that is in use.
//!
//! The launcher restarts itself afterwards. Nothing else is touched: a
//! launcher update never writes inside the game copy.

use std::path::{Path, PathBuf};

use crate::manifest::{safe_relative_path, Product};
use crate::store::StoredVersion;
use crate::version::Version;
use crate::{Error, Result};

/// Name of the launcher executable inside its release archive.
pub const LAUNCHER_EXE: &str = "ReCoF-Launcher.exe";

/// What a launcher release says about itself, once it is in the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingLauncher {
    pub version: Version,
    /// The verified executable, sitting in the version store.
    pub executable: PathBuf,
}

/// Finds the verified executable inside a stored launcher release.
pub fn pending(stored: &StoredVersion) -> Result<PendingLauncher> {
    if stored.product != Product::RecofLauncher {
        return Err(Error::Journal {
            detail: format!(
                "{} is not a launcher release",
                stored.manifest.product.as_str()
            ),
        });
    }

    // The manifest decides which file is the launcher, as it decides
    // everything else. Falling back to the conventional name keeps a
    // release that lists it differently working.
    let entry = stored
        .manifest
        .file(LAUNCHER_EXE)
        .or_else(|| {
            stored
                .manifest
                .files
                .iter()
                .find(|file| file.path.to_lowercase().ends_with(".exe"))
        })
        .ok_or_else(|| Error::MissingAsset(LAUNCHER_EXE.to_owned()))?;

    let relative =
        safe_relative_path(&entry.path).ok_or_else(|| Error::MissingAsset(entry.path.clone()))?;
    let executable = stored.files_dir().join(relative);
    if !executable.is_file() {
        return Err(Error::MissingAsset(entry.path.clone()));
    }

    Ok(PendingLauncher {
        version: stored.manifest.version.clone(),
        executable,
    })
}

/// Whether an installed ReCoF demands a newer launcher than this one.
///
/// When it does, the launcher has to update itself before it may touch
/// the game copy, which is why this is checked before anything else.
pub fn launcher_too_old(required: &Version, running: &Version) -> bool {
    required > running
}

/// Swaps the running executable for the new one.
///
/// After this returns, the file on disk is the new launcher and the
/// process still running is the old one, so the caller restarts and
/// exits.
pub fn apply(new_executable: &Path) -> Result<()> {
    if !new_executable.is_file() {
        return Err(Error::MissingAsset(new_executable.display().to_string()));
    }
    log::info!(
        "replacing this executable with {}",
        new_executable.display()
    );
    self_replace::self_replace(new_executable).map_err(|error| Error::io(new_executable, error))?;
    Ok(())
}

/// Starts the launcher again and asks the caller to exit.
pub fn restart() -> Result<()> {
    let exe =
        std::env::current_exe().map_err(|error| Error::io("the running executable", error))?;
    let working_dir = exe.parent().unwrap_or(Path::new(".")).to_path_buf();
    recof_platform::spawn(&exe, &[], &working_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashes::sha256_bytes;
    use crate::manifest::schema::{Archive, Channel, Manifest, ManifestFile, SCHEMA_VERSION};

    fn stored(product: Product, files: Vec<ManifestFile>, dir: PathBuf) -> StoredVersion {
        StoredVersion {
            product,
            version: Version::parse("0.2.0").unwrap(),
            dir: dir.clone(),
            manifest: Manifest {
                schema: SCHEMA_VERSION,
                product,
                version: Version::parse("0.2.0").unwrap(),
                channel: Channel::Stable,
                released: "2026-10-26T12:00:00Z".to_owned(),
                target_buildid: None,
                min_launcher: Version::parse("0.1.0").unwrap(),
                notes_url: None,
                archive: Archive {
                    name: "recof-launcher-0.2.0.zip".to_owned(),
                    size: 1,
                    sha256: sha256_bytes(b"x"),
                },
                files,
                data: Default::default(),
            },
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join("recof-selfupdate-tests")
            .join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn exe_entry() -> ManifestFile {
        ManifestFile {
            path: LAUNCHER_EXE.to_owned(),
            size: 4,
            sha256: sha256_bytes(b"exe!"),
            copy_of: None,
        }
    }

    #[test]
    fn the_executable_is_found_inside_a_stored_launcher_release() {
        let dir = scratch("found");
        std::fs::create_dir_all(dir.join("files")).unwrap();
        std::fs::write(dir.join("files").join(LAUNCHER_EXE), b"exe!").unwrap();

        let release = stored(Product::RecofLauncher, vec![exe_entry()], dir.clone());
        let found = pending(&release).unwrap();
        assert_eq!(found.version, Version::parse("0.2.0").unwrap());
        assert_eq!(found.executable, dir.join("files").join(LAUNCHER_EXE));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_game_release_is_not_a_launcher_release() {
        let dir = scratch("wrong-product");
        let release = stored(Product::Recof, vec![exe_entry()], dir.clone());
        assert!(pending(&release).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_launcher_release_with_no_executable_is_refused() {
        let dir = scratch("no-exe");
        std::fs::create_dir_all(dir.join("files")).unwrap();
        let release = stored(
            Product::RecofLauncher,
            vec![ManifestFile {
                path: "readme.txt".to_owned(),
                size: 1,
                sha256: sha256_bytes(b"x"),
                copy_of: None,
            }],
            dir.clone(),
        );
        assert!(matches!(pending(&release), Err(Error::MissingAsset(_))));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_manifest_entry_whose_file_is_not_there_is_refused() {
        let dir = scratch("missing-file");
        std::fs::create_dir_all(dir.join("files")).unwrap();
        let release = stored(Product::RecofLauncher, vec![exe_entry()], dir.clone());
        assert!(matches!(pending(&release), Err(Error::MissingAsset(_))));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_release_naming_the_executable_differently_still_works() {
        let dir = scratch("other-name");
        std::fs::create_dir_all(dir.join("files")).unwrap();
        std::fs::write(dir.join("files").join("launcher.exe"), b"exe!").unwrap();

        let release = stored(
            Product::RecofLauncher,
            vec![ManifestFile {
                path: "launcher.exe".to_owned(),
                size: 4,
                sha256: sha256_bytes(b"exe!"),
                copy_of: None,
            }],
            dir.clone(),
        );
        assert_eq!(
            pending(&release).unwrap().executable,
            dir.join("files").join("launcher.exe")
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_version_gate_compares_the_right_way_round() {
        let running = Version::parse("0.3.0").unwrap();
        assert!(launcher_too_old(
            &Version::parse("0.4.0").unwrap(),
            &running
        ));
        assert!(!launcher_too_old(
            &Version::parse("0.3.0").unwrap(),
            &running
        ));
        assert!(!launcher_too_old(
            &Version::parse("0.2.0").unwrap(),
            &running
        ));
        // A prerelease of the next version still counts as newer.
        assert!(launcher_too_old(
            &Version::parse("0.4.0-beta.1").unwrap(),
            &running
        ));
    }

    #[test]
    fn replacing_with_a_file_that_is_not_there_is_refused() {
        let missing = std::env::temp_dir().join("recof-no-such-launcher.exe");
        let _ = std::fs::remove_file(&missing);
        assert!(matches!(apply(&missing), Err(Error::MissingAsset(_))));
    }
}
