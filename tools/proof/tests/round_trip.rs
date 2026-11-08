//! A record-and-compare round trip on a handful of Builds of the real game data, so the harness can't rot unnoticed
//! (the full proof is a command: it takes minutes).

use std::path::Path;

use proof::{Named, compare::compare, links};
use rules::{
    Alignment, Build, ClassKey, Engine, GameData, LevelUp, Pick, RaceKey, SelectionOptionKey,
    SlotId,
};

fn engine() -> (Engine, GameData) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/game-data");
    proof::engine_on(&dir).expect("the game data reads")
}

fn character(class: &str, levels: usize) -> Build {
    let mut build = Build::default();
    build.start.race = Some(RaceKey::new("human"));
    build.start.alignment = Some(Alignment::ChaoticNeutral);
    build
        .class_choices
        .insert(ClassKey::new(class), Default::default());
    for _ in 0..levels {
        build.levels.push(LevelUp::new(ClassKey::new(class)));
    }
    build
}

/// A blank Build, a fighter 2, a witch 2 holding a hex, one no sheet holds (a pick in a slot it lacks), and a cleric of
/// Pharasma.
fn builds() -> Vec<Named> {
    let mut witch = character("witch", 2);
    witch.picks.insert(
        SlotId::Selection { level: 1, index: 0 },
        Pick::option(SelectionOptionKey::new("witch/hex/slumber")),
    );
    let mut impossible = character("fighter", 1);
    impossible.picks.insert(
        SlotId::Selection { level: 1, index: 3 },
        Pick::option(SelectionOptionKey::new("witch/hex/slumber")),
    );
    vec![
        ("blank".to_owned(), Build::default()),
        ("fighter 2".to_owned(), character("fighter", 2)),
        ("witch 2 slumber".to_owned(), witch),
        ("fighter 1 stray hex".to_owned(), impossible),
        // Ticket 86: a cleric follows a deity, which links and JSON both carry.
        (
            "cleric 1 of pharasma".to_owned(),
            proof::generate::with_faith(
                &character("cleric", 1),
                serde_json::json!({ "Deity": "pharasma" }),
            ),
        ),
    ]
}

#[test]
fn record_export_and_compare_round_trip() {
    let (engine, data) = engine();
    let builds = builds();
    let record = proof::record(&engine, &builds, 2).join("\n");

    // A record agrees with itself.
    let same = compare(&record, &record).expect("records parse");
    assert!(same.is_empty(), "{:?}", same.lines);

    // Through links, the same Builds and the same record.
    let (text, skipped) = links::export(&data, &builds);
    assert!(skipped.is_empty(), "{skipped:?}");
    let (read, failed) = links::read(&data, &text);
    assert!(failed.is_empty(), "{failed:?}");
    assert_eq!(read, builds);
    assert_eq!(proof::record(&engine, &read, 1).join("\n"), record);

    // Through JSON (ticket 86), the same Builds too.
    let (read, failed) = links::read_json(&links::export_json(&builds));
    assert!(failed.is_empty(), "{failed:?}");
    assert_eq!(read, builds);

    // A side with a Build changed (a third fighter level) and one gone.
    let mut changed = builds.clone();
    changed[1]
        .1
        .levels
        .push(LevelUp::new(ClassKey::new("fighter")));
    changed.retain(|(name, _)| name != "blank");
    let other = proof::record(&engine, &changed, 2).join("\n");
    let comparison = compare(&record, &other).expect("records parse");
    let differs: Vec<&String> = comparison
        .lines
        .iter()
        .filter(|line| !line.starts_with("slot\t"))
        .collect();
    assert_eq!(differs.len(), 2, "{:?}", comparison.lines);
    assert_eq!(differs[0], "only in A\tblank");
    assert!(
        differs[1].starts_with("differs\tfighter 2\t") && differs[1].contains("slots"),
        "{differs:?}"
    );
    // The third level's general feat slot is new; every slot line is the fighter's.
    assert!(
        comparison.lines.contains(
            &"slot\tfighter 2\tFeat { level: 3, source: General, index: 0 }\tonly in B".to_owned()
        ),
        "{:?}",
        comparison.lines
    );
    assert!(
        comparison
            .lines
            .iter()
            .filter(|line| line.starts_with("slot\t"))
            .all(|line| line.starts_with("slot\tfighter 2\t"))
    );
}
