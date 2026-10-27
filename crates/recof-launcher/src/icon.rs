//! Small line icons, drawn rather than loaded.
//!
//! A font of icons would be another file to ship and another thing to
//! render as an empty box when it is missing. These are a dozen lines
//! and a few circles each, they scale to any size, and they take their
//! colour from whatever is drawing them.
//!
//! Everything here is laid out inside a unit square and scaled to the
//! rect it is given, so one icon is never a different weight from
//! another beside it.

use egui::{Color32, Painter, Rect, Stroke, Vec2};

/// The icons the launcher uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// A filled triangle. The play button, and the row for a last launch.
    Play,
    /// A clock face. The row for hours played.
    Clock,
    /// A stack of plates. The row for which version is installed.
    Layers,
    /// A house. Back to the main screen.
    Home,
    /// Three labelled sliders. The options the game is started with.
    Sliders,
    /// A cogwheel. The launcher's own settings.
    Gear,
    /// A shield. Verify, repair, move and uninstall.
    Shield,
    /// A speech bubble. Reporting a problem.
    Bubble,
    /// A folder. Opening the install on disk.
    Folder,
    /// Three stacked lines. The menu beside the play button.
    Menu,
}

impl Icon {
    /// Draws the icon centred in `rect`.
    ///
    /// The stroke is derived from the size so a small icon does not come
    /// out as a blob and a large one does not come out as a hairline.
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        // Square, centred, with a little room so strokes do not touch
        // the edges of whatever is behind them.
        let side = rect.width().min(rect.height());
        let box_ = Rect::from_center_size(rect.center(), Vec2::splat(side));
        let width = (side * 0.085).clamp(1.0, 2.4);
        let stroke = Stroke::new(width, color);

        // Maps a point in the unit square to the box on screen.
        let at = |x: f32, y: f32| box_.lerp_inside(Vec2::new(x, y));
        let line = |from: (f32, f32), to: (f32, f32)| {
            painter.line_segment([at(from.0, from.1), at(to.0, to.1)], stroke);
        };

        match self {
            Icon::Play => {
                painter.add(egui::Shape::convex_polygon(
                    vec![at(0.28, 0.16), at(0.84, 0.5), at(0.28, 0.84)],
                    color,
                    Stroke::NONE,
                ));
            }
            Icon::Clock => {
                painter.circle_stroke(box_.center(), side * 0.37, stroke);
                // Hands at ten past ten, which reads as a clock at any
                // size and never as a full or empty circle.
                line((0.5, 0.5), (0.5, 0.26));
                line((0.5, 0.5), (0.68, 0.6));
            }
            Icon::Layers => {
                // A diamond with two lines under it: a stack seen edge on.
                painter.add(egui::Shape::closed_line(
                    vec![at(0.5, 0.14), at(0.88, 0.36), at(0.5, 0.58), at(0.12, 0.36)],
                    stroke,
                ));
                line((0.12, 0.54), (0.5, 0.76));
                line((0.88, 0.54), (0.5, 0.76));
            }
            Icon::Home => {
                line((0.12, 0.5), (0.5, 0.16));
                line((0.5, 0.16), (0.88, 0.5));
                painter.add(egui::Shape::closed_line(
                    vec![
                        at(0.24, 0.46),
                        at(0.76, 0.46),
                        at(0.76, 0.86),
                        at(0.24, 0.86),
                    ],
                    stroke,
                ));
            }
            Icon::Sliders => {
                // Three rails, each with its knob at a different stop.
                for (index, stop) in [0.66_f32, 0.36, 0.56].into_iter().enumerate() {
                    let y = 0.25 + index as f32 * 0.25;
                    line((0.14, y), (0.86, y));
                    painter.circle_filled(at(stop, y), side * 0.085, color);
                }
            }
            Icon::Gear => {
                // A ring with teeth around it. Drawn from the centre so
                // the teeth stay evenly spaced at any size.
                let centre = box_.center();
                let ring = side * 0.26;
                painter.circle_stroke(centre, ring, stroke);
                for tooth in 0..8 {
                    let angle = std::f32::consts::TAU * tooth as f32 / 8.0;
                    let (sin, cos) = angle.sin_cos();
                    let step = Vec2::new(cos, sin);
                    painter.line_segment(
                        [
                            centre + step * (ring + side * 0.04),
                            centre + step * (ring + side * 0.14),
                        ],
                        stroke,
                    );
                }
            }
            Icon::Shield => {
                painter.add(egui::Shape::closed_line(
                    vec![
                        at(0.5, 0.12),
                        at(0.84, 0.26),
                        at(0.84, 0.54),
                        at(0.5, 0.88),
                        at(0.16, 0.54),
                        at(0.16, 0.26),
                    ],
                    stroke,
                ));
                line((0.34, 0.49), (0.46, 0.61));
                line((0.46, 0.61), (0.68, 0.37));
            }
            Icon::Bubble => {
                painter.rect_stroke(
                    Rect::from_min_max(at(0.14, 0.2), at(0.86, 0.66)),
                    egui::CornerRadius::same((side * 0.12) as u8),
                    stroke,
                    egui::StrokeKind::Inside,
                );
                // The tail, which is what makes it a bubble and not a box.
                line((0.3, 0.66), (0.3, 0.84));
                line((0.3, 0.84), (0.48, 0.66));
            }
            Icon::Folder => {
                line((0.12, 0.32), (0.44, 0.32));
                line((0.44, 0.32), (0.52, 0.42));
                painter.add(egui::Shape::closed_line(
                    vec![
                        at(0.12, 0.3),
                        at(0.88, 0.42),
                        at(0.88, 0.78),
                        at(0.12, 0.78),
                    ],
                    stroke,
                ));
            }
            Icon::Menu => {
                for index in 0..3 {
                    let y = 0.3 + index as f32 * 0.2;
                    line((0.18, y), (0.82, y));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    /// Every icon, so a new one cannot be added without being drawn.
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
    ];

    /// Draws every icon once and returns how many were drawn, so a
    /// panic in any of them fails the test rather than being swallowed.
    fn paint_all(rect: Rect) -> usize {
        let mut drawn = 0;
        egui::__run_test_ui(|ui| {
            drawn = 0;
            for icon in ALL {
                icon.paint(ui.painter(), rect, Color32::WHITE);
                drawn += 1;
            }
        });
        drawn
    }

    #[test]
    fn every_icon_draws_at_the_size_it_is_used_at() {
        // The rail and the stat rows, which is every size in the app.
        assert_eq!(
            paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(18.0))),
            ALL.len()
        );
        assert_eq!(
            paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(22.0))),
            ALL.len()
        );
    }

    #[test]
    fn a_degenerate_rect_does_not_bring_the_launcher_down() {
        // A layout mid resize can hand out a zero or negative rect, and
        // an icon is not worth a panic.
        assert_eq!(
            paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(0.0))),
            ALL.len()
        );
        assert_eq!(
            paint_all(Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::ZERO)),
            ALL.len()
        );
    }

    #[test]
    fn a_huge_icon_keeps_a_sane_stroke() {
        // The clamp is what stops a large icon turning into a blob.
        assert_eq!(
            paint_all(Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0))),
            ALL.len()
        );
    }
}
