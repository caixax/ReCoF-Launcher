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
}

impl PlatformError {
    pub fn message_id(&self) -> &'static str {
        match self {
            PlatformError::Unsupported(_) => "err-not-supported",
            PlatformError::Registry { .. } => "err-registry",
            PlatformError::Api { .. } => "err-platform-api",
            PlatformError::Io(_) => "err-io",
        }
    }

    pub fn message_args(&self) -> Vec<(&'static str, String)> {
        match self {
            PlatformError::Registry { key, .. } => vec![("key", key.clone())],
            PlatformError::Api { api, .. } => vec![("api", (*api).to_string())],
            _ => Vec::new(),
        }
    }
}
