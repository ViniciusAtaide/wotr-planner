use std::time::Instant;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::middleware::map_response;
use axum::response::Response;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list, site_pkg_dir_service_route_path};
use tower_http::services::ServeDir;
use ui::{App, GameDataUrl, SiteUrl, shell};

use crate::game_data::GameDataFile;

mod game_data;

#[tokio::main]
async fn main() {
    // GAME_DATA_DIR and WOTR_MCP_PORT come from the environment or, failing that, from `.env` in the working
    // directory.
    dotenvy::dotenv().ok();
    // Read once, from the files themselves: the server picks up an edit to them on its next start.
    let dir = seeds::game_data_dir();
    let started = Instant::now();
    let game_data = seeds::check_game_data_files(&dir)
        .unwrap_or_else(|error| panic!("load the game data in {}: {error}", dir.display()));
    let game_data_file = GameDataFile::new(&game_data);
    println!(
        "game data from {} at {game_data_file} (read, checked and serialised in {} ms)",
        dir.display(),
        started.elapsed().as_millis()
    );

    let conf = get_configuration(None).expect("read [[workspace.metadata.leptos]] configuration");
    let addr = conf.leptos_options.site_addr;
    let app = app(game_data_file, conf.leptos_options, site_url());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind site-addr");
    serve_mcp(game_data).await;
    println!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
}

/// Serves the MCP server (`apps/mcp`) over streamable HTTP beside the site, on its own loopback port: `WOTR_MCP_PORT`,
/// else [`mcp::http::DEFAULT_PORT`]. It answers from the same game data as the site. A failure is a warning: the site
/// is served either way.
async fn serve_mcp(game_data: rules::CheckedGameData) {
    let port = match mcp::http::port_from_env() {
        Ok(port) => port.unwrap_or(mcp::http::DEFAULT_PORT),
        Err(error) => return eprintln!("warning: no MCP server: {error}"),
    };
    let planner = mcp::Planner::new(game_data);
    let listener = match mcp::http::Listener::bind(mcp::Server::new(planner), port).await {
        Ok(listener) => listener,
        Err(error) => {
            return eprintln!("warning: no MCP server: can't listen on port {port}: {error}");
        }
    };
    println!("MCP at {}", mcp::http::url(port));
    tokio::spawn(async move {
        if let Err(error) = listener.serve(std::future::pending()).await {
            eprintln!("warning: the MCP server stopped: {error}");
        }
    });
}

/// The site's public address from `SITE_URL`, for the page's link preview (see [`shell`]). The Pages workflow sets
/// it; a dev server runs without it.
fn site_url() -> Option<SiteUrl> {
    SiteUrl::new(&std::env::var("SITE_URL").ok()?)
}

/// The site: the page at `/`, its files under `/pkg`, the game data at its URL, and the files in the site root
/// (the link preview image).
///
/// The page names those files relative to itself (`./pkg/…`, `./game-data/…`), so that saved as files
/// (`scripts/build-site.sh`) it works under any path prefix, as a GitHub project site at `/<repo>/` does. Only the
/// link preview names its image by an absolute URL, from `site_url`, since previews need one.
fn app(
    game_data: GameDataFile,
    leptos_options: LeptosOptions,
    site_url: Option<SiteUrl>,
) -> Router {
    let game_data_url = GameDataUrl(format!(".{}", game_data.url()));
    let routes = generate_route_list(App);

    // The page's head preloads the game data from its URL, which the context gives the shell.
    // Nothing is compressed here: the published site is these files saved as they are (`scripts/build-site.sh`),
    // and GitHub Pages compresses what it serves (ticket 185).
    let site_pkg = Router::new()
        .route_service(
            &site_pkg_dir_service_route_path(&leptos_options),
            ServeDir::new(&*leptos_options.site_root),
        )
        .layer(map_response(revalidate));
    Router::new()
        .merge(game_data.routes())
        .merge(site_pkg)
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(game_data_url.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone(), site_url.clone())
            },
        )
        // A file in the site root, or else a plain 404: the app is the sheet whatever the path, so it isn't
        // rendered here.
        .fallback(leptos_axum::file_and_error_handler(|_| "Not found."))
        .with_state(leptos_options)
}

