//! Looking at an install and saying what it is.
//!
//! The 4.4 GiB of assets are trusted through the Steam buildid, as the
//! feasibility study concluded: hashing them on every start would take
//! minutes and prove little. What is hashed is the eight binaries ReCoF
//! cares about, plus a cheap look for extra executables and for the
//! fingerprints of known community patches.
//!
//! The result never blocks an install. It is shown to the user so they know
//! what they are about to copy.

use std::path::Path;

use crate::hashes::pe::{self, Normalized};
use crate::hashes::table::{
    KnownFile, ModMarker, MOD_MARKERS, RETAIL_1_6A, RETAIL_GAME_BINARIES, RETAIL_ROOT_BINARIES,
};
use crate::hashes::{digests_equal, sha256_bytes};
use crate::{Error, Result};

/// What one file turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileVerdict {
    /// Byte for byte the retail file.
    Retail,
    /// The retail file with the 4 GB patch applied: the large address flag
    /// and the PE checksum differ and nothing else does.
    RetailPatched {
        large_address_aware: bool,
        checksum: u32,
    },
    /// A hash that is not retail but is understood, named by its message id.
    KnownVariant { message_id: &'static str },
    /// Something else entirely.
    Unknown { sha256: String, size: u64 },
    /// The file is not there.
    Missing,
}

impl FileVerdict {
    /// Whether this verdict is worth showing the user.
    pub fn is_notable(&self) -> bool {
        !matches!(self, FileVerdict::Retail)
    }
}

/// One line of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReport {
    /// Path relative to the install root, with forward slashes.
    pub path: String,
    pub verdict: FileVerdict,
    /// Whether a ReCoF release replaces this file in the copy, which is
    /// what decides how much an odd hash actually matters.
    pub replaced_by_recof: bool,
}

/// What a whole install looks like.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallReport {
    /// One entry per file in the retail table, in table order.
    pub files: Vec<FileReport>,
    /// Executables found at the install root or in `cryoffear/cl_dlls` that
    /// retail does not ship.
    pub unexpected_binaries: Vec<String>,
    /// Message ids of community patches whose fingerprint was found.
    pub mod_markers: Vec<&'static str>,
}

impl InstallReport {
    /// Whether every in scope file is byte for byte retail and nothing extra
    /// was found.
    pub fn is_untouched_retail(&self) -> bool {
        self.unexpected_binaries.is_empty()
            && self.mod_markers.is_empty()
            && self
                .files
                .iter()
                .all(|file| file.verdict == FileVerdict::Retail)
    }

    /// The entries worth showing, which is usually none.
    pub fn notable(&self) -> impl Iterator<Item = &FileReport> {
        self.files.iter().filter(|file| file.verdict.is_notable())
    }

    /// Whether anything at all should be shown to the user.
    pub fn has_notices(&self) -> bool {
        !self.is_untouched_retail()
    }

    /// Whether a file ReCoF is going to replace anyway is the odd one out.
    /// Those matter least: the copy overwrites them.
    pub fn only_replaced_files_differ(&self) -> bool {
        self.unexpected_binaries.is_empty()
            && self.mod_markers.is_empty()
            && self.notable().all(|file| file.replaced_by_recof)
    }
}

/// Hashes the in scope binaries of an install and looks for extras.
pub fn scan_install(root: &Path) -> Result<InstallReport> {
    let mut report = InstallReport::default();

    for entry in RETAIL_1_6A {
        report.files.push(check_file(root, entry)?);
    }
    report.unexpected_binaries = unexpected_binaries(root)?;
    report.mod_markers = mod_markers(root);

    Ok(report)
}

