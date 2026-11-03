//! Feat and rule follow-ups (ticket 32).

use std::collections::BTreeSet;

use rules::{
    ClassKey, FeatKey, Issue, LevelUp, MythicPathKey, Parameter, ParameterKind, Pick, Prerequisite,
    Repeat, SelectionOptionKey, SkillKey, SlotId,
};

use crate::support::*;

#[test]
fn a_human_warpriest_takes_dazzling_display_at_1_on_the_class_slots_weapon_focus() {
    let mut build = build("warpriest", 1);
    build.start.race = Some(rules::RaceKey::new("human"));
    build.picks.insert(
        class_slot(1),
        feat_pick("weapon-focus", weapon("longsword")),
    );
    build
        .picks
        .insert(race_slot(), Pick::feat(FeatKey::new("dazzling-display")));
    assert!(possible(&build));
}

#[test]
fn a_human_druid_with_a_companion_takes_boon_companion_at_1() {
    let mut build = build("druid", 1);
    build.start.race = Some(rules::RaceKey::new("human"));
    build.start.alignment = Some(rules::Alignment::TrueNeutral);
    build.picks.insert(
        selection_slot(1),
        pick("druid/nature-bond/animal-companion"),
    );
    build
        .picks
        .insert(race_slot(), Pick::feat(FeatKey::new("boon-companion")));
    assert!(possible(&build));
}

#[test]
fn every_parameterised_feat_repeats_once_per_value() {
    let parameterised: Vec<&rules::Feat> = DATA
        .feats
        .iter()
        .filter(|f| f.parameter.is_some())
        .collect();
    assert_eq!(parameterised.len(), 15);
    for feat in parameterised {
        assert_eq!(feat.repeat, Repeat::PerValue, "{}", feat.key);
    }
}

#[test]
fn weapon_focus_and_greater_weapon_focus_each_in_two_weapons() {
    let mut fighter = build("fighter", 10);
    for (slot, key, value) in [
        (class_slot(1), "weapon-focus", "longsword"),
        (general_slot(1), "weapon-focus", "dagger"),
        (class_slot(8), "greater-weapon-focus", "longsword"),
        (class_slot(10), "greater-weapon-focus", "dagger"),
    ] {
        fighter.picks.insert(slot, feat_pick(key, weapon(value)));
    }
    assert!(possible(&fighter));
    assert_eq!(
        values_at(&fighter, 10, "greater-weapon-focus"),
        BTreeSet::from([weapon("dagger"), weapon("longsword")])
    );
    // Not twice in one weapon.
    fighter.picks.insert(
        class_slot(10),
        feat_pick("greater-weapon-focus", weapon("longsword")),
    );
    assert_eq!(why_not(&fighter, &class_slot(10)), Some(Issue::AlreadyHeld));
}

#[test]
fn exotic_weapon_proficiency_can_take_the_sawtooth_saber() {
    // Exotic Weapon Proficiency can take the Sawtooth Saber.
    let mut fighter = build("fighter", 1);
    fighter.picks.insert(
        general_slot(1),
        feat_pick("exotic-weapon-proficiency", weapon("sawtooth-saber")),
    );
    assert!(possible(&fighter));
}

#[test]
fn armor_focus_heavy_needs_heavy_armour_proficiency() {
    use rules::ArmourType as A;
    let focus = |armour| feat_pick("armor-focus", Parameter::ArmourType(armour));
    // A fighter has every armour proficiency.
    let mut fighter = build("fighter", 1);
    fighter.picks.insert(class_slot(1), focus(A::Heavy));
    assert!(possible(&fighter));
    // A cleric has light and medium only.
    let mut cleric = build("cleric", 3);
    cleric.picks.insert(general_slot(3), focus(A::Heavy));
    assert_eq!(
        why_not(&cleric, &general_slot(3)),
        unmet(
            Prerequisite::FeatIfValue {
                value: Parameter::ArmourType(A::Heavy),
                feat: FeatKey::new("armor-proficiency-heavy-armor"),
            },
            None
        )
    );
    cleric.picks.insert(general_slot(3), focus(A::Medium));
    assert!(possible(&cleric));
}

