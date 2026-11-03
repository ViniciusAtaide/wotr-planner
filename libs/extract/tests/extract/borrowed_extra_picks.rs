//! Ticket 64: where a class (or its archetype) has slots of a selection, by its rows in force, and how an archetype
//! meets an Extra feat's prerequisite from its first slot.

use extract::borrowed_extra_picks::{from_its_first_slot, slot_levels};
use rules::{ArchetypeKey, ClassKey, Prerequisite};

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    seeds::read_game_data_files(&fixture).unwrap()
}

#[test]
fn a_slot_level_is_a_row_in_force_holding_the_selection() {
    let data = fixture();
    let sorcerer = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "sorcerer")
        .unwrap();
    // The base class's own rows.
    assert_eq!(slot_levels(sorcerer, None, "sorcerer/power"), [2]);
    // An archetype's row replaces the base row's grants: the Seeker's row 2 holds its own choice instead.
    assert_eq!(
        slot_levels(sorcerer, Some("sorcerer/seeker"), "sorcerer/power"),
        Vec::<u8>::new()
    );
    assert_eq!(
        slot_levels(sorcerer, Some("sorcerer/seeker"), "sorcerer/seeker/choice"),
        [2]
    );
    assert_eq!(
        slot_levels(sorcerer, None, "sorcerer/seeker/choice"),
        Vec::<u8>::new()
    );
}

#[test]
fn an_archetype_meets_the_prerequisite_from_its_first_slot() {
    assert_eq!(
        from_its_first_slot("magus", "magus/hexcrafter", 6),
        Prerequisite::All(vec![
            Prerequisite::Archetype(ArchetypeKey::new("magus/hexcrafter")),
            Prerequisite::ClassLevel {
                class: ClassKey::new("magus"),
                level: 6,
            },
        ])
    );
}
