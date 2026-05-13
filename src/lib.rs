//! Unofficial async Rust client for the [PixieChess](https://www.pixiechess.xyz) API.
//!
//! This crate is being built up incrementally — see the project README for the
//! status of each endpoint and the planned shape of the public API.
//!
//! Every endpoint will eventually expose two terminal methods on its builder:
//!
//! - `.send().await?` — returns the typed model.
//! - `.raw().await?` — returns a `serde_json::Value` for callers that want
//!   to bypass deserialization or work with the raw payload.
//!
//! Shape drift between the live API and the typed models is caught by a replay
//! test in `tests/replay.rs` that walks the corpus produced by
//! [`pixiechess-har-utils`](https://github.com/pixiechess-analytics/pixiechess-har-utils).

mod client;
mod error;
mod http;

pub use client::{PixieChessClient, PixieChessClientBuilder};
pub use error::{Error, Result};
