//! The launcher's icons, from Font Awesome Free.
//!
//! Font Awesome Free 6.7.2's solid and brands fonts travel inside the
//! launcher (`assets/fa-solid-900.ttf` and `assets/fa-brands-400.ttf`,
//! unmodified, under the SIL Open Font License 1.1 in
//! `assets/FontAwesome-LICENSE.txt`) and are registered with the other
//! fonts by [`crate::brand::install_font`]. An icon is one glyph, so it
//! scales to any size and takes its colour from whatever draws it.

use egui::{Align2, Color32, FontFamily, FontId, Painter, Rect};

/// The family Font Awesome's solid icons are registered under.
pub const SOLID: &str = "fa-solid";

/// The family Font Awesome's brand icons are registered under.
pub const BRANDS: &str = "fa-brands";

/// The icons the launcher uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// play. The play button, and the row for a last launch.
    Play,
    /// clock. The row for hours played.
    Clock,
    /// layer-group. The row for which version is installed.
    Layers,
    /// house. Back to the main screen.
    Home,
    /// sliders. The options the game is started with.
    Sliders,
    /// gear. The launcher's own settings.
    Gear,
    /// shield-halved. Verify, repair, move and uninstall.
    Shield,
    /// comment. Reporting a problem.
    Bubble,
    /// folder-open. Opening the install on disk.
    Folder,
    /// bars. The menu beside the play button.
    Menu,
    /// github (brands). ReCoF's repository.
    GitHub,
    /// globe. ReCoF's website.
    Globe,
    /// discord (brands). ReCoF's Discord server.
    Discord,
}

impl Icon {
    /// The icon's character and the family it is in.
    pub fn glyph(self) -> (char, &'static str) {
        match self {
            Icon::Play => ('\u{f04b}', SOLID),
            Icon::Clock => ('\u{f017}', SOLID),
            Icon::Layers => ('\u{f5fd}', SOLID),
            Icon::Home => ('\u{f015}', SOLID),
            Icon::Sliders => ('\u{f1de}', SOLID),
            Icon::Gear => ('\u{f013}', SOLID),
            Icon::Shield => ('\u{f3ed}', SOLID),
            Icon::Bubble => ('\u{f075}', SOLID),
            Icon::Folder => ('\u{f07c}', SOLID),
            Icon::Menu => ('\u{f0c9}', SOLID),
            Icon::GitHub => ('\u{f09b}', BRANDS),
            Icon::Globe => ('\u{f0ac}', SOLID),
            Icon::Discord => ('\u{f392}', BRANDS),
        }
    }

    /// The font the icon is drawn with, at `size` points.
    pub fn font(self, size: f32) -> FontId {
        FontId::new(size, FontFamily::Name(self.glyph().1.into()))
    }

    /// Draws the icon centred in `rect`.
    ///
    /// Font Awesome draws its icons in a 512 unit em, most of them 512
    /// units wide and the widest 640, so a size of 0.8 of the side keeps
    /// every one of them inside the square. The em is centred on the
    /// font's line, so the glyph is centred both ways.
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        let side = rect.width().min(rect.height());
        // A layout mid resize can hand out a zero, negative or NaN rect,
        // and an icon is not worth a panic.
        if side.is_nan() || side <= 0.5 {
            return;
        }
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            self.glyph().0,
            self.font(side * 0.8),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, Vec2};

    /// Every icon, so a new one cannot be added without being checked.
    const ALL: &[Icon] = &[
        Icon::Play,
        Icon::Clock,
        Icon::Layers,
        Icon::Home,
        Icon::Sliders,
        Icon::Gear,
        Icon::Shield,
        Icon::Bubble,
        Icon::Folder,
        Icon::Menu,
        Icon::GitHub,
        Icon::Globe,
        Icon::Discord,
    ];

    /// A context with the launcher's fonts in place, as the app has them.
    fn context() -> egui::Context {
        let ctx = egui::Context::default();
        crate::brand::install_font(&ctx);
        // Fonts set now are taken at the start of the next pass.
        let output = ctx.run_pass(Default::default(), |_| {});
        output.drop_without_applying_deltas();
        ctx
    }

    #[test]
    fn every_icon_is_in_its_font() {
        // A wrong code point would draw a box, which only shows up when
        // someone looks at the screen.
        let ctx = context();
        for icon in ALL {
            let (glyph, _) = icon.glyph();
            let font = icon.font(16.0);
            assert!(
                ctx.fonts_mut(|fonts| fonts.has_glyph(&font, glyph)),
                "{icon:?} (U+{:04X}) is not in its font",
                glyph as u32
            );
        }
    }

    /// Draws every icon once and returns how many were drawn, so a
    /// panic in any of them fails the test rather than being swallowed.
    fn paint_all(rect: Rect) -> usize {
        let ctx = context();
        let mut drawn = 0;
        let output = ctx.run_ui(Default::default(), |ui| {
            drawn = 0;
            for icon in ALL {
                icon.paint(ui.painter(), rect, Color32::WHITE);
                drawn += 1;
            }
        });
        output.drop_without_applying_deltas();
        drawn
    }

    #[test]
    fn every_icon_draws_at_the_size_it_is_used_at() {
        // The rail, the stat rows, the play button and the links.
        for side in [16.0, 18.0, 20.0, 22.0] {
            assert_eq!(
                paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(side))),
                ALL.len()
            );
        }
    }

    #[test]
    fn a_degenerate_rect_does_not_bring_the_launcher_down() {
        assert_eq!(
            paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(0.0))),
            ALL.len()
        );
        assert_eq!(
            paint_all(Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::ZERO)),
            ALL.len()
        );
        assert_eq!(paint_all(Rect::NAN), ALL.len());
    }

    #[test]
    fn a_huge_icon_still_draws() {
        assert_eq!(
            paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0))),
            ALL.len()
        );
    }
}
