//! Typed models for `api.pixiechess.xyz` response bodies.
//!
//! Names mirror the reference Python client at `pixiechess-client-py-old`.
//! Each model derives `Deserialize` (via serde) with `camelCase` aliasing —
//! field names are `snake_case` in Rust, `camelCase` on the wire.

pub mod common;
pub mod user;

pub use common::{Helmet, PlayerInfo, ResponseMeta, SuggestSignup};
pub use user::{ColorRecord, MatchHistoryEntry, MatchHistoryPage, MatchTiming, User};
