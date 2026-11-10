//! The screen the launcher opens on once something is installed.
//!
//! The hero fills the window, a rail of icons runs down the left, and
//! what the user came for sits on top: what is installed, how long it
//! has been played, and one large button that starts it. Everything
//! else is a click away rather than on this screen, because a launcher
//! is read at a glance and then clicked.

use recof_core::launch::{self, CvarFile, CvarKind, LaunchProblem};
use recof_platform::DisplayMode;

use crate::app::{App, Confirm, ConfirmAction, Notice, Screen};
use crate::icon::Icon;
use crate::theme;

/// Width of the icon rail down the left.
const RAIL: f32 = 54.0;

/// Height reserved for the row the play button sits in.
const ACTIONS: f32 = 52.0;

pub fn home(app: &mut App, ui: &mut egui::Ui) {
    // Nothing installed yet is a different screen entirely: there is no
    // version to show, nothing has been played, and the only useful
    // thing to offer is the way in.
    if app.installed.is_none() {
        return nothing_installed(app, ui);
    }

    let full = ui.available_rect_before_wrap();
    ui.horizontal_top(|ui| {
        rail(app, ui, full.height());
        ui.add_space(18.0);
        ui.vertical(|ui| {
            ui.set_min_width(ui.available_width());
            main_column(app, ui, full);
        });
    });
}

fn nothing_installed(app: &mut App, ui: &mut egui::Ui) {
    let width = ui.available_width();
    let logo = app.wordmark().cloned();
    crate::brand::wordmark(ui, logo.as_ref(), width);

    ui.add_space(16.0);
    ui.label(egui::RichText::new(app.t("home-no-install")).color(theme::WARN));
    ui.add_space(10.0);
    if theme::primary_button(ui, &app.t("home-install-now")).clicked() {
        app.screen = Screen::Detect;
    }
}

/// The rail of icons down the left side.
///
/// Icons rather than words because the labels are eleven different
/// lengths in eleven languages, and a rail that changes width with the
/// language would move the whole screen around. Each one says what it
/// is on hover.
fn rail(app: &mut App, ui: &mut egui::Ui, height: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(RAIL, height),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.set_min_height(height);

            let entries = [
                (Icon::Home, "nav-home", Screen::Home),
                (Icon::Sliders, "nav-options", Screen::LaunchOptions),
                (Icon::Shield, "nav-maintain", Screen::Maintenance),
                (Icon::Bubble, "nav-bug", Screen::BugReport),
            ];
            for (glyph, key, screen) in entries {
                let tip = app.t(key);
                if rail_button(ui, glyph, app.screen == screen)
                    .on_hover_text(tip)
                    .clicked()
                {
                    app.screen = screen;
                }
            }

            // The two that do something rather than go somewhere sit at
            // the bottom, away from the navigation.
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let tip = app.t("nav-settings");
                if rail_button(ui, Icon::Gear, app.screen == Screen::Settings)
                    .on_hover_text(tip)
                    .clicked()
                {
                    app.screen = Screen::Settings;
                }

                let folder = app.settings.install_path.clone();
                let tip = app.t("nav-folder");
                let opened = rail_button(ui, Icon::Folder, false).on_hover_text(tip);
                if opened.clicked() {
                    if let Some(root) = folder {
                        if let Err(error) =
                            recof_platform::open_in_shell(&root.display().to_string())
                        {
                            log::warn!("the install folder could not be opened: {error}");
                        }
                    }
                }
            });
        },
    );
}

