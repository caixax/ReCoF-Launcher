//! First run, detection and installation.

use std::path::PathBuf;

use recof_core::hashes::FileVerdict;
use recof_core::install::destination;
use recof_core::space::format_bytes;

use crate::app::{App, Confirm, ConfirmAction, Progress, Screen, INTRO_SCREENS};
use crate::i18n::LANGUAGES;
use crate::theme;
use crate::worker::Job;

/// First run, step one: the language list with the system one preselected.
pub fn language(app: &mut App, ui: &mut egui::Ui) {
    theme::heading(ui, &app.t("firstrun-language-title"));
    theme::help(ui, &app.t("firstrun-language-help"));
    ui.add_space(12.0);

    let current = app.translator.language().tag;
    let mut chosen = None;

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 70.0)
        .show(ui, |ui| {
            for language in LANGUAGES {
                let selected = language.tag == current;
                let mut label = egui::RichText::new(language.native_name).size(15.0);
                if selected {
                    label = label.strong();
                }
                let response = ui.selectable_label(selected, label);
                if response.clicked() {
                    chosen = Some(*language);
                }
                if !language.reviewed {
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        theme::help(ui, &app.t("language-not-reviewed"));
                    });
                }
                ui.add_space(2.0);
            }
        });

    if let Some(language) = chosen {
        app.set_language(language);
    }

    ui.add_space(10.0);
    theme::rule(ui);
    ui.horizontal(|ui| {
        if theme::primary_button(ui, &app.t("button-continue")).clicked() {
            if app.settings.language.is_none() {
                app.settings.language = Some(app.translator.language().tag.to_owned());
                app.save_settings();
            }
            app.screen = Screen::Intro(0);
        }
    });
}

/// First run, the explanation screens.
pub fn intro(app: &mut App, ui: &mut egui::Ui, step: usize) {
    let step = step.min(INTRO_SCREENS - 1);
    let title_key = format!("intro-{}-title", step + 1);
    let body_key = format!("intro-{}-body", step + 1);

    theme::heading(ui, &app.t(&title_key));
    ui.add_space(6.0);

    let size = app
        .detection
        .as_ref()
        .and_then(|detection| detection.game.as_ref())
        .and_then(|game| game.size_on_disk)
        .map(format_bytes)
        // Before detection has run, Steam's own figure for the game is the
        // honest estimate to show.
        .unwrap_or_else(|| format_bytes(4_695_601_334));

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(app.t_args(&body_key, &[("size", size)]))
                .size(15.0)
                .color(theme::TEXT),
        );
    });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        for index in 0..INTRO_SCREENS {
            let colour = if index == step {
                theme::ACCENT
            } else {
                theme::BORDER
            };
            let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 3.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 1.0, colour);
        }
    });

    ui.add_space(10.0);
    theme::rule(ui);
    ui.horizontal(|ui| {
        if step > 0 && ui.button(app.t("button-back")).clicked() {
            app.screen = Screen::Intro(step - 1);
        }
        let last = step + 1 >= INTRO_SCREENS;
        let label = if last {
            app.t("button-continue")
        } else {
            app.t("button-next")
        };
        if theme::primary_button(ui, &label).clicked() {
            if last {
                app.settings.intro_seen = true;
                app.save_settings();
                app.screen = Screen::Detect;
            } else {
                app.screen = Screen::Intro(step + 1);
            }
        }
        if !last && ui.button(app.t("button-skip")).clicked() {
            app.settings.intro_seen = true;
            app.save_settings();
            app.screen = Screen::Detect;
        }
    });
}

/// Finding the game and reporting what it is.
pub fn detect(app: &mut App, ui: &mut egui::Ui) {
    if !app.detect_requested {
        app.detect_requested = true;
        app.worker.send(Job::Detect);
    }

    theme::heading(ui, &app.t("detect-title"));

    let Some(detection) = &app.detection else {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spinner();
            theme::body(ui, &app.t("detect-searching"));
        });
        return;
    };

    let game_path = app.game_path();

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 80.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            match &game_path {
                Some(path) => {
                    theme::card(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            egui::RichText::new(
                                app.t_args("detect-found", &[("path", path.display().to_string())]),
                            )
                            .color(theme::GOOD),
                        );

                        if let Some(game) = &detection.game {
                            if let Some(library) = &game.library {
                                theme::help(
                                    ui,
                                    &app.t_args(
                                        "detect-library",
                                        &[("path", library.display().to_string())],
                                    ),
                                );
                            }
                            build_line(app, ui, game.buildid);
                        } else {
                            theme::help(ui, &app.t("detect-buildid-unknown"));
                        }
                    });
                }
                None => {
                    theme::card(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new(app.t("detect-not-found")).color(theme::WARN));
                        theme::help(ui, &app.t("detect-not-found-help"));
                    });
                }
            }

            ui.add_space(10.0);
            if let Some(report) = &detection.report {
                file_report(app, ui, report);
            }
        });

    ui.add_space(8.0);
    theme::rule(ui);
    ui.horizontal(|ui| {
        if game_path.is_some() && theme::primary_button(ui, &app.t("button-continue")).clicked() {
            prepare_destination(app);
            app.screen = Screen::Destination;
        }
        if ui.button(app.t("detect-pick-folder")).clicked() {
            pick_game_folder(app);
        }
    });
    theme::help(ui, &app.t("detect-pick-folder-help"));
}

