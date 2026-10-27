//! Artwork the repository carries, if it carries any.
//!
//! `build.rs` looks in `assets/artwork` and sets a cfg flag for each
//! picture it finds, so this compiles to `None` when the folder is empty
//! and the launcher falls back to what it draws. Nothing here is a
//! picture of the game unless somebody put one in that folder and
//! committed it, which is their decision and not the code's.
//!
//! What wins, in order: the user's theme folder, then this, then the
//! launcher's own drawing.

use egui::{ColorImage, TextureHandle, TextureOptions};

#[cfg(bundled_hero)]
const HERO: Option<&[u8]> = Some(include_bytes!("../assets/artwork/hero.png"));
#[cfg(not(bundled_hero))]
const HERO: Option<&[u8]> = None;

#[cfg(bundled_logo)]
const LOGO: Option<&[u8]> = Some(include_bytes!("../assets/artwork/logo.png"));
#[cfg(not(bundled_logo))]
const LOGO: Option<&[u8]> = None;

#[cfg(bundled_icon)]
const ICON: Option<&[u8]> = Some(include_bytes!("../assets/artwork/icon.png"));
#[cfg(not(bundled_icon))]
const ICON: Option<&[u8]> = None;

/// The credits the repository shipped alongside them.
///
/// Empty when there are none, which is the same as saying nobody put a
/// picture in the folder.
pub const CREDITS: &str = include_str!("../assets/artwork/CREDITS.txt");

/// Whether this build carries any artwork at all.
///
/// When it does, the launcher does not go looking for any: somebody
/// already decided what this build should show.
pub fn any() -> bool {
    HERO.is_some() || LOGO.is_some() || ICON.is_some()
}

/// The credits, when there are some worth showing.
pub fn credits() -> Option<String> {
    let trimmed = CREDITS.trim();
    if trimmed.is_empty() || !any() {
        return None;
    }
    // The file is allowed to carry a note to whoever edits it, which is
    // not something to put on screen.
    let shown: Vec<&str> = trimmed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    if shown.is_empty() {
        None
    } else {
        Some(shown.join("\n"))
    }
}

/// The bundled background, decoded.
pub fn hero(ctx: &egui::Context) -> Option<TextureHandle> {
    texture(ctx, "bundled-hero", HERO?)
}

/// The bundled wordmark, decoded.
pub fn logo(ctx: &egui::Context) -> Option<TextureHandle> {
    texture(ctx, "bundled-logo", LOGO?)
}

/// The bundled window icon, in the shape `eframe` wants it.
///
/// Read before any context exists, which is why it does not go through
/// the texture path.
pub fn icon() -> Option<egui::IconData> {
    let bytes = ICON?;
    let decoded = image::load_from_memory(bytes)
        .map_err(|error| log::warn!("the bundled icon could not be decoded: {error}"))
        .ok()?;
    if decoded.width() > 1024 || decoded.height() > 1024 {
        log::warn!("the bundled icon is larger than an icon should be");
        return None;
    }

    let rgba = decoded.to_rgba8();
    Some(egui::IconData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

fn texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let decoded = image::load_from_memory(bytes)
        .map_err(|error| log::warn!("{name} could not be decoded: {error}"))
        .ok()?;
    let rgba = decoded.to_rgba8();
    let image = ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    );
    log::info!(
        "using the bundled {name}, {} by {}",
        rgba.width(),
        rgba.height()
    );
    Some(ctx.load_texture(name, image, TextureOptions::LINEAR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_slot_carries_nothing_and_says_so() {
        // The repository ships the folder empty. If this ever fails, a
        // picture was committed, which is a decision somebody made on
        // purpose and not a bug.
        if !any() {
            assert!(HERO.is_none());
            assert!(LOGO.is_none());
            assert!(ICON.is_none());
            assert_eq!(credits(), None, "no artwork, so nothing to credit");
            assert_eq!(icon(), None);
        }
    }

    #[test]
    fn artwork_without_credits_is_a_gap_worth_noticing() {
        // Not an assertion about what should be bundled, but about the
        // pair: if somebody adds a picture they should add a line saying
        // who made it, and the launcher shows whatever they wrote.
        if any() {
            assert!(
                credits().is_some(),
                "assets/artwork holds a picture but CREDITS.txt names nobody"
            );
        }
    }

    #[test]
    fn comments_and_blank_lines_never_reach_the_screen() {
        let shown = CREDITS
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .count();
        // Whatever the file holds, what is shown is only the real lines.
        match credits() {
            Some(text) => assert_eq!(text.lines().count(), shown),
            None => assert!(shown == 0 || !any()),
        }
    }
}
