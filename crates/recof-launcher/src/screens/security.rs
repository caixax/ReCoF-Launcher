//! The Windows Security exclusions: the window that offers them at start
//! and after an install, and the card in the settings that adds them
//! again.
//!
//! ReCoF and the launcher are not signed yet, so Microsoft Defender can
//! wrongly block or delete their files. The offer is made when Defender's
//! real-time protection is on, the player has not said "Don't ask again",
//! and a folder is not yet in the list the launcher remembers.

use std::path::PathBuf;

use recof_core::defender::{self, Scope};
use recof_core::paths::comparable;

use crate::app::{App, Notice, Screen};
use crate::theme;
use crate::worker::Job;

/// Where the exclusions stand in this session.
#[derive(Debug, Default)]
pub struct DefenderState {
    /// The folders as last worked out, for the settings card. `None`
    /// until a check has come back.
    pub folders: Option<Vec<PathBuf>>,
    /// Whether a check is running now.
    pub checking: bool,
    /// The folders the window offers, while it is open.
    pub offer: Option<Vec<PathBuf>>,
    /// Whether the administrator prompt is up.
    pub adding: bool,
    /// Why the last attempt did not work, shown in the window.
    pub problem: Option<String>,
    /// Folders the player answered "Not now" for, so they are not offered
    /// again until the next start. A new folder still is.
    pub dismissed: Vec<String>,
}

/// What the folders are worked out from. Steam's own folders are added by
/// the worker, which reads the registry.
fn scope(app: &App) -> Scope {
    let keep_out = app
        .settings
        .vanilla_path
        .iter()
        .cloned()
        .chain(app.game_path())
        .collect();
    Scope {
        launcher_exe: std::env::current_exe().ok(),
        app_data: app.paths.root().to_path_buf(),
        install_root: app.settings.install_path.clone(),
        keep_out,
    }
}

/// Works the folders out again on the worker's side. With `offer`, the
/// window opens afterwards when Defender is on and something is missing.
pub fn check(app: &mut App, offer: bool) {
    let ask = offer && !app.settings.defender.never_ask;
    app.defender.checking = true;
    app.worker.send_background(Job::DefenderCheck {
        scope: Box::new(scope(app)),
        ask,
    });
}

/// A check came back.
pub fn checked(app: &mut App, realtime_on: bool, folders: Vec<PathBuf>) {
    app.defender.checking = false;
    app.defender.folders = Some(folders.clone());
    if !realtime_on || app.settings.defender.never_ask || app.defender.adding {
        return;
    }

    let missing = defender::missing(&folders, &app.settings.defender);
    let fresh = missing
        .iter()
        .any(|folder| !app.defender.dismissed.contains(&comparable(folder)));
    if fresh {
        app.defender.offer = Some(missing);
        app.defender.problem = None;
    }
}

/// Forgets the folders, so the settings card works them out again: the
/// copy moved or went away.
pub fn forget_folders(app: &mut App) {
    app.defender.folders = None;
}

fn start_adding(app: &mut App, folders: Vec<PathBuf>) {
    app.defender.adding = true;
    app.defender.problem = None;
    // Its own thread: the prompt waits for the player, and nothing else
    // the worker does should wait with it.
    app.worker.send_background(Job::DefenderExclude { folders });
}

/// Windows took the exclusions.
pub fn excluded(app: &mut App, folders: &[PathBuf]) {
    app.defender.adding = false;
    app.defender.offer = None;
    app.defender.problem = None;
    defender::remember(&mut app.settings.defender, folders);
    app.save_settings();
    app.notice = Some(Notice::good(app.t("defender-done")));
}

/// Windows did not take them.
pub fn not_excluded(app: &mut App, declined: bool) {
    app.defender.adding = false;
    let message = app.t(if declined {
        "err-declined"
    } else {
        "defender-failed"
    });
    if app.defender.offer.is_some() {
        app.defender.problem = Some(message);
    } else {
        app.notice = Some(Notice::warn(message));
    }
}

/// The window offering the exclusions.
pub fn popup(app: &mut App, ctx: &egui::Context) {
    let Some(offer) = app.defender.offer.clone() else {
        return;
    };
    // Not over the first run screens or an install in progress, and one
    // window at a time: a launcher update or a question goes first.
    if matches!(
        app.screen,
        Screen::Language | Screen::Intro(_) | Screen::Installing
    ) || app.launcher_popup
        || app.restart_popup
        || app.recof_notes_popup
        || app.join_window.is_some()
        || app.confirm.is_some()
    {
        return;
    }

    egui::Window::new(app.t("defender-title"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_max_width(520.0);
            theme::body(ui, &app.t("defender-why"));
            ui.add_space(8.0);
            theme::body(ui, &app.t("defender-offer"));
            ui.add_space(4.0);
            folder_list(ui, &offer);
            ui.add_space(8.0);
            theme::help(ui, &app.t("defender-admin"));

            if let Some(problem) = &app.defender.problem {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(problem).color(theme::WARN));
            }

            ui.add_space(10.0);
            if app.defender.adding {
                ui.horizontal(|ui| {
                    ui.spinner();
                    theme::help(ui, &app.t("defender-waiting"));
                });
                return;
            }
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &app.t("defender-add")).clicked() {
                    start_adding(app, offer.clone());
                }
                if ui.button(app.t("defender-not-now")).clicked() {
                    app.defender
                        .dismissed
                        .extend(offer.iter().map(|folder| comparable(folder)));
                    app.defender.offer = None;
                    app.defender.problem = None;
                }
                if ui.button(app.t("defender-never")).clicked() {
                    app.settings.defender.never_ask = true;
                    app.save_settings();
                    app.defender.offer = None;
                    app.defender.problem = None;
                }
            });
        });
}

/// The card in the settings: add the exclusions again at any time, and
/// turn the offer back on after "Don't ask again".
pub fn settings_card(app: &mut App, ui: &mut egui::Ui) {
    if app.defender.folders.is_none() && !app.defender.checking {
        check(app, false);
    }

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("defender-title")).strong());
        theme::help(ui, &app.t("defender-settings-help"));
        ui.add_space(6.0);

        let folders = app.defender.folders.clone();
        match &folders {
            Some(folders) => folder_list(ui, folders),
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    theme::help(ui, &app.t("defender-checking"));
                });
            }
        }

        ui.add_space(6.0);
        let folders = folders.unwrap_or_default();
        ui.horizontal(|ui| {
            let can_add = !app.defender.adding && !folders.is_empty();
            ui.add_enabled_ui(can_add, |ui| {
                if ui.button(app.t("defender-add")).clicked() {
                    start_adding(app, folders.clone());
                }
            });
            if app.defender.adding {
                ui.spinner();
                theme::help(ui, &app.t("defender-waiting"));
            }
        });
        theme::help(ui, &app.t("defender-admin"));

        ui.add_space(4.0);
        let mut offer_at_start = !app.settings.defender.never_ask;
        if ui
            .checkbox(&mut offer_at_start, app.t("defender-settings-ask"))
            .changed()
        {
            app.settings.defender.never_ask = !offer_at_start;
            app.save_settings();
        }
    });
}

fn folder_list(ui: &mut egui::Ui, folders: &[PathBuf]) {
    for folder in folders {
        ui.label(
            egui::RichText::new(folder.display().to_string())
                .monospace()
                .color(theme::TEXT),
        );
    }
}
