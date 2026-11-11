//! Initiative, the saves and each skill's total (ticket 120): the character's own scores, the saves' class
//! progressions, the skill ranks and class skills, and the bonuses of the items held. Unconditional numbers only.
//! And the interactions notes (ticket 124): looked up by item, checked like the bonuses.
//!
//! Each total is read off a walked Build's state (ticket 151).

use std::collections::BTreeSet;

use rules::{
    Ability, Background, BackgroundKey, BonusAmount, BonusItem, BonusTarget, Build, DataIssue,
    Engine, Feat, FeatCategory, GameData, GuideRef, ItemBonus, ItemInteractions, MythicAbility,
    MythicAbilityGroup, MythicAbilityKey, MythicFeat, MythicFeatGroup, MythicFeatKey, Parameter,
    ParameterKind, Pick, Prerequisite, RaceKey, Repeat, Save, SkillKey, SlotId, Stat, Subrace,
    SubraceKey, Total,
};

use crate::support::*;

fn bonus(item: BonusItem, target: BonusTarget, amount: BonusAmount) -> ItemBonus {
    ItemBonus {
        item,
        target,
        amount,
        requires: None,
    }
}

fn engine_with(bonuses: impl IntoIterator<Item = ItemBonus>) -> Engine {
    let mut data = world_with_bonus_items();
    data.bonuses.extend(bonuses);
    engine_on(data)
}

/// The test world with the items these tests' bonuses name that it lacks, each with no more than a kind needs (no
/// prerequisite, feature, group or feat), named by its key as the parts already are; and Stat Focus taken for a
/// stat as the guide's is, with no prerequisite: so the data checks.
fn world_with_bonus_items() -> GameData {
    let mut data = world();
    for key in ["improved-initiative", "iron-will"] {
        data.feats.push(Feat {
            key: feat(key),
            name: key.into(),
            category: FeatCategory::Miscellaneous,
            summary: String::new(),
            guide: GuideRef::new(key),
            parameter: None,
            repeat: Repeat::Once,
            extra_pick_of: BTreeSet::new(),
            prerequisite: None,
        });
    }
    let stat_focus = data.feats.iter_mut().find(|f| f.key == feat("stat-focus"));
    let stat_focus = stat_focus.unwrap();
    (stat_focus.parameter, stat_focus.repeat) = (Some(ParameterKind::Stat), Repeat::PerValue);
    stat_focus.prerequisite = None;
    data.mythic_feats.push(MythicFeat {
        key: MythicFeatKey::new("improved-initiative-mythic"),
        name: "improved-initiative-mythic".into(),
        group: MythicFeatGroup::Miscellaneous,
        summary: String::new(),
        guide: GuideRef::new("improved-initiative-mythic"),
        parameter: None,
        repeat: Repeat::Once,
        removes_racial_penalty: false,
        extra_pick: None,
        prerequisite: None,
    });
    data.mythic_abilities.push(MythicAbility {
        key: MythicAbilityKey::new("grace"),
        name: "grace".into(),
        group: MythicAbilityGroup::Miscellaneous,
        summary: String::new(),
        guide: GuideRef::new("grace"),
        prerequisite: None,
        features: Vec::new(),
        selections: Vec::new(),
        parameter: None,
    });
    data.backgrounds.push(Background {
        key: BackgroundKey::new("pickpocket"),
        name: "pickpocket".into(),
        group: None,
        note: String::new(),
        class_skills: BTreeSet::new(),
        feats: BTreeSet::new(),
        feats_for: BTreeSet::new(),
    });
    let half_elf = data.races.iter_mut().find(|r| r.key.as_str() == "half-elf");
    half_elf.unwrap().subraces.push(Subrace {
        key: SubraceKey::new("half-elf/kindred"),
        name: "half-elf/kindred".into(),
        summary: String::new(),
        guide: GuideRef::new("half-elf/kindred"),
        ability_modifiers: Vec::new(),
        feat_slots: Vec::new(),
        skill_points: 0,
    });
    data
}

fn parts(total: &Total) -> Vec<(&str, i16)> {
    total
        .parts
        .iter()
        .map(|part| (part.name.as_str(), part.value))
        .collect()
}

