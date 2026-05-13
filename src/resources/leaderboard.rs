//! Leaderboard endpoints: `GET /leaderboard` (rating-based) and
//! `GET /points-leaderboard` (points-based). Each has a single-page
//! getter and an auto-paginating iter.

use std::pin::Pin;

use futures::Stream;

use crate::Result;
use crate::http::HttpClient;
use crate::models::leaderboard::{
    LeaderboardEntry, LeaderboardPage, PointsLeaderboardEntry, PointsLeaderboardPage,
};
use crate::pagination::page_stream;

/// Accessor for leaderboard endpoints. Obtained from
/// [`PixieChessClient::leaderboard`](crate::PixieChessClient::leaderboard).
pub struct LeaderboardResource<'c> {
    http: &'c HttpClient,
}

impl<'c> LeaderboardResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// Fetch one page of the rating leaderboard.
    #[must_use]
    pub fn get(&self) -> LeaderboardGetBuilder<'c> {
        LeaderboardGetBuilder {
            http: self.http,
            page: 1,
            page_size: None,
        }
    }

    /// Iterate every entry across all pages of the rating leaderboard.
    // Method is named `iter` to mirror the Python client; the returned
    // builder is not a real `Iterator` (it produces an async `Stream`).
    #[must_use]
    #[allow(clippy::should_implement_trait, clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> LeaderboardIterBuilder<'c> {
        LeaderboardIterBuilder {
            http: self.http,
            page_size: None,
        }
    }

    /// Fetch one page of the points leaderboard.
    #[must_use]
    pub fn points(&self) -> PointsGetBuilder<'c> {
        PointsGetBuilder {
            http: self.http,
            page: 1,
        }
    }

    /// Iterate every entry across all pages of the points leaderboard.
    #[must_use]
    pub fn points_iter(&self) -> PointsIterBuilder<'c> {
        PointsIterBuilder { http: self.http }
    }
}

fn lb_params(page: u32, page_size: Option<u32>) -> Vec<(&'static str, String)> {
    let mut params = vec![("page", page.to_string())];
    if let Some(ps) = page_size {
        params.push(("pageSize", ps.to_string()));
    }
    params
}

// `GET /leaderboard` ---------------------------------------------------

/// Builder for [`LeaderboardResource::get`].
pub struct LeaderboardGetBuilder<'c> {
    http: &'c HttpClient,
    page: u32,
    page_size: Option<u32>,
}

impl LeaderboardGetBuilder<'_> {
    /// Set the 1-indexed page number. Default: `1`.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Set the page size. Omitted from the query when not set, letting
    /// the server use its own default.
    #[must_use]
    pub fn page_size(mut self, n: u32) -> Self {
        self.page_size = Some(n);
        self
    }

    /// Fetch the page.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or deserialization
    /// failure.
    pub async fn send(self) -> Result<LeaderboardPage> {
        let params = lb_params(self.page, self.page_size);
        self.http.get_with_params("/leaderboard", &params).await
    }

    /// Fetch the page as raw JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let params = lb_params(self.page, self.page_size);
        self.http.get_with_params("/leaderboard", &params).await
    }
}

/// Builder for [`LeaderboardResource::iter`]. `.send()` returns a
/// [`Stream`] over every entry across all pages.
pub struct LeaderboardIterBuilder<'c> {
    http: &'c HttpClient,
    page_size: Option<u32>,
}

impl LeaderboardIterBuilder<'_> {
    /// Set the per-page batch size used internally while iterating.
    #[must_use]
    pub fn page_size(mut self, n: u32) -> Self {
        self.page_size = Some(n);
        self
    }

    /// Begin streaming entries. Short-circuits on `totalPages`.
    #[must_use]
    pub fn send(self) -> Pin<Box<dyn Stream<Item = Result<LeaderboardEntry>> + Send>> {
        let http = self.http.clone();
        let page_size = self.page_size;
        page_stream(
            move |page| {
                let http = http.clone();
                let params = lb_params(page, page_size);
                async move { http.get_with_params("/leaderboard", &params).await }
            },
            |data| {
                let entries = data
                    .get("entries")
                    .cloned()
                    .unwrap_or(serde_json::Value::Array(vec![]));
                let entries: Vec<LeaderboardEntry> = serde_json::from_value(entries)?;
                Ok(entries)
            },
            Some("totalPages"),
        )
    }
}

// `GET /points-leaderboard` --------------------------------------------

/// Builder for [`LeaderboardResource::points`].
pub struct PointsGetBuilder<'c> {
    http: &'c HttpClient,
    page: u32,
}

impl PointsGetBuilder<'_> {
    /// Set the 1-indexed page number. Default: `1`.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Fetch the page.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or deserialization
    /// failure.
    pub async fn send(self) -> Result<PointsLeaderboardPage> {
        let params = [("page", self.page.to_string())];
        self.http
            .get_with_params("/points-leaderboard", &params)
            .await
    }

    /// Fetch the page as raw JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let params = [("page", self.page.to_string())];
        self.http
            .get_with_params("/points-leaderboard", &params)
            .await
    }
}

