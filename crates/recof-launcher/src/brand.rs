//! The look the launcher brings with it.
//!
//! Everything here is drawn: a gradient, a vignette, a grain texture
//! generated at start up, and a wordmark set in a font. None of it is a
//! picture of the game, and none of it imitates the game's own logo. What
//! the user puts in the theme folder sits on top of it.

use egui::{
    Align2, Color32, ColorImage, FontFamily, FontId, Rect, TextureHandle, TextureOptions, Ui, Vec2,
};

use crate::theme;

/// Archivo Black, under the SIL Open Font License 1.1. The license text
/// travels with it in `assets/ArchivoBlack-OFL.txt`, which is what the
/// licence asks for.
const DISPLAY_FONT: &[u8] = include_bytes!("../assets/ArchivoBlack-Regular.ttf");

/// The family name the display font is registered under.
pub const DISPLAY: &str = "display";

/// Side of the grain tile. Small enough to be cheap, large enough that
/// the repeat is not a pattern the eye picks up.
const GRAIN_SIZE: usize = 128;

/// Registers the display font alongside the built in ones.
///
/// Called after the fonts are otherwise set up, so the system font
/// fallback still applies to body text.
pub fn install_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        DISPLAY.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(DISPLAY_FONT)),
    );
    fonts
        .families
        .insert(FontFamily::Name(DISPLAY.into()), vec![DISPLAY.to_owned()]);
    ctx.set_fonts(fonts);
}

/// A font id for the display family.
pub fn display_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY.into()))
}

/// The grain overlay, generated once.
pub struct Grain {
    texture: TextureHandle,
}

impl Grain {
    /// Builds the tile.
    ///
    /// The noise is deterministic, from a small integer hash rather than
    /// a random source, so the same build always looks the same and the
    /// launcher needs no randomness to draw itself.
    pub fn new(ctx: &egui::Context) -> Self {
        let mut pixels = Vec::with_capacity(GRAIN_SIZE * GRAIN_SIZE);
        for y in 0..GRAIN_SIZE {
            for x in 0..GRAIN_SIZE {
                let value = hash(x as u32, y as u32);
                // A narrow band around mid grey: the overlay is drawn at a
                // low alpha, so a wide band would read as static.
                let level = 118 + (value % 20) as u8;
                pixels.push(Color32::from_rgba_premultiplied(level, level, level, 255));
            }
        }

        let image = ColorImage {
            size: [GRAIN_SIZE, GRAIN_SIZE],
            pixels,
            source_size: Vec2::new(GRAIN_SIZE as f32, GRAIN_SIZE as f32),
        };
        Grain {
            texture: ctx.load_texture("grain", image, TextureOptions::NEAREST_REPEAT),
        }
    }

    /// Lays the grain over a rectangle.
    pub fn paint(&self, ui: &Ui, rect: Rect, strength: f32) {
        if strength <= 0.0 {
            return;
        }
        let repeats = Vec2::new(
            rect.width() / GRAIN_SIZE as f32,
            rect.height() / GRAIN_SIZE as f32,
        );
        let uv = Rect::from_min_size(egui::Pos2::ZERO, repeats);
        ui.painter().image(
            self.texture.id(),
            rect,
            uv,
            Color32::from_white_alpha((strength.clamp(0.0, 1.0) * 26.0) as u8),
        );
    }
}

/// Integer hash, so the grain is the same on every machine and every run.
fn hash(x: u32, y: u32) -> u32 {
    let mut value = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263));
    value = (value ^ (value >> 13)).wrapping_mul(1_274_126_177);
    value ^ (value >> 16)
}

/// Paints the background of a screen.
///
/// `hero` is the user's picture when there is one. With it, the picture
/// is drawn to cover the area and dimmed so text stays readable; without
/// it, a gradient stands in. Either way the vignette and the grain go on
/// top, which is what ties the two cases together visually.
pub fn paint_background(
    ui: &Ui,
    rect: Rect,
    hero: Option<&TextureHandle>,
    dim: f32,
    grain: Option<&Grain>,
) {
    let painter = ui.painter();

    match hero {
        Some(texture) => {
            painter.rect_filled(rect, 0.0, theme::BACKGROUND);
            let uv = cover(texture.size_vec2(), rect.size());
            painter.image(texture.id(), rect, uv, Color32::WHITE);
            // The dim is what keeps a bright screenshot from swallowing
            // the interface drawn over it.
            painter.rect_filled(
                rect,
                0.0,
                Color32::from_black_alpha((dim.clamp(0.0, 1.0) * 255.0) as u8),
            );
        }
        None => paint_gradient(ui, rect),
    }

    paint_vignette(ui, rect);
    if let Some(grain) = grain {
        grain.paint(ui, rect, 1.0);
    }
}