#[test]
fn improved_critical_mythic_in_a_longsword_needs_improved_critical_in_a_longsword() {
    let mut fighter = build("fighter", 11);
    fighter.picks.insert(
        class_slot(8),
        feat_pick("improved-critical", weapon("longsword")),
    );
    let mythic = SlotId::Mythic { rank: 4 };
    fighter.picks.insert(
        mythic.clone(),
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("improved-critical-mythic"),
            weapon("longsword"),
        ),
    );
    assert!(possible(&fighter));
    fighter.picks.insert(
        mythic.clone(),
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("improved-critical-mythic"),
            weapon("dagger"),
        ),
    );
    assert_eq!(
        why_not(&fighter, &mythic),
        unmet(
            Prerequisite::FeatForSameValue(FeatKey::new("improved-critical")),
            None
        )
    );
}

#[test]
fn the_mythic_feats_that_pick_a_value_take_one_once_per_value() {
    use ParameterKind as K;
    for (key, kind, base) in [
        (
            "improved-critical-mythic",
            K::Weapon,
            Some("improved-critical"),
        ),
        ("spell-focus-mythic", K::SpellSchool, Some("spell-focus")),
        ("skill-focus-mythic", K::Skill, Some("skill-focus")),
        ("expanded-arsenal", K::SpellSchool, None),
        ("school-mastery", K::SpellSchool, None),
    ] {
        let feat = mythic_feat(key);
        assert_eq!(
            (feat.parameter, feat.repeat),
            (Some(kind), Repeat::PerValue),
            "{key}"
        );
        assert_eq!(
            feat.prerequisite,
            base.map(|base| Prerequisite::FeatForSameValue(FeatKey::new(base))),
            "{key}"
        );
    }
    // The Trickster's chain: each needs the one before in the same weapon.
    for (key, base) in [
        ("improved-improved-critical", "improved-critical"),
        (
            "improved-improved-improved-critical",
            "improved-improved-critical",
        ),
    ] {
        let Some(Prerequisite::All(children)) = &feat(key).prerequisite else {
            panic!("{key}");
        };
        assert!(children.contains(&Prerequisite::FeatForSameValue(FeatKey::new(base))));
        assert_eq!(feat(key).parameter, Some(K::Weapon));
    }
}

#[test]
fn a_mantis_zealot_holds_its_sawtooth_saber_feats_at_1_4_8_and_12() {
    // Lawful neutral: a Mantis Zealot is LE, LN or NE (ticket 63).
    let mut zealot = archetype_build("warpriest", "warpriest/mantis-zealot", 12);
    zealot.start.alignment = Some(rules::Alignment::LawfulNeutral);
    assert!(possible(&zealot));
    let saber = || BTreeSet::from([weapon("sawtooth-saber")]);
    for (level, key) in [
        (1, "weapon-focus"),
        (1, "exotic-weapon-proficiency"),
        (4, "weapon-specialization"),
        (8, "greater-weapon-focus"),
        (12, "greater-weapon-specialization"),
    ] {
        assert_eq!(values_at(&zealot, level, key), saber(), "{key} at {level}");
        if level > 1 {
            assert_eq!(
                values_at(&zealot, level - 1, key),
                BTreeSet::new(),
                "{key} before {level}"
            );
        }
    }
}

#[test]
fn the_chirurgeon_and_the_enlightened_philosopher_hold_skill_focus_in_their_skill() {
    let chirurgeon = archetype_build("alchemist", "alchemist/chirurgeon", 5);
    assert_eq!(values_at(&chirurgeon, 4, "skill-focus"), BTreeSet::new());
    assert_eq!(
        values_at(&chirurgeon, 5, "skill-focus"),
        BTreeSet::from([skill("lore-religion")])
    );
    let philosopher = archetype_build("oracle", "oracle/enlightened-philosopher", 1);
    assert_eq!(
        values_at(&philosopher, 1, "skill-focus"),
        BTreeSet::from([skill("knowledge-world")])
    );
}