/// A human Build of these classes with these point-buy scores (Str, Dex, Con, Int, Wis, Cha) and the race's +2 to
/// Strength.
fn character(scores: [u8; 6], classes: &[&str]) -> Build {
    let mut build = build("human", classes);
    build.start.ability_scores = point_buy(scores);
    build.start.racial_ability = Some(Ability::Strength);
    build
}

#[test]
fn initiative_is_dexterity_plus_the_bonuses_held_a_mythic_one_by_rank() {
    let engine = engine_with([
        bonus(
            BonusItem::Feat(feat("improved-initiative")),
            BonusTarget::Initiative,
            BonusAmount::Flat(4),
        ),
        bonus(
            BonusItem::MythicFeat(MythicFeatKey::new("improved-initiative-mythic")),
            BonusTarget::Initiative,
            BonusAmount::MythicRank,
        ),
        bonus(
            BonusItem::Background(BackgroundKey::new("pickpocket")),
            BonusTarget::Initiative,
            BonusAmount::Flat(2),
        ),
        bonus(
            BonusItem::Feat(feat("stat-focus")),
            BonusTarget::Held,
            BonusAmount::Flat(1),
        ),
    ]);
    // A fighter 15 (Mythic rank 6) with Dexterity 16.
    let mut build = character([10, 16, 10, 10, 10, 10], &times("fighter", 15));
    build.start.background = Some(BackgroundKey::new("pickpocket"));
    build
        .picks
        .insert(general_slot(1), pick_feat("improved-initiative"));
    build.picks.insert(
        race_slot(0),
        pick_feat_for("stat-focus", Parameter::Stat(Stat::Initiative)),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(MythicFeatKey::new("improved-initiative-mythic")),
    );

    let totals = engine.totals(&last_level(&engine, &build));

    assert_eq!(
        parts(&totals.initiative),
        [
            ("Dexterity", 3),
            ("pickpocket", 2),
            ("improved-initiative", 4),
            ("stat-focus (Initiative)", 1),
            ("improved-initiative-mythic", 6),
        ]
    );
    assert_eq!(totals.initiative.total, 16);
    // Without the background, its bonus is gone.
    build.start.background = None;
    let without = engine.totals(&last_level(&engine, &build));
    assert_eq!(without.initiative.total, 14);
    assert_eq!(totals.leaves_out, rules::TOTALS_LEAVE_OUT);
}

#[test]
fn a_save_is_its_base_its_ability_and_the_bonuses_for_it_or_every_save() {
    let engine = engine_with([
        bonus(
            BonusItem::Feat(feat("iron-will")),
            BonusTarget::Save(Save::Will),
            BonusAmount::Flat(2),
        ),
        bonus(
            BonusItem::Race(RaceKey::new("human")),
            BonusTarget::AllSaves,
            BonusAmount::Flat(1),
        ),
        bonus(
            BonusItem::MythicAbility(MythicAbilityKey::new("grace")),
            BonusTarget::AllSaves,
            BonusAmount::AbilityModifier(Ability::Charisma),
        ),
        bonus(
            BonusItem::Feat(feat("stat-focus")),
            BonusTarget::Held,
            BonusAmount::Flat(1),
        ),
    ]);
    // A fighter 5 / rogue 1 (base saves 4, 3 and 1, so each save's total is its own; Mythic rank 1).
    let mut classes = times("fighter", 5);
    classes.push("rogue");
    let mut build = character([10, 12, 14, 10, 8, 18], &classes);
    build.picks.insert(general_slot(1), pick_feat("iron-will"));
    build.picks.insert(
        race_slot(0),
        pick_feat_for("stat-focus", Parameter::Stat(Stat::Reflex)),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(MythicAbilityKey::new("grace")),
    );

    let totals = engine.totals(&last_level(&engine, &build));

    assert_eq!(
        parts(&totals.fortitude),
        [
            ("Base save", 4),
            ("Constitution", 2),
            ("Human", 1),
            ("grace", 4)
        ]
    );
    assert_eq!(
        parts(&totals.reflex),
        [
            ("Base save", 3),
            ("Dexterity", 1),
            ("Human", 1),
            ("stat-focus (Reflex)", 1),
            ("grace", 4),
        ]
    );
    assert_eq!(
        parts(&totals.will),
        [
            ("Base save", 1),
            ("Wisdom", -1),
            ("Human", 1),
            ("iron-will", 2),
            ("grace", 4)
        ]
    );
    assert_eq!(Save::ALL.map(|save| totals.save(save).total), [11, 10, 7]);
}

