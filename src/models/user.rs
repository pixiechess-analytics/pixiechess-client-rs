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
///
/// All fields are required — every one is present and non-null in every
/// response captured in the fixture corpus. If the server starts omitting
/// or nulling a field, deserialization will fail loudly; that's intentional
/// (see `tools/audit_corpus.py`).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub address: String,

    pub username: String,
    pub username_display: String,
    pub wallet_client_type: String,
    pub helmet: Helmet,
    pub last_login: DateTime<Utc>,

    pub win_rate: u32,
    pub match_count: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub casual_games: u32,
    pub streak: i32,
    pub color_record: HashMap<String, ColorRecord>,
    pub trophies: u32,
    pub rating: f64,
    pub rd: f64,
    pub is_provisional: bool,
    pub peak_rating: f64,
    pub rated_games_played: u32,
    pub genuine_games_played: u32,
    pub points: u64,
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
///
/// `tournament_id` is present on tournament matches and absent on casual
/// ones. `rated` and `rating_change` appear only on rated entries. `winner`
/// is always present but null on draws.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchHistoryEntry {
    pub game_id: String,
    pub created_at: DateTime<Utc>,
    pub white: PlayerInfo,
    pub black: PlayerInfo,

    #[serde(default)]
    pub tournament_id: Option<String>,
    #[serde(default)]
    pub winner: Option<String>,
    pub result_for_user: String,
    pub outcome: String,
    #[serde(default)]
    pub rated: Option<bool>,
    #[serde(default)]
    pub rating_change: Option<f64>,
    pub timing: MatchTiming,
}

/// Paged response from `GET /user/match-history/{address}`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchHistoryPage {
    pub matches: Vec<MatchHistoryEntry>,
    pub total_pages: u32,
    pub current_page: u32,
    pub total_count: u32,
    #[serde(rename = "_meta", default)]
    pub meta: Option<ResponseMeta>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn full_user_json() -> serde_json::Value {
        json!({
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
        })
    }

    #[test]
    fn user_full_with_stats() {
        let u: User = serde_json::from_value(full_user_json()).unwrap();
        assert_eq!(u.username, "alice");
        assert_eq!(u.win_rate, 65);
        assert_eq!(u.wins, 65);
        assert_eq!(u.color_record["white"].wins, 30);
        assert_eq!(u.points, 4500);
        assert_eq!(u.helmet.key, "knightmare");
    }

    #[test]
    fn user_missing_required_field_errors() {
        // The corpus audit pins every field on `User`; removing any one
        // should fail to decode — guards against accidentally re-introducing
        // Option<> on a field the wire always supplies.
        let mut raw = full_user_json();
        raw.as_object_mut().unwrap().remove("lastLogin");
        let res: Result<User, _> = serde_json::from_value(raw);
        assert!(res.is_err());
    }

    fn full_player_json(addr: &str, name: &str) -> serde_json::Value {
        json!({
            "address": addr,
            "username": name,
            "usernameDisplay": name,
            "helmet": {"key": "knightmare", "color": "red"},
        })
    }

    #[test]
    fn match_history_page_with_entries() {
        let raw = json!({
            "matches": [{
                "gameId": "tournament_175_abc_r7_p0_rm0",
                "createdAt": "2026-05-13T12:00:00Z",
                "white": full_player_json("0xaaa", "alice"),
                "black": full_player_json("0xbbb", "bob"),
                "tournamentId": "tournament_175_abc",
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
        assert_eq!(page.matches[0].rated, Some(true));
        assert_eq!(
            page.matches[0].tournament_id.as_deref(),
            Some("tournament_175_abc")
        );
        assert_eq!(page.total_pages, 3);
        assert!(page.meta.is_none());
    }

    #[test]
    fn match_history_entry_casual_match_omits_optional_fields() {
        let raw = json!({
            "matches": [{
                "gameId": "casual_xyz",
                "createdAt": "2026-05-13T12:00:00Z",
                "white": full_player_json("0xaaa", "alice"),
                "black": full_player_json("0xbbb", "bob"),
                "winner": null,
                "resultForUser": "draw",
                "outcome": "stalemate",
                "timing": {
                    "whiteElapsedMs": 60_000,
                    "blackElapsedMs": 58_000,
                    "clockMs": 300_000,
                },
            }],
            "totalPages": 1,
            "currentPage": 1,
            "totalCount": 1,
        });
        let page: MatchHistoryPage = serde_json::from_value(raw).unwrap();
        let m = &page.matches[0];
        assert!(m.tournament_id.is_none());
        assert!(m.rated.is_none());
        assert!(m.rating_change.is_none());
        assert!(m.winner.is_none());
    }

    #[test]
    fn match_history_page_with_meta_envelope() {
        let raw = json!({
            "matches": [],
            "totalPages": 0,
            "currentPage": 0,
            "totalCount": 0,
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
