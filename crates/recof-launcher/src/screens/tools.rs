//! Settings, maintenance and the bug report.

use recof_core::backup::BackupKind;
use recof_core::install::{RepairReport, VerifyEntry};
use recof_core::manifest::Channel;
use recof_core::space::format_bytes;

use crate::app::{App, Confirm, ConfirmAction, Notice, Progress, Screen};
use crate::i18n::LANGUAGES;
use crate::theme;
use crate::worker::{Job, ShortcutKind};

/// The verify report is kept here rather than on `App` so the rest of the
/// application does not have to know about a maintenance detail.
static LAST_VERIFY: std::sync::Mutex<Option<RepairReport>> = std::sync::Mutex::new(None);

pub fn store_verify_report(_app: &mut App, report: RepairReport) {
    if let Ok(mut slot) = LAST_VERIFY.lock() {
        *slot = Some(report);
    }
}

fn last_verify() -> Option<RepairReport> {
    LAST_VERIFY.lock().ok().and_then(|slot| slot.clone())
}

fn back_bar(app: &mut App, ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        if ui.button(app.t("button-back")).clicked() {
            app.screen = Screen::Home;
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(title).size(18.0).strong());
    });
    theme::rule(ui);
}

/// Which part of the settings is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Look,
    Language,
    Updates,
    Behaviour,
    About,
}

impl SettingsTab {
    /// The tabs in the order they are shown.
    pub const ALL: [SettingsTab; 5] = [
        SettingsTab::Look,
        SettingsTab::Language,
        SettingsTab::Updates,
        SettingsTab::Behaviour,
        SettingsTab::About,
    ];

    /// The message id for the tab's name.
    pub fn key(self) -> &'static str {
        match self {
            SettingsTab::Look => "settings-tab-look",
            SettingsTab::Language => "settings-tab-language",
            SettingsTab::Updates => "settings-tab-updates",
            SettingsTab::Behaviour => "settings-tab-behaviour",
            SettingsTab::About => "settings-tab-about",
        }
    }
}

pub fn settings(app: &mut App, ui: &mut egui::Ui) {
    let title = app.t("settings-title");
    back_bar(app, ui, &title);

    ui.horizontal_wrapped(|ui| {
        for tab in SettingsTab::ALL {
            let label = app.t(tab.key());
            if ui
                .selectable_label(app.settings_tab == tab, label)
                .clicked()
            {
                app.settings_tab = tab;
            }
        }
    });
    theme::rule(ui);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        match app.settings_tab {
            SettingsTab::Look => look_tab(app, ui),
            SettingsTab::Language => language_tab(app, ui),
            SettingsTab::Updates => updates_tab(app, ui),
            SettingsTab::Behaviour => behaviour_tab(app, ui),
            SettingsTab::About => about_tab(app, ui),
        }
    });
}

