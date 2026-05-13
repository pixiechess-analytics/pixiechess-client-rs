//! Game-related endpoints: `GET /game/{gameId}` and
//! `GET /game/{gameId}/rating/{address}`.

use crate::Result;
use crate::http::HttpClient;
use crate::models::game::{Game, RatingChange};

/// Accessor for game-related endpoints. Obtained from
/// [`PixieChessClient::games`](crate::PixieChessClient::games).
pub struct GamesResource<'c> {
    http: &'c HttpClient,
}

impl<'c> GamesResource<'c> {
    pub(crate) fn new(http: &'c HttpClient) -> Self {
        Self { http }
    }

    /// Fetch a single game by its id.
    pub fn get(&self, game_id: impl Into<String>) -> GamesGetBuilder<'c> {
        GamesGetBuilder {
            http: self.http,
            game_id: game_id.into(),
        }
    }

    /// Fetch the rating change a given player saw from a given game.
    pub fn rating_change(
        &self,
        game_id: impl Into<String>,
        address: impl Into<String>,
    ) -> RatingChangeBuilder<'c> {
        RatingChangeBuilder {
            http: self.http,
            game_id: game_id.into(),
            address: address.into(),
        }
    }
}

// `GET /game/{gameId}` -------------------------------------------------

/// Builder for [`GamesResource::get`]. Terminals: `.send()` (typed) and
/// `.raw()` (raw JSON).
pub struct GamesGetBuilder<'c> {
    http: &'c HttpClient,
    game_id: String,
}

impl GamesGetBuilder<'_> {
    fn path(&self) -> String {
        format!("/game/{}", self.game_id)
    }

    /// Fetch the game.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or deserialization
    /// failure.
    pub async fn send(self) -> Result<Game> {
        let path = self.path();
        self.http.get(&path).await
    }

    /// Fetch the game as raw JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

// `GET /game/{gameId}/rating/{address}` --------------------------------

/// Builder for [`GamesResource::rating_change`]. Terminals: `.send()`
/// and `.raw()`.
pub struct RatingChangeBuilder<'c> {
    http: &'c HttpClient,
    game_id: String,
    address: String,
}

impl RatingChangeBuilder<'_> {
    fn path(&self) -> String {
        format!("/game/{}/rating/{}", self.game_id, self.address)
    }

    /// Fetch the rating change.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure or deserialization
    /// failure.
    pub async fn send(self) -> Result<RatingChange> {
        let path = self.path();
        self.http.get(&path).await
    }

    /// Fetch the rating change as raw JSON.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on HTTP failure.
    pub async fn raw(self) -> Result<serde_json::Value> {
        self.http.get_json(&self.path()).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::PixieChessClient;

    #[tokio::test]
    async fn games_get_send_returns_typed_game() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/game/g1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "_id": "x",
                "gameId": "g1",
                "board": {"moves": []},
                "createdAt": "2026-05-13T12:00:00Z",
                "updatedAt": "2026-05-13T12:00:00Z",
                "rated": true,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let g = client.games().get("g1").send().await.unwrap();
        assert_eq!(g.game_id, "g1");
        assert!(g.rated);
    }

    #[tokio::test]
    async fn games_get_raw_returns_value() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/game/g1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "_id": "x",
                "gameId": "g1",
                "board": {},
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let v = client.games().get("g1").raw().await.unwrap();
        assert_eq!(v["gameId"], "g1");
    }

    #[tokio::test]
    async fn rating_change_path_has_two_segments() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/game/g1/rating/0xabc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "rated": true,
                "ratingBefore": 1500.0,
                "ratingAfter": 1510.0,
                "change": 10.0,
            })))
            .mount(&server)
            .await;

        let client = PixieChessClient::builder()
            .base_url(server.uri())
            .build()
            .unwrap();
        let rc = client
            .games()
            .rating_change("g1", "0xabc")
            .send()
            .await
            .unwrap();
        assert!(rc.rated);
        assert!(rc.change.is_some());
    }
}