#[test]
fn a_zen_archers_way_of_the_bow_is_weapon_focus_in_a_bow_and_specialization_in_it_at_6() {
    let way = "monk/zen-archer/way-of-the-bow/bow";
    let mut archer = archetype_build("monk", "monk/zen-archer", 6);
    archer.start.alignment = Some(rules::Alignment::LawfulNeutral);
    let offered: Vec<Pick> = engine()
        .options(&archer, &selection_slot(2))
        .expect("a pick at 2")
        .into_iter()
        .map(|option| option.pick)
        .collect();
    assert_eq!(
        offered,
        [
            Pick::option_for(SelectionOptionKey::new(way), weapon("long-bow")),
            Pick::option_for(SelectionOptionKey::new(way), weapon("short-bow")),
        ]
    );
    archer.picks.insert(
        selection_slot(2),
        Pick::option_for(SelectionOptionKey::new(way), weapon("short-bow")),
    );
    assert!(possible(&archer));
    assert_eq!(
        values_at(&archer, 2, "weapon-focus"),
        BTreeSet::from([weapon("short-bow")])
    );
    assert_eq!(
        values_at(&archer, 5, "weapon-specialization"),
        BTreeSet::new()
    );
    assert_eq!(
        values_at(&archer, 6, "weapon-specialization"),
        BTreeSet::from([weapon("short-bow")])
    );
}

#[test]
fn a_battle_master_shaman_specialises_at_8_and_greater_focuses_at_16_in_one_weapon() {
    let mut shaman = build("shaman", 16);
    shaman
        .picks
        .insert(selection_slot(1), pick("shaman/spirit/battle"));
    shaman.picks.insert(
        selection_slot(2),
        Pick::option_for(
            SelectionOptionKey::new("shaman/hex/battle-master"),
            weapon("spear"),
        ),
    );
    assert!(possible(&shaman));
    assert_eq!(
        values_at(&shaman, 7, "weapon-specialization"),
        BTreeSet::new()
    );
    assert_eq!(
        values_at(&shaman, 8, "weapon-specialization"),
        BTreeSet::from([weapon("spear")])
    );
    assert_eq!(
        values_at(&shaman, 16, "greater-weapon-focus"),
        BTreeSet::from([weapon("spear")])
    );
}

#[test]
fn a_sword_saint_takes_weapon_focus_in_a_weapon_of_his_choosing_at_1() {
    let saint = archetype_build("magus", "magus/sword-saint", 1);
    let options = engine().options(&saint, &class_slot(1)).unwrap();
    assert_eq!(options.len(), DATA.weapons.len());
    assert!(
        options
            .iter()
            .all(|option| option.pick.key() == "weapon-focus" && option.status.is_ok())
    );
}

#[test]
fn a_half_elfs_race_slot_is_skill_focus_in_a_skill_of_choice() {
    let mut half_elf = build("fighter", 1);
    half_elf.start.race = Some(rules::RaceKey::new("half-elf"));
    let options = engine().options(&half_elf, &race_slot()).unwrap();
    assert_eq!(options.len(), DATA.skills.len());
    assert!(
        options
            .iter()
            .all(|option| option.pick.key() == "skill-focus")
    );
}

#[test]
fn a_slayer_takes_three_combat_style_feats_and_not_a_fourth() {
    let style = |level: u8| pick(&format!("slayer/slayer-talents/combat-style-feat-{level}"));
    let option_feat = |level: u8| SlotId::OptionFeat { level, index: 0 };
    let mut slayer = build("slayer", 12);
    for (level, stage, feat_) in [(4, 2, "dodge"), (8, 6, "toughness"), (10, 10, "iron-will")] {
        slayer.picks.insert(selection_slot(level), style(stage));
        slayer
            .picks
            .insert(option_feat(level), Pick::feat(FeatKey::new(feat_)));
    }
    // The style with the first (ticket 33).
    slayer.picks.insert(
        SlotId::OptionSelection { level: 4, index: 0 },
        pick("ranger/combat-style-feat/archery-feats"),
    );
    assert!(possible(&slayer));
    // A fourth: each window's talent is taken once, and the first window is past.
    slayer.picks.insert(selection_slot(12), style(10));
    assert_eq!(
        why_not(&slayer, &selection_slot(12)),
        Some(Issue::AlreadyHeld)
    );
    // And the first window is past at 6.
    let mut late = build("slayer", 6);
    late.picks.insert(selection_slot(6), style(2));
    assert_eq!(
        why_not(&late, &selection_slot(6)),
        unmet(
            Prerequisite::Not(Box::new(Prerequisite::ClassLevel {
                class: ClassKey::new("slayer"),
                level: 6,
            })),
            None
        )
    );
}

