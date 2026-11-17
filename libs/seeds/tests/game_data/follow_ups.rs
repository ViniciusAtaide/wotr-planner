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
    // Game: DazzlingDisplayFeature bcbd674ec70ff6f4894bb5f07b6f4095 (PrerequisiteFeature WeaponFocus)
    assert!(possible(&build));
}

#[test]
fn a_human_druid_with_a_companion_takes_boon_companion_at_1() {
    let mut build = build("druid", 1);
    build.start.race = Some(rules::RaceKey::new("human"));
    align(&mut build, rules::Alignment::TrueNeutral);
    build.picks.insert(
        selection_slot(1),
        pick("druid/nature-bond/animal-companion"),
    );
    build
        .picks
        .insert(race_slot(), Pick::feat(FeatKey::new("boon-companion")));
    // Game: CompanionBoon 8fc01f06eab4dd946baa5bc658cac556 (needs AnimalCompanionRank)
    assert!(possible(&build));
}

#[test]
fn every_parameterised_feat_repeats_once_per_value() {
    let parameterised: Vec<&rules::Feat> = DATA
        .feats
        .iter()
        .filter(|f| f.parameter.is_some())
        .collect();
    assert_eq!(parameterised.len(), 16);
    // Game: WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e, a parametrized feature: one rank per value
    for feat in parameterised {
        assert_eq!(feat.repeat(), Repeat::PerValue, "{}", feat.key);
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
    // Game: WeaponFocusGreater 09c9e82965fb4334b984a1e9df3bd088 (parametrized; needs WeaponFocus in the same weapon)
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
    // Game: ExoticWeaponProficiencySelection 9a01b6815d6c3684cb25f30b8bf20932 (SawtoothSabreProficiency
    // 5829f3b193704b8096820875f2eb2368, BAB 1)
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
    // Game: ArmorFocusHeavy c27e6d2b0d33d42439f512c6d9a6a601 (heavy armour proficiency, BAB 1); ClericProficiencies
    // 8c971173613282844888dc20d572cfc9 (light, medium, shields)
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
    // Game: ImprovedCriticalMythicFeat 8bc0190a4ec04bd489eec290aeaa6d07 (needs ImprovedCritical in the same weapon)
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
        ), // Game: ImprovedCriticalMythicFeat 8bc0190a4ec04bd489eec290aeaa6d07
        ("spell-focus-mythic", K::SpellSchool, Some("spell-focus")), // Game: SpellFocusMythicFeat 41fa2470ab50ff441b4cfbb2fc725109
        ("skill-focus-mythic", K::Skill, Some("skill-focus")), // Game: SkillFocusMythicFeatSelection a224819b9492019428c220754208ba3a, a feature per skill needing its Skill Focus (equivalent model)
        ("expanded-arsenal", K::SpellSchool, None), // Game: ExpandedArsenalSchool f137089c48364014aa3ec3b92ccaf2e2
        ("school-mastery", K::SpellSchool, None), // Game: SchoolMasteryMythicFeat ac830015569352b458efcdfae00a948c
    ] {
        let feat = mythic_feat(key);
        assert_eq!(
            (feat.parameter, feat.repeat()),
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
        ("improved-improved-critical", "improved-critical"), // Game: TricksterImprovedImprovedCritical 56f94badbba018b4b8277ce6e2e79e72
        (
            "improved-improved-improved-critical",
            "improved-improved-critical",
        ), // Game: TricksterImprovedImprovedImprovedCritical 006a966007802a0478c9e21007207aac
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
    align(&mut zealot, rules::Alignment::LawfulNeutral);
    assert!(possible(&zealot));
    let saber = || BTreeSet::from([weapon("sawtooth-saber")]);
    // Game: MantisZealotArchetype fc403240fdca4a52a413578169ea7117 (proficiency and Weapon Focus at 1, Weapon
    // Specialization at 4, Greater Weapon Focus at 8, Greater Weapon Specialization at 12, in the sawtooth sabre)
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
    // Game: ChirurgeonArchetype 063440d550b25524abcb8dc2590d0168 (SkillFocusLoreReligion at 5);
    // EnlightenedPhilosopherArchetype 4f2698f04fa63e34b8153fa3a9795649 (SkillFocusKnowledgeWorld at 1)
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
    // Game: ZenArcherArchetype 2b1a58a7917084f49b097e86271df21c (ZenArcherWayOfTheBowSelection at 2, two bows; the
    // specialization at monk 6, as ZenArcherWayOfTheBowShortbowFocus 10e90309eb10d1a4b8c730554aa590a6 adds it)
    let way = "monk/zen-archer/way-of-the-bow/bow";
    let mut archer = archetype_build("monk", "monk/zen-archer", 6);
    align(&mut archer, rules::Alignment::LawfulNeutral);
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

/// The levels agree with the game; the one weapon named with the hex doesn't
/// (`a_battle_master_hex_names_no_weapon_and_8_and_16_each_choose_one`).
// Game: ShamanHexBattleMasterProgression 840651aac9b854a4980ec19e2fced74f (a Weapon Specialization pick at shaman 8, a
// Greater Weapon Focus pick at 16)
#[test]
fn a_battle_master_shaman_specialises_at_8_and_greater_focuses_at_16() {
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
    assert_eq!(values_at(&shaman, 8, "weapon-specialization").len(), 1);
    assert_eq!(values_at(&shaman, 16, "greater-weapon-focus").len(), 1);
}

/// The Battle Master hex names no weapon: its Weapon Specialization at shaman 8 and its Greater Weapon Focus at 16 are
/// each a choice of weapon of their own.
// Game: ShamanHexBattleMasterProgression 840651aac9b854a4980ec19e2fced74f; ShamanHexBattleMasterSelection8
// f18fea9a4f78fc94e842c02e24b1dad3 (the pick at 8, prerequisites ignored)
#[test]
#[ignore = "waits on ticket 278: the game gives the Battle Master hex no weapon, and a weapon choice at shaman 8 and at 16"]
fn a_battle_master_hex_names_no_weapon_and_8_and_16_each_choose_one() {
    assert_eq!(selection_option("shaman/hex/battle-master").parameter, None);
}

#[test]
fn a_sword_saint_takes_weapon_focus_in_a_weapon_of_his_choosing_at_1() {
    let saint = archetype_build("magus", "magus/sword-saint", 1);
    let options = engine().options(&saint, &class_slot(1)).unwrap();
    // Which weapons: `a_sword_saint_chooses_a_melee_weapon`.
    // Game: SwordSaintChosenWeaponFeature c0b4ec0175e3ff940a45fc21f318a39a (Weapon Focus in the chosen weapon, its
    // prerequisites not checked)
    assert!(
        options
            .iter()
            .all(|option| option.pick.key() == "weapon-focus" && option.status.is_ok())
    );
}

/// The Sword Saint's chosen weapon is a melee weapon: no bow, crossbow, ray, bomb or kinetic blast is offered.
// Game: SwordSaintChosenWeaponFeature c0b4ec0175e3ff940a45fc21f318a39a (a weapon category filtered to Melee);
// Kingmaker/Blueprints/Classes/Selection/BlueprintParametrizedFeature.cs ExtractItemsWeaponCategory, WrathDecompiled 6ce4a6b
#[test]
#[ignore = "waits on ticket 283: the game gives the Sword Saint a melee weapon only"]
fn a_sword_saint_chooses_a_melee_weapon() {
    let saint = archetype_build("magus", "magus/sword-saint", 1);
    let options = engine().options(&saint, &class_slot(1)).unwrap();
    for ranged in [
        "long-bow",
        "short-bow",
        "light-crossbow",
        "heavy-crossbow",
        "rays",
        "bomb",
        "kinetic-blast",
    ] {
        let pick = feat_pick("weapon-focus", weapon(ranged));
        assert!(options.iter().all(|option| option.pick != pick), "{ranged}");
    }
}

#[test]
fn a_half_elfs_race_slot_is_skill_focus_in_a_skill_of_choice() {
    let mut half_elf = build("fighter", 1);
    half_elf.start.race = Some(rules::RaceKey::new("half-elf"));
    let options = engine().options(&half_elf, &race_slot()).unwrap();
    // Game: Adaptability 26a668c5a8c22354bac67bcd42e09a3f (the 11 Skill Focus feats), the default of
    // HalfElfHeritageSelection 9df7b68d60544bcf8e5b56c0a4688e04
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
    // Game: RangerStyleSelection2 c6d0da9124735a44f93ac31df803b9a9, RangerStyleSelection6
    // 61f82ba786fe05643beb3cd3910233a8, RangerStyleSelection10 78177315fc63b474ea3cbb8df38fafcd (6 needs 2 and slayer 6,
    // 10 needs 6 and slayer 10)
    // The style with the first (ticket 33).
    slayer.picks.insert(
        SlotId::OptionSelection { level: 4, index: 0 },
        pick("ranger/combat-style-feat/archery-feats"),
    );
    assert!(possible(&slayer));
    // A fourth: each window's talent is taken once.
    // Game: RangerStyleSelection10 78177315fc63b474ea3cbb8df38fafcd (PrerequisiteNoFeature itself)
    slayer.picks.insert(selection_slot(12), style(10));
    assert_eq!(
        why_not(&slayer, &selection_slot(12)),
        Some(Issue::AlreadyHeld)
    );
}

/// The first combat style feat has no upper level: a slayer 6 may take it at 6.
// Game: RangerStyleSelection2 c6d0da9124735a44f93ac31df803b9a9 (no class-level prerequisite, only not held)
#[test]
fn a_slayer_may_take_the_first_combat_style_feat_at_6() {
    let style = |level: u8| pick(&format!("slayer/slayer-talents/combat-style-feat-{level}"));
    let mut late = build("slayer", 6);
    late.picks.insert(selection_slot(6), style(2));
    assert!(possible(&late));
}

/// A wizard 17 on the Trickster path that switched to the Gold Dragon.
fn gold_dragon_wizard_17() -> rules::Build {
    let mut wizard = mythic("trickster", 17);
    wizard.levels = (0..17)
        .map(|_| LevelUp::new(ClassKey::new("wizard")))
        .collect();
    wizard.class_choices.clear();
    wizard.path_switch = Some(MythicPathKey::new("gold-dragon"));
    wizard
}

/// Perfect Mind raises the base attack bonus itself, for attacks and prerequisites alike, to 15 plus the Mythic rank:
/// a wizard 17's 8 becomes 24 at MR 9.
// Game: DragonLevel2BABOverride f4af14e9f4408d44c830d891fee8f4a5, given at GoldenDragonProgression
// a6fbca43902c6194c947546e89af64bd's level 2: an UntypedStackable BaseAttackBonus modifier of -(BAB - 15 - rank), at
// least 0, its rank the MythicLevel (ContextRankConfig), and RecalculateOnStatChange on BaseAttackBonus. The code:
// Kingmaker/UnitLogic/FactLogic/AddContextStatBonus.cs OnTurnOn (Value × Multiplier, at least Minimal),
// Kingmaker/EntitySystem/Properties/PropertyCalculatorComponentHelper.cs GetPropertyValue (the property's getters
// read the fact's own context, so its rank), Kingmaker/EntitySystem/EntityFact.cs Reapply (the modifier comes off
// before it is worked out again), Kingmaker/EntitySystem/Stats/ModifiableValue.cs CalculatePermanentValue (every
// modifier counts toward BAB's permanent value) and Kingmaker/Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs
// GetStatValue (a BAB prerequisite reads it), WrathDecompiled 6ce4a6b. The guide and the game's own text say "at
// least 15": the code says 15 + MR.
#[test]
fn perfect_mind_raises_the_bab_itself_to_15_plus_the_mythic_rank() {
    let sheet = engine().evaluate(&gold_dragon_wizard_17());
    let (before, state) = (sheet.at(16).unwrap(), sheet.at(17).unwrap());
    assert_eq!((before.mythic_rank, before.bab), (7, 8));
    assert_eq!((state.mythic_rank, state.bab), (9, 24));
    assert_eq!(state.check(&Prerequisite::BaseAttackBonus(24)), Ok(()));
    assert!(before.check(&Prerequisite::BaseAttackBonus(15)).is_err());
}

/// Perfect Mind's skill floor raises each skill's total to the Mythic rank, not its ranks, so a skill-rank
/// prerequisite doesn't see it.
// Game: GoldenDragonSharedSkillUMD 4e0b919e4fbd85142bce959fae129d1a (AddGoldenDragonSkillBonus on each skill);
// Kingmaker/UnitLogic/FactLogic/AddGoldenDragonSkillBonus.cs OnTurnOn (a modifier of max(MR - BaseValue, 0)) and
// Kingmaker/Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs GetStatValue (a skill's BaseValue: its ranks),
// WrathDecompiled 6ce4a6b
#[test]
fn perfect_minds_skill_floor_raises_totals_and_meets_no_skill_rank_prerequisite() {
    let engine = engine();
    let sheet = engine.evaluate(&gold_dragon_wizard_17());
    let state = sheet.at(17).unwrap();
    let perception = SkillKey::new("perception");
    assert_eq!(state.skill_ranks(&perception), 0);
    assert!(
        state
            .check(&Prerequisite::SkillRanks {
                skill: perception.clone(),
                ranks: 1,
            })
            .is_err()
    );
    let totals = engine.totals(&sheet.levels[16]);
    assert_eq!(totals.skill(&perception).unwrap().ranks, 9);
}

#[test]
fn destiny_beyond_birth_removes_a_dwarfs_charisma_penalty() {
    let mut dwarf = build("fighter", 7);
    dwarf.start.race = Some(rules::RaceKey::new("dwarf"));
    dwarf.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("destiny-beyond-birth")),
    );
    // Game: DwarfRace c4faf439f0e70bd40b5e36ee80d06be7 (the -2 Charisma unless DestinyBeyondBirthMythicFeat
    // 325f078c584318849bfe3da9ea245b9d is held)
    assert!(possible(&dwarf));
    let sheet = engine().evaluate(&dwarf);
    assert_eq!(sheet.at(6).unwrap().ability_scores.charisma, 8);
    assert_eq!(sheet.at(7).unwrap().ability_scores.charisma, 10);
    assert!(mythic_feat("destiny-beyond-birth").removes_racial_penalty);
}