fn rail_button(ui: &mut egui::Ui, glyph: Icon, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::click());
    let painter = ui.painter();

    if selected || response.hovered() {
        painter.rect_filled(
            rect,
            egui::CornerRadius::same(8),
            if selected {
                theme::SURFACE_RAISED
            } else {
                theme::SURFACE
            },
        );
    }
    let colour = if selected || response.hovered() {
        theme::TEXT
    } else {
        theme::TEXT_DIM
    };
    glyph.paint(painter, rect.shrink(9.0), colour);

    // A short bar on the inner edge, which is how the eye finds the
    // current screen without reading anything.
    if selected {
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 1.0, rect.center().y - 9.0),
            egui::vec2(2.5, 18.0),
        );
        painter.rect_filled(bar, egui::CornerRadius::same(1), theme::ACCENT);
    }
    response
}

/// Everything to the right of the rail.
fn main_column(app: &mut App, ui: &mut egui::Ui, full: egui::Rect) {
    let width = ui.available_width();
    let logo = app.wordmark().cloned();
    crate::brand::wordmark(ui, logo.as_ref(), width.min(460.0));

    ui.add_space(22.0);
    stats(app, ui);

    // The actions sit at the bottom of the window rather than under the
    // stats, so they do not move when a stat gains or loses a line.
    let used = ui.min_rect().height();
    let gap = full.height() - used - ACTIONS;
    ui.add_space(gap.max(18.0));

    actions(app, ui);
}

/// The rows of what the launcher knows, one icon and two lines each.
fn stats(app: &mut App, ui: &mut egui::Ui) {
    let last = last_played_text(app);
    let played = playtime_text(app);
    let version = version_text(app);

    stat_row(ui, Icon::Play, &app.t("home-stat-last-played"), &last);
    ui.add_space(14.0);
    stat_row(ui, Icon::Clock, &app.t("home-stat-playtime"), &played);
    ui.add_space(14.0);
    stat_row(ui, Icon::Layers, &app.t("home-stat-version"), &version);
}

fn stat_row(ui: &mut egui::Ui, glyph: Icon, label: &str, value: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 34.0), egui::Sense::hover());
        glyph.paint(ui.painter(), rect, theme::TEXT_DIM);

        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(
                egui::RichText::new(label.to_uppercase())
                    .size(10.5)
                    .color(theme::TEXT_DIM),
            );
            ui.label(
                egui::RichText::new(value)
                    .size(17.0)
                    .color(theme::TEXT)
                    .strong(),
            );
        });
    });
}

/// When it was last played, in words rather than a date.
fn last_played_text(app: &App) -> String {
    match app.playtime.days_since_last() {
        None => app.t("home-played-never"),
        Some(0) => app.t("home-played-today"),
        Some(1) => app.t("home-played-yesterday"),
        Some(days) => app.t_args("home-played-days", &[("days", days.to_string())]),
    }
}

/// How long it has been played, to the minute.
fn playtime_text(app: &App) -> String {
    let (hours, minutes) = app.playtime.hours_and_minutes();
    if app.playtime.seconds == 0 {
        return app.t("home-playtime-none");
    }
    if hours == 0 {
        return app.t_args("home-playtime-minutes", &[("minutes", minutes.to_string())]);
    }
    app.t_args(
        "home-playtime-hours",
        &[
            ("hours", hours.to_string()),
            ("minutes", minutes.to_string()),
        ],
    )
}

/// Which version is installed, and on which channel.
fn version_text(app: &App) -> String {
    match app.installed.as_ref() {
        Some(state) => format!(
            "{}  {}",
            state.version,
            app.t(channel_key(state.channel)).to_lowercase()
        ),
        None => String::new(),
    }
}

