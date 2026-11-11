//! Ticket 150, step 1, on the real data: every picker agrees with the walk ([`rules::agreement`]) on ticket 48's
//! bench Builds (`tools/proof/bench-links.tsv`). The suite judges their first ten levels; the whole Builds,
//! and the proof's 2,000 Builds (`tools/proof`'s `agreement` test), are ignored for their time. Run them in release:
//!
//! ```text
//! cargo test --release -p seeds --test game_data agreement -- --ignored --nocapture
//! ```

use std::time::{Duration, Instant};

use rules::agreement::{judge_all, known_for};
use rules::{Build, Edit, Engine, Pick, SlotKind};

use crate::support::*;

/// The disagreements the bench Builds find, as `agreement` writes them, with the Build's name before each. None: a new one
/// is a bug, for the user to decide (ticket 150).
const KNOWN: &[&str] = &[];

/// How many levels the suite keeps of each bench Build: about 5 s in a debug build, where the whole Builds take about
/// 40 s (a walk per option of every picker).
const SAMPLE_LEVELS: usize = 10;

#[test]
fn every_picker_agrees_with_the_walk_on_the_bench_builds_first_levels() {
    let engine = engine();
    let builds: Vec<(String, Build)> = bench()
        .into_iter()
        .map(|(name, build)| {
            (
                format!("{name} to {SAMPLE_LEVELS}"),
                cut(&engine, build, SAMPLE_LEVELS),
            )
        })
        .collect();
    agrees(&engine, &builds);
}

#[test]
#[ignore = "six whole real-data Builds: about 40 s in a debug build"]
fn every_picker_agrees_with_the_walk_on_the_whole_bench_builds() {
    agrees(&engine(), &bench());
}

/// Every picker agrees with the walk on the Builds, but for the [`KNOWN`] disagreements.
fn agrees(engine: &Engine, builds: &[(String, Build)]) {
    let sweep = judge_all(engine, builds);
    assert_eq!(
        sweep.lines,
        known_for(KNOWN, builds),
        "{} disagreements",
        sweep.lines.len()
    );
}

/// Ticket 150's design question 1, option C: what a feat slot's candidates cost when each is judged by a walk of its
/// own, against the picker's one walk to the slot and its shortcuts. Prints the times; asserts nothing about them.
#[test]
#[ignore = "a timing, for ticket 150's Findings: run in release"]
fn a_walk_per_candidate_against_the_feat_picker() {
    let engine = engine();
    for (name, build) in bench() {
        let sheet = engine.evaluate(&build);
        let feat_slots: Vec<_> = sheet
            .levels
            .iter()
            .flat_map(|level| &level.slots)
            .filter(|slot| matches!(slot.kind, SlotKind::Feat(_)))
            .collect();
        for slot in [feat_slots.first(), feat_slots.last()]
            .into_iter()
            .flatten()
        {
            let id = &slot.id;
            let (options, picker) = timed(|| engine.options(&build, id).expect("a slot"));
            let edits: Vec<Edit> = options
                .iter()
                .map(|option| Edit::Pick {
                    slot: id.clone(),
                    pick: option.pick.clone(),
                })
                .collect();
            let open = options
                .iter()
                .filter(|option| option.status.is_ok())
                .count();
            let feats = options
                .iter()
                .filter(|option| matches!(option.pick, Pick::Feat { .. }))
                .count();
            // Option C as `allows` does it today: a walk to the slot's level, then one from there, per candidate.
            let (_, allows) = timed(|| {
                edits
                    .iter()
                    .filter(|edit| engine.allows(&build, edit))
                    .count()
            });
            // Option C as one walk of the edited Build per candidate (it stops at the first choice that fails).
            let (_, walks) = timed(|| {
                edits
                    .iter()
                    .filter(|edit| {
                        let mut edited = build.clone();
                        engine.apply(&mut edited, edit);
                        engine.is_possible(&edited)
                    })
                    .count()
            });
            eprintln!(
                "OPTION-C {name} {id:?}: {} candidates ({feats} feats, {open} open), {} levels; picker {picker:?}, \
                 allows each {allows:?}, a walk each {walks:?}",
                options.len(),
                build.levels.len(),
            );
        }
    }
}

fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let out = work();
    (out, start.elapsed())
}

/// The Build cut to its first `levels` levels, as Remove level takes the last ones off.
fn cut(engine: &Engine, mut build: Build, levels: usize) -> Build {
    while build.levels.len() > levels {
        engine.apply(&mut build, &Edit::RemoveLevel);
    }
    build
}
