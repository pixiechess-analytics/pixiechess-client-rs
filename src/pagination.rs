//! Auto-paginating stream helper used by every `*_iter()` resource
//! accessor.
//!
//! The helper short-circuits on the response envelope's `totalPages`
//! field when present (used by `/leaderboard`, `/points-leaderboard`,
//! `/pieces/...`, etc.); otherwise it stops when a page comes back
//! empty (the fallback used by `/user/match-history/...`).

use std::future::Future;
use std::pin::Pin;

use async_stream::try_stream;
use futures::Stream;

use crate::Result;

/// Build an auto-paginating [`Stream`] that yields items one by one
/// across pages.
///
/// `fetch` is called with the (1-indexed) page number and returns the
/// raw JSON for that page. `parse` extracts the entry list from the
/// page's payload. `total_pages_key`, when `Some`, names the envelope
/// field that reports the total page count — if present in the first
/// response, it becomes the loop's upper bound.
pub(crate) fn page_stream<T, F, Fut>(
    fetch: F,
    parse: fn(&serde_json::Value) -> Result<Vec<T>>,
    total_pages_key: Option<&'static str>,
) -> Pin<Box<dyn Stream<Item = Result<T>> + Send>>
where
    T: Send + 'static,
    F: Fn(u32) -> Fut + Send + 'static,
    Fut: Future<Output = Result<serde_json::Value>> + Send + 'static,
{
    Box::pin(try_stream! {
        let mut page = 1u32;
        let mut total: Option<u32> = None;
        loop {
            let data = fetch(page).await?;
            if let (Some(key), None) = (total_pages_key, total) {
                total = data
                    .get(key)
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|x| u32::try_from(x).ok());
            }
            let items = parse(&data)?;
            if items.is_empty() {
                break;
            }
            for item in items {
                yield item;
            }
            page += 1;
            if let Some(t) = total {
                if page > t {
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    use futures::StreamExt;
    use serde_json::json;

    use super::*;
    use crate::Error;

    fn parse_items(v: &serde_json::Value) -> Result<Vec<i64>> {
        let arr = v
            .get("items")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default();
        arr.into_iter()
            .map(|x| {
                x.as_i64().ok_or_else(|| Error::Api {
                    status: 0,
                    message: "bad item".into(),
                })
            })
            .collect()
    }

    #[tokio::test]
    async fn empty_first_page_yields_nothing() {
        let stream = page_stream(
            |_page| async { Ok(json!({"items": []})) },
            parse_items,
            None,
        );
        let collected: Vec<Result<i64>> = stream.collect().await;
        assert!(collected.is_empty());
    }

    #[tokio::test]
    async fn single_page_yields_all_items_then_stops() {
        let stream = page_stream(
            |page| async move {
                if page == 1 {
                    Ok(json!({"items": [1, 2, 3]}))
                } else {
                    Ok(json!({"items": []}))
                }
            },
            parse_items,
            None,
        );
        let got: Vec<i64> = stream.map(Result::unwrap).collect().await;
        assert_eq!(got, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn stops_on_empty_page_when_no_total_key() {
        let stream = page_stream(
            |page| async move {
                match page {
                    1 => Ok(json!({"items": [10, 20]})),
                    2 => Ok(json!({"items": [30]})),
                    _ => Ok(json!({"items": []})),
                }
            },
            parse_items,
            None,
        );
        let got: Vec<i64> = stream.map(Result::unwrap).collect().await;
        assert_eq!(got, vec![10, 20, 30]);
    }

    #[tokio::test]
    async fn stops_on_total_pages_without_extra_fetch() {
        let fetch_count = Arc::new(AtomicU32::new(0));
        let counter = fetch_count.clone();
        let stream = page_stream(
            move |page| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    match page {
                        1 => Ok(json!({"items": [1, 2], "totalPages": 2})),
                        2 => Ok(json!({"items": [3, 4], "totalPages": 2})),
                        _ => Ok(json!({"items": []})),
                    }
                }
            },
            parse_items,
            Some("totalPages"),
        );
        let got: Vec<i64> = stream.map(Result::unwrap).collect().await;
        assert_eq!(got, vec![1, 2, 3, 4]);
        // Exactly two fetches — no probe to page 3.
        assert_eq!(fetch_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn parse_error_propagates_as_err() {
        let stream = page_stream(
            |_page| async { Ok(json!({"items": ["not a number"]})) },
            parse_items,
            None,
        );
        let collected: Vec<Result<i64>> = stream.collect().await;
        assert!(matches!(collected.first(), Some(Err(_))));
    }
}
