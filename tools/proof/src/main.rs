//! The proof's command line; see the library's docs for the commands.

use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use proof::{Named, builds, compare, fingerprint, links};
use rules::{Engine, GameData};

const USAGE: &str = "usage:
  proof record <game-data dir> [--links <file> | --builds <file>] [--threads N] [--only <name substring>]
  proof compare <a.jsonl> <b.jsonl>
  proof export <game-data dir> [--json] [--threads N] [--only <name substring>]";

/// The options `record` and `export` take.
struct Options {
    data: String,
    links: Option<String>,
    /// A file of Builds as JSON (`export --json`) to record instead of the harness's own.
    builds: Option<String>,
    /// `export` writes JSON lines instead of links.
    json: bool,
    threads: usize,
    only: Option<String>,
}

fn options(args: &[String], links_allowed: bool) -> Result<Options, String> {
    let mut args = args.iter();
    let data = args.next().ok_or("no game-data dir")?.clone();
    let mut options = Options {
        data,
        links: None,
        builds: None,
        json: false,
        threads: 8,
        only: None,
    };
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .cloned()
                .ok_or_else(|| format!("{flag} takes a value"))
        };
        match flag.as_str() {
            "--links" if links_allowed => options.links = Some(value()?),
            "--builds" if links_allowed => options.builds = Some(value()?),
            "--json" if !links_allowed => options.json = true,
            "--threads" => {
                options.threads = value()?.parse().map_err(|_| "--threads takes a number")?;
            }
            "--only" => options.only = Some(value()?),
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(options)
}

/// The engine on the game data, the game data in the engine's order, and the Builds the options name.
fn engine_and_builds(options: &Options) -> Result<(Engine, GameData, Vec<Named>), String> {
    let (engine, data) = proof::engine_on(Path::new(&options.data))
        .map_err(|error| format!("reading {}: {error:?}", options.data))?;
    let read =
        |path: &str| std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"));
    let mut builds = match (&options.links, &options.builds) {
        (Some(_), Some(_)) => return Err("--links and --builds don't go together".to_owned()),
        (None, None) => builds::all(&engine, &data, options.threads),
        (Some(path), None) => {
            let (builds, failed) = links::read(&data, &read(path)?);
            for name in &failed {
                eprintln!("doesn't decode, not recorded: {name}");
            }
            builds
        }
        (None, Some(path)) => {
            let (builds, failed) = links::read_json(&read(path)?);
            for name in &failed {
                eprintln!("not recorded: {name}");
            }
            builds
        }
    };
    if let Some(only) = &options.only {
        builds.retain(|(name, _)| name.contains(only.as_str()));
    }
    Ok((engine, data, builds))
}

fn record(options: &Options) -> Result<ExitCode, String> {
    let started = Instant::now();
    let (engine, data, builds) = engine_and_builds(options)?;
    eprintln!("{} Builds", builds.len());
    for line in proof::record(&engine, &data, &builds, options.threads) {
        println!("{line}");
    }
    let (checked, feat_options) = fingerprint::shortcut_counts();
    eprintln!("shortcut checked on {checked} Builds, {feat_options} feat options");
    let held: Vec<String> = fingerprint::held_counts()
        .into_iter()
        .map(|(kind, count)| format!("{kind} {count}"))
        .collect();
    eprintln!("held choices checked: {}", held.join(", "));
    eprintln!("recorded in {:.1?}", started.elapsed());
    Ok(ExitCode::SUCCESS)
}

fn export(options: &Options) -> Result<ExitCode, String> {
    let (_, data, builds) = engine_and_builds(options)?;
    if options.json {
        eprintln!("{} Builds exported", builds.len());
        print!("{}", links::export_json(&builds));
        return Ok(ExitCode::SUCCESS);
    }
    let (text, skipped) = links::export(&data, &builds);
    for name in &skipped {
        eprintln!("no link gives this Build back, not exported: {name}");
    }
    eprintln!("{} Builds exported", builds.len() - skipped.len());
    print!("{text}");
    Ok(ExitCode::SUCCESS)
}

fn compare(a: &str, b: &str) -> Result<ExitCode, String> {
    let read =
        |path: &str| std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"));
    let comparison = compare::compare(&read(a)?, &read(b)?)?;
    for line in &comparison.lines {
        println!("{line}");
    }
    eprintln!("{}", comparison.summary);
    Ok(if comparison.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("record") => options(&args[1..], true).and_then(|options| record(&options)),
        Some("export") => options(&args[1..], false).and_then(|options| export(&options)),
        Some("compare") if args.len() == 3 => compare(&args[1], &args[2]),
        _ => Err("no such command".to_owned()),
    };
    result.unwrap_or_else(|error| {
        eprintln!("{error}\n{USAGE}");
        ExitCode::from(2)
    })
}
