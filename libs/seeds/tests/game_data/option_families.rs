//! Domains, mysteries, blessings and elements (ticket 31), Impossible Domain and the druid (ticket 36), and the
//! Dual-Cursed Oracle's second curse (ticket 67).

use std::collections::BTreeSet;

use rules::{
    Build, ClassChoice, ClassKey, FeatKey, Issue, LevelUp, Pick, SelectionOptionKey, SlotId,
    SlotKind,
};

use crate::support::*;

// ---- domains, mysteries, blessings and elements (ticket 31) -------------------------------------------------

#[test]
fn a_cleric_8_holds_both_domains_second_powers() {
    // Fire and Glory: Sarenrae's (ticket 86: the deity offers both).
    let build = cleric(["fire", "glory"], 8);
    assert!(engine().is_possible(&build));
    // Game: FireDomainProgression 881b2137a1779294c8956fe5b497cc35 (1, and its resistance at 6) and
    // GloryDomainProgression f0a61a043bcdf0f4c8efc59962afafb8 (1, and its aura at 8).
    for power in [
        "cleric/domain/fire/fire-bolt",
        "cleric/domain/glory/touch-of-glory",
    ] {
        assert!(holds(&at(&build, 1), power), "{power}");
    }
    from_level(&build, 6, "cleric/domain/fire/fire-resistance");
    from_level(&build, 8, "cleric/domain/glory/aura-of-heroism");
    // Two picks of one domain aren't two domains.
    // Game: FireDomainProgression 881b2137a1779294c8956fe5b497cc35 (Ranks 1) and FireDomainProgressionSecondary
    // 2bd6aa3c4979fd045bbbda8da586d3fb shut each other out.
    let twice = cleric(["fire", "fire"], 1);
    assert!(why_not(&twice, &SlotId::Selection { level: 1, index: 1 }).is_some());
}

#[test]
fn a_cleric_archetype_that_may_only_select_a_single_domain_picks_one() {
    // Game: CrusaderArchetype 6bfb7e74b530f3749b590286dd2b9b30 removes SecondDomainsSelection
    // 43281c3d7fe18cc4d91928395837cd1e; EcclesitheurgeArchetype 472af8cb3de628f4a805dc4a038971bc keeps both.
    let crusader = archetype_build("cleric", "cleric/crusader", 1);
    let sheet = engine().evaluate(&crusader);
    let domain_slots = sheet.levels[0]
        .slots
        .iter()
        .filter(|slot| {
            slot.kind == SlotKind::Selection(rules::FeatureSelectionKey::new("cleric/domain"))
        })
        .count();
    assert_eq!(domain_slots, 1);
    let ecclesitheurge = engine().evaluate(&archetype_build("cleric", "cleric/ecclesitheurge", 1));
    assert_eq!(
        ecclesitheurge.levels[0]
            .slots
            .iter()
            .filter(|slot| slot.kind
                == SlotKind::Selection(rules::FeatureSelectionKey::new("cleric/domain")))
            .count(),
        2
    );
}

#[test]
fn the_war_domain_grants_a_bonus_combat_feat_at_8() {
    // War and Strength: Gorum's (ticket 86: the deity offers both).
    let build = cleric(["war", "strength"], 8);
    let slot = SlotId::OptionFeat { level: 8, index: 0 };
    // Game: WarDomainProgression 8d454cbb7f25070419a1c8eaf89b5be5 at 8: WarDomainGreaterFeatSelection
    // 79c6421dbdb028c4fa0c31b8eea95f16, of the combat feats.
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.slot(&slot).expect("the War domain's slot").kind,
        feat_kind("combat-feats")
    );
    let feats = feats_offered(&build, &slot);
    assert!(feats.contains("power-attack"));
    assert!(!feats.contains("skill-focus"), "not a combat feat");
    assert!(
        engine()
            .evaluate(&cleric(["war", "fire"], 7))
            .slot(&SlotId::OptionFeat { level: 7, index: 0 })
            .is_none()
    );
}

