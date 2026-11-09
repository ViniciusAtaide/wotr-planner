//! Rogue and Slayer, who share talents.

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::builds::{class, feat, feat_list, feature, option, skill};
use crate::test_world::world::classes::{archetype_row, row};
use crate::test_world::world::selections::{
    levelled_option, levelled_selection, option_row, selection_option,
};
use crate::test_world::world::{guide, list, plain_feature};

/// Rogue: 3/4 BAB, good Reflex, Sneak Attack at odd levels, a rogue talent at even levels.
pub(in crate::test_world::world) fn rogue() -> Class {
    let talents = FeatureSelectionKey::new("rogue/rogue-talents");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l % 2 == 1 {
                r.features.push(feature("rogue/sneak-attack"));
            } else {
                r.selections.push(talents.clone());
            }
            if l == 3 {
                r.feat_slots.push(FeatSlot::OneOf {
                    list: feat_list("rogue/skill-focus"),
                });
            }
            r
        })
        .collect();
    // Eldritch Scoundrel: arcane spells from level 1 (so a row at every level), the first Sneak Attack moves to
    // level 2.
    let scoundrel_levels = (1..=20)
        .map(|l| {
            let mut r = archetype_row(l);
            r.spell_levels = Some(SpellLevels {
                arcane: l.div_ceil(3).min(6),
                divine: 0,
            });
            if l == 2 || (l % 2 == 1 && l > 1) {
                r.features.push(feature("rogue/sneak-attack"));
            }
            if l % 2 == 0 {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    Class {
        casting_ability: None,
        key: class("rogue"),
        name: "Rogue".into(),
        summary: String::new(),
        guide: guide("rogue"),
        skill_points: 8,
        bab: BabProgression::ThreeQuarters,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics"), skill("mobility"), skill("persuasion")]),
        prerequisite: None,
        features: vec![plain_feature("rogue/sneak-attack", "Sneak Attack")],
        selections: vec![FeatureSelection {
            key: talents.clone(),
            name: "Rogue Talents".into(),
            summary: String::new(),
            guide: guide("rogue#rogue-talents"),
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
            deity_domains: None,
            options: vec![
                selection_option(
                    "rogue/rogue-talents/combat-trick",
                    Some(feat("combat-expertise")),
                    None,
                ),
                selection_option(
                    "rogue/rogue-talents/crippling-strike",
                    None,
                    Some(Prerequisite::All(vec![
                        Prerequisite::ClassLevel {
                            class: class("rogue"),
                            level: 10,
                        },
                        Prerequisite::SelectionOption(option("rogue/rogue-talents/combat-trick")),
                    ])),
                ),
                selection_option("rogue/rogue-talents/fast-stealth", None, None),
                // Weapon Focus in a weapon of the player's choosing.
                SelectionOption {
                    chosen_feat: Some(feat("weapon-focus")),
                    ..selection_option("rogue/rogue-talents/weapon-focus", None, None)
                },
                // Invented, like Increased Damage Reduction: taken up to twice.
                SelectionOption {
                    repeat: Repeat::Times(2),
                    ..selection_option("rogue/rogue-talents/resiliency", None, None)
                },
            ],
        }],
        // A Skill Focus of the player's choosing at 3, like the Master of All's.
        feat_lists: vec![list("rogue/skill-focus", true, &["skill-focus"])],
        levels,
        archetypes: vec![Archetype {
            casting_ability: Some(Ability::Intelligence),
            key: ArchetypeKey::new("rogue/eldritch-scoundrel"),
            name: "Eldritch Scoundrel".into(),
            summary: String::new(),
            guide: guide("rogue#eldritch-scoundrel"),
            prerequisite: Some(Prerequisite::Any(vec![
                Prerequisite::Race(RaceKey::new("human")),
                Prerequisite::Race(RaceKey::new("elf")),
            ])),
            skill_points: Some(4),
            class_skills: None,
            spontaneous: None,
            fort: None,
            reflex: None,
            will: None,
            features: vec![],
            selections: vec![],
            feat_lists: vec![],
            levels: scoundrel_levels,
        }],
    }
}

/// Slayer: full BAB, good Fortitude and Reflex, a slayer talent at even levels. Slayer talents share the rogue's
/// three talents and add Slayer Camouflage, which needs the rogue's Fast Stealth.
pub(in crate::test_world::world) fn slayer() -> Class {
    let talents = FeatureSelectionKey::new("slayer/slayer-talents");
    let levels = (1..=20)
        .map(|l| {
            let mut r = row(l);
            if l % 2 == 0 {
                r.selections.push(talents.clone());
            }
            r
        })
        .collect();
    Class {
        casting_ability: None,
        key: class("slayer"),
        name: "Slayer".into(),
        summary: String::new(),
        guide: guide("slayer"),
        skill_points: 6,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: Advancement::Each,
        spontaneous: false,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::from([skill("athletics"), skill("mobility")]),
        prerequisite: None,
        features: vec![],
        selections: vec![
            FeatureSelection {
                key: talents,
                name: "Slayer Talents".into(),
                summary: String::new(),
                guide: guide("slayer#slayer-talents"),
                options: vec![
                    selection_option(
                        "slayer/slayer-talents/slayer-camouflage",
                        None,
                        Some(Prerequisite::SelectionOption(option(
                            "rogue/rogue-talents/fast-stealth",
                        ))),
                    ),
                    // "Three times in total and at certain levels: 2 or 4, 6 or 8 and 10 or greater."
                    combat_style(2, Some(slayer_level_below(6))),
                    combat_style(
                        6,
                        Some(Prerequisite::All(vec![
                            slayer_level(6),
                            slayer_level_below(10),
                        ])),
                    ),
                    combat_style(10, Some(slayer_level(10))),
                    // Like the shaman's Battle Master: Weapon Specialization at 8 in a weapon of choice.
                    SelectionOption {
                        parameter: Some(ParameterKind::Weapon),
                        ..levelled_option(
                            "slayer/slayer-talents/battle-master",
                            vec![OptionLevelRow {
                                chosen_feats: BTreeSet::from([feat("weapon-specialization")]),
                                ..option_row(8, &[])
                            }],
                        )
                    },
                ],
                shared_options: [
                    "rogue/rogue-talents/combat-trick",
                    "rogue/rogue-talents/crippling-strike",
                    "rogue/rogue-talents/fast-stealth",
                ]
                .into_iter()
                .map(option)
                .collect(),
                option_levels: BTreeSet::from([class("slayer")]),
                skipped_levels: BTreeSet::new(),
                feat_lists_only: false,
                only_without_option: false,
                pool_of: None,
                levels_behind: 0,
                level_from: None,
                once_per_option: false,
                level_also_from: None,
                deity_domains: None,
            },
            slayer_style(),
        ],
        feat_lists: vec![],
        levels,
        archetypes: vec![],
    }
}

/// The slayer's combat style: one of the ranger's, picked with the first combat style feat (only by a character
/// holding none, so a ranger/slayer shares the ranger's), levelled by slayer levels. Each window's feat comes from
/// the style's list of the window's stage, whoever levels the style.
fn slayer_style() -> FeatureSelection {
    FeatureSelection {
        only_without_option: true,
        ..levelled_selection(
            "slayer/combat-style",
            &["slayer"],
            vec![],
            &["ranger/combat-style/archery", "ranger/combat-style/crane"],
        )
    }
}

fn slayer_level(level: u8) -> Prerequisite {
    Prerequisite::ClassLevel {
        class: class("slayer"),
        level,
    }
}

fn slayer_level_below(level: u8) -> Prerequisite {
    Prerequisite::Not(Box::new(slayer_level(level)))
}

/// A slayer talent that is a combat style feat: the style's pick (the first time), then a bonus feat from its list
/// at the window's stage.
fn combat_style(stage: u8, prerequisite: Option<Prerequisite>) -> SelectionOption {
    let style = FeatureSelectionKey::new("slayer/combat-style");
    SelectionOption {
        prerequisite,
        ..levelled_option(
            &format!("slayer/slayer-talents/combat-style-feat-{stage}"),
            vec![OptionLevelRow {
                selections: vec![style.clone()],
                feat_slots: vec![FeatSlot::OptionList {
                    selection: style,
                    at_level: Some(stage),
                    label: None,
                }],
                ..option_row(1, &[])
            }],
        )
    }
}
