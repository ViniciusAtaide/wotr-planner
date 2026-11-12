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
//! **When:** once per queue of tickets, by the main session, when the queue is merged (`CLAUDE.md`, "Behaviour
//! checks"). A ticket's own check is the snapshot check in `libs/seeds/tests/game_data/snapshots.rs`.
//!
//! Either takes `-- <record options>` last (`--only <name substring>`, `--threads N`, `--light`). A full record takes
//! about 45 seconds a side on 8 threads, after the two release builds. The outputs (`A.jsonl`, `B.jsonl`,
//! `compare.txt` and the logs) go to `<out-dir>`, which must be outside the repo. The script exits non-zero when the
//! comparison isn't empty.
//!
//! **Light mode** (ticket 162, `--light`): the record leaves out the `allows` battery and the consistency and shortcut
//! checks ([`fingerprint::Mode`]), about a third of a record's cost, and keeps every other field: about 29 seconds a
//! side against 45 (the pickers it keeps are most of the rest). Reach for it between the snapshot check (a few dozen Builds, one side) and the full proof:
//! to see a change's reach over the whole corpus early. It misses a change only the full proof sees (a Build whose
//! pickers and sheet agree but where `allows` of some edit differs, and a mismatch between a shortcut and its full
//! answer, which the checks panic on). A light record says so on every line, and [`compare`] refuses to compare it
//! with a full one.
//!
//! The binary's commands:
//!
//! - `proof record <game-data dir> [--links <file> | --builds <file>] [--threads N] [--only <name substring>]
//!   [--fold-values <key>,…] [--light]` writes one JSON line per Build ([`fingerprint`]), in a stable order. Without `--links`
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
//! Ticket 149: the engine takes only checked data (`rules::CheckedGameData`), and the harness reaches it past the
//! check through `rules`' `unchecked` feature ([`engine_on`]), so it records data with problems too, with a warning.
//! `proof.sh` adds the feature to a ref's `rules` from before it (where it does nothing), so the harness still
//! compiles against those crates.
//!
//! Ticket 151: the fingerprint records each level's totals and caster lines, and those two calls lost their Build
//! start, then took a level of the sheet rather than its state. The build script reads which of the three the `rules`
//! crate beside it takes, from the two functions' parameters wherever its sources define them (`cfg(lines_take_start)`,
//! `cfg(lines_take_state)`, or neither), so the harness compiles against crates back to ticket 120, which brought the
//! totals, and fails to build with what it found on anything else; the sheet hashes without the states' new
//! `bonuses_in_force`, so records from before it still compare.
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
use std::sync::atomic::{AtomicUsize, Ordering};

use rules::{Build, DataProblem, Engine, GameData};

/// A Build with the name the records know it by.
pub type Named = (String, Build);

/// The engine on the game data under `dir`, that game data in the canonical order the engine holds it in, and the
/// problems [`GameData::check`] finds in it. The harness keeps its own copy for the lists it walks (every race,
/// skill, class…): the engine doesn't hand out its data. The files reader already returns it canonical.
///
/// Data with problems gets an engine too, past the check (`rules`' `unchecked` feature, ticket 149): side A of a
/// `data` proof is the older data, which fails a check rule its ticket adds together with the data fix, and that
/// change most needs its comparison. The caller warns with the problems. `data.into()` is that way past the check,
/// and `GameData` itself for crates from before [`rules::CheckedGameData`], whose `Engine::new` takes any data.
pub fn engine_on(dir: &Path) -> Result<(Engine, GameData, Vec<DataProblem>), seeds::FilesError> {
    let data = seeds::read_game_data_files(dir)?;
    let problems = data.check();
    Ok((Engine::new(data.clone().into()), data, problems))
}

/// Runs `work` on each Build on `threads` threads, keeping the Builds' order.
///
/// Each thread takes the next Build not yet taken, so the threads finish together: a Build's cost varies a
/// hundredfold (a full level-20 Build with ticket 48's shortcut checks against a level-1 one), and the costly ones
/// come last, so fixed chunks left one thread with six times another's work.
pub fn in_parallel<T: Send>(
    builds: &[Named],
    threads: usize,
    work: impl Fn(&Named) -> T + Sync,
) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let (work, next) = (&work, &next);
    let mut done: Vec<(usize, T)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads.clamp(1, builds.len().max(1)))
            .map(|_| {
                scope.spawn(move || {
                    let mut done = Vec::new();
                    loop {
                        // Relaxed: the counter only hands out indices; the scope's join publishes the results.
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(build) = builds.get(index) else {
                            break done;
                        };
                        done.push((index, work(build)));
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a proof thread panicked"))
            .collect()
    });
    done.sort_unstable_by_key(|(index, _)| *index);
    done.into_iter().map(|(_, result)| result).collect()
}

/// The record of `builds`: one JSON line each, in their order.
pub fn record(
    engine: &Engine,
    builds: &[Named],
    threads: usize,
    mode: fingerprint::Mode,
) -> Vec<String> {
    in_parallel(builds, threads, |(name, build)| {
        fingerprint::fingerprint(engine, name, build, mode).to_string()
    })
}
