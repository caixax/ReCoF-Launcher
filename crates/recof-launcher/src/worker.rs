//! The background thread.
//!
//! The UI thread never blocks. Everything that touches a disk, a registry
//! or the network goes here, and comes back as an [`Event`] the next frame
//! picks up. There is no async runtime: one worker thread, two channels,
//! and a repaint request so a running job keeps the window moving.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use recof_core::backup::{self, Backup, BackupKind, ImportPlan, ImportReport};
use recof_core::hashes::InstallReport;
use recof_core::install::{self, InstalledState, RepairReport};
use recof_core::journal::{OperationKind, Recovery};
use recof_core::manifest::{Channel, Product};
use recof_core::net::GithubClient;
use recof_core::paths::AppPaths;
use recof_core::space::{SpacePlan, TreeSize};
use recof_core::steam::shortcuts::{ShortcutEntry, SteamUser};
use recof_core::steam::{GameInstall, SteamLayout};
use recof_core::store::{StoredVersion, VersionStore};
use recof_core::update::{self, Available, FetchProgress};
use recof_core::version::Version;
use recof_core::Error;

/// Where the release a first install applies comes from.
#[derive(Debug, Clone)]
pub enum InstallRelease {
    /// Already in the version store.
    Stored(Box<StoredVersion>),
    /// Published on GitHub. Downloaded and checked before the copy starts.
    Download(Box<Available>),
    /// A release zip the user picked, with its signed manifest next to it.
    File(PathBuf),
}

/// Something for the worker to do.
pub enum Job {
    /// Find Steam, find the game, hash its files.
    Detect,
    /// Measure a tree and work out whether it fits at a destination.
    Measure {
        source: PathBuf,
        destination: PathBuf,
        app_data: PathBuf,
    },
    /// Get the release if it is not stored yet, copy the game and apply it.
    Install {
        source: PathBuf,
        source_buildid: Option<u64>,
        destination: PathBuf,
        measured: TreeSize,
        release: InstallRelease,
        app_data: PathBuf,
        import_from: Option<PathBuf>,
        now: String,
        id: String,
    },
    /// Finish or undo an interrupted operation.
    Recover { install_root: PathBuf },
    /// Hash the copy against its manifest.
    Verify { install_root: PathBuf },
    /// Write back the files a verify found wrong.
    Repair {
        install_root: PathBuf,
        release: Box<StoredVersion>,
        broken: Vec<String>,
        now: String,
        id: String,
    },
    /// Move the copy somewhere else.
    Move {
        from: PathBuf,
        to: PathBuf,
        measured: TreeSize,
    },
    /// Remove the copy.
    Uninstall {
        install_root: PathBuf,
        app_data: PathBuf,
        options: install::UninstallOptions,
    },

    /// Ask GitHub what the newest release is, for both products.
    CheckUpdates {
        app_data: PathBuf,
        channel: Channel,
        recof_installed: Option<Version>,
    },
    /// Back up, download, verify, store and apply a ReCoF release.
    Update {
        available: Box<Available>,
        install_root: PathBuf,
        app_data: PathBuf,
        source_buildid: Option<u64>,
        now: String,
        stamp: String,
        keep_versions: usize,
    },
    /// The same, from a file the user already has.
    InstallFromFile {
        archive: PathBuf,
        install_root: PathBuf,
        app_data: PathBuf,
        source_buildid: Option<u64>,
        now: String,
        stamp: String,
    },
    /// Go back to a version already in the store.
    Rollback {
        install_root: PathBuf,
        app_data: PathBuf,
        version: Version,
        now: String,
        stamp: String,
    },
    /// Download, verify and swap in a newer launcher.
    SelfUpdate {
        available: Box<Available>,
        app_data: PathBuf,
    },

    /// Back up the saves on request.
    Backup {
        install_root: PathBuf,
        app_data: PathBuf,
        kind: BackupKind,
        now: String,
        stamp: String,
        recof_version: Option<String>,
    },
    /// List the backups there are.
    ListBackups { app_data: PathBuf },
    /// Put one back.
    Restore {
        archive: PathBuf,
        install_root: PathBuf,
    },
    /// Work out what importing from the Steam copy would do.
    PlanImport {
        vanilla: PathBuf,
        install_root: PathBuf,
    },
    /// Carry it out, backing up first when it would replace anything.
    Import {
        vanilla: PathBuf,
        install_root: PathBuf,
        plan: Box<ImportPlan>,
        app_data: PathBuf,
        now: String,
        stamp: String,
    },

