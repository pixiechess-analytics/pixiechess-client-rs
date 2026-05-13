//! Auction-related endpoints.

use serde::Deserialize;

use crate::Result;
use crate::http::HttpClient;
use crate::models::auctions::{
    Auction, AuctionDaySummary, AuctionPieceInfo, CompletedDaySummary, DailyVolume,
    PastAuctionsPage, Prices,
};

#[derive(Deserialize)]
struct AuctionEnvelope {
    auction: Auction,
}

#[derive(Deserialize)]
struct AuctionsListEnvelope {
    #[serde(default)]
    auctions: Vec<Auction>,
}

#[derive(Deserialize)]
struct DaysEnvelope {
    #[serde(default)]
    days: Vec<DailyVolume>,
}

/// Accessor for auction endpoints. Obtained from
/// [`PixieChessClient::auctions`](crate::PixieChessClient::auctions).
pub struct AuctionsResource<'c> {
    http: &'c HttpClient,
}

impl<'c> AuctionsResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// `GET /auction/{address}` (unwraps the `{"auction": …}` envelope).
    #[must_use]
    pub fn get(&self, address: impl Into<String>) -> AuctionsGetBuilder<'c> {
        AuctionsGetBuilder {
            http: self.http,
            address: address.into(),
        }
    }

    /// `GET /auctions/active` (unwraps `{"auctions": […]}`).
    #[must_use]
    pub fn active(&self) -> AuctionsActiveBuilder<'c> {
        AuctionsActiveBuilder { http: self.http }
    }

    /// `GET /auctions/past`.
    #[must_use]
    pub fn past(&self) -> AuctionsPastBuilder<'c> {
        AuctionsPastBuilder {
            http: self.http,
            page: 1,
            page_size: None,
        }
    }

    /// `GET /auctions/piece/{piece_key}`.
    #[must_use]
    pub fn piece_info(&self, piece_key: impl Into<String>) -> AuctionPieceInfoBuilder<'c> {
        AuctionPieceInfoBuilder {
            http: self.http,
            piece_key: piece_key.into(),
        }
    }

    /// `GET /auctions/piece/{piece_key}/daily-volume`.
    #[must_use]
    pub fn piece_daily_volume(&self, piece_key: impl Into<String>) -> PieceDailyVolumeBuilder<'c> {
        PieceDailyVolumeBuilder {
            http: self.http,
            piece_key: piece_key.into(),
            range: "30d".into(),
        }
    }

    /// `GET /auctions/daily-volume`.
    #[must_use]
    pub fn daily_volume(&self) -> DailyVolumeBuilder<'c> {
        DailyVolumeBuilder {
            http: self.http,
            range: "7d".into(),
        }
    }

    /// `GET /auctions/today-summary`.
    #[must_use]
    pub fn today_summary(&self) -> TodaySummaryBuilder<'c> {
        TodaySummaryBuilder { http: self.http }
    }

    /// `GET /auctions/last-completed-day-summary`.
    #[must_use]
    pub fn last_completed_day_summary(&self) -> LastCompletedDayBuilder<'c> {
        LastCompletedDayBuilder { http: self.http }
    }

    /// `GET /prices`.
    #[must_use]
    pub fn prices(&self) -> PricesBuilder<'c> {
        PricesBuilder { http: self.http }
    }
}

// `GET /auction/{address}` ---------------------------------------------

/// Builder for [`AuctionsResource::get`].
pub struct AuctionsGetBuilder<'c> {
    http: &'c HttpClient,
    address: String,
}

impl AuctionsGetBuilder<'_> {
    fn path(&self) -> String {
        format!("/auction/{}", self.address)
    }

    /// Fetch the auction and return the unwrapped [`Auction`].
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Auction> {
        let env: AuctionEnvelope = self.http.get(&self.path()).await?;
        Ok(env.auction)
    }

    /// Fetch the raw JSON (preserves the `{"auction": …}` envelope).
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

// `GET /auctions/active` ------------------------------------------------

