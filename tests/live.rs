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

// ---------------------------------------------------------------------
// Effectiveness tests — verify every advertised query param actually
// changes the response in the expected way.
// ---------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_leaderboard_page_changes_results() {
    let c = client();
    let p1 = c
        .leaderboard()
        .get()
        .page(1)
        .send()
        .await
        .expect("leaderboard page 1");
    let p2 = c
        .leaderboard()
        .get()
        .page(2)
        .send()
        .await
        .expect("leaderboard page 2");
    let r1 = p1.entries.first().expect("page 1 non-empty").rank;
    let r2 = p2.entries.first().expect("page 2 non-empty").rank;
    assert_eq!(r1, 1, "page 1 should start at rank 1");
    assert!(
        r2 > r1,
        "page 2 first rank ({r2}) should be greater than page 1's ({r1})"
    );
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_match_history_limit_actually_limits() {
    let c = client();
    let lb = c
        .leaderboard()
        .get()
        .page(1)
        .send()
        .await
        .expect("leaderboard");
    let addr = &lb.entries.first().expect("at least one entry").address;
    let h = c
        .users()
        .match_history(addr)
        .limit(3)
        .send()
        .await
        .expect("match-history");
    assert!(
        h.matches.len() <= 3,
        "limit=3 returned {} matches",
        h.matches.len()
    );
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_pieces_grouped_collapses_duplicates() {
    let c = client();
    // Pick an address with several pieces — top points-leaderboard entry
    // tends to have a deep collection.
    let pts = c
        .leaderboard()
        .points()
        .page(1)
        .send()
        .await
        .expect("points-leaderboard");
    let addr = &pts.entries.first().expect("at least one entry").address;
    let ungrouped = c
        .pieces()
        .get(addr)
        .grouped(false)
        .send()
        .await
        .expect("pieces ungrouped");
    let grouped = c
        .pieces()
        .get(addr)
        .grouped(true)
        .send()
        .await
        .expect("pieces grouped");
    assert!(
        grouped.pieces.len() <= ungrouped.pieces.len(),
        "grouped should collapse: {} vs {}",
        grouped.pieces.len(),
        ungrouped.pieces.len()
    );
    if let Some(p) = grouped.pieces.first() {
        assert!(
            p.count.is_some(),
            "grouped rows should carry a `count` field"
        );
    }
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_daily_volume_range_affects_count() {
    let c = client();
    let d7 = c
        .auctions()
        .daily_volume()
        .range("7d")
        .send()
        .await
        .expect("daily-volume 7d");
    let d30 = c
        .auctions()
        .daily_volume()
        .range("30d")
        .send()
        .await
        .expect("daily-volume 30d");
    assert!(
        d30.len() > d7.len(),
        "range=30d ({}) should return more rows than range=7d ({})",
        d30.len(),
        d7.len()
    );
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_auctions_past_page_size_actually_paginates() {
    let c = client();
    let small = c
        .auctions()
        .past()
        .page(1)
        .page_size(1)
        .send()
        .await
        .expect("auctions/past pageSize=1");
    let large = c
        .auctions()
        .past()
        .page(1)
        .page_size(10)
        .send()
        .await
        .expect("auctions/past pageSize=10");
    assert_eq!(small.day_buckets.len(), 1);
    assert_eq!(large.day_buckets.len(), 10);
    assert_eq!(small.page_size, 1);
    assert_eq!(large.page_size, 10);
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_live_feed_limit_actually_limits() {
    let events = client()
        .misc()
        .live_feed()
        .limit(3)
        .send()
        .await
        .expect("live-feed");
    assert_eq!(events.len(), 3);
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_tournaments_pinned_filters() {
    let c = client();
    let all = c
        .tournaments()
        .list()
        .limit(10)
        .pinned(false)
        .send()
        .await
        .expect("tournament/list (all)");
    let pinned = c
        .tournaments()
        .list()
        .limit(10)
        .pinned(true)
        .send()
        .await
        .expect("tournament/list (pinned)");
    assert!(
        pinned.total_count <= all.total_count,
        "pinned subset ({}) should fit within full set ({})",
        pinned.total_count,
        all.total_count
    );
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_tournaments_active_filters() {
    let c = client();
    let inactive = c
        .tournaments()
        .list()
        .limit(10)
        .active(false)
        .send()
        .await
        .expect("tournament/list (active=false)");
    let active = c
        .tournaments()
        .list()
        .limit(10)
        .active(true)
        .send()
        .await
        .expect("tournament/list (active=true)");
    assert!(
        active.total_count < inactive.total_count,
        "active subset ({}) should be a strict subset of inactive set ({})",
        active.total_count,
        inactive.total_count
    );
}

#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_tournaments_date_filter_today() {
    let c = client();
    let baseline = c
        .tournaments()
        .list()
        .limit(10)
        .send()
        .await
        .expect("tournament/list");
    let today = c
        .tournaments()
        .list()
        .limit(10)
        .date_filter("today")
        .send()
        .await
        .expect("tournament/list (dateFilter=today)");
    assert!(
        today.total_count <= baseline.total_count,
        "today subset ({}) should fit within full set ({})",
        today.total_count,
        baseline.total_count
    );
}

// Tier-3 smoke: tz_offset only meaningfully interacts with date_filter,
// so we can't construct a clean discriminator from the available data.
// Assert the parameterized call succeeds + decodes; no count assertion.
#[tokio::test]
#[ignore = "hits api.pixiechess.xyz; run with --ignored"]
async fn live_tournaments_tz_offset_smoke() {
    let _ = client()
        .tournaments()
        .list()
        .limit(5)
        .date_filter("today")
        .tz_offset(-300)
        .send()
        .await
        .expect("tournament/list (tzOffset)");
}
