//! The game data as a cacheable GET: one JSON document at a URL that names its content hash, so a browser keeps
//! it for good (`Cache-Control: immutable`) and an edit to the data files, which changes the hash, gives a new URL.
//! The page's head preloads that URL (`ui::GameDataPreload`), so the download starts with the wasm's.
//!
//! It's served as it is, never compressed: the published site is this file saved as it is, and GitHub Pages
//! compresses what it serves (ticket 185).

use std::fmt;
use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use sha2::{Digest, Sha256};

/// How long a browser keeps the game data: a year, without revalidating, since its URL changes with its content.
const CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

/// The game data's JSON and the URL it's served at.
pub struct GameDataFile {
    /// `/game-data/<hash>.json`, the hash being the first 16 hex digits of the JSON's SHA-256.
    url: String,
    json: Bytes,
}

impl GameDataFile {
    /// Serialises the game data, once per server start.
    pub fn new(game_data: &rules::GameData) -> Self {
        Self::from_json(serde_json::to_vec(game_data).expect("the game data serialises to JSON"))
    }

    pub(crate) fn from_json(json: Vec<u8>) -> Self {
        let hash: String = Sha256::digest(&json)[..8]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self {
            url: format!("/game-data/{hash}.json"),
            json: json.into(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// A router that serves the file at its URL; any other name under `/game-data/` is not found.
    pub fn routes<S: Clone + Send + Sync + 'static>(self) -> Router<S> {
        let name = self.url.trim_start_matches("/game-data/").to_owned();
        let file = Arc::new(self);
        Router::new().route(
            "/game-data/{name}",
            get(move |Path(asked): Path<String>| {
                let found = (asked == name).then(|| file.json.clone());
                async move {
                    match found {
                        Some(json) => (
                            [
                                (header::CONTENT_TYPE, "application/json"),
                                (header::CACHE_CONTROL, CACHE_CONTROL),
                            ],
                            json,
                        )
                            .into_response(),
                        None => StatusCode::NOT_FOUND.into_response(),
                    }
                }
            }),
        )
    }
}

/// The URL and the JSON's size, for the startup log.
impl fmt::Display for GameDataFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} B JSON", self.url, self.json.len())
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;

    use super::*;

    fn app(json: &str) -> (Router, String) {
        let file = GameDataFile::from_json(json.as_bytes().to_vec());
        let url = file.url().to_owned();
        (file.routes(), url)
    }

    async fn get(
        app: Router,
        url: &str,
        accept_encoding: Option<&str>,
    ) -> axum::response::Response {
        let mut request = Request::get(url);
        if let Some(encoding) = accept_encoding {
            request = request.header(header::ACCEPT_ENCODING, encoding);
        }
        app.oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[test]
    fn the_url_names_the_contents_hash() {
        let url = GameDataFile::from_json(b"{}".to_vec()).url().to_owned();
        // SHA-256 of "{}": 44136fa355b3678a…
        assert_eq!(url, "/game-data/44136fa355b3678a.json");
        assert_ne!(GameDataFile::from_json(b"[]".to_vec()).url(), url);
    }

    /// The JSON as it is, kept for good, whatever encodings the browser takes.
    #[tokio::test]
    async fn the_game_data_is_served_at_its_url_as_it_is_to_be_kept_for_good() {
        let json = format!("{{\"feats\":\"{}\"}}", "x".repeat(4096));
        for accept_encoding in [None, Some("gzip, deflate, br, zstd")] {
            let (app, url) = app(&json);
            let response = get(app, &url, accept_encoding).await;
            assert_eq!(response.status(), StatusCode::OK);
            let headers = response.headers();
            assert_eq!(headers[header::CONTENT_TYPE], "application/json");
            assert_eq!(
                headers[header::CACHE_CONTROL],
                "public, max-age=31536000, immutable"
            );
            assert_eq!(headers.get(header::CONTENT_ENCODING), None);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(body, json.as_bytes());
        }
    }

    #[tokio::test]
    async fn another_name_is_not_found() {
        let (app, url) = app("{}");
        let stale = url.replace("44136fa355b3678a", "0000000000000000");
        assert_eq!(get(app, &stale, None).await.status(), StatusCode::NOT_FOUND);
    }
}
