//! The application: what is on screen, and what the worker is doing.

use std::path::PathBuf;

use recof_core::consts;
use recof_core::install::InstalledState;
use recof_core::manifest::Product;
use recof_core::paths::AppPaths;
use recof_core::settings::Settings;
use recof_core::space::TreeSize;
use recof_core::store::VersionStore;

use crate::i18n::{Language, Translator};
use crate::screens;
use crate::theme;
use crate::worker::{Detection, Event, Job, Measurement, UpdateCheck, Worker};

/// Which screen is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// First run, step one.
    Language,
    /// First run, the two or three explanation screens.
    Intro(usize),
    /// Finding the game and showing what it is.
    Detect,
    /// Choosing where the copy goes.
    Destination,
    /// Copying and applying.
    Installing,
    /// The normal screen once something is installed.
    Home,
    Settings,
    Maintenance,
    BugReport,
}

/// How many introduction screens there are.
pub const INTRO_SCREENS: usize = 3;

/// A line of feedback shown under the content.
#[derive(Debug, Clone)]
pub struct Notice {
    pub text: String,
    pub level: theme::Level,
}

impl Notice {
    pub fn good(text: impl Into<String>) -> Self {
        Notice {
            text: text.into(),
            level: theme::Level::Good,
        }
    }

    pub fn info(text: impl Into<String>) -> Self {
        Notice {
            text: text.into(),
            level: theme::Level::Info,
        }
    }

    pub fn warn(text: impl Into<String>) -> Self {
        Notice {
            text: text.into(),
            level: theme::Level::Warn,
        }
    }

    pub fn bad(text: impl Into<String>) -> Self {
        Notice {
            text: text.into(),
            level: theme::Level::Bad,
        }
    }
}

/// Where a long running job has got to.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    pub headline: String,
    pub detail: String,
    /// Between 0 and 1, or `None` when the job cannot say.
    pub fraction: Option<f32>,
}

/// A question the user has to answer before anything else happens.
#[derive(Debug, Clone)]
pub struct Confirm {
    pub question: String,
    pub confirm_label: String,
    pub action: ConfirmAction,
    /// A second way forward, for a question with more than one
    /// sensible answer. Steam not running is the case: open Steam, or
    /// start anyway.
    pub alternative: Option<(String, ConfirmAction)>,
}

impl Confirm {
    /// A plain yes or no.
    pub fn new(question: String, confirm_label: String, action: ConfirmAction) -> Self {
        Confirm {
            question,
            confirm_label,
            action,
            alternative: None,
        }
    }

    pub fn or(mut self, label: String, action: ConfirmAction) -> Self {
        self.alternative = Some((label, action));
        self
    }
}

/// What a confirmation will do if accepted.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    CancelInstall,
    Uninstall {
        remove_backups: bool,
        export_saves: bool,
    },
    /// Open Steam and leave the game for the user to start.
    OpenSteam,
    /// Start ReCoF even though Steam is not running.
    LaunchAnyway,
    RestoreBackup {
        archive: std::path::PathBuf,
    },
}

pub struct App {
    pub translator: Translator,
    pub settings: Settings,
    pub paths: AppPaths,
    pub store: VersionStore,
    pub worker: Worker,

    pub screen: Screen,
    pub notice: Option<Notice>,
    pub progress: Option<Progress>,
    pub confirm: Option<Confirm>,

    /// What detection found, once it has run.
    pub detection: Option<Detection>,
    /// Whether detection has been asked for, so it runs once.
    pub detect_requested: bool,
    /// The folder the user picked by hand, if any.
    pub manual_game_path: Option<PathBuf>,

    /// The destination as the user is editing it.
    pub destination: String,
    pub measurement: Option<Measurement>,
    pub measured_for: Option<PathBuf>,

    /// The copy that is installed, if there is one.
    pub installed: Option<InstalledState>,

