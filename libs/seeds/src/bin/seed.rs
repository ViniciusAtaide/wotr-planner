//! `cargo run -p seeds --bin seed [--check-only] [DIR]`
//!
//! Checks the `.ron` files under `DIR` (default: `$GAME_DATA_DIR`, or `libs/game-data/`), builds a new
//! database from them (the migration, every table, the files' hash so the server can warn when the tables
//! are stale, and a foreign key check naming every key the data lacks), and renames it over the database
//! file (`$DATABASE_URL`, or `wotr-planner.db` at the repo root). A running server keeps the game data it
//! loaded at startup: restart it to serve the new data.
//!
//! With `--check-only` it reads and checks the files (`GameData::check`) and never touches the
//! database, so any number can run at once. Keys that don't exist are caught only by a real seed.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

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
    let dir = dirs.pop().unwrap_or_else(seeds::game_data_dir);

    if check_only {
        return match seeds::check_game_data_files(&dir) {
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
    let database = match seeds::database_path() {
        Ok(database) => database,
        Err(error) => {
            eprintln!("seed: reading DATABASE_URL (see .env.example): {error}");
            return ExitCode::FAILURE;
        }
    };

    let started = Instant::now();
    let hash = match seeds::seed(&database, &dir).await {
        Ok(hash) => hash,
        Err(error) => {
            eprintln!("seed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let seeded_in = started.elapsed();
    let loaded = match seeds::Database::open(&database).await {
        Ok(db) => seeds::load_game_data(&db).await,
        Err(error) => Err(error),
    };
    match loaded {
        Ok(game_data) => {
            println!(
                "seeded {} from {} in {} ms (hash {hash}): {}",
                database.display(),
                dir.display(),
                seeded_in.as_millis(),
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
