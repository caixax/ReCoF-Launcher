//! Windows security features that can stop the game from starting, and
//! the Microsoft Defender exclusions that keep it from deleting ReCoF.

use std::os::windows::process::CommandExt;
use std::path::{Component, Path, PathBuf, Prefix};
use std::process::Stdio;
use std::time::{Duration, Instant};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, WIN32_ERROR};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
use windows::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
    SHELLEXECUTEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

use super::registry::{read_u32, RegRoot};
use crate::{PlatformError, Result};

/// Where Windows keeps the state of Smart App Control.
const CI_POLICY_KEY: &str = r"SYSTEM\CurrentControlSet\Control\CI\Policy";
const SAC_STATE_VALUE: &str = "VerifiedAndReputablePolicyState";
/// "On (enforcement)" in Microsoft's documentation; 0 is off and 2 is
/// evaluation, which blocks nothing.
const SAC_ENFORCING: u32 = 1;

/// Keeps a console window from flashing up for PowerShell.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// How long the Defender status may take before the answer counts as no.
/// The first call after boot loads the Defender module and can take a few
/// seconds; anything much longer is a machine where it is broken.
const STATUS_TIMEOUT: Duration = Duration::from_secs(20);

/// What the elevated PowerShell exits with when the exclusions are in.
const EXCLUDED: u32 = 0;

/// Whether Smart App Control is on and blocking code that is neither
/// signed nor known to Microsoft.
///
/// ReCoF's DLLs are new builds without a signature, so with this on Windows
/// refuses to load them and cof.exe stops with a "Bad Image" box
/// (status 0xc0e90002). A machine without the value (Windows 10, or
/// Windows 11 that never had it) answers false.
pub fn smart_app_control_enforcing() -> Result<bool> {
    Ok(read_u32(RegRoot::LocalMachine, CI_POLICY_KEY, SAC_STATE_VALUE)? == Some(SAC_ENFORCING))
}

/// Whether Microsoft Defender's real-time protection is on.
///
/// Asked through `Get-MpComputerStatus`, which any user may run. When
/// Defender is not the antivirus in charge (another one replaced it, or it
/// was turned off) the cmdlet fails and prints nothing, which reads as
/// false: there is nothing to add an exclusion to. An error is returned
/// only when PowerShell itself cannot be started.
pub fn defender_realtime_on() -> Result<bool> {
    let mut child = std::process::Command::new(powershell()?)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-MpComputerStatus).RealTimeProtectionEnabled",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|error| PlatformError::Api {
            api: "Get-MpComputerStatus",
            detail: format!("PowerShell could not start: {error}"),
        })?;

    // The answer is one word, far below what a pipe holds, so waiting
    // before reading cannot fill the pipe and stall the child.
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < STATUS_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                log::warn!("Get-MpComputerStatus did not answer in time, taking it as off");
                return Ok(false);
            }
            Err(error) => {
                return Err(PlatformError::Api {
                    api: "Get-MpComputerStatus",
                    detail: error.to_string(),
                })
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| PlatformError::Api {
            api: "Get-MpComputerStatus",
            detail: error.to_string(),
        })?;
    let answer = String::from_utf8_lossy(&output.stdout);
    let on = answer.trim().eq_ignore_ascii_case("true");
    if !on {
        let reason = String::from_utf8_lossy(&output.stderr);
        let reason = reason.lines().find(|line| !line.trim().is_empty());
        log::info!(
            "Defender real-time protection reads as off ({})",
            reason.map(str::trim).unwrap_or(answer.trim())
        );
    }
    Ok(on)
}

/// Whether a path is narrow enough to hand to Defender as an exclusion.
///
/// Only an absolute path on a drive or a network share, at least two
/// folders below its root, so neither a drive nor a top level folder such
/// as `C:\Games` or `D:\SteamLibrary` can ever be excluded. The one top
/// level folder accepted is one named after ReCoF, such as `H:\ReCoF`,
/// which is a copy (or a launcher folder) and nothing else. Characters
/// Defender treats as wildcards or as environment variables are refused,
/// because they would make the exclusion cover more than the folder named.
pub fn defender_exclusion_allowed(path: &Path) -> bool {
    let Some(text) = path.to_str() else {
        return false;
    };
    if text.contains(['*', '?', '%', '"', '\0', '\r', '\n']) {
        return false;
    }

    let mut prefix = false;
    let mut root = false;
    let mut below_root = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(part) => {
                if !matches!(part.kind(), Prefix::Disk(_) | Prefix::UNC(_, _)) {
                    return false;
                }
                prefix = true;
            }
            Component::RootDir => root = true,
            Component::Normal(name) => below_root.push(name),
            Component::CurDir | Component::ParentDir => return false,
        }
    }
    let narrow = match below_root.as_slice() {
        [] => false,
        [only] => only.to_string_lossy().to_lowercase().contains("recof"),
        _ => true,
    };
    prefix && root && narrow
}