#[test]
fn the_animal_domains_companion_comes_at_4_and_opens_boon_companion() {
    let boon_companion = feat("boon-companion").prerequisite.clone().unwrap();
    let animal = cleric(["animal", "fire"], 4);
    // Game: AnimalDomainProgression 23d2f87aa54c89f418e68e790dba11e0 at 4: AnimalCompanionSelectionDomain
    // 2ecd6c64683b59944a7fe544033bb533.
    from_level(&animal, 4, "cleric/domain/animal/animal-companion");
    assert!(at(&animal, 3).check(&boon_companion).is_err());
    assert_eq!(at(&animal, 4).check(&boon_companion), Ok(()));
    // Another domain has none.
    assert!(
        at(&cleric(["fire", "luck"], 4), 4)
            .check(&boon_companion)
            .is_err()
    );
    // The Nature mystery's Animal Companion revelation.
    let nature = with_option(build("oracle", 1), 1, 0, "oracle/mystery/nature");
    let nature = with_pick(
        nature,
        revelation_slot(1),
        "oracle/revelations/nature/animal-companion",
    );
    assert!(engine().is_possible(&nature));
    assert_eq!(at(&nature, 1).check(&boon_companion), Ok(()));
}

#[test]
fn a_druid_with_the_weather_domain_holds_lightning_lord_at_8_and_others_dont() {
    let domain = |which: &str, levels| {
        let druid = with_option(druid(None, levels), 1, 0, "druid/nature-bond/domain");
        with_pick(druid, option_slot(1, 0), &format!("cleric/domain/{which}"))
    };
    // Game: DruidDomainSelection 5edfe84c93823d04f8c40ca2b4e0f039, eight domains; WeatherDomainProgressionDruid
    // 4a3516fdc4cda764ebd1279b22d10205 at 8: WeatherDomainGreaterFeature 8e44306af595c8d44aad2f1260fd7be2.
    assert_eq!(
        offered_at(&domain("weather", 1), &option_slot(1, 0)).len(),
        8,
        "Air, Animal, Earth, Fire, Ice, Plant, Water and Weather"
    );
    let weather = domain("weather", 8);
    assert!(engine().is_possible(&weather));
    from_level(&weather, 8, "cleric/domain/weather/lightning-lord");
    // A druid with an animal companion holds no domain powers and no second companion at 4.
    let companion = with_option(
        build("druid", 8),
        1,
        0,
        "druid/nature-bond/animal-companion",
    );
    let state = at(&companion, 8);
    assert!(
        state
            .features()
            .all(|(f, _)| !f.as_str().starts_with("cleric/domain/"))
    );
}

#[test]
fn a_blight_druid_skips_the_animal_domains_companion_and_a_winter_child_holds_ice_with_no_pick() {
    let blight = with_option(
        druid(Some("druid/blight-druid"), 4),
        1,
        0,
        "cleric/domain/animal",
    );
    assert!(engine().is_possible(&blight));
    // Game: AnimalCompanionSelectionDomain 2ecd6c64683b59944a7fe544033bb533 has NoSelectionIfAlreadyHasFeature
    // BlightDruidBond 096fc02f6cc817a43991c4b437e12b8e (equivalent model: the planner skips the row).
    assert!(!holds(
        &at(&blight, 4),
        "cleric/domain/animal/animal-companion"
    ));
    // The domain's Lore (Nature) sits on that skipped row too; a druid has it anyway.
    assert!(has_class_skill(&at(&blight, 4), "lore-nature"));

    // Game: WinterChildDomainSelection 79922e274bf4444b93ea77ae0f876bd2 lists eight, each but Ice barred to the
    // Winter Child, so IceSubdomainProgressionDruid 2ef9e78640ae4d99a7cf2ac1eedde69a: its power at 1, its greater
    // at 8 (equivalent model: held with no pick).
    let winter = druid(Some("druid/winter-child"), 8);
    assert!(engine().is_possible(&winter));
    assert!(holds(&at(&winter, 1), "cleric/domain/ice/icicle"));
    from_level(&winter, 8, "cleric/domain/ice/body-of-ice");
}

