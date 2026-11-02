use std::sync::Arc;

use app::{App, shell};
use axum::Router;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};

#[tokio::main]
async fn main() {
    // DATABASE_URL comes from the environment or, failing that, from `.env` in the working directory.
    dotenvy::dotenv().ok();
    let db_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");
    let db = data::Database::connect(&db_url)
        .await
        .expect("connect to Postgres and run migrations");
    warn_if_game_data_is_stale(&db).await;
    // Loaded once: the game data changes only through a reseed, and the server picks that up on
    // its next start.
    let game_data = Arc::new(
        data::load_game_data(&db)
            .await
            .expect("load the game data from Postgres"),
    );

    let conf = get_configuration(None).expect("read [[workspace.metadata.leptos]] configuration");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // `leptos_routes_with_context` also registers the server-function routes, so the game data is
    // available to `#[server]` fns whether they run during SSR or are called from the browser.
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(Arc::clone(&game_data)),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
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
async fn warn_if_game_data_is_stale(db: &data::Database) {
    let dir = data::game_data_dir();
    let reseed = format!(
        "run `cargo run -p data --bin seed -- {}`, then restart the server",
        dir.display()
    );
    match data::game_data_status(db, &dir).await {
        Ok(data::GameDataStatus::Current) => {}
        Ok(data::GameDataStatus::NeverSeeded) => {
            eprintln!("warning: the game data tables have never been seeded; {reseed}");
        }
        Ok(data::GameDataStatus::Stale { seeded, files }) => eprintln!(
            "warning: the game data tables are stale: seeded from data with hash {seeded}, but \
             the files in {} hash to {files}; {reseed}",
            dir.display()
        ),
        Err(error) => eprintln!("warning: couldn't check whether the game data is stale: {error}"),
    }
}