    /// Create a desktop or start menu shortcut.
    MakeShortcut {
        kind: ShortcutKind,
        install_root: PathBuf,
        arguments: String,
        name: String,
    },
    /// Add ReCoF to Steam as a non Steam game.
    AddToSteam {
        user: Box<SteamUser>,
        entry: Box<ShortcutEntry>,
        stamp: String,
    },
}

/// Where a shortcut goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutKind {
    Desktop,
    StartMenu,
}

/// What the worker found or did.
pub enum Event {
    Detected(Box<Detection>),
    Measured(Box<Measurement>),
    CopyProgress(install::CopyProgress),
    ApplyProgress {
        done: usize,
        total: usize,
        path: String,
        committing: bool,
    },
    CountProgress {
        done: usize,
        total: usize,
    },
    /// Downloading or unpacking a release.
    FetchProgress(FetchProgress),
    Installed(Box<InstalledState>),
    InstallCancelled,
    Recovered(Recovery),
    Verified(Box<RepairReport>),
    Repaired(usize),
    Moved(PathBuf),
    Uninstalled,

    UpdatesChecked(Box<UpdateCheck>),
    Updated(Box<InstalledState>),
    SelfUpdated(Version),

    BackedUp(Option<Box<Backup>>),
    Backups(Vec<Backup>),
    Restored(usize),
    ImportPlanned(Box<ImportPlan>),
    Imported(ImportReport),
    ShortcutMade,
    AddedToSteam,

    /// A job failed. The message id is what the user sees; the detail goes
    /// to the log and the bug report.
    Failed {
        message_id: &'static str,
        args: Vec<(&'static str, String)>,
        detail: String,
    },
}

/// Everything detection found.
pub struct Detection {
    pub steam: Option<SteamLayout>,
    pub game: Option<GameInstall>,
    pub report: Option<InstallReport>,
}

/// The size of a copy and whether it fits.
pub struct Measurement {
    pub size: TreeSize,
    pub plan: SpacePlan,
}

/// What a check of both repositories found.
#[derive(Default)]
pub struct UpdateCheck {
    pub recof: Option<Available>,
    pub launcher: Option<Available>,
    /// Set when GitHub could not be reached, which is a status line and
    /// not an error.
    pub offline: bool,
}

/// The UI side of the worker.
pub struct Worker {
    jobs: Sender<Job>,
    events: Receiver<Event>,
    /// Cloned for background jobs, which run on their own thread.
    event_sender: Sender<Event>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
    busy: bool,
    _thread: std::thread::JoinHandle<()>,
}

impl Worker {
    /// Starts the thread. `ctx` wakes the UI when an event arrives, so a
    /// job finishing does not wait for the next mouse move.
    pub fn start(ctx: egui::Context) -> Worker {
        let (job_sender, job_receiver) = std::sync::mpsc::channel::<Job>();
        let (event_sender, event_receiver) = std::sync::mpsc::channel::<Event>();
        let cancel = Arc::new(AtomicBool::new(false));

        let worker_cancel = Arc::clone(&cancel);
        let queue_sender = event_sender.clone();
        let queue_ctx = ctx.clone();
        let thread = std::thread::Builder::new()
            .name("recof-worker".to_owned())
            .spawn(move || {
                for job in job_receiver {
                    let emit = |event: Event| {
                        let _ = queue_sender.send(event);
                        queue_ctx.request_repaint();
                    };
                    run(job, &worker_cancel, &emit);
                }
            })
            .expect("a thread can be spawned");

        Worker {
            jobs: job_sender,
            events: event_receiver,
            event_sender,
            ctx,
            cancel,
            busy: false,
            _thread: thread,
        }
    }

    /// Queues a job. The cancel flag is cleared first, so a job never
    /// inherits a cancel meant for the previous one.
    pub fn send(&mut self, job: Job) {
        self.cancel
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.busy = true;
        if self.jobs.send(job).is_err() {
            log::error!("the worker thread is gone");
            self.busy = false;
        }
    }

