//! Game models — single-game state plus the per-player rating-change
//! response.
//!
//! Mirrors `pixiechess-client-py-old/src/pixiechess_client/models/game.py`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::common::ResponseMeta;

/// Inner block on a finished game describing how it ended (often empty
/// today, room for the server to add more diagnostic fields).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GameEnding {
    #[serde(default)]
    pub piece_key: Option<String>,
}

/// Result block on a finished game.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GameResult {
    pub code: String,
    pub winner: i64,
    #[serde(default)]
    pub piece_key: Option<String>,
    pub winner_id: String,
    #[serde(default)]
    pub ending: Option<GameEnding>,
}

/// Single-game state, returned by `GET /game/{gameId}`.
///
/// `board` and `players` are intentionally kept as raw
/// [`serde_json::Value`]: `board` is a full chess game-state document
/// (move history, FEN, draw offers, piece-mapping, …) that would explode
/// the model and bind the client tightly to upstream gameplay-engine
/// internals, and `players` carries per-side runtime state whose schema
/// genuinely varies between in-progress and finished games. The corpus
/// captures only two examples; that's too few to confidently lock down
/// the fields that vary by game state (`status`, `result`, `finished_at`,
/// `tournament_id`, etc.), so those stay `Option<_>` until we have richer
/// observation data.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[serde(rename = "_id")]
    pub id: String,
    pub game_id: String,
    pub board: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(default)]
    pub players: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub player_ids: HashMap<String, String>,
    #[serde(default)]
    pub player_statuses: HashMap<String, bool>,
    #[serde(default)]
    pub tournament_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub result: Option<GameResult>,
    #[serde(default)]
    pub rated: bool,
    #[serde(default)]
    pub spectator_limit: Option<u32>,
    #[serde(default)]
    pub game_duration_ms: Option<u64>,
    #[serde(default)]
    pub piece_selection_timeout_ms: Option<u64>,
    #[serde(default)]
    pub piece_selection_start_time: Option<i64>,
    /// Unix-milliseconds timestamp; the server emits this as a raw integer
    /// (not an RFC-3339 string like `created_at`/`updated_at`). Present on
    /// finished games (~70% of captured records).
    #[serde(default)]
    pub finished_at: Option<i64>,
    /// Only set when the server flips it; absent on most rows. Distinct
    /// from a literal `false`, hence `Option<bool>` rather than `bool` +
    /// `#[serde(default)]`.
    #[serde(default)]
    pub rematch_declined: Option<bool>,
    /// Set on games where the resign was reconciled offline by a backfill
    /// job. ~13% of finished games in the captured sample.
    #[serde(default)]
    pub backfilled_early_resign: Option<bool>,
    /// `_meta.suggestSignup` prompt the server sometimes attaches.
    #[serde(rename = "_meta", default)]
    pub meta: Option<ResponseMeta>,
}

/// Response from `GET /game/{gameId}/rating/{address}` — the rating
/// delta for a single player on a single game.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RatingChange {
    pub rated: bool,
    #[serde(default)]
    pub rating_before: Option<f64>,
    #[serde(default)]
    pub rating_after: Option<f64>,
    #[serde(default)]
    pub change: Option<f64>,
    /// `_meta.suggestSignup` prompt the server sometimes attaches.
    #[serde(rename = "_meta", default)]
    pub meta: Option<ResponseMeta>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn game_minimal_only_required_fields() {
        let raw = json!({
            "_id": "507f1f77bcf86cd799439011",
            "gameId": "game_1_abc",
            "board": {},
            "createdAt": "2026-05-13T12:00:00Z",
            "updatedAt": "2026-05-13T12:00:00Z",
        });
        let g: Game = serde_json::from_value(raw).unwrap();
        assert_eq!(g.id, "507f1f77bcf86cd799439011");
        assert_eq!(g.game_id, "game_1_abc");
        assert!(g.tournament_id.is_none());
        assert!(g.player_ids.is_empty());
        assert!(!g.rated);
    }

    #[test]
    fn game_with_result_and_tournament_binding() {
        let raw = json!({
            "_id": "abc",
            "gameId": "tournament_175_xyz_r7_p0_rm0",
            "board": {"moves": [], "fen": "..."},
            "createdAt": "2026-05-13T12:00:00Z",
            "updatedAt": "2026-05-13T12:00:00Z",
            "playerIds": {"white": "did:privy:foo", "black": "did:privy:bar"},
            "playerStatuses": {"white": true, "black": true},
            "tournamentId": "tournament_175_xyz",
            "status": "finished",
            "result": {
                "code": "checkmate",
                "winner": 1,
                "winnerId": "did:privy:foo",
                "ending": {"pieceKey": "queen"},
            },
            "rated": true,
            "finishedAt": 1_778_675_002_726_i64,
        });
        let g: Game = serde_json::from_value(raw).unwrap();
        assert_eq!(g.tournament_id.as_deref(), Some("tournament_175_xyz"));
        assert!(g.rated);
        let r = g.result.unwrap();
        assert_eq!(r.code, "checkmate");
        assert_eq!(r.ending.unwrap().piece_key.as_deref(), Some("queen"));
    }

    #[test]
    fn rating_change_unrated() {
        let raw = json!({"rated": false});
        let rc: RatingChange = serde_json::from_value(raw).unwrap();
        assert!(!rc.rated);
        assert!(rc.change.is_none());
    }

    #[test]
    fn rating_change_full() {
        let raw = json!({
            "rated": true,
            "ratingBefore": 1500.0,
            "ratingAfter": 1512.5,
            "change": 12.5,
        });
        let rc: RatingChange = serde_json::from_value(raw).unwrap();
        assert!(rc.rated);
        assert!(rc.change.is_some());
    }
}
