//! Miscellaneous models: public config, live-feed events, ETH/USD price.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current ETH/USD spot price returned by `GET /eth-usd-price`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EthUsdPrice {
    pub usd: f64,
}

/// Public configuration flags from `GET /config/public`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicConfig {
    pub open_to_all: bool,
}

/// One event from `GET /live-feed`. The `data` payload is opaque
/// (`HashMap<String, serde_json::Value>`) since shape varies by `type`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiveFeedEvent {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn eth_usd_price_decodes() {
        let p: EthUsdPrice = serde_json::from_value(json!({"usd": 2000.5})).unwrap();
        assert!((p.usd - 2000.5).abs() < f64::EPSILON);
    }

    #[test]
    fn public_config_decodes() {
        let c: PublicConfig = serde_json::from_value(json!({"openToAll": true})).unwrap();
        assert!(c.open_to_all);
    }

    #[test]
    fn live_feed_event_decodes() {
        let raw = json!({
            "_id": "evt1",
            "type": "game_finished",
            "data": {"gameId": "g1", "winner": "w"},
            "createdAt": "2026-05-13T12:00:00Z",
        });
        let e: LiveFeedEvent = serde_json::from_value(raw).unwrap();
        assert_eq!(e.id, "evt1");
        assert_eq!(e.type_, "game_finished");
        assert_eq!(e.data["gameId"], "g1");
    }
}
