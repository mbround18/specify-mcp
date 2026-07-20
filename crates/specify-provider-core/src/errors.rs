use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum ProviderError {
    /// Provider rejected credentials, or the session/token is invalid or missing.
    Auth(String),
    /// Authenticated but the acting identity lacks rights for the requested operation.
    Permission(String),
    /// Network/HTTP/protocol-level failure while reaching the provider.
    Transport(String),
    /// Provider rejected a request due to malformed or invalid input data.
    Validation(String),
    /// Provider reports a state conflict (e.g. concurrent edit, duplicate resource).
    Conflict(String),
    /// Requested capability is not implemented by this provider, or an adapter mode
    /// could not be resolved to a concrete capability set.
    Unsupported(String),
    /// Provider's behavior violates the shared provider contract (conformance failure).
    Conformance(String),
    /// Unexpected internal failure not attributable to a specific remote provider call
    /// (e.g. local filesystem I/O for a local/mock provider, or orchestration-internal
    /// logic errors). Not a substitute for the more specific variants above.
    Internal(String),
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Auth(v) => write!(f, "auth error: {v}"),
            ProviderError::Permission(v) => write!(f, "permission error: {v}"),
            ProviderError::Transport(v) => write!(f, "transport error: {v}"),
            ProviderError::Validation(v) => write!(f, "validation error: {v}"),
            ProviderError::Conflict(v) => write!(f, "conflict error: {v}"),
            ProviderError::Unsupported(v) => write!(f, "unsupported error: {v}"),
            ProviderError::Conformance(v) => write!(f, "conformance error: {v}"),
            ProviderError::Internal(v) => write!(f, "internal error: {v}"),
        }
    }
}

impl std::error::Error for ProviderError {}
