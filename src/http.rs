//! Crate-private HTTP layer.
//!
//! Every resource builder ends up calling into [`HttpClient`] for its
//! actual network I/O. The public API consumers see is the resource
//! builders (`client.leaderboard().get().send().await`), not this struct.

// Methods here are exercised by the `#[cfg(test)]` mod below and by the
// `tokio`-based test runs, but the lib-only build doesn't see those uses
// until the next branch wires up `client.rs`. Drop this attribute as soon
// as `PixieChessClient` starts consuming `HttpClient`.
#![allow(dead_code)]

use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::{Error, Result};

/// Default base URL of the `PixieChess` API.
pub(crate) const DEFAULT_BASE_URL: &str = "https://api.pixiechess.xyz";

/// Default `User-Agent` sent on every outbound request.
///
/// The upstream WAF returns a `202` empty-body challenge to non-browser-shaped
/// user agents, so the default mirrors a recent Chrome/Linux build. Callers
/// who want to identify their own integration can override this via
/// [`PixieChessClientBuilder::user_agent`](crate::PixieChessClientBuilder::user_agent)
/// — but doing so on a UA the WAF doesn't accept will start returning empty
/// bodies. The safe pattern for identification is to suffix this constant:
///
/// ```no_run
/// use pixiechess_client::{DEFAULT_USER_AGENT, PixieChessClient};
///
/// let client = PixieChessClient::builder()
///     .user_agent(format!("{DEFAULT_USER_AGENT} my-app/1.0"))
///     .build()
///     .unwrap();
/// ```
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// HTTP client wrapping a [`reqwest::Client`] and a base URL.
///
/// Browser-mimicking headers are baked in at construction because the API
/// rejects requests without same-site `Origin` / `Referer` — see the
/// `default_headers()` helper for the full set.
#[derive(Debug, Clone)]
pub(crate) struct HttpClient {
    inner: reqwest::Client,
    base_url: Url,
}

impl HttpClient {
    /// Construct a client pointing at `base_url`, with the default
    /// browser-mimicking headers attached.
    ///
    /// `user_agent` overrides [`DEFAULT_USER_AGENT`] when provided.
    pub(crate) fn new(base_url: &str, user_agent: Option<&str>) -> Result<Self> {
        let inner = reqwest::Client::builder()
            .default_headers(default_headers(user_agent.unwrap_or(DEFAULT_USER_AGENT))?)
            .build()?;
        let base_url = Url::parse(base_url).map_err(|e| Error::Api {
            status: 0,
            message: e.to_string(),
        })?;
        Ok(Self { inner, base_url })
    }

    /// Typed GET — fetch `path` and deserialize the body into `T`.
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.get_with_params(path, &[]).await
    }

    /// Typed GET with query parameters.
    pub(crate) async fn get_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        let url = self.build_url(path)?;
        let resp = self.inner.get(url).query(params).send().await?;
        handle_response(resp).await
    }

    /// GET returning the raw JSON [`serde_json::Value`] — useful for callers
    /// that want to bypass typed deserialization.
    pub(crate) async fn get_json(&self, path: &str) -> Result<serde_json::Value> {
        self.get_with_params(path, &[]).await
    }

    fn build_url(&self, path: &str) -> Result<Url> {
        // `Url::join` treats a leading slash as the path replacement; that's
        // what we want for `/leaderboard` etc.
        self.base_url.join(path).map_err(|e| Error::Api {
            status: 0,
            message: e.to_string(),
        })
    }
}

/// Map an HTTP response to a typed result.
///
/// `404` → [`Error::NotFound`]; other `4xx`/`5xx` → [`Error::Api`]; `2xx`
/// bodies are deserialized into `T`.
async fn handle_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    if status.as_u16() == 404 {
        return Err(Error::NotFound(resp.text().await.unwrap_or_default()));
    }
    if !status.is_success() {
        return Err(Error::Api {
            status: status.as_u16(),
            message: resp.text().await.unwrap_or_default(),
        });
    }
    let bytes = resp.bytes().await?;
    let value = serde_json::from_slice(&bytes)?;
    Ok(value)
}