    /// Runs a job on its own thread, without marking the worker busy.
    ///
    /// The update check uses this: it is a network request with a
    /// timeout, and neither the buttons nor an install the user starts
    /// a second later should wait for it.
    pub fn send_background(&self, job: Job) {
        let sender = self.event_sender.clone();
        let ctx = self.ctx.clone();
        // Its own flag: cancelling an install must not cancel this,
        // and this has nothing to cancel anyway.
        let cancel = Arc::new(AtomicBool::new(false));

        let spawned = std::thread::Builder::new()
            .name("recof-background".to_owned())
            .spawn(move || {
                let emit = |event: Event| {
                    let _ = sender.send(event);
                    ctx.request_repaint();
                };
                run(job, &cancel, &emit);
            });
        if let Err(error) = spawned {
            log::warn!("a background job could not start: {error}");
        }
    }

    /// Asks the running job to stop. Jobs that cannot be cancelled ignore
    /// it, which is why the flag is advisory rather than a kill.
    pub fn cancel(&self) {
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Everything the worker has sent since the last frame.
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            if !is_progress(&event) {
                self.busy = false;
            }
            events.push(event);
        }
        events
    }
}

fn is_progress(event: &Event) -> bool {
    matches!(
        event,
        Event::CopyProgress(_)
            | Event::ApplyProgress { .. }
            | Event::CountProgress { .. }
            | Event::FetchProgress(_)
    )
}

/// Turns an error into the event the UI shows.
fn failed(error: Error) -> Event {
    log::error!("{error}");
    Event::Failed {
        message_id: error.message_id(),
        args: error.message_args(),
        detail: error.to_string(),
    }
}