#[test]
fn a_skill_is_its_ranks_its_ability_the_class_skill_bonus_when_trained_and_its_bonuses() {
    let engine = engine_with([
        bonus(
            BonusItem::Feat(feat("skill-focus")),
            BonusTarget::Held,
            BonusAmount::TenRanks {
                amount: 3,
                with_ten_ranks: 6,
            },
        ),
        bonus(
            BonusItem::Feat(feat("toughness")),
            BonusTarget::Skill(skill("athletics")),
            BonusAmount::TenRanks {
                amount: 2,
                with_ten_ranks: 4,
            },
        ),
        bonus(
            BonusItem::Subrace(SubraceKey::new("half-elf/kindred")),
            BonusTarget::AllSkills,
            BonusAmount::HalfMythicRank,
        ),
    ]);
    // A bard 9 / monk 1: Persuasion and Mobility are class skills, Athletics isn't. Persuasion's ranks at every
    // level, Athletics' at the first nine. The race's +2 to Intelligence.
    let mut classes = times("bard", 9);
    classes.push("monk");
    let mut build = character([14, 10, 10, 12, 10, 16], &classes);
    build.start.racial_ability = Some(Ability::Intelligence);
    for (index, level) in build.levels.iter_mut().enumerate() {
        level.skill_ranks.insert(skill("persuasion"), 1);
        if index < 9 {
            level.skill_ranks.insert(skill("athletics"), 1);
        }
    }
    build.picks.insert(
        general_slot(1),
        pick_feat_for("skill-focus", skill_value("persuasion")),
    );
    build.picks.insert(race_slot(0), pick_feat("toughness"));

    let totals = engine.totals(&last_level(&engine, &build));
    let total = |key: &str| totals.skill(&skill(key)).expect("a skill of the data");

    // Every skill of the data, in its (canonical) order.
    assert_eq!(
        totals
            .skills
            .iter()
            .map(|total| total.skill.as_str())
            .collect::<Vec<_>>(),
        ["athletics", "knowledge-arcana", "mobility", "persuasion"]
    );
    // 10 ranks: Skill Focus's +6.
    assert_eq!(
        parts(&total("persuasion").total),
        [
            ("Ranks", 10),
            ("Charisma", 3),
            ("Class skill", 3),
            ("skill-focus (Persuasion)", 6),
        ]
    );
    assert_eq!(total("persuasion").total.total, 22);
    // 9 ranks: the lower amount; not a class skill.
    assert_eq!(
        parts(&total("athletics").total),
        [("Ranks", 9), ("Strength", 2), ("toughness", 2)]
    );
    // A class skill without ranks gets no class skill bonus; no subrace held, no bonus.
    assert_eq!(
        parts(&total("mobility").total),
        [("Ranks", 0), ("Dexterity", 0)]
    );
    assert_eq!(total("persuasion").ranks, 10);
}

#[test]
fn under_perfect_mind_every_skill_is_a_class_skill_with_ranks_at_least_the_mythic_rank() {
    let engine = engine();
    // A fighter 17 switched to Gold Dragon: Perfect Mind at MR 9, with every score at least 18 and the highest +6:
    // Strength 24, Dexterity 18. Athletics' ranks at the first twelve levels.
    let mut build = character([16, 14, 12, 13, 10, 8], &times("fighter", 17));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("gold-dragon"));
    for level in build.levels.iter_mut().take(12) {
        level.skill_ranks.insert(skill("athletics"), 1);
    }
    let level = last_level(&engine, &build);
    assert!(level.state.skill_ranks_at_least_mythic_rank);
    assert_eq!(level.state.mythic_rank, 9);

    let totals = engine.totals(&level);

    let athletics = totals.skill(&skill("athletics")).unwrap();
    assert_eq!(athletics.total.total, 22);
    let mobility = totals.skill(&skill("mobility")).unwrap();
    assert_eq!(
        parts(&mobility.total),
        [("Ranks", 9), ("Dexterity", 4), ("Class skill", 3)]
    );
}

