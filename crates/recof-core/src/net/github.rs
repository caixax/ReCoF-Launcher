//! Reading GitHub Releases.
//!
//! One unauthenticated request per repository per start, and not even
//! that when the cached answer is recent: the reply is kept with its
//! ETag, so a repeat check costs a conditional request that GitHub
//! answers with 304 and no body, and a check within the cache window
//! costs nothing at all. Unauthenticated callers get sixty requests an
//! hour, which this stays far inside.
//!
//! Nothing read here is trusted. The release tells the launcher which
//! file to download; the signed manifest decides whether to use it.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::manifest::Channel;
use crate::net::NetError;
use crate::version::Version;

/// How long a cached answer is used without asking GitHub again.
pub const CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// Largest reply the launcher will read. A release list is a few tens of
/// kilobytes; anything far beyond that is not something to parse.
const MAX_REPLY_BYTES: u64 = 4 * 1024 * 1024;

/// How long to wait for the whole request.
const TIMEOUT: Duration = Duration::from_secs(20);

/// GitHub asks that clients identify themselves.
fn user_agent() -> String {
    format!(
        "ReCoF-Launcher/{} (+https://github.com/{})",
        crate::consts::LAUNCHER_VERSION,
        crate::consts::LAUNCHER_REPO
    )
}

/// One file attached to a release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    pub browser_download_url: String,
    /// GitHub's own digest for the asset, as `sha256:<hex>`. Compared with
    /// the manifest as a second opinion; it is not what establishes trust.
    #[serde(default)]
    pub digest: Option<String>,
}

/// One release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub name: Option<String>,
    /// The release notes, shown as the changelog before updating.
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

impl Release {
    /// The version the tag names, if it is a version at all.
    pub fn version(&self) -> Option<Version> {
        Version::parse(&self.tag_name).ok()
    }

    /// An asset by exact name, compared without case.
    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets
            .iter()
            .find(|asset| asset.name.eq_ignore_ascii_case(name))
    }

    /// The first asset whose name ends with a suffix, for finding the zip
    /// and the manifest without knowing the version in advance.
    pub fn asset_ending_with(&self, suffix: &str) -> Option<&Asset> {
        self.assets
            .iter()
            .find(|asset| asset.name.to_lowercase().ends_with(&suffix.to_lowercase()))
    }

    /// Whether this release belongs to a channel.
    pub fn is_in(&self, channel: Channel) -> bool {
        if self.draft {
            return false;
        }
        match channel {
            Channel::Stable => !self.prerelease,
            // Beta sees everything, so a user on beta is never offered
            // something older than the stable release.
            Channel::Beta => true,
        }
    }
}

/// What was cached for one endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedReply {
    etag: Option<String>,
    /// Unix seconds when the body was last known to be current.
    fetched_at: u64,
    body: String,
}

/// A client for one GitHub host, with its cache.
pub struct GithubClient {
    agent: ureq::Agent,
    cache_dir: PathBuf,
    /// Base URL, so the tests can point at something local.
    base: String,
    /// Set to false in tests that must not reach the network.
    offline: bool,
}

impl GithubClient {
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .user_agent(user_agent())
            .max_redirects(5)
            .build();

