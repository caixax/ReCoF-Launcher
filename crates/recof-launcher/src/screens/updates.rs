//! The update banner, the changelog, rolling back and installing from a
//! file.

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
    check_now(app);
}

/// Asks for a check whatever the setting says, for when the user asked
/// for one, such as by changing the channel on the install screen.
pub fn check_now(app: &mut App) {
    app.update_check_requested = true;
    app.update_checking = true;
    app.worker.send_background(Job::CheckUpdates {
        app_data: app.paths.root().to_path_buf(),
        channel: app.settings.channel,
        recof_installed: app.installed.as_ref().map(|state| state.version.clone()),
    });
}

/// The line on the main screen saying where things stand.
pub fn status_line(app: &mut App, ui: &mut egui::Ui) {
    if app.updates.offline {
        theme::help(ui, &app.t("home-offline"));
        return;
    }
    if app.worker.is_busy() && !app.update_check_requested {
        ui.horizontal(|ui| {
            ui.spinner();
            theme::help(ui, &app.t("home-checking"));
        });
        return;
    }
    if app.updates.recof.is_none() && app.updates.launcher.is_none() {
        theme::help(ui, &app.t("home-up-to-date"));
    }
}

/// The banner offering an update, which is where the changelog is shown.
pub fn banner(app: &mut App, ui: &mut egui::Ui) {
    // A launcher update comes first: a ReCoF release may need it.
    if let Some(available) = app.updates.launcher.clone() {
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(app.t_args(
                    "home-launcher-update-available",
                    &[("version", available.version.to_string())],
                ))
                .strong()
                .color(theme::WARN),
            );
            changelog(app, ui, &available.changelog);

            ui.add_space(6.0);
            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                if ui.button(app.t("update-install")).clicked() {
                    app.progress = Some(Progress {
                        headline: app.t("update-launcher-title"),
                        ..Default::default()
                    });
                    app.worker.send(Job::SelfUpdate {
                        available: Box::new(available.clone()),
                        app_data: app.paths.root().to_path_buf(),
                    });
                }
            });
        });
        ui.add_space(10.0);
    }

    let Some(available) = app.updates.recof.clone() else {
        return;
    };
    let Some(install_root) = app.settings.install_path.clone() else {
        return;
    };

    // A release that needs a newer launcher is announced, not offered:
    // installing it with this launcher is exactly what must not happen.
    let running =
        Version::parse(recof_core::consts::LAUNCHER_VERSION).unwrap_or(Version::new(0, 0, 0));
    let needs_newer = app
        .installed
        .as_ref()
        .map(|state| selfupdate::launcher_too_old(&state.manifest.min_launcher, &running))
        .unwrap_or(false);

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(app.t_args(
                "update-available-title",
                &[("version", available.version.to_string())],
            ))
            .strong()
            .color(theme::GOOD),
        );
        if let Some(installed) = &available.installed {
            theme::help(
                ui,
                &app.t_args(
                    "update-installed-version",
                    &[("version", installed.to_string())],
                ),
            );
        }
        changelog(app, ui, &available.changelog);

        ui.add_space(8.0);
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
                .color(theme::WARN),
            );
            return;
        }

        ui.horizontal(|ui| {
            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                if theme::primary_button(ui, &app.t("update-install")).clicked() {
                    app.progress = Some(Progress {
                        headline: app.t("update-backup-first"),
                        ..Default::default()
                    });
                    app.worker.send(Job::Update {
                        available: Box::new(available.clone()),
                        install_root: install_root.clone(),
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
            });
            if ui.button(app.t("update-later")).clicked() {
                app.updates.recof = None;
            }
        });
    });
}

/// The release notes, drawn the way GitHub shows them, in a box that
/// scrolls so a long changelog does not push the buttons off screen.
pub fn changelog(app: &App, ui: &mut egui::Ui, body: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(app.t("update-changelog")).strong());
    if body.trim().is_empty() {
        theme::help(ui, &app.t("update-no-changelog"));
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .id_salt("changelog")
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

/// The prompt that appears once a newer launcher is on disk.
pub fn restart_prompt(app: &mut App, ui: &mut egui::Ui) {
    let Some(version) = app.self_update_ready.clone() else {
        return;
    };

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(app.t_args(
                "home-launcher-update-available",
                &[("version", version.to_string())],
            ))
            .strong(),
        );
        theme::help(ui, &app.t("update-launcher-restart"));
        ui.add_space(6.0);
        if theme::primary_button(ui, &app.t("button-continue")).clicked() {
            match selfupdate::restart() {
                Ok(()) => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
                Err(error) => {
                    log::error!("the launcher could not restart: {error}");
                    app.notice = Some(Notice::bad(app.t("launch-failed")));
                }
            }
        }
    });
    ui.add_space(10.0);
}