/// The bottom row: what the update check found on the left, and the
/// buttons that start the game on the right.
///
/// When a ReCoF update is out, the play button becomes the update button,
/// so the update cannot be missed, and playing the installed version is
/// still in the menu beside it.
fn actions(app: &mut App, ui: &mut egui::Ui) {
    let update = crate::screens::updates::recof_update(app)
        .filter(|(_, needs_newer)| !needs_newer)
        .map(|(available, _)| available);

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let install_root = app.settings.install_path.clone();
            let enabled = install_root.is_some() && !app.worker.is_busy();

            match &update {
                Some(available) => {
                    let label = app.t_args(
                        "home-update-to",
                        &[("version", available.version.to_string())],
                    );
                    if play_button(ui, &label, enabled).clicked() {
                        crate::screens::updates::start_recof_update(app, available);
                    }
                }
                None => {
                    if play_button(ui, &app.t("home-play"), enabled).clicked() {
                        if let Some(root) = install_root {
                            let ctx = ui.ctx().clone();
                            start_game(app, &ctx, &root);
                        }
                    }
                }
            }

            ui.add_space(8.0);
            more_menu(app, ui, update.is_some());

            // Whatever is left of the row belongs to the update line, or
            // to the progress while an update runs, which read from the
            // left like the rest of the screen.
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                if app.progress.is_some() {
                    crate::screens::updates::progress_compact(app, ui);
                } else {
                    crate::screens::updates::home_line(app, ui);
                }
            });
        });
    });
}

/// The small menu beside the play button, for the things that are not
/// the thing the screen is for.
fn more_menu(app: &mut App, ui: &mut egui::Ui, offering_update: bool) {
    let tip = app.t("home-more");
    let button = square_button(ui, Icon::Menu).on_hover_text(tip);

    let without_update = app.t("home-play-without-update");
    let vanilla = app.t("home-play-vanilla");
    let options = app.t("launch-options-title");
    let check = app.t("updates-check-now");
    let busy = app.update_checking;
    let can_play = !app.worker.is_busy();
    // A launcher update the user skipped, or a restart put off, stays one
    // click away.
    let launcher_update = app
        .updates
        .launcher
        .as_ref()
        .filter(|_| !app.launcher_popup)
        .map(|available| {
            app.t_args(
                "home-launcher-update-short",
                &[("version", available.version.to_string())],
            )
        });
    let restart = (app.self_update_ready.is_some() && !app.restart_popup)
        .then(|| app.t("update-restart-now"));
    let mut open_launcher_update = false;
    let mut open_restart = false;
    let mut play_installed = false;
    let mut play_vanilla = false;
    let mut open_options = false;
    let mut check_updates = false;

    egui::Popup::menu(&button).show(|ui| {
        ui.set_min_width(200.0);
        if offering_update {
            ui.add_enabled_ui(can_play, |ui| {
                play_installed = ui.button(without_update).clicked();
            });
        }
        play_vanilla = ui.button(vanilla).clicked();
        open_options = ui.button(options).clicked();
        ui.add_enabled_ui(!busy, |ui| {
            check_updates = ui.button(check).clicked();
        });
        if let Some(label) = launcher_update {
            open_launcher_update = ui.button(label).clicked();
        }
        if let Some(label) = restart {
            open_restart = ui.button(label).clicked();
        }
    });

    if open_launcher_update {
        app.launcher_popup = true;
    }
    if open_restart {
        app.restart_popup = true;
    }

    if play_installed {
        if let Some(root) = app.settings.install_path.clone() {
            let ctx = ui.ctx().clone();
            start_game(app, &ctx, &root);
        }
    }
    if open_options {
        app.screen = Screen::LaunchOptions;
    }
    if check_updates {
        crate::screens::updates::check_now(app);
    }
    if play_vanilla {
        if let Err(error) = launch::play_vanilla() {
            let id = error.message_id();
            let args = error.message_args();
            app.show_error(id, &args);
        }
    }
}

