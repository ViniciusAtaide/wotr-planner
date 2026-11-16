//! Applying changes (ticket 293b): the write fixture's tree reads as game data, a change writes only its field, a
//! planner file is refused with nothing written, and a failed check puts every file back.

use std::path::{Path, PathBuf};

use game_check::write::span::Value;
use game_check::write::{Change, WriteError, apply, entry_file};
use rules::DataAt;

use crate::support::data_copy;

fn change(file: &str, field: &'static str, value: Option<Value>) -> Change {
    Change {
        at: DataAt::Gear(rules::GearKey::new("cloak-of-test")),
        file: PathBuf::from(file),
        field,
        kind: "gear",
        value,
        old: String::new(),
        new: String::new(),
    }
}

fn components(text: &str) -> Option<Value> {
    Some(Value::List(vec![text.to_owned()]))
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
}

#[test]
fn the_write_fixture_reads_as_game_data() {
    let dir = data_copy();
    let data = seeds::read_game_data_files(dir.path()).expect("the tree reads");
    assert_eq!(data.classes.len(), 1);
    assert_eq!(data.prestige_classes.len(), 1);
    assert_eq!(data.archetypes.len(), 2);
    assert_eq!(data.gear.len(), 7, "six entries and the planner's trinket");
    let class = DataAt::Class(rules::ClassKey::new("warden"));
    assert_eq!(entry_file(&data, &class), Path::new("classes/warden.ron"));
    let prestige = DataAt::Class(rules::ClassKey::new("sentinel"));
    assert_eq!(
        entry_file(&data, &prestige),
        Path::new("prestige/sentinel.ron")
    );
    let archetype = DataAt::Archetype(rules::ArchetypeKey::new("warden/stoic"));
    assert_eq!(
        entry_file(&data, &archetype),
        Path::new("archetypes/warden/stoic.ron")
    );
    let gear = DataAt::Gear(rules::GearKey::new("cloak-of-test"));
    assert_eq!(
        entry_file(&data, &gear),
        Path::new("gear/cloak-of-test.ron")
    );
}

#[test]
fn applying_a_change_writes_only_its_field() {
    let dir = data_copy();
    let before = read(dir.path(), "gear/cloak-of-test.ron");
    let new = "GearStat(stat: AllSaves, value: 3, descriptor: Resistance)";
    let applied = apply(
        dir.path(),
        &[change(
            "gear/cloak-of-test.ron",
            "components",
            components(new),
        )],
        |path| {
            seeds::read_game_data_files(path)
                .map(|_| ())
                .map_err(|e| e.to_string())
        },
    )
    .unwrap();
    assert_eq!(applied.files, [PathBuf::from("gear/cloak-of-test.ron")]);
    let after = read(dir.path(), "gear/cloak-of-test.ron");
    assert_eq!(
        after,
        before.replace(
            "GearStat(stat: AllSaves, value: 2, descriptor: Resistance)",
            new
        )
    );
    // The comment above and the other fields survive.
    assert!(after.starts_with("// Planted: +3 in the game.\n"));
}

#[test]
fn a_planner_file_is_refused_and_nothing_is_written() {
    let dir = data_copy();
    let cloak = read(dir.path(), "gear/cloak-of-test.ron");
    let trinket = read(dir.path(), "planner/gear/test-trinket.ron");
    let error = apply(
        dir.path(),
        &[
            change("gear/cloak-of-test.ron", "components", components("X")),
            change(
                "planner/gear/test-trinket.ron",
                "components",
                components("Y"),
            ),
        ],
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Planner(_)), "{error}");
    assert_eq!(read(dir.path(), "gear/cloak-of-test.ron"), cloak);
    assert_eq!(read(dir.path(), "planner/gear/test-trinket.ron"), trinket);
    // A file that isn't there is refused too.
    let error = apply(
        dir.path(),
        &[change("gear/nothing.ron", "components", components("X"))],
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Missing(_)), "{error}");
}

#[test]
fn a_failed_validation_restores_every_file() {
    let dir = data_copy();
    let files = ["gear/cloak-of-test.ron", "gear/headband-of-test.ron"];
    let before: Vec<String> = files.iter().map(|file| read(dir.path(), file)).collect();
    let error = apply(
        dir.path(),
        &[
            change(files[0], "components", components("A")),
            change(files[1], "components", components("B")),
        ],
        |path| {
            // The files are written when the check runs.
            let text = std::fs::read_to_string(path.join(files[1])).unwrap();
            assert!(text.contains("components: [B]"), "{text}");
            Err("a planted failure".to_owned())
        },
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Invalid(_)), "{error}");
    for (file, text) in files.iter().zip(before) {
        assert_eq!(read(dir.path(), file), text, "{file}");
    }
}
