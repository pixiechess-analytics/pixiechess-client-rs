//! Miscellaneous endpoints: public config, vault balance, ETH/USD price, live feed.

use serde::Deserialize;

use crate::Result;
use crate::http::HttpClient;
use crate::models::misc::{EthUsdPrice, LiveFeedEvent, PublicConfig};

#[derive(Deserialize)]
struct VaultBalanceEnvelope {
    balance: String,
}

/// Accessor for misc endpoints. Obtained from
/// [`PixieChessClient::misc`](crate::PixieChessClient::misc).
pub struct MiscResource<'c> {
    http: &'c HttpClient,
}

impl<'c> MiscResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// `GET /config/public`.
    #[must_use]
    pub fn config(&self) -> ConfigBuilder<'c> {
        ConfigBuilder { http: self.http }
    }

    /// `GET /eth-usd-price`.
    #[must_use]
    pub fn eth_usd_price(&self) -> EthUsdPriceBuilder<'c> {
        EthUsdPriceBuilder { http: self.http }
    }

    /// `GET /vault-balance` (unwraps `{"balance": …}` to a `String`).
    #[must_use]
    pub fn vault_balance(&self) -> VaultBalanceBuilder<'c> {
        VaultBalanceBuilder { http: self.http }
    }

    /// `GET /live-feed`.
    #[must_use]
    pub fn live_feed(&self) -> LiveFeedBuilder<'c> {
        LiveFeedBuilder {
            http: self.http,
            since: None,
            type_: None,
            limit: None,
        }
    }
}

/// Builder for [`MiscResource::config`].
pub struct ConfigBuilder<'c> {
    http: &'c HttpClient,
}

impl ConfigBuilder<'_> {
    /// Fetch the typed [`PublicConfig`].
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<PublicConfig> {
        self.http.get("/config/public").await
    }

    /// Fetch as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/config/public").await
    }
}

/// Builder for [`MiscResource::eth_usd_price`].
pub struct EthUsdPriceBuilder<'c> {
    http: &'c HttpClient,
}

impl EthUsdPriceBuilder<'_> {
    /// Fetch the typed [`EthUsdPrice`].
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<EthUsdPrice> {
        self.http.get("/eth-usd-price").await
    }

    /// Fetch as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/eth-usd-price").await
    }
}

/// Builder for [`MiscResource::vault_balance`].
pub struct VaultBalanceBuilder<'c> {
    http: &'c HttpClient,
}

impl VaultBalanceBuilder<'_> {
    /// Fetch the balance string (unwrapped from the `{"balance": …}` envelope).
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<String> {
        let env: VaultBalanceEnvelope = self.http.get("/vault-balance").await?;
        Ok(env.balance)
    }

    /// Fetch the raw JSON (preserves the envelope).
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/vault-balance").await
    }
}

/// Builder for [`MiscResource::live_feed`].
pub struct LiveFeedBuilder<'c> {
    http: &'c HttpClient,
    since: Option<String>,
    type_: Option<String>,
    limit: Option<u32>,
}

impl LiveFeedBuilder<'_> {
    /// Set `since` (ISO-8601 datetime string).
    #[must_use]
    pub fn since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Set `type` filter.
    #[must_use]
    pub fn event_type(mut self, t: impl Into<String>) -> Self {
        self.type_ = Some(t.into());
        self
    }

    /// Set `limit`.
    #[must_use]
    pub fn limit(mut self, n: u32) -> Self {
        self.limit = Some(n);
        self
    }

    fn params(&self) -> Vec<(&'static str, String)> {
        let mut p = Vec::new();
        if let Some(s) = &self.since {
            p.push(("since", s.clone()));
        }
        if let Some(t) = &self.type_ {
            p.push(("type", t.clone()));
        }
        if let Some(n) = self.limit {
            p.push(("limit", n.to_string()));
        }
        p
    }

    /// Fetch the live feed as typed events.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Vec<LiveFeedEvent>> {
        let params = self.params();
        self.http.get_with_params("/live-feed", &params).await
    }

    /// Fetch as raw JSON.
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let params = self.params();
        self.http.get_with_params("/live-feed", &params).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    #[tokio::test]
    async fn config_returns_typed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/config/public"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"openToAll": true})))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let c = client.misc().config().send().await.unwrap();
        assert!(c.open_to_all);
    }

    #[tokio::test]
    async fn eth_usd_price_returns_typed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/eth-usd-price"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"usd": 2500.0})))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let p = client.misc().eth_usd_price().send().await.unwrap();
        assert_eq!(p.usd.to_string(), "2500");
    }

    #[tokio::test]
    async fn vault_balance_unwraps_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/vault-balance"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"balance": "1000000000"})),
            )
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let b = client.misc().vault_balance().send().await.unwrap();
        assert_eq!(b, "1000000000");
    }

    #[tokio::test]
    async fn live_feed_passes_all_filters() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/live-feed"))
            .and(query_param("since", "2026-05-12T00:00:00Z"))
            .and(query_param("type", "game_finished"))
            .and(query_param("limit", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "_id": "e1",
                    "type": "game_finished",
                    "data": {"gameId": "g1"},
                    "createdAt": "2026-05-13T12:00:00Z"
                }
            ])))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let events = client
            .misc()
            .live_feed()
            .since("2026-05-12T00:00:00Z")
            .event_type("game_finished")
            .limit(5)
            .send()
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "e1");
    }

    #[tokio::test]
    async fn live_feed_no_filters_sends_no_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/live-feed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let events = client.misc().live_feed().send().await.unwrap();
        assert!(events.is_empty());
    }
}
