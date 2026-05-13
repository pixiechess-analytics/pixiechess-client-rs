//! Entry point for the `PixieChess` client.

use crate::Result;
use crate::http::{DEFAULT_BASE_URL, HttpClient};
use crate::resources::users::UsersResource;

/// Async client for `api.pixiechess.xyz`.
///
/// Holds a `reqwest`-backed HTTP layer with the API's required default
/// headers (`Origin`, `Referer`, `User-Agent`, `sec-fetch-*`) baked in.
/// Resource accessors will be exposed on this type as later branches land
/// each endpoint group.
#[derive(Debug, Clone)]
pub struct PixieChessClient {
    http: HttpClient,
}

impl PixieChessClient {
    /// Build a client pointing at the default base URL
    /// (`https://api.pixiechess.xyz`).
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] if the underlying HTTP client cannot be
    /// constructed (e.g. TLS initialization failure).
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: HttpClient::new(DEFAULT_BASE_URL)?,
        })
    }

    /// Start a [`PixieChessClientBuilder`] for callers that want to
    /// override the base URL (testing against a mock server, etc).
    #[must_use]
    pub fn builder() -> PixieChessClientBuilder {
        PixieChessClientBuilder::default()
    }

    /// User-related endpoints (`GET /user/{identifier}`,
    /// `GET /user/match-history/{address}`).
    #[must_use]
    pub fn users(&self) -> UsersResource<'_> {
        UsersResource::new(&self.http)
    }
}

/// Builder for [`PixieChessClient`]. Configures the base URL (and, in
/// future, custom timeouts / retry policies).
#[derive(Debug, Default)]
pub struct PixieChessClientBuilder {
    base_url: Option<String>,
}

impl PixieChessClientBuilder {
    /// Override the base URL the client will hit. Useful for tests
    /// pointing at a `wiremock` server.
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Construct the client.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] if the base URL is invalid or the
    /// underlying HTTP client cannot be constructed.
    pub fn build(self) -> Result<PixieChessClient> {
        let url = self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL);
        Ok(PixieChessClient {
            http: HttpClient::new(url)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructs_a_client() {
        let _ = PixieChessClient::new().expect("default new() should succeed");
    }

    #[test]
    fn builder_default_constructs_a_client() {
        let _ = PixieChessClient::builder()
            .build()
            .expect("builder.build() should succeed");
    }

    #[test]
    fn builder_uses_overridden_base_url() {
        let _ = PixieChessClient::builder()
            .base_url("http://localhost:1234")
            .build()
            .expect("override base URL should succeed");
    }

    #[test]
    fn builder_rejects_invalid_base_url() {
        let res = PixieChessClient::builder().base_url("not a url").build();
        assert!(res.is_err());
    }
}
