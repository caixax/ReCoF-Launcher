//! Errors the platform layer can produce, each with a Fluent message id.

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("{0} is not implemented on this platform")]
    Unsupported(&'static str),

    #[error("registry: {operation} failed on {key}: {detail}")]
    Registry {
        operation: &'static str,
        key: String,
        detail: String,
    },

    #[error("{api} failed: {detail}")]
    Api { api: &'static str, detail: String },

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// The user said no to the administrator prompt.
    #[error("the administrator prompt was cancelled")]
    Declined,

    /// A path too broad to hand to the operation, such as a drive root.
    #[error("{0} is too broad to be excluded")]
    NotAllowed(String),
}

impl PlatformError {
    pub fn message_id(&self) -> &'static str {
        match self {
            PlatformError::Unsupported(_) => "err-not-supported",
            PlatformError::Registry { .. } => "err-registry",
            PlatformError::Api { .. } => "err-platform-api",
            PlatformError::Io(_) => "err-io",
            PlatformError::Declined => "err-declined",
            PlatformError::NotAllowed(_) => "err-path-too-broad",
        }
    }

    pub fn message_args(&self) -> Vec<(&'static str, String)> {
        match self {
            PlatformError::Registry { key, .. } => vec![("key", key.clone())],
            PlatformError::Api { api, .. } => vec![("api", (*api).to_string())],
            PlatformError::NotAllowed(path) => vec![("path", path.clone())],
            _ => Vec::new(),
        }
    }
}
