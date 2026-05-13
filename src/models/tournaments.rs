//! Tournament-related models.
//!
//! Mirrors `pixiechess-client-py-old/src/pixiechess_client/models/tournaments.py`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::common::Helmet;

/// Image URLs attached to a tournament listing.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TournamentImages {
    #[serde(default)]
    pub trophy: Option<String>,
    #[serde(default)]
    pub title_card: Option<String>,
    #[serde(default)]
    pub title_card_centered: Option<String>,
    #[serde(default)]
    pub artwork: Option<String>,
    #[serde(default)]
    pub artwork_cropped: Option<String>,
    #[serde(default)]
    pub artwork_mobile: Option<String>,
}

/// Theming colors attached to a tournament listing.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TournamentColors {
    pub primary: String,
    pub secondary: String,
    pub gradient: String,
    #[serde(default)]
    pub gradient_button: bool,
}

/// One registered (or pending-registration) player's snapshot on a tournament.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TournamentUserInfo {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub username_display: Option<String>,
    #[serde(default)]
    pub helmet: Option<Helmet>,
    #[serde(default)]
    pub expires: Option<i64>,
    #[serde(default)]
    pub chosen_pieces: Vec<String>,
    #[serde(default)]
    pub free_piece_keys: Vec<String>,
    #[serde(default)]
    pub confirmed_entry_tx_hash: Option<String>,
    #[serde(default)]
    pub pending_burn_asset_ids: Vec<serde_json::Value>,
}

/// Burn ruleset for a tournament.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BurnRuleset {
    pub min: u32,
    pub max: u32,
    #[serde(default)]
    pub exclusive_pieces: Vec<String>,
    #[serde(default)]
    pub exclusive_piece_types: Vec<String>,
    #[serde(default)]
    pub banned_pieces: Vec<String>,
    #[serde(default)]
    pub banned_piece_types: Vec<String>,
}

/// Substitution rule for a gameplay ruleset.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubstitutionRule {
    pub min: u32,
    pub max: u32,
}

/// Gameplay ruleset attached to a tournament.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameplayRuleset {
    #[serde(default)]
    pub substitution: Option<SubstitutionRule>,
}

/// Full ruleset for a tournament.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TournamentRuleset {
    #[serde(default)]
    pub burn: Option<BurnRuleset>,
    #[serde(default)]
    pub gameplay: Option<GameplayRuleset>,
}

/// UI notification state for a tournament.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationStatus {
    #[serde(default)]
    pub registration_soon: bool,
    #[serde(default)]
    pub registration_open: bool,
    #[serde(default)]
    pub starting_soon_for: Vec<String>,
    #[serde(default)]
    pub starting_now: bool,
}

/// One placement entry in a tournament's payout split.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PayoutSplit {
    pub placement: u32,
    pub percentage: f64,
}

/// A single tournament listing.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)] // mirrors server payload 1:1
pub struct Tournament {
    #[serde(default, rename = "_id")]
    pub id: Option<String>,
    pub tournament_id: String,
    pub registration_opens: i64,
    pub start_time: i64,
    #[serde(default)]
    pub preset: Option<String>,
    pub prize_amount: f64,
    pub prize_currency: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub images: TournamentImages,
    pub colors: TournamentColors,
    pub slots: u32,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub test_pieces: Vec<String>,
    #[serde(default)]
    pub enable_free_pieces: bool,
    #[serde(default)]
    pub free_piece_keys: Vec<String>,
    #[serde(default)]
    pub game_duration_ms: Option<i64>,
    #[serde(default)]
    pub piece_selection_timeout_ms: Option<i64>,
    #[serde(default)]
    pub rematch_duration_ms: Option<i64>,
    #[serde(default)]
    pub additional_rematch_duration_ms: Option<i64>,
    #[serde(default)]
    pub schedule_id: Option<String>,
    #[serde(default)]
    pub schedule_position: Option<i64>,
    #[serde(default)]
    pub user_infos: Vec<TournamentUserInfo>,
    #[serde(default)]
    pub matchups_by_round: Vec<serde_json::Value>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ruleset: Option<TournamentRuleset>,
    #[serde(default)]
    pub notification_status: Option<NotificationStatus>,
    #[serde(default)]
    pub payout_mode: Option<String>,
    #[serde(default)]
    pub payout_splits: Vec<PayoutSplit>,
    #[serde(default)]
    pub payout_skipped_players: Vec<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub has_play_in_round: bool,
    #[serde(default)]
    pub winner_id: Option<String>,
    #[serde(default)]
    pub payout_status: Option<String>,
    #[serde(default)]
    pub payout_total_eth: Option<f64>,
    #[serde(default)]
    pub payout_started_at: Option<i64>,
    #[serde(default)]
    pub payout_completed_at: Option<i64>,
    #[serde(default)]
    pub user_infos_count: Option<i64>,
    #[serde(default)]
    pub confirmed_entries_count: Option<i64>,
    #[serde(default)]
    pub is_free_tournament: bool,
    #[serde(default)]
    pub hidden: bool,
}

