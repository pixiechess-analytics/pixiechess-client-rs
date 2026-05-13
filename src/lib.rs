//! Unofficial async Rust client for the [`PixieChess`](https://www.pixiechess.xyz) API.
//!
//! Not affiliated with `PixieChess`. Consumes the public `api.pixiechess.xyz` surface.
//!
//! # Quickstart
//!
//! ```no_run
//! use pixiechess_client::PixieChessClient;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = PixieChessClient::new()?;
//!
//! let lb = client.leaderboard().get().page(1).send().await?;
//! for entry in lb.entries.iter().take(5) {
//!     println!("#{:<3} {} ({:.1})", entry.rank, entry.username_display, entry.rating);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! See `examples/basic.rs` for a complete runnable version.
//!
//! # Resource groups
//!
//! [`PixieChessClient`] exposes one accessor per logical endpoint group:
//!
//! - [`users`](PixieChessClient::users) — `GET /user/{id}`, `GET /user/match-history/{address}`
//! - [`games`](PixieChessClient::games) — `GET /game/{gameId}` and the rating-change endpoint
//! - [`leaderboard`](PixieChessClient::leaderboard) — main + points leaderboards (paged + streamed)
//! - [`pieces`](PixieChessClient::pieces) — current + burned pieces for a wallet
//! - [`auctions`](PixieChessClient::auctions) — auction history, daily volume, prices
//! - [`tournaments`](PixieChessClient::tournaments) — list, details, waitlist
//! - [`misc`](PixieChessClient::misc) — public config, ETH/USD, vault balance, live feed
//! - [`ranks`](PixieChessClient::ranks) — `GET /ranks/masters`
//!
//! # Typed vs raw
//!
//! Every endpoint goes through a builder that ends in one of two terminals:
//!
//! - `.send().await?` — returns the typed model.
//! - `.raw().await?` — returns a [`serde_json::Value`] for callers that want
//!   to bypass deserialization.
//!
//! Some paged endpoints also expose an `iter()` builder that returns a
//! [`futures::Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html)
//! walking every page.
//!
//! # Custom User-Agent
//!
//! The upstream WAF returns a `202` empty-body challenge to non-browser-shaped
//! user agents, so the default `User-Agent` mirrors a recent Chrome build (see
//! [`DEFAULT_USER_AGENT`]). To identify your own integration safely, suffix the
//! default via [`PixieChessClientBuilder::user_agent`]:
//!
//! ```no_run
//! use pixiechess_client::{DEFAULT_USER_AGENT, PixieChessClient};
//!
//! let client = PixieChessClient::builder()
//!     .user_agent(format!("{DEFAULT_USER_AGENT} my-app/1.0"))
//!     .build()
//!     .unwrap();
//! ```
//!

mod client;
mod error;
mod http;
pub mod models;
mod pagination;
pub mod resources;

pub use client::{PixieChessClient, PixieChessClientBuilder};
pub use error::{Error, Result};
pub use http::DEFAULT_USER_AGENT;