#[test]
fn an_inquisitors_domain_runs_on_inquisitor_levels() {
    let build = with_option(build("inquisitor", 6), 1, 0, "cleric/domain/luck");
    assert!(engine().is_possible(&build));
    // Game: LuckDomainProgression 8bd8cfad69085654b9118534e4aa215e lists InquisitorClass; its 6th-level power.
    from_level(&build, 6, "cleric/domain/luck/divine-fortune");
    let huntsmaster = archetype_build("inquisitor", "inquisitor/sacred-huntsmaster", 1);
    // Game: AnimalDomainProgression 23d2f87aa54c89f418e68e790dba11e0 and ScalykindDomainProgression
    // 67034c0985b240e5ad6e41e905461951 have PrerequisiteNoArchetype SacredHuntsmasterArchetype
    // 46eb929c8b6d7164188eb4d9bcd0a012; FaithHunterArchetype ac87f40932cfd324c90b441f93857c40 removes the domains.
    let domains = offered(&huntsmaster, 1);
    assert!(!domains.contains_key("cleric/domain/animal"));
    assert!(!domains.contains_key("cleric/domain/scalykind"));
    assert!(domains.contains_key("cleric/domain/luck"));
    // "Cannot select a domain."
    let faith_hunter =
        engine().evaluate(&archetype_build("inquisitor", "inquisitor/faith-hunter", 1));
    assert!(faith_hunter.levels[0].slots.iter().all(
        |slot| !matches!(&slot.kind, SlotKind::Selection(s) if s.as_str().ends_with("/domain"))
    ));
}

#[test]
fn an_oracles_revelation_slot_offers_only_its_mysterys_revelations() {
    // Besides the Dual-Cursed Oracle's Fortune and Misfortune, which no mystery admits (ticket 243).
    let mystery_free = |key: &&String| !key.ends_with("/fortune") && !key.ends_with("/misfortune");
    // Game: each option of OracleRevelationSelection 60008a10ad7ad6543b1f63016741a5d2 needs its mystery's feature.
    let battle = offered_at(&oracle("battle", 1), &revelation_slot(1));
    assert!(battle.contains("oracle/revelations/battle/weapon-mastery"));
    assert!(
        battle
            .iter()
            .filter(mystery_free)
            .all(|key| key.starts_with("oracle/revelations/battle/")),
        "{battle:?}"
    );
    let flame = offered_at(&oracle("flame", 1), &revelation_slot(1));
    assert!(
        flame
            .iter()
            .filter(mystery_free)
            .all(|key| key.starts_with("oracle/revelations/flame/")),
        "{flame:?}"
    );
    // "Prerequisites: Level 7"
    // Game: OracleRevelationCombatHealer db1d9829383e78841a6f1145411a54b4 needs oracle 7.
    let mut early = oracle("battle", 3);
    early.picks.insert(
        revelation_slot(3),
        pick("oracle/revelations/battle/combat-healer"),
    );
    assert!(why_not(&early, &revelation_slot(3)).is_some());
}

#[test]
fn weapon_mastery_grants_improved_critical_at_8_in_its_weapon() {
    let mut build = oracle("battle", 12);
    build.picks.insert(
        revelation_slot(1),
        Pick::option_for(
            SelectionOptionKey::new("oracle/revelations/battle/weapon-mastery"),
            weapon("heavy-mace"),
        ),
    );
    assert!(engine().is_possible(&build));
    // A weapon the oracle is proficient with (ticket 251): Weapon Mastery's 1st-level pick is Weapon Focus
    // (OracleRevelationWeaponMasteryWeaponFocusSelection 7e360e025592a734bb6c3ae1210824f3), which requires it.
    // Game: OracleRevelationWeaponMastery 0a4c3556355747241b2d3dcc0a88ec10: Weapon Focus at 1, Improved Critical at
    // 8, Greater Weapon Focus at 12, in the chosen weapon.
    assert!(at(&build, 1).has_feat_for(&FeatKey::new("weapon-focus"), &weapon("heavy-mace")));
    let critical = FeatKey::new("improved-critical");
    assert!(!at(&build, 7).has_feat_for(&critical, &weapon("heavy-mace")));
    assert!(at(&build, 8).has_feat_for(&critical, &weapon("heavy-mace")));
    assert!(!at(&build, 8).has_feat_for(&critical, &weapon("dagger")));
    assert!(
        at(&build, 12).has_feat_for(&FeatKey::new("greater-weapon-focus"), &weapon("heavy-mace"))
    );
}

