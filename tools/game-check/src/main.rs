//! `game-check [--binz <file>] [--data <dir>]`: compares the game data with the game's pinned dump and prints the
//! report.
//!
//! - `--binz`: the dump; by default `blueprints/<the pinned asset>` in the workspace this was compiled in.
//! - `--data`: the game data; by default `seeds::game_data_dir()`.
//!
//! The report goes to stdout, the run's time to stderr. Exit status: 0 when every difference is known
//! (`known.txt`), 1 when one isn't or a known line matches none, 2 when the dump is refused or nothing could be read.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use game_check::{Known, Pins, Report, binz, check};

fn main() -> ExitCode {
    let start = Instant::now();
    match run() {
        Ok(report) => {
            print!("{}", report.text());
            eprintln!("game-check: {:.1} s", start.elapsed().as_secs_f64());
            if report.failed() {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            eprintln!("game-check: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<Report, String> {
    let mut binz_path = None;
    let mut data_dir = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = |args: &mut dyn Iterator<Item = String>| {
            args.next()
                .map(PathBuf::from)
                .ok_or_else(|| format!("{arg} takes a path"))
        };
        match arg.as_str() {
            "--binz" => binz_path = Some(value(&mut args)?),
            "--data" => data_dir = Some(value(&mut args)?),
            _ => {
                return Err(format!(
                    "usage: game-check [--binz <file>] [--data <dir>]; not {arg}"
                ));
            }
        }
    }
    let pins = Pins::embedded().map_err(|error| error.to_string())?;
    let known = Known::embedded().map_err(|error| error.to_string())?;
    let binz_path = binz_path.unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../blueprints")
            .join(&pins.asset)
    });
    let data_dir = data_dir.unwrap_or_else(seeds::game_data_dir);
    let data = seeds::check_game_data_files(&data_dir).map_err(|error| error.to_string())?;
    let dump = binz::read(&binz_path, &pins).map_err(|error| error.to_string())?;
    Ok(check(&pins, &dump, &data, &known))
}
