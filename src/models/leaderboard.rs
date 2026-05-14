//! Leaderboard models — both the rating-based `/leaderboard` and the
//! points-based `/points-leaderboard`.

use serde::{Deserialize, Serialize};

use crate::models::common::Helmet;

/// One row from `GET /leaderboard`.
///
/// `current_game_id` / `current_game_player` only appear on rows whose
/// player is mid-match; everything else is always present.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub address: String,
    pub username: String,
    pub username_display: String,
    /// Absent on some accounts.
    #[serde(default)]
    pub helmet: Option<Helmet>,
    pub rating: f64,
    pub is_provisional: bool,
    pub games_played: u32,
    pub genuine_games_played: u32,
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
///
/// `current_user`'s shape varies with the caller's auth state — this
/// client doesn't model the auth surface, so the field is intentionally
/// kept as raw [`serde_json::Value`].
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total_count: u32,
    pub page: u32,
    pub total_pages: u32,
    #[serde(default)]
    pub current_user: Option<serde_json::Value>,
    pub stats: LeaderboardStats,
}

/// One row from `GET /points-leaderboard`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PointsLeaderboardEntry {
    pub rank: u32,
    pub address: String,
    pub username: String,
    pub username_display: String,
    /// Absent on some accounts.
    #[serde(default)]
    pub helmet: Option<Helmet>,
    pub total_points: u64,
    pub today: u64,
    pub this_week: u64,
    pub rank_change: i32,
    pub is_online: bool,
    pub rating: f64,
    pub genuine_games_played: u32,
}

/// Compact `current_user` projection that the points-leaderboard endpoint
/// attaches when the caller is identified (or as a zeroed placeholder for
/// guest requests). Distinct from [`PointsLeaderboardEntry`] because the
/// payload omits address/username/helmet/etc.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PointsLeaderboardCurrentUser {
    pub rank: u32,
    pub rank_change: i32,
    pub total_points: u64,
    pub today: u64,
    pub this_week: u64,
}

/// One page of `GET /points-leaderboard`.
///
/// `current_user` is only emitted when the caller is authenticated.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PointsLeaderboardPage {
    pub entries: Vec<PointsLeaderboardEntry>,
    pub total_count: u32,
    pub page: u32,
    pub total_pages: u32,
    #[serde(default)]
    pub current_user: Option<PointsLeaderboardCurrentUser>,
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

    fn points_entry_payload() -> serde_json::Value {
        json!({
            "rank": 7,
            "address": "0xdef",
            "username": "bob",
            "usernameDisplay": "Bob",
            "helmet": {"key": "knightmare", "color": "green"},
            "totalPoints": 5000,
            "today": 120,
            "thisWeek": 800,
            "rankChange": -2,
            "isOnline": false,
            "rating": 1750.0,
            "genuineGamesPlayed": 60,
        })
    }

    #[test]
    fn leaderboard_entry_full() {
        let e: LeaderboardEntry = serde_json::from_value(entry_payload()).unwrap();
        assert_eq!(e.rank, 1);
        assert_eq!(e.username, "alice");
        assert_eq!(e.helmet.as_ref().unwrap().key, "knightmare");
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
        assert_eq!(page.stats.total_ranked_players, 100);
    }

    #[test]
    fn points_leaderboard_entry_full() {
        let e: PointsLeaderboardEntry = serde_json::from_value(points_entry_payload()).unwrap();
        assert_eq!(e.rank, 7);
        assert_eq!(e.total_points, 5000);
        assert_eq!(e.helmet.as_ref().unwrap().key, "knightmare");
    }

    #[test]
    fn points_leaderboard_page_decodes() {
        let raw = json!({
            "entries": [],
            "totalCount": 0,
            "page": 1,
            "totalPages": 0,
            "currentUser": {
                "rank": 0,
                "rankChange": 0,
                "totalPoints": 0,
                "today": 0,
                "thisWeek": 0,
            },
        });
        let page: PointsLeaderboardPage = serde_json::from_value(raw).unwrap();
        assert!(page.entries.is_empty());
        assert_eq!(page.current_user.as_ref().unwrap().rank, 0);
    }

    #[test]
    fn points_leaderboard_page_omits_current_user_when_unauthenticated() {
        let raw = json!({
            "entries": [],
            "totalCount": 0,
            "page": 1,
            "totalPages": 0,
        });
        let page: PointsLeaderboardPage = serde_json::from_value(raw).unwrap();
        assert!(page.current_user.is_none());
    }
}
