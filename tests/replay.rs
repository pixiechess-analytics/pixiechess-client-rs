//! Replay test against a vendored corpus of real responses.
//!
//! Iterates every example in `tests/fixtures/pixiechess-api.json` and
//! asserts that the captured response body deserializes into the typed
//! model. Fails loud on API shape drift.
//!
//! The corpus is produced by `pixiechess-har-utils`. Each top-level key
//! is `"<METHOD> <path-template>"` and the value carries an `examples`
//! list of `{request, response}` entries. This test owns its endpoint-key
//! → model dispatch table; the corpus has no opinion about type names.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use pixiechess_client::models::{
    Auction, AuctionDaySummary, AuctionPieceInfo, CompletedDaySummary, DailyVolume, EthUsdPrice,
    Game, LeaderboardPage, LiveFeedEvent, MatchHistoryPage, PastAuctionsPage, PiecesPage,
    PointsLeaderboardPage, Prices, PublicConfig, RatingChange, TournamentDetails, TournamentList,
    User, WaitlistEntry,
};

type ParseFn = fn(&Value) -> Result<(), serde_json::Error>;

fn parse<T: for<'de> Deserialize<'de>>(body: &Value) -> Result<(), serde_json::Error> {
    let _: T = serde_json::from_value(body.clone())?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct AuctionEnvelope {
    auction: Auction,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct AuctionsListEnvelope {
    #[serde(default)]
    auctions: Vec<Auction>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DaysEnvelope {
    #[serde(default)]
    days: Vec<DailyVolume>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct UserEnvelope {
    user: User,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct VaultBalanceEnvelope {
    balance: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct RanksMastersEnvelope {
    #[serde(default)]
    addresses: Vec<String>,
}

fn parse_auction_envelope(body: &Value) -> Result<(), serde_json::Error> {
    parse::<AuctionEnvelope>(body)
}

fn parse_auctions_list_envelope(body: &Value) -> Result<(), serde_json::Error> {
    parse::<AuctionsListEnvelope>(body)
}

fn parse_days_envelope(body: &Value) -> Result<(), serde_json::Error> {
    parse::<DaysEnvelope>(body)
}

fn parse_user_envelope(body: &Value) -> Result<(), serde_json::Error> {
    parse::<UserEnvelope>(body)
}

fn parse_game_envelope(body: &Value) -> Result<(), serde_json::Error> {
    // /game/{gameId} arrives as a bare Game in our model (see games resource).
    parse::<Game>(body)
}

fn parse_vault_balance(body: &Value) -> Result<(), serde_json::Error> {
    parse::<VaultBalanceEnvelope>(body)
}

fn parse_ranks_masters(body: &Value) -> Result<(), serde_json::Error> {
    parse::<RanksMastersEnvelope>(body)
}

fn dispatch() -> HashMap<&'static str, ParseFn> {
    let mut m: HashMap<&'static str, ParseFn> = HashMap::new();

    m.insert("GET /auction/{address}", parse_auction_envelope);
    m.insert("GET /auctions/active", parse_auctions_list_envelope);
    m.insert("GET /auctions/daily-volume", parse_days_envelope);
    m.insert(
        "GET /auctions/last-completed-day-summary",
        parse::<CompletedDaySummary>,
    );
    m.insert("GET /auctions/past", parse::<PastAuctionsPage>);
    m.insert("GET /auctions/piece/{pieceKey}", parse::<AuctionPieceInfo>);
    m.insert(
        "GET /auctions/piece/{pieceKey}/daily-volume",
        parse_days_envelope,
    );
    m.insert("GET /auctions/today-summary", parse::<AuctionDaySummary>);

    m.insert("GET /burned-pieces/{address}", parse::<PiecesPage>);
    m.insert("GET /config/public", parse::<PublicConfig>);
    m.insert("GET /eth-usd-price", parse::<EthUsdPrice>);
    m.insert("GET /game/{gameId}", parse_game_envelope);
    m.insert("GET /game/{gameId}/rating/{address}", parse::<RatingChange>);
    m.insert("GET /leaderboard", parse::<LeaderboardPage>);
    m.insert("GET /live-feed", parse::<Vec<LiveFeedEvent>>);
    m.insert("GET /pieces/{address}", parse::<PiecesPage>);
    m.insert("GET /points-leaderboard", parse::<PointsLeaderboardPage>);
    m.insert("GET /prices", parse::<Prices>);
    m.insert("GET /ranks/masters", parse_ranks_masters);

    m.insert(
        "GET /tournament/details/{tournamentId}",
        parse::<TournamentDetails>,
    );
    m.insert("GET /tournament/list", parse::<TournamentList>);
    m.insert(
        "GET /tournament/waitlist/{tournamentId}",
        parse::<Vec<WaitlistEntry>>,
    );

    m.insert(
        "GET /user/match-history/{address}",
        parse::<MatchHistoryPage>,
    );
    m.insert("GET /user/{userId}", parse_user_envelope);
    m.insert("GET /vault-balance", parse_vault_balance);

    m
}

#[derive(Debug, Deserialize)]
struct Example {
    request: ExampleRequest,
    response: ExampleResponse,
}

#[derive(Debug, Deserialize)]
struct ExampleRequest {
    url: String,
}

#[derive(Debug, Deserialize)]
struct ExampleResponse {
    status: u16,
    body: Value,
}

#[derive(Debug, Deserialize)]
struct Group {
    examples: Vec<Example>,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pixiechess-api.json")
}

#[test]
fn every_example_deserializes_into_typed_model() {
    let path = fixture_path();
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let corpus: HashMap<String, Group> =
        serde_json::from_str(&raw).expect("fixture is well-formed JSON");

    let dispatch = dispatch();

    let mut failures: Vec<String> = Vec::new();
    let mut unknown_keys: Vec<String> = Vec::new();
    let mut examples_seen: usize = 0;

    for (endpoint_key, group) in &corpus {
        let Some(parser) = dispatch.get(endpoint_key.as_str()) else {
            unknown_keys.push(endpoint_key.clone());
            continue;
        };
        for (idx, ex) in group.examples.iter().enumerate() {
            examples_seen += 1;
            // Only assert deserialization on 2xx responses.
            if !(200..300).contains(&ex.response.status) {
                continue;
            }
            if let Err(e) = parser(&ex.response.body) {
                failures.push(format!(
                    "{endpoint_key} [example #{idx}, url={}]: {e}",
                    ex.request.url
                ));
            }
        }
    }

    // Conversely, flag dispatch entries the corpus doesn't cover so the
    // table doesn't silently drift away from what we ship.
    let mut uncovered: Vec<&str> = dispatch
        .keys()
        .copied()
        .filter(|k| !corpus.contains_key(*k))
        .collect();
    uncovered.sort_unstable();

    let mut messages = Vec::new();
    if !unknown_keys.is_empty() {
        unknown_keys.sort();
        messages.push(format!(
            "corpus has {} unmapped endpoint key(s):\n  - {}",
            unknown_keys.len(),
            unknown_keys.join("\n  - ")
        ));
    }
    if !failures.is_empty() {
        messages.push(format!(
            "{} example(s) failed to deserialize:\n  - {}",
            failures.len(),
            failures.join("\n  - ")
        ));
    }

    assert!(
        messages.is_empty(),
        "replay test found drift across {examples_seen} example(s):\n\n{}",
        messages.join("\n\n")
    );
    println!(
        "replay test exercised {examples_seen} examples across {} endpoint(s); {} dispatch entries uncovered by current corpus",
        corpus.len(),
        uncovered.len()
    );
}
