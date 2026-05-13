//! Piece (NFT) models — owned pieces, burned-piece records, and the
//! shared metadata/attribute shapes.

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

/// Tournament context attached to a burned piece. `name` and `color`
/// are always present; `tournament_id` is absent on burns not tied to
/// a tournament redemption.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BurnedTournament {
    #[serde(default)]
    pub tournament_id: Option<String>,
    pub name: String,
    pub color: String,
}

/// Burn record attached to a piece. `time` is the Unix-ish timestamp
/// of the burn; `tournament` is always present (the server emits a stub
/// block even when not tournament-linked).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BurnedInfo {
    pub time: i64,
    pub tournament: BurnedTournament,
}

/// A single piece (NFT). Shared shape for both `/pieces/{address}` (live
/// pieces, carry `count`) and `/burned-pieces/{address}` (burned pieces,
/// carry `burned` + `original_asset_id`). Fields that appear in *both*
/// payloads are required; per-endpoint extras stay optional.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    #[serde(rename = "_id")]
    pub id: String,
    pub collection_address: String,
    pub token_id: i64,
    pub owner: String,
    /// Almost always present; a small fraction of older NFTs ship without
    /// a `metadata` block.
    #[serde(default)]
    pub metadata: Option<PieceMetadata>,
    pub last_transfer_block_number: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// Live pieces (`/pieces/{address}` with `grouped=true`) carry the
    /// number of duplicate tokens collapsed into this row. Absent on
    /// `/burned-pieces/{address}`.
    #[serde(default)]
    pub count: Option<u32>,
    /// Set on `/burned-pieces/{address}`; absent on live pieces.
    #[serde(default)]
    pub burned: Option<BurnedInfo>,
    /// Set on `/burned-pieces/{address}`; absent on live pieces.
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

    fn live_piece_payload() -> serde_json::Value {
        json!({
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
            "lastTransferBlockNumber": 12345,
            "createdAt": "2026-05-13T12:00:00Z",
            "updatedAt": "2026-05-13T12:00:00Z",
            "count": 3,
        })
    }

    #[test]
    fn piece_live_with_metadata_and_attributes() {
        let p: Piece = serde_json::from_value(live_piece_payload()).unwrap();
        assert_eq!(p.token_id, 42);
        assert_eq!(p.count, Some(3));
        assert!(p.burned.is_none());
        let md = p
            .metadata
            .as_ref()
            .expect("metadata present in this payload");
        assert_eq!(md.attributes.len(), 3);
        assert_eq!(md.attributes[1].value, json!(7));
    }

    #[test]
    fn piece_burned_with_tournament_metadata() {
        let raw = json!({
            "_id": "abc",
            "collectionAddress": "0xc0ll",
            "tokenId": 42,
            "owner": "0x000",
            "metadata": {"attributes": []},
            "lastTransferBlockNumber": 12345,
            "createdAt": "2026-05-13T12:00:00Z",
            "updatedAt": "2026-05-13T12:00:00Z",
            "originalAssetId": "original-asset",
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
        assert_eq!(b.tournament.name, "Daily");
        assert_eq!(
            b.tournament.tournament_id.as_deref(),
            Some("tournament_175_xyz")
        );
        assert_eq!(p.original_asset_id.as_deref(), Some("original-asset"));
    }

    #[test]
    fn burned_tournament_accepts_missing_tournament_id() {
        // Burns not tied to a tournament redemption omit tournamentId;
        // name + color stay required.
        let raw = json!({"name": "Daily", "color": "white"});
        let t: BurnedTournament = serde_json::from_value(raw).unwrap();
        assert!(t.tournament_id.is_none());
    }

    #[test]
    fn piece_missing_required_field_errors() {
        let mut raw = live_piece_payload();
        raw.as_object_mut().unwrap().remove("collectionAddress");
        let res: Result<Piece, _> = serde_json::from_value(raw);
        assert!(res.is_err());
    }

    #[test]
    fn pieces_page_decodes_with_meta_envelope() {
        let raw = json!({
            "pieces": [live_piece_payload()],
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