fn run(job: Job, cancel: &Arc<AtomicBool>, emit: &dyn Fn(Event)) {
    match job {
        Job::Detect => emit(detect()),

        Job::Measure {
            source,
            destination,
            app_data,
        } => emit(measure(&source, &destination, &app_data, cancel)),

        Job::Install {
            source,
            source_buildid,
            destination,
            measured,
            release,
            app_data,
            import_from,
            now,
            id,
        } => {
            // The release is fetched and checked first, so a download that
            // fails or is refused costs no copy of the game.
            let release = match store_release(release, &app_data, source_buildid, cancel, emit) {
                Ok(Some(release)) => release,
                Ok(None) => {
                    emit(Event::InstallCancelled);
                    return;
                }
                Err(error) => {
                    emit(failed(error));
                    return;
                }
            };

            let outcome = install::install(
                install::InstallRequest {
                    source: &source,
                    source_buildid,
                    destination: &destination,
                    measured,
                    release: &release,
                    now: &now,
                    id: &id,
                },
                cancel,
                &mut |stage| emit(stage_event(stage)),
            );

            match outcome {
                Ok(install::InstallOutcome::Installed(state)) => {
                    // Bringing the saves over is part of the install when
                    // the user asked for it, so it happens before the
                    // screen says the install is done.
                    if let Some(vanilla) = import_from {
                        if let Err(error) = import_now(&vanilla, &destination, emit) {
                            emit(failed(error));
                            return;
                        }
                    }
                    emit(Event::Installed(state));
                }
                Ok(install::InstallOutcome::Cancelled) => emit(Event::InstallCancelled),
                Err(error) => emit(failed(error)),
            }
        }

        Job::Recover { install_root } => emit(match install::recover(&install_root) {
            Ok(recovery) => Event::Recovered(recovery),
            Err(error) => failed(error),
        }),

        Job::Verify { install_root } => {
            let outcome = install::verify(&install_root, &mut |done, total| {
                emit(Event::CountProgress { done, total })
            });
            emit(match outcome {
                Ok(report) => Event::Verified(Box::new(report)),
                Err(error) => failed(error),
            });
        }

        Job::Repair {
            install_root,
            release,
            broken,
            now,
            id,
        } => {
            let borrowed: Vec<&str> = broken.iter().map(String::as_str).collect();
            let outcome = install::maintain::repair(
                &install_root,
                &release,
                &borrowed,
                &now,
                &id,
                &mut |progress| emit(journal_event(progress)),
            );
            emit(match outcome {
                Ok(count) => Event::Repaired(count),
                Err(error) => failed(error),
            });
        }

        Job::Move { from, to, measured } => {
            let outcome = install::move_install(&from, &to, measured, cancel, &mut |progress| {
                emit(Event::CopyProgress(progress.clone()))
            });
            emit(match outcome {
                Ok(install::maintain::MoveOutcome::Cancelled) => Event::InstallCancelled,
                Ok(_) => Event::Moved(to),
                Err(error) => failed(error),
            });
        }

        Job::Uninstall {
            install_root,
            app_data,
            options,
        } => {
            let paths = AppPaths::at(app_data);
            emit(match install::uninstall(&install_root, &paths, options) {
                Ok(_) => Event::Uninstalled,
                Err(error) => failed(error),
            });
        }

        Job::CheckUpdates {
            app_data,
            channel,
            recof_installed,
        } => emit(check_updates(&app_data, channel, recof_installed)),

        Job::Update {
            available,
            install_root,
            app_data,
            source_buildid,
            now,
            stamp,
            keep_versions,
        } => emit(update_recof(
            &available,
            &install_root,
            &app_data,
            source_buildid,
            &now,
            &stamp,
            keep_versions,
            cancel,
            emit,
        )),

        Job::InstallFromFile {
            archive,
            install_root,
            app_data,
            source_buildid,
            now,
            stamp,
        } => emit(install_from_file(
            &archive,
            &install_root,
            &app_data,
            source_buildid,
            &now,
            &stamp,
            emit,
        )),

        Job::Rollback {
            install_root,
            app_data,
            version,
            now,
            stamp,
        } => emit(rollback(
            &install_root,
            &app_data,
            &version,
            &now,
            &stamp,
            emit,
        )),

        Job::SelfUpdate {
            available,
            app_data,
        } => emit(self_update(&available, &app_data, cancel, emit)),

        Job::Backup {
            install_root,
            app_data,
            kind,
            now,
            stamp,
            recof_version,
        } => {
            let paths = AppPaths::at(app_data);
            let outcome = backup::create(
                &install_root,
                &paths.backups_dir(),
                kind,
                &now,
                &stamp,
                recof_version,
            );
            emit(match outcome {
                Ok(made) => Event::BackedUp(made.map(Box::new)),
                Err(error) => failed(error),
            });
        }

        Job::ListBackups { app_data } => {
            let paths = AppPaths::at(app_data);
            emit(match backup::list(&paths.backups_dir()) {
                Ok(found) => Event::Backups(found),
                Err(error) => failed(error),
            });
        }

        Job::Restore {
            archive,
            install_root,
        } => emit(match backup::restore(&archive, &install_root) {
            Ok(report) => Event::Restored(report.files),
            Err(error) => failed(error),
        }),

        Job::PlanImport {
            vanilla,
            install_root,
        } => emit(match backup::import::plan(&vanilla, &install_root) {
            Ok(plan) => Event::ImportPlanned(Box::new(plan)),
            Err(error) => failed(error),
        }),

        Job::Import {
            vanilla,
            install_root,
            plan,
            app_data,
            now,
            stamp,
        } => {
            // Anything that would be replaced is backed up first, so an
            // import can always be undone.
            if !plan.overwrites().is_empty() {
                let paths = AppPaths::at(&app_data);
                if let Err(error) = backup::create(
                    &install_root,
                    &paths.backups_dir(),
                    BackupKind::BeforeImport,
                    &now,
                    &stamp,
                    None,
                ) {
                    emit(failed(error));
                    return;
                }
            }
            let outcome =
                backup::import_from_vanilla(&vanilla, &install_root, &plan, &mut |done, total| {
                    emit(Event::CountProgress { done, total })
                });
            emit(match outcome {
                Ok(report) => Event::Imported(report),
                Err(error) => failed(error),
            });
        }

        Job::MakeShortcut {
            kind,
            install_root,
            arguments,
            name,
        } => emit(make_shortcut(kind, &install_root, &arguments, &name)),

        Job::AddToSteam { user, entry, stamp } => emit(
            match recof_core::steam::shortcuts::add(&user, &entry, &stamp) {
                Ok(backup) => {
                    log::info!("the previous shortcuts.vdf is at {}", backup.display());
                    Event::AddedToSteam
                }
                Err(problem) => {
                    log::warn!("{problem}");
                    Event::Failed {
                        message_id: problem.message_id(),
                        args: Vec::new(),
                        detail: problem.to_string(),
                    }
                }
            },
        ),
    }
}