fn build_line(app: &App, ui: &mut egui::Ui, buildid: Option<u64>) {
    match buildid {
        Some(found) if found == recof_core::consts::EXPECTED_BUILDID => {
            ui.label(
                egui::RichText::new(
                    app.t_args("detect-buildid-ok", &[("buildid", found.to_string())]),
                )
                .color(theme::GOOD),
            );
        }
        Some(found) => {
            ui.label(
                egui::RichText::new(app.t_args(
                    "detect-buildid-wrong",
                    &[
                        ("found", found.to_string()),
                        ("expected", recof_core::consts::EXPECTED_BUILDID.to_string()),
                    ],
                ))
                .color(theme::WARN),
            );
        }
        None => theme::help(ui, &app.t("detect-buildid-unknown")),
    }
}

/// The per file report, which never blocks but is always shown in full.
fn file_report(app: &App, ui: &mut egui::Ui, report: &recof_core::hashes::InstallReport) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("detect-check-title")).strong());
        ui.add_space(4.0);

        if report.is_untouched_retail() {
            ui.label(egui::RichText::new(app.t("detect-check-clean")).color(theme::GOOD));
            return;
        }

        theme::help(ui, &app.t("detect-check-notices"));
        ui.add_space(6.0);

        for file in report.notable() {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(&file.path)
                        .monospace()
                        .color(theme::TEXT),
                );
                let (text, colour) = match &file.verdict {
                    FileVerdict::Retail => (app.t("verdict-retail"), theme::GOOD),
                    FileVerdict::RetailPatched { .. } => {
                        (app.t("verdict-patched"), theme::TEXT_DIM)
                    }
                    FileVerdict::KnownVariant { message_id } => (app.t(message_id), theme::WARN),
                    FileVerdict::Unknown { sha256, .. } => (
                        app.t_args("verdict-unknown", &[("sha256", sha256.clone())]),
                        theme::WARN,
                    ),
                    FileVerdict::Missing => (app.t("verdict-missing"), theme::BAD),
                };
                ui.label(egui::RichText::new(text).color(colour).size(12.5));
            });
        }

        if !report.mod_markers.is_empty() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(app.t("detect-mods-title")).strong());
            for marker in &report.mod_markers {
                ui.label(egui::RichText::new(app.t(marker)).color(theme::WARN));
            }
        }

        if !report.unexpected_binaries.is_empty() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(app.t("detect-unexpected-title")).strong());
            theme::help(ui, &app.t("detect-unexpected-help"));
            for name in &report.unexpected_binaries {
                ui.label(egui::RichText::new(name).monospace().size(12.5));
            }
        }
    });
}

fn pick_game_folder(app: &mut App) {
    let Some(folder) = rfd::FileDialog::new()
        .set_title(app.t("detect-pick-folder"))
        .pick_folder()
    else {
        return;
    };

    match recof_core::steam::inspect_folder(&folder) {
        Ok(install) => {
            app.manual_game_path = Some(install.root.clone());
            match recof_core::hashes::scan_install(&install.root) {
                Ok(report) => {
                    if let Some(detection) = app.detection.as_mut() {
                        detection.game = Some(install);
                        detection.report = Some(report);
                    }
                }
                Err(error) => log::warn!("scanning the chosen folder: {error}"),
            }
            app.notice = None;
        }
        Err(error) => {
            let message = error.message_id();
            let args = error.message_args();
            app.show_error(message, &args);
        }
    }
}

