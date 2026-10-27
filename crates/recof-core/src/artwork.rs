//! Fetching the artwork, so nobody has to go and find it.
//!
//! The launcher ships no picture of the game and the repository holds
//! none: rule 3, and the art is not this project's to redistribute. What
//! it does instead is fetch it, once, from the assets of the newest ReCoF
//! release, which is a repository the project owner controls and where
//! the decision about what may be published belongs.
//!
//! So a player installs the launcher and the artwork is simply there.
//! Nothing was bundled, nothing was committed, and whoever made each
//! picture is named on screen.
//!
//! A release opts in by attaching files with these names:
//!
//! | Asset | What it becomes |
//! |---|---|
//! | `artwork-hero.png` or `.jpg` | the background |
//! | `artwork-logo.png` | the title |
//! | `artwork-icon.png` | the window icon |
//! | `artwork-credits.txt` | who made them, shown in the launcher |
//!
//! A release that attaches none of them leaves the launcher drawing its
//! own, which is the default and always works offline.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::net::{self, DownloadOutcome, GithubClient, NetError, Release};

/// Prefix every artwork asset carries, so they cannot be confused with
/// the release archive or its manifest.
pub const PREFIX: &str = "artwork-";

/// The credits file, kept beside the pictures it describes.
pub const CREDITS_FILE: &str = "credits.txt";

/// Largest a picture may be. A 4K hero as a PNG is a few megabytes.
const MAX_BYTES: u64 = 24 * 1024 * 1024;

/// One picture a release can offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wanted {
    /// Asset names tried in the release, best first.
    pub assets: &'static [&'static str],
    /// What it is saved as in the theme folder.
    pub saved_as: &'static str,
}

/// Everything the launcher will take from a release.
pub const WANTED: &[Wanted] = &[
    Wanted {
        assets: &["artwork-hero.png", "artwork-hero.jpg"],
        saved_as: "hero.png",
    },
    Wanted {
        assets: &["artwork-logo.png"],
        saved_as: "logo.png",
    },
    Wanted {
        assets: &["artwork-icon.png"],
        saved_as: "icon.png",
    },
];

/// What a fetch did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fetched {
    /// Files written into the theme folder.
    pub written: Vec<String>,
    /// The credits, when the release published them.
    pub credits: Option<String>,
}

impl Fetched {
    pub fn is_empty(&self) -> bool {
        self.written.is_empty()
    }
}

/// Whether the theme folder already holds something, in which case the
/// launcher leaves it alone: a picture the user put there by hand is
/// theirs, and overwriting it on every start would be rude.
pub fn already_present(theme_dir: &Path) -> bool {
    WANTED
        .iter()
        .any(|wanted| theme_dir.join(wanted.saved_as).is_file())
}

