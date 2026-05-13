//! Error type and `Result` alias used throughout the crate.

use thiserror::Error;

/// Errors returned by every public operation in this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// The server responded `404 Not Found`. The contained string is the
    /// response body (often a short JSON error or empty).
    #[error("not found: {0}")]
    NotFound(String),

    /// The server responded with a `4xx` (other than `404`) or `5xx`.
    #[error("HTTP {status}: {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Response body, captured for diagnostics.
        message: String,
    },

    /// Underlying transport or HTTP-layer failure.
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// JSON deserialization failed for a typed call.
    #[error(transparent)]
    Decode(#[from] serde_json::Error),
}

/// Convenience alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = Error::NotFound("no such user".into());
        let s = err.to_string();
        assert!(s.contains("not found"));
        assert!(s.contains("no such user"));
    }

    #[test]
    fn api_display() {
        let err = Error::Api {
            status: 500,
            message: "boom".into(),
        };
        assert_eq!(err.to_string(), "HTTP 500: boom");
    }
}
