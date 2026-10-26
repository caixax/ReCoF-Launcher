//! The background thread.
//!
//! The UI thread never blocks. Everything that touches a disk, a registry
//! or the network goes here, and comes back as an [`Event`] the next frame
//! picks up. There is no async runtime: one worker thread, two channels,
//! and a repaint request so a running job keeps the window moving.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use recof_core::hashes::InstallReport;
use recof_core::install::{self, InstalledState, RepairReport};
use recof_core::journal::Recovery;
use recof_core::space::{SpacePlan, TreeSize};
use recof_core::steam::{GameInstall, SteamLayout};
use recof_core::store::StoredVersion;
use recof_core::Error;

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
    /// Copy the game and apply a release.
    Install {
        source: PathBuf,
        source_buildid: Option<u64>,
        destination: PathBuf,
        measured: TreeSize,
        release: Box<StoredVersion>,
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
}

/// What the worker found or did.
pub enum Event {
    Detected(Box<Detection>),
    Measured(Box<Measurement>),
    /// Copying the base game.
    CopyProgress(install::CopyProgress),
    /// Writing release files through the journal.
    ApplyProgress {
        done: usize,
        total: usize,
        path: String,
        committing: bool,
    },
    /// A step with no better progress than "N of M".
    CountProgress {
        done: usize,
        total: usize,
    },
    Installed(Box<InstalledState>),
    InstallCancelled,
    Recovered(Recovery),
    Verified(Box<RepairReport>),
    Repaired(usize),
    Moved(PathBuf),
    Uninstalled,
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

/// The UI side of the worker.
pub struct Worker {
    jobs: Sender<Job>,
    events: Receiver<Event>,
    cancel: Arc<AtomicBool>,
    busy: bool,
    _thread: std::thread::JoinHandle<()>,
}

impl Worker {
    /// Starts the thread. `ctx` is used to wake the UI when an event
    /// arrives, so a job finishing does not wait for the next mouse move.
    pub fn start(ctx: egui::Context) -> Worker {
        let (job_sender, job_receiver) = std::sync::mpsc::channel::<Job>();
        let (event_sender, event_receiver) = std::sync::mpsc::channel::<Event>();
        let cancel = Arc::new(AtomicBool::new(false));

        let worker_cancel = Arc::clone(&cancel);
        let thread = std::thread::Builder::new()
            .name("recof-worker".to_owned())
            .spawn(move || {
                for job in job_receiver {
                    let emit = |event: Event| {
                        let _ = event_sender.send(event);
                        ctx.request_repaint();
                    };
                    run(job, &worker_cancel, &emit);
                }
            })
            .expect("a thread can be spawned");

        Worker {
            jobs: job_sender,
            events: event_receiver,
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

    /// Asks the running job to stop. Jobs that cannot be cancelled ignore
    /// it, which is why the flag is advisory rather than a kill.
    pub fn cancel(&self) {
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    /// Whether a job is running, so the UI can disable the buttons that
    /// would start another one.
    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Everything the worker has sent since the last frame.
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            // Progress does not end a job; anything else does.
            if !matches!(
                event,
                Event::CopyProgress(_) | Event::ApplyProgress { .. } | Event::CountProgress { .. }
            ) {
                self.busy = false;
            }
            events.push(event);
        }
        events
    }
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
            now,
            id,
        } => {
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
            emit(match outcome {
                Ok(install::InstallOutcome::Installed(state)) => Event::Installed(state),
                Ok(install::InstallOutcome::Cancelled) => Event::InstallCancelled,
                Err(error) => failed(error),
            });
        }
        Job::Recover { install_root } => {
            emit(match install::recover(&install_root) {
                Ok(recovery) => Event::Recovered(recovery),
                Err(error) => failed(error),
            });
        }
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
            let paths = recof_core::paths::AppPaths::at(app_data);
            emit(match install::uninstall(&install_root, &paths, options) {
                Ok(_) => Event::Uninstalled,
                Err(error) => failed(error),
            });
        }
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

fn measure(
    source: &std::path::Path,
    destination: &std::path::Path,
    app_data: &std::path::Path,
    cancel: &Arc<AtomicBool>,
) -> Event {
    let cluster = match recof_platform::cluster_size(destination) {
        Ok(cluster) => cluster,
        // A destination whose volume cannot be queried still has a size,
        // and the NTFS default is the right guess.
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
