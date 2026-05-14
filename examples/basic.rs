//! Quickstart example for `pixiechess-client`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example basic
//! ```
//!
//! Demonstrates:
//! - Constructing the default client
//! - The uniform `.send()` (typed) / `.raw()` (`serde_json::Value`) terminals
//! - Fetching the top leaderboard page, then drilling into a single user

use pixiechess_client::PixieChessClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PixieChessClient::new()?;

    let lb = client.leaderboard().get().page(1).send().await?;

    println!(
        "Leaderboard page 1 ({} entries; {} total pages). Top 5:",
        lb.entries.len(),
        lb.total_pages
    );
    for entry in lb.entries.iter().take(5) {
        println!(
            "  #{:>3}  {:<20}  {:>7.1}  W:{:<4} S:{:<3}",
            entry.rank, entry.username_display, entry.rating, entry.wins, entry.streak
        );
    }

    let Some(top) = lb.entries.first() else {
        println!("(no entries — nothing else to show)");
        return Ok(());
    };

    // Typed: parse the response into the User model.
    let user = client.users().get(&top.address).send().await?;
    println!(
        "\nTop player profile (typed):\n  {} — rating {:.1}, {} match(es), trophies {}",
        user.username_display.as_deref().unwrap_or("<no username>"),
        user.rating,
        user.match_count,
        user.trophies,
    );

    // Raw: bypass the typed model and pull out one nested field.
    let raw = client.users().get(&top.address).raw().await?;
    if let Some(helmet) = raw.pointer("/user/helmet") {
        println!("  helmet (raw JSON): {helmet}");
    }

    let history = client
        .users()
        .match_history(&top.address)
        .page(1)
        .limit(3)
        .send()
        .await?;
    println!("\nRecent matches ({}):", history.matches.len());
    for m in &history.matches {
        println!(
            "  {}  {} vs {}  →  {} ({})",
            m.created_at.format("%Y-%m-%d"),
            m.white.username_display.as_deref().unwrap_or("<ghost>"),
            m.black.username_display.as_deref().unwrap_or("<ghost>"),
            m.outcome,
            m.result_for_user,
        );
    }

    Ok(())
}
