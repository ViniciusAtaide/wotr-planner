//! Options with rows by option level, the second group (ticket 31): how a domain's short name, a revelation's level
//! prerequisite and an element's feat names read, and how the files write an option row's feat slots and chosen
//! feats.

use extract::option_families::{domain_short, element_feat, revelation_level};

#[test]
fn a_domain_heading_names_the_domain() {
    assert_eq!(domain_short("air-domain"), "air");
    assert_eq!(domain_short("ice-subdomain"), "ice");
    assert_eq!(domain_short("murder-subdomain"), "murder");
}

#[test]
fn a_revelations_level_prerequisite_reads_either_spelling() {
    assert_eq!(
        revelation_level("Prerequisites: Level 7 You create a 20' radius burst."),
        Some(7)
    );
    assert_eq!(
        revelation_level("Prequisites: Level 11 When casting a spell."),
        Some(11)
    );
    assert_eq!(
        revelation_level("As a move action, you gain a bonus."),
        None
    );
}

#[test]
fn an_elements_feat_names_are_the_feats_names() {
    assert_eq!(element_feat("Lighting Reflexes"), "Lightning Reflexes");
    assert_eq!(
        element_feat("Improved Lighting Reflexes"),
        "Improved Lightning Reflexes"
    );
    assert_eq!(element_feat("Dodge"), "Dodge");
}

#[test]
fn the_files_write_an_option_rows_feat_slots_and_chosen_feats() {
    // The data crate's fixture has a Draconic row with two feat slots and an abyssal power whose row grants a feat
    // for the pick's value.
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    let data = data::read_game_data_files(&fixture).unwrap();
    let files = extract::ron_out::files(&data);
    let sorcerer = &files
        .iter()
        .find(|(path, _)| path.ends_with("classes/sorcerer.ron"))
        .unwrap()
        .1;
    assert!(
        sorcerer.contains(
            "(level: 3, feat_slots: [CombatFeat, OneOf(list: \"sorcerer/bloodline/abyssal-feats\")])"
        ),
        "{sorcerer}"
    );
    assert!(
        sorcerer.contains(
            "(level: 3, feats: [\"toughness\"], chosen_feats: [\"weapon-specialization\"])"
        ),
        "{sorcerer}"
    );
}
