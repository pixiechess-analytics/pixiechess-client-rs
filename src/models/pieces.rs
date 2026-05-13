//! Piece (NFT) models — owned pieces, burned-piece records, and the
//! shared metadata/attribute shapes.
//!
//! Mirrors `pixiechess-client-py-old/src/pixiechess_client/models/pieces.py`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::common::ResponseMeta;

/// One attribute on a piece. `value` is open-shape (string / int / float)
/// so it's kept as a [`serde_json::Value`].
///
/// Unlike most models in this crate, `trait_type` is `snake_case` on the
/// wire too (NFT metadata convention), so no `camelCase` rename here.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PieceAttribute {
    pub trait_type: String,
    pub value: serde_json::Value,
}

/// NFT-style metadata block on a piece.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PieceMetadata {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub animation_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: Vec<PieceAttribute>,
}

/// Tournament context attached to a burned piece (when the piece was
/// burned through a tournament redemption). All fields nullable per the
/// nullability fixes from the recent Python sync.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct BurnedTournament {
    #[serde(default)]
    pub tournament_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// Burn record attached to a piece. `time` is the Unix-ish timestamp
/// of the burn; `tournament` is set when the burn came from a
/// tournament redemption.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BurnedInfo {
    pub time: i64,
    #[serde(default)]
    pub tournament: Option<BurnedTournament>,
}

/// A single piece (NFT). Owned pieces and burned pieces share this shape;
/// burned pieces carry the optional `burned` block.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub collection_address: Option<String>,
    #[serde(default)]
    pub token_id: Option<i64>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub metadata: Option<PieceMetadata>,
    #[serde(default)]
    pub last_transfer_block_number: Option<i64>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default)]
    pub burned: Option<BurnedInfo>,
    #[serde(default)]
    pub original_asset_id: Option<String>,
}

/// One page of `GET /pieces/{address}` or `GET /burned-pieces/{address}`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PiecesPage {
    pub pieces: Vec<Piece>,
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

    #[test]
    fn piece_minimal_only_id() {
        let raw = json!({"_id": "abc"});
        let p: Piece = serde_json::from_value(raw).unwrap();
        assert_eq!(p.id, "abc");
        assert!(p.metadata.is_none());
        assert!(p.burned.is_none());
    }

    #[test]
    fn piece_with_metadata_and_attributes() {
        let raw = json!({
            "_id": "abc",
            "collectionAddress": "0xc0ll",
            "tokenId": 42,
            "owner": "0x000",
            "metadata": {
                "name": "Knightmare",
                "image": "ipfs://…",
                "attributes": [
                    {"trait_type": "rarity", "value": "epic"},
                    {"trait_type": "level", "value": 7},
                    {"trait_type": "power", "value": 12.5},
                ],
            },
            "count": 3,
        });
        let p: Piece = serde_json::from_value(raw).unwrap();
        assert_eq!(p.token_id, Some(42));
        let m = p.metadata.unwrap();
        assert_eq!(m.attributes.len(), 3);
        assert_eq!(m.attributes[1].value, json!(7));
        assert_eq!(m.attributes[2].value, json!(12.5));
    }

    #[test]
    fn piece_with_burned_info() {
        let raw = json!({
            "_id": "abc",
            "burned": {
                "time": 1_700_000_000,
                "tournament": {
                    "tournamentId": "tournament_175_xyz",
                    "name": "Daily",
                    "color": "white",
                },
            },
        });
        let p: Piece = serde_json::from_value(raw).unwrap();
        let b = p.burned.unwrap();
        assert_eq!(b.time, 1_700_000_000);
        assert_eq!(
            b.tournament.unwrap().tournament_id.as_deref(),
            Some("tournament_175_xyz")
        );
    }

    #[test]
    fn piece_burned_with_null_tournament_fields() {
        // The Python sync established that burned.tournament's fields
        // are all nullable; verify the Rust side matches.
        let raw = json!({
            "_id": "abc",
            "burned": {
                "time": 1,
                "tournament": {},
            },
        });
        let p: Piece = serde_json::from_value(raw).unwrap();
        let t = p.burned.unwrap().tournament.unwrap();
        assert!(t.tournament_id.is_none());
        assert!(t.name.is_none());
        assert!(t.color.is_none());
    }

    #[test]
    fn pieces_page_decodes_with_meta_envelope() {
        let raw = json!({
            "pieces": [{"_id": "p1"}],
            "totalPages": 3,
            "currentPage": 1,
            "totalCount": 25,
            "_meta": {"suggestSignup": {"reason": "guest_view"}},
        });
        let page: PiecesPage = serde_json::from_value(raw).unwrap();
        assert_eq!(page.pieces.len(), 1);
        assert_eq!(page.total_pages, 3);
        assert!(page.meta.is_some());
    }
}
