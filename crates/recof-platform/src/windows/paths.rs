//! Known folders.
//!
//! Read through `SHGetKnownFolderPath` rather than the `LOCALAPPDATA`
//! environment variable, which a parent process can set to anything.

use std::path::PathBuf;

use windows::core::GUID;
use windows::Win32::Foundation::S_OK;
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_LocalAppData, FOLDERID_Programs, SHGetKnownFolderPath,
    KF_FLAG_DEFAULT,
};

use crate::{PlatformError, Result};

/// `%LOCALAPPDATA%`, where the launcher keeps settings, logs, backups, cache
/// and the version store.
pub fn local_app_data() -> Result<PathBuf> {
    known_folder(&FOLDERID_LocalAppData, "FOLDERID_LocalAppData")
}

/// The user's desktop, for the optional shortcut.
pub fn desktop_dir() -> Result<PathBuf> {
    known_folder(&FOLDERID_Desktop, "FOLDERID_Desktop")
}

/// The user's start menu programs folder, for the optional shortcut.
pub fn start_menu_programs_dir() -> Result<PathBuf> {
    known_folder(&FOLDERID_Programs, "FOLDERID_Programs")
}

fn known_folder(id: &GUID, name: &'static str) -> Result<PathBuf> {
    // SAFETY: `id` is one of the FOLDERID constants, the token is null for
    // the current user, and the returned buffer is freed before returning.
    unsafe {
        let wide = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).map_err(|error| {
            PlatformError::Api {
                api: "SHGetKnownFolderPath",
                detail: format!("{name}: {error}"),
            }
        })?;

        if wide.is_null() {
            return Err(PlatformError::Api {
                api: "SHGetKnownFolderPath",
                detail: format!("{name}: returned a null path"),
            });
        }

        let path = wide.to_string().map_err(|error| PlatformError::Api {
            api: "SHGetKnownFolderPath",
            detail: format!("{name}: path is not valid UTF-16: {error}"),
        });

        windows::Win32::System::Com::CoTaskMemFree(Some(wide.0 as *const _));

        let _ = S_OK;
        Ok(PathBuf::from(path?))
    }
}
