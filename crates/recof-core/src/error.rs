//! One error type for the whole launcher.
//!
//! Every variant carries a stable Fluent message id so the UI can show a
//! translated, understandable sentence, plus a `Display` form with the
//! technical detail that goes to the log. The two are never the same text:
//! the user gets the message id, the log gets `Display`.

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

/// A name and value pair handed to Fluent when formatting a message.
pub type MessageArgs = Vec<(&'static str, String)>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("io error: {0}")]
    PlainIo(#[from] std::io::Error),

    #[error("this operation is not available on this platform: {0}")]
    NotSupported(&'static str),

    #[error("{0}")]
    Platform(#[from] recof_platform::PlatformError),
}

impl Error {
    /// Wraps an io error with the path that caused it.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    /// The Fluent message id the UI shows for this error.
    pub fn message_id(&self) -> &'static str {
        match self {
            Error::Io { .. } | Error::PlainIo(_) => "err-io",
            Error::NotSupported(_) => "err-not-supported",
            Error::Platform(inner) => inner.message_id(),
        }
    }

    /// Values the translated message may interpolate.
    pub fn message_args(&self) -> MessageArgs {
        match self {
            Error::Io { path, .. } => vec![("path", path.display().to_string())],
            Error::PlainIo(_) => Vec::new(),
            Error::NotSupported(_) => Vec::new(),
            Error::Platform(inner) => inner.message_args(),
        }
    }
}

/// Adds the path to an io error without a closure at every call site.
pub trait IoResultExt<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> IoResultExt<T> for std::io::Result<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| Error::io(path, source))
    }
}