    /// Whether the install should bring the vanilla saves over. Not a
    /// setting: it only matters while the install wizard is open.
    pub import_on_install: bool,

    /// Which part of the settings is showing.
    pub settings_tab: crate::screens::tools::SettingsTab,

    /// Artwork the repository carries, when it carries any.
    pub bundled_hero: Option<egui::TextureHandle>,
    pub bundled_logo: Option<egui::TextureHandle>,
    /// The pictures in the theme folder, fetched or put there by hand.
    pub artwork: crate::art::Artwork,
    /// Who made them, when the release said.
    pub artwork_credits: Option<String>,
    /// Whether the fetch has been asked for, so it runs once.
    pub artwork_requested: bool,
    /// The grain overlay, generated once at start up.
    pub grain: crate::brand::Grain,
    /// Whether to make a desktop shortcut once the install finishes.
    pub shortcut_on_install: bool,

    /// Discord Rich Presence, which is off unless the setting is on.
    pub presence: recof_core::presence::Presence,
    /// When the game was last checked for, so the poll is not run on
    /// every frame.
    pub presence_checked: std::time::Instant,

    /// What the last update check found.
    pub updates: UpdateCheck,
    /// Whether the check has been asked for, so it runs once per start.
    pub update_check_requested: bool,
    /// Whether a check is running now.
    pub update_checking: bool,
    /// A release zip picked on the install screen, instead of downloading.
    pub install_file: Option<PathBuf>,
    /// Set once a newer launcher has been written to disk, so the user
    /// can be asked to restart.
    pub self_update_ready: Option<recof_core::version::Version>,

    /// The backups there are, once they have been listed.
    pub backups: Vec<recof_core::backup::Backup>,
    pub backups_listed: bool,
    /// What an import would do, worked out before it is offered.
    pub import_plan: Option<recof_core::backup::ImportPlan>,

    /// The console variables the game saved in the copy, and when they
    /// were read.
    saved_cvars: Option<(
        std::time::Instant,
        std::collections::BTreeMap<String, String>,
    )>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> App {
        theme::apply(&cc.egui_ctx);
        // After the theme, because it replaces the font definitions
        // and the system fallback has to be set up around it.
        crate::brand::install_font(&cc.egui_ctx);

        let paths = AppPaths::discover().unwrap_or_else(|error| {
            log::error!("LOCALAPPDATA could not be found: {error}");
            AppPaths::at(std::env::temp_dir().join(consts::APP_DIR_NAME))
        });
        if let Err(error) = paths.create_all() {
            log::error!("the launcher folders could not be created: {error}");
        }

        let settings = Settings::load(&paths.settings_file());
        let tag = settings
            .language
            .clone()
            .unwrap_or_else(|| crate::i18n::default_language().tag.to_owned());
        // Fonts need no wiring: eframe's system_fonts feature supplies a
        // glyph from an installed font whenever the built in ones lack it,
        // which is how the Chinese translation renders.
        let translator = Translator::load(&tag);

        let screen = if settings.language.is_none() {
            Screen::Language
        } else if !settings.intro_seen {
            Screen::Intro(0)
        } else if settings.install_path.is_some() {
            Screen::Home
        } else {
            Screen::Detect
        };

        let installed = settings
            .install_path
            .as_ref()
            .and_then(|path| InstalledState::load(path).ok().flatten());

        let store = VersionStore::new(paths.versions_dir());
        let mut app = App {
            translator,
            settings,
            paths,
            store,
            worker: Worker::start(cc.egui_ctx.clone()),
            screen,
            notice: None,
            progress: None,
            confirm: None,
            detection: None,
            detect_requested: false,
            manual_game_path: None,
            destination: String::new(),
            measurement: None,
            measured_for: None,
            installed,
            import_on_install: true,
            settings_tab: Default::default(),
            bundled_hero: crate::bundled::hero(&cc.egui_ctx),
            bundled_logo: crate::bundled::logo(&cc.egui_ctx),
            artwork: crate::art::Artwork::default(),
            artwork_credits: None,
            artwork_requested: false,
            grain: crate::brand::Grain::new(&cc.egui_ctx),
            shortcut_on_install: false,
            presence: recof_core::presence::Presence::new(),
            presence_checked: std::time::Instant::now(),
            updates: UpdateCheck::default(),
            update_check_requested: false,
            update_checking: false,
            install_file: None,
            self_update_ready: None,
            backups: Vec::new(),
            backups_listed: false,
            import_plan: None,
            saved_cvars: None,
        };

        app.artwork.load(&cc.egui_ctx, &app.paths.theme_dir());
        app.artwork_credits =
            recof_core::artwork::credits(&app.paths.theme_dir()).or_else(crate::bundled::credits);

        // Nothing in the folder means nobody has been offered the
        // artwork yet. It comes from the ReCoF release rather than
        // from this repository, which ships no picture of the game.
        if !crate::bundled::any() && !recof_core::artwork::already_present(&app.paths.theme_dir()) {
            app.artwork_requested = true;
            app.worker.send_background(Job::FetchArtwork {
                app_data: app.paths.root().to_path_buf(),
                channel: app.settings.channel,
            });
        }

        // An interrupted operation is finished or undone before anything
        // else is allowed to touch the copy.
        if let Some(path) = app.settings.install_path.clone() {
            app.worker.send(Job::Recover { install_root: path });
        }
        // And the update check starts now, on its own thread, whatever
        // screen the launcher opens on.
        screens::updates::check_once(&mut app);
        app
    }

