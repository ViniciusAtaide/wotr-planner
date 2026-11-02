//! The prestige classes: Mystic Theurge, Arcane Trickster, Eldritch Knight, Winter Witch, Dragon Disciple.

use std::collections::BTreeSet;

use rules::*;

use crate::support::builds::{class, feature, option, skill};
use crate::support::world::classes::row;
use crate::support::world::selections::levelled_selection;
use crate::support::world::{bonus_feature, guide};

/// Mystic Theurge: every level advances both the chosen arcane class and the chosen divine class.
pub(in crate::support::world) fn mystic_theurge() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = true;
            r.advances_divine = true;
            r
        })
        .collect();
    Class {
        key: class("mystic-theurge"),
        name: "Mystic Theurge".into(),
        summary: String::new(),
        guide: guide("prestige-classes#mystic-theurge"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: Some(Prerequisite::All(vec![
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Divine,
                level: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Arcane Trickster: every level advances the chosen arcane class; Sneak Attack at even levels.
pub(in crate::support::world) fn arcane_trickster() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = true;
            if l % 2 == 0 {
                r.features.push(feature("rogue/sneak-attack"));
            }
            r
        })
        .collect();
    Class {
        key: class("arcane-trickster"),
        name: "Arcane Trickster".into(),
        summary: String::new(),
        guide: guide("prestige-classes#arcane-trickster"),
        skill_points: 4,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("mobility")]),
        prerequisite: Some(Prerequisite::All(vec![
            Prerequisite::SkillRanks {
                skill: skill("mobility"),
                ranks: 4,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::Feature {
                feature: feature("rogue/sneak-attack"),
                count: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Eldritch Knight: full BAB, good Fortitude, a Combat feat bonus slot at 1, 5 and 9, the arcane class advanced
/// from 2; its levels count as fighter levels for feat prerequisites (Diverse Training).
pub(in crate::support::world) fn eldritch_knight() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = l >= 2;
            if l % 4 == 1 {
                r.feat_slots.push(FeatSlot::CombatFeat);
            }
            r
        })
        .collect();
    Class {
        key: class("eldritch-knight"),
        name: "Eldritch Knight".into(),
        summary: String::new(),
        guide: guide("prestige-classes#eldritch-knight"),
        skill_points: 2,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::from([CountsAs {
            class: class("fighter"),
            toward: CountsToward::FeatPrerequisites,
        }]),
        class_skills: BTreeSet::from([skill("athletics")]),
        prerequisite: Some(Prerequisite::SpellLevel {
            spell_type: SpellType::Arcane,
            level: 2,
        }),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Winter Witch: every level advances one class, arcane or divine; its levels stack with witch levels for Major
/// and Grand Hexes (Frost Power).
pub(in crate::support::world) fn winter_witch() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = true;
            r.advances_divine = true;
            r
        })
        .collect();
    let toward_witch = |key: &str| CountsAs {
        class: class("witch"),
        toward: CountsToward::Feature(feature(key)),
    };
    Class {
        key: class("winter-witch"),
        name: "Winter Witch".into(),
        summary: String::new(),
        guide: guide("prestige-classes#winter-witch"),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Either,
        spontaneous: false,
        counts_as: BTreeSet::from([
            toward_witch("witch/major-hexes"),
            toward_witch("witch/grand-hexes"),
        ]),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: Some(Prerequisite::Any(vec![
            Prerequisite::SpellLevel {
                spell_type: SpellType::Arcane,
                level: 2,
            },
            Prerequisite::SpellLevel {
                spell_type: SpellType::Divine,
                level: 2,
            },
        ])),
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Dragon Disciple: entry needs spontaneous 1st-level arcane spells and a Draconic bloodline or none; a sorcerer's
/// bloodline continues, anyone else picks a Draconic one on entry (levelled by Dragon Disciple levels). +2
/// Strength at 2 and 4.
pub(in crate::support::world) fn dragon_disciple() -> Class {
    let levels = (1..=10)
        .map(|l| {
            let mut r = row(l);
            r.advances_arcane = ![1, 5, 9].contains(&l);
            if l == 1 {
                r.selections
                    .push(FeatureSelectionKey::new("dragon-disciple/bloodline"));
            }
            if l == 2 || l == 4 {
                r.features.push(feature("dragon-disciple/strength"));
            }
            r
        })
        .collect();
    Class {
        key: class("dragon-disciple"),
        name: "Dragon Disciple".into(),
        summary: String::new(),
        guide: guide("prestige-classes#dragon-disciple"),
        skill_points: 2,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("knowledge-arcana")]),
        prerequisite: Some(Prerequisite::All(vec![
            Prerequisite::SpontaneousSpellLevel {
                spell_type: SpellType::Arcane,
                level: 1,
            },
            Prerequisite::Any(vec![
                Prerequisite::SelectionOption(option("sorcerer/bloodline/draconic-gold")),
                Prerequisite::SelectionOption(option("sorcerer/bloodline/draconic-red")),
                Prerequisite::NoOptionFrom(FeatureSelectionKey::new("sorcerer/bloodline")),
            ]),
        ])),
        features: vec![bonus_feature(
            "dragon-disciple/strength",
            vec![AbilityBonus::Fixed {
                ability: Ability::Strength,
                amount: 2,
            }],
        )],
        selections: vec![FeatureSelection {
            only_without_option: true,
            ..levelled_selection(
                "dragon-disciple/bloodline",
                &["dragon-disciple"],
                vec![],
                &[
                    "sorcerer/bloodline/draconic-gold",
                    "sorcerer/bloodline/draconic-red",
                ],
            )
        }],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}