fn look_tab(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("look-title")).strong());
        ui.add_space(6.0);

        // Labels first: each control borrows the settings mutably.
        let hero_label = app.t("look-show-hero");
        let dim_label = app.t("look-hero-dim");
        let dim_help = app.t("look-hero-dim-help");
        let grain_label = app.t("look-grain");
        let line_label = app.t("look-command-line");
        let mut changed = false;

        changed |= ui
            .checkbox(&mut app.settings.look.show_hero, hero_label)
            .changed();

        ui.add_enabled_ui(app.settings.look.show_hero, |ui| {
            ui.horizontal(|ui| {
                ui.label(dim_label);
                changed |= ui
                    .add(
                        egui::Slider::new(&mut app.settings.look.hero_dim, 0.0..=0.95)
                            .show_value(false),
                    )
                    .changed();
            });
            theme::help(ui, &dim_help);
        });

        changed |= ui
            .checkbox(&mut app.settings.look.grain, grain_label)
            .changed();
        changed |= ui
            .checkbox(&mut app.settings.look.show_command_line, line_label)
            .changed();

        if changed {
            app.save_settings();
        }
    });

    ui.add_space(10.0);
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("look-artwork-title")).strong());
        theme::help(ui, &app.t("look-artwork-help"));
        ui.add_space(6.0);

        // What is actually in use, so the folder is never a guess.
        let mut pieces = Vec::new();
        if app.artwork.hero().is_some() {
            pieces.push("hero");
        }
        if app.artwork.logo().is_some() {
            pieces.push("logo");
        }
        if app.paths.theme_dir().join("icon.png").is_file() {
            pieces.push("icon");
        }

        if pieces.is_empty() {
            theme::help(ui, &app.t("look-no-art"));
        } else {
            ui.label(
                egui::RichText::new(app.t_args("look-using-art", &[("pieces", pieces.join(", "))]))
                    .color(theme::GOOD),
            );
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button(app.t("look-open-theme")).clicked() {
                let folder = app.paths.theme_dir().display().to_string();
                if let Err(error) = recof_platform::open_in_shell(&folder) {
                    log::warn!("the theme folder could not be opened: {error}");
                }
            }
            if ui.button(app.t("look-reload-art")).clicked() {
                let ctx = ui.ctx().clone();
                app.reload_artwork(&ctx);
            }
            // Only offered for artwork the launcher fetched. A picture
            // the user put there by hand is theirs to remove.
            if recof_core::artwork::source(&app.paths.theme_dir())
                == recof_core::artwork::Source::Release
                && ui.button(app.t("look-forget-art")).clicked()
            {
                if let Err(error) = recof_core::artwork::clear(&app.paths.theme_dir()) {
                    log::warn!("the artwork could not be removed: {error}");
                }
                let ctx = ui.ctx().clone();
                app.reload_artwork(&ctx);
                app.artwork_credits = None;
            }
        });
    });

    ui.add_space(10.0);
    credits_card(app, ui);
}

/// Who made the pictures, which is the whole reason the launcher ships
/// none of its own: the art belongs to whoever drew it.
fn credits_card(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("look-credits-title")).strong());
        theme::help(ui, &app.t("look-credits-help"));
        ui.add_space(6.0);

        match &app.artwork_credits {
            Some(credits) => {
                for line in credits.lines().filter(|line| !line.trim().is_empty()) {
                    ui.label(egui::RichText::new(line).size(12.5).color(theme::TEXT));
                }
            }
            None => theme::help(ui, &app.t("look-credits-none")),
        }
    });
}

fn language_tab(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("settings-language")).strong());
        ui.add_space(4.0);

        let current = app.translator.language();
        let mut chosen = None;
        egui::ComboBox::from_id_salt("language")
            .selected_text(current.native_name)
            .show_ui(ui, |ui| {
                for language in LANGUAGES {
                    if ui
                        .selectable_label(language.tag == current.tag, language.native_name)
                        .clicked()
                    {
                        chosen = Some(*language);
                    }
                }
            });
        if let Some(language) = chosen {
            app.set_language(language);
        }
        if !app.translator.language().reviewed {
            ui.add_space(4.0);
            theme::help(ui, &app.t("language-not-reviewed-notice"));
        }
    });
}

fn updates_tab(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("channel-title")).strong());
        ui.add_space(4.0);

        let mut channel = app.settings.channel;
        let stable = app.t("channel-stable");
        let beta = app.t("channel-beta");
        let beta_help = app.t("channel-beta-help");
        let mut changed = false;

        changed |= ui
            .radio_value(&mut channel, Channel::Stable, stable)
            .changed();
        changed |= ui.radio_value(&mut channel, Channel::Beta, beta).changed();
        theme::help(ui, &beta_help);

        if changed {
            app.settings.channel = channel;
            app.save_settings();
            // The other channel's answer says nothing about this one, so
            // the check runs again rather than keeping a stale result.
            app.updates = Default::default();
            app.update_check_requested = false;
            crate::screens::updates::check_once(app);
        }
    });

    ui.add_space(10.0);
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let updates_label = app.t("settings-check-updates");
        let keep_label = app.t("settings-keep-versions");
        let mut changed = false;

        changed |= ui
            .checkbox(&mut app.settings.check_updates_on_start, updates_label)
            .changed();

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(keep_label);
            let mut keep = app.settings.keep_versions as u32;
            if ui
                .add(egui::DragValue::new(&mut keep).range(1..=10))
                .changed()
            {
                app.settings.keep_versions = keep as usize;
                changed = true;
            }
        });

        if changed {
            app.save_settings();
        }
    });
}

