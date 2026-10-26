//! A dark theme with a restrained horror feel.
//!
//! Nothing here comes from the retail game: no art, no colours sampled
//! from it, no fonts taken out of it. It is a plain dark palette with a
//! dried blood accent, meant to stay out of the way.

use egui::{Color32, CornerRadius, Stroke, Visuals};

/// Near black, the window background.
pub const BACKGROUND: Color32 = Color32::from_rgb(0x0d, 0x0d, 0x0f);

/// Panels and cards, a shade above the background.
pub const SURFACE: Color32 = Color32::from_rgb(0x16, 0x16, 0x19);

/// Raised surfaces: hovered rows, inputs.
pub const SURFACE_RAISED: Color32 = Color32::from_rgb(0x1f, 0x1f, 0x23);

/// Hairlines and separators.
pub const BORDER: Color32 = Color32::from_rgb(0x2b, 0x2b, 0x30);

/// Body text.
pub const TEXT: Color32 = Color32::from_rgb(0xd2, 0xd0, 0xcc);

/// Captions, help text, anything secondary.
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8a, 0x87, 0x82);

/// The accent: dried blood rather than bright red, so a warning can still
/// be louder than a heading.
pub const ACCENT: Color32 = Color32::from_rgb(0x8f, 0x23, 0x24);

/// The accent when a control is hovered.
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xa8, 0x2c, 0x2d);

/// Something went well.
pub const GOOD: Color32 = Color32::from_rgb(0x6f, 0x9e, 0x5f);

/// Something is worth reading but is not a failure.
pub const WARN: Color32 = Color32::from_rgb(0xc8, 0x9c, 0x3c);

/// Something failed.
pub const BAD: Color32 = Color32::from_rgb(0xc4, 0x4a, 0x3f);

/// Applies the theme to a context. Called once at start up.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = Color32::from_rgb(0x08, 0x08, 0x0a);
    visuals.faint_bg_color = SURFACE;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = ACCENT_HOVER;

    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(4);
    visuals.menu_corner_radius = CornerRadius::same(4);

    let radius = CornerRadius::same(3);
    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    visuals.widgets.noninteractive.corner_radius = radius;

    visuals.widgets.inactive.bg_fill = SURFACE_RAISED;
    visuals.widgets.inactive.weak_bg_fill = SURFACE_RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.corner_radius = radius;

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x2a, 0x2a, 0x2f);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x2a, 0x2a, 0x2f);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.corner_radius = radius;

    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_HOVER);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.corner_radius = radius;

    visuals.widgets.open.bg_fill = SURFACE_RAISED;
    visuals.widgets.open.weak_bg_fill = SURFACE_RAISED;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.open.corner_radius = radius;

    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.55);
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);

    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.window_margin = egui::Margin::same(16);
        style.spacing.interact_size.y = 26.0;
    });
}

/// A heading, in the accent colour.
pub fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).size(22.0).color(TEXT).strong());
    ui.add_space(2.0);
}

/// Secondary text under a label or a heading.
pub fn help(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(12.5).color(TEXT_DIM));
}

/// A line of body text that wraps.
pub fn body(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(14.0).color(TEXT));
}

/// A thin separator that is quieter than egui's default.
pub fn rule(ui: &mut egui::Ui) {
    ui.add_space(6.0);
    let available = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(available, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BORDER);
    ui.add_space(6.0);
}

/// A card: a panel with a border, for grouping related things.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::same(14))
        .show(ui, add)
        .inner
}

/// The big primary button, used once per screen at most.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(egui::RichText::new(text).size(16.0).color(Color32::WHITE))
        .fill(ACCENT)
        .corner_radius(CornerRadius::same(3))
        .min_size(egui::vec2(180.0, 40.0));
    ui.add(button)
}

/// Colour for a status line, by how bad it is.
pub fn status_color(level: Level) -> Color32 {
    match level {
        Level::Good => GOOD,
        Level::Info => TEXT_DIM,
        Level::Warn => WARN,
        Level::Bad => BAD,
    }
}

/// How loud a status line should be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Good,
    Info,
    Warn,
    Bad,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Text on the background has to be readable. The threshold is the
    /// WCAG AA ratio for body text.
    #[test]
    fn body_text_has_enough_contrast() {
        assert!(
            contrast(TEXT, BACKGROUND) >= 4.5,
            "body text on the background is {:.2}:1",
            contrast(TEXT, BACKGROUND)
        );
        assert!(
            contrast(TEXT, SURFACE) >= 4.5,
            "body text on a card is {:.2}:1",
            contrast(TEXT, SURFACE)
        );
    }

    /// Help text is smaller but still has to be legible. The AA threshold
    /// for large text is 3:1 and dim text sits between the two.
    #[test]
    fn dim_text_is_still_legible() {
        assert!(
            contrast(TEXT_DIM, BACKGROUND) >= 3.0,
            "dim text is {:.2}:1",
            contrast(TEXT_DIM, BACKGROUND)
        );
    }

    #[test]
    fn white_on_the_accent_is_readable_for_buttons() {
        assert!(
            contrast(Color32::WHITE, ACCENT) >= 4.5,
            "button text is {:.2}:1",
            contrast(Color32::WHITE, ACCENT)
        );
    }

    #[test]
    fn the_status_colours_are_distinguishable_from_the_background() {
        for colour in [GOOD, WARN, BAD] {
            assert!(
                contrast(colour, SURFACE) >= 3.0,
                "{colour:?} on a card is {:.2}:1",
                contrast(colour, SURFACE)
            );
        }
    }

    #[test]
    fn every_level_maps_to_a_colour() {
        for level in [Level::Good, Level::Info, Level::Warn, Level::Bad] {
            let _ = status_color(level);
        }
    }

    /// WCAG relative luminance.
    fn luminance(colour: Color32) -> f32 {
        fn channel(value: u8) -> f32 {
            let value = value as f32 / 255.0;
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(colour.r()) + 0.7152 * channel(colour.g()) + 0.0722 * channel(colour.b())
    }

    fn contrast(foreground: Color32, background: Color32) -> f32 {
        let (a, b) = (luminance(foreground), luminance(background));
        let (lighter, darker) = if a > b { (a, b) } else { (b, a) };
        (lighter + 0.05) / (darker + 0.05)
    }
}
