//! User-related endpoints: `GET /user/{identifier}` and
//! `GET /user/match-history/{address}`.

use serde::Deserialize;

use crate::Result;
use crate::http::HttpClient;
use crate::models::user::{MatchHistoryPage, User};

/// Accessor for user-related endpoints. Obtained from
/// [`PixieChessClient::users`](crate::PixieChessClient::users).
pub struct UsersResource<'c> {
    http: &'c HttpClient,
}

impl<'c> UsersResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// Look up a user by username or wallet address.
    ///
    /// ```ignore
    /// let user = client.users().get("iron-pawn").send().await?;
    /// let raw  = client.users().get("0xabc...").raw().await?;
    /// ```
    pub fn get(&self, identifier: impl Into<String>) -> UsersGetBuilder<'c> {
        UsersGetBuilder {
            http: self.http,
            identifier: identifier.into(),
        }
    }

    /// Fetch a page of a user's match history.
    ///
    /// ```ignore
    /// let page = client.users()
    ///     .match_history("0xabc...")
    ///     .page(2)
    ///     .limit(25)
    ///     .send()
    ///     .await?;
    /// ```
    pub fn match_history(&self, address: impl Into<String>) -> MatchHistoryBuilder<'c> {
        MatchHistoryBuilder {
            http: self.http,
            address: address.into(),
            page: 1,
            limit: 15,
        }
    }
}

// `GET /user/{identifier}` ---------------------------------------------

/// Builder for [`UsersResource::get`]. Terminal methods are
/// `.send().await` (typed) and `.raw().await` (raw JSON).
pub struct UsersGetBuilder<'c> {
    http: &'c HttpClient,
    identifier: String,
}

impl UsersGetBuilder<'_> {
    fn path(&self) -> String {
        format!("/user/{}", self.identifier)
    }

    /// Fetch the user and return the typed [`User`] model.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or if the response
    /// can't be deserialized into [`User`].
    pub async fn send(self) -> Result<User> {
        // The server wraps the response in {"user": ...}; unwrap here.
        #[derive(Deserialize)]
        struct Envelope {
            user: User,
        }
        let path = self.path();
        let env: Envelope = self.http.get(&path).await?;
        Ok(env.user)
    }

    /// Fetch the user and return the raw JSON response (including the
    /// `{"user": ...}` envelope).
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

// `GET /user/match-history/{address}` -----------------------------------

/// Builder for [`UsersResource::match_history`]. Supports `.page()` and
/// `.limit()`; terminal methods are `.send().await` and `.raw().await`.
pub struct MatchHistoryBuilder<'c> {
    http: &'c HttpClient,
    address: String,
    page: u32,
    limit: u32,
}

impl MatchHistoryBuilder<'_> {
    /// Set the 1-indexed page number. Default: `1`.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Set the page size. Default: `15` (matches the live API).
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    fn path(&self) -> String {
        format!("/user/match-history/{}", self.address)
    }

    fn params(&self) -> [(&'static str, String); 2] {
        [
            ("page", self.page.to_string()),
            ("limit", self.limit.to_string()),
        ]
    }

    /// Fetch the match-history page and return the typed
    /// [`MatchHistoryPage`].
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or if the response
    /// can't be deserialized.
    pub async fn send(self) -> Result<MatchHistoryPage> {
        let path = self.path();
        let params = self.params();
        self.http.get_with_params(&path, &params).await
    }

    /// Fetch the match-history page and return the raw JSON response.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let path = self.path();
        let params = self.params();
        self.http.get_with_params(&path, &params).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    fn user_payload() -> serde_json::Value {
        json!({
            "user": {
                "_id": "507f1f77bcf86cd799439011",
                "address": "0xabc",
                "username": "alice",
                "rating": 1500.0,
            }
        })
    }

    #[tokio::test]
    async fn get_send_unwraps_user_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/alice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(user_payload()))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let user = client.users().get("alice").send().await.unwrap();
        assert_eq!(user.username.as_deref(), Some("alice"));
        assert_eq!(user.address, "0xabc");
    }

    #[tokio::test]
    async fn get_raw_keeps_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/alice"))
            .respond_with(ResponseTemplate::new(200).set_body_json(user_payload()))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let raw = client.users().get("alice").raw().await.unwrap();
        assert_eq!(raw["user"]["username"], "alice");
    }

    #[tokio::test]
    async fn match_history_attaches_page_and_limit_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/match-history/0xabc"))
            .and(query_param("page", "2"))
            .and(query_param("limit", "25"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "matches": [],
                "totalPages": 0,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let page = client
            .users()
            .match_history("0xabc")
            .page(2)
            .limit(25)
            .send()
            .await
            .unwrap();
        assert!(page.matches.is_empty());
        assert_eq!(page.total_pages, Some(0));
    }

    #[tokio::test]
    async fn match_history_defaults_page_1_limit_15() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/match-history/0xabc"))
            .and(query_param("page", "1"))
            .and(query_param("limit", "15"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"matches": []})))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let page = client.users().match_history("0xabc").send().await.unwrap();
        assert!(page.matches.is_empty());
    }

    #[tokio::test]
    async fn match_history_raw_returns_value() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/match-history/0xabc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "matches": [],
                "currentPage": 1,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let raw = client.users().match_history("0xabc").raw().await.unwrap();
        assert_eq!(raw["currentPage"], 1);
    }
}