fn detect() -> Event {
    let steam = match recof_core::steam::steam_path() {
        Ok(Some(path)) => match recof_core::steam::libraries(&path) {
            Ok(layout) => Some(layout),
            Err(error) => return failed(error),
        },
        Ok(None) => None,
        Err(error) => return failed(error),
    };

    let game = match &steam {
        Some(layout) => match recof_core::steam::detect::find_game_in(layout) {
            Ok(game) => game,
            Err(error) => return failed(error),
        },
        None => None,
    };

    let report = match &game {
        Some(game) => match recof_core::hashes::scan_install(&game.root) {
            Ok(report) => Some(report),
            // A scan that fails is worth logging but must not stop
            // detection: the user can still pick the folder by hand.
            Err(error) => {
                log::warn!("scanning {}: {error}", game.root.display());
                None
            }
        },
        None => None,
    };

    Event::Detected(Box::new(Detection {
        steam,
        game,
        report,
    }))
}

fn measure(source: &Path, destination: &Path, app_data: &Path, cancel: &Arc<AtomicBool>) -> Event {
    let cluster = match recof_platform::cluster_size(destination) {
        Ok(cluster) => cluster,
        Err(error) => {
            log::warn!("cluster size of {}: {error}", destination.display());
            4096
        }
    };

    let size = match recof_core::space::measure_tree(source, cluster, cancel, &mut |_| {}) {
        Ok(Ok(size)) => size,
        Ok(Err(_)) => return Event::InstallCancelled,
        Err(error) => return failed(error),
    };

    match SpacePlan::new(size, destination, app_data) {
        Ok(plan) => Event::Measured(Box::new(Measurement { size, plan })),
        Err(error) => failed(error),
    }
}

fn check_updates(app_data: &Path, channel: Channel, recof_installed: Option<Version>) -> Event {
    let paths = AppPaths::at(app_data);
    let client = GithubClient::new(paths.cache_dir());
    let running = Version::parse(recof_core::consts::LAUNCHER_VERSION).ok();

    let mut found = UpdateCheck::default();

    match update::check(
        &client,
        recof_core::consts::RECOF_REPO,
        Product::Recof,
        channel,
        recof_installed,
    ) {
        Ok(available) => found.recof = available.filter(Available::is_newer),
        Err(error) if error.is_only_offline() => {
            log::info!("ReCoF update check: {error}");
            found.offline = true;
        }
        Err(error) => log::warn!("ReCoF update check: {error}"),
    }

    match update::check(
        &client,
        recof_core::consts::LAUNCHER_REPO,
        Product::RecofLauncher,
        channel,
        running,
    ) {
        Ok(available) => found.launcher = available.filter(Available::is_newer),
        Err(error) if error.is_only_offline() => {
            log::info!("launcher update check: {error}");
            found.offline = true;
        }
        Err(error) => log::warn!("launcher update check: {error}"),
    }

    Event::UpdatesChecked(Box::new(found))
}

