//! `cargo run -p data --bin seed [--check-only] [DIR]`
//!
//! Checks the `.ron` files under `DIR` (default: `$GAME_DATA_DIR`, or `libs/game-data/`),
//! replaces every game table with them in one transaction, and stores the files' hash so the
//! server can warn when the tables are stale. Runs pending migrations first. A running server keeps
//! the game data it loaded at startup: restart it to serve the new data.
//!
//! With `--check-only` it reads and checks the files (`GameData::check`) and never connects to
//! Postgres, so any number can run at once. Keys that don't exist are caught only by a real seed.

use std::path::PathBuf;
use std::process::ExitCode;

use rules::GameData;

#[tokio::main]
async fn main() -> ExitCode {
    let mut check_only = false;
    let mut dirs = Vec::new();
    for arg in std::env::args_os().skip(1) {
        if arg == "--check-only" {
            check_only = true;
        } else {
            dirs.push(PathBuf::from(arg));
        }
    }
    if dirs.len() > 1 {
        eprintln!("usage: seed [--check-only] [DIR]");
        return ExitCode::FAILURE;
    }
    let dir = dirs.pop().unwrap_or_else(data::game_data_dir);

    if check_only {
        return match data::check_game_data_files(&dir) {
            Ok(game_data) => {
                println!("checked {}: {}", dir.display(), counts(&game_data));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("seed: {error}");
                ExitCode::FAILURE
            }
        };
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
            println!(
                "seeded {} (hash {hash}): {}",
                dir.display(),
                counts(&game_data)
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("seed: seeded, but reading the data back failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn counts(game_data: &GameData) -> String {
    game_data
        .counts()
        .iter()
        .map(|(label, count)| format!("{count} {}", label.to_lowercase()))
        .collect::<Vec<_>>()
        .join(", ")
}
