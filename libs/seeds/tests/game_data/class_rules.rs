//! Class rules (ticket 28).

use std::collections::BTreeSet;

use rules::{
    Advancement, Build, CharacterState, ClassChoice, ClassKey, CountsAs, CountsToward, FeatKey,
    FeatureKey, MythicPathKey, Parameter, Pick, SkillKey, Story,
};

use crate::support::*;

#[test]
fn signifer_loremaster_and_winter_witch_advance_one_class_arcane_or_divine() {
    let either: BTreeSet<&str> = DATA
        .all_classes()
        .filter(|class| class.advances == Advancement::Either)
        .map(|class| class.key.as_str())
        .collect();
    // Game: one ReplaceSpellbook selection each, mixing arcane and divine books: HellknightSigniferSpellbook
    // 68782aa7a302b6d43a42a71c6e9b5277, LoremasterSpellbookSelection 7a28ab4dfc010834eabc770152997e87,
    // WinterWitchSpellbookSelection ea20b26d9d0ede540af3c74246dade41
    assert_eq!(
        either,
        BTreeSet::from(["hellknight-signifer", "loremaster", "winter-witch"])
    );
    // Game: MysticTheurgeProgression 08c1075ef2786ef4fae11e82698a16e0 (a divine and an arcane spellbook selection)
    assert_eq!(class("mystic-theurge").advances, Advancement::Each);

    let mut both = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "cleric",
        "cleric",
        "cleric",
        "cleric",
        "cleric",
        "loremaster",
    ]);
    let choose = |build: &mut Build, advances: &[&str]| {
        build.class_choices.insert(
            ClassKey::new("loremaster"),
            ClassChoice {
                archetype: None,
                advances: advances.iter().map(|key| ClassKey::new(*key)).collect(),
            },
        );
    };
    // What the Loremaster's entry needs: Skill Focus (Knowledge (Arcana)), a metamagic feat and 3rd-level spells.
    // Game: LoremasterClass 4a7c05adfbaf05446a6bf664d28fb103 (Skill Focus in Knowledge (Arcana) or (World), one of 10
    // metamagic feats, 3rd-level arcane or divine spells; the book advanced is one, from LoremasterSpellbookSelection
    // 7a28ab4dfc010834eabc770152997e87)
    both.picks.insert(
        general_slot(1),
        feat_pick(
            "skill-focus",
            Parameter::Skill(SkillKey::new("knowledge-arcana")),
        ),
    );
    both.picks.insert(
        general_slot(3),
        Pick::feat(FeatKey::new("metamagic-extend-spell")),
    );
    choose(&mut both, &["wizard", "cleric"]);
    assert!(!engine().is_possible(&both));
    choose(&mut both, &["cleric"]);
    assert!(engine().is_possible(&faithful(both)));
}

#[test]
fn eldritch_knight_levels_count_as_fighter_levels_for_feats() {
    // Game: EldritchKnightDiverseTraining 983e8ad193160b44da80b38af4927e75 (ClassLevelsForPrerequisites: Eldritch Knight
    // levels as fighter levels for the prerequisites of the feats of BasicFeatSelection 247a4068296e8be42890143f451b4b45);
    // PrerequisiteClassLevel.GetClassLevel, WrathDecompiled 6ce4a6b
    assert_eq!(
        class("eldritch-knight").counts_as,
        BTreeSet::from([CountsAs {
            class: ClassKey::new("fighter"),
            toward: CountsToward::FeatPrerequisites,
        }])
    );
    // Wizard 5, Fighter 1, Eldritch Knight 3: fighter level 4 for feats at character level 9.
    let mut build = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "fighter",
        "eldritch-knight",
        "eldritch-knight",
        "eldritch-knight",
    ]);
    build.picks.insert(
        class_slot(6),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
    );
    let specialization = Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword"));
    let status = |level| {
        engine()
            .options(&build, &general_slot(level))
            .unwrap()
            .into_iter()
            .find(|option| option.pick == specialization)
            .unwrap()
            .status
    };
    // Game: WeaponSpecialization 31470b17e8446ae4ea0dacd6c5817d86 (Fighter 4 and Weapon Focus; in BasicFeatSelection)
    assert!(status(7).is_err(), "fighter 1 / Eldritch Knight 1");
    assert_eq!(status(9), Ok(()));
}