#[allow(clippy::too_many_arguments)]
fn update_recof(
    available: &Available,
    install_root: &Path,
    app_data: &Path,
    source_buildid: Option<u64>,
    now: &str,
    stamp: &str,
    keep_versions: usize,
    cancel: &Arc<AtomicBool>,
    emit: &dyn Fn(Event),
) -> Event {
    let paths = AppPaths::at(app_data);
    let store = VersionStore::new(paths.versions_dir());
    let running = match Version::parse(recof_core::consts::LAUNCHER_VERSION) {
        Ok(version) => version,
        Err(error) => {
            return failed(Error::Journal {
                detail: error.to_string(),
            })
        }
    };

    // The saves are backed up before anything is downloaded, so a failed
    // update has already left the player a way back.
    let installed = install::InstalledState::load(install_root).ok().flatten();
    match backup::create(
        install_root,
        &paths.backups_dir(),
        BackupKind::BeforeUpdate,
        now,
        stamp,
        installed.as_ref().map(|state| state.version.to_string()),
    ) {
        Ok(made) => emit(Event::BackedUp(made.map(Box::new))),
        Err(error) => return failed(error),
    }

    let fetched = match update::fetch(
        &available.release,
        Product::Recof,
        &paths.downloads_dir(),
        cancel,
        &mut |progress| emit(Event::FetchProgress(progress)),
    ) {
        Ok(Some(fetched)) => fetched,
        Ok(None) => return Event::InstallCancelled,
        Err(error) => return failed(error),
    };

    let stored = match update::verify_and_store(
        &fetched,
        Product::Recof,
        &running,
        source_buildid,
        &store,
        &mut |progress| emit(Event::FetchProgress(progress)),
    ) {
        Ok(stored) => stored,
        Err(error) => return failed(error),
    };

    let state = match install::apply_release(
        install_root,
        &stored,
        OperationKind::Update,
        now,
        &format!("update-{}", stored.version),
        &mut |progress| emit(journal_event(progress)),
    ) {
        Ok(state) => state,
        Err(error) => return failed(error),
    };

    // Older releases are dropped only once the new one is in place, and
    // never the one that is installed.
    if let Err(error) = store.prune(Product::Recof, keep_versions, Some(&state.version)) {
        log::warn!("the version store could not be pruned: {error}");
    }

    Event::Updated(Box::new(state))
}

/// Puts the release a first install needs in the version store.
///
/// Returns `None` when the download was cancelled.
fn store_release(
    release: InstallRelease,
    app_data: &Path,
    source_buildid: Option<u64>,
    cancel: &Arc<AtomicBool>,
    emit: &dyn Fn(Event),
) -> Result<Option<StoredVersion>, Error> {
    let paths = AppPaths::at(app_data);
    let store = VersionStore::new(paths.versions_dir());
    let running =
        Version::parse(recof_core::consts::LAUNCHER_VERSION).map_err(|error| Error::Journal {
            detail: error.to_string(),
        })?;
    let progress = &mut |progress: FetchProgress| emit(Event::FetchProgress(progress));

    match release {
        InstallRelease::Stored(stored) => Ok(Some(*stored)),
        InstallRelease::Download(available) => {
            let Some(fetched) = update::fetch(
                &available.release,
                Product::Recof,
                &paths.downloads_dir(),
                cancel,
                progress,
            )?
            else {
                return Ok(None);
            };
            update::verify_and_store(
                &fetched,
                Product::Recof,
                &running,
                source_buildid,
                &store,
                progress,
            )
            .map(Some)
        }
        InstallRelease::File(archive) => update::from_local_archive(
            &archive,
            Product::Recof,
            &running,
            source_buildid,
            &store,
            progress,
        )
        .map(Some),
    }
}

fn install_from_file(
    archive: &Path,
    install_root: &Path,
    app_data: &Path,
    source_buildid: Option<u64>,
    now: &str,
    stamp: &str,
    emit: &dyn Fn(Event),
) -> Event {
    let paths = AppPaths::at(app_data);
    let store = VersionStore::new(paths.versions_dir());
    let running = match Version::parse(recof_core::consts::LAUNCHER_VERSION) {
        Ok(version) => version,
        Err(error) => {
            return failed(Error::Journal {
                detail: error.to_string(),
            })
        }
    };

    if let Err(error) = backup::create(
        install_root,
        &paths.backups_dir(),
        BackupKind::BeforeUpdate,
        now,
        stamp,
        None,
    ) {
        return failed(error);
    }

    let stored = match update::from_local_archive(
        archive,
        Product::Recof,
        &running,
        source_buildid,
        &store,
        &mut |progress| emit(Event::FetchProgress(progress)),
    ) {
        Ok(stored) => stored,
        Err(error) => return failed(error),
    };

    match install::apply_release(
        install_root,
        &stored,
        OperationKind::Update,
        now,
        &format!("local-{}", stored.version),
        &mut |progress| emit(journal_event(progress)),
    ) {
        Ok(state) => Event::Updated(Box::new(state)),
        Err(error) => failed(error),
    }
}

