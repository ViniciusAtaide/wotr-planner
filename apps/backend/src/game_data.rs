//! The game data as a cacheable GET: one JSON document at a URL that names its content hash, so a browser keeps
//! it for good (`Cache-Control: immutable`) and a reseed, which changes the hash, gives a new URL. The page's head
//! preloads that URL (`ui::GameDataPreload`), so the download starts with the wasm's.
//!
//! The JSON is compressed once, at startup, at each format's best level (brotli quality 11 in a release build,
//! gzip level 9), and a request gets the body its `Accept-Encoding` takes, brotli first. The server's
//! `CompressionLayer` leaves a response that already has `Content-Encoding` alone; compressing on the fly, it
//! would use brotli quality 4.

use std::fmt;
use std::io::Write;
use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use sha2::{Digest, Sha256};

/// How long a browser keeps the game data: a year, without revalidating, since its URL changes with its content.
const CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

/// The game data's JSON, compressed copies of it, and the URL they're served at.
pub struct GameDataFile {
    /// `/game-data/<hash>.json`, the hash being the first 16 hex digits of the JSON's SHA-256.
    url: String,
    json: Bytes,
    /// The JSON at brotli quality 11, or 5 in a debug build ([`BROTLI_QUALITY`]).
    br: Bytes,
    /// The JSON at gzip level 9, for a browser without brotli.
    gzip: Bytes,
}

impl GameDataFile {
    /// Serialises and compresses the game data, once per server start: about 1.7 s in a release build, nearly
    /// all of it brotli at quality 11.
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
            br: brotli(&json).into(),
            gzip: gzip(&json).into(),
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
            get(move |Path(asked): Path<String>, headers: HeaderMap| {
                let found = (asked == name).then(|| file.body_for(&headers));
                async move {
                    match found {
                        Some((encoding, body)) => {
                            let mut response = (
                                [
                                    (header::CONTENT_TYPE, "application/json"),
                                    (header::CACHE_CONTROL, CACHE_CONTROL),
                                    (header::VARY, "accept-encoding"),
                                ],
                                body,
                            )
                                .into_response();
                            if let Some(encoding) = encoding.header_value() {
                                response
                                    .headers_mut()
                                    .insert(header::CONTENT_ENCODING, encoding);
                            }
                            response
                        }
                        None => StatusCode::NOT_FOUND.into_response(),
                    }
                }
            }),
        )
    }

    /// The body a request with these headers takes, and its encoding.
    fn body_for(&self, headers: &HeaderMap) -> (Encoding, Bytes) {
        let encoding = Encoding::accepted(headers);
        let body = match encoding {
            Encoding::Identity => &self.json,
            Encoding::Gzip => &self.gzip,
            Encoding::Brotli => &self.br,
        };
        (encoding, body.clone())
    }
}

/// The URL and the three bodies' sizes, for the startup log.
impl fmt::Display for GameDataFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} B JSON, {} B brotli, {} B gzip",
            self.url,
            self.json.len(),
            self.br.len(),
            self.gzip.len()
        )
    }
}

/// The encodings the game data is kept in, in the order a tie between equal q-values is broken (best last).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Encoding {
    Identity,
    Gzip,
    Brotli,
}

impl Encoding {
    /// The kept encoding with the highest q-value in `Accept-Encoding`, brotli on a tie; identity when it takes
    /// neither brotli nor gzip, or refuses both with `q=0`. Unknown codings and malformed q-values are ignored, as
    /// tower-http's `CompressionLayer` and `ServeDir` do.
    fn accepted(headers: &HeaderMap) -> Self {
        headers
            .get_all(header::ACCEPT_ENCODING)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(','))
            .filter_map(|item| {
                let mut parts = item.split(';');
                let coding = parts.next()?.trim();
                let encoding = if coding.eq_ignore_ascii_case("br") {
                    Self::Brotli
                } else if coding.eq_ignore_ascii_case("gzip")
                    || coding.eq_ignore_ascii_case("x-gzip")
                {
                    Self::Gzip
                } else {
                    return None;
                };
                let q = match parts.next().map(str::trim) {
                    None => 1.0,
                    Some(param) => param
                        .strip_prefix("q=")
                        .or_else(|| param.strip_prefix("Q="))?
                        .parse::<f32>()
                        .ok()
                        .filter(|q| (0.0..=1.0).contains(q))?,
                };
                (q > 0.0).then_some((q, encoding))
            })
            .max_by(|(q1, e1), (q2, e2)| q1.total_cmp(q2).then(e1.cmp(e2)))
            .map_or(Self::Identity, |(_, encoding)| encoding)
    }

    fn header_value(self) -> Option<HeaderValue> {
        match self {
            Self::Identity => None,
            Self::Gzip => Some(HeaderValue::from_static("gzip")),
            Self::Brotli => Some(HeaderValue::from_static("br")),
        }
    }
}

/// The game data's brotli quality: the best, 11, in a release build. Unoptimised, quality 11 takes about 8 s on
/// the game data (brotli's generic code is compiled in this crate, out of the dev profile's `opt-level` for
/// `brotli`), and a dev server restarts on every change, so a debug build uses 5.
const BROTLI_QUALITY: i32 = if cfg!(debug_assertions) { 5 } else { 11 };

