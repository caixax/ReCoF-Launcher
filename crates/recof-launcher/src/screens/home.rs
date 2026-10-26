//! The screen the launcher opens on once something is installed.

use recof_core::launch::{self, CvarFile, CvarKind, LaunchProblem};
use recof_platform::DisplayMode;

use crate::app::{App, Notice, Screen};
use crate::theme;

pub fn home(app: &mut App, ui: &mut egui::Ui) {
    top_bar(app, ui);
    theme::rule(ui);

    match app.installed.clone() {
        Some(state) => {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(app.t_args(
                        "home-version-installed",
                        &[("version", state.version.to_string())],
                    ))
                    .size(16.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::help(
                        ui,
                        &app.t_args(
                            "home-channel",
                            &[("channel", app.t(channel_key(state.channel)))],
                        ),
                    );
                });
            });
        }
        None => {
            ui.label(egui::RichText::new(app.t("home-no-install")).color(theme::WARN));
            ui.add_space(8.0);
            if theme::primary_button(ui, &app.t("home-install-now")).clicked() {
                app.screen = Screen::Detect;
            }
            return;
        }
    }

    ui.add_space(14.0);
    play_buttons(app, ui);

    ui.add_space(14.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        launch_options(app, ui);
        ui.add_space(10.0);
        recof_options(app, ui);
    });
}

fn channel_key(channel: recof_core::manifest::Channel) -> &'static str {
    match channel {
        recof_core::manifest::Channel::Stable => "channel-stable",
        recof_core::manifest::Channel::Beta => "channel-beta",
    }
}

fn top_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(app.t("app-name"))
                .size(18.0)
                .color(theme::TEXT)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(app.t("settings-title")).clicked() {
                app.screen = Screen::Settings;
            }
            if ui.button(app.t("maintain-title")).clicked() {
                app.screen = Screen::Maintenance;
            }
            if ui.button(app.t("bug-title")).clicked() {
                app.screen = Screen::BugReport;
            }
        });
    });
}

fn play_buttons(app: &mut App, ui: &mut egui::Ui) {
    let install_root = app.settings.install_path.clone();

    ui.horizontal(|ui| {
        let enabled = install_root.is_some() && !app.worker.is_busy();
        ui.add_enabled_ui(enabled, |ui| {
            if theme::primary_button(ui, &app.t("home-play")).clicked() {
                if let Some(root) = install_root.clone() {
                    start_game(app, &root);
                }
            }
        });

        ui.add_space(8.0);
        if ui.button(app.t("home-play-vanilla")).clicked() {
            if let Err(error) = launch::play_vanilla() {
                let id = error.message_id();
                let args = error.message_args();
                app.show_error(id, &args);
            }
        }
    });
}

fn start_game(app: &mut App, install_root: &std::path::Path) {
    let problems = launch::check(install_root);

    // A hard problem stops the launch; Steam not running is reported and
    // the user decides.
    if let Some(problem) = problems.iter().find(|problem| !problem.is_only_a_warning()) {
        let id = problem.message_id();
        app.notice = Some(Notice::warn(
            app.t_args(id, &[("path", install_root.display().to_string())]),
        ));
        return;
    }

    if problems.contains(&LaunchProblem::SteamNotRunning) {
        app.notice = Some(Notice::warn(format!(
            "{} {}",
            app.t("launch-steam-not-running"),
            app.t("launch-steam-not-running-help")
        )));
        if let Err(error) = launch::open_steam() {
            log::warn!("Steam could not be opened: {error}");
        }
        return;
    }

    let cvars = cvar_args(app);
    match launch::play_recof(install_root, &app.settings.launch, &cvars) {
        Ok(pid) => {
            log::info!("ReCoF started as process {pid}");
            app.notice = Some(Notice::good(app.t("launch-starting")));
        }
        Err(error) => {
            log::error!("starting ReCoF: {error}");
            app.notice = Some(Notice::bad(app.t("launch-failed")));
        }
    }
}

/// The cvar values to pass, from the data file the installed release
/// shipped.
fn cvar_args(app: &App) -> Vec<launch::CvarArg> {
    match load_cvars(app) {
        Some(file) => file.to_args(&app.settings.launch.cvars),
        None => Vec::new(),
    }
}

/// The cvar file of the installed release, if it ships one.
pub fn load_cvars(app: &App) -> Option<CvarFile> {
    let state = app.installed.as_ref()?;
    let name = state.manifest.cvars_file()?;
    let stored = app
        .store
        .get(recof_core::manifest::Product::Recof, &state.version)
        .ok()??;

    let relative = recof_core::manifest::safe_relative_path(name)?;
    CvarFile::load(&stored.files_dir().join(relative))
        .map_err(|error| log::warn!("the cvar file could not be read: {error}"))
        .ok()
}

