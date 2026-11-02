//! Wizard, Witch (with False God) and Bard.

use std::collections::BTreeSet;

use rules::*;

use crate::support::builds::{class, feat, feature, skill};
use crate::support::world::classes::{archetype, archetype_row, row};
use crate::support::world::selections::{
    levelled_option, levelled_selection, option_row, selection_option,
};
use crate::support::world::{guide, plain_feature};

/// Wizard: 1/2 BAB, good Will, full arcane spell-level access, Scribe Scrolls granted at 1, a Magic bonus slot
/// every 5 levels.
pub(in crate::support::world) fn wizard() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.arcane_spell_level = l.div_ceil(2).min(9);
            if l == 1 {
                r.feats.insert(feat("scribe-scrolls"));
                r.selections.push(FeatureSelectionKey::new("wizard/school"));
            }
            if l % 5 == 0 {
                r.feat_slots.push(FeatSlot::Category(FeatCategory::Magic));
            }
            r
        })
        .collect();
    Class {
        key: class("wizard"),
        name: "Wizard".into(),
        summary: String::new(),
        guide: guide("wizard"),
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
        prerequisite: None,
        features: vec![
            plain_feature("wizard/hand-of-the-apprentice", "Hand of the Apprentice"),
            plain_feature("wizard/metamagic-mastery", "Metamagic Mastery"),
            Feature {
                daily_choice: true,
                ..plain_feature("wizard/resistance", "Resistance")
            },
            plain_feature("wizard/protective-ward", "Protective Ward"),
            plain_feature("wizard/energy-absorption", "Energy Absorption"),
        ],
        // A school at 1, levelled by wizard levels; the Abjurer's Resistance is chosen anew each day.
        selections: vec![levelled_selection(
            "wizard/school",
            &["wizard"],
            vec![
                levelled_option(
                    "wizard/school/universalist",
                    vec![
                        option_row(1, &["wizard/hand-of-the-apprentice"]),
                        option_row(8, &["wizard/metamagic-mastery"]),
                    ],
                ),
                levelled_option(
                    "wizard/school/abjurer",
                    vec![
                        option_row(1, &["wizard/protective-ward", "wizard/resistance"]),
                        option_row(6, &["wizard/energy-absorption"]),
                    ],
                ),
            ],
            &[],
        )],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// Witch: 1/2 BAB, good Will, full arcane spell-level access like the wizard, a hex at 1 and every even level,
/// Major Hexes at 10 and Grand Hexes at 18 (which the hexes that need them check).
pub(in crate::support::world) fn witch() -> Class {
    let hex = FeatureSelectionKey::new("witch/hex");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.arcane_spell_level = l.div_ceil(2).min(9);
            if l == 1 || l % 2 == 0 {
                r.selections.push(hex.clone());
            }
            if l == 10 {
                r.features.push(feature("witch/major-hexes"));
            }
            if l == 18 {
                r.features.push(feature("witch/grand-hexes"));
            }
            r
        })
        .collect();
    let needs_feature = |key: &str| {
        Some(Prerequisite::Feature {
            feature: feature(key),
            count: 1,
        })
    };
    Class {
        key: class("witch"),
        name: "Witch".into(),
        summary: String::new(),
        guide: guide("witch"),
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
        prerequisite: None,
        features: vec![
            plain_feature("witch/major-hexes", "Major Hexes"),
            plain_feature("witch/grand-hexes", "Grand Hexes"),
        ],
        selections: vec![FeatureSelection {
            key: hex,
            name: "Hex".into(),
            summary: String::new(),
            guide: guide("witch#hex"),
            shared_options: BTreeSet::new(),
            option_levels: BTreeSet::new(),
            skipped_levels: BTreeSet::new(),
            feat_lists_only: false,
            only_without_option: false,
            pool_of: None,
            levels_behind: 0,
            level_from: None,
            once_per_option: false,
            level_also_from: None,
            options: vec![
                selection_option("witch/hex/slumber", None, None),
                selection_option("witch/hex/misfortune", None, None),
                selection_option("witch/hex/agony", None, needs_feature("witch/major-hexes")),
                selection_option(
                    "witch/hex/life-giver",
                    None,
                    needs_feature("witch/grand-hexes"),
                ),
            ],
        }],
        feat_lists: vec![],
        levels,
        archetypes: vec![false_god()],
    }
}

/// False God (a Living Deity on the witch): a way at 1 (its one option, so granted), levelled by witch levels, whose
/// row 20 is a domain. Impossible Domain levels its domain by the way's level.
fn false_god() -> Archetype {
    let way = FeatureSelectionKey::new("witch/false-god/way");
    Archetype {
        selections: vec![levelled_selection(
            way.as_str(),
            &["witch"],
            vec![levelled_option(
                "witch/false-god/way/deity",
                vec![OptionLevelRow {
                    selections: vec![FeatureSelectionKey::new("cleric/domain")],
                    ..option_row(20, &[])
                }],
            )],
            &[],
        )],
        levels: vec![ArchetypeLevelRow {
            selections: vec![FeatureSelectionKey::new("witch/hex"), way],
            ..archetype_row(1)
        }],
        ..archetype("witch/false-god")
    }
}

/// Bard: 3/4 BAB, spontaneous arcane spell-level access from 1 (a way into Dragon Disciple without a bloodline).
pub(in crate::support::world) fn bard() -> Class {
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            r.arcane_spell_level = l.div_ceil(3).min(6);
            r
        })
        .collect();
    Class {
        key: class("bard"),
        name: "Bard".into(),
        summary: String::new(),
        guide: guide("bard"),
        skill_points: 6,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: true,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("persuasion")]),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}
