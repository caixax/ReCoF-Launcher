//! `cargo xtask smoke`: the whole install path against a published
//! release and the real game on this machine.
//!
//! It runs the code the launcher runs, in the order the launcher runs it:
//! find the game, ask GitHub for the newest release on a channel, read its
//! notes, download and verify it into a version store, copy the game and
//! apply the release, verify the copy, damage two files and repair them,
//! build the command line, and uninstall. The Steam install is listed and
//! hashed before and after, and the run fails if anything in it changed.
//!
//! Nothing here touches the launcher's own settings or store: everything
//! it keeps lives in a folder next to the destination, which goes away at
//! the end unless `--keep` is given.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use recof_core::changelog::{self, Block};
use recof_core::consts;
use recof_core::hashes::{sha256_file, RETAIL_1_6A};
use recof_core::install::{self, destination, InstallOutcome, InstallRequest, InstallStage};
use recof_core::launch::{self, CvarFile};
use recof_core::manifest::{safe_relative_path, Channel, Product};
use recof_core::net::GithubClient;
use recof_core::paths::AppPaths;
use recof_core::settings::LaunchSettings;
use recof_core::steam;
use recof_core::store::VersionStore;
use recof_core::update;
use recof_core::version::Version;

pub fn run(args: &[String]) -> Result<(), String> {
    let destination = PathBuf::from(
        value_of(args, "--dest")
            .ok_or("pass --dest <folder> for the copy, which must not exist")?,
    );
    let channel = match value_of(args, "--channel").as_deref() {
        None | Some("beta") => Channel::Beta,
        Some("stable") => Channel::Stable,
        Some(other) => return Err(format!("unknown channel {other:?}")),
    };
    let keep = args.iter().any(|arg| arg == "--keep");
    if destination.exists() {
        return Err(format!(
            "{} already exists; pick a folder that does not",
            destination.display()
        ));
    }

    let app_data = destination.with_file_name(format!(
        "{}.smoke-appdata",
        destination
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "recof".to_owned())
    ));
    let _ = std::fs::remove_dir_all(&app_data);
    let paths = AppPaths::at(&app_data);
    paths.create_all().map_err(|error| error.to_string())?;
    let store = VersionStore::new(paths.versions_dir());
    let running = Version::parse(consts::LAUNCHER_VERSION).map_err(|error| error.to_string())?;
    let cancel = Arc::new(AtomicBool::new(false));

    // The game, found the way the launcher finds it.
    let steam_path = steam::steam_path()
        .map_err(|error| error.to_string())?
        .ok_or("Steam is not installed")?;
    let layout = steam::libraries(&steam_path).map_err(|error| error.to_string())?;
    let game = steam::detect::find_game_in(&layout)
        .map_err(|error| error.to_string())?
        .ok_or("Cry of Fear is not installed in any Steam library")?;
    step(
        "game",
        format!("{} build {:?}", game.root.display(), game.buildid),
    );
    destination::validate(&destination, Some(&layout), Some(&game.root))
        .map_err(|problem| format!("the destination is refused: {problem:?}"))?;
    step(
        "destination",
        "accepted by the same rules the launcher applies",
    );

    let before = snapshot(&game.root)?;
    step(
        "steam before",
        format!(
            "{} files listed, in scope binaries hashed",
            before.files.len()
        ),
    );

    // The release, as the update check sees it.
    let client = GithubClient::new(paths.cache_dir());
    let available = update::check(&client, consts::RECOF_REPO, Product::Recof, channel, None)
        .map_err(|error| error.to_string())?
        .ok_or(format!("no release on the {} channel", channel.as_str()))?;
    step(
        "release",
        format!(
            "{} (version {}, prerelease {}), assets: {}",
            available.release.tag_name,
            available.version,
            available.release.prerelease,
            available
                .release
                .assets
                .iter()
                .map(|asset| asset.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    describe_notes(&available.changelog);

    // Download and verify, into the store.
    let fetched = update::fetch(
        &available.release,
        Product::Recof,
        &paths.downloads_dir(),
        &cancel,
        &mut |_| {},
    )
    .map_err(|error| format!("fetch: {error}"))?
    .ok_or("the download was cancelled")?;
    step(
        "fetch",
        format!(
            "archive {}, digest from GitHub {}",
            fetched
                .archive
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            fetched.asset_digest.as_deref().unwrap_or("none")
        ),
    );
    let stored = update::verify_and_store(
        &fetched,
        Product::Recof,
        &running,
        game.buildid,
        &store,
        &mut |_| {},
    )
    .map_err(|error| format!("verify: {error}"))?;
    step(
        "verify and store",
        format!(
            "{} {} on {}, {} files, signature checked",
            stored.product.as_str(),
            stored.version,
            stored.manifest.channel.as_str(),
            stored.manifest.files.len()
        ),
    );

    // The options the release describes.
    let cvars = match stored.manifest.cvars_file() {
        Some(name) => {
            let relative = safe_relative_path(name).ok_or("the cvar file name is unsafe")?;
            let file = CvarFile::load(&stored.files_dir().join(relative))
                .map_err(|error| format!("cvars: {error}"))?;
            step(
                "options",
                format!(
                    "{} in groups {}",
                    file.cvars.len(),
                    file.groups().join(", ")
                ),
            );
            Some(file)
        }
        None => {
            step("options", "the release ships none");
            None
        }
    };

    // Copy and apply.
    let cluster =
        recof_platform::cluster_size(destination.parent().unwrap_or(&destination)).unwrap_or(4096);
    let measured = recof_core::space::measure_tree(&game.root, cluster, &cancel, &mut |_| {})
        .map_err(|error| error.to_string())?
        .map_err(|_| "measuring was cancelled".to_owned())?;
    let started = std::time::Instant::now();
    let mut last_tenth = 0;
    let outcome = install::install(
        InstallRequest {
            source: &game.root,
            source_buildid: game.buildid,
            destination: &destination,
            measured,
            release: &stored,
            now: &crate::release::now_rfc3339(),
            id: "smoke-install",
        },
        &cancel,
        &mut |stage| {
            if let InstallStage::Copying(progress) = stage {
                let tenth = progress.bytes_done * 10 / progress.bytes_total.max(1);
                if tenth > last_tenth {
                    last_tenth = tenth;
                    println!("    copied {}0%", tenth);
                }
            }
        },
    )
    .map_err(|error| format!("install: {error}"))?;
    let InstallOutcome::Installed(state) = outcome else {
        return Err("the install was cancelled".to_owned());
    };
    step(
        "install",
        format!(
            "ReCoF {} in {} after {:.0?}",
            state.version,
            destination.display(),
            started.elapsed()
        ),
    );

    let report = install::verify(&destination, &mut |_, _| {}).map_err(|e| e.to_string())?;
    if !report.is_intact() {
        return Err(format!(
            "verify after install: broken {:?}",
            report.broken()
        ));
    }
    step(
        "verify",
        format!("{} files match the manifest", report.entries.len()),
    );

    // Damage two files and let repair put them back.
    let damaged = "cryoffear/cl_dlls/client.dll";
    let removed = "opengl32.dll";
    std::fs::write(destination.join(damaged), b"damaged on purpose")
        .map_err(|error| error.to_string())?;
    std::fs::remove_file(destination.join(removed)).map_err(|error| error.to_string())?;
    let report = install::verify(&destination, &mut |_, _| {}).map_err(|e| e.to_string())?;
    let mut broken = report.broken();
    broken.sort_unstable();
    if broken != [damaged, removed] {
        return Err(format!(
            "verify after damage found {broken:?}, expected {damaged} and {removed}"
        ));
    }
    step("damage", format!("verify finds {broken:?}"));
    let repaired = install::maintain::repair(
        &destination,
        &stored,
        &broken,
        &crate::release::now_rfc3339(),
        "smoke-repair",
        &mut |_| {},
    )
    .map_err(|error| format!("repair: {error}"))?;
    let report = install::verify(&destination, &mut |_, _| {}).map_err(|e| e.to_string())?;
    if !report.is_intact() {
        return Err(format!("verify after repair: broken {:?}", report.broken()));
    }
    step(
        "repair",
        format!("{repaired} files written back, verify is clean"),
    );

    // The command line the play button would use.
    let exe = destination.join(consts::GAME_EXE);
    let chosen = cvars.as_ref().map(CvarFile::defaults).unwrap_or_default();
    let cvar_args = cvars
        .as_ref()
        .map(|file| file.to_args(&chosen))
        .unwrap_or_default();
    let line = launch::preview(
        &exe.display().to_string(),
        &launch::build(&LaunchSettings::default(), &cvar_args),
    );
    if line.len() > launch::MAX_COMMAND_LINE {
        return Err(format!(
            "with every option set the command line is {} bytes, over the limit",
            line.len()
        ));
    }
    step(
        "command line",
        format!("{} bytes with every option set: {line}", line.len()),
    );

    let after = snapshot(&game.root)?;
    if let Some(difference) = before.difference(&after) {
        return Err(format!("the Steam install changed: {difference}"));
    }
    step("steam after", "unchanged, file for file and hash for hash");

    if keep {
        step(
            "kept",
            format!(
                "{} and {} are left in place",
                destination.display(),
                app_data.display()
            ),
        );
    } else {
        install::uninstall(
            &destination,
            &paths,
            install::UninstallOptions {
                remove_version_store: true,
                remove_backups: true,
            },
        )
        .map_err(|error| format!("uninstall: {error}"))?;
        if destination.exists() {
            return Err(format!("{} is still there", destination.display()));
        }
        let _ = std::fs::remove_dir_all(&app_data);
        step("uninstall", "the copy is gone");
    }

    println!();
    println!("smoke: everything passed");
    Ok(())
}

fn step(name: &str, detail: impl AsRef<str>) {
    println!("ok  {name:<17} {}", detail.as_ref());
}

/// What the notes are made of, to see they read as more than one paragraph.
fn describe_notes(body: &str) {
    let blocks = changelog::parse(body);
    let count = |test: fn(&Block) -> bool| blocks.iter().filter(|block| test(block)).count();
    let alerts: Vec<String> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Quote {
                alert: Some(alert), ..
            } => Some(format!("{alert:?}")),
            _ => None,
        })
        .collect();
    step(
        "notes",
        format!(
            "{} blocks: {} headings, {} list items, {} tables, alerts {:?}",
            blocks.len(),
            count(|block| matches!(block, Block::Heading { .. })),
            count(|block| matches!(block, Block::Item { .. })),
            count(|block| matches!(block, Block::Table { .. })),
            alerts
        ),
    );
}

