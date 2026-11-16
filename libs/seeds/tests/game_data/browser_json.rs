//! Ticket 244, review: the flat `GameData` as the browser receives it. The server serialises it to JSON
//! (`apps/backend/src/game_data.rs`, `GameDataFile::new`) and the browser reads that JSON back and checks it
//! (`libs/ui/src/game_data.rs`, `load`): every catalog comes through but what the JSON leaves out.

use rules::GameData;

use crate::support::DATA;

/// `data` without what the JSON leaves out (`#[serde(skip_serializing)]`): the spells, the spell lists and every
/// entry's blueprint.
fn as_serialised(mut data: GameData) -> GameData {
    data.spells.clear();
    data.spell_lists.clear();
    macro_rules! no_blueprints {
        ($($catalog:ident),*) => {
            $(for entry in &mut data.$catalog {
                entry.blueprint = None;
            })*
        };
    }
    no_blueprints!(
        races,
        subraces,
        weapons,
        feats,
        classes,
        prestige_classes,
        archetypes,
        features,
        selections,
        options,
        mythic_abilities,
        mythic_feats,
        gear,
        mythic_paths,
        backgrounds,
        deities
    );
    data
}

#[test]
fn the_game_data_comes_through_the_browsers_json_whole_but_for_what_it_leaves_out() {
    let json = serde_json::to_vec(&*DATA).expect("the game data serialises to JSON");
    let read: GameData = serde_json::from_slice(&json).expect("the browser reads the JSON");

    assert_eq!(read.check(), []);
    assert!(read.blueprints().iter().all(|(_, guid)| guid.is_none()));
    assert!(read.spells.is_empty() && read.spell_lists.is_empty());
    let expected = as_serialised(DATA.clone());
    // Catalog by catalog, so a failure names the one that differs.
    assert_eq!(read.features, expected.features);
    assert_eq!(read.selections, expected.selections);
    assert_eq!(read.options, expected.options);
    assert_eq!(read.archetypes, expected.archetypes);
    assert_eq!(read.classes, expected.classes);
    assert_eq!(read, expected);
}
