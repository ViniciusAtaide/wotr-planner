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
//! `game-check --write <area>[,<area>...] [--dry-run] [--log <file>] [--data <dir>] [--binz <file>]` (ticket 293): brings
//! the data's text to the game's values for the areas named (`items`, `class-tables`, `option-lists`; `--only` limits `option-lists` to the selections named), changing only the fields the importer owns and
//! keeping every other byte. It prints, per area, the fields written and the files touched, then what it left
//! unwritten. `--dry-run` plans and runs the guards and writes nothing. `--log` writes one TSV row per change (not with
//! `--dry-run`, which has no report after). Exit status: 0 when it wrote or had nothing to write, 2 when a guard
//! refuses (nothing is written), the dump is refused or the written data fails its check (every file is put back).
//!
//! Blueprint lookups (ticket 299), each taking `--binz` too and printing no localized text:
//!
//! - `game-check show <name|guid>…`: each blueprint's name, GUID and type, then its JSON with every reference
//!   resolved to `<name> <guid>`.
//! - `game-check grep <regex>`: `guid name type` of each blueprint whose name or JSON matches.
//! - `game-check refs <name|guid>`: `guid name type` of each blueprint that references it.
//! - `game-check granted-by <name|guid>`: the chains of referrers from it up to the first progression, archetype,
//!   class, race or unit (ticket 311).
//!
//! `show`, `refs` and `granted-by` begin with a banner when the name is shared by several blueprints (each GUID,
//! with what grants it): pick the blueprint by what grants it, never the first one printed.
//!
//! A lookup exits 0, or 2 when the dump is refused, the name is unknown or the pattern doesn't parse.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use game_check::write::facts::Facts;
use game_check::write::{WriteArea, log, run as write};
use game_check::{Known, Library, Pins, Report, binz, check, default_dump};

fn main() -> ExitCode {
    let start = Instant::now();
    match run() {
        Ok(Outcome::Lookup(text) | Outcome::Written(text)) => {
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
    Written(String),
}

const USAGE: &str = "usage: game-check [--binz <file>] [--data <dir>]\n       game-check --write <area>[,<area>...] [--only <key>[,<key>...]] [--dry-run] [--log <file>] [--data <dir>] [--binz <file>]   (areas: items, class-tables, option-lists)\n       game-check [--binz <file>] show <name|guid>... | grep <regex> | refs <name|guid> | granted-by <name|guid>";

fn run() -> Result<Outcome, String> {
    let mut binz_path = None;
    let mut data_dir = None;
    let mut write_areas: Option<String> = None;
    let mut log_path = None;
    let mut dry_run = false;
    let mut only: Option<std::collections::BTreeSet<String>> = None;
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
            "--log" => log_path = Some(value(&mut args)?),
            "--dry-run" => dry_run = true,
            "--only" => {
                let list = args
                    .next()
                    .ok_or_else(|| format!("--only takes selection keys\n{USAGE}"))?;
                only = Some(list.split(',').map(str::to_owned).collect());
            }
            "--write" => {
                write_areas = Some(
                    args.next()
                        .ok_or_else(|| format!("--write takes areas\n{USAGE}"))?,
                );
            }
            _ if arg.starts_with("--") => return Err(format!("{USAGE}; not {arg}")),
            _ => words.push(arg),
        }
    }
    if write_areas.is_some() && !words.is_empty() {
        return Err(format!(
            "--write takes no lookup command, not `{}`\n{USAGE}",
            words.join(" ")
        ));
    }
    if let Some(path) = &log_path
        && write_areas.is_some()
    {
        // A log that can't be written is found before anything is: never after the data is.
        std::fs::File::create(path)
            .map_err(|error| format!("--log {}: {error}", path.display()))?;
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
    if (dry_run || log_path.is_some()) && write_areas.is_none() {
        return Err(format!("--dry-run and --log go with --write\n{USAGE}"));
    }
    if only.is_some() && write_areas.is_none() {
        return Err(format!("--only goes with --write\n{USAGE}"));
    }
    if dry_run && log_path.is_some() {
        return Err(format!(
            "--log records what was written, so not with --dry-run\n{USAGE}"
        ));
    }
    let known = Known::embedded().map_err(|error| error.to_string())?;
    let data_dir = data_dir.unwrap_or_else(seeds::game_data_dir);
    if let Some(areas) = write_areas {
        let areas = parse_areas(&areas)?;
        return write_run(
            (&areas, only.as_ref()),
            dry_run,
            log_path.as_deref(),
            &data_dir,
            &binz_path,
            &pins,
            &known,
        )
        .map(Outcome::Written);
    }
    let data = seeds::check_game_data_files(&data_dir).map_err(|error| error.to_string())?;
    let dump = binz::read(&binz_path, &pins).map_err(|error| error.to_string())?;
    check(&pins, &dump, &data, &known)
        .map(Outcome::Report)
        .map_err(|error| error.to_string())
}

/// `show`, `grep`, `refs` or `granted-by`, over the pinned dump's every blueprint.
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
        "refs" | "granted-by" => {
            one("name or GUID")?;
        }
        _ => return Err(format!("{USAGE}; not {command}")),
    }
    let bytes = binz::pinned_bytes(binz_path, pins).map_err(|error| error.to_string())?;
    let library = Library::parse(&bytes).map_err(|error| error.to_string())?;
    match command.as_str() {
        "show" => library.show(rest),
        "grep" => library.grep(&rest[0]),
        "refs" => library.refs(&rest[0]),
        _ => library.granted_by(&rest[0]),
    }
}

