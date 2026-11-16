//! The test world's races and subraces.

use std::collections::BTreeSet;

use crate::test_world::world::nested::{Race, Subrace};
use crate::*;

use crate::test_world::builds::feat_list;
use crate::test_world::world::{guide, list};

pub(super) fn races() -> Vec<Race> {
    let fixed = |ability, amount| AbilityModifier::Fixed { ability, amount };
    vec![
        // Adaptability: Skill Focus as a bonus feat at 1, from the race's own one-feat list.
        Race {
            key: RaceKey::new("half-elf"),
            blueprint: None,
            name: "Half-Elf".into(),
            summary: String::new(),
            guide: guide("character-creation#half-elf"),
            ability_modifiers: vec![AbilityModifier::Chosen { amount: 2 }],
            selections: vec![feat_list("half-elf/adaptability")],
            feat_lists: vec![list("half-elf/adaptability", true, &["skill-focus"])],
            skill_points: 0,
            weapons: BTreeSet::new(),
            weapons_as_martial: BTreeSet::new(),
            // A subrace's slots replace the race's: Dual Heritage has none; an invented one keeps Adaptability.
            subraces: vec![
                Subrace {
                    key: SubraceKey::new("half-elf/dual-heritage"),
                    blueprint: None,
                    name: "Dual Heritage".into(),
                    summary: String::new(),
                    guide: guide("character-creation#half-elf-dual-heritage"),
                    ability_modifiers: vec![AbilityModifier::Chosen { amount: 2 }],
                    selections: vec![],
                    skill_points: 0,
                    weapons: BTreeSet::new(),
                    weapons_as_martial: BTreeSet::new(),
                },
                Subrace {
                    key: SubraceKey::new("half-elf/adaptable"),
                    blueprint: None,
                    name: "Adaptable".into(),
                    summary: String::new(),
                    guide: guide("character-creation#half-elf-adaptable"),
                    ability_modifiers: vec![AbilityModifier::Chosen { amount: 2 }],
                    selections: vec![feat_list("half-elf/adaptability")],
                    // Invented, as the Human-Raised half-orc's: a subrace's skill points replace the race's.
                    skill_points: 1,
                    weapons: BTreeSet::new(),
                    weapons_as_martial: BTreeSet::new(),
                },
            ],
        },
        Race {
            key: RaceKey::new("human"),
            blueprint: None,
            name: "Human".into(),
            summary: String::new(),
            guide: guide("character-creation#human"),
            ability_modifiers: vec![AbilityModifier::Chosen { amount: 2 }],
            selections: vec![feat_list(BASIC_FEAT_SELECTION)],
            feat_lists: vec![],
            skill_points: 1,
            weapons: BTreeSet::new(),
            weapons_as_martial: BTreeSet::new(),
            subraces: vec![],
        },
        Race {
            key: RaceKey::new("elf"),
            blueprint: None,
            name: "Elf".into(),
            summary: String::new(),
            guide: guide("character-creation#elf"),
            ability_modifiers: vec![
                fixed(Ability::Dexterity, 2),
                fixed(Ability::Intelligence, 2),
                fixed(Ability::Constitution, -2),
            ],
            selections: vec![],
            feat_lists: vec![],
            skill_points: 0,
            weapons: BTreeSet::new(),
            weapons_as_martial: BTreeSet::new(),
            subraces: vec![Subrace {
                key: SubraceKey::new("elf/blightborn"),
                blueprint: None,
                name: "Blightborn".into(),
                summary: String::new(),
                guide: guide("character-creation#elf-blightborn"),
                ability_modifiers: vec![
                    fixed(Ability::Dexterity, 2),
                    fixed(Ability::Wisdom, 2),
                    fixed(Ability::Constitution, -2),
                ],
                selections: vec![],
                skill_points: 0,
                weapons: BTreeSet::new(),
                weapons_as_martial: BTreeSet::new(),
            }],
        },
        Race {
            key: RaceKey::new("dwarf"),
            blueprint: None,
            name: "Dwarf".into(),
            summary: String::new(),
            guide: guide("character-creation#dwarf"),
            ability_modifiers: vec![
                fixed(Ability::Constitution, 2),
                fixed(Ability::Wisdom, 2),
                fixed(Ability::Charisma, -2),
            ],
            selections: vec![],
            feat_lists: vec![],
            skill_points: 0,
            weapons: BTreeSet::new(),
            weapons_as_martial: BTreeSet::new(),
            subraces: vec![],
        },
    ]
}
