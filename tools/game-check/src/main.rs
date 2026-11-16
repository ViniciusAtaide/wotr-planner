//! `game-check [--binz <file>] [--data <dir>]`: compares the game data with the game's pinned dump and prints the
//! report.
//!
//! - `--binz`: the dump; by default `blueprints/<the pinned asset>` in the workspace this was compiled in, or in the
//!   main checkout when that has none (a worktree).
//! - `--data`: the game data; by default `seeds::game_data_dir()`.
//!
//! The report goes to stdout, the run's time to stderr. Exit status: 0 when every difference is known
//! (`known.txt`), 1 when one isn't or a known line matches none, 2 when the dump is refused, a blueprint's JSON
//! doesn't parse, or nothing could be read.
//!
//! Blueprint lookups (ticket 299), each taking `--binz` too and printing no localized text:
//!
//! - `game-check show <name|guid>…`: each blueprint's name, GUID and type, then its JSON with every reference
//!   resolved to `<name> <guid>`.
//! - `game-check grep <regex>`: `guid name type` of each blueprint whose name or JSON matches.
//! - `game-check refs <name|guid>`: `guid name type` of each blueprint that references it.
//!
//! A lookup exits 0, or 2 when the dump is refused, the name is unknown or the pattern doesn't parse.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use game_check::{Known, Library, Pins, Report, binz, check, default_dump};

fn main() -> ExitCode {
    let start = Instant::now();
    match run() {
        Ok(Outcome::Lookup(text)) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Ok(Outcome::Report(report)) => {
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

enum Outcome {
    Report(Report),
    Lookup(String),
}

const USAGE: &str = "usage: game-check [--binz <file>] [--data <dir>]\n       game-check [--binz <file>] show <name|guid>... | grep <regex> | refs <name|guid>";

fn run() -> Result<Outcome, String> {
    let mut binz_path = None;
    let mut data_dir = None;
    let mut words = Vec::new();
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
            _ if arg.starts_with("--") => return Err(format!("{USAGE}; not {arg}")),
            _ => words.push(arg),
        }
    }
    let pins = Pins::embedded().map_err(|error| error.to_string())?;
    let binz_path = binz_path.unwrap_or_else(|| {
        default_dump(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
            &pins.asset,
        )
    });
    if !words.is_empty() {
        return lookup(&words, &binz_path, &pins).map(Outcome::Lookup);
    }
    let known = Known::embedded().map_err(|error| error.to_string())?;
    let data_dir = data_dir.unwrap_or_else(seeds::game_data_dir);
    let data = seeds::check_game_data_files(&data_dir).map_err(|error| error.to_string())?;
    let dump = binz::read(&binz_path, &pins).map_err(|error| error.to_string())?;
    check(&pins, &dump, &data, &known)
        .map(Outcome::Report)
        .map_err(|error| error.to_string())
}

/// `show`, `grep` or `refs`, over the pinned dump's every blueprint.
fn lookup(words: &[String], binz_path: &Path, pins: &Pins) -> Result<String, String> {
    let (command, rest) = words.split_first().ok_or(USAGE)?;
    let one = |what: &str| match rest {
        [one] => Ok(one),
        _ => Err(format!("{command} takes one {what}\n{USAGE}")),
    };
    // Checked before the dump is read: a wrong command or count costs no 2-second read.
    match command.as_str() {
        "show" if rest.is_empty() => return Err(format!("show takes a name or GUID\n{USAGE}")),
        "show" => {}
        "grep" => {
            one("pattern")?;
        }
        "refs" => {
            one("name or GUID")?;
        }
        _ => return Err(format!("{USAGE}; not {command}")),
    }
    let bytes = binz::pinned_bytes(binz_path, pins).map_err(|error| error.to_string())?;
    let library = Library::parse(&bytes).map_err(|error| error.to_string())?;
    match command.as_str() {
        "show" => library.show(rest),
        "grep" => library.grep(&rest[0]),
        _ => library.refs(&rest[0]),
    }
}
