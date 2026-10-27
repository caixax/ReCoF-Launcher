//! Artwork, and where it comes from.
//!
//! The launcher ships no picture of the game. A release carries no retail
//! asset, and that includes art: rule 3. What it ships instead is a
//! wordmark drawn from an open licensed font and a background painted in
//! code, both original.
//!
//! Anyone who wants the real thing puts it in
//! `%LOCALAPPDATA%\ReCoF Launcher\theme`:
//!
//! | File | What it replaces |
//! |---|---|
//! | `hero.png` or `hero.jpg` | the painted background |
//! | `logo.png` | the wordmark |
//! | `icon.png` | the window icon |
//!
//! Those files stay on the machine that put them there. Nothing uploads
//! them, nothing packs them into a release, and the folder is outside the
//! repository, so artwork somebody else owns never ends up in it.

use std::path::{Path, PathBuf};

use egui::{ColorImage, TextureHandle, TextureOptions};

/// Largest image the launcher will decode. A 4K hero is about 25 MB
/// decoded; this leaves room for that and stops a crafted file from
/// eating the memory of a launcher that is about to copy 4 GB.
const MAX_PIXELS: u32 = 8192;

/// Which picture a file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    /// The background of the main screen.
    Hero,
    /// The title, drawn over it.
    Logo,
    /// The window icon.
    Icon,
}

impl Piece {
    /// The names tried, in order. The first one found wins.
    pub fn file_names(self) -> &'static [&'static str] {
        match self {
            Piece::Hero => &["hero.png", "hero.jpg", "hero.jpeg", "background.png"],
            Piece::Logo => &["logo.png", "logo.jpg"],
            Piece::Icon => &["icon.png", "icon.jpg"],
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Piece::Hero => "hero",
            Piece::Logo => "logo",
            Piece::Icon => "icon",
        }
    }
}

/// The user's artwork, loaded once and kept.
#[derive(Default)]
pub struct Artwork {
    hero: Option<TextureHandle>,
    logo: Option<TextureHandle>,
    /// Set once the folder has been looked at, so a missing folder is not
    /// searched again on every frame.
    loaded: bool,
}

impl Artwork {
    /// Looks in the theme folder and loads whatever is there.
    ///
    /// Called once at start up and again when the user presses reload, so
    /// dropping a file in the folder does not need a restart.
    pub fn load(&mut self, ctx: &egui::Context, theme_dir: &Path) {
        self.hero = load_piece(ctx, theme_dir, Piece::Hero);
        self.logo = load_piece(ctx, theme_dir, Piece::Logo);
        self.loaded = true;
    }

    /// Drops what is loaded, so the next `load` reads from disk again.
    pub fn forget(&mut self) {
        self.hero = None;
        self.logo = None;
        self.loaded = false;
    }

    pub fn hero(&self) -> Option<&TextureHandle> {
        self.hero.as_ref()
    }

    pub fn logo(&self) -> Option<&TextureHandle> {
        self.logo.as_ref()
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

fn load_piece(ctx: &egui::Context, theme_dir: &Path, piece: Piece) -> Option<TextureHandle> {
    let path = find(theme_dir, piece)?;
    match decode(&path) {
        Ok(image) => {
            log::info!(
                "using {} for the {}, {} by {}",
                path.display(),
                piece.as_str(),
                image.width(),
                image.height()
            );
            Some(ctx.load_texture(piece.as_str(), image, TextureOptions::LINEAR))
        }
        Err(error) => {
            // A picture that will not decode is the user's problem to fix
            // and never the launcher's problem to crash on.
            log::warn!("{} could not be read: {error}", path.display());
            None
        }
    }
}

/// The first file of a piece that is actually there.
pub fn find(theme_dir: &Path, piece: Piece) -> Option<PathBuf> {
    piece
        .file_names()
        .iter()
        .map(|name| theme_dir.join(name))
        .find(|path| path.is_file())
}

/// Decodes a picture into something egui can upload.
fn decode(path: &Path) -> Result<ColorImage, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let decoded = image::load_from_memory(&bytes).map_err(|error| error.to_string())?;

    let (width, height) = (decoded.width(), decoded.height());
    if width == 0 || height == 0 {
        return Err("the image has no pixels".to_owned());
    }
    if width > MAX_PIXELS || height > MAX_PIXELS {
        return Err(format!(
            "the image is {width} by {height}, larger than the {MAX_PIXELS} pixel limit"
        ));
    }

    let rgba = decoded.to_rgba8();
    Ok(ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        rgba.as_raw(),
    ))
}