fn behaviour_tab(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let minimize_label = app.t("behaviour-minimize");
        let close_label = app.t("behaviour-close");
        let close_help = app.t("behaviour-close-help");
        let mut changed = false;

        ui.add_enabled_ui(!app.settings.behaviour.close_on_launch, |ui| {
            changed |= ui
                .checkbox(
                    &mut app.settings.behaviour.minimize_on_launch,
                    minimize_label,
                )
                .changed();
        });
        changed |= ui
            .checkbox(&mut app.settings.behaviour.close_on_launch, close_label)
            .changed();
        theme::help(ui, &close_help);

        if changed {
            app.save_settings();
        }
    });

    ui.add_space(10.0);
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let discord_label = app.t("settings-discord");
        let discord_help = app.t("settings-discord-help");

        // A build with no Discord application id compiled in can never
        // show a presence. Leaving the box tickable would mean a setting
        // that is on and does nothing, which is the hardest kind of
        // thing to debug from the other side of a bug report.
        let configured = recof_core::presence::Presence::is_configured();
        ui.add_enabled_ui(configured, |ui| {
            if ui
                .checkbox(&mut app.settings.discord_rich_presence, discord_label)
                .changed()
            {
                app.save_settings();
            }
        });
        theme::help(ui, &discord_help);
        if !configured {
            ui.label(
                egui::RichText::new(app.t("settings-discord-unavailable"))
                    .size(12.0)
                    .color(theme::WARN),
            );
        }
    });

    ui.add_space(10.0);
    if ui.button(app.t("settings-open-logs")).clicked() {
        let folder = app.paths.logs_dir().display().to_string();
        if let Err(error) = recof_platform::open_in_shell(&folder) {
            log::warn!("the log folder could not be opened: {error}");
        }
    }
}

fn about_tab(app: &mut App, ui: &mut egui::Ui) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());

        let logo = app.wordmark().cloned();
        let width = ui.available_width();
        crate::brand::wordmark(ui, logo.as_ref(), width);

        ui.add_space(10.0);
        theme::body(ui, &app.t("about-what"));

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(app.t_args("about-made-by", &[("author", AUTHOR.to_owned())]))
                .color(theme::TEXT)
                .strong(),
        );

        ui.add_space(8.0);
        theme::help(
            ui,
            &app.t_args(
                "about-version",
                &[("version", recof_core::consts::LAUNCHER_VERSION.to_owned())],
            ),
        );
        theme::help(
            ui,
            &app.t_args(
                "about-licence",
                &[(
                    "url",
                    format!("github.com/{}", recof_core::consts::LAUNCHER_REPO),
                )],
            ),
        );
        theme::help(
            ui,
            &app.t_args(
                "about-recof",
                &[(
                    "url",
                    format!("github.com/{}", recof_core::consts::RECOF_REPO),
                )],
            ),
        );
        theme::help(ui, &app.t("about-font"));

        ui.add_space(8.0);
        theme::help(ui, &app.t("about-not-affiliated"));

        if let Some(credits) = app.artwork_credits.clone() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(app.t("look-credits-title")).strong());
            for line in credits.lines().filter(|line| !line.trim().is_empty()) {
                theme::help(ui, line);
            }
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button(app.t("settings-open-logs")).clicked() {
                let folder = app.paths.logs_dir().display().to_string();
                if let Err(error) = recof_platform::open_in_shell(&folder) {
                    log::warn!("the log folder could not be opened: {error}");
                }
            }
        });
    });
}

/// Who made it. One place, so the about screen and anything else that
/// needs it cannot drift apart.
pub const AUTHOR: &str = "caixax";