        GithubClient {
            agent: config.into(),
            cache_dir: cache_dir.into(),
            base: "https://api.github.com".to_owned(),
            offline: false,
        }
    }

    /// Points the client at another host, for tests.
    pub fn with_base(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }

    /// Refuses to touch the network, so a test can exercise the cache.
    pub fn offline(mut self) -> Self {
        self.offline = true;
        self
    }

    /// The newest release on a channel, or nothing when there is none.
    ///
    /// Stable asks for `releases/latest`, which GitHub already filters to
    /// skip drafts and prereleases. Beta reads the list, because a
    /// prerelease never appears as "latest".
    pub fn newest(&self, repo: &str, channel: Channel) -> Result<Option<Release>, NetError> {
        match channel {
            Channel::Stable => match self.latest(repo) {
                Ok(release) => Ok(Some(release)),
                Err(NetError::NotFound) => Ok(None),
                Err(error) => Err(error),
            },
            Channel::Beta => {
                let mut releases = self.list(repo)?;
                releases.retain(|release| release.is_in(Channel::Beta));
                // Ordered by version where the tag is one, so a release
                // published out of order does not win.
                releases.sort_by(|left, right| match (left.version(), right.version()) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    _ => left.published_at.cmp(&right.published_at),
                });
                Ok(releases.pop())
            }
        }
    }

    /// `GET /repos/{repo}/releases/latest`.
    pub fn latest(&self, repo: &str) -> Result<Release, NetError> {
        let url = format!("{}/repos/{repo}/releases/latest", self.base);
        let body = self.get_cached(&url, &cache_name(repo, "latest"))?;
        serde_json::from_str(&body).map_err(|error| NetError::BadReply(error.to_string()))
    }

    /// `GET /repos/{repo}/releases`.
    pub fn list(&self, repo: &str) -> Result<Vec<Release>, NetError> {
        let url = format!("{}/repos/{repo}/releases?per_page=30", self.base);
        let body = self.get_cached(&url, &cache_name(repo, "list"))?;
        serde_json::from_str(&body).map_err(|error| NetError::BadReply(error.to_string()))
    }

    /// A GET that uses the cache, and a conditional request when it is
    /// stale.
    fn get_cached(&self, url: &str, cache_key: &str) -> Result<String, NetError> {
        let cache_path = self.cache_dir.join(cache_key);
        let cached = read_cache(&cache_path);

        if let Some(cached) = &cached {
            if now_seconds().saturating_sub(cached.fetched_at) < CACHE_TTL.as_secs() {
                log::debug!("{url}: using the cached reply");
                return Ok(cached.body.clone());
            }
        }

        if self.offline {
            return match cached {
                Some(cached) => Ok(cached.body),
                None => Err(NetError::Offline),
            };
        }

        let mut request = self
            .agent
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(etag) = cached.as_ref().and_then(|cached| cached.etag.clone()) {
            request = request.header("If-None-Match", etag);
        }

        let response = match request.call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(code)) => return Err(self.status_error(code, cached)),
            Err(error) => {
                log::warn!("{url}: {error}");
                // A network failure falls back to whatever is cached: the
                // launcher works offline, it just cannot see new releases.
                return match cached {
                    Some(cached) => Ok(cached.body),
                    None => Err(NetError::Offline),
                };
            }
        };

        let status = response.status().as_u16();
        let etag = response
            .headers()
            .get("etag")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);

        if status == 304 {
            log::debug!("{url}: not modified");
            if let Some(mut cached) = cached {
                cached.fetched_at = now_seconds();
                write_cache(&cache_path, &cached);
                return Ok(cached.body);
            }
            return Err(NetError::BadReply("304 with nothing cached".to_owned()));
        }

        let mut response = response;
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_REPLY_BYTES)
            .read_to_string()
            .map_err(|error| NetError::BadReply(error.to_string()))?;

        write_cache(
            &cache_path,
            &CachedReply {
                etag,
                fetched_at: now_seconds(),
                body: body.clone(),
            },
        );
        Ok(body)
    }

    fn status_error(&self, code: u16, cached: Option<CachedReply>) -> NetError {
        match code {
            404 => NetError::NotFound,
            // GitHub answers 403 or 429 when the hourly limit is spent.
            403 | 429 => NetError::RateLimited,
            _ => {
                if let Some(cached) = cached {
                    log::warn!("GitHub answered {code}, using the cached reply");
                    return NetError::Http {
                        code,
                        cached_body: Some(cached.body),
                    };
                }
                NetError::Http {
                    code,
                    cached_body: None,
                }
            }
        }
    }
}

/// A file name for the cached reply, safe on any filesystem.
fn cache_name(repo: &str, endpoint: &str) -> String {
    let slug: String = repo
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect();
    format!("github-{slug}-{endpoint}.json")
}

