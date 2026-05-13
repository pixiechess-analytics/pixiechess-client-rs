//! Pieces endpoints: `GET /pieces/{address}` and
//! `GET /burned-pieces/{address}`, each with a single-page builder and
//! an auto-paginating iter.

use std::pin::Pin;

use futures::Stream;

use crate::Result;
use crate::http::HttpClient;
use crate::models::pieces::{Piece, PiecesPage};
use crate::pagination::page_stream;

/// Accessor for piece-related endpoints. Obtained from
/// [`PixieChessClient::pieces`](crate::PixieChessClient::pieces).
pub struct PiecesResource<'c> {
    http: &'c HttpClient,
}

impl<'c> PiecesResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// Fetch one page of pieces owned by `address`.
    #[must_use]
    pub fn get(&self, address: impl Into<String>) -> PiecesGetBuilder<'c> {
        PiecesGetBuilder {
            http: self.http,
            address: address.into(),
            page: 1,
            limit: None,
            grouped: false,
        }
    }

    /// Iterate every piece owned by `address`, fetching pages on demand.
    // Mirrors the Python client's `iter()`. Not a `std::iter::Iterator`.
    #[must_use]
    #[allow(clippy::should_implement_trait, clippy::iter_not_returning_iterator)]
    pub fn iter(&self, address: impl Into<String>) -> PiecesIterBuilder<'c> {
        PiecesIterBuilder {
            http: self.http,
            address: address.into(),
            limit: None,
            grouped: false,
        }
    }

    /// Fetch one page of burned pieces tied to `address`.
    #[must_use]
    pub fn burned(&self, address: impl Into<String>) -> BurnedGetBuilder<'c> {
        BurnedGetBuilder {
            http: self.http,
            address: address.into(),
            page: 1,
            limit: None,
        }
    }

    /// Iterate every burned piece tied to `address`, fetching pages on
    /// demand.
    #[must_use]
    pub fn burned_iter(&self, address: impl Into<String>) -> BurnedIterBuilder<'c> {
        BurnedIterBuilder {
            http: self.http,
            address: address.into(),
            limit: None,
        }
    }
}

fn pieces_params(page: u32, limit: Option<u32>, grouped: bool) -> Vec<(&'static str, String)> {
    let mut params = vec![("page", page.to_string())];
    if let Some(l) = limit {
        params.push(("limit", l.to_string()));
    }
    if grouped {
        params.push(("grouped", "true".to_string()));
    }
    params
}

fn burned_params(page: u32, limit: Option<u32>) -> Vec<(&'static str, String)> {
    let mut params = vec![("page", page.to_string())];
    if let Some(l) = limit {
        params.push(("limit", l.to_string()));
    }
    params
}

// `GET /pieces/{address}` ----------------------------------------------

/// Builder for [`PiecesResource::get`].
pub struct PiecesGetBuilder<'c> {
    http: &'c HttpClient,
    address: String,
    page: u32,
    limit: Option<u32>,
    grouped: bool,
}

impl PiecesGetBuilder<'_> {
    /// Set the 1-indexed page. Default: `1`.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Set the per-page limit. Omitted from the query when unset.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// When `true`, ask the server to group pieces by metadata
    /// (collapses duplicates of the same piece into a single row with
    /// `count`).
    #[must_use]
    pub fn grouped(mut self, grouped: bool) -> Self {
        self.grouped = grouped;
        self
    }

    fn path(&self) -> String {
        format!("/pieces/{}", self.address)
    }

    /// Fetch the page.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or deserialization
    /// failure.
    pub async fn send(self) -> Result<PiecesPage> {
        let path = self.path();
        let params = pieces_params(self.page, self.limit, self.grouped);
        self.http.get_with_params(&path, &params).await
    }

    /// Fetch the page as raw JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let path = self.path();
        let params = pieces_params(self.page, self.limit, self.grouped);
        self.http.get_with_params(&path, &params).await
    }
}

/// Builder for [`PiecesResource::iter`].
pub struct PiecesIterBuilder<'c> {
    http: &'c HttpClient,
    address: String,
    limit: Option<u32>,
    grouped: bool,
}

impl PiecesIterBuilder<'_> {
    /// Set the per-page batch size used while iterating.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// See [`PiecesGetBuilder::grouped`].
    #[must_use]
    pub fn grouped(mut self, grouped: bool) -> Self {
        self.grouped = grouped;
        self
    }

    /// Begin streaming pieces. Short-circuits on `totalPages`.
    #[must_use]
    pub fn send(self) -> Pin<Box<dyn Stream<Item = Result<Piece>> + Send>> {
        let http = self.http.clone();
        let address = self.address;
        let limit = self.limit;
        let grouped = self.grouped;
        page_stream(
            move |page| {
                let http = http.clone();
                let path = format!("/pieces/{address}");
                let params = pieces_params(page, limit, grouped);
                async move { http.get_with_params(&path, &params).await }
            },
            parse_pieces,
            Some("totalPages"),
        )
    }
}