/// Fills the destination box with the suggestion.
fn prepare_destination(app: &mut App) {
    if !app.destination.is_empty() {
        return;
    }
    let layout = app
        .detection
        .as_ref()
        .and_then(|detection| detection.steam.as_ref());
    let game = app.game_path();
    let home = home_dir();

    if let Some(path) = destination::suggest(layout, game.as_deref(), &home) {
        app.destination = path.display().to_string();
        app.request_measure();
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

/// Choosing where the copy goes, with the space check.
pub fn destination(app: &mut App, ui: &mut egui::Ui) {
    theme::heading(ui, &app.t("install-title"));

    ui.add_space(6.0);
    ui.label(egui::RichText::new(app.t("install-destination")).strong());
    theme::help(ui, &app.t("install-destination-help"));

    let mut changed = false;
    ui.horizontal(|ui| {
        let width = ui.available_width() - 120.0;
        let response = ui.add_sized(
            [width.max(200.0), 26.0],
            egui::TextEdit::singleline(&mut app.destination),
        );
        changed |= response.changed();

        if ui.button(app.t("button-browse")).clicked() {
            if let Some(folder) = rfd::FileDialog::new()
                .set_title(app.t("install-destination"))
                .pick_folder()
            {
                app.destination = folder
                    .join(destination::DEFAULT_FOLDER_NAME)
                    .display()
                    .to_string();
                changed = true;
            }
        }
    });

    if changed {
        app.measurement = None;
        app.measured_for = None;
    }

    let chosen = PathBuf::from(app.destination.trim());
    let layout = app
        .detection
        .as_ref()
        .and_then(|detection| detection.steam.as_ref());
    let game = app.game_path();
    let problem = if app.destination.trim().is_empty() {
        None
    } else {
        destination::validate(&chosen, layout, game.as_deref()).err()
    };

    ui.add_space(10.0);
    if let Some(problem) = &problem {
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(app.t(problem.message_id())).color(theme::BAD));
        });
    } else if let Some(measurement) = &app.measurement {
        let plan = &measurement.plan;
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(app.t_args(
                "install-size-line",
                &[
                    ("size", format_bytes(measurement.size.bytes_on_disk)),
                    ("volume", plan.destination_space.volume.clone()),
                    ("available", format_bytes(plan.destination_space.available)),
                ],
            ));
            theme::help(
                ui,
                &app.t_args(
                    "detect-size",
                    &[
                        ("files", measurement.size.files.to_string()),
                        ("size", format_bytes(measurement.size.bytes)),
                    ],
                ),
            );
            ui.add_space(4.0);
            match plan.shortfalls().first() {
                None => {
                    ui.label(egui::RichText::new(app.t("install-space-ok")).color(theme::GOOD));
                }
                Some(shortfall) => {
                    ui.label(
                        egui::RichText::new(app.t_args(
                            "install-space-short",
                            &[
                                ("volume", shortfall.volume.clone()),
                                ("missing", format_bytes(shortfall.missing())),
                            ],
                        ))
                        .color(theme::BAD),
                    );
                }
            }
            ui.add_space(4.0);
            theme::help(ui, &app.t("install-space-explain"));
        });
    } else if app.progress.is_some() {
        ui.horizontal(|ui| {
            ui.spinner();
            theme::help(ui, &app.t("detect-check-title"));
        });
    }

    ui.add_space(8.0);
    let import_label = app.t("install-offer-import");
    ui.checkbox(&mut app.import_on_install, import_label);
    let shortcut_label = app.t("install-offer-shortcuts");
    ui.checkbox(&mut app.shortcut_on_install, shortcut_label);

    ui.add_space(8.0);
    theme::rule(ui);

    let stored = app.newest_stored();
    let fits = app
        .measurement
        .as_ref()
        .map(|measurement| measurement.plan.fits())
        .unwrap_or(false);
    let ready = problem.is_none() && fits && stored.is_some() && !app.worker.is_busy();

    ui.horizontal(|ui| {
        if ui.button(app.t("button-back")).clicked() {
            app.screen = Screen::Detect;
        }

        ui.add_enabled_ui(ready, |ui| {
            if theme::primary_button(ui, &app.t("install-title")).clicked() {
                start_install(app);
            }
        });

        if app.measurement.is_none()
            && problem.is_none()
            && !app.worker.is_busy()
            && ui.button(app.t("button-retry")).clicked()
        {
            app.request_measure();
        }
    });

    if stored.is_none() {
        ui.add_space(6.0);
        theme::help(ui, &app.t("err-no-release"));
    }
}

fn start_install(app: &mut App) {
    let (Some(source), Some(release), Some(measured)) =
        (app.game_path(), app.newest_stored(), app.measured_size())
    else {
        return;
    };

    let id = format!("install-{}", app.settings.channel.as_str());
    app.progress = Some(Progress {
        headline: app.t("install-copying-counted"),
        ..Default::default()
    });
    app.screen = Screen::Installing;
    let import_from = if app.import_on_install {
        app.game_path()
    } else {
        None
    };
    app.worker.send(Job::Install {
        source,
        import_from,
        source_buildid: app.game_buildid(),
        destination: PathBuf::from(app.destination.trim()),
        measured,
        release: Box::new(release),
        now: crate::now_rfc3339(),
        id,
    });
}

/// The progress screen, with a cancel button that asks first.
pub fn installing(app: &mut App, ui: &mut egui::Ui) {
    theme::heading(ui, &app.t("install-title"));
    ui.add_space(16.0);

    let progress = app.progress.clone().unwrap_or_default();
    ui.label(egui::RichText::new(&progress.headline).size(15.0));
    ui.add_space(6.0);

    let bar = egui::ProgressBar::new(progress.fraction.unwrap_or(0.0))
        .desired_width(ui.available_width())
        .fill(theme::ACCENT)
        .animate(progress.fraction.is_none());
    ui.add(bar);

    ui.add_space(6.0);
    theme::help(ui, &progress.detail);

    ui.add_space(20.0);
    if ui.button(app.t("button-cancel")).clicked() {
        app.confirm = Some(Confirm::new(
            app.t("install-cancel-confirm"),
            app.t("button-yes"),
            ConfirmAction::CancelInstall,
        ));
    }
}
