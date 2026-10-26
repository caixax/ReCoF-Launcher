//! What a release says about itself.
//!
//! A release is three files: the zip, `recof-manifest.json`, and
//! `recof-manifest.json.sig`. The manifest is the only thing the launcher
//! trusts, and only after the signature over its exact bytes checks out
//! against a key compiled into the executable. Everything the launcher then
//! does, including which files it writes and where, comes from the
//! manifest and never from a list hardcoded here.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::version::Version;

/// Schema version of the manifest format itself. A launcher refuses a
/// manifest from the future rather than guessing what the new fields mean.
pub const SCHEMA_VERSION: u32 = 1;

/// Canonical file name of a manifest.
pub const MANIFEST_NAME: &str = "recof-manifest.json";

/// Canonical file name of a launcher manifest.
pub const LAUNCHER_MANIFEST_NAME: &str = "recof-launcher-manifest.json";

/// Signature files are the manifest name with this appended.
pub const SIGNATURE_SUFFIX: &str = ".sig";

/// Which product a manifest describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Product {
    /// The game files that go into the ReCoF copy.
    Recof,
    /// The launcher executable itself.
    RecofLauncher,
}

impl Product {
    pub fn as_str(self) -> &'static str {
        match self {
            Product::Recof => "recof",
            Product::RecofLauncher => "recof-launcher",
        }
    }
}

/// Release channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Channel {
    /// GitHub's `releases/latest`, which skips drafts and prereleases.
    #[default]
    Stable,
    /// The release list, prereleases included.
    Beta,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Beta => "beta",
        }
    }
}

/// The archive the files come out of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Archive {
    /// File name of the asset in the GitHub release.
    pub name: String,
    pub size: u64,
    /// Lower case hex SHA-256 of the whole zip.
    pub sha256: String,
}

/// One file the release installs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestFile {
    /// Path inside the install, relative, with forward slashes.
    pub path: String,
    pub size: u64,
    /// Lower case hex SHA-256 of the file as it must end up on disk.
    pub sha256: String,
    /// When set, this file is not in the archive: it is a copy of another
    /// entry. Retail ships `sw.dll` as a byte identical copy of `hw.dll`
    /// and ReCoF keeps doing that, so the archive carries it once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_of: Option<String>,
}

impl ManifestFile {
    /// Whether this entry is written by copying another one.
    pub fn is_copy(&self) -> bool {
        self.copy_of.is_some()
    }
}

/// A signed description of one release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Must equal [`SCHEMA_VERSION`].
    pub schema: u32,
    pub product: Product,
    pub version: Version,
    pub channel: Channel,
    /// RFC 3339 timestamp, for display and for ordering equal versions.
    pub released: String,
    /// Steam buildid this release was built against. Absent for a launcher
    /// manifest, which does not care about the game build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_buildid: Option<u64>,
    /// Oldest launcher that may install this release. A ReCoF release that
    /// needs a newer launcher forces the launcher to update first.
    pub min_launcher: Version,
    /// Where the human readable notes live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes_url: Option<String>,
    pub archive: Archive,
    /// Every file the release installs, in the order it should be applied.
    pub files: Vec<ManifestFile>,
    /// Extra data files shipped with the release, by role. The launcher
    /// looks for the `cvars` role to build the ReCoF options screen.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub data: BTreeMap<String, String>,
}

impl Manifest {
    /// The file that lists the ReCoF cvars, if the release ships one.
    pub fn cvars_file(&self) -> Option<&str> {
        self.data.get("cvars").map(String::as_str)
    }

    /// Total size of everything the release writes.
    pub fn installed_size(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }

    /// Looks a file up by path, comparing without case.
    pub fn file(&self, path: &str) -> Option<&ManifestFile> {
        let wanted = normalize_relative(path);
        self.files
            .iter()
            .find(|file| normalize_relative(&file.path) == wanted)
    }
}