/// Builder for [`LeaderboardResource::points_iter`]. `.send()` returns
/// a [`Stream`] over every points-leaderboard entry across all pages.
pub struct PointsIterBuilder<'c> {
    http: &'c HttpClient,
}

impl PointsIterBuilder<'_> {
    /// Begin streaming entries. Short-circuits on `totalPages`.
    #[must_use]
    pub fn send(self) -> Pin<Box<dyn Stream<Item = Result<PointsLeaderboardEntry>> + Send>> {
        let http = self.http.clone();
        page_stream(
            move |page| {
                let http = http.clone();
                let params = [("page", page.to_string())];
                async move { http.get_with_params("/points-leaderboard", &params).await }
            },
            |data| {
                let entries = data
                    .get("entries")
                    .cloned()
                    .unwrap_or(serde_json::Value::Array(vec![]));
                let entries: Vec<PointsLeaderboardEntry> = serde_json::from_value(entries)?;
                Ok(entries)
            },
            Some("totalPages"),
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

    fn entry(rank: u32) -> serde_json::Value {
        json!({
            "rank": rank,
            "address": format!("0x{rank:040x}"),
            "username": format!("u{rank}"),
            "usernameDisplay": format!("U{rank}"),
            "helmet": {"key": "knightmare", "color": "red"},
            "rating": 1500.0,
            "isProvisional": false,
            "gamesPlayed": 10,
            "genuineGamesPlayed": 8,
            "wins": 5,
            "streak": 0,
            "isOnline": false,
            "isInGame": false,
        })
    }

    fn stats_payload() -> serde_json::Value {
        json!({"totalRankedPlayers": 100, "gamesToday": 10, "activeNow": 5})
    }

    fn points_current_user_payload() -> serde_json::Value {
        json!({"rank": 0, "rankChange": 0, "totalPoints": 0, "today": 0, "thisWeek": 0})
    }

    #[tokio::test]
    async fn leaderboard_get_attaches_page_and_page_size_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/leaderboard"))
            .and(query_param("page", "2"))
            .and(query_param("pageSize", "25"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "entries": [entry(26)],
                "totalCount": 100,
                "page": 2,
                "totalPages": 4,
                "stats": stats_payload(),
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let page = client
            .leaderboard()
            .get()
            .page(2)
            .page_size(25)
            .send()
            .await
            .unwrap();
        assert_eq!(page.page, 2);
        assert_eq!(page.entries.len(), 1);
    }

    #[tokio::test]
    async fn leaderboard_get_omits_page_size_when_unset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/leaderboard"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "entries": [],
                "totalCount": 0,
                "page": 1,
                "totalPages": 0,
                "stats": stats_payload(),
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let _ = client.leaderboard().get().send().await.unwrap();
    }

    #[tokio::test]
    async fn leaderboard_iter_walks_pages_until_total() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/leaderboard"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "entries": [entry(1), entry(2)],
                "totalCount": 4,
                "page": 1,
                "totalPages": 2,
                "stats": stats_payload(),
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/leaderboard"))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "entries": [entry(3), entry(4)],
                "totalCount": 4,
                "page": 2,
                "totalPages": 2,
                "stats": stats_payload(),
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let collected: Vec<_> = client.leaderboard().iter().send().collect().await;
        assert_eq!(collected.len(), 4);
        let ranks: Vec<u32> = collected.into_iter().map(|r| r.unwrap().rank).collect();
        assert_eq!(ranks, vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn points_send_uses_page_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/points-leaderboard"))
            .and(query_param("page", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "entries": [],
                "totalCount": 0,
                "page": 3,
                "totalPages": 0,
                "currentUser": points_current_user_payload(),
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let page = client.leaderboard().points().page(3).send().await.unwrap();
        assert_eq!(page.page, 3);
    }

    #[tokio::test]
    async fn points_iter_single_page_then_stops() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/points-leaderboard"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "entries": [{
                    "rank": 1,
                    "address": "0xabc",
                    "username": "alice",
                    "usernameDisplay": "Alice",
                    "helmet": {"key": "knightmare", "color": "red"},
                    "totalPoints": 100,
                    "today": 10,
                    "thisWeek": 50,
                    "rankChange": 0,
                    "isOnline": true,
                    "rating": 1500.0,
                    "genuineGamesPlayed": 5,
                }],
                "totalCount": 1,
                "page": 1,
                "totalPages": 1,
                "currentUser": points_current_user_payload(),
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let collected: Vec<_> = client.leaderboard().points_iter().send().collect().await;
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].as_ref().unwrap().username, "alice");
    }
}
