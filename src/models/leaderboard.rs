//! Leaderboard models — both the rating-based `/leaderboard` and the
//! points-based `/points-leaderboard`.
//!
//! Mirrors `pixiechess-client-py-old/src/pixiechess_client/models/leaderboard.py`.

use serde::{Deserialize, Serialize};

use crate::models::common::Helmet;

/// One row from `GET /leaderboard`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub address: String,
    pub username: String,
    pub username_display: String,
    #[serde(default)]
    pub helmet: Option<Helmet>,
    pub rating: f64,
    pub is_provisional: bool,
    pub games_played: u32,
    #[serde(default)]
    pub genuine_games_played: Option<u32>,
    pub wins: u32,
    pub streak: i32,
    pub is_online: bool,
    pub is_in_game: bool,
    #[serde(default)]
    pub current_game_id: Option<String>,
    #[serde(default)]
    pub current_game_player: Option<u32>,
}

/// Page-level stats served alongside the leaderboard entries.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardStats {
    pub total_ranked_players: u32,
    pub games_today: u32,
    pub active_now: u32,
}

/// One page of `GET /leaderboard`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total_count: u32,
    pub page: u32,
    pub total_pages: u32,
    #[serde(default)]
    pub current_user: Option<serde_json::Value>,
    #[serde(default)]
    pub stats: Option<LeaderboardStats>,
}

/// One row from `GET /points-leaderboard`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PointsLeaderboardEntry {
    pub rank: u32,
    pub address: String,
    pub username: String,
    pub username_display: String,
    #[serde(default)]
    pub helmet: Option<Helmet>,
    pub total_points: u64,
    pub today: u64,
    pub this_week: u64,
    pub rank_change: i32,
    pub is_online: bool,
    #[serde(default)]
    pub rating: Option<f64>,
    #[serde(default)]
    pub genuine_games_played: Option<u32>,
}

/// One page of `GET /points-leaderboard`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PointsLeaderboardPage {
    pub entries: Vec<PointsLeaderboardEntry>,
    pub total_count: u32,
    pub page: u32,
    pub total_pages: u32,
    #[serde(default)]
    pub current_user: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry_payload() -> serde_json::Value {
        json!({
            "rank": 1,
            "address": "0xabc",
            "username": "alice",
            "usernameDisplay": "Alice",
            "helmet": {"key": "knightmare", "color": "red"},
            "rating": 1900.0,
            "isProvisional": false,
            "gamesPlayed": 120,
            "genuineGamesPlayed": 115,
            "wins": 90,
            "streak": 4,
            "isOnline": true,
            "isInGame": false,
        })
    }

    #[test]
    fn leaderboard_entry_full() {
        let e: LeaderboardEntry = serde_json::from_value(entry_payload()).unwrap();
        assert_eq!(e.rank, 1);
        assert_eq!(e.username, "alice");
        assert!(e.helmet.is_some());
    }

    #[test]
    fn leaderboard_page_with_stats() {
        let raw = json!({
            "entries": [entry_payload()],
            "totalCount": 100,
            "page": 1,
            "totalPages": 4,
            "stats": {
                "totalRankedPlayers": 100,
                "gamesToday": 250,
                "activeNow": 30,
            },
        });
        let page: LeaderboardPage = serde_json::from_value(raw).unwrap();
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.total_pages, 4);
        let s = page.stats.unwrap();
        assert_eq!(s.total_ranked_players, 100);
    }

    #[test]
    fn points_leaderboard_entry_minimal() {
        // `rating`, `genuineGamesPlayed`, and `helmet` are all optional.
        let raw = json!({
            "rank": 7,
            "address": "0xdef",
            "username": "bob",
            "usernameDisplay": "Bob",
            "totalPoints": 5000,
            "today": 120,
            "thisWeek": 800,
            "rankChange": -2,
            "isOnline": false,
        });
        let e: PointsLeaderboardEntry = serde_json::from_value(raw).unwrap();
        assert_eq!(e.rank, 7);
        assert_eq!(e.total_points, 5000);
        assert!(e.rating.is_none());
    }

    #[test]
    fn points_leaderboard_page_decodes() {
        let raw = json!({
            "entries": [],
            "totalCount": 0,
            "page": 1,
            "totalPages": 0,
        });
        let page: PointsLeaderboardPage = serde_json::from_value(raw).unwrap();
        assert!(page.entries.is_empty());
        assert!(page.current_user.is_none());
    }
}
