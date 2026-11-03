//! What only the finished data shows of the families' owners, asserted ([`check`]).

use rules::{Class, ClassKey, FeatSlot, FeatureKey, FeatureSelectionKey, GameData, Prerequisite};

/// What the providers of this module can only assert on the finished data: Dragon Disciple's, the Eldritch Scion's,
/// the Dragonheir Scion's and the Elemental Specialist's picks are on their rows in force, Dragon Disciple's ability
/// bonuses on the features of those names, and the Winter Witch has at most one shaman stand-in (a hand fix names
/// the Frost spirit there; a fresh extraction leaves the whole "Required classes" line out). Changes nothing.
pub(crate) fn check(data: &GameData) {
    let class = |key: &str| {
        data.classes
            .iter()
            .chain(&data.prestige_classes)
            .find(|class| class.key.as_str() == key)
            .unwrap_or_else(|| panic!("no class {key}"))
    };
    let row = |class: &str, archetype: &str, level: u8| {
        class_row(class_of(data, class), archetype, level)
            .unwrap_or_else(|| panic!("{archetype} has no row {level}"))
    };
    let disciple = class("dragon-disciple");
    let first = disciple
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("level 1");
    assert!(
        first
            .selections
            .iter()
            .any(|s| s.as_str() == "dragon-disciple/bloodline")
    );
    for level in [2, 5, 8] {
        let row = disciple
            .levels
            .iter()
            .find(|row| row.level == level)
            .expect("a row");
        assert!(
            row.feat_slots.iter().any(|slot| matches!(slot, FeatSlot::OptionList { selection, .. } if selection.as_str() == "sorcerer/bloodline")),
            "dragon-disciple {level}: the bloodline's feat"
        );
    }
    for feature in &disciple.features {
        let bonus =
            ["Strength +2", "Constitution +2", "Intelligence +2"].contains(&feature.name.as_str());
        assert_eq!(
            bonus,
            !feature.ability_bonuses.is_empty(),
            "{}",
            feature.key
        );
    }
    let (selections, _) = row("magus", "magus/eldritch-scion", 1);
    assert!(
        selections
            .iter()
            .any(|s| s.as_str() == "magus/eldritch-scion/bloodline")
    );
    let (selections, features) = row("fighter", "fighter/dragonheir-scion", 1);
    assert!(
        selections
            .iter()
            .any(|s| s.as_str() == "fighter/dragonheir-scion/bloodline")
    );
    assert!(
        !features
            .iter()
            .any(|f| f.as_str() == "fighter/dragonheir-scion/draconic-bloodline")
    );
    let (selections, features) = row("wizard", "wizard/elemental-specialist", 1);
    assert!(
        selections
            .iter()
            .any(|s| s.as_str() == "wizard/elemental-specialist/focused-element")
    );
    assert!(
        !features
            .iter()
            .any(|f| f.as_str() == "wizard/elemental-specialist/focused-element")
    );

    let stand_in = Prerequisite::ClassLevel {
        class: ClassKey::new("shaman"),
        level: 1,
    };
    let witch = class("winter-witch");
    let mut found = 0;
    if let Some(prerequisite) = &witch.prerequisite {
        count_leaf(prerequisite, &stand_in, &mut found);
    }
    assert!(found <= 1, "the Winter Witch's shaman stand-in");
}

fn class_of<'d>(data: &'d GameData, key: &str) -> &'d Class {
    data.classes
        .iter()
        .find(|class| class.key.as_str() == key)
        .unwrap_or_else(|| panic!("no class {key}"))
}

/// An archetype's row in force at `level`: its own, or its class's.
fn class_row<'c>(
    class: &'c Class,
    archetype: &str,
    level: u8,
) -> Option<(&'c [FeatureSelectionKey], &'c [FeatureKey])> {
    let archetype = class
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)?;
    match archetype.levels.iter().find(|row| row.level == level) {
        Some(row) => Some((&row.selections, &row.features)),
        None => class
            .levels
            .iter()
            .find(|row| row.level == level)
            .map(|row| (&row.selections[..], &row.features[..])),
    }
}

fn count_leaf(prerequisite: &Prerequisite, leaf: &Prerequisite, found: &mut usize) {
    if prerequisite == leaf {
        *found += 1;
    }
    if let Prerequisite::All(children)
    | Prerequisite::Any(children)
    | Prerequisite::AtLeast(_, children) = prerequisite
    {
        for child in children {
            count_leaf(child, leaf, found);
        }
    }
}
