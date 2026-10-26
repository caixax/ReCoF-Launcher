//! Desktop and start menu shortcuts.
//!
//! Written through `IShellLinkW`, the same way Explorer writes them, so a
//! `.lnk` the launcher creates behaves like any other.

use std::path::Path;

use windows::core::{Interface, PCWSTR};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

use crate::{PlatformError, Result};

/// What a shortcut points at.
#[derive(Debug, Clone)]
pub struct Shortcut<'a> {
    /// Where the `.lnk` goes, including the file name.
    pub link_path: &'a Path,
    pub target: &'a Path,
    pub working_dir: &'a Path,
    pub arguments: &'a str,
    pub description: &'a str,
}

/// Creates or replaces a shortcut.
pub fn create_shortcut(shortcut: &Shortcut<'_>) -> Result<()> {
    if let Some(parent) = shortcut.link_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // SAFETY: COM is initialised and uninitialised on this thread around
    // the whole operation, and every pointer handed to the API comes from
    // a NUL terminated buffer that outlives the call.
    unsafe {
        let initialised = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        // S_FALSE means COM was already up on this thread, which is fine;
        // only this call's own initialisation is undone at the end.
        let needs_uninit = initialised.is_ok();

        let result = write_link(shortcut);

        if needs_uninit {
            CoUninitialize();
        }
        result
    }
}

/// SAFETY: the caller has initialised COM on this thread.
unsafe fn write_link(shortcut: &Shortcut<'_>) -> Result<()> {
    let link: IShellLinkW =
        CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(|error| {
            PlatformError::Api {
                api: "CoCreateInstance(ShellLink)",
                detail: error.to_string(),
            }
        })?;

    let target = wide(&shortcut.target.to_string_lossy());
    link.SetPath(PCWSTR(target.as_ptr()))
        .map_err(|error| api_error("IShellLink::SetPath", error))?;

    let working = wide(&shortcut.working_dir.to_string_lossy());
    link.SetWorkingDirectory(PCWSTR(working.as_ptr()))
        .map_err(|error| api_error("IShellLink::SetWorkingDirectory", error))?;

    if !shortcut.arguments.is_empty() {
        let arguments = wide(shortcut.arguments);
        link.SetArguments(PCWSTR(arguments.as_ptr()))
            .map_err(|error| api_error("IShellLink::SetArguments", error))?;
    }

    if !shortcut.description.is_empty() {
        // The description is what Explorer shows as the tooltip, and it
        // is capped at 260 characters.
        let trimmed: String = shortcut.description.chars().take(259).collect();
        let description = wide(&trimmed);
        link.SetDescription(PCWSTR(description.as_ptr()))
            .map_err(|error| api_error("IShellLink::SetDescription", error))?;
    }

    // The icon comes from the target, which is what Explorer does by
    // default for a program.
    link.SetIconLocation(PCWSTR(target.as_ptr()), 0)
        .map_err(|error| api_error("IShellLink::SetIconLocation", error))?;

    let persist: IPersistFile = link
        .cast()
        .map_err(|error| api_error("IShellLink as IPersistFile", error))?;
    let link_path = wide(&shortcut.link_path.to_string_lossy());
    persist
        .Save(PCWSTR(link_path.as_ptr()), true)
        .map_err(|error| api_error("IPersistFile::Save", error))?;

    Ok(())
}

fn api_error(api: &'static str, error: windows::core::Error) -> PlatformError {
    PlatformError::Api {
        api,
        detail: error.to_string(),
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shortcut_is_written_and_can_be_read_back_by_the_shell() {
        let base = std::env::temp_dir().join("recof-shortcut-lnk");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        // Pointing at a real program, so the shell has something valid.
        let target = std::env::current_exe().unwrap();
        let link = base.join("ReCoF de prueba ñ.lnk");

        create_shortcut(&Shortcut {
            link_path: &link,
            target: &target,
            working_dir: &base,
            arguments: "-basedir cryoffear -game cryoffear",
            description: "ReCoF",
        })
        .expect("a shortcut can be written");

        assert!(link.is_file());
        assert!(
            std::fs::metadata(&link).unwrap().len() > 0,
            "an empty .lnk is not a shortcut"
        );

        // Writing it again replaces it rather than failing.
        create_shortcut(&Shortcut {
            link_path: &link,
            target: &target,
            working_dir: &base,
            arguments: "",
            description: "",
        })
        .expect("a shortcut can be replaced");

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_shortcut_in_a_folder_that_does_not_exist_yet_creates_it() {
        let base = std::env::temp_dir().join("recof-shortcut-nested");
        let _ = std::fs::remove_dir_all(&base);

        let link = base.join("sub/folder/ReCoF.lnk");
        create_shortcut(&Shortcut {
            link_path: &link,
            target: &std::env::current_exe().unwrap(),
            working_dir: &std::env::temp_dir(),
            arguments: "",
            description: "",
        })
        .unwrap();

        assert!(link.is_file());
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn a_very_long_description_does_not_make_the_call_fail() {
        let base = std::env::temp_dir().join("recof-shortcut-long");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        let link = base.join("long.lnk");
        create_shortcut(&Shortcut {
            link_path: &link,
            target: &std::env::current_exe().unwrap(),
            working_dir: &base,
            arguments: "",
            description: &"x".repeat(1000),
        })
        .unwrap();

        assert!(link.is_file());
        std::fs::remove_dir_all(&base).unwrap();
    }
}
