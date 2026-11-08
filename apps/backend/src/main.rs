use std::time::Instant;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::middleware::map_response;
use axum::response::Response;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list, site_pkg_dir_service_route_path};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use ui::{App, GameDataUrl, shell};

use crate::game_data::GameDataFile;

mod game_data;

#[tokio::main]
async fn main() {
    // DATABASE_URL comes from the environment or, failing that, from `.env` in the working directory; without it,
    // the database is `wotr-planner.db` at the repo root.
    dotenvy::dotenv().ok();
    let db_path = seeds::database_path().expect("read DATABASE_URL (see .env.example)");
    let db = seeds::Database::open(&db_path)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "open the game data database {}: {error} (build it with `cargo run -p seeds --bin seed`)",
                db_path.display()
            )
        });
    warn_if_game_data_is_stale(&db).await;
    // Loaded once: the game data changes only through a reseed, and the server picks that up on
    // its next start.
    let game_data = seeds::load_game_data(&db)
        .await
        .expect("load the game data from the database");
    let started = Instant::now();
    let game_data = GameDataFile::new(&game_data);
    println!(
        "game data at {game_data} (serialised and compressed in {} ms)",
        started.elapsed().as_millis()
    );

    let conf = get_configuration(None).expect("read [[workspace.metadata.leptos]] configuration");
    let addr = conf.leptos_options.site_addr;
    let app = app(game_data, conf.leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind site-addr");
    println!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
}