#[test]
fn perfect_mind_gives_bab_15_and_skill_ranks_at_the_mythic_rank_for_prerequisites() {
    let mut wizard = mythic("trickster", 17);
    wizard.levels = (0..17)
        .map(|_| LevelUp::new(ClassKey::new("wizard")))
        .collect();
    wizard.class_choices.clear();
    wizard.path_switch = Some(MythicPathKey::new("gold-dragon"));
    let sheet = engine().evaluate(&wizard);
    let state = sheet.at(17).unwrap();
    assert_eq!(state.bab, 8);
    assert_eq!(state.check(&Prerequisite::BaseAttackBonus(15)), Ok(()));
    assert_eq!(
        state.check(&Prerequisite::SkillRanks {
            skill: SkillKey::new("perception"),
            ranks: 9,
        }),
        Ok(())
    );
    assert!(
        sheet
            .at(16)
            .unwrap()
            .check(&Prerequisite::BaseAttackBonus(15))
            .is_err()
    );
}

#[test]
fn destiny_beyond_birth_removes_a_dwarfs_charisma_penalty() {
    let mut dwarf = build("fighter", 7);
    dwarf.start.race = Some(rules::RaceKey::new("dwarf"));
    dwarf.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("destiny-beyond-birth")),
    );
    assert!(possible(&dwarf));
    let sheet = engine().evaluate(&dwarf);
    assert_eq!(sheet.at(6).unwrap().ability_scores.charisma, 8);
    assert_eq!(sheet.at(7).unwrap().ability_scores.charisma, 10);
    assert!(mythic_feat("destiny-beyond-birth").removes_racial_penalty);
}

#[test]
fn the_razmiri_infiltrator_is_true_neutral_or_evil() {
    let way = DATA
        .classes
        .iter()
        .flat_map(|c| &c.archetypes)
        .flat_map(|a| &a.selections)
        .flat_map(|s| &s.options)
        .find(|o| {
            o.key.as_str() == "arcanist/magic-deceiver/ways-of-the-deceiver/razmiri-infiltrator"
        })
        .unwrap();
    let Some(Prerequisite::Any(alignments)) = &way.prerequisite else {
        panic!("{:?}", way.prerequisite);
    };
    use rules::Alignment as A;
    assert_eq!(
        alignments.iter().cloned().collect::<BTreeSet<_>>(),
        [
            A::TrueNeutral,
            A::LawfulEvil,
            A::NeutralEvil,
            A::ChaoticEvil
        ]
        .into_iter()
        .map(Prerequisite::Alignment)
        .collect()
    );
}

#[test]
fn the_tortured_crusader_gets_3_skill_points_and_its_own_class_skills() {
    let crusader = archetype_build("paladin", "paladin/tortured-crusader", 1);
    let sheet = engine().evaluate(&crusader);
    // Intelligence 10: no modifier.
    assert_eq!(sheet.levels[0].skill_points.available, 3);
    let archetype = class("paladin")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "paladin/tortured-crusader")
        .unwrap();
    assert_eq!(
        archetype.class_skills,
        Some(
            [
                "athletics",
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
                "perception",
                "persuasion"
            ]
            .into_iter()
            .map(SkillKey::new)
            .collect()
        )
    );
}

#[test]
fn the_arcane_trickster_lacks_use_magic_device_and_the_dragon_disciple_has_knowledge_world() {
    assert!(
        !class("arcane-trickster")
            .class_skills
            .contains(&SkillKey::new("use-magic-device"))
    );
    assert!(
        class("dragon-disciple")
            .class_skills
            .contains(&SkillKey::new("knowledge-world"))
    );
}
