//! What is in a copy, recorded inside the copy itself.
//!
//! Kept in the install rather than in `%LOCALAPPDATA%` so that moving the
//! folder, or pointing a reinstalled launcher at it, does not lose track of
//! what version it holds.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::IoResultExt;
use crate::manifest::{Channel, Manifest};
use crate::paths::installed_state_file;
use crate::version::Version;
use crate::{Error, Result};

/// Format version of `installed.json`.
pub const STATE_SCHEMA: u32 = 1;

/// The record of what a ReCoF copy holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledState {
    pub schema: u32,
    /// Version of ReCoF that is applied.
    pub version: Version,
    pub channel: Channel,
    /// When it was applied, RFC 3339.
    pub installed_at: String,
    /// Launcher that applied it, for the bug report.
    pub launcher_version: String,
    /// Where the base game was copied from, for the bug report and so a
    /// repair can say what the copy was made of.
    pub source_path: String,
    /// Steam buildid of that source, when it was known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_buildid: Option<u64>,
    /// The manifest that was applied, kept whole so verify and repair work
    /// with no network and no version store.
    pub manifest: Manifest,
}

impl InstalledState {
    /// Reads the state from a copy, if it is a ReCoF copy at all.
    pub fn load(install_root: &Path) -> Result<Option<InstalledState>> {
        let path = installed_state_file(install_root);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(Error::io(&path, error)),
        };

        let state: InstalledState =
            serde_json::from_slice(&bytes).map_err(|error| Error::Journal {
                detail: format!("{} is not readable: {error}", path.display()),
            })?;
        if state.schema != STATE_SCHEMA {
            return Err(Error::Journal {
                detail: format!(
                    "{} uses schema {}, this launcher understands {STATE_SCHEMA}",
                    path.display(),
                    state.schema
                ),
            });
        }
        Ok(Some(state))
    }

    /// Writes the state, replacing the previous one atomically.
    pub fn save(&self, install_root: &Path) -> Result<()> {
        use std::io::Write;

        let path = installed_state_file(install_root);
        let directory = path.parent().expect("the state file is inside .recof");
        std::fs::create_dir_all(directory).at(directory)?;

        let temporary = path.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(self).map_err(|error| Error::Journal {
            detail: format!("the install state could not be serialised: {error}"),
        })?;
        {
            let mut file = std::fs::File::create(&temporary).at(&temporary)?;
            file.write_all(&json).at(&temporary)?;
            file.sync_all().at(&temporary)?;
        }
        std::fs::rename(&temporary, &path).at(&path)?;
        Ok(())
    }

    /// Whether a given launcher may work with this install.
    pub fn needs_newer_launcher(&self, running: &Version) -> bool {
        self.manifest.min_launcher > *running
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashes::sha256_bytes;
    use crate::manifest::schema::{Archive, ManifestFile, SCHEMA_VERSION};
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-state-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn state() -> InstalledState {
        InstalledState {
            schema: STATE_SCHEMA,
            version: Version::parse("0.3.0").unwrap(),
            channel: Channel::Stable,
            installed_at: "2026-10-26T12:00:00Z".to_owned(),
            launcher_version: "0.1.0".to_owned(),
            source_path: r"C:\Program Files (x86)\Steam\steamapps\common\Cry of Fear".to_owned(),
            source_buildid: Some(crate::consts::EXPECTED_BUILDID),
            manifest: Manifest {
                schema: SCHEMA_VERSION,
                product: crate::manifest::Product::Recof,
                version: Version::parse("0.3.0").unwrap(),
                channel: Channel::Stable,
                released: "2026-10-26T12:00:00Z".to_owned(),
                target_buildid: Some(crate::consts::EXPECTED_BUILDID),
                min_launcher: Version::parse("0.1.0").unwrap(),
                notes_url: None,
                archive: Archive {
                    name: "recof-0.3.0.zip".to_owned(),
                    size: 1,
                    sha256: sha256_bytes(b"x"),
                },
                files: vec![ManifestFile {
                    path: "hw.dll".to_owned(),
                    size: 1,
                    sha256: sha256_bytes(b"x"),
                    copy_of: None,
                }],
                data: Default::default(),
            },
        }
    }

    #[test]
    fn saving_and_loading_round_trips() {
        let root = scratch("round-trip");
        state().save(&root).unwrap();
        assert_eq!(InstalledState::load(&root).unwrap(), Some(state()));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_folder_that_is_not_a_recof_copy_reads_as_none() {
        let root = scratch("not-recof");
        assert_eq!(InstalledState::load(&root).unwrap(), None);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_state_lives_inside_the_copy() {
        let root = scratch("location");
        state().save(&root).unwrap();
        assert!(root.join(".recof").join("installed.json").is_file());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn saving_leaves_no_temporary_file() {
        let root = scratch("no-temp");
        state().save(&root).unwrap();
        let names: Vec<String> = std::fs::read_dir(root.join(".recof"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["installed.json".to_owned()]);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_corrupt_state_is_an_error_rather_than_a_silent_none() {
        let root = scratch("corrupt");
        std::fs::create_dir_all(root.join(".recof")).unwrap();
        std::fs::write(root.join(".recof/installed.json"), b"nope").unwrap();
        assert!(InstalledState::load(&root).is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_state_from_a_newer_launcher_is_refused() {
        let root = scratch("future");
        let mut state = state();
        state.schema = STATE_SCHEMA + 1;
        state.save(&root).unwrap();
        assert!(InstalledState::load(&root).is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_install_can_demand_a_newer_launcher() {
        let mut state = state();
        assert!(!state.needs_newer_launcher(&Version::parse("0.1.0").unwrap()));
        assert!(!state.needs_newer_launcher(&Version::parse("1.0.0").unwrap()));

        state.manifest.min_launcher = Version::parse("0.9.0").unwrap();
        assert!(state.needs_newer_launcher(&Version::parse("0.1.0").unwrap()));
        assert!(!state.needs_newer_launcher(&Version::parse("0.9.0").unwrap()));
    }
}