#[test]
fn an_oracle_holds_the_mysterys_class_skill_and_final_revelation() {
    let build = oracle("battle", 20);
    // Game: OracleBattleMysteryFeature 6b8e676943cb91648b21b7ca72fbb833 adds Perception as a class skill; its final
    // revelation at 20. DivineHerbalistBattleMysteryFeature c880da2dea6b54f4bad7d1f4b79c2d63 adds no class skill;
    // WIndWhispererArchetype 77769e43e63647c2b32298efcc577aca adds OracleWindMysteryFeature
    // 7c1fdd831af747b47bb2cce0051f309b.
    assert!(has_class_skill(&at(&build, 1), "perception"));
    from_level(&build, 20, "oracle/mystery/battle/battle-final-revelation");
    // "he does not gain additional class skills from his choice of mystery" (Divine Herbalist).
    let herbalist = with_option(
        archetype_build("oracle", "oracle/divine-herbalist", 1),
        1,
        0,
        "oracle/mystery/battle",
    );
    assert!(engine().is_possible(&herbalist));
    assert!(!has_class_skill(&at(&herbalist, 1), "perception"));
    // "Must select the Wind Mystery": held with no pick.
    let whisperer = archetype_build("oracle", "oracle/wind-whisperer", 1);
    assert_eq!(
        at(&whisperer, 1).option_count(&SelectionOptionKey::new("oracle/mystery/wind")),
        1
    );
}

#[test]
fn a_warpriest_10_holds_both_major_blessings() {
    let build = build("warpriest", 10);
    let build = with_option(build, 1, 0, "warpriest/blessings/fire");
    // Fire and Sun: Sarenrae's (ticket 86: the deity offers both).
    let build = with_option(build, 1, 1, "warpriest/blessings/sun");
    assert!(engine().is_possible(&build));
    // Game: FireBlessingFeature 2368212fa3856d74589e924d3e2074d8's AddFeatureOnClassLevel 10 (the major blessing);
    // FeralChampionArchetype f68ca492c9c15e241ab73735fbd0fb9f removes both blessing selections and adds
    // FeralChampionBlessing 6c52c34b462be9047b83ae7f2a3b5822.
    for blessing in ["fire", "sun"] {
        assert!(holds(
            &at(&build, 1),
            &format!("warpriest/blessings/{blessing}/minor-blessing")
        ));
        from_level(
            &build,
            10,
            &format!("warpriest/blessings/{blessing}/major-blessing"),
        );
    }
    // No more blessing picks at 10.
    let sheet = engine().evaluate(&build);
    assert!(sheet.levels[9].slots.iter().all(
        |slot| !matches!(&slot.kind, SlotKind::Selection(s) if s.as_str() == "warpriest/blessings")
    ));
    // "A Feral Champion must take the Animal Domain blessing and does not gain a second."
    let feral = archetype_build("warpriest", "warpriest/feral-champion", 10);
    assert!(engine().is_possible(&feral));
    from_level(
        &feral,
        10,
        "warpriest/blessings/animal-domain/major-blessing",
    );
}

#[test]
fn a_kineticist_holds_the_elements_class_skills_list_and_defensive_talent() {
    let build = kineticist("earth", 2);
    assert!(engine().is_possible(&build));
    // Game: ElementalFocusEarth c6816ad80a3df9c4ea7d3b012b06bacd: Athletics and Knowledge (World), Flesh of Stone
    // (FleshOfStoneFeature a275b35f282601944a97e694f6bc79f8) at 2; KineticKnightElementalFocusEarth
    // d2a93ab18fcff8c419b03a2c3d573606 at 4.
    let one = at(&build, 1);
    assert!(has_class_skill(&one, "athletics"));
    assert!(has_class_skill(&one, "knowledge-world"));
    // Its bonus feat list is the one the wild talent feat slot nests for earth (ticket 304).
    assert_eq!(
        DATA.selection_option(&SelectionOptionKey::new(
            "kineticist/wild-talent-feats/earth"
        ))
        .and_then(|option| option.nested_selection()),
        Some(&rules::FeatureSelectionKey::new(
            "kineticist/element/earth/wild-talent-feats"
        ))
    );
    assert!(one.option_count(&SelectionOptionKey::new("kineticist/element/earth")) > 0);
    from_level(&build, 2, "kineticist/element/earth/flesh-of-stone");
    // The Kinetic Knight's comes at 4.
    let knight = with_option(
        archetype_build("kineticist", "kineticist/kinetic-knight", 4),
        1,
        0,
        "kineticist/element/earth",
    );
    assert!(engine().is_possible(&knight));
    from_level(&knight, 4, "kineticist/element/earth/flesh-of-stone");
}

