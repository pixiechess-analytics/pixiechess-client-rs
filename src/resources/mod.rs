//! Resource modules — one per logical `PixieChess` endpoint group.
//!
//! Each resource exposes a chain of the form
//! `client.<resource>().<endpoint>(args).send()/.raw().await`. The
//! `send()` terminal returns the typed model; `raw()` returns a
//! [`serde_json::Value`] for callers that want to bypass deserialization.

pub mod users;
