//! The behaviour proof: proves that a change to `rules` or to `libs/game-data` changes no behaviour but the
//! intended one, by fingerprinting about 2,000 Builds on each side and comparing the two records.
//!
//! **Run it through `scripts/proof.sh`**, which builds this crate in release mode once (in a shared
//! `CARGO_TARGET_DIR` outside the repo), records both sides and compares them:
//!
//! - `scripts/proof.sh data <out-dir> [<ref>]`: a game-data change. The working tree's harness records `<ref>`'s
//!   `libs/game-data` (default `HEAD`) as side A and the working tree's as side B.
//! - `scripts/proof.sh rules <out-dir> [<ref>]`: a rules change. The harness compiled against `<ref>`'s crates
//!   (default `HEAD`) is side A and the one compiled against the working tree's is side B, both on the working tree's
//!   game data. The working tree's `export` supplies the Builds as links, so both sides record the same Builds.
//! - `scripts/proof.sh both <out-dir> [<ref>]`: a rules change with its game data, which may change the link layout
//!   too (ticket 86). As `rules`, but side A records `<ref>`'s own game data (which its crates can read), and the
//!   working tree's `export --json` supplies the Builds as JSON ([`links::read_json`]): a side whose Build start holds
//!   no faith drops it, and leaves out the deity Builds (`t86 …`, full ones too).
//!
//! Either takes `-- <record options>` last (`--only <name substring>`, `--threads N`). A full record takes about 4
//! minutes on 8 threads. The outputs (`A.jsonl`, `B.jsonl`, `compare.txt` and the logs) go to `<out-dir>`, which must
//! be outside the repo. The script exits non-zero when the comparison isn't empty.
//!
//! The binary's commands:
//!
//! - `proof record <game-data dir> [--links <file> | --builds <file>] [--threads N] [--only <name substring>]
//!   [--fold-values <key>,…]` writes one JSON line per Build ([`fingerprint`]), in a stable order. Without `--links`
//!   or `--builds` it records the harness's own Builds ([`builds`]); with one, exactly the Builds of that file (links,
//!   or JSON lines). `--fold-values` folds each listed item's options into one per slot (ticket 125: items that came
//!   to take a value).
//! - `proof compare <a.jsonl> <b.jsonl>` lists the Builds only in one side and, per Build that differs, the fields that
//!   differ and, for slots, the slot ids ([`compare`]). It exits 1 when anything differs.
//! - `proof export <game-data dir> [--json] [--threads N] [--only <name substring>]` writes the harness's own Builds
//!   as share links, one `name<TAB>link` line each, for `record --links`, or with `--json` as `name<TAB>json` lines
//!   for `record --builds` ([`links`]).
//!
//! Ticket 86: a deity-bound class needs a faith, so each Build that's impossible without one gets the first of
//! atheism and the deities it's possible with ([`generate::faithful`]), set through serde so the harness still
//! compiles against crates from before deities.
//!
//! Ticket 94: an alignment needs a faith first, so the Builds the groups start from are atheists (a deity-bound class
//! still gets its deity), and a benchmark link with an alignment and no faith opens as an atheist.
//!
//! Ticket 95: the fingerprint reads the engine's held choices ([`rules::Engine::held_choices`]), so the harness
//! doesn't compile against crates from before it. A `rules` proof against such a ref runs from a copy of the repo
//! whose `tools/proof` has the ref's `fingerprint.rs`.
//!
//! Ticket 96: the battery is the engine's Build start and level options and steps ([`fingerprint::controls`]), so
//! the harness doesn't compile against crates from before it either (the same copy, with the ref's `fingerprint.rs`
//! changed only to hash the options without their edits and to drop `Engine::allows_many`); and each option's status
//! is checked against `allows`. The options hash as they did before (without their edits), so older records compare.
//!
//! A ticket that needs a special breakdown (only the Builds with some class, say) filters `compare`'s output or the
//! records themselves (each line names its Build's classes in `who`); the fingerprint keeps only what isn't tied to
//! one ticket.

pub mod builds;
pub mod compare;
pub mod fingerprint;
pub mod generate;
pub mod links;

use std::path::Path;

use rules::{Build, Engine, GameData};

/// A Build with the name the records know it by.
pub type Named = (String, Build);

/// The engine on the game data under `dir`, and that game data in the canonical order the engine holds it in. The
/// harness keeps its own copy for the lists it walks (every race, skill, class…): the engine doesn't hand out its
/// data. The files reader already returns it canonical. The data must pass [`GameData::check`], as the engine's
/// must.
pub fn engine_on(dir: &Path) -> Result<(Engine, GameData), seeds::Error> {
    let data = seeds::check_game_data_files(dir)?;
    let copy = GameData::clone(&data);
    Ok((Engine::new(data), copy))
}

/// Runs `work` on each Build on `threads` threads, keeping the Builds' order.
pub fn in_parallel<T: Send>(
    builds: &[Named],
    threads: usize,
    work: impl Fn(&Named) -> T + Sync,
) -> Vec<T> {
    if builds.is_empty() {
        return Vec::new();
    }
    let work = &work;
    std::thread::scope(|scope| {
        let handles: Vec<_> = builds
            .chunks(builds.len().div_ceil(threads.max(1)))
            .map(|chunk| scope.spawn(move || chunk.iter().map(work).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a proof thread panicked"))
            .collect()
    })
}

/// The record of `builds`: one JSON line each, in their order.
pub fn record(engine: &Engine, builds: &[Named], threads: usize) -> Vec<String> {
    in_parallel(builds, threads, |(name, build)| {
        fingerprint::fingerprint(engine, name, build).to_string()
    })
}
