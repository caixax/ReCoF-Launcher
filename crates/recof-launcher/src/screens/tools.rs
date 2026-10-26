//! Settings, maintenance and the bug report.

use recof_core::install::{RepairReport, VerifyEntry};
use recof_core::manifest::Channel;

use crate::app::{App, Confirm, ConfirmAction, Notice, Progress, Screen};
use crate::i18n::LANGUAGES;
use crate::theme;
use crate::worker::Job;

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

pub fn settings(app: &mut App, ui: &mut egui::Ui) {
    let title = app.t("settings-title");
    back_bar(app, ui, &title);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

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
                theme::help(ui, &app.t("language-not-reviewed-notice"));
            }
        });

        ui.add_space(10.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new(app.t("channel-title")).strong());
            ui.add_space(4.0);

            let mut channel = app.settings.channel;
            let mut changed = false;
            changed |= ui
                .radio_value(&mut channel, Channel::Stable, app.t("channel-stable"))
                .changed();
            changed |= ui
                .radio_value(&mut channel, Channel::Beta, app.t("channel-beta"))
                .changed();
            theme::help(ui, &app.t("channel-beta-help"));
            if changed {
                app.settings.channel = channel;
                app.save_settings();
            }
        });

        ui.add_space(10.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            // Labels first: a checkbox borrows the settings mutably.
            let updates_label = app.t("settings-check-updates");
            let discord_label = app.t("settings-discord");
            let discord_help = app.t("settings-discord-help");
            let keep_label = app.t("settings-keep-versions");
            let mut changed = false;

            changed |= ui
                .checkbox(&mut app.settings.check_updates_on_start, updates_label)
                .changed();

            changed |= ui
                .checkbox(&mut app.settings.discord_rich_presence, discord_label)
                .changed();
            theme::help(ui, &discord_help);

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

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button(app.t("settings-open-logs")).clicked() {
                let folder = app.paths.logs_dir().display().to_string();
                if let Err(error) = recof_platform::open_in_shell(&folder) {
                    log::warn!("the log folder could not be opened: {error}");
                }
            }
        });

        ui.add_space(10.0);
        theme::help(
            ui,
            &format!(
                "{} {}  ({})",
                app.t("app-name"),
                recof_core::consts::LAUNCHER_VERSION,
                recof_platform::PLATFORM_NAME
            ),
        );
    });
}

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

            let mut keep_backups = true;
            ui.checkbox(&mut keep_backups, app.t("uninstall-keep-backups"));

            ui.add_enabled_ui(!app.worker.is_busy(), |ui| {
                if ui.button(app.t("uninstall-title")).clicked() {
                    app.confirm = Some(Confirm {
                        question: app.t_args(
                            "uninstall-confirm",
                            &[("path", install_root.display().to_string())],
                        ),
                        confirm_label: app.t("uninstall-title"),
                        action: ConfirmAction::Uninstall {
                            remove_backups: !keep_backups,
                        },
                    });
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