fn read_cache(path: &Path) -> Option<CachedReply> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn write_cache(path: &Path, reply: &CachedReply) {
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    match serde_json::to_vec(reply) {
        Ok(bytes) => {
            if let Err(error) = std::fs::write(path, bytes) {
                log::warn!("the reply cache could not be written: {error}");
            }
        }
        Err(error) => log::warn!("the reply cache could not be serialised: {error}"),
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-github-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn release(tag: &str, prerelease: bool) -> Release {
        Release {
            tag_name: tag.to_owned(),
            name: Some(tag.to_owned()),
            body: Some("notes".to_owned()),
            draft: false,
            prerelease,
            published_at: Some("2026-10-26T12:00:00Z".to_owned()),
            html_url: None,
            assets: vec![Asset {
                name: format!("recof-{}.zip", tag.trim_start_matches('v')),
                size: 100,
                browser_download_url: "https://example.invalid/recof.zip".to_owned(),
                digest: Some("sha256:abc".to_owned()),
            }],
        }
    }

    #[test]
    fn a_release_reply_parses_including_the_fields_the_launcher_uses() {
        // Trimmed down from a real reply, with fields the launcher does
        // not read left in so the parser is shown to ignore them.
        let json = r#"{
            "url": "https://api.github.com/repos/caixax/ReCoF/releases/1",
            "id": 1,
            "tag_name": "v0.3.0",
            "name": "ReCoF 0.3.0",
            "draft": false,
            "prerelease": false,
            "published_at": "2026-10-26T12:00:00Z",
            "html_url": "https://github.com/caixax/ReCoF/releases/tag/v0.3.0",
            "body": "Fixed the flashlight.",
            "author": { "login": "caixax", "id": 2 },
            "assets": [
                {
                    "name": "recof-0.3.0.zip",
                    "size": 4200000,
                    "content_type": "application/zip",
                    "browser_download_url": "https://github.com/x/recof-0.3.0.zip",
                    "digest": "sha256:aaaa"
                },
                {
                    "name": "recof-manifest.json",
                    "size": 900,
                    "browser_download_url": "https://github.com/x/recof-manifest.json"
                }
            ]
        }"#;

        let release: Release = serde_json::from_str(json).unwrap();
        assert_eq!(release.version(), Some(Version::parse("0.3.0").unwrap()));
        assert_eq!(release.body.as_deref(), Some("Fixed the flashlight."));
        assert_eq!(release.assets.len(), 2);

        let zip = release.asset("recof-0.3.0.zip").unwrap();
        assert_eq!(zip.size, 4_200_000);
        assert_eq!(zip.digest.as_deref(), Some("sha256:aaaa"));

        // An asset with no digest is normal for an older release.
        assert_eq!(release.asset("recof-manifest.json").unwrap().digest, None);
        assert!(release.asset_ending_with(".zip").is_some());
        assert!(release.asset_ending_with(".sig").is_none());
    }

    #[test]
    fn a_tag_that_is_not_a_version_is_handled() {
        let mut release = release("not-a-version", false);
        assert_eq!(release.version(), None);
        release.tag_name = "v1.2.3".to_owned();
        assert_eq!(release.version(), Some(Version::parse("1.2.3").unwrap()));
    }

    #[test]
    fn channels_filter_the_way_they_should() {
        assert!(release("v1.0.0", false).is_in(Channel::Stable));
        assert!(release("v1.0.0", false).is_in(Channel::Beta));
        assert!(!release("v1.1.0-beta.1", true).is_in(Channel::Stable));
        assert!(release("v1.1.0-beta.1", true).is_in(Channel::Beta));

        let mut draft = release("v2.0.0", false);
        draft.draft = true;
        assert!(!draft.is_in(Channel::Stable));
        assert!(!draft.is_in(Channel::Beta));
    }

    #[test]
    fn a_recent_cached_reply_is_used_without_asking() {
        let dir = scratch("cache-hit");
        let client = GithubClient::new(&dir)
            // A base nothing listens on, so touching the network would be
            // an error rather than a slow test.
            .with_base("http://127.0.0.1:1");

        let body = serde_json::to_string(&release("v0.3.0", false)).unwrap();
        write_cache(
            &dir.join(cache_name("caixax/ReCoF", "latest")),
            &CachedReply {
                etag: Some("\"abc\"".to_owned()),
                fetched_at: now_seconds(),
                body,
            },
        );

        let found = client.latest("caixax/ReCoF").expect("the cache answers");
        assert_eq!(found.tag_name, "v0.3.0");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_stale_cache_is_still_used_when_the_network_is_down() {
        let dir = scratch("cache-stale-offline");
        let client = GithubClient::new(&dir).with_base("http://127.0.0.1:1");

        let body = serde_json::to_string(&release("v0.2.0", false)).unwrap();
        write_cache(
            &dir.join(cache_name("caixax/ReCoF", "latest")),
            &CachedReply {
                etag: None,
                // Older than the window, so a request is attempted and
                // fails against a port nothing is listening on.
                fetched_at: now_seconds() - CACHE_TTL.as_secs() - 1,
                body,
            },
        );

        let found = client
            .latest("caixax/ReCoF")
            .expect("a failed request falls back to the cache");
        assert_eq!(found.tag_name, "v0.2.0");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn with_no_cache_and_no_network_the_error_says_offline() {
        let dir = scratch("offline");
        let client = GithubClient::new(&dir).offline();
        assert_eq!(client.latest("caixax/ReCoF"), Err(NetError::Offline));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_cache_file_name_is_safe_on_any_filesystem() {
        let name = cache_name("caixax/ReCoF-Launcher", "latest");
        assert_eq!(name, "github-caixax-ReCoF-Launcher-latest.json");
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
        assert!(!name.contains(':'));
    }

    #[test]
    fn a_corrupt_cache_file_is_ignored_rather_than_fatal() {
        let dir = scratch("corrupt-cache");
        let path = dir.join(cache_name("caixax/ReCoF", "latest"));
        std::fs::write(&path, b"not json").unwrap();

        assert!(read_cache(&path).is_none());
        let client = GithubClient::new(&dir).offline();
        assert_eq!(client.latest("caixax/ReCoF"), Err(NetError::Offline));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_user_agent_names_the_project_as_github_asks() {
        let agent = user_agent();
        assert!(agent.starts_with("ReCoF-Launcher/"));
        assert!(agent.contains("caixax/ReCoF-Launcher"));
    }

    #[test]
    fn the_newest_beta_wins_even_when_published_out_of_order() {
        // Sorting is by version, so a hotfix published after a newer beta
        // does not become "newest".
        let mut releases = vec![
            release("v1.0.0", false),
            release("v1.1.0-beta.2", true),
            release("v1.1.0-beta.1", true),
        ];
        releases.retain(|release| release.is_in(Channel::Beta));
        releases.sort_by_key(|release| release.version());
        assert_eq!(releases.pop().unwrap().tag_name, "v1.1.0-beta.2");
    }
}
