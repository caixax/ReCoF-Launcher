//! Updates: the line and the button the main screen shows, the launcher's
//! update windows, the changelog, rolling back and installing from a file.

use recof_core::manifest::Product;
use recof_core::update::selfupdate;
use recof_core::version::Version;

use crate::app::{App, Notice, Progress};
use crate::theme;
use crate::worker::Job;

/// Asks for the update check, once per start.
///
/// Called as the launcher opens rather than when the main screen is
/// reached, so a first run that goes through the install wizard has
/// still checked by the time it gets there. It runs on its own thread,
/// so nothing waits for it.
pub fn check_once(app: &mut App) {
    if app.update_check_requested || !app.settings.check_updates_on_start {
        return;
    }
    send_check(app, false);
}

/// Asks for a check whatever the setting says, for when the user asked
/// for one, such as by changing the channel on the install screen.
pub fn check_now(app: &mut App) {
    send_check(app, true);
}

/// Starts a check; `asked` makes it ask GitHub even when the cached answer
/// is fresh.
fn send_check(app: &mut App, asked: bool) {
    app.update_check_requested = true;
    app.update_checking = true;
    app.worker.send_background(Job::CheckUpdates {
        app_data: app.paths.root().to_path_buf(),
        channel: app.settings.channel,
        recof_installed: app.installed.as_ref().map(|state| state.version.clone()),
        asked,
    });
}

/// What the last check found, in one line.
///
/// Shown under the check button, so pressing it says something even
/// when the answer is that there is nothing new.
pub fn last_result(app: &mut App, ui: &mut egui::Ui) {
    if app.update_checking {
        return;
    }
    if app.updates.offline {
        ui.label(
            egui::RichText::new(app.t("home-offline"))
                .size(12.0)
                .color(theme::WARN),
        );
        return;
    }
    if !app.update_check_requested {
        return;
    }

    match (&app.updates.launcher, &app.updates.recof) {
        (None, None) => ui.label(
            egui::RichText::new(app.t("home-up-to-date"))
                .size(12.0)
                .color(theme::GOOD),
        ),
        (launcher, recof) => {
            let text = match (launcher, recof) {
                (Some(available), _) => app.t_args(
                    "home-launcher-update-available",
                    &[("version", available.version.to_string())],
                ),
                (None, Some(available)) => app.t_args(
                    "home-update-available",
                    &[("version", available.version.to_string())],
                ),
                (None, None) => unreachable!("the empty case is handled above"),
            };
            ui.label(egui::RichText::new(text).size(12.0).color(theme::TEXT))
        }
    };
}

/// The line on the main screen saying where things stand.
pub fn status_line(app: &mut App, ui: &mut egui::Ui) {
    if app.updates.offline {
        theme::help(ui, &app.t("home-offline"));
        return;
    }
    if app.update_checking {
        theme::help(ui, &app.t("home-checking"));
        ui.spinner();
        return;
    }
    if app.update_check_requested && app.updates.recof.is_none() {
        theme::help(ui, &app.t("home-up-to-date"));
    }
}

/// The ReCoF update the main screen offers, and whether this launcher is
/// too old to install it.
///
/// A release that needs a newer launcher is announced, not offered:
/// installing it with this launcher is exactly what must not happen.
pub fn recof_update(app: &App) -> Option<(recof_core::update::Available, bool)> {
    let available = app.updates.recof.clone()?;
    let running =
        Version::parse(recof_core::consts::LAUNCHER_VERSION).unwrap_or(Version::new(0, 0, 0));
    let needs_newer = app
        .installed
        .as_ref()
        .map(|state| selfupdate::launcher_too_old(&state.manifest.min_launcher, &running))
        .unwrap_or(false);
    Some((available, needs_newer))
}

/// Starts the ReCoF update: a backup of the saves, then the download and
/// the journal, with the progress under the play buttons.
pub fn start_recof_update(app: &mut App, available: &recof_core::update::Available) {
    let Some(install_root) = app.settings.install_path.clone() else {
        return;
    };
    app.progress = Some(Progress {
        headline: app.t("update-backup-first"),
        ..Default::default()
    });
    app.worker.send(Job::Update {
        available: Box::new(available.clone()),
        install_root,
        app_data: app.paths.root().to_path_buf(),
        source_buildid: app
            .installed
            .as_ref()
            .and_then(|state| state.source_buildid),
        now: crate::now_rfc3339(),
        stamp: crate::now_stamp(),
        keep_versions: app.settings.keep_versions,
    });
}

