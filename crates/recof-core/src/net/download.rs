//! Downloading a file.
//!
//! Everything is written to a temporary name under the cache folder and
//! only renamed once the whole file has arrived, so a half downloaded
//! release is never mistaken for a complete one. Nothing downloaded is
//! used before the manifest signature and the hashes have been checked.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::net::NetError;

/// Refuse anything larger than this. A ReCoF release is a handful of
/// rebuilt binaries; the launcher itself is a few megabytes.
pub const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

/// Read in chunks of this size, which is also how often progress is
/// reported.
const CHUNK: usize = 64 * 1024;

/// How long to wait for the whole download.
const TIMEOUT: Duration = Duration::from_secs(600);

/// How far a download has got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub done: u64,
    /// What the server said, or what the manifest expects.
    pub total: u64,
}

impl DownloadProgress {
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.done as f64 / self.total as f64).clamp(0.0, 1.0) as f32
    }
}

/// How a download ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadOutcome {
    Downloaded { path: PathBuf, bytes: u64 },
    Cancelled,
}

/// Downloads a URL into `directory` under `file_name`.
///
/// `expected_size` comes from the manifest or the release asset and bounds
/// what is read; a server claiming more is refused rather than filling the
/// disk. Pass zero when the size is genuinely unknown, and the hard cap
/// applies instead.
pub fn download(
    url: &str,
    directory: &Path,
    file_name: &str,
    expected_size: u64,
    cancel: &Arc<AtomicBool>,
    progress: &mut dyn FnMut(DownloadProgress),
) -> Result<DownloadOutcome, NetError> {
    let limit = if expected_size == 0 {
        MAX_DOWNLOAD_BYTES
    } else {
        expected_size
    };
    if limit > MAX_DOWNLOAD_BYTES {
        return Err(NetError::TooLarge {
            size: limit,
            limit: MAX_DOWNLOAD_BYTES,
        });
    }

    std::fs::create_dir_all(directory)
        .map_err(|error| NetError::Io(format!("{}: {error}", directory.display())))?;

    let final_path = directory.join(file_name);
    let temporary = directory.join(format!("{file_name}.part"));
    let _ = std::fs::remove_file(&temporary);

    let config = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .user_agent(format!(
            "ReCoF-Launcher/{}",
            crate::consts::LAUNCHER_VERSION
        ))
        .max_redirects(5)
        .build();
    let agent: ureq::Agent = config.into();

    let response = match agent.get(url).call() {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(code)) => {
            return Err(NetError::Http {
                code,
                cached_body: None,
            })
        }
        Err(error) => {
            log::warn!("{url}: {error}");
            return Err(NetError::Offline);
        }
    };

    let reported = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if reported > limit {
        return Err(NetError::TooLarge {
            size: reported,
            limit,
        });
    }

    let total = if reported > 0 { reported } else { limit };
    let mut written = 0u64;
    let mut response = response;
    // One byte past the limit, so a body that is too long is noticed
    // rather than silently truncated into something that then fails its
    // hash check for no obvious reason.
    let mut reader = response.body_mut().as_reader().take(limit + 1);

    {
        let mut file = std::fs::File::create(&temporary)
            .map_err(|error| NetError::Io(format!("{}: {error}", temporary.display())))?;
        let mut buffer = vec![0u8; CHUNK];

        loop {
            if cancel.load(Ordering::Relaxed) {
                drop(file);
                let _ = std::fs::remove_file(&temporary);
                return Ok(DownloadOutcome::Cancelled);
            }

            let read = reader
                .read(&mut buffer)
                .map_err(|error| NetError::Io(error.to_string()))?;
            if read == 0 {
                break;
            }

            written += read as u64;
            if written > limit {
                drop(file);
                let _ = std::fs::remove_file(&temporary);
                return Err(NetError::TooLarge {
                    size: written,
                    limit,
                });
            }

            file.write_all(&buffer[..read])
                .map_err(|error| NetError::Io(format!("{}: {error}", temporary.display())))?;
            progress(DownloadProgress {
                done: written,
                total,
            });
        }

        file.flush()
            .map_err(|error| NetError::Io(error.to_string()))?;
        // The bytes have to be on the disk before the rename, or a power
        // cut could leave a complete looking file with nothing in it.
        file.sync_all()
            .map_err(|error| NetError::Io(error.to_string()))?;
    }

    if expected_size != 0 && written != expected_size {
        let _ = std::fs::remove_file(&temporary);
        return Err(NetError::WrongSize {
            expected: expected_size,
            found: written,
        });
    }

    std::fs::rename(&temporary, &final_path)
        .map_err(|error| NetError::Io(format!("{}: {error}", final_path.display())))?;

    Ok(DownloadOutcome::Downloaded {
        path: final_path,
        bytes: written,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join("recof-download-tests").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn no_cancel() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    #[test]
    fn a_size_beyond_the_hard_cap_is_refused_before_anything_is_opened() {
        let dir = scratch("too-large");
        let outcome = download(
            "http://127.0.0.1:1/x.zip",
            &dir,
            "x.zip",
            MAX_DOWNLOAD_BYTES + 1,
            &no_cancel(),
            &mut |_| {},
        );
        assert!(matches!(outcome, Err(NetError::TooLarge { .. })));
        assert!(!dir.join("x.zip").exists());
        assert!(!dir.join("x.zip.part").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_server_that_is_not_there_reports_offline_and_leaves_nothing() {
        let dir = scratch("offline");
        let outcome = download(
            "http://127.0.0.1:1/x.zip",
            &dir,
            "x.zip",
            1024,
            &no_cancel(),
            &mut |_| {},
        );
        assert_eq!(outcome, Err(NetError::Offline));
        assert!(!dir.join("x.zip").exists());
        assert!(!dir.join("x.zip.part").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_cancel_before_the_first_byte_still_leaves_nothing_behind() {
        let dir = scratch("cancelled");
        let cancel = Arc::new(AtomicBool::new(true));
        // Nothing is listening, so this fails before the loop; what the
        // test pins is that neither path is left on disk either way.
        let _ = download(
            "http://127.0.0.1:1/x.zip",
            &dir,
            "x.zip",
            1024,
            &cancel,
            &mut |_| {},
        );
        assert!(!dir.join("x.zip").exists());
        assert!(!dir.join("x.zip.part").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_progress_fraction_is_well_behaved() {
        assert_eq!(
            DownloadProgress { done: 0, total: 0 }.fraction(),
            0.0,
            "an unknown size reads as not started rather than finished"
        );
        assert_eq!(
            DownloadProgress {
                done: 50,
                total: 100
            }
            .fraction(),
            0.5
        );
        assert_eq!(
            DownloadProgress {
                done: 200,
                total: 100
            }
            .fraction(),
            1.0
        );
    }
}
