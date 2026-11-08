//! The test world's mythic paths: the Trickster and Aeon, and the switches.

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::builds::{feat, feat_list, feature, option, path};
use crate::test_world::world::selections::selection_option;
use crate::test_world::world::{bonus_feature, guide, list, plain_feature};

pub(super) fn trickster() -> MythicPath {
    let tricks = FeatureSelectionKey::new("trickster/tricks");
    let rank = |rank, features: Vec<FeatureKey>, feat_slots, selections| MythicRankRow {
        rank,
        features,
        feat_slots,
        selections,
    };
    MythicPath {
        key: path("trickster"),
        name: "Trickster".into(),
        summary: String::new(),
        guide: guide("trickster"),
        features: vec![
            plain_feature("trickster/trickster-tricks", "Trickster Tricks"),
            plain_feature(
                "trickster/bypass-epic-damage-reduction",
                "Bypass Epic Damage Reduction",
            ),
            plain_feature("trickster/bit-of-fun", "Bit of Fun"),
            plain_feature("trickster/hard-to-kill", "Hard to Kill"),
        ],
        selections: vec![FeatureSelection {
            key: tricks.clone(),
            name: "Trickster Tricks".into(),
            summary: String::new(),
            guide: guide("trickster#tricks"),
            shared_options: BTreeSet::new(),
            option_levels: BTreeSet::new(),
            skipped_levels: BTreeSet::new(),
            feat_lists_only: false,
            only_without_option: false,
            pool_of: None,
            levels_behind: 0,
            // Like the Azata's superpowers: an option's rows come by Mythic ranks since its pick.
            level_from: Some(LevelFrom::MythicRanks),
            once_per_option: false,
            level_also_from: None,
            deity_domains: None,
            options: vec![
                selection_option("trickster/tricks/perception-1", None, None),
                selection_option(
                    "trickster/tricks/perception-2",
                    None,
                    Some(Prerequisite::SelectionOption(option(
                        "trickster/tricks/perception-1",
                    ))),
                ),
                selection_option("trickster/tricks/athletics-1", None, None),
                selection_option("trickster/tricks/mobility-1", None, None),
                selection_option("trickster/tricks/persuasion-1", None, None),
                // Invented, like Azata's All-skilled: one option granting several feats.
                SelectionOption {
                    feats: BTreeSet::from([
                        feat("shield-proficiency"),
                        feat("tower-shield-proficiency"),
                    ]),
                    ..selection_option("trickster/tricks/all-armour", None, None)
                },
                // Like All-skilled's Exotic Weapon Proficiency: every value of a parameterised feat.
                selection_option(
                    "trickster/tricks/all-weapons",
                    Some(feat("exotic-weapon-proficiency")),
                    None,
                ),
                // Invented, like Azata's Life-Bonding Friendship: a teamwork feat this rank and the next.
                SelectionOption {
                    levels: [1, 2]
                        .map(|level| OptionLevelRow {
                            level,
                            feat_slots: vec![FeatSlot::Category(FeatCategory::Teamwork)],
                            ..OptionLevelRow::default()
                        })
                        .into(),
                    ..selection_option("trickster/tricks/friendship", None, None)
                },
            ],
        }],
        feat_lists: vec![list(
            "trickster/bonus-feats",
            false,
            &["dodge", "toughness"],
        )],
        ranks: vec![
            // Like every path's MR 1 row: the fixed Bypass Epic Damage Reduction and the path-linked ability.
            rank(
                1,
                vec![
                    feature("trickster/bypass-epic-damage-reduction"),
                    feature("trickster/bit-of-fun"),
                ],
                vec![],
                vec![],
            ),
            rank(2, vec![feature("trickster/hard-to-kill")], vec![], vec![]),
            rank(
                3,
                vec![feature("trickster/trickster-tricks")],
                vec![],
                vec![tricks.clone()],
            ),
            rank(
                4,
                vec![],
                vec![FeatSlot::OneOf {
                    list: feat_list("trickster/bonus-feats"),
                }],
                vec![],
            ),
            rank(5, vec![], vec![], vec![tricks.clone()]),
            rank(7, vec![], vec![], vec![tricks.clone()]),
            rank(9, vec![], vec![], vec![tricks.clone()]),
        ],
        switch: None,
    }
}

