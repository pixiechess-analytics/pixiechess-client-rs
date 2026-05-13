//! Tournament endpoints.

use crate::Result;
use crate::http::HttpClient;
use crate::models::tournaments::{TournamentDetails, TournamentList, WaitlistEntry};

/// Accessor for tournament endpoints. Obtained from
/// [`PixieChessClient::tournaments`](crate::PixieChessClient::tournaments).
pub struct TournamentsResource<'c> {
    http: &'c HttpClient,
}

impl<'c> TournamentsResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// `GET /tournament/list`. `limit` and `offset` are required by the
    /// server; the builder always sends them. `pinned`, `active`,
    /// `dateFilter`, and `tzOffset` are honored.
    #[must_use]
    pub fn list(&self) -> TournamentsListBuilder<'c> {
        TournamentsListBuilder {
            http: self.http,
            limit: 10,
            offset: 0,
            pinned: false,
            date_filter: None,
            tz_offset: None,
            active: None,
        }
    }

    /// `GET /tournament/details/{tournament_id}`.
    #[must_use]
    pub fn details(&self, tournament_id: impl Into<String>) -> TournamentDetailsBuilder<'c> {
        TournamentDetailsBuilder {
            http: self.http,
            tournament_id: tournament_id.into(),
        }
    }

    /// `GET /tournament/waitlist/{tournament_id}`.
    #[must_use]
    pub fn waitlist(&self, tournament_id: impl Into<String>) -> TournamentWaitlistBuilder<'c> {
        TournamentWaitlistBuilder {
            http: self.http,
            tournament_id: tournament_id.into(),
        }
    }
}

// `GET /tournament/list` ------------------------------------------------

/// Builder for [`TournamentsResource::list`].
pub struct TournamentsListBuilder<'c> {
    http: &'c HttpClient,
    limit: u32,
    offset: u32,
    pinned: bool,
    date_filter: Option<String>,
    tz_offset: Option<i32>,
    active: Option<bool>,
}

impl TournamentsListBuilder<'_> {
    /// Set `limit`. Default `10`.
    #[must_use]
    pub fn limit(mut self, n: u32) -> Self {
        self.limit = n;
        self
    }

    /// Set `offset`. Default `0`.
    #[must_use]
    pub fn offset(mut self, n: u32) -> Self {
        self.offset = n;
        self
    }

    /// Filter to pinned tournaments. Default `false`.
    #[must_use]
    pub fn pinned(mut self, v: bool) -> Self {
        self.pinned = v;
        self
    }

    /// Set `dateFilter` (omitted when unset).
    #[must_use]
    pub fn date_filter(mut self, s: impl Into<String>) -> Self {
        self.date_filter = Some(s.into());
        self
    }

    /// Set `tzOffset` minutes (omitted when unset). Only meaningful in
    /// combination with [`Self::date_filter`].
    #[must_use]
    pub fn tz_offset(mut self, n: i32) -> Self {
        self.tz_offset = Some(n);
        self
    }

    /// Set `active` filter (omitted when unset).
    #[must_use]
    pub fn active(mut self, v: bool) -> Self {
        self.active = Some(v);
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        let mut p = vec![
            ("limit", self.limit.to_string()),
            ("offset", self.offset.to_string()),
            ("pinned", self.pinned.to_string()),
        ];
        if let Some(d) = &self.date_filter {
            p.push(("dateFilter", d.clone()));
        }
        if let Some(t) = self.tz_offset {
            p.push(("tzOffset", t.to_string()));
        }
        if let Some(a) = self.active {
            p.push(("active", a.to_string()));
        }
        p
    }

    /// Fetch the typed [`TournamentList`].
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<TournamentList> {
        let params = self.params();
        self.http.get_with_params("/tournament/list", &params).await
    }

    /// Fetch as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let params = self.params();
        self.http.get_with_params("/tournament/list", &params).await
    }
}

// `GET /tournament/details/{tournament_id}` ----------------------------

/// Builder for [`TournamentsResource::details`].
pub struct TournamentDetailsBuilder<'c> {
    http: &'c HttpClient,
    tournament_id: String,
}

impl TournamentDetailsBuilder<'_> {
    fn path(&self) -> String {
        format!("/tournament/details/{}", self.tournament_id)
    }

    /// Fetch the typed [`TournamentDetails`].
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<TournamentDetails> {
        self.http.get(&self.path()).await
    }

    /// Fetch as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

// `GET /tournament/waitlist/{tournament_id}` ---------------------------

/// Builder for [`TournamentsResource::waitlist`].
pub struct TournamentWaitlistBuilder<'c> {
    http: &'c HttpClient,
    tournament_id: String,
}

impl TournamentWaitlistBuilder<'_> {
    fn path(&self) -> String {
        format!("/tournament/waitlist/{}", self.tournament_id)
    }

    /// Fetch the waitlist as a typed list.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Vec<WaitlistEntry>> {
        self.http.get(&self.path()).await
    }

    /// Fetch as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    fn minimal_tournament_json() -> serde_json::Value {
        json!({
            "tournamentId": "t-1",
            "registrationOpens": 1,
            "startTime": 2,
            "prizeAmount": 0.5,
            "prizeCurrency": "ETH",
            "name": "Cup",
            "description": "A cup",
            "images": {},
            "colors": {
                "primary": "#fff",
                "secondary": "#000",
                "gradient": "linear-gradient(...)",
            },
            "slots": 16,
            "pinned": false,
            "status": "scheduled",
            "createdAt": "2026-05-13T12:00:00Z",
        })
    }

    #[tokio::test]
    async fn list_attaches_default_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tournament/list"))
            .and(query_param("limit", "10"))
            .and(query_param("offset", "0"))
            .and(query_param("pinned", "false"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "totalCount": 0,
                "tournaments": [],
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let l = client.tournaments().list().send().await.unwrap();
        assert_eq!(l.total_count, 0);
    }

    #[tokio::test]
    async fn list_passes_optional_filters() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tournament/list"))
            .and(query_param("dateFilter", "today"))
            .and(query_param("tzOffset", "-300"))
            .and(query_param("active", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "totalCount": 1,
                "tournaments": [minimal_tournament_json()],
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let l = client
            .tournaments()
            .list()
            .date_filter("today")
            .tz_offset(-300)
            .active(true)
            .send()
            .await
            .unwrap();
        assert_eq!(l.tournaments.len(), 1);
    }

    #[tokio::test]
    async fn details_returns_typed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tournament/details/t-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": minimal_tournament_json(),
                "gameTimings": {},
                "gamePlayerIds": {},
                "gameStatuses": {},
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let d = client.tournaments().details("t-1").send().await.unwrap();
        assert_eq!(d.data.tournament_id, "t-1");
    }

    #[tokio::test]
    async fn waitlist_returns_vec_of_entries() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tournament/waitlist/t-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "_id": "w1",
                    "tournamentId": "t-1",
                    "address": "0xabc",
                    "createdAt": "2026-05-13T12:00:00Z",
                }
            ])))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let w = client.tournaments().waitlist("t-1").send().await.unwrap();
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].address, "0xabc");
    }
}
