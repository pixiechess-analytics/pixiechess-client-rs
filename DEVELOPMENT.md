# Development notes

Internal notes on how the crate is built and how the API surface is kept in sync with the upstream `api.pixiechess.xyz`. Consumers of the published client don't need any of this; it's here for contributors and for the next person who picks up the repo cold.

## How the API surface was discovered

The upstream has no published OpenAPI / schema. The endpoint list, payload shapes, and parameter behavior all come from observation:

1. **HAR captures** from a logged-in browser session (see `pixiechess-har-utils`) produced a corpus of real request / response pairs across most endpoints.
2. **A live sampler** (`examples/sample_live.rs`) drives hundreds of throttled requests across users, pieces, tournaments, games, auctions, leaderboards, and pages, dumping bodies into a JSON corpus matching `tests/fixtures/pixiechess-api.json`.
3. **An audit script** (`tools/audit_corpus.py`) walks the corpus and prints per-field stats per endpoint (presence %, null %, observed value-types) so every required / optional decision is grounded in evidence rather than assumption.

The vendored `tests/fixtures/pixiechess-api.json` is a deduped, capped subset of a recent sample. To refresh:

```
cargo run --example sample_live --release -- --out /tmp/pixiechess-api-expanded.json
python3 tools/audit_corpus.py
```

Then update the fixture and the model file(s) the audit surfaces.

## How no-op query params are handled

Some query parameters are accepted by the server but silently ignored — they don't change the response. The audit catches these by varying one param at a time and comparing responses. Confirmed examples:

| Endpoint | Param | Verdict |
|---|---|---|
| `/leaderboard` | `pageSize` | ignored (server pins page size at 15) |
| `/tournament/list` | `sort` | ignored (identical ordering across `date`, `newest`, `oldest`) |
| `/live-feed` | `since` | ignored (returns events older than the cutoff) |
| `/live-feed` | `type` | ignored (returns all event types) |

**Policy**: confirmed silently-ignored params are *dropped from the public builder* — not deprecated, not documented as "no-op". Advertising a knob that doesn't turn is misleading. If the server later starts honoring one, the method is added back as a non-breaking minor bump.

Every honored param has an effectiveness test in `tests/live.rs` that calls the endpoint twice (varied param) and asserts the responses differ in the expected direction. This catches future silent-drift.

## Regression tests

- `tests/replay.rs` — for every endpoint in the corpus, deserialize every captured response body into the typed model. Fails loud on shape drift.
- `tests/live.rs` — `#[tokio::test] #[ignore]` smoke + effectiveness tests against the real API. Run with `cargo test --test live -- --ignored`.

## When the server changes shape

1. Re-run the sampler to refresh `tests/fixtures/pixiechess-api.json`.
2. Run `cargo test`. The replay test will fail loud on any shape drift.
3. Run `python3 tools/audit_corpus.py` to see what changed.
4. Update the relevant model file (`src/models/<thing>.rs`).
5. Re-run `cargo test --test live -- --ignored` to confirm the change works against the live API.