/// The one button the screen is built around.
fn play_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    const HEIGHT: f32 = 46.0;
    /// Room for the triangle on the left and the padding on both sides.
    const AROUND: f32 = 82.0;
    /// Never narrower than this, so a short label still reads as the
    /// main button rather than as one more small one.
    const NARROWEST: f32 = 176.0;

    let text = label.to_uppercase();
    let font = egui::FontId::new(17.0, egui::FontFamily::Name(crate::brand::DISPLAY.into()));
    // Measured rather than assumed: the label is a different length in
    // every language, and a fixed width clipped the longer ones.
    let text_width = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), theme::TEXT)
        .size()
        .x;
    let width = (text_width + AROUND).max(NARROWEST);
    let size = egui::vec2(width.min(ui.available_width().max(NARROWEST)), HEIGHT);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let painter = ui.painter();

    let fill = match (enabled, response.hovered()) {
        (false, _) => theme::SURFACE,
        (true, true) => theme::ACCENT_HOVER,
        (true, false) => theme::ACCENT,
    };
    painter.rect_filled(rect, egui::CornerRadius::same(6), fill);

    let colour = if enabled {
        theme::TEXT
    } else {
        theme::TEXT_DIM
    };
    let glyph = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 34.0, rect.center().y),
        egui::Vec2::splat(16.0),
    );
    Icon::Play.paint(painter, glyph, colour);
    painter.text(
        egui::pos2(rect.left() + 58.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        colour,
    );
    response
}

/// A small square button beside the play button.
fn square_button(ui: &mut egui::Ui, glyph: Icon) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(46.0, 46.0), egui::Sense::click());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        egui::CornerRadius::same(6),
        if response.hovered() {
            theme::SURFACE_RAISED
        } else {
            theme::SURFACE
        },
    );
    glyph.paint(painter, rect.shrink(14.0), theme::TEXT);
    response
}

fn channel_key(channel: recof_core::manifest::Channel) -> &'static str {
    match channel {
        recof_core::manifest::Channel::Stable => "channel-stable",
        recof_core::manifest::Channel::Beta => "channel-beta",
    }
}

fn start_game(app: &mut App, ctx: &egui::Context, install_root: &std::path::Path) {
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

    // Steam not running is the user's call, not the launcher's: the
    // game works without it, it just loses what Steam provides.
    if problems.contains(&LaunchProblem::SteamNotRunning) {
        app.confirm = Some(
            Confirm::new(
                format!(
                    "{} {}",
                    app.t("launch-steam-not-running"),
                    app.t("launch-steam-not-running-help")
                ),
                app.t("launch-open-steam"),
                ConfirmAction::OpenSteam,
            )
            .or(app.t("launch-start-anyway"), ConfirmAction::LaunchAnyway),
        );
        return;
    }

    start_game_now(app, ctx, install_root);
}

