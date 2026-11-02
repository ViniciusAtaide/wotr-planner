//! Options with rows by option level (ticket 30): which cells of an option's table are its own, how the guide's
//! lists of names read, which report items they settle, and how the files write them.

use std::collections::BTreeMap;

use extract::option_levels::{name_list, own_cells, settled};
use extract::tables::parse_table;

fn table(rows: &[&[&str]]) -> Vec<extract::tables::Row> {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|cell| (*cell).to_owned()).collect())
        .collect();
    parse_table(&cells[0], &cells[1..]).unwrap()
}

fn cells(pairs: &[(u8, &[&str])]) -> BTreeMap<u8, Vec<String>> {
    pairs
        .iter()
        .map(|(level, cells)| (*level, cells.iter().map(|c| (*c).to_owned()).collect()))
        .collect()
}

#[test]
fn an_options_own_cells_are_those_the_familys_tables_dont_share() {
    let header: &[&str] = &["Level", "Feats", "Special 1", "Special 2"];
    let tables = [
        table(&[
            header,
            &["0", "-", "Proficiencies", "Class Skill: Athletics"],
            &["1", "Feat", "Bonus Feat", "Grow Claws"],
            &["3", "Feat", "Bloodline spell - Cause Fear", "Resistances"],
            &["7", "Feat", "Bloodline Feat", "-"],
        ]),
        table(&[
            header,
            &["0", "-", "Proficiencies", "Class Skill"],
            &["1", "Feat", "Bonus Feat", "Arcane Bond"],
            &["3", "Feat", "Bloodline spell - Magic Missile", "-"],
            &["7", "Feat", "Bloodline Feat", "-"],
            &["9", "Feat", "-", "Bonus spell"],
        ]),
    ];
    let own = own_cells(&tables);
    // Row 0 folds into 1; the shared cells and the spells are the class's or left out.
    assert_eq!(
        own[0],
        cells(&[
            (1, &["Class Skill: Athletics", "Grow Claws"]),
            (3, &["Resistances"])
        ])
    );
    assert_eq!(own[1], cells(&[(1, &["Class Skill", "Arcane Bond"])]));
}

#[test]
fn a_cell_shared_by_some_options_only_is_each_ones_own() {
    let header: &[&str] = &["Level", "Special 1"];
    let tables = [
        table(&[header, &["2", "Trick Magic"]]),
        table(&[header, &["2", "Trick Magic"]]),
        table(&[header, &["2", "Fake Enhancement"]]),
    ];
    let own = own_cells(&tables);
    assert_eq!(own[0], cells(&[(2, &["Trick Magic"])]));
    assert_eq!(own[2], cells(&[(2, &["Fake Enhancement"])]));
}

#[test]
fn a_list_of_names_reads_commas_and_the_last_and() {
    assert_eq!(
        name_list(" Bull Rush, Cleave, Skill Focus (Lore: Religion) and Sunder Armor."),
        [
            "Bull Rush",
            "Cleave",
            "Skill Focus (Lore: Religion)",
            "Sunder Armor"
        ]
    );
    assert_eq!(name_list("Improved Critical"), ["Improved Critical"]);
}

#[test]
fn the_option_tables_and_the_lines_they_state_are_settled_report_items() {
    for at in [
        "sorcerer#abyssal",
        "bloodrager#hag-bloodrager",
        "wizard#universalist",
        "cavalier#order-of-the-lion",
        "arcanist#living-deity",
        "feats#disruptive",
        "character-creation#bloodlines",
    ] {
        assert!(settled(at, ""), "{at}");
    }
    assert!(settled(
        "prestige-classes#dragon-disciple",
        "Must have a Draconic bloodline or no known bloodline"
    ));
    assert!(!settled(
        "prestige-classes#dragon-disciple",
        "Minimum character level: 6"
    ));
    assert!(!settled("sorcerer#crossblooded", "Bonus Feat"));
    assert!(!settled("prestige-classes#winter-witch", ""));
}

#[test]
fn the_files_write_options_with_rows_and_read_them_back() {
    // The seeds crate's fixture holds every shape: option rows with features, feats, selections, a feat list and
    // class skills (and a row of class skills only), option levels, skipped levels, list-only and
    // only-without-option selections, pools, a replaced row, an option-row feature, a daily choice, spontaneous
    // casting, an option-list slot and a mythic ability's grants.
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    let data = seeds::read_game_data_files(&fixture).unwrap();
    let out = tempfile::tempdir().unwrap();
    for (path, text) in extract::ron_out::files(&data) {
        let target = out.path().join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, text).unwrap();
    }
    assert_eq!(seeds::read_game_data_files(out.path()).unwrap(), data);
    let sorcerer = std::fs::read_to_string(out.path().join("classes/sorcerer.ron")).unwrap();
    // An option with rows is a block, a row per line.
    assert!(
        sorcerer.contains(
            "                    levels: [\n                        (level: 1, features: \
             [\"sorcerer/bloodline/abyssal/claws\"], feat_list: \"sorcerer/bloodline/abyssal-feats\", \
             class_skills: [\"athletics\"]),\n"
        ),
        "{sorcerer}"
    );
}
