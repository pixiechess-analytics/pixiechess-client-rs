//! Live smoke tests against `api.pixiechess.xyz`.
//!
//! Every test is `#[ignore]` by default — running them hits the real API
//! and would be noisy under `cargo test`. Run on demand with:
//!
//! ```text
//! cargo test --test live -- --ignored
//! ```
//!
//! These are smoke tests, not exhaustive coverage. The intent is one
//! call per endpoint group so that during pre-release we can quickly
//! confirm the client still works against production. Per-shape
//! regression coverage lives in `tests/replay.rs`.

use pixiechess_client::PixieChessClient;

fn client() -> PixieChessClient {
    PixieChessClient::new().expect("default client constructs cleanly")
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_misc_config() {
    let _ = client().misc().config().send().await.expect("config");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_misc_eth_usd_price() {
    let p = client()
        .misc()
        .eth_usd_price()
        .send()
        .await
        .expect("eth-usd-price");
    assert!(p.usd > 0.0, "eth/usd should be positive, got {}", p.usd);
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_misc_vault_balance() {
    let b = client()
        .misc()
        .vault_balance()
        .send()
        .await
        .expect("vault-balance");
    assert!(!b.is_empty(), "vault balance string should not be empty");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_misc_live_feed() {
    let _ = client()
        .misc()
        .live_feed()
        .limit(5)
        .send()
        .await
        .expect("live-feed");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_ranks_masters() {
    let _ = client().ranks().masters().send().await.expect("masters");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_leaderboard_first_page() {
    let p = client()
        .leaderboard()
        .get()
        .page(1)
        .page_size(10)
        .send()
        .await
        .expect("leaderboard");
    assert!(
        !p.entries.is_empty(),
        "leaderboard page 1 should have entries"
    );
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_points_leaderboard_first_page() {
    let _ = client()
        .leaderboard()
        .points()
        .page(1)
        .send()
        .await
        .expect("points-leaderboard");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_users_get_top_leaderboard_entry() {
    let c = client();
    let lb = c
        .leaderboard()
        .get()
        .page(1)
        .page_size(1)
        .send()
        .await
        .expect("leaderboard");
    let entry = lb.entries.first().expect("at least one entry");
    let _ = c
        .users()
        .get(&entry.address)
        .send()
        .await
        .expect("user get");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_users_match_history_top_leaderboard_entry() {
    let c = client();
    let lb = c
        .leaderboard()
        .get()
        .page(1)
        .page_size(1)
        .send()
        .await
        .expect("leaderboard");
    let entry = lb.entries.first().expect("at least one entry");
    let _ = c
        .users()
        .match_history(&entry.address)
        .send()
        .await
        .expect("match-history");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_pieces_for_top_leaderboard_entry() {
    let c = client();
    let lb = c
        .leaderboard()
        .get()
        .page(1)
        .page_size(1)
        .send()
        .await
        .expect("leaderboard");
    let entry = lb.entries.first().expect("at least one entry");
    let _ = c.pieces().get(&entry.address).send().await.expect("pieces");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_active() {
    let _ = client()
        .auctions()
        .active()
        .send()
        .await
        .expect("auctions/active");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_past_first_page() {
    let _ = client()
        .auctions()
        .past()
        .page(1)
        .page_size(5)
        .send()
        .await
        .expect("auctions/past");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_today_summary() {
    let _ = client()
        .auctions()
        .today_summary()
        .send()
        .await
        .expect("auctions/today-summary");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_last_completed_day_summary() {
    let _ = client()
        .auctions()
        .last_completed_day_summary()
        .send()
        .await
        .expect("auctions/last-completed-day-summary");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_daily_volume() {
    let _ = client()
        .auctions()
        .daily_volume()
        .range("7d")
        .send()
        .await
        .expect("auctions/daily-volume");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_prices() {
    // `vrgda` may legitimately be empty between auction windows, so just
    // assert the call succeeds + decodes.
    let _ = client().auctions().prices().send().await.expect("prices");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_tournaments_list_first_page() {
    let _ = client()
        .tournaments()
        .list()
        .limit(5)
        .send()
        .await
        .expect("tournament/list");
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_game_lookup_from_top_match_history() {
    let c = client();
    let lb = c
        .leaderboard()
        .get()
        .page(1)
        .page_size(1)
        .send()
        .await
        .expect("leaderboard");
    let entry = lb.entries.first().expect("at least one entry");
    let history = c
        .users()
        .match_history(&entry.address)
        .send()
        .await
        .expect("match-history");
    if let Some(m) = history.matches.first() {
        let _ = c.games().get(&m.game_id).send().await.expect("game get");
    }
}