    /// A translated message.
    pub fn t(&self, key: &str) -> String {
        self.translator.get(key)
    }

    /// A translated message with values.
    pub fn t_args(&self, key: &str, args: &[(&str, String)]) -> String {
        self.translator.get_with(key, args)
    }

    /// Switches language.
    pub fn set_language(&mut self, language: Language) {
        self.translator = Translator::load(language.tag);
        self.settings.language = Some(language.tag.to_owned());
        self.save_settings();
    }

    pub fn save_settings(&mut self) {
        if let Err(error) = self.settings.save(&self.paths.settings_file()) {
            log::error!("the settings could not be saved: {error}");
        }
    }

    /// The game folder to copy from: the one detection found, or the one
    /// the user picked.
    pub fn game_path(&self) -> Option<PathBuf> {
        self.manual_game_path.clone().or_else(|| {
            self.detection
                .as_ref()
                .and_then(|detection| detection.game.as_ref())
                .map(|game| game.root.clone())
        })
    }

    /// The Steam buildid of that folder, when it is known.
    pub fn game_buildid(&self) -> Option<u64> {
        self.detection
            .as_ref()
            .and_then(|detection| detection.game.as_ref())
            .and_then(|game| game.buildid)
    }

    /// The console variables the game saved in the copy, read again at
    /// most every two seconds rather than on every frame, so a value the
    /// game wrote on exit shows up soon after.
    pub fn saved_cvars(&mut self) -> std::collections::BTreeMap<String, String> {
        let fresh = self
            .saved_cvars
            .as_ref()
            .is_some_and(|(read, _)| read.elapsed() < std::time::Duration::from_secs(2));
        if !fresh {
            let values = self
                .settings
                .install_path
                .as_deref()
                .map(recof_core::launch::saved_values)
                .unwrap_or_default();
            self.saved_cvars = Some((std::time::Instant::now(), values));
        }
        self.saved_cvars
            .as_ref()
            .map(|(_, values)| values.clone())
            .unwrap_or_default()
    }

    /// The newest release sitting in the version store.
    pub fn newest_stored(&self) -> Option<recof_core::store::StoredVersion> {
        self.store
            .list(Product::Recof)
            .ok()?
            .into_iter()
            .next_back()
    }

