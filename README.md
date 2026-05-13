# pixiechess-client (Rust)

Unofficial async Rust client for the [PixieChess](https://www.pixiechess.xyz) API.

> Not affiliated with PixieChess. Consumes the public `api.pixiechess.xyz` surface.

## Status

Scaffold. The crate compiles cleanly; no endpoints implemented yet. See the v0.1.0 milestone for the planned scope.

## Planned shape

Every endpoint exposes two terminal methods on its builder, so callers can either get a typed model or the raw JSON:

```rust
// Typed:
let lb = client.leaderboard().get().page(2).page_size(25).send().await?;

// Raw (bypasses deserialization):
let raw = client.leaderboard().get().page(2).page_size(25).raw().await?;
```

One-shot endpoints (no parameters) go through the same builder pattern for consistency:

```rust
let masters = client.ranks().masters().send().await?;
let raw_masters = client.ranks().masters().raw().await?;
```

## How shape drift is caught

A `tests/replay.rs` test reads a JSON corpus produced by [`pixiechess-har-utils`](https://github.com/pixiechess-analytics/pixiechess-har-utils) (vendored into `tests/fixtures/pixiechess-api.json`) and tries to deserialize every captured response body into the typed model the client owns for that endpoint. Any deserialization failure means the live API has drifted away from the typed model — fix the model, not the corpus.

The corpus is not consulted at runtime; the client makes real HTTP requests against `api.pixiechess.xyz`.

## Stack

- Rust, edition 2024, MSRV 1.85
- Async via `tokio` (added in a later branch)
- HTTP via `reqwest` with `rustls-tls`
- Serde for typed models, `serde_json::Value` for the raw path

## License

MIT. See [`LICENSE`](LICENSE).