/// `Cache-Control: no-cache` on the wasm, JS and CSS: their names never change, so without it a browser guesses a
/// lifetime from their age and keeps an old build's files after a rebuild. It asks every time instead, and an
/// unchanged file costs a 304.
async fn revalidate(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::*;

    const SITE_URL: &str = "https://example.github.io/wotr-planner/";

    /// The server's app with a small game data file, and that file's URL.
    fn test_app() -> (Router, String) {
        test_app_at(SiteUrl::new(SITE_URL))
    }

    fn test_app_at(site_url: Option<SiteUrl>) -> (Router, String) {
        let game_data = GameDataFile::from_json(b"{}".to_vec());
        let url = game_data.url().to_owned();
        let options = LeptosOptions::builder().output_name("wotr-planner").build();
        (app(game_data, options, site_url), url)
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

    /// A shared link shows a preview card: the title, the description and the image, which crawlers fetch only
    /// by an absolute URL, so it's named from the site's address.
    #[tokio::test]
    async fn the_page_describes_itself_for_link_previews() {
        let (app, _) = test_app();
        let (_, html) = get(app, "/").await;
        for tag in [
            r#"<meta property="og:title" content="WotR Planner">"#,
            r#"<meta property="og:description" content="Plan a Pathfinder"#,
            r#"<meta name="description" content="Plan a Pathfinder"#,
            r#"<meta name="twitter:card" content="summary_large_image">"#,
            &format!(r#"<meta property="og:url" content="{SITE_URL}">"#),
            &format!(r#"<meta property="og:image" content="{SITE_URL}social-preview.png">"#),
            &format!(r#"<meta name="twitter:image" content="{SITE_URL}social-preview.png">"#),
        ] {
            assert!(html.contains(tag), "{tag} is missing from {html}");
        }
    }

    /// Without the site's address (a dev server) the page still describes itself, but names no image.
    #[tokio::test]
    async fn without_a_site_url_the_page_names_no_preview_image() {
        let (app, _) = test_app_at(None);
        let (_, html) = get(app, "/").await;
        assert!(html.contains(r#"<meta property="og:title""#), "{html}");
        assert!(!html.contains("og:image"), "{html}");
        assert!(!html.contains("og:url"), "{html}");
    }

    /// `SITE_URL` as the workflow or a person gives it: stray spaces and slashes don't change the address.
    #[test]
    fn a_site_url_ends_in_one_slash() {
        for given in [
            "https://example.github.io/wotr-planner",
            "https://example.github.io/wotr-planner/",
            " https://example.github.io/wotr-planner// ",
        ] {
            assert_eq!(SiteUrl::new(given).unwrap().as_str(), SITE_URL, "{given:?}");
        }
        assert_eq!(SiteUrl::new(" / "), None);
    }

    #[tokio::test]
    async fn the_preview_image_is_served_from_the_site_root() {
        let game_data = GameDataFile::from_json(b"{}".to_vec());
        let options = LeptosOptions::builder()
            .output_name("wotr-planner")
            .site_root(concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/public"))
            .build();
        let app = app(game_data, options, None);
        let response = app
            .oneshot(
                Request::get("/social-preview.png")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
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

    /// The site's wasm, JS and CSS as they are on disk, never a compressed copy beside them, and revalidated on
    /// every load, so a rebuild's files reach the browser at once.
    #[tokio::test]
    async fn the_site_files_are_served_as_they_are_and_revalidated() {
        let site_root = tempfile::tempdir().unwrap();
        let pkg = site_root.path().join("pkg");
        std::fs::create_dir(&pkg).unwrap();
        std::fs::write(pkg.join("wotr-planner.js"), "fresh").unwrap();
        std::fs::write(pkg.join("wotr-planner.js.br"), "stale").unwrap();
        let options = LeptosOptions::builder()
            .output_name("wotr-planner")
            .site_root(site_root.path().to_str().unwrap())
            .build();
        let app = app(GameDataFile::from_json(b"{}".to_vec()), options, None);
        let request = Request::get("/pkg/wotr-planner.js")
            .header(header::ACCEPT_ENCODING, "gzip, deflate, br")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers.get(header::CONTENT_ENCODING), None);
        assert_eq!(headers[header::CACHE_CONTROL], "no-cache");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, "fresh");
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
}