/// Builder for [`AuctionsResource::active`].
pub struct AuctionsActiveBuilder<'c> {
    http: &'c HttpClient,
}

impl AuctionsActiveBuilder<'_> {
    /// Fetch the list of active auctions.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Vec<Auction>> {
        let env: AuctionsListEnvelope = self.http.get("/auctions/active").await?;
        Ok(env.auctions)
    }

    /// Fetch the raw JSON (preserves the `{"auctions": …}` envelope).
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/auctions/active").await
    }
}

// `GET /auctions/past` --------------------------------------------------

/// Builder for [`AuctionsResource::past`].
pub struct AuctionsPastBuilder<'c> {
    http: &'c HttpClient,
    page: u32,
    page_size: Option<u32>,
}

impl AuctionsPastBuilder<'_> {
    /// Set the 1-indexed page. Default `1`.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Set the page size. Omitted from the query when unset.
    #[must_use]
    pub fn page_size(mut self, n: u32) -> Self {
        self.page_size = Some(n);
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        let mut p = vec![("page", self.page.to_string())];
        if let Some(s) = self.page_size {
            p.push(("pageSize", s.to_string()));
        }
        p
    }

    /// Fetch the page.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<PastAuctionsPage> {
        let params = self.params();
        self.http.get_with_params("/auctions/past", &params).await
    }

    /// Fetch the page as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let params = self.params();
        self.http.get_with_params("/auctions/past", &params).await
    }
}

// `GET /auctions/piece/{piece_key}` ------------------------------------

/// Builder for [`AuctionsResource::piece_info`].
pub struct AuctionPieceInfoBuilder<'c> {
    http: &'c HttpClient,
    piece_key: String,
}

impl AuctionPieceInfoBuilder<'_> {
    fn path(&self) -> String {
        format!("/auctions/piece/{}", self.piece_key)
    }

    /// Fetch the auction info for this piece.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<AuctionPieceInfo> {
        self.http.get(&self.path()).await
    }

    /// Fetch the raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

// `GET /auctions/piece/{piece_key}/daily-volume` -----------------------

/// Builder for [`AuctionsResource::piece_daily_volume`].
pub struct PieceDailyVolumeBuilder<'c> {
    http: &'c HttpClient,
    piece_key: String,
    range: String,
}

impl PieceDailyVolumeBuilder<'_> {
    /// Set the `range` query param. Default `"30d"`.
    #[must_use]
    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = range.into();
        self
    }

    fn path(&self) -> String {
        format!("/auctions/piece/{}/daily-volume", self.piece_key)
    }

    /// Fetch the day-rows for this piece.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Vec<DailyVolume>> {
        let path = self.path();
        let params = [("range", self.range)];
        let env: DaysEnvelope = self.http.get_with_params(&path, &params).await?;
        Ok(env.days)
    }

    /// Fetch the raw JSON (preserves the `{"days": …}` envelope).
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let path = self.path();
        let params = [("range", self.range)];
        self.http.get_with_params(&path, &params).await
    }
}

// `GET /auctions/daily-volume` -----------------------------------------

/// Builder for [`AuctionsResource::daily_volume`].
pub struct DailyVolumeBuilder<'c> {
    http: &'c HttpClient,
    range: String,
}

impl DailyVolumeBuilder<'_> {
    /// Set the `range` query param. Default `"7d"`.
    #[must_use]
    pub fn range(mut self, range: impl Into<String>) -> Self {
        self.range = range.into();
        self
    }

    /// Fetch the day-rows.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Vec<DailyVolume>> {
        let params = [("range", self.range)];
        let env: DaysEnvelope = self
            .http
            .get_with_params("/auctions/daily-volume", &params)
            .await?;
        Ok(env.days)
    }

    /// Fetch the raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let params = [("range", self.range)];
        self.http
            .get_with_params("/auctions/daily-volume", &params)
            .await
    }
}

// One-shot summary endpoints -------------------------------------------

/// Builder for [`AuctionsResource::today_summary`].
pub struct TodaySummaryBuilder<'c> {
    http: &'c HttpClient,
}