/// Starts the game without asking anything further.
///
/// Reached from the play button once the checks have passed, and from
/// the "start anyway" answer when Steam is not running.
pub fn start_game_now(app: &mut App, ctx: &egui::Context, install_root: &std::path::Path) {
    let cvars = cvar_args(app);
    match launch::play_recof(install_root, &app.settings.launch, &cvars) {
        Ok(pid) => {
            log::info!("ReCoF started as process {pid}");
            app.notice = Some(Notice::good(app.t("launch-starting")));
            // Only once it really started: a launcher that closes on a
            // failed launch takes the error message with it.
            app.step_aside(ctx);
        }
        Err(error @ recof_core::Error::CommandLineTooLong { .. }) => {
            log::error!("starting ReCoF: {error}");
            app.show_error(error.message_id(), &error.message_args());
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

/// Everything that used to sit under the play button, on a screen of
/// its own: resolution, the retail switches, and whatever the installed
/// release lets the user change, by section.
pub fn launch_screen(app: &mut App, ui: &mut egui::Ui) {
    let title = app.t("launch-options-title");
    ui.horizontal(|ui| {
        if ui.button(app.t("button-back")).clicked() {
            app.screen = Screen::Home;
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(title).size(18.0).strong());
    });
    theme::rule(ui);
    options(app, ui);
}

const SECTION_DISPLAY: &str = "display";
const SECTION_ADVANCED: &str = "advanced";

/// The sections of the options, by id: the display settings, one per
/// group of the release's cvar file, and the advanced ones last.
fn sections(app: &App, file: Option<&CvarFile>) -> Vec<(String, String)> {
    let mut sections = vec![(SECTION_DISPLAY.to_owned(), app.t("options-section-display"))];
    if let Some(file) = file {
        for group in file.groups() {
            sections.push((format!("group:{group}"), group_label(app, group)));
        }
    }
    sections.push((
        SECTION_ADVANCED.to_owned(),
        app.t("options-section-advanced"),
    ));
    sections
}

/// A group name from the cvar file in the user's language when the
/// launcher has it, else as the release wrote it.
fn group_label(app: &App, group: &str) -> String {
    if group.is_empty() {
        return app.t("options-group-other");
    }
    let key = format!("options-group-{}", group.to_lowercase());
    if app.translator.has(&key) {
        app.t(&key)
    } else {
        group.to_owned()
    }
}

/// The sections on the left and the chosen one on the right.
fn options(app: &mut App, ui: &mut egui::Ui) {
    let file = load_cvars(app);
    let sections = sections(app, file.as_ref());
    if !sections.iter().any(|(id, _)| *id == app.options_section) {
        app.options_section = SECTION_DISPLAY.to_owned();
    }

    ui.horizontal_top(|ui| {
        // The list scrolls on its own when the window is too short for
        // every section.
        ui.allocate_ui_with_layout(
            egui::vec2(180.0, ui.available_height()),
            egui::Layout::top_down_justified(egui::Align::LEFT),
            |ui| {
                ui.set_width(180.0);
                egui::ScrollArea::vertical()
                    .id_salt("options-sections")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        for (id, label) in &sections {
                            let selected = app.options_section == *id;
                            if ui.selectable_label(selected, label.as_str()).clicked() {
                                app.options_section = id.clone();
                            }
                        }
                    });
            },
        );

        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("options-section")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // The scroll area takes the row's layout; the options read
                // top to bottom.
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    let section = app.options_section.clone();
                    let label = sections
                        .iter()
                        .find(|(id, _)| *id == section)
                        .map(|(_, label)| label.clone())
                        .unwrap_or_default();
                    ui.label(egui::RichText::new(label).size(16.0).strong());
                    ui.add_space(8.0);

                    if section == SECTION_DISPLAY {
                        display_options(app, ui);
                    } else if section == SECTION_ADVANCED {
                        advanced_options(app, ui, file.as_ref());
                    } else if let (Some(group), Some(file)) =
                        (section.strip_prefix("group:"), file.as_ref())
                    {
                        group_options(app, ui, file, group);
                    }
                });
            });
    });
}

/// Resolution, window and menu.
fn display_options(app: &mut App, ui: &mut egui::Ui) {
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
    ui.add_space(4.0);

    // The labels are taken before the checkboxes because each one
    // borrows the settings mutably while the label reads them.
    let windowed_label = app.t("launch-windowed");
    let widescreen_label = app.t("launch-widescreen");
    let menu_label = app.t("launch-3dmenu");
    let menu_help = app.t("launch-3dmenu-help");

    let mut changed = false;
    changed |= ui
        .checkbox(&mut app.settings.launch.windowed, windowed_label)
        .changed();
    changed |= ui
        .checkbox(&mut app.settings.launch.widescreen, widescreen_label)
        .changed();
    changed |= ui
        .checkbox(&mut app.settings.launch.three_d_menu, menu_label)
        .changed();
    description(ui, &menu_help);

    if changed {
        app.save_settings();
    }
}

/// The options of one group of the cvar file.
fn group_options(app: &mut App, ui: &mut egui::Ui, file: &CvarFile, group: &str) {
    // What the game saved, so an option the user never set in the
    // launcher shows what the game will really use.
    let saved = app.saved_cvars();
    let mut changed = false;
    for cvar in file.cvars.iter().filter(|cvar| cvar.group == group) {
        changed |= one_cvar(app, ui, cvar, &saved);
        ui.add_space(4.0);
    }
    if changed {
        app.save_settings();
    }
}