fn parse_areas(list: &str) -> Result<Vec<WriteArea>, String> {
    let mut areas = Vec::new();
    for name in list.split(',') {
        let area = WriteArea::parse(name)
            .ok_or_else(|| format!("{USAGE}; `{name}` is not an area to write"))?;
        if !areas.contains(&area) {
            areas.push(area);
        }
    }
    Ok(areas)
}

/// `--write`: the check before, the writer, the check after (for the log), and what it did as text.
/// `wanted` is the areas and, for `option-lists`, the selections `--only` names.
fn write_run(
    wanted: (&[WriteArea], Option<&std::collections::BTreeSet<String>>),
    dry_run: bool,
    log_path: Option<&Path>,
    data_dir: &Path,
    binz_path: &Path,
    pins: &Pins,
    known: &Known,
) -> Result<String, String> {
    let dump = binz::read(binz_path, pins).map_err(|error| error.to_string())?;
    let data = seeds::check_game_data_files(data_dir).map_err(|error| error.to_string())?;
    let before = check(pins, &dump, &data, known).map_err(|error| error.to_string())?;
    let validate = |path: &Path| {
        seeds::check_game_data_files(path)
            .map(|_| ())
            .map_err(|error| error.to_string())
    };
    let (areas, only) = wanted;
    let facts = Facts::embedded().map_err(|error| error.to_string())?;
    let outcome = write(&dump, data_dir, areas, only, dry_run, &facts, validate)
        .map_err(|error| error.to_string())?;
    let mut text = String::new();
    if dry_run {
        text.push_str("dry run: nothing written\n");
    }
    for area in areas {
        let (name, kinds) = (area.name(), area.kinds());
        let changes: Vec<_> = outcome
            .changes
            .iter()
            .filter(|c| kinds.contains(&c.kind))
            .collect();
        let files: std::collections::BTreeSet<_> = changes.iter().map(|c| &c.file).collect();
        text.push_str(&format!(
            "{name}: {} fields {}, {} files\n",
            changes.len(),
            if dry_run { "to write" } else { "written" },
            files.len()
        ));
        for change in changes {
            text.push_str(&format!(
                "  {} | {} | was: {} | now: {}\n",
                change.at, change.field, change.old, change.new
            ));
        }
    }
    if !outcome.unexpressed.is_empty() {
        text.push_str("left unwritten, no form the writer expresses:\n");
        for item in &outcome.unexpressed {
            text.push_str(&format!("  {} | {}\n", item.at, item.what));
        }
    }
    if !outcome.to_the_user.is_empty() {
        text.push_str("to the user, the user's facts held, not written:\n");
        for change in &outcome.to_the_user {
            text.push_str(&format!(
                "  {} | {} | was: {} | now: {}\n",
                change.at, change.field, change.old, change.new
            ));
        }
    }
    if outcome.changes.is_empty() && outcome.to_the_user.is_empty() {
        text.push_str("nothing to write\n");
    }
    if let Some(path) = log_path {
        let data = seeds::check_game_data_files(data_dir).map_err(|error| error.to_string())?;
        let after = check(pins, &dump, &data, known).map_err(|error| error.to_string())?;
        std::fs::write(
            path,
            log::tsv(&outcome.changes, &outcome.facts, &before, &after),
        )
        .map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(text)
}