#[test]
fn expanded_element_takes_a_second_element_or_the_first_again_but_not_three_times() {
    let mut build = kineticist("fire", 15);
    // Game: SecondatyElementalFocusSelection 4204bc10b3d5db440b1f52f0c375848b offers four; SecondaryElementWater
    // faa5f1233600d864fa998bc0afe351ab gives the blast only; ThirdElementFire 56e2fc3abed8f2247a621ac37e75f303 has
    // PrerequisiteNoFeature SecondaryElementFire caa7edca64af1914d9e14785beb6a143.
    assert_eq!(offered_at(&build, &selection_slot(7)).len(), 4);
    build
        .picks
        .insert(selection_slot(7), pick("kineticist/element/fire"));
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/water"));
    assert!(engine().is_possible(&build));
    assert_eq!(
        at(&build, 7).option_count(&SelectionOptionKey::new("kineticist/element/fire")),
        2
    );
    // The second element gives its list, not its defensive talent.
    assert!(!holds(
        &at(&build, 15),
        "kineticist/element/water/shroud-of-water"
    ));
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/fire"));
    assert!(
        why_not(&build, &selection_slot(15)).is_some(),
        "a third Fire"
    );
}

#[test]
fn a_living_deity_takes_a_mystery_revelations_and_two_domains_at_20() {
    let deceiver = "arcanist/magic-deceiver";
    let build = archetype_build("arcanist", deceiver, 20);
    let build = with_option(
        build,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let build = with_pick(build, option_slot(1, 0), "oracle/mystery/battle");
    let build = with_pick(
        build,
        option_slot(1, 1),
        "oracle/revelations/battle/battlecry",
    );
    let build = with_pick(build, option_slot(1, 2), "oracle/oracles-curse/plagued");
    let build = with_pick(build, option_slot(20, 0), "cleric/domain/fire");
    let build = with_pick(build, option_slot(20, 1), "cleric/domain/luck");
    assert!(engine().is_possible(&build));
    // Game: MagicDeceiverWayOfLivingGodProgression 464ffaf88140474a949618bec7955e78: mystery, curse and revelation at
    // 1, FakeDivineSparkSelection e6d337e709b4478cb6e5389d3f85b601 twice at 20; FireDomainProgression
    // 881b2137a1779294c8956fe5b497cc35 lists the Magic Deceiver (Blueprints/Classes/BlueprintProgression.CalcLevel,
    // WrathDecompiled 6ce4a6b).
    from_level(&build, 20, "oracle/mystery/battle/battle-final-revelation");
    let twenty = at(&build, 20);
    assert!(holds(&twenty, "cleric/domain/fire/fire-resistance"));
    assert!(holds(&twenty, "cleric/domain/luck/divine-fortune"));
}

#[test]
fn domain_zealot_needs_a_domain_held_whoever_gave_it() {
    let domain_zealot = DATA
        .mythic_abilities
        .iter()
        .find(|a| a.key.as_str() == "domain-zealot")
        .unwrap()
        .prerequisite
        .clone()
        .unwrap();
    // Game: DomainMastery 2de64f6a1f2baee4f9b7e52e3f046ec5 needs (Any) DomainsSelection
    // 48525e5da45c9c243a343fc6545dbdb9, DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b,
    // HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894, the Trickster's Religion tier 2 or the
    // Living Deity's FakeDivineSparkFeature: a cleric's domain counts, a Faith Hunter has none.
    assert!(
        at(&build("cleric", 1), 1).check(&domain_zealot).is_err(),
        "no domain picked yet"
    );
    assert_eq!(
        at(&cleric(["fire", "earth"], 1), 1).check(&domain_zealot),
        Ok(())
    );
    let faith_hunter = archetype_build("inquisitor", "inquisitor/faith-hunter", 1);
    assert!(at(&faith_hunter, 1).check(&domain_zealot).is_err());
}

/// A druid's domain doesn't open Domain Zealot: a Winter Child druid can't take it.
#[test]
#[ignore = "waits on ticket 307: the game gives Domain Zealot no druid's domain"]
fn domain_zealot_isnt_open_to_a_druids_domain() {
    let domain_zealot = DATA
        .mythic_abilities
        .iter()
        .find(|a| a.key.as_str() == "domain-zealot")
        .unwrap()
        .prerequisite
        .clone()
        .unwrap();
    // Game: DomainMastery 2de64f6a1f2baee4f9b7e52e3f046ec5's Any prerequisites don't name WinterChildDomainSelection
    // 79922e274bf4444b93ea77ae0f876bd2 or DruidDomainSelection 5edfe84c93823d04f8c40ca2b4e0f039.
    assert!(
        at(&archetype_build("druid", "druid/winter-child", 1), 1)
            .check(&domain_zealot)
            .is_err()
    );
}

#[test]
fn a_hellknight_picks_an_order_unless_held_and_pentamic_faith_gives_a_domain() {
    let hellknight = class("hellknight");
    let order = DATA
        .selection(&rules::FeatureSelectionKey::new("hellknight/order"))
        .expect("the Hellknight's order");
    // Game: HellKnightOrderSelection e31d849e4eb2578458419c9df622270f, seven orders, with PrerequisiteNoFeature of
    // itself; HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894 offers eleven domains, levelled by
    // HellknightDisciplinePentamicFaithFeature 3758edaa5cc54cb88b0d87a9f4a34df1's ranks, one a Hellknight level
    // (Blueprints/Classes/BlueprintProgression.CalcLevel, WrathDecompiled 6ce4a6b; equivalent model: levelled by
    // Hellknight levels).
    assert!(order.only_without_option);
    assert_eq!(order.offered().len(), 7);
    assert!(hellknight.levels[0].selections.contains(&order.key));
    let disciplines = DATA
        .selection(&rules::FeatureSelectionKey::new("hellknight/disciplines"))
        .unwrap();
    let faith = DATA
        .options_of(disciplines)
        .find(|o| o.key.as_str() == "hellknight/disciplines/pentamic-faith")
        .unwrap();
    assert_eq!(
        faith.levels[0].selections,
        [rules::FeatureSelectionKey::new("hellknight/pentamic-faith")]
    );
    let domains = DATA
        .selection(&rules::FeatureSelectionKey::new(
            "hellknight/pentamic-faith",
        ))
        .unwrap();
    assert_eq!(domains.offered().len(), 11);
    assert_eq!(
        domains.option_levels,
        BTreeSet::from([ClassKey::new("hellknight")])
    );
}

// ---- Impossible Domain and the druid (ticket 36) ---------------------------------------------------------------

/// `levels` of one class (and an archetype), then fighter 3: Mythic rank 1 at character level 6.
fn then_fighter_3(mut build: Build) -> Build {
    build
        .class_choices
        .insert(ClassKey::new("fighter"), ClassChoice::default());
    for _ in 0..3 {
        build.levels.push(LevelUp::new(ClassKey::new("fighter")));
    }
    build
}

/// Impossible Domain's status in the Mythic rank 1 slot.
fn impossible_domain_at_mr_1(build: &Build) -> Option<Result<(), Issue>> {
    engine()
        .options(build, &SlotId::Mythic { rank: 1 })
        .unwrap()
        .into_iter()
        .find(|option| option.pick.key() == "impossible-domain")
        .map(|option| option.status)
}

#[test]
fn no_druid_can_take_impossible_domain_whatever_its_domain() {
    // "If you choose a domain, you can't take the Impossible Domain mythic ability." (the druid's page)
    let nature_bond = with_pick(
        with_option(druid(None, 3), 1, 0, "druid/nature-bond/domain"),
        option_slot(1, 0),
        "cleric/domain/weather",
    );
    let blight = with_option(
        druid(Some("druid/blight-druid"), 3),
        1,
        0,
        "cleric/domain/death",
    );
    let winter = druid(Some("druid/winter-child"), 3);
    let companion = with_option(druid(None, 3), 1, 0, "druid/nature-bond/animal-companion");
    // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf needs DomainsSelection 48525e5da45c9c243a343fc6545dbdb9,
    // DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b or FakeDivineSparkSelection
    // e6d337e709b4478cb6e5389d3f85b601; a druid's domain is none of them (equivalent model).
    for (name, build, domain) in [
        ("Nature Bond", nature_bond, Some("weather")),
        ("Blight Druid", blight, Some("death")),
        ("Winter Child", winter, Some("ice")),
        ("animal companion", companion, None),
    ] {
        let build = then_fighter_3(build);
        assert!(possible(&build), "{name}");
        // Each holds its druid's domain at druid level 3, so only being a druid's keeps it out.
        if let Some(domain) = domain {
            assert_eq!(
                at(&build, 6)
                    .option_level(&SelectionOptionKey::new(format!("cleric/domain/{domain}"))),
                3,
                "{name}"
            );
        }
        let status = impossible_domain_at_mr_1(&build);
        assert!(matches!(status, Some(Err(_))), "{name}: {status:?}");
    }
    // A cleric 3 / fighter 3 can.
    assert_eq!(
        impossible_domain_at_mr_1(&then_fighter_3(cleric(["air", "water"], 3))),
        Some(Ok(()))
    );
}

// ---- the Dual-Cursed Oracle's second curse (ticket 67) -----------------------------------------------------------

const DUAL_CURSED: &str = "oracle/dual-cursed-oracle";
const SECOND_CURSES: &str = "oracle/second-curse";

/// The Dual-Cursed Oracle's level-1 slots in key order: its own revelation, the mystery, and the two curses.
const FIRST_CURSE: SlotId = SlotId::Selection { level: 1, index: 2 };
const SECOND_CURSE: SlotId = SlotId::Selection { level: 1, index: 3 };

/// "The Oracle must choose a second curse" (oracle#second-curse): its table's "Second Curse" at 1 is a slot of its
/// own, which offers the oracle's nine curses; the plain oracle and the other archetypes have none.
/// Game: DualCursedOracleArchetype 4a6eb63858244f7ea4ed950a7e798a70 adds SecondCurseSelection
/// cc6fda79e8c340b88c84689414a9abbe at 1, whose options are the nine `<Curse>CurseNoProgression`s
/// (LameCurseNoProgression 857c97115ec745d58cc8fa7bdedc794f).
#[test]
fn the_dual_cursed_oracles_second_curse_is_a_slot_of_its_own() {
    let oracle = class("oracle");
    let count = |key: &str, archetype: Option<&rules::Archetype>, level| {
        let key = rules::FeatureSelectionKey::new(key);
        rules::effective_row(oracle, archetype, level).map_or(0, |row| {
            row.selections.iter().filter(|s| **s == key).count()
        })
    };
    let dual = DATA
        .archetypes_of(&oracle.key)
        .find(|a| a.key.as_str() == DUAL_CURSED)
        .expect("the Dual-Cursed Oracle");
    assert_eq!(count(CURSES, Some(dual), 1), 1);
    assert_eq!(count(SECOND_CURSES, Some(dual), 1), 1);
    assert!((2..=20).all(|level| count(CURSES, Some(dual), level)
        + count(SECOND_CURSES, Some(dual), level)
        == 0));
    assert_eq!(count(SECOND_CURSES, None, 1), 0);
    for archetype in DATA
        .archetypes_of(&oracle.key)
        .filter(|a| a.key.as_str() != DUAL_CURSED)
    {
        assert_eq!(
            count(SECOND_CURSES, Some(archetype), 1),
            0,
            "{}",
            archetype.key
        );
    }
}

#[test]
fn a_dual_cursed_oracle_1_picks_two_curses_the_second_not_offering_the_first() {
    let engine = engine();
    let build = archetype_build("oracle", DUAL_CURSED, 1);
    let sheet = engine.evaluate(&build);
    for (id, key, label) in [
        (&FIRST_CURSE, CURSES, "Oracle's Curse"),
        (&SECOND_CURSE, SECOND_CURSES, "Second Curse"),
    ] {
        let slot = sheet.slot(id).expect("a curse slot");
        assert_eq!(slot.kind, selection_kind(key), "{id:?}");
        assert_eq!(engine.slot_label(slot), label);
    }
    let curse_slots = |build: &Build| {
        engine.evaluate(build).levels[0]
            .slots
            .iter()
            .filter(|slot| {
                [selection_kind(CURSES), selection_kind(SECOND_CURSES)].contains(&slot.kind)
            })
            .count()
    };
    assert_eq!(curse_slots(&build), 2);
    assert_eq!(curse_slots(&self::build("oracle", 1)), 1);
    assert!(!holds(
        &at(&build, 1),
        "oracle/dual-cursed-oracle/second-curse"
    ));

    // Either slot offers the nine curses, the one taken in the other locked: "Already taken" in the second slot, and
    // in the first, breaking the second's pick (as a cleric's two domain slots answer).
    // Game: each NoProgression curse (LameCurseNoProgression 857c97115ec745d58cc8fa7bdedc794f) has
    // PrerequisiteNoFeature of its regular progression (LameCurseProgression 3d319da9614c76a41a095a4467c3233f).
    let first = "oracle/oracles-curse/plagued";
    let second = "oracle/oracles-curse/lame";
    for (taken_in, other, issue) in [
        (&FIRST_CURSE, &SECOND_CURSE, Issue::AlreadyHeld),
        (&SECOND_CURSE, &FIRST_CURSE, Issue::BreaksLater),
    ] {
        let one = with_pick(build.clone(), taken_in.clone(), first);
        let curses = offered_in(&one, other);
        assert_eq!(curses.len(), 9);
        for (curse, status) in &curses {
            if curse == first {
                assert_eq!(*status, Err(issue.clone()), "{other:?}");
            } else {
                assert_eq!(*status, Ok(()), "{curse}");
            }
        }
    }
    let both = with_pick(
        with_pick(build.clone(), FIRST_CURSE, first),
        SECOND_CURSE,
        second,
    );
    assert!(possible(&both));
    let state = at(&both, 1);
    for curse in [first, second] {
        assert_eq!(state.option_count(&SelectionOptionKey::new(curse)), 1);
    }
    let twice = with_pick(with_pick(build, FIRST_CURSE, first), SECOND_CURSE, first);
    assert_eq!(why_not(&twice, &SECOND_CURSE), Some(Issue::AlreadyHeld));
}

#[test]
fn beneficial_curse_offers_a_dual_cursed_oracle_neither_of_her_curses() {
    let oracle = with_selections_filled(archetype_build("oracle", DUAL_CURSED, 6));
    let held: BTreeSet<String> = [&FIRST_CURSE, &SECOND_CURSE]
        .into_iter()
        .map(|slot| oracle.picks[slot].key().to_owned())
        .collect();
    assert_eq!(held.len(), 2);
    let oracle = with_mr_1_ability(oracle, BENEFICIAL_CURSE);
    assert!(possible(&oracle));
    // Game: each NoPenalty curse (DemonicCurseNoPenaltyProgression b1de8f8f1250b0c4aa342e1895348cac) has
    // PrerequisiteNoFeature of both the regular and the NoProgression progression.
    let curses = offered_in(&oracle, &mr_1_selection());
    assert_eq!(curses.len(), 9);
    for (curse, status) in &curses {
        if held.contains(curse) {
            assert_eq!(*status, Err(Issue::AlreadyHeld), "{curse}");
        } else {
            assert_eq!(*status, Ok(()), "{curse}");
        }
    }
    for curse in &held {
        let again = with_pick(oracle.clone(), mr_1_selection(), curse);
        assert_eq!(why_not(&again, &mr_1_selection()), Some(Issue::AlreadyHeld));
    }
    let third = curses.keys().find(|curse| !held.contains(*curse)).unwrap();
    assert!(possible(&with_pick(oracle, mr_1_selection(), third)));
}
