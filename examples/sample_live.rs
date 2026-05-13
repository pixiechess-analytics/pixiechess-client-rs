//! Live-API sampler. Drives many requests against `api.pixiechess.xyz`
//! across users, pieces, tournaments, games, auctions, and pages, then
//! writes the responses to a corpus-shaped JSON file for offline auditing.
//!
//! The output matches the schema of `tests/fixtures/pixiechess-api.json`:
//!
//! ```jsonc
//! {
//!   "GET /user/{userId}": {
//!     "examples": [
//!       { "request": {"url": "/user/0xabc"}, "response": {"status": 200, "body": {...}} },
//!       ...
//!     ]
//!   },
//!   ...
//! }
//! ```
//!
//! Run with:
//!
//! ```text
//! cargo run --example sample_live --release -- --out /tmp/pixiechess-api-expanded.json
//! ```
//!
//! Then feed the file into `python3 tools/audit_corpus.py` (after pointing
//! `FIX` at the expanded path) to surface drift the small vendored corpus
//! couldn't.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};
use tokio::sync::Mutex;
use tokio::time::sleep;

use pixiechess_client::{Error, PixieChessClient};

/// Inter-request throttle. The upstream WAF rate-limits aggressively;
/// ~250ms between calls keeps us under the threshold across hundreds of
/// requests in a row.
const REQUEST_DELAY_MS: u64 = 250;
/// Max retries on `HTTP 429: Too many requests`. Backoff doubles each time.
const MAX_429_RETRIES: u32 = 5;
const INITIAL_429_BACKOFF_MS: u64 = 2_000;

const LEADERBOARD_PAGES: u32 = 5;
const POINTS_PAGES: u32 = 5;
const MATCH_HISTORY_PAGES: u32 = 2;
const PIECES_PAGES: u32 = 2;
const PAST_AUCTION_PAGES: u32 = 4;
const TOURNAMENT_LIST_PAGES: u32 = 3;
const USERS_TO_SAMPLE: usize = 30;
const GAMES_TO_SAMPLE: usize = 30;

type Corpus = BTreeMap<String, Vec<CorpusExample>>;