/// Lower case with forward slashes, for comparing two relative paths.
pub fn normalize_relative(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn sample() -> Manifest {
        Manifest {
            schema: SCHEMA_VERSION,
            product: Product::Recof,
            version: Version::parse("0.3.0").unwrap(),
            channel: Channel::Stable,
            released: "2026-10-26T12:00:00Z".to_owned(),
            target_buildid: Some(crate::consts::EXPECTED_BUILDID),
            min_launcher: Version::parse("0.3.0").unwrap(),
            notes_url: Some("https://github.com/caixax/ReCoF/releases/tag/v0.3.0".to_owned()),
            archive: Archive {
                name: "recof-0.3.0.zip".to_owned(),
                size: 4_200_000,
                sha256: "a".repeat(64),
            },
            files: vec![
                ManifestFile {
                    path: "hw.dll".to_owned(),
                    size: 2_121_828,
                    sha256: "b".repeat(64),
                    copy_of: None,
                },
                ManifestFile {
                    path: "sw.dll".to_owned(),
                    size: 2_121_828,
                    sha256: "b".repeat(64),
                    copy_of: Some("hw.dll".to_owned()),
                },
            ],
            data: BTreeMap::from([("cvars".to_owned(), "recof-cvars.json".to_owned())]),
        }
    }

    #[test]
    fn round_trips_through_json() {
        let manifest = sample();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, manifest);
    }

    #[test]
    fn the_json_uses_the_field_names_the_format_documents() {
        let json = serde_json::to_value(sample()).unwrap();
        assert_eq!(json["schema"], 1);
        assert_eq!(json["product"], "recof");
        assert_eq!(json["channel"], "stable");
        assert_eq!(json["version"], "0.3.0");
        assert_eq!(json["target_buildid"], 5_821_301u64);
        assert_eq!(json["min_launcher"], "0.3.0");
        assert_eq!(json["archive"]["sha256"], "a".repeat(64));
        assert_eq!(json["files"][1]["copy_of"], "hw.dll");
        assert_eq!(json["data"]["cvars"], "recof-cvars.json");
    }

    #[test]
    fn optional_fields_may_be_absent() {
        let json = r#"{
            "schema": 1,
            "product": "recof-launcher",
            "version": "0.1.0",
            "channel": "beta",
            "released": "2026-10-26T12:00:00Z",
            "min_launcher": "0.1.0",
            "archive": { "name": "x.zip", "size": 1, "sha256": "ab" },
            "files": [{ "path": "ReCoF-Launcher.exe", "size": 1, "sha256": "cd" }]
        }"#;
        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.product, Product::RecofLauncher);
        assert_eq!(manifest.channel, Channel::Beta);
        assert_eq!(manifest.target_buildid, None);
        assert_eq!(manifest.notes_url, None);
        assert!(manifest.data.is_empty());
        assert!(manifest.cvars_file().is_none());
    }

    #[test]
    fn an_unknown_product_or_channel_is_refused() {
        let base = sample();
        let mut json = serde_json::to_value(&base).unwrap();
        json["product"] = serde_json::json!("something-else");
        assert!(serde_json::from_value::<Manifest>(json).is_err());

        let mut json = serde_json::to_value(&base).unwrap();
        json["channel"] = serde_json::json!("nightly");
        assert!(serde_json::from_value::<Manifest>(json).is_err());
    }

    #[test]
    fn file_lookup_ignores_case_and_separators() {
        let manifest = sample();
        assert!(manifest.file("HW.DLL").is_some());
        assert!(manifest.file("hw.dll").is_some());
        assert!(manifest.file("nothing.dll").is_none());
    }

    #[test]
    fn a_copy_entry_is_marked_as_one() {
        let manifest = sample();
        assert!(!manifest.file("hw.dll").unwrap().is_copy());
        assert!(manifest.file("sw.dll").unwrap().is_copy());
        assert_eq!(manifest.installed_size(), 2 * 2_121_828);
    }
}
