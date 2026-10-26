//! Talking to GitHub, and nothing else.
//!
//! The launcher makes network requests to exactly one host, to read the
//! release list and to fetch what a release points at. It sends nothing:
//! no telemetry, no crash reports, no identifiers. The bug report opens a
//! browser and lets the user press the button.

pub mod download;
pub mod github;

pub use download::{download, DownloadOutcome, DownloadProgress};
pub use github::{Asset, GithubClient, Release};

/// Everything that can go wrong on the way to GitHub.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NetError {
    #[error("GitHub could not be reached")]
    Offline,

    #[error("there is no such release")]
    NotFound,

    #[error("GitHub is rate limiting this connection")]
    RateLimited,

    #[error("GitHub answered with status {code}")]
    Http {
        code: u16,
        /// A cached body that can still be used, when there is one.
        cached_body: Option<String>,
    },

    #[error("GitHub's answer could not be read: {0}")]
    BadReply(String),

    #[error("the file is {size} bytes, more than the {limit} allowed")]
    TooLarge { size: u64, limit: u64 },

    #[error("the download is {found} bytes, the release says {expected}")]
    WrongSize { expected: u64, found: u64 },

    #[error("io: {0}")]
    Io(String),
}

impl NetError {
    /// Fluent message id for the sentence the user sees.
    pub fn message_id(&self) -> &'static str {
        match self {
            NetError::Offline => "err-network",
            NetError::NotFound => "err-no-release",
            NetError::RateLimited => "err-rate-limited",
            NetError::Http { .. } | NetError::BadReply(_) => "err-github",
            NetError::TooLarge { .. } | NetError::WrongSize { .. } => "err-hash-mismatch",
            NetError::Io(_) => "err-io",
        }
    }

    /// Values the translated message interpolates.
    pub fn message_args(&self) -> Vec<(&'static str, String)> {
        match self {
            NetError::Http { code, .. } => vec![("detail", code.to_string())],
            NetError::BadReply(detail) => vec![("detail", detail.clone())],
            NetError::Io(detail) => vec![("path", detail.clone())],
            _ => Vec::new(),
        }
    }

    /// Whether this is a reason to stay quiet rather than to complain.
    ///
    /// A launcher that cannot reach GitHub still installs, plays, repairs
    /// and rolls back, so being offline is a status line and not an error
    /// dialog.
    pub fn is_only_offline(&self) -> bool {
        matches!(self, NetError::Offline | NetError::RateLimited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_has_a_message_and_reads_sensibly() {
        let errors = [
            NetError::Offline,
            NetError::NotFound,
            NetError::RateLimited,
            NetError::Http {
                code: 500,
                cached_body: None,
            },
            NetError::BadReply("x".to_owned()),
            NetError::TooLarge { size: 10, limit: 5 },
            NetError::WrongSize {
                expected: 10,
                found: 5,
            },
            NetError::Io("x".to_owned()),
        ];
        for error in errors {
            assert!(error.message_id().starts_with("err-"), "{error}");
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn being_offline_is_not_treated_as_a_failure() {
        assert!(NetError::Offline.is_only_offline());
        assert!(NetError::RateLimited.is_only_offline());
        assert!(!NetError::NotFound.is_only_offline());
        assert!(!NetError::BadReply("x".to_owned()).is_only_offline());
    }
}