/// One player's per-game timing.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameTimingPlayer {
    pub user_id: String,
    #[serde(default)]
    pub turn_start_time: Option<i64>,
    pub elapsed: i64,
    #[serde(default)]
    pub status: Option<String>,
}

/// Per-game timing attached to `TournamentDetails`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameTiming {
    #[serde(rename = "_id")]
    pub id: String,
    pub game_id: String,
    pub duration_ms: i64,
    pub players: Vec<GameTimingPlayer>,
    #[serde(default)]
    pub move_deadline: Option<i64>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// `GET /tournament/details/{tournament_id}` payload.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TournamentDetails {
    pub data: Tournament,
    #[serde(default)]
    pub game_timings: HashMap<String, GameTiming>,
    #[serde(default)]
    pub game_player_ids: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub game_statuses: HashMap<String, String>,
}

/// `GET /tournament/list` paged response.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TournamentList {
    pub total_count: u32,
    pub tournaments: Vec<Tournament>,
}

/// One row from `GET /tournament/waitlist/{tournament_id}`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WaitlistEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub tournament_id: String,
    pub address: String,
    #[serde(default)]
    pub has_been_attempted: bool,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn minimal_tournament_json() -> serde_json::Value {
        json!({
            "tournamentId": "t-1",
            "registrationOpens": 1,
            "startTime": 2,
            "prizeAmount": 0.5,
            "prizeCurrency": "ETH",
            "name": "Cup",
            "images": {},
            "colors": {
                "primary": "#fff",
                "secondary": "#000",
                "gradient": "linear-gradient(...)",
            },
            "slots": 16,
        })
    }

    #[test]
    fn tournament_decodes_minimal_payload() {
        let t: Tournament = serde_json::from_value(minimal_tournament_json()).unwrap();
        assert_eq!(t.tournament_id, "t-1");
        assert_eq!(t.slots, 16);
        assert!(!t.pinned);
    }

    #[test]
    fn tournament_list_decodes() {
        let raw = json!({
            "totalCount": 1,
            "tournaments": [minimal_tournament_json()],
        });
        let l: TournamentList = serde_json::from_value(raw).unwrap();
        assert_eq!(l.total_count, 1);
        assert_eq!(l.tournaments.len(), 1);
    }

    #[test]
    fn tournament_details_decodes_with_dict_fields() {
        let raw = json!({
            "data": minimal_tournament_json(),
            "gameTimings": {
                "g1": {
                    "_id": "id1",
                    "gameId": "g1",
                    "durationMs": 60_000,
                    "players": [{"userId": "u1", "elapsed": 1000}],
                }
            },
            "gamePlayerIds": {"g1": {"white": "u1", "black": "u2"}},
            "gameStatuses": {"g1": "ongoing"},
        });
        let d: TournamentDetails = serde_json::from_value(raw).unwrap();
        assert_eq!(d.game_timings.len(), 1);
        assert_eq!(d.game_player_ids["g1"]["white"], "u1");
        assert_eq!(d.game_statuses["g1"], "ongoing");
    }

    #[test]
    fn waitlist_entry_decodes() {
        let raw = json!({
            "_id": "w1",
            "tournamentId": "t-1",
            "address": "0xabc",
            "createdAt": "2026-05-13T12:00:00Z",
        });
        let e: WaitlistEntry = serde_json::from_value(raw).unwrap();
        assert_eq!(e.address, "0xabc");
        assert!(!e.has_been_attempted);
    }
}
