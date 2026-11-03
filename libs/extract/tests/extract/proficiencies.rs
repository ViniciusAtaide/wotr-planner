//! Proficiency feats: a class's proficiency text, read from its page, becomes the real Armor, Shield and Weapon
//! Proficiency feats of the cell that names its proficiency feature; a feature those feats cover in full is those feats
//! alone.

use std::collections::BTreeSet;

use extract::build::FeatureOwner;
use extract::picks::{Pick, Picks};
use extract::proficiencies::provide;
use rules::{ArchetypeKey, ClassKey, FeatKey, FeatureKey};

fn feats(keys: &[&str]) -> BTreeSet<FeatKey> {
    keys.iter().map(|key| FeatKey::new(*key)).collect()
}

fn picks() -> Picks {
    let mut picks = Picks::default();
    provide(&mut picks);
    picks
}

/// The feats a cell naming `feature` in `table` grants, and whether the feature stays beside them.
fn cell(picks: &Picks, table: &str, feature: &str, level: u8) -> Option<(BTreeSet<FeatKey>, bool)> {
    match picks.pick(table, "", &FeatureKey::new(feature), level)? {
        Pick::Feats(feats) => Some((feats.clone(), false)),
        Pick::FeatureAndFeats(feats) => Some((feats.clone(), true)),
        other => panic!("{feature}: {other:?}"),
    }
}

fn archetype(class: &str, archetype: &str) -> FeatureOwner {
    FeatureOwner::Archetype(ClassKey::new(class), ArchetypeKey::new(archetype))
}

#[test]
fn a_proficiency_feature_the_feats_cover_gives_way_to_them() {
    // "Fighters are proficient with all simple and martial weapons, all types of armour and shields (including
    // tower shields)."
    let picks = picks();
    assert_eq!(
        cell(&picks, "fighter", "fighter/fighter-proficiencies", 1),
        Some((
            feats(&[
                "armor-proficiency-heavy-armor",
                "armor-proficiency-light-armor",
                "armor-proficiency-medium-armor",
                "martial-weapons-proficiency",
                "shield-proficiency",
                "simple-weapon-proficiency",
                "tower-shield-proficiency",
            ]),
            false
        ))
    );
    assert_eq!(cell(&picks, "fighter", "fighter/bravery", 2), None);
}

#[test]
fn a_feature_that_names_more_than_the_feats_stays() {
    // "Rogues are proficient with all simple weapons, rapier, short sword and short bow. They are proficient with
    // light armour but not with shields." The named weapons have no feat.
    let picks = picks();
    assert_eq!(
        cell(&picks, "rogue", "rogue/rogue-proficiencies", 1),
        Some((
            feats(&["armor-proficiency-light-armor", "simple-weapon-proficiency"]),
            true
        ))
    );
    // "The Eldritch Scoundrel is not proficient with light armour."
    assert_eq!(
        cell(
            &picks,
            "rogue/eldritch-scoundrel",
            "rogue/rogue-proficiencies",
            1
        ),
        Some((feats(&["simple-weapon-proficiency"]), true))
    );
}

#[test]
fn a_class_whose_table_names_no_proficiency_feature_gets_its_feats_at_level_1() {
    // "Kineticists are proficient with all simple weapons and light armour", and so are its archetypes; the
    // Kinetic Knight's Elemental Bastion adds medium and heavy armour and shields.
    let picks = picks();
    let base = feats(&["armor-proficiency-light-armor", "simple-weapon-proficiency"]);
    let kineticist = FeatureOwner::Class(ClassKey::new("kineticist"));
    let knight = archetype("kineticist", "kineticist/kinetic-knight");
    assert_eq!(picks.row_feats_of(&kineticist, 1), base);
    assert_eq!(picks.row_feats_of(&kineticist, 2), BTreeSet::new());
    assert_eq!(picks.row_feats_of(&knight, 1), base);
    assert_eq!(picks.row_feats_of(&knight, 2), BTreeSet::new());
    assert_eq!(
        cell(
            &picks,
            "kineticist/kinetic-knight",
            "kineticist/kinetic-knight/elemental-bastion",
            1
        ),
        Some((
            feats(&[
                "armor-proficiency-heavy-armor",
                "armor-proficiency-medium-armor",
                "shield-proficiency",
            ]),
            true
        ))
    );
}