/// Every file in an install with its size and time, and the hash of each
/// binary in scope.
struct Snapshot {
    files: Vec<(String, u64, Option<std::time::SystemTime>)>,
    hashes: Vec<(String, String)>,
}

impl Snapshot {
    fn difference(&self, other: &Snapshot) -> Option<String> {
        if self.files.len() != other.files.len() {
            return Some(format!(
                "{} files before, {} after",
                self.files.len(),
                other.files.len()
            ));
        }
        for (before, after) in self.files.iter().zip(&other.files) {
            if before != after {
                return Some(format!("{before:?} became {after:?}"));
            }
        }
        for (before, after) in self.hashes.iter().zip(&other.hashes) {
            if before != after {
                return Some(format!("{} hashed differently", before.0));
            }
        }
        None
    }
}

fn snapshot(root: &Path) -> Result<Snapshot, String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root).sort_by_file_name() {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned();
        files.push((relative, metadata.len(), metadata.modified().ok()));
    }

    let mut hashes = Vec::new();
    for known in RETAIL_1_6A {
        let path = root.join(known.path);
        if path.is_file() {
            hashes.push((
                known.path.to_owned(),
                sha256_file(&path).map_err(|error| error.to_string())?,
            ));
        }
    }
    Ok(Snapshot { files, hashes })
}

fn value_of(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
}