/// The line beside the play button: a ReCoF update and the way to its
/// notes, or where the update check stands.
pub fn home_line(app: &mut App, ui: &mut egui::Ui) {
    let Some((available, needs_newer)) = recof_update(app) else {
        status_line(app, ui);
        return;
    };

    ui.vertical(|ui| {
        let line = app.t_args(
            "update-available-title",
            &[("version", available.version.to_string())],
        );
        ui.label(egui::RichText::new(line).strong().color(theme::GOOD));
        if needs_newer {
            ui.label(
                egui::RichText::new(
                    app.t_args(
                        "update-launcher-first",
                        &[(
                            "needed",
                            app.installed
                                .as_ref()
                                .map(|state| state.manifest.min_launcher.to_string())
                                .unwrap_or_default(),
                        )],
                    ),
                )
                .size(12.0)
                .color(theme::WARN),
            );
        }
        if ui.small_button(app.t("update-changelog")).clicked() {
            app.recof_notes_popup = true;
        }
    });
}

/// The progress in the row beside the play button: one line of text and
/// the bar.
pub fn progress_compact(app: &App, ui: &mut egui::Ui) {
    let Some(progress) = app.progress.clone() else {
        return;
    };
    ui.vertical(|ui| {
        let mut text = progress.headline.clone();
        if !progress.detail.is_empty() {
            text = format!("{text}  {}", progress.detail);
        }
        ui.label(egui::RichText::new(text).size(12.5).color(theme::TEXT));
        let bar = egui::ProgressBar::new(progress.fraction.unwrap_or(0.0))
            .desired_width(ui.available_width().min(420.0))
            .fill(theme::ACCENT)
            .animate(progress.fraction.is_none());
        ui.add(bar);
    });
}

/// The progress of what the worker is doing, on the screens that have no
/// progress screen of their own.
pub fn progress(app: &App, ui: &mut egui::Ui) {
    let Some(progress) = app.progress.clone() else {
        return;
    };
    ui.add_space(8.0);
    ui.label(egui::RichText::new(&progress.headline).strong());
    let bar = egui::ProgressBar::new(progress.fraction.unwrap_or(0.0))
        .desired_width(ui.available_width())
        .fill(theme::ACCENT)
        .animate(progress.fraction.is_none());
    ui.add(bar);
    if !progress.detail.is_empty() {
        theme::help(ui, &progress.detail);
    }
}

/// The two windows a launcher update brings: the offer, as soon as the
/// check finds a version the user has not skipped, and the restart once
/// the new launcher is on disk.
pub fn popups(app: &mut App, ctx: &egui::Context) {
    use crate::app::Screen;

    // Not over the first run screens or an install in progress.
    if matches!(
        app.screen,
        Screen::Language | Screen::Intro(_) | Screen::Installing
    ) {
        return;
    }

    // The notes belong to the main screen, where the update button is.
    if app.recof_notes_popup && app.screen == Screen::Home {
        recof_notes_popup(app, ctx);
    }

    if app.restart_popup && app.self_update_ready.is_some() {
        restart_popup(app, ctx);
        return;
    }

    if !app.launcher_popup {
        return;
    }
    let Some(available) = app.updates.launcher.clone() else {
        app.launcher_popup = false;
        return;
    };

    let busy = app.worker.is_busy();
    egui::Window::new(app.t("update-launcher-title"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_max_width(520.0);
            ui.label(
                egui::RichText::new(app.t_args(
                    "home-launcher-update-available",
                    &[("version", available.version.to_string())],
                ))
                .strong()
                .color(theme::GOOD),
            );
            theme::help(
                ui,
                &app.t_args(
                    "update-installed-version",
                    &[("version", recof_core::consts::LAUNCHER_VERSION.to_owned())],
                ),
            );
            ui.add_space(6.0);
            ui.label(egui::RichText::new(app.t("update-changelog")).strong());
            changelog_body(app, ui, &available.changelog, "launcher-changelog-body");

            if busy {
                progress(app, ui);
                return;
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &app.t("update-install")).clicked() {
                    app.progress = Some(Progress {
                        headline: app.t("update-launcher-title"),
                        ..Default::default()
                    });
                    app.worker.send(Job::SelfUpdate {
                        available: Box::new(available.clone()),
                        app_data: app.paths.root().to_path_buf(),
                    });
                }
                if ui.button(app.t("update-skip-version")).clicked() {
                    app.settings.skipped_launcher_version = Some(available.version.to_string());
                    app.save_settings();
                    app.launcher_popup = false;
                }
            });
        });
}

