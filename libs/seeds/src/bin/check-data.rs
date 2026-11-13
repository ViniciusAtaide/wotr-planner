//! `cargo run -p seeds --bin check-data [DIR]`
//!
//! Reads the `.ron` files under `DIR` (default: `$GAME_DATA_DIR`, or `libs/game-data/`) and checks them
//! (`GameData::check`), then prints how many of each entity they hold, or every problem. A quick check while editing
//! the data; the real-data test `the_game_data_passes_the_data_check` runs the same check on `libs/game-data/`. A
//! running server keeps the data it read at startup: restart it to serve the edited files.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut dirs: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if dirs.len() > 1 {
        eprintln!("usage: check-data [DIR]");
        return ExitCode::FAILURE;
    }
    let dir = dirs.pop().unwrap_or_else(seeds::game_data_dir);
    match seeds::check_game_data_files(&dir) {
        Ok(game_data) => {
            let counts = game_data
                .counts()
                .iter()
                .map(|(label, count)| format!("{count} {}", label.to_lowercase()))
                .collect::<Vec<_>>()
                .join(", ");
            println!("checked {}: {counts}", dir.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("check-data: {error}");
            ExitCode::FAILURE
        }
    }
}
