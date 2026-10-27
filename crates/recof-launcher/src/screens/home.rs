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
fn actions(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let install_root = app.settings.install_path.clone();
            let enabled = install_root.is_some() && !app.worker.is_busy();

            if play_button(ui, &app.t("home-play"), enabled).clicked() {
                if let Some(root) = install_root {
                    let ctx = ui.ctx().clone();
                    start_game(app, &ctx, &root);
                }
            }

            ui.add_space(8.0);
            more_menu(app, ui);

            // Whatever is left of the row belongs to the update line,
            // which reads from the left like the rest of the screen.
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                crate::screens::updates::status_line(app, ui);
            });
        });
    });
}

/// The small menu beside the play button, for the things that are not
/// the thing the screen is for.
fn more_menu(app: &mut App, ui: &mut egui::Ui) {
    let tip = app.t("home-more");
    let button = square_button(ui, Icon::Menu).on_hover_text(tip);

    let vanilla = app.t("home-play-vanilla");
    let options = app.t("launch-options-title");
    let mut play_vanilla = false;
    let mut open_options = false;

    egui::Popup::menu(&button).show(|ui| {
        ui.set_min_width(190.0);
        play_vanilla = ui.button(vanilla).clicked();
        open_options = ui.button(options).clicked();
    });

    if open_options {
        app.screen = Screen::LaunchOptions;
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
/// release lets the user change.
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

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        crate::screens::updates::restart_prompt(app, ui);
        crate::screens::updates::banner(app, ui);
        launch_options(app, ui);
        ui.add_space(10.0);
        recof_options(app, ui);
    });
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
        if !app.settings.look.show_command_line {
            return;
        }
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

        // What the game saved, so an option the user never set in the
        // launcher shows what the game will really use.
        let saved = app.saved_cvars();
        let mut changed = false;
        for group in file.groups() {
            if !group.is_empty() {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(group).color(theme::TEXT_DIM).strong());
            }
            for cvar in file.cvars.iter().filter(|cvar| cvar.group == group) {
                changed |= one_cvar(app, ui, cvar, &saved);
            }
        }

        ui.add_space(6.0);
        if ui.button(app.t("launch-cvars-defaults")).clicked() {
            app.settings.launch.cvars.extend(file.defaults());
            changed = true;
        }
        if changed {
            app.save_settings();
        }
    });
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

    if !description.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            theme::help(ui, &description);
        });
    }
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
