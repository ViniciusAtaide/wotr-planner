//! The game check (ticket 238): each game-data entry that names a blueprint, compared with that blueprint in the
//! game's pinned dump.
//!
//! [`binz::read`] refuses a dump that isn't the pinned one ([`Pins`], `scripts/game-sources.pins`) before reading
//! it; [`check`] compares the areas of [`Area`] and builds the [`Report`]: plain text, the same bytes for the same
//! dump and data, which names blueprints by asset name and GUID and never prints the game's text.

use std::fmt;

use rules::GameData;

mod areas;
pub mod binz;
mod dump;
pub mod known;
pub mod lookup;
mod pins;
mod planner;
mod report;

pub use dump::Dump;
pub use known::Known;
pub use lookup::Library;
pub use pins::Pins;
pub use report::{Area, Report};

use areas::Ctx;
use planner::Planner;
use report::Out;

/// Compares the data with the dump, area by area. A blueprint a comparison read whose JSON doesn't parse is an
/// error, naming it: its entries would otherwise count as missing from the dump.
pub fn check(pins: &Pins, dump: &Dump, data: &GameData, known: &Known) -> Result<Report, Error> {
    let planner = Planner::new(data);
    let ctx = Ctx {
        dump,
        planner: &planner,
    };
    let mut out = Out::default();
    areas::mapping::compare(&ctx, &mut out);
    areas::prerequisites::compare(&ctx, &mut out);
    areas::class_tables::compare(&ctx, &mut out);
    areas::rows::compare_classes(&ctx, &mut out);
    areas::options::compare(&ctx, &mut out);
    areas::rows::compare_mythic(&ctx, &mut out);
    areas::items::compare(&ctx, &mut out);
    let unparsed = dump.unparsed();
    if !unparsed.is_empty() {
        return Err(Error::Format(format!(
            "JSON that doesn't parse: {}",
            unparsed.join("; ")
        )));
    }
    Ok(Report::build(
        pins,
        dump.len(),
        (planner.entries.len(), planner.with_blueprint()),
        out,
        known,
    ))
}

/// Where the pinned dump is when no `--binz` names it: `blueprints/<asset>` in `checkout` (the workspace the tool was
/// compiled in) when that has one, else in the main checkout. A worktree has no `blueprints/`; the main checkout
/// is found through `git rev-parse --git-common-dir`, which every worktree shares.
pub fn default_dump(checkout: &std::path::Path, asset: &str) -> std::path::PathBuf {
    let own = checkout.join("blueprints").join(asset);
    if own.exists() {
        return own;
    }
    let common = std::process::Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok());
    match common {
        Some(common) => {
            let common = checkout.join(common.trim());
            let main = common.parent().unwrap_or(checkout);
            main.join("blueprints").join(asset)
        }
        None => own,
    }
}

#[derive(Debug)]
pub enum Error {
    Io {
        path: String,
        error: std::io::Error,
    },
    /// The pins file can't be read as pins.
    Pins(String),
    /// The dump isn't the pinned one: nothing was compared.
    Refused {
        what: &'static str,
        expected: String,
        found: String,
    },
    /// The dump isn't a `.binz` the check can read.
    Format(String),
    /// `known.txt` can't be read.
    Known(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, error } => write!(f, "{path}: {error}"),
            Error::Pins(why) => write!(f, "the pins: {why}"),
            Error::Refused {
                what,
                expected,
                found,
            } => write!(
                f,
                "refused: the dump's {what} is {found}, the pin's is {expected}; nothing was compared"
            ),
            Error::Format(why) => write!(f, "the dump: {why}"),
            Error::Known(why) => write!(f, "known.txt: {why}"),
        }
    }
}

impl std::error::Error for Error {}