/// Extra arguments, the command line, and the way back to the defaults.
fn advanced_options(app: &mut App, ui: &mut egui::Ui, file: Option<&CvarFile>) {
    let extra_label = app.t("launch-extra-args");
    let extra_help = app.t("launch-extra-args-help");

    ui.label(extra_label);
    if ui
        .add_sized(
            [ui.available_width(), 24.0],
            egui::TextEdit::singleline(&mut app.settings.launch.extra_args),
        )
        .changed()
    {
        app.save_settings();
    }
    description(ui, &extra_help);

    if app.settings.look.show_command_line {
        ui.add_space(10.0);
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
        let line = launch::preview(&exe, &args);
        let too_long = line.len() > launch::MAX_COMMAND_LINE;
        ui.label(
            egui::RichText::new(&line)
                .monospace()
                .size(11.5)
                .color(theme::TEXT_DIM),
        );
        if too_long {
            ui.label(
                egui::RichText::new(app.t_args(
                    "err-command-line-too-long",
                    &[
                        ("length", line.len().to_string()),
                        ("limit", launch::MAX_COMMAND_LINE.to_string()),
                    ],
                ))
                .color(theme::BAD),
            );
        }
    }

    ui.add_space(10.0);
    match file {
        Some(file) => {
            theme::help(ui, &app.t("launch-cvars-from-release"));
            if ui.button(app.t("launch-cvars-defaults")).clicked() {
                app.settings.launch.cvars.extend(file.defaults());
                app.save_settings();
            }
        }
        None => theme::help(ui, &app.t("launch-cvars-none")),
    }
}

/// The explanation under an option, indented under its label.
fn description(ui: &mut egui::Ui, text: &str) {
    if text.is_empty() {
        return;
    }
    ui.horizontal(|ui| {
        ui.add_space(24.0);
        ui.add(
            egui::Label::new(egui::RichText::new(text).size(12.5).color(theme::TEXT_DIM)).wrap(),
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

/// Draws one option and returns whether it changed.
fn one_cvar(
    app: &mut App,
    ui: &mut egui::Ui,
    cvar: &recof_core::launch::Cvar,
    saved: &std::collections::BTreeMap<String, String>,
) -> bool {
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

    let current = cvar.current(&app.settings.launch.cvars, saved).to_owned();
    let mut changed = false;

    match cvar.kind {
        CvarKind::Bool | CvarKind::Flag => {
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

    self::description(ui, &description);
    changed
}

#[cfg(test)]
mod tests {
    use recof_core::playtime::Playtime;

    /// The wording the stat row picks, without needing an App.
    fn bucket(played: &Playtime) -> &'static str {
        match played.days_since_last() {
            None => "never",
            Some(0) => "today",
            Some(1) => "yesterday",
            Some(_) => "days",
        }
    }

    fn seconds_ago(seconds: u64) -> Playtime {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Playtime {
            last_played: Some(now - seconds),
            ..Default::default()
        }
    }

    #[test]
    fn a_copy_nobody_has_opened_says_never_rather_than_a_date() {
        assert_eq!(bucket(&Playtime::default()), "never");
    }

    #[test]
    fn the_last_launch_reads_in_words() {
        assert_eq!(bucket(&seconds_ago(60)), "today");
        assert_eq!(bucket(&seconds_ago(86_400 + 60)), "yesterday");
        assert_eq!(bucket(&seconds_ago(86_400 * 9)), "days");
    }

    #[test]
    fn an_hour_boundary_does_not_read_as_sixty_minutes() {
        // "1 h 0 min" is right and "0 h 60 min" is not, which is the
        // whole reason the split is done in one place.
        let played = Playtime {
            seconds: 3_600,
            ..Default::default()
        };
        assert_eq!(played.hours_and_minutes(), (1, 0));
    }
}