impl TodaySummaryBuilder<'_> {
    /// Fetch today's summary.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<AuctionDaySummary> {
        self.http.get("/auctions/today-summary").await
    }

    /// Fetch the raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/auctions/today-summary").await
    }
}

/// Builder for [`AuctionsResource::last_completed_day_summary`].
pub struct LastCompletedDayBuilder<'c> {
    http: &'c HttpClient,
}

impl LastCompletedDayBuilder<'_> {
    /// Fetch the last-completed-day summary.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<CompletedDaySummary> {
        self.http.get("/auctions/last-completed-day-summary").await
    }

    /// Fetch the raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http
            .get_json("/auctions/last-completed-day-summary")
            .await
    }
}

// `GET /prices` ---------------------------------------------------------

/// Builder for [`AuctionsResource::prices`].
pub struct PricesBuilder<'c> {
    http: &'c HttpClient,
}

impl PricesBuilder<'_> {
    /// Fetch the current price snapshot.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Prices> {
        self.http.get("/prices").await
    }

    /// Fetch the raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/prices").await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    fn auction_payload() -> serde_json::Value {
        json!({
            "_id": "auctionId",
            "address": "0xabc",
            "createdAt": "2026-05-13T12:00:00Z",
            "endTime": 1,
            "startTime": 0,
            "metadata": {"pieceKey": "marauder", "subKey": "white"},
            "type": "vrgda",
        })
    }

    #[tokio::test]
    async fn get_unwraps_auction_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/auction/0xabc"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"auction": auction_payload()})),
            )
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let a = client.auctions().get("0xabc").send().await.unwrap();
        assert_eq!(a.address, "0xabc");
    }

    #[tokio::test]
    async fn active_unwraps_auctions_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/auctions/active"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "auctions": [auction_payload()],
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let list = client.auctions().active().send().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn past_attaches_page_and_page_size() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/auctions/past"))
            .and(query_param("page", "2"))
            .and(query_param("pageSize", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "page": 2,
                "pageSize": 10,
                "totalDayGroups": 0,
                "totalPages": 0,
                "totalCount": 0,
                "dayBuckets": [],
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let p = client
            .auctions()
            .past()
            .page(2)
            .page_size(10)
            .send()
            .await
            .unwrap();
        assert_eq!(p.page, 2);
    }

    #[tokio::test]
    async fn piece_info_returns_typed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/auctions/piece/marauder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pieceKey": "marauder",
                "hasAuctionHistory": true,
                "totalUnitsSold": 50,
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let info = client
            .auctions()
            .piece_info("marauder")
            .send()
            .await
            .unwrap();
        assert_eq!(info.total_units_sold, 50);
    }

    #[tokio::test]
    async fn daily_volume_attaches_range_and_unwraps_days_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/auctions/daily-volume"))
            .and(query_param("range", "30d"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "days": [{
                    "date": {"year": 2026, "month": 5, "day": 13},
                    "piecesSold": 2,
                    "totalEth": 0.5,
                }],
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let days = client
            .auctions()
            .daily_volume()
            .range("30d")
            .send()
            .await
            .unwrap();
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].pieces_sold, 2);
    }

    #[tokio::test]
    async fn prices_decodes_full_snapshot() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/prices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "vrgda": [{
                    "address": "0xa",
                    "price": "100",
                    "totalSold": 1,
                    "maxMints": 10,
                }],
                "pollIntervalMs": 2500,
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let p = client.auctions().prices().send().await.unwrap();
        assert_eq!(p.vrgda.len(), 1);
        assert!(p.instant_mint.is_none());
        assert_eq!(p.poll_interval_ms, 2500);
    }

    #[tokio::test]
    async fn today_summary_send() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/auctions/today-summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "piecesSold": 12,
                "totalSalesEth": 1.25,
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let s = client.auctions().today_summary().send().await.unwrap();
        assert_eq!(s.pieces_sold, 12);
    }
}
