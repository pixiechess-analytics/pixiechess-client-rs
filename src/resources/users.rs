//! User-related endpoints: `GET /user/{identifier}` and
//! `GET /user/match-history/{address}`.

use std::pin::Pin;

use futures::Stream;
use serde::Deserialize;

use crate::Result;
use crate::http::HttpClient;
use crate::models::user::{MatchHistoryEntry, MatchHistoryPage, User};
use crate::pagination::page_stream;

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

    /// Iterate every match in a user's history, fetching pages on demand.
    ///
    /// ```ignore
    /// use futures::StreamExt;
    ///
    /// let mut stream = client.users().match_history_iter("0xabc...").send();
    /// while let Some(entry) = stream.next().await {
    ///     let entry = entry?;
    ///     // …
    /// }
    /// ```
    pub fn match_history_iter(&self, address: impl Into<String>) -> MatchHistoryIterBuilder<'c> {
        MatchHistoryIterBuilder {
            http: self.http,
            address: address.into(),
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

// `GET /user/match-history/{address}` (auto-paginating) ---------------

/// Builder for [`UsersResource::match_history_iter`]. The terminal
/// method `.send()` returns a [`Stream`] that yields every match in the
/// user's history, fetching pages on demand. Stops when an empty page
/// comes back (the live API doesn't report `totalPages` on this
/// endpoint).
pub struct MatchHistoryIterBuilder<'c> {
    http: &'c HttpClient,
    address: String,
    limit: u32,
}

impl MatchHistoryIterBuilder<'_> {
    /// Set the per-page batch size used internally while iterating.
    /// Default: `15` (matches the live API).
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    /// Begin streaming entries. The returned [`Stream`] short-circuits
    /// on the first empty page.
    #[must_use]
    pub fn send(self) -> Pin<Box<dyn Stream<Item = Result<MatchHistoryEntry>> + Send>> {
        let http = self.http.clone();
        let address = self.address;
        let limit = self.limit;
        page_stream(
            move |page| {
                let http = http.clone();
                let path = format!("/user/match-history/{address}");
                let params = [("page", page.to_string()), ("limit", limit.to_string())];
                async move { http.get_with_params(&path, &params).await }
            },
            |data| {
                let matches = data
                    .get("matches")
                    .cloned()
                    .unwrap_or(serde_json::Value::Array(vec![]));
                let entries: Vec<MatchHistoryEntry> = serde_json::from_value(matches)?;
                Ok(entries)
            },
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
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
                "usernameDisplay": "Alice",
                "walletClientType": "metamask",
                "helmet": {"key": "knightmare", "color": "red"},
                "lastLogin": "2026-05-13T12:34:56Z",
                "winRate": 50,
                "matchCount": 10,
                "wins": 5,
                "losses": 4,
                "draws": 1,
                "casualGames": 2,
                "streak": 0,
                "colorRecord": {},
                "trophies": 0,
                "rating": 1500.0,
                "rd": 50.0,
                "isProvisional": false,
                "peakRating": 1600.0,
                "ratedGamesPlayed": 8,
                "genuineGamesPlayed": 8,
                "points": 100,
            }
        })
    }

    fn player_info_json(addr: &str, name: &str) -> serde_json::Value {
        json!({
            "address": addr,
            "username": name,
            "usernameDisplay": name,
            "helmet": {"key": "knightmare", "color": "red"},
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
                "currentPage": 2,
                "totalCount": 0,
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
        assert_eq!(page.total_pages, 0);
    }

    #[tokio::test]
    async fn match_history_defaults_page_1_limit_15() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/user/match-history/0xabc"))
            .and(query_param("page", "1"))
            .and(query_param("limit", "15"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "matches": [],
                "totalPages": 0,
                "currentPage": 1,
                "totalCount": 0,
            })))
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
    async fn match_history_iter_walks_pages_and_stops_on_empty() {
        let server = MockServer::start().await;
        // Page 1 returns two entries; page 2 returns an empty matches array.
        // The stream stops on the empty page without further requests.
        let entry = |game: &str| {
            json!({
                "gameId": game,
                "createdAt": "2026-05-13T12:00:00Z",
                "white": player_info_json("0xaaa", "alice"),
                "black": player_info_json("0xbbb", "bob"),
                "winner": "white",
                "resultForUser": "win",
                "outcome": "checkmate",
                "rated": true,
                "timing": {"whiteElapsedMs": 1, "blackElapsedMs": 2, "clockMs": 3},
            })
        };
        Mock::given(method("GET"))
            .and(path("/user/match-history/0xabc"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "matches": [entry("g1"), entry("g2")],
                "totalPages": 2,
                "currentPage": 1,
                "totalCount": 4,
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/user/match-history/0xabc"))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "matches": [],
                "totalPages": 2,
                "currentPage": 2,
                "totalCount": 4,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let stream = client.users().match_history_iter("0xabc").send();
        let collected: Vec<_> = stream.collect().await;
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].as_ref().unwrap().game_id, "g1");
        assert_eq!(collected[1].as_ref().unwrap().game_id, "g2");
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