#[test]
fn a_bonus_counts_only_with_what_it_requires() {
    let engine = engine_with([
        bonus(
            BonusItem::Feat(feat("toughness")),
            BonusTarget::Initiative,
            BonusAmount::Flat(2),
        ),
        ItemBonus {
            requires: Some(Prerequisite::ClassLevel {
                class: class("fighter"),
                level: 8,
            }),
            ..bonus(
                BonusItem::Feat(feat("toughness")),
                BonusTarget::Initiative,
                BonusAmount::Flat(4),
            )
        },
    ]);
    let mut build = character([10, 10, 10, 10, 10, 10], &times("fighter", 8));
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    let sheet = walked(&engine, &build);
    let initiative = |level: usize| engine.totals(&sheet.levels[level - 1]).initiative.total;

    // Fighter 8 brings the second.
    assert_eq!([initiative(7), initiative(8)], [2, 6]);
}

#[test]
fn the_data_check_refuses_a_held_or_ten_ranks_bonus_without_a_skill_and_two_notes_on_one_item() {
    let mut data: GameData = world();
    data.bonuses.extend([
        bonus(
            BonusItem::Feat(feat("toughness")),
            BonusTarget::Held,
            BonusAmount::Flat(1),
        ),
        bonus(
            BonusItem::Feat(feat("dodge")),
            BonusTarget::Initiative,
            BonusAmount::TenRanks {
                amount: 2,
                with_ten_ranks: 4,
            },
        ),
        bonus(
            BonusItem::Feat(feat("dodge")),
            BonusTarget::Skill(SkillKey::new("cooking")),
            BonusAmount::Flat(1),
        ),
        bonus(
            BonusItem::Background(BackgroundKey::new("no-such-background")),
            BonusTarget::Initiative,
            BonusAmount::Flat(1),
        ),
    ]);
    let note = |text: &str| ItemInteractions {
        item: BonusItem::Feat(feat("dodge")),
        interactions: text.to_owned(),
    };
    data.interactions.extend([note("One."), note("Two.")]);
    let issues: Vec<(String, DataIssue)> = data
        .check()
        .into_iter()
        .map(|problem| (problem.at.to_string(), problem.issue))
        .collect();

    for expected in [
        ("feat toughness", DataIssue::HeldBonusWithoutSkillOrStat),
        ("feat dodge", DataIssue::TenRanksWithoutSkill),
        ("feat dodge", DataIssue::InteractionsTwice),
        (
            "feat dodge",
            DataIssue::UnknownKey(rules::DataAt::Skill(SkillKey::new("cooking"))),
        ),
        (
            "background no-such-background",
            DataIssue::UnknownKey(rules::DataAt::Background(BackgroundKey::new(
                "no-such-background",
            ))),
        ),
    ] {
        assert!(
            issues.contains(&(expected.0.to_owned(), expected.1.clone())),
            "{expected:?} in {issues:?}"
        );
    }
}

#[test]
fn the_data_check_refuses_a_ten_ranks_bonus_on_every_skill() {
    // The walk works out a bonus's amount once, so a ten-ranks bonus is for one skill.
    let mut data: GameData = world();
    data.bonuses.push(bonus(
        BonusItem::Feat(feat("toughness")),
        BonusTarget::AllSkills,
        BonusAmount::TenRanks {
            amount: 2,
            with_ten_ranks: 4,
        },
    ));
    let issues: Vec<(String, DataIssue)> = data
        .check()
        .into_iter()
        .map(|problem| (problem.at.to_string(), problem.issue))
        .collect();

    assert!(
        issues.contains(&("feat toughness".to_owned(), DataIssue::TenRanksWithoutSkill)),
        "{issues:?}"
    );
}

#[test]
fn an_items_interactions_note_is_found_by_the_item() {
    let mut data = world();
    data.interactions.push(ItemInteractions {
        item: BonusItem::Feat(feat("dodge")),
        interactions: "The guide doesn't say.".to_owned(),
    });
    let engine = engine_on(data);

    assert_eq!(
        engine.interactions(&BonusItem::Feat(feat("dodge"))),
        Some("The guide doesn't say.")
    );
    assert_eq!(
        engine.interactions(&BonusItem::Feat(feat("toughness"))),
        None
    );
}