/// The Razmiri Infiltrator is any alignment but a good one (it also needs a deity, not asserted here).
// Game: MagicDeceiverWayOfRazmiriInfiltratorProgression cd96109ca30b45dcb6d7390945c8487d (LN, TN, CN, LE, NE, CE)
#[test]
#[ignore = "waits on ticket 267: the game gives the Razmiri Infiltrator LN, TN, CN, LE, NE and CE, beside deity \
            features no entry maps, so 308e's writer leaves it whole"]
fn the_razmiri_infiltrator_is_any_alignment_but_good() {
    let way = DATA
        .options
        .iter()
        .find(|o| {
            o.key.as_str() == "arcanist/magic-deceiver/ways-of-the-deceiver/razmiri-infiltrator"
        })
        .unwrap();
    let alignments: BTreeSet<&Prerequisite> = way
        .prerequisite
        .iter()
        .flat_map(|prerequisite| prerequisite.leaves())
        .filter(|leaf| matches!(leaf, Prerequisite::Alignment(_)))
        .collect();
    use rules::Alignment as A;
    let non_good = [
        A::LawfulNeutral,
        A::TrueNeutral,
        A::ChaoticNeutral,
        A::LawfulEvil,
        A::NeutralEvil,
        A::ChaoticEvil,
    ]
    .map(Prerequisite::Alignment);
    assert_eq!(alignments, non_good.iter().collect());
}

#[test]
fn the_tortured_crusader_gets_3_skill_points_and_its_own_class_skills() {
    // Game: TorturedCrusader f91ea9596e6f42f8a23803c1b3e976b4 (AddSkillPoints 1 on PaladinClass
    // bfa11238e7ae3544bbeb4d0b92e897ec's 2; ReplaceClassSkills with these seven): the user's fact, which the game bears out
    let crusader = archetype_build("paladin", "paladin/tortured-crusader", 1);
    let sheet = engine().evaluate(&crusader);
    // Intelligence 10: no modifier.
    assert_eq!(sheet.levels[0].skill_points.available, 3);
    let archetype = DATA
        .archetypes_of(&class("paladin").key)
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
    // Game: ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3; DragonDiscipleClass 72051275b1dbb2d42ba9118237794f7c
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
