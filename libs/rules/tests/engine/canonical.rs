//! Game data has no meaningful order, so its canonical form must not depend on how lists were written.

use rules::{GameData, Prerequisite};

use crate::support::world;

fn reverse_prerequisite(prerequisite: &mut Prerequisite) {
    if let Prerequisite::All(children)
    | Prerequisite::Any(children)
    | Prerequisite::AtLeast(_, children) = prerequisite
    {
        children.reverse();
        children.iter_mut().for_each(reverse_prerequisite);
    }
}

/// The same content with every list it can reach written backwards.
fn reversed(mut data: GameData) -> GameData {
    data.progression.mythic_ascensions.reverse();
    data.races.reverse();
    for race in &mut data.races {
        race.ability_modifiers.reverse();
    }
    data.skills.reverse();
    data.feats.reverse();
    for feat in &mut data.feats {
        if let Some(prerequisite) = &mut feat.prerequisite {
            reverse_prerequisite(prerequisite);
        }
    }
    data.classes.reverse();
    for class in data.all_classes_mut() {
        class.levels.reverse();
        for selection in &mut class.selections {
            selection.options.reverse();
        }
        for archetype in &mut class.archetypes {
            archetype.levels.reverse();
        }
    }
    data.mythic_abilities.reverse();
    data.mythic_feats.reverse();
    data.mythic_paths.reverse();
    for path in &mut data.mythic_paths {
        path.ranks.reverse();
    }
    data.backgrounds.reverse();
    data
}

#[test]
fn canonical_game_data_does_not_depend_on_list_order() {
    let written = world();
    let backwards = reversed(world());
    assert_ne!(written, backwards, "the reversal changes something");

    assert_eq!(backwards.canonical(), written.canonical());
}

#[test]
fn canonical_catalogs_are_ordered_by_key() {
    let data = reversed(world()).canonical();
    let keys: Vec<_> = data.classes.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "bard",
            "cleric",
            "druid",
            "fighter",
            "kineticist",
            "monk",
            "priest",
            "ranger",
            "rogue",
            "seeker",
            "slayer",
            "sorcerer",
            "warpriest",
            "witch",
            "wizard"
        ]
    );
    let levels: Vec<_> = data.classes[0].levels.iter().map(|r| r.level).collect();
    assert_eq!(levels, (1..=20).collect::<Vec<_>>());
}

#[test]
fn counts_summarise_each_catalog() {
    assert_eq!(
        world().counts(),
        [
            ("Races", 4),
            ("Skills", 4),
            ("Weapons", 5),
            ("Feats", 40),
            ("Classes", 15),
            ("Archetypes", 14),
            ("Prestige classes", 5),
            ("Mythic abilities", 9),
            ("Mythic feats", 10),
            ("Mythic paths", 6),
            ("Backgrounds", 5),
            ("Deities", 3),
            ("Spells", 0),
            ("Spell lists", 0),
        ]
    );
}
