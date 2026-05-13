# pixiechess-client (Rust)

Unofficial async Rust client for the [PixieChess](https://www.pixiechess.xyz) API.

> Not affiliated with PixieChess. Consumes the public `api.pixiechess.xyz` surface.

## Endpoints

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

Auth and websocket realtime are out of scope.

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
let history = client.users().match_history("0xabc").page(2).limit(25).send().await?;

// Raw: returns serde_json::Value (skips typed deserialization).
let raw = client.users().match_history("0xabc").page(2).limit(25).raw().await?;
```

One-shot endpoints follow the same shape:

```rust
let masters = client.ranks().masters().send().await?;
let raw_masters = client.ranks().masters().raw().await?;
```

Some paged endpoints additionally expose an `iter()` / `iter_*()` builder that returns a `Stream` walking every page:

```rust
use futures::StreamExt;

let mut s = client.leaderboard().iter().send();
while let Some(entry) = s.next().await {
    let entry = entry?;
    // …
}
```

## Custom User-Agent

The upstream WAF returns a `202` empty-body challenge to non-browser-shaped user agents, so the default `User-Agent` mirrors a recent Chrome build (`pixiechess_client::DEFAULT_USER_AGENT`). To identify your own integration without losing WAF compatibility, suffix the default rather than replacing it:

```rust
use pixiechess_client::{DEFAULT_USER_AGENT, PixieChessClient};

let client = PixieChessClient::builder()
    .user_agent(format!("{DEFAULT_USER_AGENT} my-app/1.0"))
    .build()?;
```

`PixieChessClientBuilder::user_agent` has replace semantics, so a fully-custom UA is fine too — just be aware the WAF may reject it.

## Stack

- Rust, edition 2024, MSRV 1.85
- Async via `tokio`
- HTTP via `reqwest` with `rustls-tls`
- `serde` for typed models, `serde_json::Value` for the raw path
- `chrono` for date / datetime fields
- Paged iters via `async-stream` + `futures::Stream`

## License

MIT. See [`LICENSE`](LICENSE).

---

See [`DEVELOPMENT.md`](DEVELOPMENT.md) for notes on how the API surface is kept in sync with the upstream and how no-op query params are filtered out.