/// Which part of a texture to show so it covers an area without
/// stretching, the way a CSS background cover does.
fn cover(texture: Vec2, area: Vec2) -> Rect {
    if texture.x <= 0.0 || texture.y <= 0.0 || area.x <= 0.0 || area.y <= 0.0 {
        return Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    }

    let texture_ratio = texture.x / texture.y;
    let area_ratio = area.x / area.y;

    if texture_ratio > area_ratio {
        // Wider than the area: trim the sides.
        let visible = area_ratio / texture_ratio;
        let margin = (1.0 - visible) / 2.0;
        Rect::from_min_max(egui::pos2(margin, 0.0), egui::pos2(1.0 - margin, 1.0))
    } else {
        // Taller: trim top and bottom, biased upward, because a hero's
        // subject is usually not at the very bottom.
        let visible = texture_ratio / area_ratio;
        let margin = 1.0 - visible;
        Rect::from_min_max(
            egui::pos2(0.0, margin * 0.35),
            egui::pos2(1.0, 1.0 - margin * 0.65),
        )
    }
}

/// A vertical gradient, dark at the top and a touch warmer at the bottom.
fn paint_gradient(ui: &Ui, rect: Rect) {
    const STEPS: usize = 48;
    let painter = ui.painter();
    let top = Color32::from_rgb(0x0a, 0x0a, 0x0c);
    let bottom = Color32::from_rgb(0x19, 0x11, 0x11);

    let height = rect.height() / STEPS as f32;
    for step in 0..STEPS {
        let t = step as f32 / (STEPS - 1) as f32;
        let band = Rect::from_min_size(
            egui::pos2(rect.left(), rect.top() + height * step as f32),
            // A hair taller than the step, so rounding never shows a seam.
            Vec2::new(rect.width(), height + 1.0),
        );
        painter.rect_filled(band, 0.0, mix(top, bottom, t));
    }
}

/// Darkens the edges, which is most of what makes a flat panel look lit.
fn paint_vignette(ui: &Ui, rect: Rect) {
    const STEPS: usize = 14;
    let painter = ui.painter();

    for step in 0..STEPS {
        let t = step as f32 / STEPS as f32;
        let inset = rect.width().min(rect.height()) * 0.5 * t;
        let band = rect.shrink(inset);
        if band.width() <= 0.0 || band.height() <= 0.0 {
            break;
        }
        // Each ring is barely visible; stacked they fall off smoothly.
        painter.rect_stroke(
            band,
            0.0,
            egui::Stroke::new(
                rect.width().min(rect.height()) * 0.5 / STEPS as f32 + 1.0,
                Color32::from_black_alpha(7),
            ),
            egui::StrokeKind::Inside,
        );
    }
}

fn mix(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    Color32::from_rgb(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
    )
}

/// What the wordmark says. The title names the game it recompiles, which
/// is what the launcher is for; the second line is what this project adds.
pub const TITLE: &str = "CRY OF FEAR";
pub const SUBTITLE: &str = "RECOMPILED";

