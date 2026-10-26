//! Drawing release notes.
//!
//! The reading is done by [`recof_core::changelog`]; this only turns the
//! blocks into widgets. Links open in the browser only when they are
//! `https`, so a note cannot make the launcher open anything else.

use egui::{Color32, RichText, Stroke};
use recof_core::changelog::{self, Alert, Block, Span};

use crate::app::App;
use crate::theme;

/// Size of body text in the notes.
const BODY_SIZE: f32 = 13.0;

/// Draws release notes.
pub fn show(app: &App, ui: &mut egui::Ui, body: &str) {
    for (index, block) in changelog::parse(body).iter().enumerate() {
        if index > 0 {
            ui.add_space(4.0);
        }
        draw_block(app, ui, block, index);
    }
}

fn draw_block(app: &App, ui: &mut egui::Ui, block: &Block, index: usize) {
    match block {
        Block::Heading { level, spans } => {
            ui.add_space(4.0);
            let size = if *level <= 2 { 15.5 } else { 14.0 };
            ui.label(
                RichText::new(changelog::plain_text(spans))
                    .size(size)
                    .strong()
                    .color(theme::TEXT),
            );
        }
        Block::Paragraph(spans) => line(ui, spans),
        Block::Item {
            depth,
            number,
            spans,
        } => {
            ui.horizontal_top(|ui| {
                ui.add_space(4.0 + 16.0 * *depth as f32);
                let marker = match number {
                    Some(number) => format!("{number}."),
                    None => "\u{2022}".to_owned(),
                };
                ui.label(RichText::new(marker).size(BODY_SIZE).color(theme::TEXT_DIM));
                ui.vertical(|ui| line(ui, spans));
            });
        }
        Block::Quote { alert, paragraphs } => {
            let color = alert.map(alert_color).unwrap_or(theme::BORDER);
            egui::Frame::new()
                .fill(theme::SURFACE_RAISED)
                .stroke(Stroke::new(1.0, color))
                .corner_radius(egui::CornerRadius::same(3))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if let Some(alert) = alert {
                        ui.label(
                            RichText::new(app.t(alert.message_id()))
                                .size(BODY_SIZE)
                                .strong()
                                .color(color),
                        );
                    }
                    for spans in paragraphs {
                        line(ui, spans);
                    }
                });
        }
        Block::Table { header, rows } => {
            egui::ScrollArea::horizontal()
                .id_salt(("changelog-table", index))
                .show(ui, |ui| {
                    egui::Grid::new(("changelog-grid", index))
                        .striped(true)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            for cell in header {
                                ui.label(
                                    RichText::new(changelog::plain_text(cell))
                                        .size(BODY_SIZE)
                                        .strong()
                                        .color(theme::TEXT),
                                );
                            }
                            ui.end_row();
                            for row in rows {
                                for cell in row {
                                    line(ui, cell);
                                }
                                ui.end_row();
                            }
                        });
                });
        }
        Block::Code(code) => {
            egui::Frame::new()
                .fill(theme::BACKGROUND)
                .inner_margin(egui::Margin::same(6))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(code)
                            .monospace()
                            .size(12.0)
                            .color(theme::TEXT),
                    );
                });
        }
        Block::Rule => {
            ui.separator();
        }
    }
}

/// One line of inline text, wrapping as needed.
fn line(ui: &mut egui::Ui, spans: &[Span]) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for span in spans {
            match span {
                Span::Text(text) => {
                    ui.label(RichText::new(text).size(BODY_SIZE).color(theme::TEXT_DIM));
                }
                Span::Bold(text) => {
                    ui.label(
                        RichText::new(text)
                            .size(BODY_SIZE)
                            .strong()
                            .color(theme::TEXT),
                    );
                }
                Span::Code(text) => {
                    ui.label(RichText::new(text).size(BODY_SIZE - 0.5).code());
                }
                Span::Link { text, url } if url.starts_with("https://") => {
                    ui.hyperlink_to(RichText::new(text).size(BODY_SIZE), url);
                }
                Span::Link { text, .. } => {
                    ui.label(RichText::new(text).size(BODY_SIZE).color(theme::TEXT_DIM));
                }
            }
        }
    });
}

fn alert_color(alert: Alert) -> Color32 {
    match alert {
        Alert::Note | Alert::Tip => theme::GOOD,
        Alert::Important | Alert::Warning => theme::WARN,
        Alert::Caution => theme::BAD,
    }
}