#[test]
fn winter_witch_levels_stack_with_witch_levels_for_major_and_grand_hexes() {
    // Game: WitchBetterHexProgression 38d01811fcb32444a8fe372c029fa0c6 (classes Witch and Winter Witch: Major Hex at
    // 10, Grand Hex at 18)
    let toward = |feature: &str| CountsAs {
        class: ClassKey::new("witch"),
        toward: CountsToward::Feature(FeatureKey::new(feature)),
    };
    assert_eq!(
        class("winter-witch").counts_as,
        BTreeSet::from([toward("major-hexes"), toward("grand-hexes")])
    );
    let mut keys = vec!["witch"; 9];
    keys.push("winter-witch");
    let sheet = engine().evaluate(&classes(&keys));
    let major = FeatureKey::new("major-hexes");
    assert_eq!(sheet.at(9).unwrap().feature_count(&major), 0);
    assert_eq!(sheet.at(10).unwrap().feature_count(&major), 1);
}

#[test]
fn the_gold_dragon_breaths_story_conditions_are_notes_that_never_block() {
    let engine = engine();
    let breath = |key: &str| pick(&format!("gold-dragon/improved-gold-dragon-breath/{key}"));
    let mut build = mythic("trickster", 17);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    let entries = engine.slot_entries(&build, &path_selection(9)).unwrap();
    // Game: GoldDragonBreathAnnihilating e8b9aad5295c49539f1bbf901d314964 and GoldDragonBreathDecaying
    // c606a6897125480ebde14b63bb8fb18e need CorruptedGoldenDragonFeature; GoldDragonBreathEnlighting
    // 0351c6c95adc48b3b39073167094cd19 and GoldDragonBreathRejuvinating ef3690c36872422d9bfadeb276b5c3fe need its absence.
    // That a story condition is a note and never blocks is the planner's policy, not a game fact.
    for (key, story) in [
        ("annihilating-breath", Story::VowOfDahak),
        ("decaying-breath", Story::VowOfDahak),
        ("enlightening-breath", Story::Uncorrupted),
        ("rejuvenating-breath", Story::Uncorrupted),
    ] {
        let entry = entries
            .iter()
            .find(|entry| entry.options[0].option.pick == breath(key))
            .unwrap_or_else(|| panic!("no entry {key}"));
        assert_eq!(
            entry.story_note,
            Some(format!("(Story: {})", story.name())),
            "{key}"
        );
        if matches!(key, "annihilating-breath" | "enlightening-breath") {
            assert_eq!(entry.options[0].option.status, Ok(()), "{key}");
        }
    }
}

#[test]
fn a_crusaders_bonus_feats_still_skip_every_prerequisite_and_the_data_says_so() {
    // Weapon Focus needs BAB 1; a crusader cleric 1 has 0.
    // Game: CrusaderArchetype 6bfb7e74b530f3749b590286dd2b9b30 adds CrusaderBonusFeat1 c5357c05cf4f8414ebd0a33e534aec50
    // at 1 (IgnorePrerequisites, Weapon Focus listed); BlueprintFeatureSelection.IsIgnorePrerequisites, WrathDecompiled
    // 6ce4a6b. The note in the Crusader's feat-list files is not a game fact.
    // A weapon he is proficient with (ticket 251): the cleric's simple weapons.
    // Game: ClericProficiencies 8c971173613282844888dc20d572cfc9; WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e
    // (RequireProficiency).
    let crusader = archetype_build("cleric", "cleric/crusader", 1);
    let focus = Pick::feat_for(FeatKey::new("weapon-focus"), weapon("heavy-mace"));
    let status = engine()
        .options(&crusader, &class_slot(1))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == focus)
        .unwrap()
        .status;
    assert_eq!(status, Ok(()));

    let lists: BTreeSet<String> = DATA
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "cleric/crusader")
        .unwrap()
        .levels
        .iter()
        .flat_map(|row| &row.feat_slots)
        .filter_map(|slot| match slot {
            rules::FeatSlot::OneOf { list } => Some(list.to_string()),
            _ => None,
        })
        .collect();
    assert!(!lists.is_empty());
    for list in lists {
        let file = seeds::game_data_dir().join(format!("feat-lists/{list}.ron"));
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(
            text.contains("// Crusader Bonus Feats:"),
            "the note on the Crusader's lists"
        );
    }
}

