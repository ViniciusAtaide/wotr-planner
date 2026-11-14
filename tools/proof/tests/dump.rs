//! Ticket 203: `dump` writes the facts behind a record in clear, sorted and one path to a fact, so two dumps line up.

use std::path::Path;
use std::sync::OnceLock;

use proof::dump::{dump, dump_build};
use proof::fingerprint::Mode;
use proof::{Named, builds};

/// The engine and the two smallest possible Builds with a level or two of the proof's corpus (cheap: the tests run in a debug build),
/// by name.
fn corpus() -> &'static (rules::Engine, Vec<Named>) {
    static CORPUS: OnceLock<(rules::Engine, Vec<Named>)> = OnceLock::new();
    CORPUS.get_or_init(|| {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/game-data");
        let (engine, data, _) = proof::engine_on(&dir).expect("the game data reads");
        let mut all = builds::all(&engine, &data, 4);
        all.retain(|(_, build)| build.levels.len() >= 2 && engine.is_possible(build));
        all.sort_by_key(|(name, build)| (build.levels.len(), name.clone()));
        all.truncate(2);
        (engine, all)
    })
}

#[test]
fn a_dump_holds_every_kind_of_fact_sorted_by_path_with_no_path_twice() {
    let (engine, all) = corpus();
    let (_, build) = &all[0];
    let lines = dump_build(engine, build, Mode::Light);
    let paths: Vec<&str> = lines.iter().map(|(path, _)| path.as_str()).collect();
    assert!(
        paths.windows(2).all(|pair| pair[0] < pair[1]),
        "sorted, each path once"
    );
    for prefix in [
        "build.who",
        "build.possible",
        "sheet.levels.1.class",
        "sheet.levels.2.state.bab",
        "slot.Feat.1.General.0.kind",
        "slot.Feat.1.General.0.open",
        "slot.Feat.1.General.0.total",
        "slot.Feat.1.General.0.order-hash",
        "slot.Feat.1.General.0.reason-hash",
        "slot.Feat.1.General.0.option/power-attack",
        "classes.1.option/",
        "paths.option/",
        "switches.option/",
        "removable/",
        "totals.1.",
        "caster.1.",
    ] {
        assert!(
            paths.iter().any(|path| path.starts_with(prefix)),
            "no line at {prefix}"
        );
    }
    assert!(
        !paths.iter().any(|path| path.starts_with("allows/")),
        "a light dump has no allows"
    );
    // A lock reason is its text, an open option `open`.
    let value = |path: &str| {
        lines
            .iter()
            .find(|(line, _)| line == path)
            .map(|(_, value)| value.as_str())
    };
    assert_eq!(
        value("slot.Feat.1.General.0.option/power-attack"),
        Some("locked: Needs Str 13 (have 10)")
    );
    assert!(
        lines
            .iter()
            .any(|(path, value)| path.contains(".option/") && value == "open"),
        "an open option"
    );
    assert_eq!(dump_build(engine, build, Mode::Light), lines, "stable");
}

#[test]
fn a_full_dump_adds_what_allows_says_of_each_edit() {
    let (engine, all) = corpus();
    let (_, build) = &all[0];
    let full = dump_build(engine, build, Mode::Full);
    let allows: Vec<&String> = full
        .iter()
        .filter(|(path, _)| path.starts_with("allows/"))
        .map(|(_, value)| value)
        .collect();
    assert!(!allows.is_empty());
    assert!(allows.iter().all(|value| *value == "0" || *value == "1"));
    assert!(dump_build(engine, build, Mode::Light).len() < full.len());
}

#[test]
fn builds_are_written_by_name_in_the_order_a_dump_is_read() {
    let (engine, all) = corpus();
    let mut reversed = all.clone();
    reversed.reverse();
    let text = |builds: Vec<Named>| {
        let mut text = String::new();
        dump(engine, builds, 2, Mode::Light, |chunk| {
            text.push_str(chunk);
        });
        text
    };
    let (a, b) = (text(all.clone()), text(reversed));
    assert_eq!(a, b, "the order the Builds come in doesn't matter");
    let names: Vec<&str> = a
        .lines()
        .map(|line| line.split('\t').next().unwrap())
        .collect();
    let mut deduped = names.clone();
    deduped.dedup();
    let mut sorted: Vec<&str> = all.iter().map(|(name, _)| name.as_str()).collect();
    sorted.sort_unstable();
    assert_eq!(deduped, sorted, "by name, a Build's lines together");
    assert!(
        a.lines().all(|line| line.matches('\t').count() >= 2),
        "build, path and value"
    );
}
