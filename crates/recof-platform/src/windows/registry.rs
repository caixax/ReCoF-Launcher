//! Reading the registry.
//!
//! Read only on purpose: nothing in this launcher writes to the registry.
//! The game writes its own video settings to
//! `HKCU\Software\Valve\Cry of Fear\Settings`; launch options live in the
//! launcher's own settings file instead.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    KEY_READ, REG_DWORD, REG_EXPAND_SZ, REG_SZ, REG_VALUE_TYPE,
};

use crate::{PlatformError, Result};

/// The two hives the launcher ever looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegRoot {
    CurrentUser,
    LocalMachine,
}

impl RegRoot {
    fn hkey(self) -> HKEY {
        match self {
            RegRoot::CurrentUser => HKEY_CURRENT_USER,
            RegRoot::LocalMachine => HKEY_LOCAL_MACHINE,
        }
    }

    fn label(self) -> &'static str {
        match self {
            RegRoot::CurrentUser => "HKCU",
            RegRoot::LocalMachine => "HKLM",
        }
    }
}

/// Reads a `REG_SZ` or `REG_EXPAND_SZ` value.
///
/// A missing key or a missing value is `Ok(None)`, because "Steam is not
/// installed" is an answer and not a failure. Anything else is an error.
pub fn read_string(root: RegRoot, subkey: &str, value: &str) -> Result<Option<String>> {
    let Some(key) = open(root, subkey)? else {
        return Ok(None);
    };
    match key.query(root, subkey, value, &[REG_SZ, REG_EXPAND_SZ])? {
        Some((_, bytes)) => Ok(Some(utf16_from_bytes(&bytes))),
        None => Ok(None),
    }
}

/// Reads a `REG_DWORD` value.
pub fn read_u32(root: RegRoot, subkey: &str, value: &str) -> Result<Option<u32>> {
    let Some(key) = open(root, subkey)? else {
        return Ok(None);
    };
    match key.query(root, subkey, value, &[REG_DWORD])? {
        Some((_, bytes)) if bytes.len() >= 4 => Ok(Some(u32::from_ne_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
        ]))),
        Some(_) => Err(PlatformError::Registry {
            operation: "RegQueryValueExW",
            key: format!("{}\\{subkey}\\{value}", root.label()),
            detail: "a REG_DWORD value shorter than four bytes".to_owned(),
        }),
        None => Ok(None),
    }
}

/// An open registry key that closes itself.
struct OwnedKey(HKEY);

impl Drop for OwnedKey {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful RegOpenKeyExW and is
        // closed exactly once.
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

impl OwnedKey {
    /// Reads one value, checking that it has one of the expected types.
    fn query(
        &self,
        root: RegRoot,
        subkey: &str,
        value: &str,
        expected: &[REG_VALUE_TYPE],
    ) -> Result<Option<(REG_VALUE_TYPE, Vec<u8>)>> {
        let name = wide(value);
        let mut kind = REG_VALUE_TYPE(0);
        let mut size: u32 = 0;

        // SAFETY: a null data pointer with a valid size pointer asks for the
        // size only, which is the documented way to size the buffer.
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut kind),
                None,
                Some(&mut size),
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(self.error(root, subkey, value, format!("error {}", status.0)));
        }
        if !expected.contains(&kind) {
            let detail = format!("unexpected value type {}", kind.0);
            return Err(self.error(root, subkey, value, detail));
        }

        let mut buffer = vec![0u8; size as usize];
        let mut read = size;
        // SAFETY: the buffer is exactly the size the call above reported.
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut kind),
                Some(buffer.as_mut_ptr()),
                Some(&mut read),
            )
        };
        if status != ERROR_SUCCESS {
            let detail = format!("error {} on the second call", status.0);
            return Err(self.error(root, subkey, value, detail));
        }
        buffer.truncate(read as usize);
        Ok(Some((kind, buffer)))
    }

    fn error(&self, root: RegRoot, subkey: &str, value: &str, detail: String) -> PlatformError {
        PlatformError::Registry {
            operation: "RegQueryValueExW",
            key: format!("{}\\{subkey}\\{value}", root.label()),
            detail,
        }
    }
}

fn open(root: RegRoot, subkey: &str) -> Result<Option<OwnedKey>> {
    let path = wide(subkey);
    let mut handle = HKEY(std::ptr::null_mut());

    // SAFETY: `path` is a NUL terminated wide string that outlives the call,
    // and `handle` is a valid out pointer.
    let status = unsafe {
        RegOpenKeyExW(
            root.hkey(),
            PCWSTR(path.as_ptr()),
            None,
            KEY_READ,
            &mut handle,
        )
    };

    if status == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if status != ERROR_SUCCESS {
        return Err(PlatformError::Registry {
            operation: "RegOpenKeyExW",
            key: format!("{}\\{subkey}", root.label()),
            detail: format!("error {}", status.0),
        });
    }

    Ok(Some(OwnedKey(handle)))
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Turns the raw bytes of a `REG_SZ` into a string, dropping the trailing
/// NUL the registry stores and anything after it.
fn utf16_from_bytes(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_ne_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    OsString::from_wide(&units).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(text: &str, with_nul: bool) -> Vec<u8> {
        let mut units: Vec<u16> = text.encode_utf16().collect();
        if with_nul {
            units.push(0);
        }
        units.iter().flat_map(|unit| unit.to_ne_bytes()).collect()
    }

    #[test]
    fn strips_the_trailing_nul() {
        assert_eq!(utf16_from_bytes(&encode("c:/steam", true)), "c:/steam");
    }

    #[test]
    fn tolerates_a_value_without_a_nul() {
        assert_eq!(utf16_from_bytes(&encode("c:/steam", false)), "c:/steam");
    }

    #[test]
    fn keeps_non_ascii_paths() {
        let path = "D:/Juegos/Librería Steam/游戏";
        assert_eq!(utf16_from_bytes(&encode(path, true)), path);
    }

    #[test]
    fn an_odd_trailing_byte_is_ignored() {
        let mut bytes = encode("ab", true);
        bytes.push(0x41);
        assert_eq!(utf16_from_bytes(&bytes), "ab");
    }

    /// Reads the real hive. This key exists on every Windows install, so it
    /// is a safe smoke test of the whole path.
    #[test]
    fn reads_a_well_known_value() {
        let value = read_string(
            RegRoot::LocalMachine,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "CurrentVersion",
        )
        .expect("the key exists on every Windows install");
        assert!(value.is_some());
    }

    #[test]
    fn a_missing_key_is_none_not_an_error() {
        let value = read_string(
            RegRoot::CurrentUser,
            r"Software\ReCoF Launcher\this key does not exist",
            "whatever",
        );
        assert!(matches!(value, Ok(None)));
    }

    #[test]
    fn a_missing_value_in_an_existing_key_is_none() {
        let value = read_string(
            RegRoot::LocalMachine,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "ThisValueDoesNotExist",
        );
        assert!(matches!(value, Ok(None)));
    }
}