/// Downloads whatever artwork the newest release offers.
///
/// Returns an empty result when the release publishes none, which is not
/// a failure: the launcher draws its own and says nothing.
pub fn fetch(
    client: &GithubClient,
    repo: &str,
    channel: crate::manifest::Channel,
    theme_dir: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<Fetched, NetError> {
    let Some(release) = client.newest(repo, channel)? else {
        return Ok(Fetched::default());
    };
    fetch_from(&release, theme_dir, cancel)
}

/// The same, from a release that has already been read.
pub fn fetch_from(
    release: &Release,
    theme_dir: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<Fetched, NetError> {
    std::fs::create_dir_all(theme_dir).map_err(|error| NetError::Io(error.to_string()))?;

    let mut fetched = Fetched::default();

    for wanted in WANTED {
        let Some(asset) = wanted.assets.iter().find_map(|name| release.asset(name)) else {
            continue;
        };
        if asset.size > MAX_BYTES {
            log::warn!(
                "{} is {} bytes, too large for artwork",
                asset.name,
                asset.size
            );
            continue;
        }

        let outcome = net::download(
            &asset.browser_download_url,
            theme_dir,
            wanted.saved_as,
            asset.size,
            cancel,
            &mut |_| {},
        );
        match outcome {
            Ok(DownloadOutcome::Downloaded { .. }) => {
                log::info!("fetched {} as {}", asset.name, wanted.saved_as);
                fetched.written.push(wanted.saved_as.to_owned());
            }
            Ok(DownloadOutcome::Cancelled) => return Ok(fetched),
            // One picture failing is not a reason to go without the rest.
            Err(error) => log::warn!("{} could not be fetched: {error}", asset.name),
        }
    }

    if let Some(asset) = release.asset("artwork-credits.txt") {
        let outcome = net::download(
            &asset.browser_download_url,
            theme_dir,
            CREDITS_FILE,
            asset.size.min(64 * 1024),
            cancel,
            &mut |_| {},
        );
        if let Ok(DownloadOutcome::Downloaded { path, .. }) = outcome {
            fetched.credits = std::fs::read_to_string(path).ok();
        }
    }

    Ok(fetched)
}

/// The credits for whatever is in the theme folder.
///
/// Read fresh rather than remembered, so a user who edits the file after
/// adding their own picture sees their own line.
pub fn credits(theme_dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(theme_dir.join(CREDITS_FILE)).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Bounded, because it ends up in a label on screen.
    Some(trimmed.chars().take(2000).collect())
}

/// Where a picture in the theme folder came from, as far as the launcher
/// can tell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Fetched from a release.
    Release,
    /// Put there by the user.
    User,
    /// Nothing is there.
    None,
}

/// Whether the folder was filled by a fetch or by hand.
///
/// A credits file means the launcher wrote it; anything else means the
/// user did, and the launcher keeps its hands off.
pub fn source(theme_dir: &Path) -> Source {
    if !already_present(theme_dir) {
        return Source::None;
    }
    if theme_dir.join(CREDITS_FILE).is_file() {
        Source::Release
    } else {
        Source::User
    }
}

/// Removes what a fetch wrote, so the launcher goes back to its own look
/// or picks the artwork up again next time.
pub fn clear(theme_dir: &Path) -> std::io::Result<()> {
    for wanted in WANTED {
        let path = theme_dir.join(wanted.saved_as);
        if path.is_file() {
            std::fs::remove_file(path)?;
        }
    }
    let credits = theme_dir.join(CREDITS_FILE);
    if credits.is_file() {
        std::fs::remove_file(credits)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join("recof-artwork-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn every_wanted_asset_carries_the_prefix() {
        // Without it an artwork asset could collide with the release
        // archive or the manifest.
        for wanted in WANTED {
            for name in wanted.assets {
                assert!(name.starts_with(PREFIX), "{name}");
            }
            assert!(!wanted.saved_as.starts_with(PREFIX), "{}", wanted.saved_as);
        }
    }

    #[test]
    fn what_is_saved_matches_what_the_launcher_looks_for() {
        // The names here have to be the ones art.rs reads, or a fetch
        // would write files nothing ever uses.
        let saved: Vec<&str> = WANTED.iter().map(|wanted| wanted.saved_as).collect();
        assert!(saved.contains(&"hero.png"));
        assert!(saved.contains(&"logo.png"));
        assert!(saved.contains(&"icon.png"));
    }

    #[test]
    fn an_empty_folder_has_nothing_and_no_source() {
        let dir = scratch("empty");
        assert!(!already_present(&dir));
        assert_eq!(source(&dir), Source::None);
        assert_eq!(credits(&dir), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_picture_the_user_added_is_recognised_as_theirs() {
        let dir = scratch("user-art");
        std::fs::write(dir.join("hero.png"), b"not really a png").unwrap();

        assert!(already_present(&dir));
        assert_eq!(source(&dir), Source::User);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_fetched_set_is_recognised_by_its_credits() {
        let dir = scratch("fetched-art");
        std::fs::write(dir.join("hero.png"), b"x").unwrap();
        std::fs::write(dir.join(CREDITS_FILE), "Hero by somebody\n").unwrap();

        assert_eq!(source(&dir), Source::Release);
        assert_eq!(credits(&dir).as_deref(), Some("Hero by somebody"));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn empty_credits_read_as_none_rather_than_a_blank_line() {
        let dir = scratch("blank-credits");
        std::fs::write(dir.join(CREDITS_FILE), "   \n\n").unwrap();
        assert_eq!(credits(&dir), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn very_long_credits_are_cut_rather_than_filling_the_screen() {
        let dir = scratch("long-credits");
        std::fs::write(dir.join(CREDITS_FILE), "x".repeat(10_000)).unwrap();
        assert_eq!(credits(&dir).unwrap().chars().count(), 2000);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn clearing_removes_what_a_fetch_wrote_and_nothing_else() {
        let dir = scratch("clear");
        std::fs::write(dir.join("hero.png"), b"x").unwrap();
        std::fs::write(dir.join("logo.png"), b"x").unwrap();
        std::fs::write(dir.join(CREDITS_FILE), "by somebody").unwrap();
        // The note the launcher writes explaining the folder stays.
        std::fs::write(dir.join("README.txt"), b"what goes here").unwrap();
        // And so does anything else the user keeps there.
        std::fs::write(dir.join("my-notes.txt"), b"mine").unwrap();

        clear(&dir).unwrap();

        assert!(!dir.join("hero.png").exists());
        assert!(!dir.join("logo.png").exists());
        assert!(!dir.join(CREDITS_FILE).exists());
        assert!(dir.join("README.txt").is_file());
        assert!(dir.join("my-notes.txt").is_file());
        assert_eq!(source(&dir), Source::None);

        // Clearing an already empty folder is fine.
        clear(&dir).unwrap();

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_release_offering_no_artwork_fetches_nothing_and_is_not_an_error() {
        let dir = scratch("no-artwork");
        let release = Release {
            tag_name: "v0.1.0".to_owned(),
            name: None,
            body: None,
            draft: false,
            prerelease: false,
            published_at: None,
            html_url: None,
            assets: vec![crate::net::Asset {
                name: "recof-0.1.0.zip".to_owned(),
                size: 10,
                browser_download_url: "https://example.invalid/a.zip".to_owned(),
                digest: None,
            }],
        };

        let fetched = fetch_from(&release, &dir, &Arc::new(AtomicBool::new(false))).unwrap();
        assert!(fetched.is_empty());
        assert_eq!(fetched.credits, None);
        assert!(!already_present(&dir));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_oversized_asset_is_skipped_rather_than_downloaded() {
        let dir = scratch("huge");
        let release = Release {
            tag_name: "v0.1.0".to_owned(),
            name: None,
            body: None,
            draft: false,
            prerelease: false,
            published_at: None,
            html_url: None,
            assets: vec![crate::net::Asset {
                name: "artwork-hero.png".to_owned(),
                size: MAX_BYTES + 1,
                // Nothing listens here, so downloading it would fail
                // anyway; the point is that it is never attempted.
                browser_download_url: "http://127.0.0.1:1/hero.png".to_owned(),
                digest: None,
            }],
        };

        let fetched = fetch_from(&release, &dir, &Arc::new(AtomicBool::new(false))).unwrap();
        assert!(fetched.is_empty());
        assert!(!dir.join("hero.png").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_download_that_fails_leaves_the_rest_alone() {
        let dir = scratch("one-fails");
        let release = Release {
            tag_name: "v0.1.0".to_owned(),
            name: None,
            body: None,
            draft: false,
            prerelease: false,
            published_at: None,
            html_url: None,
            assets: vec![crate::net::Asset {
                name: "artwork-hero.png".to_owned(),
                size: 100,
                browser_download_url: "http://127.0.0.1:1/hero.png".to_owned(),
                digest: None,
            }],
        };

        // The whole call still succeeds: artwork is decoration, and a
        // launcher that refuses to start because a picture is missing
        // would be worse than one with no picture.
        let fetched = fetch_from(&release, &dir, &Arc::new(AtomicBool::new(false))).unwrap();
        assert!(fetched.is_empty());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