pub fn maintenance(app: &mut App, ui: &mut egui::Ui) {
    let title = app.t("maintain-title");
    back_bar(app, ui, &title);

    let Some(install_root) = app.settings.install_path.clone() else {
        ui.label(egui::RichText::new(app.t("home-no-install")).color(theme::WARN));
        return;
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(app.t("maintain-verify")).strong());
            theme::help(ui, &app.t("maintain-verify-help"));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                    if ui.button(app.t("maintain-verify")).clicked() {
                        app.progress = Some(Progress {
                            headline: app.t("maintain-verify"),
                            ..Default::default()
                        });
                        app.worker.send(Job::Verify {
                            install_root: install_root.clone(),
                        });
                    }
                });

                if let Some(report) = last_verify() {
                    if !report.is_intact() {
                        let can_repair = app
                            .store
                            .get(
                                recof_core::manifest::Product::Recof,
                                &app.installed.as_ref().unwrap().version,
                            )
                            .ok()
                            .flatten()
                            .is_some();

                        ui.add_enabled_ui(can_repair && !app.worker.is_busy(), |ui| {
                            if ui.button(app.t("verify-repair")).clicked() {
                                start_repair(app, &install_root, &report);
                            }
                        });
                        if !can_repair {
                            theme::help(ui, &app.t("verify-cannot-repair"));
                        }
                    }
                }
            });

            if let Some(report) = last_verify() {
                ui.add_space(6.0);
                for entry in report.entries.iter().filter(|entry| !entry.is_ok()) {
                    let (text, colour) = match entry {
                        VerifyEntry::Missing { path } => {
                            (format!("{path}: {}", app.t("verdict-missing")), theme::BAD)
                        }
                        VerifyEntry::WrongContent { path, .. } => (
                            format!("{path}: {}", app.t("err-hash-mismatch")),
                            theme::WARN,
                        ),
                        VerifyEntry::Ok { .. } => continue,
                    };
                    ui.label(egui::RichText::new(text).size(12.5).color(colour));
                }
            }
        });

        ui.add_space(10.0);
        crate::screens::updates::from_file(app, ui);

        ui.add_space(10.0);
        crate::screens::updates::rollback(app, ui);

        ui.add_space(10.0);
        backups(app, ui, &install_root);

        ui.add_space(10.0);
        import(app, ui, &install_root);

        ui.add_space(10.0);
        shortcuts(app, ui, &install_root);

        ui.add_space(10.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(app.t("move-title")).strong());
            theme::help(ui, &app.t("move-same-volume"));
            ui.add_space(6.0);
            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                if ui.button(app.t("move-to")).clicked() {
                    start_move(app, &install_root);
                }
            });
        });

        ui.add_space(10.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(app.t("uninstall-title"))
                    .strong()
                    .color(theme::BAD),
            );
            theme::help(ui, &app.t("uninstall-explain"));
            ui.add_space(6.0);

            // Both are remembered across frames, because a checkbox
            // that resets itself every repaint cannot be ticked.
            let keep_id = ui.scope_id().with("keep-backups");
            let export_id = ui.scope_id().with("export-saves");
            let mut keep_backups = ui.data(|data| data.get_temp(keep_id)).unwrap_or(true);
            let mut export_saves = ui.data(|data| data.get_temp(export_id)).unwrap_or(true);

            let export_label = app.t("uninstall-export-first");
            let keep_label = app.t("uninstall-keep-backups");
            if ui.checkbox(&mut export_saves, export_label).changed() {
                ui.data_mut(|data| data.insert_temp(export_id, export_saves));
            }
            if ui.checkbox(&mut keep_backups, keep_label).changed() {
                ui.data_mut(|data| data.insert_temp(keep_id, keep_backups));
            }

            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                if ui.button(app.t("uninstall-title")).clicked() {
                    app.confirm = Some(Confirm::new(
                        app.t_args(
                            "uninstall-confirm",
                            &[("path", install_root.display().to_string())],
                        ),
                        app.t("uninstall-title"),
                        ConfirmAction::Uninstall {
                            remove_backups: !keep_backups,
                            export_saves,
                        },
                    ));
                }
            });
        });
    });
}

fn start_repair(app: &mut App, install_root: &std::path::Path, report: &RepairReport) {
    let Some(state) = app.installed.clone() else {
        return;
    };
    let Ok(Some(release)) = app
        .store
        .get(recof_core::manifest::Product::Recof, &state.version)
    else {
        app.notice = Some(Notice::warn(app.t("verify-cannot-repair")));
        return;
    };

    let broken: Vec<String> = report.broken().into_iter().map(str::to_owned).collect();
    app.progress = Some(Progress {
        headline: app.t("verify-repair"),
        ..Default::default()
    });
    app.worker.send(Job::Repair {
        install_root: install_root.to_path_buf(),
        release: Box::new(release),
        broken,
        now: crate::now_rfc3339(),
        id: "repair".to_owned(),
    });
}