#[derive(Debug, Clone, serde::Serialize)]
struct CorpusExample {
    request: CorpusRequest,
    response: CorpusResponse,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CorpusRequest {
    url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CorpusResponse {
    status: u16,
    body: Value,
}

#[derive(Clone)]
struct Sink {
    inner: Arc<Mutex<Corpus>>,
}

impl Sink {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    async fn push(&self, key: &'static str, url: String, body: Value) {
        let mut g = self.inner.lock().await;
        g.entry(key.to_string()).or_default().push(CorpusExample {
            request: CorpusRequest { url },
            response: CorpusResponse { status: 200, body },
        });
    }

    async fn note_failure(&self, key: &'static str, url: &str, msg: &str) {
        let _ = self.inner.lock().await;
        eprintln!("  [warn] {key} {url} → {}", truncate(msg, 100));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out: PathBuf = std::env::args()
        .skip_while(|a| a != "--out")
        .nth(1)
        .map_or_else(
            || PathBuf::from("/tmp/pixiechess-api-expanded.json"),
            PathBuf::from,
        );

    let started = Instant::now();
    let client = PixieChessClient::new()?;
    let sink = Sink::new();

    println!("== one-shot endpoints ==");
    one_shot(&client, &sink).await;

    println!("== leaderboards ==");
    let addrs = leaderboards(&client, &sink).await;

    println!(
        "== users + match history + pieces ({} users) ==",
        addrs.len()
    );
    let game_ids = users_and_pieces(&client, &sink, &addrs).await;

    println!(
        "== games ({} discovered, sampling {}) ==",
        game_ids.len(),
        GAMES_TO_SAMPLE.min(game_ids.len())
    );
    games(&client, &sink, &game_ids, &addrs).await;

    println!("== auctions ==");
    let piece_keys = auctions(&client, &sink).await;

    println!(
        "== piece-specific auction endpoints ({} keys) ==",
        piece_keys.len()
    );
    auctions_piece(&client, &sink, &piece_keys).await;

    println!("== tournaments ==");
    tournaments(&client, &sink).await;

    let final_corpus = Arc::try_unwrap(sink.inner).ok().unwrap().into_inner();

    let mut total = 0usize;
    println!();
    println!("== corpus summary ==");
    for (k, v) in &final_corpus {
        println!("  {:<48} {:>5} examples", k, v.len());
        total += v.len();
    }
    println!(
        "  total: {total} examples across {} endpoints",
        final_corpus.len()
    );

    let mut root = Map::new();
    for (k, v) in final_corpus {
        root.insert(k, json!({ "examples": v }));
    }
    std::fs::create_dir_all(out.parent().unwrap_or(std::path::Path::new(".")))?;
    std::fs::write(&out, serde_json::to_string_pretty(&Value::Object(root))?)?;
    println!();
    println!(
        "wrote {} in {:.1}s",
        out.display(),
        started.elapsed().as_secs_f64()
    );

    Ok(())
}

/// Wraps the request closure with throttling + exponential backoff on 429.
async fn fetch<F, Fut>(sink: &Sink, key: &'static str, url: &str, mut make: F) -> Option<Value>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = pixiechess_client::Result<Value>>,
{
    sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;
    let mut attempt = 0u32;
    let mut backoff = INITIAL_429_BACKOFF_MS;
    loop {
        match make().await {
            Ok(v) => {
                sink.push(key, url.to_string(), v.clone()).await;
                return Some(v);
            }
            Err(Error::Api {
                status: 429,
                message,
            }) if attempt < MAX_429_RETRIES => {
                eprintln!(
                    "  [429] {key} {url} attempt {attempt} sleep {backoff}ms: {}",
                    truncate(&message, 60)
                );
                sleep(Duration::from_millis(backoff)).await;
                backoff = backoff.saturating_mul(2);
                attempt += 1;
            }
            Err(e) => {
                sink.note_failure(key, url, &e.to_string()).await;
                return None;
            }
        }
    }
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn write_active(url: &mut String, active: bool) {
    use std::fmt::Write as _;
    let _ = write!(url, "&active={active}");
}

async fn one_shot(client: &PixieChessClient, sink: &Sink) {
    fetch(sink, "GET /config/public", "/config/public", || {
        client.misc().config().raw()
    })
    .await;
    fetch(sink, "GET /eth-usd-price", "/eth-usd-price", || {
        client.misc().eth_usd_price().raw()
    })
    .await;
    fetch(sink, "GET /vault-balance", "/vault-balance", || {
        client.misc().vault_balance().raw()
    })
    .await;
    fetch(sink, "GET /ranks/masters", "/ranks/masters", || {
        client.ranks().masters().raw()
    })
    .await;
    fetch(sink, "GET /prices", "/prices", || {
        client.auctions().prices().raw()
    })
    .await;
    fetch(
        sink,
        "GET /auctions/today-summary",
        "/auctions/today-summary",
        || client.auctions().today_summary().raw(),
    )
    .await;
    fetch(
        sink,
        "GET /auctions/last-completed-day-summary",
        "/auctions/last-completed-day-summary",
        || client.auctions().last_completed_day_summary().raw(),
    )
    .await;

    for limit in [10u32, 50, 100] {
        let url = format!("/live-feed?limit={limit}");
        fetch(sink, "GET /live-feed", &url, || {
            client.misc().live_feed().limit(limit).raw()
        })
        .await;
    }

    for range in ["7d", "14d", "30d"] {
        let url = format!("/auctions/daily-volume?range={range}");
        fetch(sink, "GET /auctions/daily-volume", &url, || {
            client.auctions().daily_volume().range(range).raw()
        })
        .await;
    }
}

async fn leaderboards(client: &PixieChessClient, sink: &Sink) -> Vec<String> {
    let mut addrs: BTreeSet<String> = BTreeSet::new();

    for page in 1..=LEADERBOARD_PAGES {
        let url = format!("/leaderboard?page={page}");
        if let Some(v) = fetch(sink, "GET /leaderboard", &url, || {
            client.leaderboard().get().page(page).raw()
        })
        .await
        {
            if let Some(entries) = v.get("entries").and_then(|e| e.as_array()) {
                for e in entries {
                    if let Some(a) = e.get("address").and_then(|a| a.as_str()) {
                        addrs.insert(a.to_string());
                    }
                }
            }
        }
    }

    for page in 1..=POINTS_PAGES {
        let url = format!("/points-leaderboard?page={page}");
        if let Some(v) = fetch(sink, "GET /points-leaderboard", &url, || {
            client.leaderboard().points().page(page).raw()
        })
        .await
        {
            if let Some(entries) = v.get("entries").and_then(|e| e.as_array()) {
                for e in entries {
                    if let Some(a) = e.get("address").and_then(|a| a.as_str()) {
                        addrs.insert(a.to_string());
                    }
                }
            }
        }
    }

    addrs.into_iter().take(USERS_TO_SAMPLE).collect()
}

async fn users_and_pieces(client: &PixieChessClient, sink: &Sink, addrs: &[String]) -> Vec<String> {
    let mut game_ids: BTreeSet<String> = BTreeSet::new();

    for (i, addr) in addrs.iter().enumerate() {
        if i % 5 == 0 {
            println!("  user {}/{}", i + 1, addrs.len());
        }
        let url = format!("/user/{addr}");
        fetch(sink, "GET /user/{userId}", &url, || {
            client.users().get(addr.clone()).raw()
        })
        .await;

        // also fetch by username when we can — same shape, exercises the
        // alternate routing path.
        // (skipped to keep request volume bounded)

        for page in 1..=MATCH_HISTORY_PAGES {
            let url = format!("/user/match-history/{addr}?page={page}&limit=15");
            if let Some(v) = fetch(sink, "GET /user/match-history/{address}", &url, || {
                client
                    .users()
                    .match_history(addr.clone())
                    .page(page)
                    .limit(15)
                    .raw()
            })
            .await
            {
                if let Some(matches) = v.get("matches").and_then(|m| m.as_array()) {
                    for m in matches {
                        if let Some(gid) = m.get("gameId").and_then(|g| g.as_str()) {
                            game_ids.insert(gid.to_string());
                        }
                    }
                }
            }
        }

        for grouped in [false, true] {
            for page in 1..=PIECES_PAGES {
                let url = format!("/pieces/{addr}?page={page}&grouped={grouped}");
                fetch(sink, "GET /pieces/{address}", &url, || {
                    client
                        .pieces()
                        .get(addr.clone())
                        .page(page)
                        .grouped(grouped)
                        .raw()
                })
                .await;
            }
        }

        let url = format!("/burned-pieces/{addr}?page=1");
        fetch(sink, "GET /burned-pieces/{address}", &url, || {
            client.pieces().burned(addr.clone()).page(1).raw()
        })
        .await;
    }

    game_ids.into_iter().collect()
}

async fn games(client: &PixieChessClient, sink: &Sink, game_ids: &[String], addrs: &[String]) {
    for gid in game_ids.iter().take(GAMES_TO_SAMPLE) {
        let url = format!("/game/{gid}");
        fetch(sink, "GET /game/{gameId}", &url, || {
            client.games().get(gid.clone()).raw()
        })
        .await;
        if let Some(addr) = addrs.first() {
            let url = format!("/game/{gid}/rating/{addr}");
            fetch(sink, "GET /game/{gameId}/rating/{address}", &url, || {
                client
                    .games()
                    .rating_change(gid.clone(), addr.clone())
                    .raw()
            })
            .await;
        }
    }
}

async fn auctions(client: &PixieChessClient, sink: &Sink) -> Vec<String> {
    let mut piece_keys: BTreeSet<String> = BTreeSet::new();
    let mut addresses: Vec<String> = Vec::new();

    if let Some(v) = fetch(sink, "GET /auctions/active", "/auctions/active", || {
        client.auctions().active().raw()
    })
    .await
    {
        if let Some(arr) = v.get("auctions").and_then(|a| a.as_array()) {
            for a in arr {
                if let Some(pk) = a.pointer("/metadata/pieceKey").and_then(|p| p.as_str()) {
                    piece_keys.insert(pk.to_string());
                }
                if let Some(addr) = a.get("address").and_then(|a| a.as_str()) {
                    addresses.push(addr.to_string());
                }
            }
        }
    }

    for addr in &addresses {
        let url = format!("/auction/{addr}");
        fetch(sink, "GET /auction/{address}", &url, || {
            client.auctions().get(addr.clone()).raw()
        })
        .await;
    }

    for page in 1..=PAST_AUCTION_PAGES {
        let url = format!("/auctions/past?page={page}&pageSize=5");
        if let Some(v) = fetch(sink, "GET /auctions/past", &url, || {
            client.auctions().past().page(page).page_size(5).raw()
        })
        .await
        {
            if let Some(buckets) = v.get("dayBuckets").and_then(|b| b.as_array()) {
                for bucket in buckets {
                    if let Some(items) = bucket.get("auctions").and_then(|a| a.as_array()) {
                        for a in items {
                            if let Some(pk) =
                                a.pointer("/metadata/pieceKey").and_then(|p| p.as_str())
                            {
                                piece_keys.insert(pk.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    piece_keys.into_iter().collect()
}

async fn auctions_piece(client: &PixieChessClient, sink: &Sink, piece_keys: &[String]) {
    for pk in piece_keys {
        let url = format!("/auctions/piece/{pk}");
        fetch(sink, "GET /auctions/piece/{pieceKey}", &url, || {
            client.auctions().piece_info(pk.clone()).raw()
        })
        .await;
        for range in ["7d", "30d"] {
            let url = format!("/auctions/piece/{pk}/daily-volume?range={range}");
            fetch(
                sink,
                "GET /auctions/piece/{pieceKey}/daily-volume",
                &url,
                || {
                    client
                        .auctions()
                        .piece_daily_volume(pk.clone())
                        .range(range)
                        .raw()
                },
            )
            .await;
        }
    }
}

async fn tournaments(client: &PixieChessClient, sink: &Sink) {
    let mut tournament_ids: BTreeSet<String> = BTreeSet::new();

    for active in [None, Some(true), Some(false)] {
        for pinned in [false, true] {
            for page in 0..TOURNAMENT_LIST_PAGES {
                let offset = page * 10;
                let mut url = format!("/tournament/list?limit=10&offset={offset}&pinned={pinned}");
                if let Some(a) = active {
                    write_active(&mut url, a);
                }
                let v = fetch(sink, "GET /tournament/list", &url, || {
                    let mut b = client
                        .tournaments()
                        .list()
                        .limit(10)
                        .offset(offset)
                        .pinned(pinned);
                    if let Some(a) = active {
                        b = b.active(a);
                    }
                    b.raw()
                })
                .await;
                if let Some(v) = v {
                    if let Some(arr) = v.get("tournaments").and_then(|t| t.as_array()) {
                        for t in arr {
                            if let Some(id) = t.get("tournamentId").and_then(|i| i.as_str()) {
                                tournament_ids.insert(id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    for tid in &tournament_ids {
        let url = format!("/tournament/details/{tid}");
        fetch(sink, "GET /tournament/details/{tournamentId}", &url, || {
            client.tournaments().details(tid.clone()).raw()
        })
        .await;

        let url = format!("/tournament/waitlist/{tid}");
        fetch(
            sink,
            "GET /tournament/waitlist/{tournamentId}",
            &url,
            || client.tournaments().waitlist(tid.clone()).raw(),
        )
        .await;
    }
}