/// Reads `icon.png` for the window icon, before any context exists.
///
/// Returned in the shape `eframe` wants: raw RGBA plus the size.
pub fn window_icon(theme_dir: &Path) -> Option<egui::IconData> {
    let path = find(theme_dir, Piece::Icon)?;
    let bytes = std::fs::read(&path).ok()?;
    let decoded = image::load_from_memory(&bytes).ok()?;
    if decoded.width() > 1024 || decoded.height() > 1024 {
        log::warn!("{} is too large for a window icon", path.display());
        return None;
    }

    let rgba = decoded.to_rgba8();
    log::info!("using {} as the window icon", path.display());
    Some(egui::IconData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

/// Creates the theme folder and writes the note explaining what goes in
/// it, so the folder is never an empty mystery.
pub fn prepare(theme_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(theme_dir)?;
    let readme = theme_dir.join("README.txt");
    if readme.exists() {
        return Ok(());
    }
    std::fs::write(
        readme,
        "\
ReCoF Launcher, artwork
=======================

Drop pictures here and the launcher uses them instead of what it draws
itself. Press reload in Settings, or restart it.

  hero.png    the background of the main screen (hero.jpg also works)
  logo.png    the title drawn over it
  icon.png    the window icon

Nothing here is uploaded anywhere, and nothing here goes into a release.
The launcher ships no artwork of the game on purpose: whatever you put
in this folder stays on this machine.

SteamGridDB is where most people get theirs.
",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-art-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// A tiny valid PNG, encoded here so the test needs no file on disk.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut buffer = std::io::Cursor::new(Vec::new());
        let image = image::RgbaImage::from_pixel(width, height, image::Rgba([40, 10, 10, 255]));
        image
            .write_to(&mut buffer, image::ImageFormat::Png)
            .unwrap();
        buffer.into_inner()
    }

    #[test]
    fn each_piece_looks_for_its_own_names() {
        assert!(Piece::Hero.file_names().contains(&"hero.png"));
        assert!(Piece::Hero.file_names().contains(&"hero.jpg"));
        assert!(Piece::Logo.file_names().contains(&"logo.png"));
        assert!(Piece::Icon.file_names().contains(&"icon.png"));
        // No piece claims a name another one uses.
        for (left, right) in [
            (Piece::Hero, Piece::Logo),
            (Piece::Hero, Piece::Icon),
            (Piece::Logo, Piece::Icon),
        ] {
            for name in left.file_names() {
                assert!(!right.file_names().contains(name), "{name}");
            }
        }
    }

    #[test]
    fn the_first_name_that_exists_is_the_one_used() {
        let dir = scratch("first-wins");
        std::fs::write(dir.join("hero.jpg"), png(4, 4)).unwrap();
        assert_eq!(find(&dir, Piece::Hero), Some(dir.join("hero.jpg")));

        // png comes first in the list, so adding it takes over.
        std::fs::write(dir.join("hero.png"), png(4, 4)).unwrap();
        assert_eq!(find(&dir, Piece::Hero), Some(dir.join("hero.png")));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_empty_folder_offers_nothing() {
        let dir = scratch("empty");
        assert_eq!(find(&dir, Piece::Hero), None);
        assert_eq!(find(&dir, Piece::Logo), None);
        assert_eq!(window_icon(&dir), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_folder_that_is_not_there_is_not_an_error() {
        let missing = std::env::temp_dir().join("recof-art-never-created");
        let _ = std::fs::remove_dir_all(&missing);
        assert_eq!(find(&missing, Piece::Hero), None);
        assert_eq!(window_icon(&missing), None);
    }

    #[test]
    fn a_real_picture_decodes() {
        let dir = scratch("decode");
        let path = dir.join("hero.png");
        std::fs::write(&path, png(32, 16)).unwrap();

        let image = decode(&path).expect("a valid png decodes");
        assert_eq!(image.size, [32, 16]);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_that_is_not_a_picture_is_refused_rather_than_fatal() {
        let dir = scratch("not-a-picture");
        let path = dir.join("logo.png");
        std::fs::write(&path, b"this is not a png at all").unwrap();

        assert!(decode(&path).is_err());
        // And it is still found, so the warning names the right file.
        assert_eq!(find(&dir, Piece::Logo), Some(path));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_oversized_icon_is_refused() {
        let dir = scratch("huge-icon");
        std::fs::write(dir.join("icon.png"), png(2048, 2048)).unwrap();
        assert_eq!(window_icon(&dir), None, "a 2048 pixel icon is not an icon");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_real_icon_loads_with_its_size() {
        let dir = scratch("icon");
        std::fs::write(dir.join("icon.png"), png(64, 64)).unwrap();

        let icon = window_icon(&dir).expect("a 64 pixel icon loads");
        assert_eq!((icon.width, icon.height), (64, 64));
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn preparing_the_folder_explains_what_goes_in_it() {
        let dir = scratch("prepare");
        let theme = dir.join("theme");
        prepare(&theme).unwrap();

        let note = std::fs::read_to_string(theme.join("README.txt")).unwrap();
        assert!(note.contains("hero.png"));
        assert!(note.contains("logo.png"));
        assert!(note.contains("icon.png"));
        assert!(note.contains("stays on this machine"));

        // Running it again keeps whatever the user wrote there.
        std::fs::write(theme.join("README.txt"), b"mis notas").unwrap();
        prepare(&theme).unwrap();
        assert_eq!(
            std::fs::read_to_string(theme.join("README.txt")).unwrap(),
            "mis notas"
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn artwork_starts_empty_and_forgets_cleanly() {
        let mut artwork = Artwork::default();
        assert!(!artwork.is_loaded());
        assert!(artwork.hero().is_none());
        assert!(artwork.logo().is_none());

        artwork.forget();
        assert!(!artwork.is_loaded());
    }
}
