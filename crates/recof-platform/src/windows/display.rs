//! Display modes, from the driver rather than from a hardcoded list.
//!
//! Retail `CoFLaunchApp.exe` offers thirteen fixed resolutions, which is
//! why the game has no 2560 by 1440 option. This asks the display what it
//! can actually do.

use windows::Win32::Graphics::Gdi::{
    EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE,
};

use crate::{DisplayMode, Result};

/// Every distinct width and height the primary display reports, smallest
/// first. Refresh rates and colour depths are collapsed: the game is told
/// a resolution and nothing else.
pub fn display_modes() -> Result<Vec<DisplayMode>> {
    let mut modes = Vec::new();
    let mut index = 0u32;

    loop {
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

        // SAFETY: a null device name means the primary display, and
        // devmode is correctly sized as the API requires.
        let ok = unsafe {
            EnumDisplaySettingsW(
                None,
                ENUM_DISPLAY_SETTINGS_MODE(index),
                &mut devmode as *mut DEVMODEW,
            )
        };
        if !ok.as_bool() {
            break;
        }
        index += 1;

        // Interlaced and very small modes are not worth offering.
        if devmode.dmPelsWidth < 640 || devmode.dmPelsHeight < 480 {
            continue;
        }
        let mode = DisplayMode {
            width: devmode.dmPelsWidth,
            height: devmode.dmPelsHeight,
        };
        if !modes.contains(&mode) {
            modes.push(mode);
        }

        // A driver that never stops enumerating must not hang the launcher.
        if index > 4096 {
            break;
        }
    }

    modes.sort_by_key(|mode| (mode.width, mode.height));
    Ok(modes)
}

/// What the display is set to right now, for the default selection.
pub fn current_display_mode() -> Result<Option<DisplayMode>> {
    let mut devmode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };

    // SAFETY: as above.
    let ok =
        unsafe { EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut devmode as *mut DEVMODEW) };
    if !ok.as_bool() {
        return Ok(None);
    }
    Ok(Some(DisplayMode {
        width: devmode.dmPelsWidth,
        height: devmode.dmPelsHeight,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_display_reports_at_least_one_mode() {
        let modes = display_modes().expect("a display is attached");
        assert!(!modes.is_empty());
        assert!(modes.iter().all(|mode| mode.width >= 640));
        assert!(modes.iter().all(|mode| mode.height >= 480));
    }

    #[test]
    fn modes_are_unique_and_sorted() {
        let modes = display_modes().unwrap();
        let mut sorted = modes.clone();
        sorted.sort_by_key(|mode| (mode.width, mode.height));
        sorted.dedup();
        assert_eq!(modes, sorted);
    }

    #[test]
    fn the_current_mode_is_one_of_the_reported_ones() {
        let modes = display_modes().unwrap();
        if let Some(current) = current_display_mode().unwrap() {
            assert!(
                modes.contains(&current),
                "{current:?} is not in {} reported modes",
                modes.len()
            );
        }
    }
}
