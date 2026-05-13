# pixiechess-client (Rust)

Unofficial async Rust client for the [PixieChess](https://www.pixiechess.xyz) API.

> Not affiliated with PixieChess. Consumes the public `api.pixiechess.xyz` surface.

## Status

All read-only public endpoints covered (auth and websocket realtime are intentionally out of scope). Mirrors the resource split of the reference Python client.

| Resource | Endpoints |
|---|---|
| `users` | `GET /user/{id-or-address}`, `GET /user/match-history/{address}` (+ async stream) |
| `games` | `GET /game/{gameId}`, `GET /game/{gameId}/rating/{address}` |
| `leaderboard` | `GET /leaderboard`, `GET /points-leaderboard` (both + async streams) |
| `pieces` | `GET /pieces/{address}`, `GET /burned-pieces/{address}` (+ async streams) |
| `auctions` | `GET /auction/{address}`, `GET /auctions/{active,past,daily-volume,today-summary,last-completed-day-summary}`, `GET /auctions/piece/{key}[/daily-volume]`, `GET /prices` |
| `tournaments` | `GET /tournament/{list,details/{id},waitlist/{id}}` |
| `misc` | `GET /config/public`, `GET /eth-usd-price`, `GET /vault-balance`, `GET /live-feed` |
| `ranks` | `GET /ranks/masters` |

## Install

```toml
[dependencies]
pixiechess-client = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quickstart

```rust
use pixiechess_client::PixieChessClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PixieChessClient::new()?;

    let lb = client.leaderboard().get().page(1).send().await?;
    for entry in lb.entries.iter().take(5) {
        println!("#{:<3} {:<20} {:>7.1}", entry.rank, entry.username_display, entry.rating);
    }

    Ok(())
}
```

A complete runnable version lives at [`examples/basic.rs`](examples/basic.rs):

```text
cargo run --example basic
```

## Typed vs raw

Every endpoint goes through a builder that exposes two terminal methods. Pick the one that fits the caller:

```rust
// Typed: parses into the model.
let lb = client.leaderboard().get().page(2).page_size(25).send().await?;

// Raw: returns serde_json::Value (skips typed deserialization).
let raw = client.leaderboard().get().page(2).page_size(25).raw().await?;
```

One-shot endpoints follow the same shape:

```rust
let masters = client.ranks().masters().send().await?;
let raw_masters = client.ranks().masters().raw().await?;
```

Some paged endpoints additionally expose an `iter()` / `iter_*()` builder that returns a `Stream` walking every page:

```rust
use futures::StreamExt;

let mut s = client.leaderboard().iter().page_size(25).send();
while let Some(entry) = s.next().await {
    let entry = entry?;
    // …
}
```

## Shape-drift regression test

`tests/replay.rs` reads a JSON corpus produced by
[`pixiechess-har-utils`](https://github.com/pixiechess-analytics/pixiechess-har-utils)
(vendored into `tests/fixtures/pixiechess-api.json`) and deserializes every
captured response body into the model the client owns for that endpoint. Any
failure means the live API has drifted from the typed model — fix the model,
not the corpus.

The corpus is not consulted at runtime; the client always makes real HTTP
requests against `api.pixiechess.xyz`.

To refresh the corpus, run `pcha` from the har-utils repo and overwrite
`tests/fixtures/pixiechess-api.json` with the result.

## Live smoke

`tests/live.rs` carries one `#[tokio::test] #[ignore]` per endpoint group that
hits the real API. Ignored by default; run on demand:

```text
cargo test --test live -- --ignored
```

## Stack

- Rust, edition 2024, MSRV 1.85
- Async via `tokio`
- HTTP via `reqwest` with `rustls-tls`
- `serde` for typed models, `serde_json::Value` for the raw path
- `chrono` for date / datetime fields
- Paged iters via `async-stream` + `futures::Stream`

## License

MIT. See [`LICENSE`](LICENSE).
