//! The same dump and data give the same bytes, whatever order the records and entries come in.

use game_check::{Known, check};

use crate::support::{binz, dump, planner, records, report};

#[test]
fn a_shuffled_dump_and_data_give_a_byte_identical_report() {
    let expected = report();
    // The header names the pinned dump's hash: the shuffled file's report is built with the original's pins.
    let (pins, _) = dump(&binz(&records(), 2));
    let mut shuffled = records();
    shuffled.reverse();
    shuffled.rotate_left(7);
    let (_, dump) = dump(&binz(&shuffled, 3));
    let mut data = planner();
    data.feats.reverse();
    data.gear.reverse();
    data.mythic_abilities.reverse();
    data.mythic_feats.reverse();
    data.features.reverse();
    // A selection's shared options are a set: reversing the selections moves an option's own selection to the other
    // side of the one that shares it.
    data.selections.reverse();
    data.options.reverse();
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        for row in &mut class.levels {
            row.features.reverse();
            row.selections.reverse();
        }
    }
    for archetype in &mut data.archetypes {
        archetype.levels.reverse();
    }
    data.archetypes.reverse();
    data.classes.reverse();
    data.prestige_classes.reverse();
    for path in &mut data.mythic_paths {
        path.ranks.reverse();
        for row in &mut path.ranks {
            row.features.reverse();
            row.selections.reverse();
        }
    }
    data.mythic_paths.reverse();
    let shuffled = check(&pins, &dump, &data, &Known::default()).unwrap();
    assert_eq!(shuffled.text(), expected.text());
    assert!(!expected.text().is_empty());
}

#[test]
fn the_report_holds_no_path_or_time() {
    let text = report().text().to_owned();
    for path in ["/tmp/", "/var/", "/Users/", env!("CARGO_MANIFEST_DIR")] {
        assert!(!text.contains(path), "{path} in:\n{text}");
    }
    assert!(
        !text.contains(" s\n") && !text.contains(" ms") && !text.contains("seconds"),
        "{text}"
    );
}