/// What changed in the ReCoF update, with the button that installs it.
fn recof_notes_popup(app: &mut App, ctx: &egui::Context) {
    let Some((available, needs_newer)) = recof_update(app) else {
        app.recof_notes_popup = false;
        return;
    };

    let busy = app.worker.is_busy();
    let mut open = true;
    egui::Window::new(app.t_args(
        "update-available-title",
        &[("version", available.version.to_string())],
    ))
    .collapsible(false)
    .resizable(false)
    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
    .show(ctx, |ui| {
        ui.set_max_width(560.0);
        if let Some(installed) = &available.installed {
            theme::help(
                ui,
                &app.t_args(
                    "update-installed-version",
                    &[("version", installed.to_string())],
                ),
            );
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(app.t("update-changelog")).strong());
        changelog_body(app, ui, &available.changelog, "recof-notes-body");

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if !needs_newer {
                ui.add_enabled_ui(!busy, |ui| {
                    if theme::primary_button(ui, &app.t("update-install")).clicked() {
                        start_recof_update(app, &available);
                        open = false;
                    }
                });
            }
            if ui.button(app.t("button-close")).clicked() {
                open = false;
            }
        });
    });

    if !open {
        app.recof_notes_popup = false;
    }
}

/// Asks to restart once a newer launcher is on disk.
fn restart_popup(app: &mut App, ctx: &egui::Context) {
    let version = app
        .self_update_ready
        .as_ref()
        .map(|version| version.to_string())
        .unwrap_or_default();

    egui::Window::new(app.t("update-launcher-title"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_max_width(460.0);
            ui.label(
                egui::RichText::new(app.t_args("update-launcher-ready", &[("version", version)]))
                    .strong(),
            );
            theme::help(ui, &app.t("update-launcher-restart"));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &app.t("update-restart-now")).clicked() {
                    match selfupdate::restart() {
                        Ok(()) => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
                        Err(error) => {
                            log::error!("the launcher could not restart: {error}");
                            app.notice = Some(Notice::bad(app.t("launch-failed")));
                        }
                    }
                }
                if ui.button(app.t("update-later")).clicked() {
                    app.restart_popup = false;
                }
            });
        });
}

/// The release notes, drawn the way GitHub shows them, in a box that
/// scrolls so a long changelog does not push the buttons off screen.
pub fn changelog(app: &App, ui: &mut egui::Ui, body: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(app.t("update-changelog")).strong());
    changelog_body(app, ui, body, "changelog");
}

/// The notes themselves, in a box of their own that scrolls.
fn changelog_body(app: &App, ui: &mut egui::Ui, body: &str, id: &str) {
    if body.trim().is_empty() {
        theme::help(ui, &app.t("update-no-changelog"));
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .id_salt(id)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            crate::markdown::show(app, ui, body);
        });
}

/// Going back to a version the store still holds.
pub fn rollback(app: &mut App, ui: &mut egui::Ui) {
    let Some(install_root) = app.settings.install_path.clone() else {
        return;
    };
    let installed = app.installed.as_ref().map(|state| state.version.clone());

    let stored = app.store.list(Product::Recof).unwrap_or_default();
    let older: Vec<_> = stored
        .into_iter()
        .filter(|entry| Some(&entry.version) != installed.as_ref())
        .collect();

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("rollback-title")).strong());
        ui.add_space(4.0);

        if older.is_empty() {
            theme::help(ui, &app.t("rollback-none"));
            return;
        }
        theme::help(ui, &app.t("rollback-offer-backup"));
        ui.add_space(4.0);

        for entry in older.iter().rev() {
            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                let label = app.t_args("rollback-to", &[("version", entry.version.to_string())]);
                if ui.button(label).clicked() {
                    app.progress = Some(Progress {
                        headline: app.t("update-backup-first"),
                        ..Default::default()
                    });
                    app.worker.send(Job::Rollback {
                        install_root: install_root.clone(),
                        app_data: app.paths.root().to_path_buf(),
                        version: entry.version.clone(),
                        now: crate::now_rfc3339(),
                        stamp: crate::now_stamp(),
                    });
                }
            });
        }
    });
}

/// Installing a release the user already has on disk.
pub fn from_file(app: &mut App, ui: &mut egui::Ui) {
    let Some(install_root) = app.settings.install_path.clone() else {
        return;
    };

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("update-from-file")).strong());
        theme::help(ui, &app.t("update-from-file-help"));
        ui.add_space(6.0);

        ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
            if ui.button(app.t("button-browse")).clicked() {
                let picked = rfd::FileDialog::new()
                    .set_title(app.t("update-from-file"))
                    .add_filter("zip", &["zip"])
                    .pick_file();
                if let Some(archive) = picked {
                    app.progress = Some(Progress {
                        headline: app.t("update-verifying"),
                        ..Default::default()
                    });
                    app.worker.send(Job::InstallFromFile {
                        archive,
                        install_root: install_root.clone(),
                        app_data: app.paths.root().to_path_buf(),
                        source_buildid: app
                            .installed
                            .as_ref()
                            .and_then(|state| state.source_buildid),
                        now: crate::now_rfc3339(),
                        stamp: crate::now_stamp(),
                    });
                }
            }
        });
    });
}
