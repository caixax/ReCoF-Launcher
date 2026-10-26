//! What retail Cry of Fear 1.6a looks like.
//!
//! The hashes come from `docs/versions.csv` in the ReCoF repository, rows
//! with version `1.6a`, and were confirmed against a real Steam install of
//! build 5821301. `sw.dll` is not in that file because retail ships it as a
//! byte identical copy of `hw.dll`, which is also what `tools/make_dist.py`
//! assumes.
//!
//! Nothing here blocks an install. An unknown hash is reported file by file
//! so the user knows what they are copying, and that is all.

/// A file ReCoF cares about, with the hash retail ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownFile {
    /// Path relative to the install root, with forward slashes.
    pub path: &'static str,
    /// Size of the retail file, in bytes.
    pub size: u64,
    /// SHA-256 of the retail file, lower case hex.
    pub sha256: &'static str,
    /// Whether a ReCoF release may replace this file. Informational: the
    /// real list always comes from the signed manifest.
    pub replaced_by_recof: bool,
    /// Other hashes that are known and explainable rather than suspicious.
    pub variants: &'static [KnownVariant],
}

/// A hash that is not retail but is understood.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownVariant {
    pub sha256: &'static str,
    pub size: u64,
    /// Fluent message id describing what this variant is.
    pub message_id: &'static str,
}

/// The eight binaries that make up the game, as retail 1.6a ships them.
pub const RETAIL_1_6A: &[KnownFile] = &[
    KnownFile {
        path: "cof.exe",
        size: 90_173,
        sha256: "d8d5006ecc06f2b4523512f324aa662dc078cb2765f492a8823563ac84282aea",
        replaced_by_recof: true,
        variants: &[],
    },
    KnownFile {
        path: "hw.dll",
        size: 2_121_828,
        sha256: "9dd34e536c4bb7cda3bc1bc4f0f5f6163687566a16f35c0ea0be59f93da62875",
        replaced_by_recof: true,
        variants: &[],
    },
    KnownFile {
        // Retail ships this as a byte identical copy of hw.dll.
        path: "sw.dll",
        size: 2_121_828,
        sha256: "9dd34e536c4bb7cda3bc1bc4f0f5f6163687566a16f35c0ea0be59f93da62875",
        replaced_by_recof: true,
        variants: &[],
    },
    KnownFile {
        path: "opengl32.dll",
        size: 57_344,
        sha256: "26732d7a9aeb8079f2d0c0920f7e433521a2fcc570017c7dafd45bdd30cdbeab",
        replaced_by_recof: true,
        variants: &[],
    },
    KnownFile {
        // ReCoF releases carry the rebuilt settings dialog.
        path: "CoFLaunchApp.exe",
        size: 2_683_904,
        sha256: "bbdbac8c321e5f7561e2669f333dad3261b94f67459beb1907ce5164af6e54cb",
        replaced_by_recof: true,
        variants: &[KnownVariant {
            // Seen on a real install: the Cry of Fear Enhanced community
            // patch replaces this file and keeps the retail one in
            // cof-enhanced-backup.
            sha256: "d4ea1177e96a184e39aa7ed65553379ade583dd5d945071e6b69bc1d3f58733f",
            size: 208_896,
            message_id: "variant-cof-enhanced",
        }],
    },
    KnownFile {
        path: "cryoffear/cl_dlls/client.dll",
        size: 1_754_112,
        sha256: "d2a04641b301804f6f449aa68265042b13adc360925b80033d417ec9f38c9c00",
        replaced_by_recof: true,
        variants: &[],
    },
    KnownFile {
        path: "cryoffear/cl_dlls/hl.dll",
        size: 2_413_568,
        sha256: "0036b91c01e92ed205513f52563053a55a66a32d257efb5f476b8f1f3dde0e63",
        replaced_by_recof: true,
        variants: &[],
    },
    KnownFile {
        path: "cryoffear/cl_dlls/GameUI.dll",
        size: 1_056_863,
        sha256: "8255926f84eaa08e9096f2f78dea7f2827c3733e7af8f4408efa0b423acca838",
        replaced_by_recof: true,
        variants: &[],
    },
];