/// Draws the wordmark, or the user's logo when there is one.
///
/// Returns the height it took, so the caller can lay out under it.
pub fn wordmark(ui: &mut Ui, logo: Option<&TextureHandle>, width: f32) -> f32 {
    if let Some(texture) = logo {
        // The user's logo is shown at its own shape, bounded so a large
        // file does not push the rest of the screen off.
        let size = texture.size_vec2();
        let scale = (width * 0.55 / size.x).min(120.0 / size.y).min(1.0);
        let shown = size * scale;
        let (rect, _) = ui.allocate_exact_size(shown, egui::Sense::hover());
        ui.painter().image(
            texture.id(),
            rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        return shown.y;
    }

    paint_wordmark(ui, width)
}

/// The built in wordmark: the title in the display font, the subtitle
/// under it in letter spaced capitals, and a rule between them.
fn paint_wordmark(ui: &mut Ui, width: f32) -> f32 {
    let title_size = (width * 0.062).clamp(26.0, 46.0);
    let subtitle_size = (title_size * 0.30).clamp(9.0, 14.0);
    let height = title_size * 1.25 + subtitle_size * 2.6;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
    let painter = ui.painter();

    // The title, with a shadow that lifts it off a busy hero.
    let baseline = egui::pos2(rect.left(), rect.top());
    painter.text(
        baseline + Vec2::new(1.0, 2.0),
        Align2::LEFT_TOP,
        TITLE,
        display_font(title_size),
        Color32::from_black_alpha(190),
    );
    let title = painter.text(
        baseline,
        Align2::LEFT_TOP,
        TITLE,
        display_font(title_size),
        theme::TEXT,
    );

    // A short rule in the accent, as wide as the word under it is long.
    let rule_y = title.bottom() + subtitle_size * 0.75;
    let subtitle = spaced(SUBTITLE);
    let subtitle_width = painter
        .layout_no_wrap(
            subtitle.clone(),
            display_font(subtitle_size),
            theme::ACCENT_HOVER,
        )
        .rect
        .width();

    painter.rect_filled(
        Rect::from_min_size(
            egui::pos2(rect.left(), rule_y),
            Vec2::new(subtitle_width.max(60.0), 2.0),
        ),
        0.0,
        theme::ACCENT,
    );

    painter.text(
        egui::pos2(rect.left(), rule_y + 6.0),
        Align2::LEFT_TOP,
        subtitle,
        display_font(subtitle_size),
        theme::TEXT_DIM,
    );

    height
}

/// Spreads a word out, which is the one typographic trick the subtitle
/// needs and the font cannot do on its own.
fn spaced(text: &str) -> String {
    text.chars()
        .map(|character| character.to_string())
        .collect::<Vec<_>>()
        .join("\u{2009}\u{2009}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_display_font_is_a_real_font_file() {
        // A truncated or wrong file would only show up as missing text at
        // run time, which is a bad place to find out.
        assert!(DISPLAY_FONT.len() > 20_000, "{}", DISPLAY_FONT.len());
        // Every TrueType file starts with one of these.
        let magic = &DISPLAY_FONT[..4];
        assert!(
            magic == [0x00, 0x01, 0x00, 0x00] || magic == b"true" || magic == b"OTTO",
            "{magic:?} is not a font signature"
        );
    }

    #[test]
    fn the_font_licence_travels_with_the_font() {
        let licence = include_str!("../assets/ArchivoBlack-OFL.txt");
        assert!(licence.contains("SIL OPEN FONT LICENSE"));
        assert!(licence.contains("Archivo Black"));
    }

    #[test]
    fn the_wordmark_says_what_it_should() {
        assert_eq!(TITLE, "CRY OF FEAR");
        assert_eq!(SUBTITLE, "RECOMPILED");
    }

    #[test]
    fn letter_spacing_keeps_every_letter() {
        let spread = spaced("ABC");
        assert!(spread.starts_with('A'));
        assert!(spread.ends_with('C'));
        assert_eq!(
            spread.chars().filter(|c| c.is_ascii_alphabetic()).count(),
            3
        );
    }

    #[test]
    fn the_grain_hash_is_the_same_everywhere_and_varies_per_pixel() {
        assert_eq!(hash(3, 7), hash(3, 7), "the grain must not move");
        let values: std::collections::HashSet<u32> =
            (0..40).map(|index| hash(index, index * 3) % 20).collect();
        assert!(values.len() > 5, "the grain is too uniform to be grain");
    }

    #[test]
    fn a_wide_picture_is_trimmed_at_the_sides() {
        // A 21 by 9 hero in a 4 by 3 window: the sides go.
        let uv = cover(Vec2::new(2100.0, 900.0), Vec2::new(800.0, 600.0));
        assert!(uv.left() > 0.0 && uv.right() < 1.0, "{uv:?}");
        assert!((uv.top() - 0.0).abs() < f32::EPSILON);
        assert!((uv.bottom() - 1.0).abs() < f32::EPSILON);
        // And it stays centred.
        assert!(((uv.left() + (1.0 - uv.right())) / 2.0 - uv.left()).abs() < 0.001);
    }

    #[test]
    fn a_tall_picture_is_trimmed_top_and_bottom() {
        let uv = cover(Vec2::new(600.0, 900.0), Vec2::new(960.0, 540.0));
        assert!(uv.top() > 0.0 && uv.bottom() < 1.0, "{uv:?}");
        assert!((uv.left() - 0.0).abs() < f32::EPSILON);
        assert!((uv.right() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_picture_the_shape_of_the_window_is_not_trimmed() {
        let uv = cover(Vec2::new(1920.0, 1080.0), Vec2::new(960.0, 540.0));
        assert!((uv.width() - 1.0).abs() < 0.001, "{uv:?}");
        assert!((uv.height() - 1.0).abs() < 0.001, "{uv:?}");
    }

    #[test]
    fn a_degenerate_size_does_not_divide_by_zero() {
        let uv = cover(Vec2::new(0.0, 0.0), Vec2::new(800.0, 600.0));
        assert_eq!(uv.width(), 1.0);
        let uv = cover(Vec2::new(100.0, 100.0), Vec2::new(0.0, 0.0));
        assert_eq!(uv.width(), 1.0);
    }

    #[test]
    fn mixing_lands_on_the_ends() {
        let from = Color32::from_rgb(0, 0, 0);
        let to = Color32::from_rgb(100, 200, 50);
        assert_eq!(mix(from, to, 0.0), from);
        assert_eq!(mix(from, to, 1.0), to);
        // And clamps rather than overshooting.
        assert_eq!(mix(from, to, 5.0), to);
        assert_eq!(mix(from, to, -5.0), from);
    }
}