fn start_move(app: &mut App, install_root: &std::path::Path) {
    let Some(folder) = rfd::FileDialog::new()
        .set_title(app.t("move-to"))
        .pick_folder()
    else {
        return;
    };
    let destination = folder.join(recof_core::install::destination::DEFAULT_FOLDER_NAME);

    let cluster = recof_platform::cluster_size(&folder).unwrap_or(4096);
    let measured = recof_core::space::measure_tree(
        install_root,
        cluster,
        &app.worker.cancel_flag(),
        &mut |_| {},
    )
    .ok()
    .and_then(|outcome| outcome.ok())
    .unwrap_or_default();

    app.progress = Some(Progress {
        headline: app.t("move-running"),
        ..Default::default()
    });
    app.worker.send(Job::Move {
        from: install_root.to_path_buf(),
        to: destination,
        measured,
    });
}

pub fn bug_report(app: &mut App, ui: &mut egui::Ui) {
    let title = app.t("bug-title");
    back_bar(app, ui, &title);

    theme::body(ui, &app.t("bug-explain"));
    ui.add_space(4.0);
    ui.label(egui::RichText::new(app.t("bug-never-sends")).color(theme::GOOD));
    ui.add_space(10.0);

    let mut report = collect_report(app);

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 60.0)
        .show(ui, |ui| {
            ui.add_sized(
                [ui.available_width(), 320.0],
                egui::TextEdit::multiline(&mut report).font(egui::TextStyle::Monospace),
            );
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button(app.t("button-copy")).clicked() {
            ui.ctx().copy_text(report.clone());
            app.notice = Some(Notice::good(app.t("bug-copied")));
        }
        if theme::primary_button(ui, &app.t("bug-open-issue")).clicked() {
            open_issue(app, &report);
        }
    });
}

/// Everything the report collects. Built fresh each frame so it is always
/// what the user is about to send.
fn collect_report(app: &App) -> String {
    use std::fmt::Write;
    let mut text = String::new();

    let _ = writeln!(
        text,
        "Launcher: {} ({})",
        recof_core::consts::LAUNCHER_VERSION,
        recof_platform::PLATFORM_NAME
    );
    match &app.installed {
        Some(state) => {
            let _ = writeln!(
                text,
                "ReCoF: {} ({})",
                state.version,
                state.channel.as_str()
            );
            let _ = writeln!(text, "Installed at: {}", state.installed_at);
            let _ = writeln!(text, "Copied from: {}", state.source_path);
            let _ = writeln!(
                text,
                "Source buildid: {}",
                state
                    .source_buildid
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "unknown".to_owned())
            );
        }
        None => {
            let _ = writeln!(text, "ReCoF: not installed");
        }
    }
    let _ = writeln!(text, "Language: {}", app.translator.language().tag);

    if let Some(detection) = &app.detection {
        if let Some(game) = &detection.game {
            let _ = writeln!(text, "\nVanilla: {}", game.root.display());
            let _ = writeln!(text, "Vanilla buildid: {:?}", game.buildid);
        }
        if let Some(report) = &detection.report {
            let _ = writeln!(text, "Untouched retail: {}", report.is_untouched_retail());
            for file in report.notable() {
                let _ = writeln!(text, "  {}: {:?}", file.path, file.verdict);
            }
            for marker in &report.mod_markers {
                let _ = writeln!(text, "  modification: {marker}");
            }
            for name in &report.unexpected_binaries {
                let _ = writeln!(text, "  extra binary: {name}");
            }
        }
    }

    if let Some(verify) = last_verify() {
        let _ = writeln!(text, "\nLast verification: intact {}", verify.is_intact());
        for path in verify.broken() {
            let _ = writeln!(text, "  does not match: {path}");
        }
    }

    let _ = writeln!(text, "\nLast log lines:");
    let _ = writeln!(
        text,
        "{}",
        recof_core::log_file::FileLogger::tail(&app.paths.logs_dir(), 60)
    );

    text
}

