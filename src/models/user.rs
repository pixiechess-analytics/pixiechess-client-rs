//! User profile + match history models.
//!
//! Mirrors `pixiechess-client-py-old/src/pixiechess_client/models/user.py`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::common::{Helmet, PlayerInfo, ResponseMeta};

/// Per-color win/loss/draw tally on a [`User`] record.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ColorRecord {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub total: u32,
}

/// Full user profile returned from `GET /user/{identifier}` (under the
/// `{"user": …}` envelope, which the resource unwraps before returning
/// this type).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub address: String,

    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub username_display: Option<String>,
    #[serde(default)]
    pub wallet_client_type: Option<String>,
    #[serde(default)]
    pub helmet: Option<Helmet>,
    #[serde(default)]
    pub last_login: Option<DateTime<Utc>>,

    #[serde(default)]
    pub win_rate: u32,
    #[serde(default)]
    pub match_count: u32,
    #[serde(default)]
    pub wins: u32,
    #[serde(default)]
    pub losses: u32,
    #[serde(default)]
    pub draws: u32,
    #[serde(default)]
    pub casual_games: Option<u32>,
    #[serde(default)]
    pub streak: i32,
    #[serde(default)]
    pub color_record: HashMap<String, ColorRecord>,
    #[serde(default)]
    pub trophies: u32,
    #[serde(default)]
    pub rating: f64,
    #[serde(default)]
    pub rd: Option<f64>,
    #[serde(default)]
    pub is_provisional: bool,
    #[serde(default)]
    pub peak_rating: Option<f64>,
    #[serde(default)]
    pub rated_games_played: Option<u32>,
    #[serde(default)]
    pub genuine_games_played: Option<u32>,
    #[serde(default)]
    pub points: Option<u64>,
}

/// Per-side timing on a single match entry.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MatchTiming {
    pub white_elapsed_ms: u64,
    pub black_elapsed_ms: u64,
    pub clock_ms: u64,
}

/// One row from `GET /user/match-history/{address}`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchHistoryEntry {
    pub game_id: String,
    pub created_at: DateTime<Utc>,
    pub white: PlayerInfo,
    pub black: PlayerInfo,

    #[serde(default)]
    pub winner: Option<String>,
    pub result_for_user: String,
    pub outcome: String,
    #[serde(default)]
    pub rated: bool,
    #[serde(default)]
    pub rating_change: Option<f64>,
    pub timing: MatchTiming,
}

/// Paged response from `GET /user/match-history/{address}`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MatchHistoryPage {
    pub matches: Vec<MatchHistoryEntry>,
    #[serde(default)]
    pub total_pages: Option<u32>,
    #[serde(default)]
    pub current_page: Option<u32>,
    #[serde(default)]
    pub total_count: Option<u32>,
    #[serde(rename = "_meta", default)]
    pub meta: Option<ResponseMeta>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn user_minimal_only_id_and_address() {
        let raw = json!({
            "_id": "507f1f77bcf86cd799439011",
            "address": "0xabc",
        });
        let u: User = serde_json::from_value(raw).unwrap();
        assert_eq!(u.id, "507f1f77bcf86cd799439011");
        assert_eq!(u.address, "0xabc");
        assert_eq!(u.win_rate, 0);
        assert!(u.username.is_none());
        assert!(u.color_record.is_empty());
    }

    #[test]
    fn user_full_with_stats() {
        let raw = json!({
            "_id": "507f1f77bcf86cd799439011",
            "address": "0xabc",
            "username": "alice",
            "usernameDisplay": "Alice",
            "walletClientType": "metamask",
            "helmet": {"key": "knightmare", "color": "red"},
            "lastLogin": "2026-05-13T12:34:56Z",
            "winRate": 65,
            "matchCount": 100,
            "wins": 65,
            "losses": 30,
            "draws": 5,
            "casualGames": 10,
            "streak": 3,
            "colorRecord": {
                "white": {"wins": 30, "losses": 15, "draws": 3, "total": 48},
                "black": {"wins": 35, "losses": 15, "draws": 2, "total": 52},
            },
            "trophies": 2,
            "rating": 1850.5,
            "rd": 45.2,
            "isProvisional": false,
            "peakRating": 1900.0,
            "ratedGamesPlayed": 90,
            "genuineGamesPlayed": 80,
            "points": 4500,
        });
        let u: User = serde_json::from_value(raw).unwrap();
        assert_eq!(u.username.as_deref(), Some("alice"));
        assert_eq!(u.win_rate, 65);
        assert_eq!(u.wins, 65);
        assert_eq!(u.color_record["white"].wins, 30);
        assert_eq!(u.points, Some(4500));
    }

    #[test]
    fn match_history_page_with_entries() {
        let raw = json!({
            "matches": [{
                "gameId": "tournament_175_abc_r7_p0_rm0",
                "createdAt": "2026-05-13T12:00:00Z",
                "white": {"address": "0xaaa", "username": "alice"},
                "black": {"address": "0xbbb", "username": "bob"},
                "winner": "white",
                "resultForUser": "win",
                "outcome": "checkmate",
                "rated": true,
                "ratingChange": 12.5,
                "timing": {
                    "whiteElapsedMs": 60_000,
                    "blackElapsedMs": 58_000,
                    "clockMs": 300_000,
                },
            }],
            "totalPages": 3,
            "currentPage": 1,
            "totalCount": 25,
        });
        let page: MatchHistoryPage = serde_json::from_value(raw).unwrap();
        assert_eq!(page.matches.len(), 1);
        assert_eq!(page.matches[0].game_id, "tournament_175_abc_r7_p0_rm0");
        assert_eq!(page.matches[0].rating_change, Some(12.5));
        assert_eq!(page.total_pages, Some(3));
        assert!(page.meta.is_none());
    }

    #[test]
    fn match_history_page_with_meta_envelope() {
        let raw = json!({
            "matches": [],
            "_meta": {"suggestSignup": {"reason": "guest_view"}},
        });
        let page: MatchHistoryPage = serde_json::from_value(raw).unwrap();
        assert!(page.matches.is_empty());
        assert_eq!(
            page.meta
                .as_ref()
                .unwrap()
                .suggest_signup
                .as_ref()
                .unwrap()
                .reason,
            "guest_view"
        );
    }
}