/// A path's rank row.
fn rank_row(
    rank: u8,
    features: &[&str],
    feat_slots: Vec<FeatSlot>,
    selections: &[&str],
) -> MythicRankRow {
    MythicRankRow {
        rank,
        features: features.iter().map(|key| feature(key)).collect(),
        feat_slots,
        selections: selections
            .iter()
            .map(|key| FeatureSelectionKey::new(*key))
            .collect(),
    }
}

/// Legend: keeps MR 1-2 (general picks and the old path's grants), stays at MR 3, raises the cap to 40, and the
/// switch grants +4 to every score.
pub(super) fn legend() -> MythicPath {
    MythicPath {
        key: path("legend"),
        name: "Legend".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#legend"),
        features: vec![bonus_feature(
            "legend/legend",
            vec![AbilityBonus::Every { amount: 4 }],
        )],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![],
        switch: Some(PathSwitch {
            from: BTreeSet::new(),
            kept_through_rank: 2,
            path_kept_through_rank: 2,
            dropped_features: BTreeSet::new(),
            dropped_selections: BTreeSet::new(),
            features: BTreeSet::from([feature("legend/legend")]),
            further_ascensions: false,
            level_cap: Some(40),
        }),
    }
}

/// Gold Dragon: keeps the general picks of MR 1-7 but the old path's grants of MR 1-2 only, and carries on
/// ascending. Perfect Body, Mind and Soul raise the scores; Dragon Feats are three picks of any feat ignoring
/// prerequisites; two breaths need MR 9.
pub(super) fn gold_dragon() -> MythicPath {
    let dragon_feats = || FeatSlot::OneOf {
        list: feat_list("gold-dragon/dragon-feats"),
    };
    MythicPath {
        key: path("gold-dragon"),
        name: "Gold Dragon".into(),
        summary: String::new(),
        guide: guide("gold-dragon"),
        features: vec![
            plain_feature("gold-dragon/breath", "Breath"),
            bonus_feature(
                "gold-dragon/perfect-body",
                vec![
                    AbilityBonus::AtLeast { score: 14 },
                    AbilityBonus::Highest { amount: 4 },
                ],
            ),
            // Also "your base attack bonus is at least 15" and "your rank in any skill is at least your mythic rank".
            Feature {
                bab_floor: Some(15),
                skill_ranks_at_least_mythic_rank: true,
                ..bonus_feature(
                    "gold-dragon/perfect-mind",
                    vec![
                        AbilityBonus::AtLeast { score: 18 },
                        AbilityBonus::Highest { amount: 2 },
                    ],
                )
            },
            bonus_feature(
                "gold-dragon/perfect-soul",
                vec![AbilityBonus::Highest { amount: 2 }],
            ),
        ],
        selections: vec![FeatureSelection {
            key: FeatureSelectionKey::new("gold-dragon/breaths"),
            name: "Breaths".into(),
            summary: String::new(),
            guide: guide("gold-dragon#breaths"),
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
                selection_option("gold-dragon/breaths/empowered", None, None),
                // The guide's "Mythic Rank: 9" and "Vow of Dahak": the story condition never blocks.
                selection_option(
                    "gold-dragon/breaths/annihilating",
                    None,
                    Some(Prerequisite::All(vec![
                        Prerequisite::MythicRank(9),
                        Prerequisite::Story(Story::VowOfDahak),
                    ])),
                ),
            ],
        }],
        feat_lists: vec![FeatList {
            every_feat: true,
            ..list("gold-dragon/dragon-feats", true, &[])
        }],
        ranks: vec![
            rank_row(
                8,
                &["gold-dragon/breath", "gold-dragon/perfect-body"],
                vec![],
                &["gold-dragon/breaths"],
            ),
            rank_row(
                9,
                &["gold-dragon/perfect-mind"],
                vec![dragon_feats(), dragon_feats(), dragon_feats()],
                &["gold-dragon/breaths"],
            ),
            rank_row(10, &["gold-dragon/perfect-soul"], vec![], &[]),
        ],
        switch: Some(PathSwitch {
            from: BTreeSet::new(),
            kept_through_rank: 7,
            path_kept_through_rank: 2,
            dropped_features: BTreeSet::new(),
            dropped_selections: BTreeSet::new(),
            features: BTreeSet::new(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}

/// Swarm That Walks: Gold Dragon's keeps; Eye of the Swarm at MR 9 raises Strength and Dexterity by 4.
pub(super) fn swarm_that_walks() -> MythicPath {
    MythicPath {
        key: path("swarm-that-walks"),
        name: "Swarm That Walks".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#swarm-that-walks"),
        features: vec![bonus_feature(
            "swarm-that-walks/eye-of-the-swarm",
            vec![
                AbilityBonus::Fixed {
                    ability: Ability::Strength,
                    amount: 4,
                },
                AbilityBonus::Fixed {
                    ability: Ability::Dexterity,
                    amount: 4,
                },
            ],
        )],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![rank_row(
            9,
            &["swarm-that-walks/eye-of-the-swarm"],
            vec![],
            &[],
        )],
        switch: Some(PathSwitch {
            from: BTreeSet::new(),
            kept_through_rank: 7,
            path_kept_through_rank: 2,
            dropped_features: BTreeSet::new(),
            dropped_selections: BTreeSet::new(),
            features: BTreeSet::new(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}

/// Aeon: MR 1-2 features, Aeon Bane at 4, an Enforcing Gaze pick at MR 3-7 (Area Effects needs MR 5) and Extra
/// Gaze at 6.
pub(super) fn aeon() -> MythicPath {
    MythicPath {
        key: path("aeon"),
        name: "Aeon".into(),
        summary: String::new(),
        guide: guide("aeon"),
        features: vec![
            plain_feature(
                "aeon/bypass-epic-damage-reduction",
                "Bypass Epic Damage Reduction",
            ),
            plain_feature("aeon/force-reality", "Force Reality"),
            plain_feature("aeon/hard-to-kill", "Hard to Kill"),
            plain_feature("aeon/aeon-bane", "Aeon Bane"),
            plain_feature("aeon/enforcing-gaze-extra-gaze", "Extra Gaze"),
        ],
        selections: vec![FeatureSelection {
            key: FeatureSelectionKey::new("aeon/enforcing-gaze"),
            name: "Enforcing Gaze".into(),
            summary: String::new(),
            guide: guide("aeon#enforcing-gaze"),
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
                selection_option("aeon/enforcing-gaze/attack", None, None),
                selection_option("aeon/enforcing-gaze/damage", None, None),
                selection_option("aeon/enforcing-gaze/save-dc", None, None),
                selection_option(
                    "aeon/enforcing-gaze/area-effects",
                    None,
                    Some(Prerequisite::MythicRank(5)),
                ),
            ],
        }],
        feat_lists: vec![],
        ranks: vec![
            rank_row(
                1,
                &["aeon/bypass-epic-damage-reduction", "aeon/force-reality"],
                vec![],
                &[],
            ),
            rank_row(2, &["aeon/hard-to-kill"], vec![], &[]),
            rank_row(3, &[], vec![], &["aeon/enforcing-gaze"]),
            rank_row(4, &["aeon/aeon-bane"], vec![], &["aeon/enforcing-gaze"]),
            rank_row(5, &[], vec![], &["aeon/enforcing-gaze"]),
            rank_row(
                6,
                &["aeon/enforcing-gaze-extra-gaze"],
                vec![],
                &["aeon/enforcing-gaze"],
            ),
            rank_row(7, &[], vec![], &["aeon/enforcing-gaze"]),
        ],
        switch: None,
    }
}

/// Devil: only from Aeon; keeps MR 1-7 but loses the Aeon's gaze (its picks and Extra Gaze).
pub(super) fn devil() -> MythicPath {
    MythicPath {
        key: path("devil"),
        name: "Devil".into(),
        summary: String::new(),
        guide: guide("other-mythic-paths#devil"),
        features: vec![plain_feature("devil/hells-dominance", "Hell's Dominance")],
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![rank_row(8, &["devil/hells-dominance"], vec![], &[])],
        switch: Some(PathSwitch {
            from: BTreeSet::from([path("aeon")]),
            kept_through_rank: 7,
            path_kept_through_rank: 7,
            dropped_features: BTreeSet::from([feature("aeon/enforcing-gaze-extra-gaze")]),
            dropped_selections: BTreeSet::from([FeatureSelectionKey::new("aeon/enforcing-gaze")]),
            features: BTreeSet::new(),
            further_ascensions: true,
            level_cap: None,
        }),
    }
}