/// Adds folders to Microsoft Defender's exclusions, behind one UAC prompt.
///
/// Only paths: never a process or an extension. Every path is checked
/// with [`defender_exclusion_allowed`] first and the whole call is refused
/// if one fails. Cancelling the prompt is [`PlatformError::Declined`].
pub fn add_defender_exclusions(paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    if let Some(path) = paths.iter().find(|path| !defender_exclusion_allowed(path)) {
        return Err(PlatformError::NotAllowed(path.display().to_string()));
    }

    let script = exclusion_script(paths);
    // Encoded, so no quoting rule of the command line can change what
    // PowerShell runs: the script travels as UTF-16 in base64.
    let parameters = format!(
        "-NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand {}",
        encode_command(&script)
    );
    log::info!("asking for administrator rights to run: {script}");

    match run_elevated(&powershell()?, &parameters)? {
        EXCLUDED => Ok(()),
        code => Err(PlatformError::Api {
            api: "Add-MpPreference",
            detail: format!("PowerShell exited with code {code}"),
        }),
    }
}

/// The PowerShell that adds the exclusions and says how it went.
fn exclusion_script(paths: &[PathBuf]) -> String {
    let list: Vec<String> = paths
        .iter()
        .map(|path| single_quoted(&path.to_string_lossy()))
        .collect();
    format!(
        "try {{ Add-MpPreference -ExclusionPath {} -ErrorAction Stop; exit 0 }} catch {{ exit 2 }}",
        list.join(",")
    )
}

/// A PowerShell string literal that is taken as written.
///
/// Inside single quotes nothing is expanded, and a quote is written twice.
/// PowerShell also accepts the typographic single quotes as quotes, so
/// those are doubled as well.
fn single_quoted(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('\'');
    for character in text.chars() {
        if matches!(
            character,
            '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}'
        ) {
            quoted.push(character);
        }
        quoted.push(character);
    }
    quoted.push('\'');
    quoted
}