fn rollback(
    install_root: &Path,
    app_data: &Path,
    version: &Version,
    now: &str,
    stamp: &str,
    emit: &dyn Fn(Event),
) -> Event {
    let paths = AppPaths::at(app_data);
    let store = VersionStore::new(paths.versions_dir());

    if let Err(error) = backup::create(
        install_root,
        &paths.backups_dir(),
        BackupKind::BeforeUpdate,
        now,
        stamp,
        None,
    ) {
        return failed(error);
    }

    let stored = match store.get(Product::Recof, version) {
        Ok(Some(stored)) => stored,
        Ok(None) => {
            return failed(Error::MissingAsset(format!(
                "ReCoF {version} is not in the version store"
            )))
        }
        Err(error) => return failed(error),
    };

    match install::apply_release(
        install_root,
        &stored,
        OperationKind::Rollback,
        now,
        &format!("rollback-{version}"),
        &mut |progress| emit(journal_event(progress)),
    ) {
        Ok(state) => Event::Updated(Box::new(state)),
        Err(error) => failed(error),
    }
}

fn self_update(
    available: &Available,
    app_data: &Path,
    cancel: &Arc<AtomicBool>,
    emit: &dyn Fn(Event),
) -> Event {
    let paths = AppPaths::at(app_data);
    let store = VersionStore::new(paths.versions_dir());
    let running = match Version::parse(recof_core::consts::LAUNCHER_VERSION) {
        Ok(version) => version,
        Err(error) => {
            return failed(Error::Journal {
                detail: error.to_string(),
            })
        }
    };

    let fetched = match update::fetch(
        &available.release,
        Product::RecofLauncher,
        &paths.downloads_dir(),
        cancel,
        &mut |progress| emit(Event::FetchProgress(progress)),
    ) {
        Ok(Some(fetched)) => fetched,
        Ok(None) => return Event::InstallCancelled,
        Err(error) => return failed(error),
    };

    let stored = match update::verify_and_store(
        &fetched,
        Product::RecofLauncher,
        &running,
        None,
        &store,
        &mut |progress| emit(Event::FetchProgress(progress)),
    ) {
        Ok(stored) => stored,
        Err(error) => return failed(error),
    };

    let pending = match update::selfupdate::pending(&stored) {
        Ok(pending) => pending,
        Err(error) => return failed(error),
    };
    if let Err(error) = update::selfupdate::apply(&pending.executable) {
        return failed(error);
    }

    Event::SelfUpdated(pending.version)
}

fn import_now(vanilla: &Path, install_root: &Path, emit: &dyn Fn(Event)) -> Result<(), Error> {
    let plan = backup::import::plan(vanilla, install_root)?;
    if plan.to_write() == 0 {
        return Ok(());
    }
    backup::import_from_vanilla(vanilla, install_root, &plan, &mut |done, total| {
        emit(Event::CountProgress { done, total })
    })?;
    Ok(())
}

fn make_shortcut(kind: ShortcutKind, install_root: &Path, arguments: &str, name: &str) -> Event {
    let folder = match kind {
        ShortcutKind::Desktop => recof_platform::desktop_dir(),
        ShortcutKind::StartMenu => recof_platform::start_menu_programs_dir(),
    };
    let folder = match folder {
        Ok(folder) => folder,
        Err(error) => return failed(error.into()),
    };

    let target = install_root.join(recof_core::consts::GAME_EXE);
    let link = folder.join(format!("{name}.lnk"));

    match recof_platform::create_shortcut(&recof_platform::Shortcut {
        link_path: &link,
        target: &target,
        working_dir: install_root,
        arguments,
        description: name,
    }) {
        Ok(()) => Event::ShortcutMade,
        Err(error) => failed(error.into()),
    }
}

fn stage_event(stage: install::InstallStage) -> Event {
    match stage {
        install::InstallStage::Copying(progress) => Event::CopyProgress(progress),
        install::InstallStage::Applying(progress) => journal_event(progress),
        install::InstallStage::Finished => Event::CountProgress { done: 1, total: 1 },
    }
}

fn journal_event(progress: recof_core::journal::Progress) -> Event {
    use recof_core::journal::Progress;
    match progress {
        Progress::Staging { done, total, path } => Event::ApplyProgress {
            done,
            total,
            path,
            committing: false,
        },
        Progress::Committing { done, total, path } => Event::ApplyProgress {
            done,
            total,
            path,
            committing: true,
        },
        Progress::Finished => Event::CountProgress { done: 1, total: 1 },
    }
}
