//! Ticket 150, step 1, on the proof's own Builds: every picker agrees with the walk ([`rules::agreement`]) on every
//! Build `proof record` records (its own, the benchmark's and the full ones, about 2,000). Ignored: it takes far
//! longer than the suite should (an `is_possible` walk per option of every picker). Run it in release:
//!
//! ```text
//! cargo test --release -p proof --test agreement -- --ignored --nocapture
//! ```
//!
//! `AGREEMENT_ONLY=<name substring>` sweeps only the Builds whose name holds it, and `AGREEMENT_THREADS` sets the
//! threads (8 by default). The fast sample, ticket 48's bench Builds, runs in the suite: `libs/seeds`'
//! `game_data::agreement`.

use std::path::Path;
use std::time::Instant;

use proof::{builds, in_parallel};
use rules::agreement::{judge, known_for};

/// The disagreements the sweep finds, as `agreement` writes them, with the Build's name before each. None: a new one is a
/// bug, for the user to decide (ticket 150).
const KNOWN: &[&str] = &[];

#[test]
#[ignore = "sweeps about 2,000 real-data Builds: minutes in release"]
fn every_picker_agrees_with_the_walk_on_the_proof_builds() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/game-data");
    let (engine, data, problems) = proof::engine_on(&dir).expect("the game data reads");
    assert_eq!(problems, [], "the working tree's game data passes `check`");
    let threads = std::env::var("AGREEMENT_THREADS")
        .ok()
        .and_then(|threads| threads.parse().ok())
        .unwrap_or(8)
        .max(1);
    let only = std::env::var("AGREEMENT_ONLY").unwrap_or_default();
    let start = Instant::now();
    let mut all = builds::all(&engine, &data, threads);
    // The pickers answer only for possible Builds; the proof records a few impossible ones on purpose.
    all.retain(|(name, build)| name.contains(&only) && engine.is_possible(build));
    eprintln!("{} Builds in {:?}", all.len(), start.elapsed());
    // `in_parallel` gives each thread one run of Builds, and the full Builds come together at the end: deal them out
    // in turn instead, so each thread gets its share of them.
    let all: Vec<_> = (0..threads)
        .flat_map(|first| all.iter().skip(first).step_by(threads).cloned())
        .collect();
    let judged: Vec<(usize, Vec<String>)> = in_parallel(&all, threads, |(name, build)| {
        let agreement = judge(&engine, build);
        let lines = agreement.lines(name);
        for line in &lines {
            eprintln!("DISAGREEMENT {line}");
        }
        (agreement.judged, lines)
    });
    let options: usize = judged.iter().map(|(options, _)| options).sum();
    let mut found: Vec<String> = judged.into_iter().flat_map(|(_, lines)| lines).collect();
    found.sort();
    eprintln!("{options} options judged in {:?}", start.elapsed());
    // The pinned disagreements of the Builds swept: those `AGREEMENT_ONLY` keeps and the walk finds possible.
    assert_eq!(
        found,
        known_for(KNOWN, &all),
        "{} disagreements",
        found.len()
    );
}
