// A console window behind the launcher would look like a crash. Debug builds
// keep it so logs and panics are visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use recof_core::consts;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([800.0, 560.0])
            .with_title(consts::APP_DISPLAY_NAME),
        ..Default::default()
    };

    eframe::run_native(
        consts::APP_DISPLAY_NAME,
        options,
        Box::new(|_cc| Ok(Box::new(App))),
    )
}

/// Placeholder shell. The real screens arrive in phase 4.
struct App;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The root Ui comes with no margin and no background.
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            ui.heading(consts::APP_DISPLAY_NAME);
            ui.label(format!("version {}", consts::LAUNCHER_VERSION));
        });
    }
}