/// What `-EncodedCommand` takes: the script as UTF-16LE, in base64.
fn encode_command(script: &str) -> String {
    use base64::Engine;

    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// The system's own PowerShell, by full path, so a `powershell.exe`
/// earlier in `PATH` can never be the one that gets administrator rights.
fn powershell() -> Result<PathBuf> {
    let mut buffer = [0u16; 260];
    // SAFETY: the buffer is writable and its length is passed with it.
    let length = unsafe { GetSystemDirectoryW(Some(&mut buffer)) } as usize;
    if length == 0 || length > buffer.len() {
        return Err(PlatformError::Api {
            api: "GetSystemDirectory",
            detail: std::io::Error::last_os_error().to_string(),
        });
    }
    let system = PathBuf::from(String::from_utf16_lossy(&buffer[..length]));
    Ok(system
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe"))
}

/// Starts a program with administrator rights, hidden, waits for it and
/// returns its exit code.
fn run_elevated(program: &Path, parameters: &str) -> Result<u32> {
    let verb = wide("runas");
    let file = wide(&program.to_string_lossy());
    let parameters = wide(parameters);

    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };

    // SAFETY: every string handed over is a NUL terminated buffer that
    // outlives the call, COM is set up and torn down on this thread around
    // it, and the process handle is closed on every path out.
    unsafe {
        // The shell may hand the request to an extension, which is why
        // Microsoft asks for COM to be up. S_FALSE means it already was.
        let initialised = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        let started = ShellExecuteExW(&mut info);
        if initialised.is_ok() {
            CoUninitialize();
        }

        if let Err(error) = started {
            if WIN32_ERROR::from_error(&error) == Some(ERROR_CANCELLED) {
                return Err(PlatformError::Declined);
            }
            return Err(PlatformError::Api {
                api: "ShellExecuteEx",
                detail: error.to_string(),
            });
        }
        if info.hProcess.is_invalid() {
            return Err(PlatformError::Api {
                api: "ShellExecuteEx",
                detail: "no process handle came back".to_owned(),
            });
        }

        WaitForSingleObject(info.hProcess, INFINITE);
        let mut code = 0u32;
        let read = GetExitCodeProcess(info.hProcess, &mut code);
        let _ = CloseHandle(info.hProcess);
        read.map_err(|error| PlatformError::Api {
            api: "GetExitCodeProcess",
            detail: error.to_string(),
        })?;
        Ok(code)
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asking_about_smart_app_control_never_fails() {
        // Windows 10 and machines that never had it have no value, which
        // reads as off rather than as an error.
        assert!(smart_app_control_enforcing().is_ok());
    }

    #[test]
    fn asking_about_defender_never_fails() {
        // Either answer is right; a machine with another antivirus says
        // no. Only reading, nothing is changed.
        assert!(defender_realtime_on().is_ok());
    }

    #[test]
    fn the_system_powershell_is_found_by_full_path() {
        let path = powershell().unwrap();
        assert!(path.is_absolute());
        assert!(path.is_file(), "{} should exist", path.display());
    }

    #[test]
    fn drives_and_top_level_folders_are_never_excluded() {
        for refused in [
            r"C:\",
            r"C:",
            r"C:\Games",
            r"C:\Users",
            r"D:\SteamLibrary",
            r"\\server\share",
            r"\\server\share\folder",
            r"Games\ReCoF",
            r"\Games\ReCoF",
            r"C:\Games\..\ReCoF",
            r"\\?\C:\Games\ReCoF",
            r"C:\Games\*",
            r"C:\Games\ReCoF?",
            r"C:\%USERPROFILE%\ReCoF",
            r"C:\Program Files",
            r"C:\Windows",
        ] {
            assert!(
                !defender_exclusion_allowed(Path::new(refused)),
                "{refused} should be refused"
            );
        }
    }

    #[test]
    fn folders_two_levels_down_or_named_after_recof_are_accepted() {
        for accepted in [
            r"H:\ReCoF",
            r"H:\ReCoF\",
            r"C:\ReCoF Launcher",
            r"\\server\share\ReCoF",
            r"C:\Games\ReCoF",
            r"C:\Users\x\AppData\Local\ReCoF Launcher",
            r"D:\Juegos\Librería\ReCoF",
            r"C:\Users\x\Downloads\ReCoF-Launcher.exe",
            r"\\server\share\games\ReCoF",
            r"C:\Games\O'Brien\ReCoF",
        ] {
            assert!(
                defender_exclusion_allowed(Path::new(accepted)),
                "{accepted} should be accepted"
            );
        }
    }

    #[test]
    fn a_refused_path_stops_the_call_before_any_prompt() {
        let error =
            add_defender_exclusions(&[PathBuf::from(r"C:\Games\ReCoF"), PathBuf::from(r"C:\")])
                .unwrap_err();
        assert!(matches!(error, PlatformError::NotAllowed(_)), "{error}");
    }

    #[test]
    fn nothing_to_add_asks_for_nothing() {
        assert!(add_defender_exclusions(&[]).is_ok());
    }

    #[test]
    fn quotes_are_doubled_so_a_path_cannot_end_the_string() {
        assert_eq!(single_quoted(r"C:\Games\ReCoF"), r"'C:\Games\ReCoF'");
        assert_eq!(single_quoted(r"C:\O'Brien\x"), r"'C:\O''Brien\x'");
        assert_eq!(
            single_quoted("C:\\a\u{2019}b\\c"),
            "'C:\\a\u{2019}\u{2019}b\\c'"
        );
        // Nothing else is touched: inside single quotes these are literal.
        assert_eq!(single_quoted(r"C:\$x`y\z"), r"'C:\$x`y\z'");
    }

    #[test]
    fn the_script_adds_paths_and_nothing_else() {
        let script = exclusion_script(&[
            PathBuf::from(r"C:\Users\x\AppData\Local\ReCoF Launcher"),
            PathBuf::from(r"D:\Games\ReCoF"),
        ]);
        assert_eq!(
            script,
            "try { Add-MpPreference -ExclusionPath 'C:\\Users\\x\\AppData\\Local\\ReCoF Launcher','D:\\Games\\ReCoF' -ErrorAction Stop; exit 0 } catch { exit 2 }"
        );
        assert!(!script.contains("ExclusionProcess"));
        assert!(!script.contains("ExclusionExtension"));
    }

    #[test]
    fn the_command_is_encoded_the_way_powershell_reads_it() {
        // [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes("exit 0"))
        assert_eq!(encode_command("exit 0"), "ZQB4AGkAdAAgADAA");
    }
}
