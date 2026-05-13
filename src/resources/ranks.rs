//! Ranks endpoint group (`GET /ranks/masters`).

use serde::Deserialize;

use crate::Result;
use crate::http::HttpClient;

#[derive(Deserialize)]
struct MastersEnvelope {
    #[serde(default)]
    addresses: Vec<String>,
}

/// Accessor for ranks endpoints. Obtained from
/// [`PixieChessClient::ranks`](crate::PixieChessClient::ranks).
pub struct RanksResource<'c> {
    http: &'c HttpClient,
}

impl<'c> RanksResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// `GET /ranks/masters` (unwraps `{"addresses": …}` to a `Vec<String>`).
    #[must_use]
    pub fn masters(&self) -> MastersBuilder<'c> {
        MastersBuilder { http: self.http }
    }
}

/// Builder for [`RanksResource::masters`].
pub struct MastersBuilder<'c> {
    http: &'c HttpClient,
}

impl MastersBuilder<'_> {
    /// Fetch the master-rank addresses.
    ///
    /// # Errors
    /// HTTP or deserialization failure.
    pub async fn send(self) -> Result<Vec<String>> {
        let env: MastersEnvelope = self.http.get("/ranks/masters").await?;
        Ok(env.addresses)
    }

    /// Fetch as raw JSON (preserves the `{"addresses": …}` envelope).
    ///
    /// # Errors
    /// HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json("/ranks/masters").await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    #[tokio::test]
    async fn masters_unwraps_addresses_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ranks/masters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "addresses": ["0xa", "0xb", "0xc"],
            })))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let m = client.ranks().masters().send().await.unwrap();
        assert_eq!(
            m,
            vec!["0xa".to_string(), "0xb".to_string(), "0xc".to_string()]
        );
    }

    #[tokio::test]
    async fn masters_handles_missing_field_as_empty() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ranks/masters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(&server)
            .await;
        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let m = client.ranks().masters().send().await.unwrap();
        assert!(m.is_empty());
    }
}