/// Decides what one known file is.
fn check_file(root: &Path, entry: &KnownFile) -> Result<FileReport> {
    let path = root.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let verdict = match std::fs::read(&path) {
        Ok(bytes) => verdict_for(entry, &bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => FileVerdict::Missing,
        Err(error) => return Err(Error::io(&path, error)),
    };
    Ok(FileReport {
        path: entry.path.to_owned(),
        verdict,
        replaced_by_recof: entry.replaced_by_recof,
    })
}

/// The decision itself, split out so it can be tested without a filesystem.
pub fn verdict_for(entry: &KnownFile, bytes: &[u8]) -> FileVerdict {
    let digest = sha256_bytes(bytes);
    if digests_equal(&digest, entry.sha256) {
        return FileVerdict::Retail;
    }

    // Second chance: the 4 GB patch changes only the large address flag and
    // the PE checksum, and a retail binary carries neither.
    if let Normalized::Pe {
        bytes: normalized,
        large_address_aware,
        checksum,
    } = pe::normalize(bytes)
    {
        if (large_address_aware || checksum != 0)
            && digests_equal(&sha256_bytes(&normalized), entry.sha256)
        {
            return FileVerdict::RetailPatched {
                large_address_aware,
                checksum,
            };
        }
    }

    for variant in entry.variants {
        if digests_equal(&digest, variant.sha256) {
            return FileVerdict::KnownVariant {
                message_id: variant.message_id,
            };
        }
    }

    FileVerdict::Unknown {
        sha256: digest,
        size: bytes.len() as u64,
    }
}

/// Executables at the install root and in `cryoffear/cl_dlls` that retail
/// does not ship.
fn unexpected_binaries(root: &Path) -> Result<Vec<String>> {
    let mut found = Vec::new();
    collect_unexpected(root, "", RETAIL_ROOT_BINARIES, &mut found)?;
    collect_unexpected(
        &root.join("cryoffear").join("cl_dlls"),
        "cryoffear/cl_dlls/",
        RETAIL_GAME_BINARIES,
        &mut found,
    )?;
    found.sort();
    Ok(found)
}

fn collect_unexpected(
    directory: &Path,
    prefix: &str,
    allowed: &[&str],
    found: &mut Vec<String>,
) -> Result<()> {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(Error::io(directory, error)),
    };

    for entry in entries {
        let entry = entry.map_err(|error| Error::io(directory, error))?;
        if !entry
            .file_type()
            .map(|kind| kind.is_file())
            .unwrap_or(false)
        {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let lowered = name.to_lowercase();
        let is_executable =
            lowered.ends_with(".dll") || lowered.ends_with(".exe") || lowered.ends_with(".asi");
        if is_executable && !allowed.contains(&lowered.as_str()) {
            found.push(format!("{prefix}{name}"));
        }
    }
    Ok(())
}

/// Known community patches, recognised by a file or folder they leave.
fn mod_markers(root: &Path) -> Vec<&'static str> {
    MOD_MARKERS
        .iter()
        .filter(|marker: &&ModMarker| {
            root.join(
                marker
                    .relative_path
                    .replace('/', std::path::MAIN_SEPARATOR_STR),
            )
            .exists()
        })
        .map(|marker| marker.message_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashes::table::KnownVariant;

    /// A retail file is whatever these bytes hash to.
    fn entry_for(bytes: &[u8], variants: &'static [KnownVariant]) -> KnownFile {
        // The table is a set of &'static str, so leak the digest. Only the
        // tests do this, and only a handful of times.
        let digest: &'static str = Box::leak(sha256_bytes(bytes).into_boxed_str());
        KnownFile {
            path: "cof.exe",
            size: bytes.len() as u64,
            sha256: digest,
            replaced_by_recof: true,
            variants,
        }
    }

    /// A structurally valid PE32 whose payload is `seed`.
    fn fake_pe(characteristics: u16, checksum: u32, seed: u8) -> Vec<u8> {
        let nt = 0x80usize;
        let mut bytes = vec![0u8; nt + 4 + 20 + 240];
        bytes[0..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&(nt as u32).to_le_bytes());
        bytes[nt..nt + 4].copy_from_slice(b"PE\0\0");
        let coff = nt + 4;
        bytes[coff + 18..coff + 20].copy_from_slice(&characteristics.to_le_bytes());
        let optional = coff + 20;
        bytes[optional..optional + 2].copy_from_slice(&0x010bu16.to_le_bytes());
        bytes[optional + 64..optional + 68].copy_from_slice(&checksum.to_le_bytes());
        let last = bytes.len() - 1;
        bytes[last] = seed;
        bytes
    }

    #[test]
    fn an_exact_match_is_retail() {
        let bytes = fake_pe(0x010f, 0, 1);
        let entry = entry_for(&bytes, &[]);
        assert_eq!(verdict_for(&entry, &bytes), FileVerdict::Retail);
    }

    #[test]
    fn the_four_gigabyte_patch_is_recognised_as_retail_patched() {
        let retail = fake_pe(0x010f, 0, 1);
        let entry = entry_for(&retail, &[]);
        let patched = fake_pe(0x010f | 0x0020, 0x0002_1a7f, 1);

        assert_eq!(
            verdict_for(&entry, &patched),
            FileVerdict::RetailPatched {
                large_address_aware: true,
                checksum: 0x0002_1a7f,
            }
        );
    }

    #[test]
    fn a_known_variant_is_named() {
        let retail = fake_pe(0x010f, 0, 1);
        let other = fake_pe(0x010f, 0, 2);
        let digest: &'static str = Box::leak(sha256_bytes(&other).into_boxed_str());
        let variants: &'static [KnownVariant] = Box::leak(Box::new([KnownVariant {
            sha256: digest,
            size: other.len() as u64,
            message_id: "variant-cof-enhanced",
        }]));
        let entry = entry_for(&retail, variants);

        assert_eq!(
            verdict_for(&entry, &other),
            FileVerdict::KnownVariant {
                message_id: "variant-cof-enhanced"
            }
        );
    }

    #[test]
    fn anything_else_is_unknown_and_reports_its_hash() {
        let retail = fake_pe(0x010f, 0, 1);
        let entry = entry_for(&retail, &[]);
        let other = fake_pe(0x010f, 0, 9);

        match verdict_for(&entry, &other) {
            FileVerdict::Unknown { sha256, size } => {
                assert_eq!(sha256, sha256_bytes(&other));
                assert_eq!(size, other.len() as u64);
            }
            other => panic!("expected unknown, got {other:?}"),
        }
    }

    #[test]
    fn a_non_pe_file_with_the_wrong_hash_is_unknown_not_a_crash() {
        let entry = entry_for(b"retail bytes", &[]);
        assert!(matches!(
            verdict_for(&entry, b"something else"),
            FileVerdict::Unknown { .. }
        ));
    }

    #[test]
    fn normalization_cannot_turn_a_different_file_into_retail() {
        // Two files that differ in their payload must never normalize to
        // the same thing, however their headers are patched.
        let retail = fake_pe(0x010f, 0, 1);
        let entry = entry_for(&retail, &[]);
        let different = fake_pe(0x010f | 0x0020, 0x1234, 2);
        assert!(matches!(
            verdict_for(&entry, &different),
            FileVerdict::Unknown { .. }
        ));
    }

    #[test]
    fn scanning_a_tree_reports_missing_files_rather_than_failing() {
        let root = std::env::temp_dir().join("recof-scan-empty");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let report = scan_install(&root).expect("an empty folder still scans");
        assert_eq!(report.files.len(), RETAIL_1_6A.len());
        assert!(report
            .files
            .iter()
            .all(|file| file.verdict == FileVerdict::Missing));
        assert!(report.has_notices());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn extra_executables_and_mod_markers_are_found() {
        let root = std::env::temp_dir().join("recof-scan-modded");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("cryoffear").join("cl_dlls")).unwrap();
        std::fs::create_dir_all(root.join("cof-enhanced")).unwrap();

        std::fs::write(root.join("xash.dll"), b"mod").unwrap();
        std::fs::write(root.join("vgui.dll"), b"retail name").unwrap();
        std::fs::write(root.join("readme.txt"), b"not an executable").unwrap();
        std::fs::write(root.join("cryoffear/cl_dlls/menu.dll"), b"mod").unwrap();

        let report = scan_install(&root).unwrap();
        assert_eq!(
            report.unexpected_binaries,
            vec![
                "cryoffear/cl_dlls/menu.dll".to_owned(),
                "xash.dll".to_owned()
            ]
        );
        assert!(report.mod_markers.contains(&"mod-cof-enhanced"));
        assert!(report.mod_markers.contains(&"mod-xash"));
        assert!(report.mod_markers.contains(&"mod-xash-menu"));
        assert!(!report.is_untouched_retail());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_path_with_spaces_and_non_ascii_scans() {
        let root = std::env::temp_dir().join("Librería ReCoF/Mi copia");
        let _ = std::fs::remove_dir_all(std::env::temp_dir().join("Librería ReCoF"));
        std::fs::create_dir_all(&root).unwrap();
        assert!(scan_install(&root).is_ok());
        std::fs::remove_dir_all(std::env::temp_dir().join("Librería ReCoF")).unwrap();
    }

    #[test]
    fn table_lookup_is_wired_up() {
        assert!(crate::hashes::table::known_file("hw.dll").is_some());
    }
}
