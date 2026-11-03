//! Ticket 62's mythic ability picks: the "holds one of these selections' options" prerequisite.

use extract::mythic_ability_picks::holding_one_of;
use rules::{FeatureSelectionKey, Prerequisite, SelectionOptionKey};

fn fixture() -> rules::GameData {
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../seeds/tests/fixtures/game-data");
    seeds::read_game_data_files(&fixture).unwrap()
}

#[test]
fn holding_one_of_takes_each_option_once_shared_ones_included_in_key_order() {
    let data = fixture();
    // The Seeker's bloodline shares the sorcerer's two; an archetype's selection is found as a class's is.
    let held = holding_one_of(
        &data,
        &[
            FeatureSelectionKey::new("sorcerer/seeker/bloodline"),
            FeatureSelectionKey::new("sorcerer/bloodline"),
        ],
    );
    let option = |key: &str| Prerequisite::SelectionOption(SelectionOptionKey::new(key));
    assert_eq!(
        held,
        Prerequisite::Any(vec![
            option("sorcerer/bloodline/abyssal"),
            option("sorcerer/bloodline/draconic-red"),
        ])
    );
    // A selection the data lacks offers nothing.
    assert_eq!(
        holding_one_of(&data, &[FeatureSelectionKey::new("witch/hex")]),
        Prerequisite::Any(Vec::new())
    );
}