/// Default request headers attached to every outbound request.
///
/// The API rejects requests without same-site `Origin` / `Referer`; the
/// other headers mirror what a real browser sends to keep us out of
/// edge-case branches in the upstream WAF.
fn default_headers(user_agent: &str) -> Result<HeaderMap> {
    let mut h = HeaderMap::new();
    h.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://www.pixiechess.xyz"),
    );
    h.insert(
        header::REFERER,
        HeaderValue::from_static("https://www.pixiechess.xyz/"),
    );
    let ua = HeaderValue::from_str(user_agent).map_err(|e| Error::Api {
        status: 0,
        message: format!("invalid User-Agent: {e}"),
    })?;
    h.insert(header::USER_AGENT, ua);
    h.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    h.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.8"),
    );
    h.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    h.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    h.insert("sec-fetch-site", HeaderValue::from_static("same-site"));
    Ok(h)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    #[derive(Deserialize, PartialEq, Debug)]
    struct TestUser {
        id: u64,
        name: String,
    }

    #[tokio::test]
    async fn get_typed_deserializes_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/users/42"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": 42, "name": "alice"})),
            )
            .mount(&server)
            .await;

        let client = HttpClient::new(&server.uri(), None).unwrap();
        let user: TestUser = client.get("/users/42").await.unwrap();
        assert_eq!(
            user,
            TestUser {
                id: 42,
                name: "alice".into()
            }
        );
    }

    #[tokio::test]
    async fn get_with_params_attaches_query_string() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/leaderboard"))
            .and(query_param("page", "2"))
            .and(query_param("pageSize", "25"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;

        let client = HttpClient::new(&server.uri(), None).unwrap();
        let v: serde_json::Value = client
            .get_with_params(
                "/leaderboard",
                &[("page", "2".into()), ("pageSize", "25".into())],
            )
            .await
            .unwrap();
        assert_eq!(v, serde_json::json!({"ok": true}));
    }

    #[tokio::test]
    async fn get_json_returns_raw_value() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/raw"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"a": 1, "b": [2, 3]})),
            )
            .mount(&server)
            .await;

        let client = HttpClient::new(&server.uri(), None).unwrap();
        let v = client.get_json("/raw").await.unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"][0], 2);
    }

    #[tokio::test]
    async fn get_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not here"))
            .mount(&server)
            .await;

        let client = HttpClient::new(&server.uri(), None).unwrap();
        let res: Result<serde_json::Value> = client.get("/missing").await;
        match res {
            Err(Error::NotFound(body)) => assert!(body.contains("not here")),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_500_maps_to_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/broken"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let client = HttpClient::new(&server.uri(), None).unwrap();
        let res: Result<serde_json::Value> = client.get("/broken").await;
        match res {
            Err(Error::Api { status, message }) => {
                assert_eq!(status, 500);
                assert!(message.contains("boom"));
            }
            other => panic!("expected Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn non_json_body_maps_to_decode_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/garbage"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json {{{"))
            .mount(&server)
            .await;

        let client = HttpClient::new(&server.uri(), None).unwrap();
        let res: Result<serde_json::Value> = client.get("/garbage").await;
        assert!(matches!(res, Err(Error::Decode(_))));
    }

    #[tokio::test]
    async fn default_user_agent_reaches_the_wire() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ua"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        let client = HttpClient::new(&server.uri(), None).unwrap();
        let _: serde_json::Value = client.get("/ua").await.unwrap();
        let reqs = server.received_requests().await.unwrap();
        let ua = reqs[0]
            .headers
            .get("user-agent")
            .expect("user-agent should be present")
            .to_str()
            .unwrap();
        assert_eq!(ua, DEFAULT_USER_AGENT);
    }

    #[tokio::test]
    async fn custom_user_agent_overrides_default() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ua"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        let client = HttpClient::new(&server.uri(), Some("my-app/1.0")).unwrap();
        let _: serde_json::Value = client.get("/ua").await.unwrap();
        let reqs = server.received_requests().await.unwrap();
        let ua = reqs[0]
            .headers
            .get("user-agent")
            .expect("user-agent should be present")
            .to_str()
            .unwrap();
        assert_eq!(ua, "my-app/1.0");
    }

    #[tokio::test]
    async fn invalid_user_agent_returns_api_error() {
        // A newline isn't a valid header value.
        let res = HttpClient::new("http://localhost:9999", Some("bad\nua"));
        assert!(matches!(res, Err(Error::Api { .. })));
    }
}