/// `json` at [`BROTLI_QUALITY`], with brotli's default 4 MiB window.
fn brotli(json: &[u8]) -> Vec<u8> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: BROTLI_QUALITY,
        lgwin: 22,
        size_hint: json.len(),
        ..Default::default()
    };
    let mut compressed = Vec::new();
    brotli::BrotliCompress(&mut &json[..], &mut compressed, &params)
        .expect("brotli writes to memory");
    compressed
}

/// `json` at gzip's best level, 9.
fn gzip(json: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(json).expect("gzip writes to memory");
    encoder.finish().expect("gzip writes to memory")
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;
    use tower_http::compression::CompressionLayer;

    use super::*;

    /// The file's routes behind the compression layer, as the server has them.
    fn app(json: &str) -> (Router, String) {
        let file = GameDataFile::from_json(json.as_bytes().to_vec());
        let url = file.url().to_owned();
        (file.routes().layer(CompressionLayer::new()), url)
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

    #[tokio::test]
    async fn the_game_data_is_served_at_its_url_to_be_kept_for_good() {
        let json = format!("{{\"feats\":\"{}\"}}", "x".repeat(4096));
        let (app, url) = app(&json);
        let response = get(app, &url, None).await;
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers[header::CONTENT_TYPE], "application/json");
        assert_eq!(
            headers[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, json.as_bytes());
    }

    /// A JSON document shaped like the game data, big enough for the compression layer to want it.
    fn feats_json() -> String {
        let feats: Vec<String> = (0..300)
            .map(|i| format!(r#"{{"key":"feat-{i}","name":"Feat {i}","prerequisites":[]}}"#))
            .collect();
        format!(r#"{{"feats":[{}]}}"#, feats.join(","))
    }

    /// Asks for the game data, behind the compression layer, with `accept_encoding`, and checks that the response
    /// has `expected`'s `Content-Encoding` (none: the plain JSON), `Vary: accept-encoding` and the unchanged
    /// cache headers, and that its body is the copy compressed at startup (the layer didn't touch it) and
    /// decodes to the JSON.
    async fn assert_served(accept_encoding: Option<&str>, expected: Option<&str>) {
        let json = feats_json();
        let file = GameDataFile::from_json(json.clone().into_bytes());
        let (br, gzip) = (file.br.clone(), file.gzip.clone());
        let url = file.url().to_owned();
        let response = get(
            file.routes().layer(CompressionLayer::new()),
            &url,
            accept_encoding,
        )
        .await;

        let asked = format!("Accept-Encoding: {accept_encoding:?}");
        assert_eq!(response.status(), StatusCode::OK, "{asked}");
        let headers = response.headers();
        let encoding = headers
            .get(header::CONTENT_ENCODING)
            .map(|value| value.to_str().unwrap());
        assert_eq!(encoding, expected, "{asked}");
        let vary: Vec<_> = headers.get_all(header::VARY).iter().collect();
        assert_eq!(vary, ["accept-encoding"], "{asked}");
        assert_eq!(headers[header::CONTENT_TYPE], "application/json", "{asked}");
        assert_eq!(
            headers[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable",
            "{asked}"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let mut decoded = Vec::new();
        match expected {
            None => decoded = body.to_vec(),
            Some("br") => {
                assert_eq!(body, br, "{asked}");
                brotli::Decompressor::new(&body[..], 4096)
                    .read_to_end(&mut decoded)
                    .unwrap();
            }
            Some("gzip") => {
                assert_eq!(body, gzip, "{asked}");
                flate2::read::GzDecoder::new(&body[..])
                    .read_to_end(&mut decoded)
                    .unwrap();
            }
            Some(other) => panic!("no {other} copy is kept"),
        }
        assert!(
            decoded == json.as_bytes(),
            "{asked}: the body isn't the JSON"
        );
    }

    #[tokio::test]
    async fn a_browser_that_takes_brotli_gets_the_brotli_copy() {
        assert_served(Some("br"), Some("br")).await;
    }

    #[tokio::test]
    async fn a_browser_that_takes_only_gzip_gets_the_gzip_copy() {
        assert_served(Some("gzip"), Some("gzip")).await;
        assert_served(Some("gzip, deflate"), Some("gzip")).await;
    }

    #[tokio::test]
    async fn a_browser_that_takes_both_gets_brotli() {
        assert_served(Some("gzip, deflate, br, zstd"), Some("br")).await;
        assert_served(Some("br, gzip"), Some("br")).await;
    }

    #[tokio::test]
    async fn a_browser_that_takes_neither_gets_the_plain_json() {
        assert_served(None, None).await;
        assert_served(Some("identity"), None).await;
        assert_served(Some("deflate, zstd"), None).await;
    }

    #[tokio::test]
    async fn the_q_values_choose_between_brotli_and_gzip() {
        assert_served(Some("br;q=0, gzip"), Some("gzip")).await;
        assert_served(Some("gzip, br;q=0.5"), Some("gzip")).await;
        assert_served(Some("gzip;q=0.5, br;q=0.5"), Some("br")).await;
        assert_served(Some("gzip;q=0, br;q=0"), None).await;
    }

    #[tokio::test]
    async fn another_name_is_not_found() {
        let (app, url) = app("{}");
        let stale = url.replace("44136fa355b3678a", "0000000000000000");
        assert_eq!(get(app, &stale, None).await.status(), StatusCode::NOT_FOUND);
    }
}
