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
//!
//! Either takes `-- <record options>` last (`--only <name substring>`, `--threads N`). A full record takes about 4
//! minutes on 8 threads. The outputs (`A.jsonl`, `B.jsonl`, `compare.txt` and the logs) go to `<out-dir>`, which must
//! be outside the repo. The script exits non-zero when the comparison isn't empty.
//!
//! The binary's commands:
//!
//! - `proof record <game-data dir> [--links <file>] [--threads N] [--only <name substring>]` writes one JSON line per
//!   Build ([`fingerprint`]), in a stable order. Without `--links` it records the harness's own Builds ([`builds`]);
//!   with it, exactly the Builds of that file.
//! - `proof compare <a.jsonl> <b.jsonl>` lists the Builds only in one side and, per Build that differs, the fields that
//!   differ and, for slots, the slot ids ([`compare`]). It exits 1 when anything differs.
//! - `proof export <game-data dir> [--threads N] [--only <name substring>]` writes the harness's own Builds as share
//!   links, one `name<TAB>link` line each, for `record --links` ([`links`]).
//!
//! A ticket that needs a special breakdown (only the Builds with some class, say) filters `compare`'s output or the
//! records themselves (each line names its Build's classes in `who`); the fingerprint keeps only what isn't tied to
//! one ticket.

pub mod builds;
pub mod compare;
pub mod fingerprint;
pub mod generate;
pub mod links;

use rules::{Build, Engine};

/// A Build with the name the records know it by.
pub type Named = (String, Build);

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
