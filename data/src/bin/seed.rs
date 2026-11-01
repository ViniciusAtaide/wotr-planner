//! `cargo run -p data --bin seed [DIR]`
//!
//! Checks the `.ron` files under `DIR` (default: `$GAME_DATA_DIR`, or `game-data/` at the repo
//! root), replaces every game table with them in one transaction, and stores the files' hash so the
//! server can warn when the tables are stale. Runs pending migrations first. A running server keeps
//! the game data it loaded at startup: restart it to serve the new data.

use std::path::PathBuf;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let dir = args.next().map_or_else(data::game_data_dir, PathBuf::from);
    if args.next().is_some() {
        eprintln!("usage: seed [DIR]");
        return ExitCode::FAILURE;
    }

    dotenvy::dotenv().ok();
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("seed: DATABASE_URL must be set (see .env.example)");
        return ExitCode::FAILURE;
    };
    let db = match data::Database::connect(&url).await {
        Ok(db) => db,
        Err(error) => {
            eprintln!("seed: connecting to Postgres: {error}");
            return ExitCode::FAILURE;
        }
    };

    let hash = match data::seed(&db, &dir).await {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("seed: {error}");
            return ExitCode::FAILURE;
        }
    };
    match data::load_game_data(&db).await {
        Ok(game_data) => {
            let counts: Vec<String> = game_data
                .counts()
                .iter()
                .map(|(label, count)| format!("{count} {}", label.to_lowercase()))
                .collect();
            println!(
                "seeded {} (hash {hash}): {}",
                dir.display(),
                counts.join(", ")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("seed: seeded, but reading the data back failed: {error}");
            ExitCode::FAILURE
        }
    }
}
