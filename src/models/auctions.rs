//! Auction-related models.
//!
//! Mirrors `pixiechess-client-py-old/src/pixiechess_client/models/auctions.py`.

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::models::common::date_dict;

/// Piece-key + sub-key identifiers attached to an auction.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuctionMetadata {
    pub piece_key: String,
    pub sub_key: String,
}

/// Active or scheduled auction. `GET /auction/{address}` returns the
/// `auction` field of `{auction: …}` (the resource unwraps that).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Auction {
    #[serde(rename = "_id")]
    pub id: String,
    pub address: String,
    pub created_at: DateTime<Utc>,
    pub end_time: i64,
    pub start_time: i64,
    pub metadata: AuctionMetadata,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_mint_price_in_wei: Option<String>,
}

/// A finished auction returned under `/auctions/past`. Shares all
/// `Auction` fields and adds `end_date` (from a `"MM/DD"` string the
/// server emits) plus `final_price`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PastAuction {
    #[serde(rename = "_id")]
    pub id: String,
    pub address: String,
    pub created_at: DateTime<Utc>,
    pub end_time: i64,
    pub start_time: i64,
    pub metadata: AuctionMetadata,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_mint_price_in_wei: Option<String>,

    #[serde(default, deserialize_with = "md_date::deserialize_opt")]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub final_price: Option<String>,
}

/// `GET /auctions/piece/{piece_key}` summary.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuctionPieceInfo {
    pub piece_key: String,
    pub has_auction_history: bool,
    pub total_units_sold: u32,
    #[serde(default)]
    pub most_recent_past_auction: Option<PastAuction>,
}

/// One row from the VRGDA price block returned by `/prices`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VrgdaPrice {
    pub address: String,
    pub price: String,
    pub total_sold: u32,
    pub max_mints: u32,
    #[serde(default)]
    pub price_trend: Option<String>,
}

/// Instant-mint price block on `/prices` (optional).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstantMintPrice {
    pub address: String,
    pub price: String,
    pub total_sold: u32,
    pub max_mints: u32,
    #[serde(default)]
    pub price_trend: Option<String>,
}

/// `GET /auctions/today-summary` — today's totals so far.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuctionDaySummary {
    pub pieces_sold: u32,
    pub total_sales_eth: f64,
}

/// `GET /auctions/last-completed-day-summary`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletedDaySummary {
    #[serde(deserialize_with = "date_dict::deserialize")]
    pub date: NaiveDate,
    pub total_eth: f64,
    pub pieces_sold: u32,
    pub eth_change_percent: f64,
}

/// One day-row from `/auctions/daily-volume` or
/// `/auctions/piece/{piece_key}/daily-volume`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DailyVolume {
    #[serde(deserialize_with = "date_dict::deserialize")]
    pub date: NaiveDate,
    pub pieces_sold: u32,
    pub total_eth: f64,
}

/// `GET /prices` — current pricing snapshot.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Prices {
    pub vrgda: Vec<VrgdaPrice>,
    #[serde(default)]
    pub instant_mint: Option<InstantMintPrice>,
    pub poll_interval_ms: u32,
}

/// Aggregate stats attached to a `PastDayBucket`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SalesStats {
    pub pieces_sold: u32,
    pub total_eth_volume: f64,
    pub lowest_price: f64,
    pub highest_price: f64,
}

/// One past-auction row inside a day bucket.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PastAuctionEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub address: String,
    pub end_time: i64,
    pub start_time: i64,
    pub metadata: AuctionMetadata,
    #[serde(rename = "type")]
    pub type_: String,
    pub date_obj: DateTime<Utc>,
    pub sales_stats: SalesStats,
}

/// One day-bucket inside `PastAuctionsPage`. The server sometimes sends
/// the date under `_id` instead of `date`; `#[serde(alias = "_id")]`
/// accepts both.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PastDayBucket {
    #[serde(alias = "_id", deserialize_with = "date_dict::deserialize")]
    pub date: NaiveDate,
    pub auctions: Vec<PastAuctionEntry>,
}

/// `GET /auctions/past` paged response.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PastAuctionsPage {
    pub page: u32,
    pub page_size: u32,
    pub total_day_groups: u32,
    pub total_pages: u32,
    pub total_count: u32,
    pub day_buckets: Vec<PastDayBucket>,
}

/// Deserializer for `"MM/DD"` date strings the server sends on
/// `PastAuction.end_date`. The year defaults to the current calendar
/// year (mirrors the Python validator).
mod md_date {
    use chrono::{Datelike, NaiveDate, Utc};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize_opt<'de, D: Deserializer<'de>>(d: D) -> Result<Option<NaiveDate>, D::Error> {
        let Some(s) = Option::<String>::deserialize(d)? else {
            return Ok(None);
        };
        let mut parts = s.splitn(2, '/');
        let month: u32 = parts
            .next()
            .and_then(|m| m.parse().ok())
            .ok_or_else(|| serde::de::Error::custom("invalid month"))?;
        let day: u32 = parts
            .next()
            .and_then(|d| d.parse().ok())
            .ok_or_else(|| serde::de::Error::custom("invalid day"))?;
        let year = Utc::now().year();
        NaiveDate::from_ymd_opt(year, month, day)
            .map(Some)
            .ok_or_else(|| serde::de::Error::custom("invalid date"))
    }
}

