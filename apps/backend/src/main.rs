use axum::Router;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use ui::{App, GameDataUrl, shell};

use crate::game_data::GameDataFile;

mod game_data;

#[tokio::main]
async fn main() {
    // DATABASE_URL comes from the environment or, failing that, from `.env` in the working directory.
    dotenvy::dotenv().ok();
    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let db = seeds::Database::connect(&db_url)
        .await
        .expect("connect to Postgres and run migrations");
    warn_if_game_data_is_stale(&db).await;
    // Loaded once: the game data changes only through a reseed, and the server picks that up on
    // its next start.
    let game_data = seeds::load_game_data(&db)
        .await
        .expect("load the game data from Postgres");
    let game_data = GameDataFile::new(&game_data);
    let game_data_url = GameDataUrl(game_data.url().to_owned());

    let conf = get_configuration(None).expect("read [[workspace.metadata.leptos]] configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // The page's head preloads the game data from its URL, which the context gives the shell.
    // Everything is compressed (brotli or gzip) for a browser that takes it.
    let app = Router::new()
        .merge(game_data.routes())
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(game_data_url.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(CompressionLayer::new())
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind site-addr");
    println!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
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
