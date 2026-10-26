//! Starting programs, and finding out whether Steam is running.

use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, HANDLE, WIN32_ERROR};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// `STILL_ACTIVE`, the exit code a process that has not exited reports.
const STILL_ACTIVE: u32 = 259;

use crate::{PlatformError, Result};

/// Where the Steam client records the pid of the running client.
const ACTIVE_PROCESS_KEY: &str = r"Software\Valve\Steam\ActiveProcess";

/// Whether the Steam client is running.
///
/// Steam writes its own pid under `ActiveProcess` and clears it on exit,
/// but a crash can leave it behind, so the pid is confirmed by opening the
/// process. A pid of zero means Steam is not running.
pub fn is_steam_running() -> Result<bool> {
    let Some(pid) = crate::read_u32(crate::RegRoot::CurrentUser, ACTIVE_PROCESS_KEY, "pid")? else {
        return Ok(false);
    };
    if pid == 0 {
        return Ok(false);
    }
    Ok(process_is_alive(pid))
}

/// Whether a process id belongs to something that is still running.
fn process_is_alive(pid: u32) -> bool {
    // SAFETY: a failed open returns an error rather than a usable handle,
    // and the handle is closed on every path out.
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) => handle,
            // Access denied means the process is there but belongs to
            // somebody else, which still counts as running. Any other
            // error, in particular an invalid parameter, means there is no
            // such process. The error has to be unwrapped from its HRESULT
            // before it can be compared with a Win32 code.
            Err(error) => {
                return WIN32_ERROR::from_error(&error) == Some(ERROR_ACCESS_DENIED);
            }
        };
        if handle.is_invalid() {
            return false;
        }

        let mut code = 0u32;
        let alive = GetExitCodeProcess(handle, &mut code).is_ok() && code == STILL_ACTIVE;
        let _ = CloseHandle(handle);
        alive
    }
}

/// Starts a program with a working directory, without waiting for it.
///
/// Arguments are passed as a list and quoted by the standard library, so a
/// path with a space cannot turn into two arguments.
pub fn spawn(program: &Path, args: &[String], working_dir: &Path) -> Result<u32> {
    let child = std::process::Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .spawn()
        .map_err(|error| PlatformError::Api {
            api: "CreateProcess",
            detail: format!("{}: {error}", program.display()),
        })?;
    Ok(child.id())
}

/// Opens a URL, a `steam://` link or a folder with whatever is registered
/// to handle it.
pub fn open_in_shell(target: &str) -> Result<()> {
    // `cmd /c start` is avoided: it would need the target quoted exactly
    // right and would flash a console window. `explorer` handles folders,
    // URLs and steam links the same way the shell does.
    std::process::Command::new("explorer.exe")
        .arg(target)
        .spawn()
        .map_err(|error| PlatformError::Api {
            api: "ShellExecute",
            detail: format!("{target}: {error}"),
        })?;
    Ok(())
}

/// Whether a process with this executable name is running.
///
/// Used to notice that the game is already open. Compared without case,
/// because Windows does.
pub fn is_running_by_name(name: &str) -> Result<bool> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    // SAFETY: the snapshot handle is closed before returning, and the
    // entry is sized as the API requires.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|error| {
            PlatformError::Api {
                api: "CreateToolhelp32Snapshot",
                detail: error.to_string(),
            }
        })?;
        let snapshot: HANDLE = snapshot;

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut found = false;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let length = entry
                    .szExeFile
                    .iter()
                    .position(|unit| *unit == 0)
                    .unwrap_or(entry.szExeFile.len());
                let current = String::from_utf16_lossy(&entry.szExeFile[..length]);
                if current.eq_ignore_ascii_case(name) {
                    found = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asking_whether_steam_runs_never_fails() {
        // Either answer is correct; what matters is that reading the
        // registry and opening the process cannot make this blow up.
        let _ = is_steam_running().expect("the check itself must not fail");
    }

    #[test]
    fn this_very_process_counts_as_alive() {
        assert!(process_is_alive(std::process::id()));
    }

    #[test]
    fn a_pid_that_cannot_exist_is_not_alive() {
        // 0 is the system idle process and is never a real target, and a
        // pid that is not a multiple of four is never valid on Windows.
        assert!(!process_is_alive(0xffff_fffe));
    }

    #[test]
    fn the_running_test_binary_is_found_by_name() {
        let exe = std::env::current_exe().unwrap();
        let name = exe.file_name().unwrap().to_string_lossy().into_owned();
        assert!(is_running_by_name(&name).unwrap(), "{name} should be found");
        assert!(!is_running_by_name("this-program-does-not-exist.exe").unwrap());
    }

    #[test]
    fn spawning_something_that_is_not_there_is_an_error() {
        let missing = std::env::temp_dir().join("recof-not-a-program.exe");
        assert!(spawn(&missing, &[], &std::env::temp_dir()).is_err());
    }
}