/// Executable files a retail install has at its root.
///
/// Taken from a real build 5821301 install, minus the files a community
/// patch had added there. Used only to tell the user that something extra
/// is present; an unlisted name is a notice, never a refusal, because this
/// list cannot be proved complete without the Steam depot file list.
pub const RETAIL_ROOT_BINARIES: &[&str] = &[
    "a3dapi.dll",
    "cg.dll",
    "cggl.dll",
    "cof.exe",
    "coflaunchapp.exe",
    "crashhandler.dll",
    "dbg.dll",
    "demoplayer.dll",
    "devil.dll",
    "filesystem_stdio.dll",
    "filesystem_steam.dll",
    "hlds.exe",
    "hltv.exe",
    "hw.dll",
    "id3lib.dll",
    "ikpflac.dll",
    "ikpmp3.dll",
    "irrklang.dll",
    "mp3dec.asi",
    "mss32.dll",
    "mssv12.asi",
    "mssv29.asi",
    "opengl32.dll",
    "sierrapt.dll",
    "sierraupres.dll",
    "sierraupres_ati.dll",
    "steam.dll",
    "steam_api.dll",
    "steam_api_c.dll",
    "steamapiupdater.dll",
    "steamclient.dll",
    "survey1109.dll",
    "sw.dll",
    "swds.dll",
    "symsrv.dll",
    "tier0.dll",
    "tier0_s.dll",
    "trackernet.dll",
    "vgui.dll",
    "vgui2.dll",
    "voice_miles.dll",
    "voice_speex.dll",
    "vstdlib.dll",
    "vstdlib_s.dll",
];

/// Executable files retail has in `cryoffear/cl_dlls`.
pub const RETAIL_GAME_BINARIES: &[&str] = &["client.dll", "gameui.dll", "hl.dll"];

/// Names and folders that identify a known third party modification, so the
/// notice can say what it is instead of only that something is different.
pub const MOD_MARKERS: &[ModMarker] = &[
    ModMarker {
        relative_path: "cof-enhanced",
        message_id: "mod-cof-enhanced",
    },
    ModMarker {
        relative_path: "xash.dll",
        message_id: "mod-xash",
    },
    ModMarker {
        relative_path: "BunnymodXT.dll",
        message_id: "mod-bunnymod-xt",
    },
    ModMarker {
        relative_path: "cryoffear/cl_dlls/menu.dll",
        message_id: "mod-xash-menu",
    },
];

/// A file or folder whose presence names a known modification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModMarker {
    /// Path relative to the install root, with forward slashes.
    pub relative_path: &'static str,
    pub message_id: &'static str,
}

/// Looks a path up in the retail table, comparing without case.
pub fn known_file(relative_path: &str) -> Option<&'static KnownFile> {
    let wanted = normalize_path(relative_path);
    RETAIL_1_6A
        .iter()
        .find(|entry| normalize_path(entry.path) == wanted)
}

/// Lower case with forward slashes, so `cryoffear\CL_DLLS\HL.DLL` and
/// `cryoffear/cl_dlls/hl.dll` are the same path.
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hash_is_lower_case_hex_of_the_right_length() {
        for entry in RETAIL_1_6A {
            assert_eq!(entry.sha256.len(), 64, "{}", entry.path);
            assert!(
                entry
                    .sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit()
                        && !character.is_ascii_uppercase()),
                "{}",
                entry.path
            );
            for variant in entry.variants {
                assert_eq!(variant.sha256.len(), 64, "{}", entry.path);
            }
        }
    }

    #[test]
    fn paths_use_forward_slashes_and_are_relative() {
        for entry in RETAIL_1_6A {
            assert!(!entry.path.contains('\\'), "{}", entry.path);
            assert!(!entry.path.starts_with('/'), "{}", entry.path);
            assert!(!entry.path.contains(".."), "{}", entry.path);
        }
    }

    #[test]
    fn the_table_has_no_duplicate_paths() {
        let mut seen = std::collections::HashSet::new();
        for entry in RETAIL_1_6A {
            assert!(seen.insert(normalize_path(entry.path)), "{}", entry.path);
        }
    }

    #[test]
    fn sw_dll_is_the_same_file_as_hw_dll() {
        let hw = known_file("hw.dll").unwrap();
        let sw = known_file("sw.dll").unwrap();
        assert_eq!(hw.sha256, sw.sha256);
        assert_eq!(hw.size, sw.size);
    }

    #[test]
    fn lookup_ignores_case_and_separator() {
        assert!(known_file(r"cryoffear\cl_dlls\HL.dll").is_some());
        assert!(known_file("CRYOFFEAR/CL_DLLS/hl.dll").is_some());
        assert!(known_file("nothing.dll").is_none());
    }

    #[test]
    fn the_settings_dialog_is_replaced_by_a_release() {
        assert!(known_file("CoFLaunchApp.exe").unwrap().replaced_by_recof);
    }

    #[test]
    fn root_binary_names_are_stored_lower_case() {
        for name in RETAIL_ROOT_BINARIES {
            assert_eq!(*name, name.to_lowercase(), "{name}");
        }
        for name in RETAIL_GAME_BINARIES {
            assert_eq!(*name, name.to_lowercase(), "{name}");
        }
    }

    #[test]
    fn every_in_scope_root_binary_is_in_the_allowlist() {
        for entry in RETAIL_1_6A {
            if entry.path.contains('/') {
                continue;
            }
            assert!(
                RETAIL_ROOT_BINARIES.contains(&entry.path.to_lowercase().as_str()),
                "{} is missing from the root allowlist",
                entry.path
            );
        }
    }
}
