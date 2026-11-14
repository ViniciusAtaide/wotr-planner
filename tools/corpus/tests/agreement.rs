//! Ticket 150, step 1, on the corpus: every picker agrees with the walk ([`rules::agreement`]) on every Build of
//! [`corpus::builds::all`] (its own, the benchmark's and the full ones, about 2,000). Ignored: it takes far longer
//! than the suite should (an `is_possible` walk per option of every picker). Run it in release:
//!
//! ```text
//! cargo test --release -p corpus --test agreement -- --ignored --nocapture
//! ```
//!
//! `AGREEMENT_ONLY=<name substring>` sweeps only the Builds whose name holds it, and `AGREEMENT_THREADS` sets the
//! threads (8 by default). The fast sample, ticket 48's bench Builds, runs in the suite: `libs/seeds`'
//! `game_data::agreement`.

use std::time::Instant;

use corpus::{builds, in_parallel};
use rules::agreement::{judge, known_for};

/// The disagreements the sweep finds, as `agreement` writes them, with the Build's name before each. None: a new one is a
/// bug, for the user to decide (ticket 150).
const KNOWN: &[&str] = &[];

#[test]
#[ignore = "sweeps about 2,000 real-data Builds: minutes in release"]
fn every_picker_agrees_with_the_walk_on_the_corpus() {
    let (engine, data) = corpus::engine().expect("the game data reads and passes `check`");
    let threads = corpus::threads("AGREEMENT_THREADS");
    let only = std::env::var("AGREEMENT_ONLY").unwrap_or_default();
    let start = Instant::now();
    let mut all = builds::all(&engine, &data, threads);
    // The pickers answer only for possible Builds; the corpus holds a few impossible ones on purpose.
    all.retain(|(name, build)| name.contains(&only) && engine.is_possible(build));
    eprintln!("{} Builds in {:?}", all.len(), start.elapsed());
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