/// The site: the page at `/`, its files under `/pkg`, and the game data at its URL.
///
/// The page names those files relative to itself (`./pkg/…`, `./game-data/…`), so that saved as files
/// (`scripts/build-site.sh`) it works under any path prefix, as a GitHub project site at `/<repo>/` does.
fn app(game_data: GameDataFile, leptos_options: LeptosOptions) -> Router {
    let game_data_url = GameDataUrl(format!(".{}", game_data.url()));
    let routes = generate_route_list(App);

    // The page's head preloads the game data from its URL, which the context gives the shell.
    // Everything is compressed (brotli or gzip) for a browser that takes it:
    // - the game data once, at startup;
    // - in a release build, the wasm, JS and CSS once, by `cargo leptos build --release --precompress`. Only its
    //   `.br` files are served (`precompressed_br`): its `.gz` files (libflate) are 11–17 % bigger than the gzip
    //   `CompressionLayer` makes on the fly, so a browser without brotli still gets that. A dev build has no
    //   `.br` files, so everything is compressed on the fly there;
    // - the rest, the HTML, on the fly.
    let site_pkg = ServeDir::new(&*leptos_options.site_root).precompressed_br();
    Router::new()
        .merge(game_data.routes())
        .route_service(&site_pkg_dir_service_route_path(&leptos_options), site_pkg)
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(game_data_url.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        // A file in the site root, or else a plain 404: the app is the sheet whatever the path, so it isn't
        // rendered here.
        .fallback(leptos_axum::file_and_error_handler(|_| "Not found."))
        .layer(map_response(vary_on_accept_encoding))
        .layer(CompressionLayer::new())
        .with_state(leptos_options)
}

/// Adds `Vary: Accept-Encoding` to a compressed response that lacks it. A precompressed static file comes
/// without it (tower-http's `ServeDir` never sets `Vary`), and `CompressionLayer` adds it only to the responses
/// it compresses itself.
async fn vary_on_accept_encoding(mut response: Response) -> Response {
    let headers = response.headers_mut();
    let varies = headers.get_all(header::VARY).iter().any(|value| {
        value
            .to_str()
            .is_ok_and(|value| value.to_ascii_lowercase().contains("accept-encoding"))
    });
    if headers.contains_key(header::CONTENT_ENCODING) && !varies {
        headers.append(header::VARY, HeaderValue::from_static("accept-encoding"));
    }
    response
}

/// Logs a warning when the game tables don't match the data files. Never reseeds.
async fn warn_if_game_data_is_stale(db: &seeds::Database) {
    let dir = seeds::game_data_dir();
    let reseed = format!(
        "run `cargo run -p seeds --bin seed -- {}`, then restart the server",
        dir.display()
    );
    match seeds::game_data_status(db, &dir).await {
        Ok(seeds::GameDataStatus::Current) => {}
        Ok(seeds::GameDataStatus::NeverSeeded) => {
            eprintln!("warning: the game data tables have never been seeded; {reseed}");
        }
        Ok(seeds::GameDataStatus::Stale { seeded, files }) => eprintln!(
            "warning: the game data tables are stale: seeded from data with hash {seeded}, but \
             the files in {} hash to {files}; {reseed}",
            dir.display()
        ),
        Err(error) => eprintln!("warning: couldn't check whether the game data is stale: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::*;

    /// The server's app with a small game data file, and that file's URL.
    fn test_app() -> (Router, String) {
        let game_data = GameDataFile::from_json(b"{}".to_vec());
        let url = game_data.url().to_owned();
        let options = LeptosOptions::builder().output_name("wotr-planner").build();
        (app(game_data, options), url)
    }

    async fn get(app: Router, url: &str) -> (StatusCode, String) {
        let response = app
            .oneshot(Request::get(url).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    /// The static site (`scripts/build-site.sh`) is this page saved as files, served under any path prefix (a
    /// GitHub project site is at `/<repo>/`), so the page names its stylesheet, scripts, wasm and game data
    /// relative to itself.
    #[tokio::test]
    async fn the_page_names_its_files_relative_to_itself() {
        let (app, url) = test_app();
        let (status, html) = get(app, "/").await;
        assert_eq!(status, StatusCode::OK);
        for file in [
            r#"href="./pkg/wotr-planner.css""#,
            r#"href="./pkg/wotr-planner.js""#,
            "./pkg/wotr-planner_bg.wasm",
            &format!(r#"href=".{url}""#),
        ] {
            assert!(html.contains(file), "{file} is missing from {html}");
        }
        for absolute in [r#"="/"#, r#""/pkg"#, r#""/game-data"#] {
            assert!(!html.contains(absolute), "{absolute} is in {html}");
        }
    }

    /// Without an icon link a browser asks for `/favicon.ico` at the host's root, outside a project site: a 404.
    #[tokio::test]
    async fn the_page_names_its_icon_so_the_browser_asks_for_none() {
        let (app, _) = test_app();
        let (_, html) = get(app, "/").await;
        assert!(
            html.contains(r#"<link rel="icon" href="data:,">"#),
            "{html}"
        );
    }

    #[tokio::test]
    async fn the_game_data_is_at_the_url_the_page_names() {
        let (app, url) = test_app();
        assert_eq!(get(app, &url).await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn any_other_page_is_not_found() {
        let (app, _) = test_app();
        let (status, html) = get(app, "/wotr-planner/").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(html.contains("Not found."), "{html}");
    }

    async fn vary_of(headers: &[(header::HeaderName, &'static str)]) -> Vec<HeaderValue> {
        let mut response = Response::new(axum::body::Body::empty());
        for (name, value) in headers {
            response
                .headers_mut()
                .append(name, HeaderValue::from_static(value));
        }
        let response = vary_on_accept_encoding(response).await;
        response
            .headers()
            .get_all(header::VARY)
            .iter()
            .cloned()
            .collect()
    }

    #[tokio::test]
    async fn a_precompressed_file_varies_on_accept_encoding() {
        assert_eq!(
            vary_of(&[(header::CONTENT_ENCODING, "br")]).await,
            ["accept-encoding"]
        );
        assert_eq!(
            vary_of(&[(header::CONTENT_ENCODING, "gzip"), (header::VARY, "origin")]).await,
            ["origin", "accept-encoding"]
        );
    }

    #[tokio::test]
    async fn vary_is_left_alone_when_it_names_accept_encoding_or_nothing_is_compressed() {
        assert_eq!(
            vary_of(&[
                (header::CONTENT_ENCODING, "br"),
                (header::VARY, "Accept-Encoding")
            ])
            .await,
            ["Accept-Encoding"]
        );
        assert!(vary_of(&[]).await.is_empty());
    }
}
