//! Asks the real GitHub what the launcher would see.
//!
//! The update path is the one thing that cannot be proved by a test
//! against a local server: what matters is what the live repository
//! actually publishes, which channel it says it is on, and whether the
//! signature over the manifest still checks against the key this build
//! trusts. This runs that whole path and prints what it found, without
//! installing anything.

use recof_core::consts::{LAUNCHER_REPO, LAUNCHER_VERSION, RECOF_REPO};
use recof_core::manifest::{verify_manifest, Channel, Product};
use recof_core::net::github::GithubClient;
use recof_core::update;
use recof_core::version::Version;

pub fn run(args: &[String]) -> Result<(), String> {
    // A cache would answer from whatever was fetched up to six hours
    // ago, which is exactly the wrong thing when the question is what
    // is published right now.
    let cache = std::env::temp_dir().join("recof-xtask-update-check");
    let _ = std::fs::remove_dir_all(&cache);
    std::fs::create_dir_all(&cache).map_err(|error| format!("{}: {error}", cache.display()))?;

    let running = Version::parse(LAUNCHER_VERSION)
        .map_err(|error| format!("{LAUNCHER_VERSION} is not a version: {error}"))?;
    println!("launcher {running}\n");

    let only = args
        .iter()
        .position(|flag| flag == "--repo")
        .and_then(|at| args.get(at + 1))
        .cloned();

    let client = GithubClient::new(&cache);
    let mut failures = 0;

    for (product, repo) in [
        (Product::Recof, RECOF_REPO),
        (Product::RecofLauncher, LAUNCHER_REPO),
    ] {
        if only.as_deref().is_some_and(|wanted| wanted != repo) {
            continue;
        }
        println!("{repo}");
        for channel in [Channel::Stable, Channel::Beta] {
            match one(&client, repo, product, channel, &running, &cache) {
                Ok(()) => {}
                Err(message) => {
                    println!("  {channel:?}: {message}");
                    failures += 1;
                }
            }
        }
        println!();
    }

    if failures > 0 {
        return Err(format!("{failures} checks could not be completed"));
    }
    Ok(())
}

fn one(
    client: &GithubClient,
    repo: &str,
    product: Product,
    channel: Channel,
    running: &Version,
    cache: &std::path::Path,
) -> Result<(), String> {
    let found = update::check(client, repo, product, channel, None)
        .map_err(|error| format!("the check failed: {error}"))?;

    let Some(available) = found else {
        // Not a fault: a repository whose releases are all prereleases
        // has nothing on stable, and that is the correct answer.
        println!("  {channel:?}: nothing published");
        return Ok(());
    };

    let release = &available.release;
    println!(
        "  {channel:?}: {} -> {} ({})",
        release.tag_name,
        available.version,
        if release.prerelease {
            "prerelease"
        } else {
            "release"
        }
    );

    // Level one of the three, against the key compiled into this build.
    let manifest_name = update::manifest_name(product);
    let signature_name = format!("{manifest_name}.sig");

    let manifest = asset_bytes(release, manifest_name, cache)?;
    let signature = asset_bytes(release, &signature_name, cache)?;

    match verify_manifest(&manifest, &signature, product, running) {
        Ok(verified) => {
            let manifest = verified.manifest();
            println!(
                "    signature ok, signed by {}, manifest says {} on {:?}",
                verified.signed_by, manifest.version, manifest.channel
            );
            println!(
                "    archive {} ({} bytes), {} files, needs launcher {} or newer",
                manifest.archive.name,
                manifest.archive.size,
                manifest.files.len(),
                manifest.min_launcher
            );

            // The launcher reads the channel from the release, and the
            // manifest states one of its own. A release that disagrees
            // with its manifest would offer a beta to a stable user.
            let release_channel = if release.prerelease {
                Channel::Beta
            } else {
                Channel::Stable
            };
            if manifest.channel != release_channel {
                println!(
                    "    MISMATCH: the release is a {release_channel:?} but the manifest says {:?}",
                    manifest.channel
                );
            }
            if manifest.version != available.version {
                println!(
                    "    MISMATCH: the tag is {} but the manifest says {}",
                    available.version, manifest.version
                );
            }
        }
        Err(error) => return Err(format!("the signature did not check out: {error}")),
    }
    Ok(())
}

/// Downloads one asset of a release into the scratch folder.
fn asset_bytes(
    release: &recof_core::net::github::Release,
    name: &str,
    cache: &std::path::Path,
) -> Result<Vec<u8>, String> {
    let asset = release
        .asset(name)
        .ok_or_else(|| format!("the release has no {name}"))?;

    let into = cache.join(&release.tag_name);
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let outcome = recof_core::net::download(
        &asset.browser_download_url,
        &into,
        name,
        asset.size,
        &cancel,
        &mut |_| {},
    )
    .map_err(|error| format!("{name} could not be downloaded: {error}"))?;

    match outcome {
        recof_core::net::download::DownloadOutcome::Downloaded { path, .. } => {
            std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))
        }
        recof_core::net::download::DownloadOutcome::Cancelled => {
            Err(format!("{name} was cancelled, which cannot happen here"))
        }
    }
}