#[test]
fn an_assassin_is_true_neutral_or_evil() {
    use rules::Alignment::*;
    // Game: AssassinClass eb284ea8d40a2d045911f525eb96c43d (TrueNeutral or Evil; Stealth 5)
    let assassin = class("assassin").prerequisite.as_ref().unwrap();
    let allowed = [
        LawfulGood,
        NeutralGood,
        ChaoticGood,
        LawfulNeutral,
        TrueNeutral,
        ChaoticNeutral,
        LawfulEvil,
        NeutralEvil,
        ChaoticEvil,
    ]
    .into_iter()
    .filter(|alignment| {
        let state = CharacterState {
            alignment: Some(*alignment),
            character_level: 6,
            skill_ranks: [(SkillKey::new("stealth"), 5)].into(),
            ..CharacterState::default()
        };
        state.check(assassin).is_ok()
    })
    .collect::<Vec<_>>();
    assert_eq!(allowed, [TrueNeutral, LawfulEvil, NeutralEvil, ChaoticEvil]);
}

fn nature_bond(archetype: Option<&str>, option: Option<&str>) -> CharacterState {
    let mut druid = match archetype {
        Some(archetype) => archetype_build("druid", archetype, 1),
        None => build("druid", 1),
    };
    if let Some(option) = option {
        druid.picks.insert(
            selection_slot(1),
            pick(&format!("druid/nature-bond/{option}")),
        );
    }
    engine().evaluate(&druid).at(1).cloned().unwrap()
}

#[test]
fn a_druid_chooses_an_animal_companion_or_a_domain_at_level_1() {
    // Game: DruidBondSelection 3830f3630a33eba49b60f511b4c8f2a8 (DruidDomainSelection, AnimalCompanionSelectionDruid)
    let offered: BTreeSet<String> = offered(&build("druid", 1), 1).into_keys().collect();
    assert_eq!(
        offered,
        BTreeSet::from([
            "druid/nature-bond/animal-companion".to_owned(),
            "druid/nature-bond/domain".to_owned()
        ])
    );
    // The archetypes whose table keeps both columns choose too; the others have their companion, their domain
    // or neither.
    let chooses = |archetype: &str| {
        let row = DATA
            .archetypes_of(&class("druid").key)
            .find(|a| a.key.as_str() == archetype)
            .unwrap()
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap();
        row.selections
            .contains(&rules::FeatureSelectionKey::new("druid/nature-bond"))
    };
    // Game: DefenderOfTheTrueWorld 782c46afacd88d448afba6178471a744 and FeyspeakerArchetype
    // da69747aa3dd0044ebff5f3d701cdde3 keep DruidBondSelection at 1; BlightDruid 35a3b7bfc663ac74aa8bb50adfe70813,
    // DrovierArchetype a6bf603aa4d666f4f86dab0289e2048c, ElementalRampagerArchetype f815594bd1e3454182022d375bf70fd1,
    // PrimalDruidArchetype c1c86e2997fd4257a13ef5601b5dc6dd and WinterChildArchetype 9f835c539f6e4a96a7b64461f4267d06
    // remove it
    for archetype in ["druid/defender-of-the-true-world", "druid/feyspeaker"] {
        assert!(chooses(archetype), "{archetype}");
    }
    for archetype in [
        "druid/blight-druid",
        "druid/drovier",
        "druid/elemental-rampager",
        "druid/primal-druid",
        "druid/winter-child",
    ] {
        assert!(!chooses(archetype), "{archetype}");
    }
}

/// Domain Zealot's prerequisite.
fn domain_zealot() -> rules::Prerequisite {
    DATA.mythic_abilities
        .iter()
        .find(|a| a.key.as_str() == "domain-zealot")
        .and_then(|a| a.prerequisite.clone())
        .expect("Domain Zealot has a prerequisite")
}

/// A druid 1 of the Weather domain, through the Nature Bond.
fn weather_druid() -> CharacterState {
    let weather = with_pick(
        with_option(build("druid", 1), 1, 0, "druid/nature-bond/domain"),
        option_slot(1, 0),
        "cleric/domain/weather",
    );
    at(&weather, 1)
}