// Helper so callers can construct dates without pulling in chrono::Datelike.
#[doc(hidden)]
#[must_use]
pub fn current_year_md(month: u32, day: u32) -> Option<NaiveDate> {
    let year = Utc::now().year();
    NaiveDate::from_ymd_opt(year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn auction_basic_decode() {
        let raw = json!({
            "_id": "auctionId",
            "address": "0xabc",
            "createdAt": "2026-05-13T12:00:00Z",
            "endTime": 1_700_000_000,
            "startTime": 1_699_000_000,
            "metadata": {"pieceKey": "knightmare", "subKey": "white"},
            "type": "vrgda",
            "lastMintPriceInWei": "1000000000000000000",
        });
        let a: Auction = serde_json::from_value(raw).unwrap();
        assert_eq!(a.metadata.piece_key, "knightmare");
        assert_eq!(a.type_, "vrgda");
        assert_eq!(
            a.last_mint_price_in_wei.as_deref(),
            Some("1000000000000000000")
        );
    }

    #[test]
    fn past_auction_decodes_end_date_from_md_string() {
        let raw = json!({
            "_id": "x",
            "address": "0xabc",
            "createdAt": "2026-05-13T12:00:00Z",
            "endTime": 1,
            "startTime": 0,
            "metadata": {"pieceKey": "marauder", "subKey": "white"},
            "type": "vrgda",
            "endDate": "5/13",
            "finalPrice": "999",
        });
        let pa: PastAuction = serde_json::from_value(raw).unwrap();
        let d = pa.end_date.unwrap();
        let expected = current_year_md(5, 13).unwrap();
        assert_eq!(d, expected);
    }

    #[test]
    fn past_auction_handles_missing_end_date() {
        let raw = json!({
            "_id": "x",
            "address": "0xabc",
            "createdAt": "2026-05-13T12:00:00Z",
            "endTime": 1,
            "startTime": 0,
            "metadata": {"pieceKey": "samurai", "subKey": "white"},
            "type": "vrgda",
        });
        let pa: PastAuction = serde_json::from_value(raw).unwrap();
        assert!(pa.end_date.is_none());
    }

    #[test]
    fn daily_volume_decodes_date_dict() {
        let raw = json!({
            "date": {"year": 2026, "month": 5, "day": 13},
            "piecesSold": 12,
            "totalEth": 1.5,
        });
        let dv: DailyVolume = serde_json::from_value(raw).unwrap();
        assert_eq!(dv.date, NaiveDate::from_ymd_opt(2026, 5, 13).unwrap());
        assert_eq!(dv.pieces_sold, 12);
    }

    #[test]
    fn past_day_bucket_accepts_date_under_id_field() {
        // Server sometimes sends the date under `_id`.
        let raw = json!({
            "_id": {"year": 2026, "month": 5, "day": 13},
            "auctions": [],
        });
        let b: PastDayBucket = serde_json::from_value(raw).unwrap();
        assert_eq!(b.date, NaiveDate::from_ymd_opt(2026, 5, 13).unwrap());
    }

    #[test]
    fn past_day_bucket_accepts_date_under_date_field() {
        let raw = json!({
            "date": {"year": 2026, "month": 5, "day": 14},
            "auctions": [],
        });
        let b: PastDayBucket = serde_json::from_value(raw).unwrap();
        assert_eq!(b.date, NaiveDate::from_ymd_opt(2026, 5, 14).unwrap());
    }

    #[test]
    fn prices_with_optional_instant_mint() {
        let raw_with = json!({
            "vrgda": [],
            "instantMint": {
                "address": "0x000",
                "price": "1000",
                "totalSold": 5,
                "maxMints": 100,
            },
            "pollIntervalMs": 2000,
        });
        let p: Prices = serde_json::from_value(raw_with).unwrap();
        assert!(p.instant_mint.is_some());

        let raw_without = json!({
            "vrgda": [],
            "pollIntervalMs": 2000,
        });
        let p2: Prices = serde_json::from_value(raw_without).unwrap();
        assert!(p2.instant_mint.is_none());
    }

    #[test]
    fn completed_day_summary_uses_date_dict() {
        let raw = json!({
            "date": {"year": 2026, "month": 5, "day": 12},
            "totalEth": 2.5,
            "piecesSold": 100,
            "ethChangePercent": -5.3,
        });
        let s: CompletedDaySummary = serde_json::from_value(raw).unwrap();
        assert_eq!(s.date, NaiveDate::from_ymd_opt(2026, 5, 12).unwrap());
    }
}