fn open_issue(app: &mut App, report: &str) {
    // GitHub truncates a very long query string, and a browser has its own
    // limit, so the body is capped and the user is told to paste the rest.
    const MAX_BODY: usize = 4000;
    let body = if report.len() > MAX_BODY {
        format!("{}\n...\n", &report[..MAX_BODY])
    } else {
        report.to_owned()
    };

    let url = format!(
        "https://github.com/{}/issues/new?title={}&body={}",
        recof_core::consts::LAUNCHER_REPO,
        percent_encode("Bug report"),
        percent_encode(&body)
    );

    if let Err(error) = recof_platform::open_in_shell(&url) {
        log::warn!("the browser could not be opened: {error}");
        app.notice = Some(Notice::warn(app.t("bug-copied")));
    }
}

/// Percent encoding for a URL query value.
fn percent_encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// Making and restoring save backups.
fn backups(app: &mut App, ui: &mut egui::Ui, install_root: &std::path::Path) {
    if !app.backups_listed && !app.worker.is_busy() {
        app.backups_listed = true;
        app.worker.send(Job::ListBackups {
            app_data: app.paths.root().to_path_buf(),
        });
    }

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("backup-title")).strong());
        theme::help(ui, &app.t("backup-what"));
        ui.add_space(6.0);

        ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
            if ui.button(app.t("backup-create")).clicked() {
                app.progress = Some(Progress {
                    headline: app.t("backup-create"),
                    ..Default::default()
                });
                app.worker.send(Job::Backup {
                    install_root: install_root.to_path_buf(),
                    app_data: app.paths.root().to_path_buf(),
                    kind: BackupKind::Manual,
                    now: crate::now_rfc3339(),
                    stamp: crate::now_stamp(),
                    recof_version: app
                        .installed
                        .as_ref()
                        .map(|state| state.version.to_string()),
                });
            }
        });

        ui.add_space(6.0);
        if app.backups.is_empty() {
            theme::help(ui, &app.t("backup-none"));
            return;
        }

        let found = app.backups.clone();
        for backup in &found {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(app.t_args(
                        "backup-list-entry",
                        &[
                            ("when", backup.manifest.created.clone()),
                            ("size", format_bytes(backup.size)),
                        ],
                    ))
                    .size(12.5),
                );
                ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                    if ui.small_button(app.t("maintain-restore")).clicked() {
                        // Restoring replaces the saves in the copy, so
                        // it asks before it does it.
                        app.confirm = Some(Confirm::new(
                            app.t_args("backup-restore-confirm", &[("name", backup.name.clone())]),
                            app.t("maintain-restore"),
                            ConfirmAction::RestoreBackup {
                                archive: backup.path.clone(),
                            },
                        ));
                    }
                });
            });
        }
    });
}

/// Bringing progress over from the Steam copy.
fn import(app: &mut App, ui: &mut egui::Ui, install_root: &std::path::Path) {
    let vanilla = app
        .settings
        .vanilla_path
        .clone()
        .or_else(|| app.game_path());

    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("import-title")).strong());
        theme::help(ui, &app.t("import-explain"));
        ui.add_space(6.0);

        let Some(vanilla) = vanilla else {
            theme::help(ui, &app.t("detect-not-found"));
            return;
        };

        match app.import_plan.clone() {
            None => {
                ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                    if ui.button(app.t("maintain-import")).clicked() {
                        app.progress = Some(Progress {
                            headline: app.t("import-title"),
                            ..Default::default()
                        });
                        app.worker.send(Job::PlanImport {
                            vanilla: vanilla.clone(),
                            install_root: install_root.to_path_buf(),
                        });
                    }
                });
            }
            Some(plan) => {
                if plan.to_write() == 0 {
                    theme::help(ui, &app.t("import-nothing-found"));
                    if ui.button(app.t("button-close")).clicked() {
                        app.import_plan = None;
                    }
                    return;
                }

                let overwrites = plan.overwrites().len();
                if overwrites > 0 {
                    ui.label(
                        egui::RichText::new(app.t("import-overwrite-title"))
                            .color(theme::WARN)
                            .strong(),
                    );
                    ui.label(app.t_args(
                        "import-overwrite-body",
                        &[("count", overwrites.to_string())],
                    ));
                }
                theme::help(
                    ui,
                    &app.t_args(
                        "detect-size",
                        &[
                            ("files", plan.to_write().to_string()),
                            ("size", format_bytes(plan.total_bytes())),
                        ],
                    ),
                );

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                        if ui.button(app.t("button-continue")).clicked() {
                            app.progress = Some(Progress {
                                headline: app.t("import-title"),
                                ..Default::default()
                            });
                            app.worker.send(Job::Import {
                                vanilla: vanilla.clone(),
                                install_root: install_root.to_path_buf(),
                                plan: Box::new(plan.clone()),
                                app_data: app.paths.root().to_path_buf(),
                                now: crate::now_rfc3339(),
                                stamp: crate::now_stamp(),
                            });
                        }
                    });
                    if ui.button(app.t("button-cancel")).clicked() {
                        app.import_plan = None;
                    }
                });
            }
        }
    });
}