#[test]
fn domain_zealot_and_boon_companion_follow_a_druids_nature_bond() {
    let domain_zealot = domain_zealot();
    let boon_companion = feat("boon-companion").prerequisite.clone().unwrap();
    // Game: DomainMastery 2de64f6a1f2baee4f9b7e52e3f046ec5 (no qualifying fact without a domain); CompanionBoon
    // 8fc01f06eab4dd946baa5bc658cac556 (needs AnimalCompanionRank)
    assert!(
        nature_bond(None, Some("domain"))
            .check(&domain_zealot)
            .is_err()
    );
    assert!(weather_druid().check(&boon_companion).is_err());

    let companion = nature_bond(None, Some("animal-companion"));
    assert!(companion.check(&domain_zealot).is_err());
    assert_eq!(companion.check(&boon_companion), Ok(()));

    assert!(nature_bond(None, None).check(&domain_zealot).is_err());
    // Game: PrimalDruidArchetype c1c86e2997fd4257a13ef5601b5dc6dd (no domain)
    assert!(
        nature_bond(Some("druid/primal-druid"), None)
            .check(&domain_zealot)
            .is_err()
    );
}

/// The game's Domain Zealot needs a cleric's or inquisitor's domains selection, the Divine Hunter's, Pentamic Faith,
/// Trickster Lore (Religion) 2 or a divine spark: a druid's domain (Weather through the Nature Bond, a Blight Druid's
/// Death, a Winter Child's) is none of these.
// Game: DomainMastery 2de64f6a1f2baee4f9b7e52e3f046ec5 (DomainsSelection 48525e5da45c9c243a343fc6545dbdb9,
// DivineHunterDomainsSelection, HellknightDisciplinePentamicFaith, TricksterLoreReligionTier2Parametrized,
// FakeDivineSparkFeature); DruidDomainSelection 5edfe84c93823d04f8c40ca2b4e0f039, BlightDruidBond
// 096fc02f6cc817a43991c4b437e12b8e and WinterChildArchetype 9f835c539f6e4a96a7b64461f4267d06 give none of them;
// LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b (a selection is added as a fact)
#[test]
#[ignore = "waits on ticket 265: the game closes Domain Zealot to every druid, Blight Druid and Winter Child"]
fn domain_zealot_is_closed_to_a_druids_domain() {
    let domain_zealot = domain_zealot();
    assert!(weather_druid().check(&domain_zealot).is_err());
    let mut blight = archetype_build("druid", "druid/blight-druid", 1);
    blight
        .picks
        .insert(selection_slot(1), pick("cleric/domain/death"));
    assert!(
        engine()
            .evaluate(&blight)
            .at(1)
            .unwrap()
            .check(&domain_zealot)
            .is_err()
    );
    assert!(
        nature_bond(Some("druid/winter-child"), None)
            .check(&domain_zealot)
            .is_err()
    );
}

/// The Titan Fighter has no Weapon Training: no Weapon Training at 5, so what needs it, such as Advanced Weapon
/// Training, stays closed. Weapon Mastery at 20 stays. (The game's Incredible Heft takes Armor Training's place at 3,
/// 7, 11, 15 and 19, which the user approved over ticket 131's reading; it is the rows' concern, not this test's.)
#[test]
fn the_titan_fighter_has_no_weapon_training() {
    let advanced_weapon_training = |build: &Build| {
        engine()
            .options(build, &general_slot(7))
            .unwrap()
            .into_iter()
            .find(|option| option.pick.key() == "advanced-weapon-training")
            .unwrap()
            .status
    };
    // Game: AdvancedWeaponTraining1 3aa4cbdd4af5ba54888b0dc7f07f80c4 (WeaponTrainingSelection and Fighter 5);
    // FighterProgression b50e94b57be32f74892f381ae2a8905a (Weapon Training at 5)
    let fighter = build("fighter", 7);
    assert!(holds(&at(&fighter, 5), "fighter/weapon-training"));
    assert_eq!(advanced_weapon_training(&fighter), Ok(()));

    // Game: TitanFighterArchetype 88a9dbb92a674ed995bae800896795f5 (removes WeaponTrainingSelection at 5, 9, 13 and 17
    // and its rank-ups; keeps Weapon Mastery)
    let titan = archetype_build("fighter", "fighter/titan-fighter", 20);
    for level in 1..=20 {
        assert!(
            !holds(&at(&titan, level), "fighter/weapon-training"),
            "level {level}"
        );
    }
    assert!(holds(&at(&titan, 20), "fighter/weapon-mastery"));
    let titan_7 = archetype_build("fighter", "fighter/titan-fighter", 7);
    assert!(advanced_weapon_training(&titan_7).is_err());
}