// `GET /burned-pieces/{address}` ---------------------------------------

/// Builder for [`PiecesResource::burned`].
pub struct BurnedGetBuilder<'c> {
    http: &'c HttpClient,
    address: String,
    page: u32,
    limit: Option<u32>,
}

impl BurnedGetBuilder<'_> {
    /// Set the 1-indexed page. Default: `1`.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Set the per-page limit. Omitted from the query when unset.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    fn path(&self) -> String {
        format!("/burned-pieces/{}", self.address)
    }

    /// Fetch the page.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or deserialization
    /// failure.
    pub async fn send(self) -> Result<PiecesPage> {
        let path = self.path();
        let params = burned_params(self.page, self.limit);
        self.http.get_with_params(&path, &params).await
    }

    /// Fetch the page as raw JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        let path = self.path();
        let params = burned_params(self.page, self.limit);
        self.http.get_with_params(&path, &params).await
    }
}

/// Builder for [`PiecesResource::burned_iter`].
pub struct BurnedIterBuilder<'c> {
    http: &'c HttpClient,
    address: String,
    limit: Option<u32>,
}

impl BurnedIterBuilder<'_> {
    /// Set the per-page batch size used while iterating.
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Begin streaming burned pieces. Short-circuits on `totalPages`.
    #[must_use]
    pub fn send(self) -> Pin<Box<dyn Stream<Item = Result<Piece>> + Send>> {
        let http = self.http.clone();
        let address = self.address;
        let limit = self.limit;
        page_stream(
            move |page| {
                let http = http.clone();
                let path = format!("/burned-pieces/{address}");
                let params = burned_params(page, limit);
                async move { http.get_with_params(&path, &params).await }
            },
            parse_pieces,
            Some("totalPages"),
        )
    }
}

fn parse_pieces(data: &serde_json::Value) -> Result<Vec<Piece>> {
    let pieces = data
        .get("pieces")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));
    let parsed: Vec<Piece> = serde_json::from_value(pieces)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    fn piece(id: &str) -> serde_json::Value {
        json!({
            "_id": id,
            "collectionAddress": "0xc0ll",
            "tokenId": 1,
            "owner": "0xowner",
            "metadata": {"attributes": []},
            "lastTransferBlockNumber": 1,
            "createdAt": "2026-05-13T12:00:00Z",
            "updatedAt": "2026-05-13T12:00:00Z",
        })
    }

    #[allow(dead_code)] // referenced by a future burned-iter test
    fn burned_piece(id: &str) -> serde_json::Value {
        json!({
            "_id": id,
            "collectionAddress": "0xc0ll",
            "tokenId": 1,
            "owner": "0xowner",
            "metadata": {"attributes": []},
            "lastTransferBlockNumber": 1,
            "createdAt": "2026-05-13T12:00:00Z",
            "updatedAt": "2026-05-13T12:00:00Z",
            "originalAssetId": "orig-1",
            "burned": {
                "time": 1,
                "tournament": {"name": "Daily", "color": "white"},
            },
        })
    }

    #[tokio::test]
    async fn pieces_get_attaches_page_limit_grouped() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pieces/0xabc"))
            .and(query_param("page", "2"))
            .and(query_param("limit", "10"))
            .and(query_param("grouped", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pieces": [piece("p1")],
                "totalPages": 5,
                "currentPage": 2,
                "totalCount": 50,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let page = client
            .pieces()
            .get("0xabc")
            .page(2)
            .limit(10)
            .grouped(true)
            .send()
            .await
            .unwrap();
        assert_eq!(page.pieces.len(), 1);
        assert_eq!(page.total_pages, 5);
    }

    #[tokio::test]
    async fn pieces_iter_walks_pages_with_total() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pieces/0xabc"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pieces": [piece("p1"), piece("p2")],
                "totalPages": 2,
                "currentPage": 1,
                "totalCount": 3,
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/pieces/0xabc"))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pieces": [piece("p3")],
                "totalPages": 2,
                "currentPage": 2,
                "totalCount": 3,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let collected: Vec<_> = client.pieces().iter("0xabc").send().collect().await;
        let ids: Vec<String> = collected.into_iter().map(|r| r.unwrap().id).collect();
        assert_eq!(ids, vec!["p1", "p2", "p3"]);
    }

    #[tokio::test]
    async fn burned_get_uses_burned_pieces_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/burned-pieces/0xabc"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pieces": [],
                "totalPages": 0,
                "currentPage": 1,
                "totalCount": 0,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let page = client.pieces().burned("0xabc").send().await.unwrap();
        assert!(page.pieces.is_empty());
    }

    #[tokio::test]
    async fn burned_iter_single_page_stops() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/burned-pieces/0xabc"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pieces": [piece("burned1")],
                "totalPages": 1,
                "currentPage": 1,
                "totalCount": 1,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let collected: Vec<_> = client.pieces().burned_iter("0xabc").send().collect().await;
        assert_eq!(collected.len(), 1);
    }
}