fn launch_options(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("launch-options-title")).strong());
        ui.add_space(6.0);

        let modes = launch::available_resolutions();
        let current = (app.settings.launch.width, app.settings.launch.height);
        let label = match current {
            (Some(width), Some(height)) => format!("{width} x {height}"),
            _ => app.t("launch-resolution-default"),
        };

        ui.horizontal(|ui| {
            ui.label(app.t("launch-resolution"));
            egui::ComboBox::from_id_salt("resolution")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(current.0.is_none(), app.t("launch-resolution-default"))
                        .clicked()
                    {
                        app.settings.launch.width = None;
                        app.settings.launch.height = None;
                        app.save_settings();
                    }
                    for mode in &modes {
                        let selected = current == (Some(mode.width), Some(mode.height));
                        if ui.selectable_label(selected, mode.to_string()).clicked() {
                            choose_resolution(app, *mode);
                        }
                    }
                });
        });

        // The labels are taken before the checkboxes because each one
        // borrows the settings mutably while the label reads them.
        let windowed_label = app.t("launch-windowed");
        let widescreen_label = app.t("launch-widescreen");
        let menu_label = app.t("launch-3dmenu");
        let menu_help = app.t("launch-3dmenu-help");
        let extra_label = app.t("launch-extra-args");
        let extra_help = app.t("launch-extra-args-help");

        let mut changed = false;
        changed |= ui
            .checkbox(&mut app.settings.launch.windowed, windowed_label)
            .changed();
        changed |= ui
            .checkbox(&mut app.settings.launch.widescreen, widescreen_label)
            .changed();
        let menu = ui.checkbox(&mut app.settings.launch.three_d_menu, menu_label);
        changed |= menu.changed();
        if menu.hovered() {
            theme::help(ui, &menu_help);
        }

        ui.add_space(6.0);
        ui.label(extra_label);
        changed |= ui
            .add_sized(
                [ui.available_width(), 24.0],
                egui::TextEdit::singleline(&mut app.settings.launch.extra_args),
            )
            .changed();
        theme::help(ui, &extra_help);

        if changed {
            app.save_settings();
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new(app.t("launch-preview")).strong());
        let args = launch::build(&app.settings.launch, &cvar_args(app));
        let exe = app
            .settings
            .install_path
            .as_ref()
            .map(|root| {
                root.join(recof_core::consts::GAME_EXE)
                    .display()
                    .to_string()
            })
            .unwrap_or_else(|| recof_core::consts::GAME_EXE.to_owned());
        ui.label(
            egui::RichText::new(launch::preview(&exe, &args))
                .monospace()
                .size(11.5)
                .color(theme::TEXT_DIM),
        );
    });
}

fn choose_resolution(app: &mut App, mode: DisplayMode) {
    app.settings.launch.width = Some(mode.width);
    app.settings.launch.height = Some(mode.height);
    // The widescreen flag follows the shape of the mode, which is what it
    // meant in the retail launcher, but the user can still override it.
    app.settings.launch.widescreen = mode.is_widescreen();
    app.save_settings();
}

fn recof_options(app: &mut App, ui: &mut egui::Ui) {
    let Some(file) = load_cvars(app) else {
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(app.t("launch-cvars-title")).strong());
            theme::help(ui, &app.t("launch-cvars-none"));
        });
        return;
    };

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("launch-cvars-title")).strong());
        theme::help(ui, &app.t("launch-cvars-from-release"));
        ui.add_space(6.0);

        let mut changed = false;
        for group in file.groups() {
            if !group.is_empty() {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(group).color(theme::TEXT_DIM).strong());
            }
            for cvar in file.cvars.iter().filter(|cvar| cvar.group == group) {
                changed |= one_cvar(app, ui, cvar);
            }
        }
        if changed {
            app.save_settings();
        }
    });
}

/// Draws one option and returns whether it changed.
fn one_cvar(app: &mut App, ui: &mut egui::Ui, cvar: &recof_core::launch::Cvar) -> bool {
    // A label the launcher has a translation for wins; otherwise the
    // English text the release shipped is used, so an option added after
    // this launcher was built is still readable.
    let label = match &cvar.key {
        Some(key) if app.translator.has(key) => app.t(key),
        _ => cvar.label_en.clone(),
    };
    let description = match &cvar.key {
        Some(key) => app
            .translator
            .attribute(key, "desc")
            .unwrap_or_else(|| cvar.desc_en.clone()),
        None => cvar.desc_en.clone(),
    };

    let current = app
        .settings
        .launch
        .cvars
        .get(&cvar.name)
        .cloned()
        .unwrap_or_else(|| cvar.default.clone());
    let mut changed = false;

    match cvar.kind {
        CvarKind::Bool => {
            let mut on = current == "1";
            if ui.checkbox(&mut on, label).changed() {
                app.settings
                    .launch
                    .cvars
                    .insert(cvar.name.clone(), if on { "1" } else { "0" }.to_owned());
                changed = true;
            }
        }
        CvarKind::Int | CvarKind::Float => {
            ui.horizontal(|ui| {
                ui.label(label);
                let mut text = current.clone();
                if ui
                    .add_sized([90.0, 22.0], egui::TextEdit::singleline(&mut text))
                    .changed()
                    && cvar.accepts(&text)
                {
                    app.settings.launch.cvars.insert(cvar.name.clone(), text);
                    changed = true;
                }
                if let (Some(min), Some(max)) = (cvar.min, cvar.max) {
                    theme::help(ui, &format!("{min} to {max}"));
                }
            });
        }
        CvarKind::Text => {
            ui.horizontal(|ui| {
                ui.label(label);
                let mut text = current.clone();
                if ui
                    .add_sized([180.0, 22.0], egui::TextEdit::singleline(&mut text))
                    .changed()
                    && cvar.accepts(&text)
                {
                    app.settings.launch.cvars.insert(cvar.name.clone(), text);
                    changed = true;
                }
            });
        }
    }

    if !description.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            theme::help(ui, &description);
        });
    }
    changed
}