    /// Reads the theme folder again, so a picture dropped in while the
    /// launcher is open shows up without a restart.
    pub fn reload_artwork(&mut self, ctx: &egui::Context) {
        self.artwork.forget();
        self.artwork.load(ctx, &self.paths.theme_dir());
    }

    /// The background picture, when there is one and the user wants it.
    pub fn hero(&self) -> Option<&egui::TextureHandle> {
        if self.settings.look.show_hero {
            self.artwork.hero().or(self.bundled_hero.as_ref())
        } else {
            None
        }
    }

    /// Whichever wordmark should be drawn: the user's, then the one
    /// the repository carries, then none, which leaves the type to be
    /// laid out live.
    pub fn wordmark(&self) -> Option<&egui::TextureHandle> {
        self.artwork.logo().or(self.bundled_logo.as_ref())
    }

    /// The grain, when the setting is on.
    pub fn grain(&self) -> Option<&crate::brand::Grain> {
        if self.settings.look.grain {
            Some(&self.grain)
        } else {
            None
        }
    }

    /// Gets out of the way once the game is running, if asked to.
    pub fn step_aside(&self, ctx: &egui::Context) {
        if self.settings.behaviour.close_on_launch {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if self.settings.behaviour.minimize_on_launch {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
    }

    /// Shows a translated error.
    pub fn show_error(&mut self, message_id: &str, args: &[(&str, String)]) {
        self.notice = Some(Notice::bad(self.t_args(message_id, args)));
    }

    fn handle(&mut self, event: Event) {
        match event {
            Event::Detected(detection) => {
                self.detection = Some(*detection);
            }
            Event::Measured(measurement) => {
                self.measured_for = Some(PathBuf::from(&self.destination));
                self.measurement = Some(*measurement);
                self.progress = None;
            }
            Event::CopyProgress(progress) => {
                self.progress = Some(Progress {
                    headline: self.t_args(
                        "install-copying-counted",
                        &[
                            ("done", progress.files_done.to_string()),
                            ("total", progress.files_total.to_string()),
                        ],
                    ),
                    detail: progress.current.clone(),
                    fraction: Some(progress.fraction()),
                });
            }
            Event::ApplyProgress {
                done,
                total,
                path,
                committing,
            } => {
                let version = self
                    .newest_stored()
                    .map(|stored| stored.version.to_string())
                    .unwrap_or_default();
                let headline = if committing {
                    self.t("install-finishing")
                } else {
                    self.t_args("install-applying", &[("version", version)])
                };
                self.progress = Some(Progress {
                    headline,
                    detail: path,
                    fraction: Some(if total == 0 {
                        1.0
                    } else {
                        done as f32 / total as f32
                    }),
                });
            }
            Event::CountProgress { done, total } => {
                if let Some(progress) = self.progress.as_mut() {
                    progress.fraction = Some(if total == 0 {
                        1.0
                    } else {
                        done as f32 / total as f32
                    });
                }
            }
            Event::Installed(state) => {
                let version = state.version.to_string();
                self.settings.install_path = Some(PathBuf::from(&self.destination));
                self.settings.vanilla_path = self.game_path();
                self.installed = Some(*state);
                self.save_settings();
                self.progress = None;
                self.screen = Screen::Home;
                self.notice = Some(Notice::good(
                    self.t_args("install-done", &[("version", version)]),
                ));
                if self.shortcut_on_install {
                    self.shortcut_on_install = false;
                    if let Some(root) = self.settings.install_path.clone() {
                        let arguments =
                            recof_core::launch::build(&self.settings.launch, &[]).join(" ");
                        self.worker.send(crate::worker::Job::MakeShortcut {
                            kind: crate::worker::ShortcutKind::Desktop,
                            install_root: root,
                            arguments,
                            name: "ReCoF".to_owned(),
                        });
                    }
                }
            }
            Event::InstallCancelled => {
                self.progress = None;
                self.screen = Screen::Destination;
                self.notice = Some(Notice::info(self.t("install-cancelled")));
            }
            Event::Recovered(recovery) => {
                if let Some(key) = recovery.message_id() {
                    let version = match &recovery {
                        recof_core::journal::Recovery::RolledForward { version, .. } => {
                            version.to_string()
                        }
                        _ => String::new(),
                    };
                    self.notice = Some(Notice::warn(self.t_args(key, &[("version", version)])));
                    // The copy may now hold a different version.
                    if let Some(path) = &self.settings.install_path {
                        self.installed = InstalledState::load(path).ok().flatten();
                    }
                }
            }
            Event::Verified(report) => {
                self.progress = None;
                self.notice = Some(if report.is_intact() {
                    Notice::good(self.t("verify-intact"))
                } else {
                    Notice::warn(self.t_args(
                        "verify-broken",
                        &[("count", report.broken().len().to_string())],
                    ))
                });
                screens::tools::store_verify_report(self, *report);
            }
            Event::Repaired(count) => {
                self.progress = None;
                self.notice = Some(Notice::good(
                    self.t_args("verify-repaired", &[("count", count.to_string())]),
                ));
            }
            Event::Moved(path) => {
                self.progress = None;
                self.settings.install_path = Some(path.clone());
                self.save_settings();
                self.notice = Some(Notice::good(
                    self.t_args("move-done", &[("path", path.display().to_string())]),
                ));
            }
            Event::Uninstalled => {
                self.progress = None;
                self.settings.install_path = None;
                self.installed = None;
                self.save_settings();
                self.screen = Screen::Detect;
                self.detect_requested = false;
                self.notice = Some(Notice::good(self.t("uninstall-done")));
            }
            Event::FetchProgress(progress) => {
                self.progress = Some(self.fetch_progress(progress));
            }
            Event::UpdatesChecked(found) => {
                self.updates = *found;
                self.update_checking = false;
            }
            Event::Updated(state) => {
                let version = state.version.to_string();
                self.installed = Some(*state);
                self.progress = None;
                // What was offered is now installed, so the banner goes.
                self.updates.recof = None;
                self.notice = Some(Notice::good(
                    self.t_args("update-done", &[("version", version)]),
                ));
            }
            Event::SelfUpdated(version) => {
                self.progress = None;
                self.self_update_ready = Some(version);
                self.updates.launcher = None;
                self.notice = Some(Notice::good(self.t("update-launcher-restart")));
            }
            Event::BackedUp(made) => {
                self.progress = None;
                self.backups_listed = false;
                self.notice = Some(match made {
                    Some(backup) => Notice::good(
                        self.t_args("backup-created", &[("name", backup.name.clone())]),
                    ),
                    None => Notice::info(self.t("import-nothing-found")),
                });
            }
            Event::Backups(found) => {
                self.backups = found;
                self.backups_listed = true;
            }
            Event::Restored(count) => {
                self.progress = None;
                let _ = count;
                self.notice = Some(Notice::good(self.t("backup-restored")));
            }
            Event::ImportPlanned(plan) => {
                self.progress = None;
                self.import_plan = Some(*plan);
            }
            Event::Imported(report) => {
                self.progress = None;
                self.import_plan = None;
                self.notice = Some(Notice::good(
                    self.t_args("import-done", &[("count", report.copied.to_string())]),
                ));
            }
            Event::ArtworkFetched(fetched) => {
                if !fetched.is_empty() {
                    log::info!("artwork fetched: {:?}", fetched.written);
                    self.artwork_credits = fetched
                        .credits
                        .clone()
                        .or_else(|| recof_core::artwork::credits(&self.paths.theme_dir()));
                    // The textures are loaded on the next frame, which
                    // is where a context is to hand.
                    self.artwork.forget();
                }
            }
            Event::ShortcutMade => {
                self.notice = Some(Notice::good(self.t("shortcut-created")));
            }
            Event::AddedToSteam => {
                self.notice = Some(Notice::good(self.t("shortcut-steam-added")));
            }
            Event::Failed {
                message_id,
                args,
                detail,
            } => {
                log::error!("{detail}");
                self.progress = None;
                // An install that failed has nothing left to show progress
                // for, and the choices that led to it are one step back.
                if self.screen == Screen::Installing {
                    self.screen = Screen::Destination;
                }
                self.notice = Some(Notice::bad(self.translator.get_with(message_id, &args)));
            }
        }
    }

    /// Turns a download or unpack step into something to show.
    fn fetch_progress(&self, progress: recof_core::update::FetchProgress) -> Progress {
        use recof_core::update::FetchProgress;
        match progress {
            FetchProgress::Metadata => Progress {
                headline: self.t("update-verifying"),
                ..Default::default()
            },
            FetchProgress::Archive(state) => Progress {
                headline: self.t_args(
                    "update-downloading",
                    &[("name", recof_core::space::format_bytes(state.total))],
                ),
                detail: format!(
                    "{} / {}",
                    recof_core::space::format_bytes(state.done),
                    recof_core::space::format_bytes(state.total)
                ),
                fraction: Some(state.fraction()),
            },
            FetchProgress::Verifying => Progress {
                headline: self.t("update-verifying"),
                ..Default::default()
            },
            FetchProgress::Extracting { done, total } => Progress {
                headline: self.t("update-extracting"),
                detail: format!("{done} / {total}"),
                fraction: Some(if total == 0 {
                    1.0
                } else {
                    done as f32 / total as f32
                }),
            },
        }
    }

    /// Runs a confirmed action.
    pub fn run_confirmed(&mut self, action: ConfirmAction, context: Option<&egui::Context>) {
        match action {
            ConfirmAction::CancelInstall => self.worker.cancel(),
            ConfirmAction::Uninstall {
                remove_backups,
                export_saves,
            } => {
                let Some(install_root) = self.settings.install_path.clone() else {
                    return;
                };
                // Exporting goes first and through the same queue, so
                // the saves are safely in the backups folder before the
                // copy is removed.
                if export_saves {
                    self.worker.send(Job::Backup {
                        install_root: install_root.clone(),
                        app_data: self.paths.root().to_path_buf(),
                        kind: recof_core::backup::BackupKind::Export,
                        now: crate::now_rfc3339(),
                        stamp: crate::now_stamp(),
                        recof_version: self
                            .installed
                            .as_ref()
                            .map(|state| state.version.to_string()),
                    });
                }
                self.progress = Some(Progress {
                    headline: self.t("uninstall-title"),
                    ..Default::default()
                });
                self.worker.send(Job::Uninstall {
                    install_root,
                    app_data: self.paths.root().to_path_buf(),
                    options: recof_core::install::UninstallOptions {
                        remove_version_store: true,
                        remove_backups,
                    },
                });
            }
            ConfirmAction::OpenSteam => {
                if let Err(error) = recof_core::launch::open_steam() {
                    log::warn!("Steam could not be opened: {error}");
                }
            }
            ConfirmAction::LaunchAnyway => {
                if let (Some(root), Some(ctx)) =
                    (self.settings.install_path.clone(), context.cloned())
                {
                    crate::screens::home::start_game_now(self, &ctx, &root);
                }
            }
            ConfirmAction::RestoreBackup { archive } => {
                let Some(install_root) = self.settings.install_path.clone() else {
                    return;
                };
                self.progress = Some(Progress {
                    headline: self.t("maintain-restore"),
                    ..Default::default()
                });
                self.worker.send(Job::Restore {
                    archive,
                    install_root,
                });
            }
        }
    }

    /// Asks the worker to measure the current source and destination.
    pub fn request_measure(&mut self) {
        let Some(source) = self.game_path() else {
            return;
        };
        if self.destination.trim().is_empty() {
            return;
        }
        self.progress = Some(Progress {
            headline: self.t("detect-check-title"),
            ..Default::default()
        });
        self.worker.send(Job::Measure {
            source,
            destination: PathBuf::from(&self.destination),
            app_data: self.paths.root().to_path_buf(),
        });
    }

    /// The measured size, if it was measured for the current destination.
    pub fn measured_size(&self) -> Option<TreeSize> {
        let wanted = PathBuf::from(&self.destination);
        match (&self.measurement, &self.measured_for) {
            (Some(measurement), Some(path)) if *path == wanted => Some(measurement.size),
            _ => None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for event in self.worker.drain() {
            self.handle(event);
        }
        if !self.artwork.is_loaded() {
            let ctx = ui.ctx().clone();
            self.artwork.load(&ctx, &self.paths.theme_dir());
        }

        let ctx = ui.ctx().clone();
        crate::brand::paint_background(
            ui,
            ui.max_rect(),
            self.hero(),
            self.settings.look.dim(),
            self.grain(),
        );
        egui::Frame::new()
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                self.draw(ui);
            });

        self.draw_confirm(&ctx);
        self.update_presence();
    }
}

impl App {
    /// Keeps the Discord presence in step with whether the game is
    /// open.
    ///
    /// Polled every few seconds rather than every frame: it asks the
    /// system for the process list, which is not something to do sixty
    /// times a second.
    fn update_presence(&mut self) {
        const EVERY: std::time::Duration = std::time::Duration::from_secs(3);
        if self.presence_checked.elapsed() < EVERY {
            return;
        }
        self.presence_checked = std::time::Instant::now();

        if !self.settings.discord_rich_presence {
            if self.presence.is_showing() {
                self.presence.disconnect();
            }
            return;
        }

        let running = matches!(
            recof_platform::is_running_by_name(recof_core::launch::GAME_PROCESS),
            Ok(true)
        );
        match (running, self.installed.as_ref()) {
            (true, Some(state)) => {
                let version = state.version.to_string();
                let _ = self.presence.show_playing(&version);
            }
            _ => {
                if self.presence.is_showing() {
                    self.presence.clear();
                }
            }
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        match self.screen {
            Screen::Language => screens::setup::language(self, ui),
            Screen::Intro(step) => screens::setup::intro(self, ui, step),
            Screen::Detect => screens::setup::detect(self, ui),
            Screen::Destination => screens::setup::destination(self, ui),
            Screen::Installing => screens::setup::installing(self, ui),
            Screen::Home => screens::home::home(self, ui),
            Screen::Settings => screens::tools::settings(self, ui),
            Screen::Maintenance => screens::tools::maintenance(self, ui),
            Screen::BugReport => screens::tools::bug_report(self, ui),
        }

        if let Some(notice) = self.notice.clone() {
            ui.add_space(10.0);
            theme::rule(ui);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&notice.text).color(theme::status_color(notice.level)),
                );
                if ui.small_button(self.t("button-close")).clicked() {
                    self.notice = None;
                }
            });
        }
    }

    fn draw_confirm(&mut self, ctx: &egui::Context) {
        let Some(confirm) = self.confirm.clone() else {
            return;
        };

        let mut open = true;
        egui::Window::new(self.t("app-name"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_max_width(460.0);
                theme::body(ui, &confirm.question);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(&confirm.confirm_label).clicked() {
                        self.run_confirmed(confirm.action.clone(), Some(ctx));
                        open = false;
                    }
                    if let Some((label, action)) = &confirm.alternative {
                        if ui.button(label).clicked() {
                            self.run_confirmed(action.clone(), Some(ctx));
                            open = false;
                        }
                    }
                    if ui.button(self.t("button-cancel")).clicked() {
                        open = false;
                    }
                });
            });

        if !open {
            self.confirm = None;
        }
    }
}