/// Desktop, start menu and the non Steam shortcut.
fn shortcuts(app: &mut App, ui: &mut egui::Ui, install_root: &std::path::Path) {
    theme::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new(app.t("maintain-shortcuts")).strong());
        ui.add_space(6.0);

        // The shortcut starts the game the same way the play button does.
        let arguments = recof_core::launch::build(&app.settings.launch, &[]).join(" ");

        ui.horizontal(|ui| {
            if ui.button(app.t("shortcut-desktop")).clicked() {
                app.worker.send(Job::MakeShortcut {
                    kind: ShortcutKind::Desktop,
                    install_root: install_root.to_path_buf(),
                    arguments: arguments.clone(),
                    name: "ReCoF".to_owned(),
                });
            }
            if ui.button(app.t("shortcut-start-menu")).clicked() {
                app.worker.send(Job::MakeShortcut {
                    kind: ShortcutKind::StartMenu,
                    install_root: install_root.to_path_buf(),
                    arguments: arguments.clone(),
                    name: "ReCoF".to_owned(),
                });
            }
        });

        ui.add_space(10.0);
        ui.label(egui::RichText::new(app.t("shortcut-steam")).strong());
        theme::help(ui, &app.t("shortcut-steam-help"));

        let steam = app
            .detection
            .as_ref()
            .and_then(|detection| detection.steam.as_ref())
            .map(|layout| layout.steam_path.clone());
        let Some(steam) = steam else {
            theme::help(ui, &app.t("detect-not-found"));
            return;
        };

        let users = recof_core::steam::shortcuts::users(&steam).unwrap_or_default();
        if users.is_empty() {
            theme::help(ui, &app.t("detect-not-found"));
            return;
        }

        ui.add_space(4.0);
        for user in &users {
            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                let label = format!("{}  {}", app.t("shortcut-steam"), user.id);
                if ui.button(label).clicked() {
                    app.worker.send(Job::AddToSteam {
                        user: Box::new(user.clone()),
                        entry: Box::new(recof_core::steam::shortcuts::ShortcutEntry {
                            app_name: "ReCoF".to_owned(),
                            exe: install_root.join(recof_core::consts::GAME_EXE),
                            start_dir: install_root.to_path_buf(),
                            launch_options: arguments.clone(),
                            icon: String::new(),
                        }),
                        stamp: crate::now_stamp(),
                    });
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encoding_leaves_unreserved_characters_alone() {
        assert_eq!(percent_encode("abcXYZ019-_.~"), "abcXYZ019-_.~");
    }

    #[test]
    fn percent_encoding_escapes_everything_else() {
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("a\nb"), "a%0Ab");
        assert_eq!(percent_encode("c:\\x"), "c%3A%5Cx");
        assert_eq!(percent_encode("&=?#"), "%26%3D%3F%23");
    }

    #[test]
    fn percent_encoding_handles_non_ascii_as_utf8_bytes() {
        assert_eq!(percent_encode("ñ"), "%C3%B1");
        assert_eq!(percent_encode("游"), "%E6%B8%B8");
    }
}
