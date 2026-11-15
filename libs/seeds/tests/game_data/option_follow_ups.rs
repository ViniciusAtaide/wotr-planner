//! Option follow-ups (ticket 33) and the second follow-ups (ticket 35).

use std::collections::HashMap;

use rules::{
    Build, ClassKey, FeatKey, FeatSource, FeatureKey, Issue, LevelUp, Pick, Prerequisite,
    SelectionOptionKey, SkillKey, SlotId, SlotKind, SlotOption, Unmet,
};

use crate::support::*;

// ---- option follow-ups (ticket 33) ----------------------------------------------------------------------------

fn one_of_group(group: rules::OptionGroup) -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::NoOtherOptionOfGroup(group),
        has: None,
    })
}

#[test]
fn a_crossblooded_sorcerer_cant_take_red_after_blue() {
    // Game: the PrerequisiteCondition on each Crossblooded secondary
    // (CrossbloodedSecondaryBloodlineDraconicBlackProgression 601387ac1873bba448bddcfbf00af8d7) and on
    // BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8 bars a second dragon or element (equivalent
    // model: one option group each).
    let blue_red = crossblooded("draconic-blue", "draconic-red");
    let second = SlotId::Selection { level: 1, index: 1 };
    assert_eq!(
        why_not(&blue_red, &second),
        Some(one_of_group(rules::OptionGroup::Draconic))
    );
    // Two Elemental bloodlines neither; a dragon and an element are fine ("Elemental and Draconic bloodlines").
    assert_eq!(
        why_not(&crossblooded("elemental-air", "elemental-fire"), &second),
        Some(one_of_group(rules::OptionGroup::Elemental))
    );
    assert!(possible(&crossblooded("draconic-blue", "elemental-fire")));
    // Nor through Second Bloodline.
    let mut second = with_option(
        build("sorcerer", 6),
        1,
        0,
        "sorcerer/bloodline/draconic-blue",
    );
    second.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodline")),
    );
    let slot = SlotId::MythicSelection { rank: 1, index: 0 };
    let options: HashMap<String, Result<(), Issue>> = engine()
        .options(&second, &slot)
        .unwrap()
        .into_iter()
        .map(|SlotOption { pick, status, .. }| (pick.key().to_owned(), status))
        .collect();
    assert_eq!(
        options["sorcerer/bloodline/draconic-red"],
        Err(one_of_group(rules::OptionGroup::Draconic))
    );
    assert_eq!(options[ABYSSAL], Ok(()));
}

#[test]
fn a_crossblooded_sorcerers_second_bloodline_gives_its_arcana() {
    let build = crossblooded("arcane", "abyssal");
    assert!(possible(&build));
    // Game: CrossbloodedSecondaryBloodlineAbyssalProgression 7e9c25e11e2333b4ea16b39964c2f8f1: the arcana and the class
    // skill at 1, then spells only; SorcererBloodlineSelection 24bef8d1bee12274686f6da6ccbc8914 offers 21.
    let one = at(&build, 1);
    assert!(holds(&one, "sorcerer/bloodline/arcane/arcane-arcana"));
    assert!(holds(&one, "sorcerer/bloodline/abyssal/demonic-arcana"));
    assert!(!holds(&one, "sorcerer/bloodline/abyssal/grow-claws"));
    assert!(!holds(&one, "sorcerer/crossblooded/bloodline-2-arcana"));
    // Every sorcerer bloodline names its arcana.
    let bloodlines = &class("sorcerer")
        .selections
        .iter()
        .find(|s| s.key.as_str() == "sorcerer/bloodline")
        .unwrap()
        .options;
    assert_eq!(bloodlines.len(), 21);
    assert!(bloodlines.iter().all(|o| o.arcana.is_some()));
}

#[test]
fn the_unsworn_shamans_spirits_and_the_hunters_animal_focus_are_daily_choices() {
    let unsworn = at(&archetype_build("shaman", "shaman/unsworn-shaman", 6), 6);
    let daily: Vec<String> = engine()
        .daily_choices(&unsworn)
        .iter()
        .map(|key| key.to_string())
        .collect();
    assert!(daily.contains(&"shaman/unsworn-shaman/minor-spirit".to_owned()));
    assert!(daily.contains(&"shaman/unsworn-shaman/wandering-spirit-2".to_owned()));
    // The hunter's Animal Focus is a note, not a pick.
    let hunter = build("hunter", 8);
    let sheet = engine().evaluate(&hunter);
    assert!(
        sheet.levels[0]
            .slots
            .iter()
            .all(|slot| !matches!(slot.kind, SlotKind::Selection(_))),
        "{:?}",
        sheet.levels[0].slots
    );
    let daily = engine().daily_choices(sheet.at(8).unwrap());
    assert!(daily.contains(&FeatureKey::new("hunter/animal-focus")));
    assert!(daily.contains(&FeatureKey::new("hunter/second-animal-focus")));
}

#[test]
fn elemental_movement_follows_the_element_and_earth_has_none() {
    let element = |element: &str| {
        at(
            &with_option(
                build("sorcerer", 15),
                1,
                0,
                &format!("sorcerer/bloodline/elemental-{element}"),
            ),
            15,
        )
    };
    // Game: BloodlineElementalAirProgression cd788df497c6f10439c7025e87864ee4, BloodlineElementalFireProgression
    // 17cc794d47408bc4986c55265475c06f and BloodlineElementalWaterProgression 7c692e90592257a4e901d12ae6ec1e41: an
    // elemental movement feature at 15. Earth's: `the_earth_bloodline_has_its_own_elemental_movement_at_15`.
    for name in ["air", "fire", "water"] {
        assert!(
            holds(
                &element(name),
                &format!("sorcerer/bloodline/elemental/elemental-movement-{name}")
            ),
            "{name}"
        );
    }
}

/// The Earth bloodline has an elemental movement too, at 15 (a burrow ability).
#[test]
#[ignore = "waits on ticket 281: the game gives the Earth bloodline an elemental movement at 15"]
fn the_earth_bloodline_has_its_own_elemental_movement_at_15() {
    let earth = at(
        &with_option(
            build("sorcerer", 15),
            1,
            0,
            "sorcerer/bloodline/elemental-earth",
        ),
        15,
    );
    // Game: BloodlineElementalEarthProgression 32393034410fb2f4d9c8beaa5c8c8ab7 at 15:
    // BloodlineElementalEarthElementalMovementFeature 277d9c2a2392e8940a452888aa67f32e.
    assert!(
        earth
            .features
            .keys()
            .any(|key| key.as_str().contains("elemental-movement")),
        "{:?}",
        earth.features
    );
}

#[test]
fn a_separatists_second_domain_comes_two_levels_late() {
    let one = separatist(1);
    assert!(possible(&one));
    let at_1 = at(&one, 1);
    // Game: FireDomainProgression 881b2137a1779294c8956fe5b497cc35: its power at 1, its resistance at 6.
    assert!(holds(&at_1, "cleric/domain/fire/fire-bolt"));
    // Luck's power at cleric 1: `a_separatists_second_domain_gives_its_1st_level_power_from_cleric_1`.
    // Game: LuckDomainProgressionSeparatist 1f26551e7e61436c8ee188064038a896: its greater power at 8.
    let three = separatist(3);
    let at_3 = at(&three, 3);
    assert!(holds(&at_3, "cleric/domain/fire/fire-bolt"));
    assert!(holds(&at_3, "cleric/domain/luck/bit-of-luck"));
    // Luck's second power at 6, two levels late: cleric 8.
    from_level(&separatist(8), 8, "cleric/domain/luck/divine-fortune");
    from_level(&separatist(8), 6, "cleric/domain/fire/fire-resistance");
}

/// The Separatist's second domain gives its 1st-level power from cleric 1; only its numbers count two levels lower.
#[test]
#[ignore = "waits on ticket 253: the game gives the Separatist's second domain power from cleric 1"]
fn a_separatists_second_domain_gives_its_1st_level_power_from_cleric_1() {
    // Game: LuckDomainProgressionSeparatist 1f26551e7e61436c8ee188064038a896 grants its base power at its level 1
    // (cleric 1); SeparatistCasterLevelPropery 93d211bd5f4946f3bbed1b651d107377 lowers its numbers by two.
    assert!(holds(
        &at(&separatist(1), 1),
        "cleric/domain/luck/bit-of-luck"
    ));
}

/// A Divine Hunter's domain counts the hunter's level less two: Nobility's 8th-level power at hunter 10.
#[test]
#[ignore = "waits on ticket 253: the game gives the Divine Hunter's domain hunter level - 2"]
fn a_divine_hunters_domain_runs_two_levels_behind_its_hunter_level() {
    let build = with_option(
        archetype_build("hunter", "hunter/divine-hunter", 10),
        1,
        0,
        "cleric/domain/nobility",
    );
    assert!(possible(&build));
    // Game: NobilityDomainProgression 8480f2d1ca764774895ee6fd610a568e lists DivineHunterArchetype
    // f996f0a18e5d945459e710ee3a6dd485 with AdditionalLevel -2 (Blueprints/Classes/BlueprintProgression.CalcLevel,
    // WrathDecompiled 6ce4a6b).
    from_level(&build, 10, "cleric/domain/nobility/inspiring-command");
}

/// A Living Deity with the Battle mystery and the Plagued curse, arcanist `levels`, its two domains at 20.
fn living_deity(levels: usize) -> Build {
    let build = archetype_build("arcanist", "arcanist/magic-deceiver", levels);
    let build = with_option(
        build,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let build = with_pick(build, option_slot(1, 0), "oracle/mystery/battle");
    let build = with_pick(build, option_slot(1, 2), "oracle/oracles-curse/plagued");
    let build = with_pick(build, option_slot(20, 0), "cleric/domain/fire");
    with_pick(build, option_slot(20, 1), "cleric/domain/luck")
}

/// `build` with a mythic ability at MR 1 (character level 6) and `option` in its selection slot.
fn with_mythic_selection(mut build: Build, ability: &str, option: &str) -> Build {
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(rules::MythicAbilityKey::new(ability)),
    );
    build
        .picks
        .insert(SlotId::MythicSelection { rank: 1, index: 0 }, pick(option));
    build
}

#[test]
fn a_living_deity_with_impossible_domain_holds_a_third_domain() {
    let build = with_pick(
        living_deity(20),
        option_slot(1, 1),
        "oracle/revelations/battle/battlecry",
    );
    let build = with_mythic_selection(build, "impossible-domain", "cleric/domain/nobility");
    assert!(possible(&build));
    // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf offers the domain progressions, which list the Magic Deceiver
    // and the cleric (NobilityDomainProgression 8480f2d1ca764774895ee6fd610a568e; Blueprints/Classes/
    // BlueprintProgression.CalcLevel, WrathDecompiled 6ce4a6b).
    let twenty = at(&build, 20);
    for domain in ["fire", "luck", "nobility"] {
        assert_eq!(
            twenty.option_count(&SelectionOptionKey::new(format!("cleric/domain/{domain}"))),
            1,
            "{domain}"
        );
    }
    // Levelled by arcanist levels (ticket 35): 20 here.
    assert!(holds(&twenty, "cleric/domain/nobility/inspiring-command"));
    // A cleric's third domain runs on cleric levels.
    let cleric = with_mythic_selection(
        cleric(["fire", "earth"], 8),
        "impossible-domain",
        "cleric/domain/nobility",
    );
    assert!(possible(&cleric));
    from_level(&cleric, 6, "cleric/domain/nobility/inspiring-word");
    from_level(&cleric, 8, "cleric/domain/nobility/inspiring-command");
}

#[test]
fn cinder_dance_needs_no_lame_curse() {
    let flame = |curse: &str| {
        let build = oracle("flame", 1);
        let build = with_option(build, 1, 1, &format!("oracle/oracles-curse/{curse}"));
        with_option(build, 1, 2, "oracle/revelations/flame/cinder-dance")
    };
    // Game: OracleRevelationCinderDance 6e67eae3081853544b191943f5ed4534's PrerequisiteNoFeature
    // LameCurseProgression 3d319da9614c76a41a095a4467c3233f.
    assert!(possible(&flame("plagued")));
    assert!(!possible(&flame("lame")));
}

#[test]
fn a_dual_cursed_oracle_is_offered_misfortune_and_fortune() {
    // Its level-1 slots in key order: its own revelation, the mystery, the curse.
    let build = with_option(
        archetype_build("oracle", "oracle/dual-cursed-oracle", 13),
        1,
        1,
        "oracle/mystery/battle",
    );
    // Fortune from oracle 5: `a_dual_cursed_oracle_takes_fortune_from_oracle_5_and_misfortune_from_1`.
    for level in [1, 3, 5, 7, 11, 13] {
        let offered = offered_in(&build, &selection_slot(level));
        for key in [
            "oracle/revelations/misfortune",
            "oracle/revelations/battle/weapon-mastery",
        ] {
            assert_eq!(offered.get(key), Some(&Ok(())), "{level}: {key}");
        }
    }
    // A plain oracle is offered them locked, as the game shows them (ticket 243): FortuneRevelationFeature
    // 16935c4660ee4c7da976b5f6cafb43dd and MisfortuneRevelationFeature 8121fe70a7974970a8a560bd2267cc24 are options of
    // OracleRevelationSelection 60008a10ad7ad6543b1f63016741a5d2 whose PrerequisiteArchetypeLevel names the
    // Dual-Cursed Oracle.
    assert!(matches!(
        offered_in(&oracle("battle", 1), &revelation_slot(1))["oracle/revelations/misfortune"],
        Err(Issue::Unmet(_))
    ));
}

#[test]
fn a_living_deitys_maneuver_mastery_and_second_mystery_count_arcanist_levels() {
    // Maneuver Mastery's Trip at the revelation's 7th level: arcanist 7.
    let build = with_pick(
        living_deity(7),
        option_slot(1, 1),
        "oracle/revelations/battle/maneuver-mastery",
    );
    let build = with_pick(build, option_slot(1, 3), "oracle/maneuver-mastery/trip");
    // Game: OracleRevelationManeuverMasteryTrip 86f1c6ea242b6c74e81e370e4ffad4d1's AddFeatureOnClassLevel 7
    // (ImprovedTrip 0f15c6f70d8fb2b49aa6cc24239cc5fa), counting the Magic Deceiver's arcanist levels.
    let trip = FeatKey::new("trip");
    assert_eq!(at(&build, 6).feat_count(&trip), 0);
    assert_eq!(at(&build, 7).feat_count(&trip), 1);
    // The second mystery's class skill from the pick (arcanist 6), its Final Revelation at 20.
    let build = with_pick(
        living_deity(20),
        option_slot(1, 1),
        "oracle/revelations/battle/battlecry",
    );
    let build = with_mythic_selection(build, "second-mystery", "oracle/mystery/flame");
    assert!(possible(&build));
    // Game: OracleFlameMysteryFeature 3b68909df737cd4458509d7f3a9c3706 adds Athletics as a class skill; its final
    // revelation at 20.
    class_skill_from_level(&build, 6, "athletics");
    from_level(&build, 20, "oracle/mystery/flame/flame-final-revelation");
}

#[test]
fn a_champion_of_the_faiths_alignment_blessing_is_his_own_alignment() {
    let champion = |alignment: rules::Alignment, blessing: &str| {
        let mut build = archetype_build("warpriest", "warpriest/champion-of-the-faith", 1);
        align(&mut build, alignment);
        let slot = engine().evaluate(&build).levels[0]
            .slots
            .iter()
            .find(|slot| {
                matches!(&slot.kind, SlotKind::Selection(s)
                    if s.as_str() == "warpriest/champion-of-the-faith/chosen-alignment")
            })
            .unwrap()
            .id
            .clone();
        build.picks.insert(
            slot,
            pick(&format!("warpriest/blessings/{blessing}-domain")),
        );
        // A deity offering that alignment's domain, if any (ticket 86, question 13).
        possible(&realigned(build, alignment))
    };
    // Game: ChampionOfTheFaithGoodFeature 92b34412524dda24fa544c54189792d7 (not evil),
    // ChampionOfTheFaithLawFeature 980004b88420d6248975b278a7efbdcd (not chaotic), ChampionOfTheFaithEvilFeature
    // cf6e5f8927653434eab6179cbc76eb04 (not good) and ChampionOfTheFaithChaosFeature 4ee098d102ccf574a99f1cf63ee01a05
    // (not lawful), by PrerequisiteAlignment (Blueprints/Classes/Prerequisites/PrerequisiteAlignment.CheckInternal,
    // WrathDecompiled 6ce4a6b). A true neutral's Chaos: `a_true_neutral_champion_of_the_faith_takes_chaos`.
    assert!(champion(rules::Alignment::LawfulGood, "good"));
    assert!(champion(rules::Alignment::LawfulGood, "law"));
    assert!(!champion(rules::Alignment::LawfulGood, "evil"));
    assert!(champion(rules::Alignment::ChaoticEvil, "chaos"));
    // A warpriest's own Evil blessing, as a neutral evil follower of a deity offering Evil.
    // Game: EvilBlessingFeature f38f3abf6ca3a07499a61f96e342bb16 needs the deity's Evil domain. Without that
    // alignment: `a_warpriests_evil_blessing_needs_the_deity_not_the_alignment`.
    let blessings = with_option(
        build("warpriest", 1),
        1,
        0,
        "warpriest/blessings/evil-domain",
    );
    assert!(possible(&realigned(
        blessings,
        rules::Alignment::NeutralEvil
    )));
}

/// A true neutral Champion of the Faith may take Chaos as his chosen alignment: it needs only an alignment that
/// isn't lawful.
#[test]
#[ignore = "waits on ticket 267: the game gives a true neutral Champion of the Faith Chaos"]
fn a_true_neutral_champion_of_the_faith_takes_chaos() {
    let mut build = archetype_build("warpriest", "warpriest/champion-of-the-faith", 1);
    align(&mut build, rules::Alignment::TrueNeutral);
    let slot = slot_of(
        &build,
        1,
        "warpriest/champion-of-the-faith/chosen-alignment",
    );
    build
        .picks
        .insert(slot, pick("warpriest/blessings/chaos-domain"));
    // Game: ChampionOfTheFaithChaosFeature 4ee098d102ccf574a99f1cf63ee01a05's PrerequisiteAlignment admits every
    // alignment that isn't lawful, and names no deity.
    assert!(possible(&realigned(build, rules::Alignment::TrueNeutral)));
}

/// A warpriest's Evil blessing needs a deity offering Evil, not an evil warpriest: a lawful neutral follower of
/// Asmodeus takes it.
#[test]
#[ignore = "waits on ticket 267: the game gives a warpriest's alignment blessing a deity condition only"]
fn a_warpriests_evil_blessing_needs_the_deity_not_the_alignment() {
    let mut build = with_option(
        build("warpriest", 1),
        1,
        0,
        "warpriest/blessings/evil-domain",
    );
    build.start.alignment = Some(rules::Alignment::LawfulNeutral);
    build.start.deity = Some(rules::Faith::Deity(rules::DeityKey::new("asmodeus")));
    // Game: EvilBlessingFeature f38f3abf6ca3a07499a61f96e342bb16 needs only EvilDomainAllowed, which AsmodeusFeature
    // a3a5ccc9c670e6f4ca4a686d23b89900 grants to its followers, lawful neutral among them.
    assert!(possible(&build));
}

#[test]
fn a_signifer_picks_one_of_the_seven_hellknight_orders() {
    // Game: HellknightSigniferProgression 9dbe8102673d3e94e98e25cb9260e1c5 at 1: HellKnightOrderSelection
    // e31d849e4eb2578458419c9df622270f, seven orders.
    let signifer = class("hellknight-signifer");
    let order = signifer
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight-signifer/order")
        .expect("the Signifer's order");
    assert_eq!(order.offered().len(), 7);
    assert!(signifer.levels[0].selections.contains(&order.key));
    assert!(
        signifer
            .features
            .iter()
            .all(|f| f.key.as_str() != "hellknight-signifer/order")
    );
}

#[test]
fn expanded_element_at_15_isnt_the_7th_level_element() {
    let mut build = kineticist("fire", 15);
    // Game: ThirdElementWater 86eff374d040404438ad97fedd7218bc's PrerequisiteNoFeature SecondaryElementWater
    // faa5f1233600d864fa998bc0afe351ab.
    build
        .picks
        .insert(selection_slot(7), pick("kineticist/element/water"));
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/water"));
    assert!(!possible(&build), "the 7th-level element again");
    build
        .picks
        .insert(selection_slot(15), pick("kineticist/element/fire"));
    assert!(possible(&build));
}

#[test]
fn a_slayer_who_picked_archery_cant_take_two_weapon_fighting_through_a_combat_style_talent() {
    let mut slayer = build("slayer", 6);
    slayer.picks.insert(
        selection_slot(2),
        pick("slayer/slayer-talents/combat-style-feat-2"),
    );
    let style = SlotId::OptionSelection { level: 2, index: 0 };
    // The five ranger styles.
    // Game: RangerStyleSelection2 c6d0da9124735a44f93ac31df803b9a9 offers five; RangerStyleArcherySelection2
    // 3cf94df3f1cf9a443a880bc1ae9be3c6 lists Rapid Shot, not Two-Weapon Fighting; RangerStyleArcherySelection6
    // 6c799d09d5b93f344b9ade0e0c765c2d (Manyshot) needs Archery's stage 2.
    assert_eq!(offered_at(&slayer, &style).len(), 5);
    slayer
        .picks
        .insert(style, pick("ranger/combat-style-feat/archery-feats"));
    let feat_slot = |level| SlotId::OptionFeat { level, index: 0 };
    let offered = feats_offered(&slayer, &feat_slot(2));
    assert!(offered.contains("rapid-shot") && !offered.contains("two-weapon-fighting"));
    slayer.picks.insert(
        feat_slot(2),
        Pick::feat(FeatKey::new("two-weapon-fighting")),
    );
    assert_eq!(why_not(&slayer, &feat_slot(2)), None);
    // The second talent picks no style; its feat comes from Archery's list of the second stage.
    slayer
        .picks
        .insert(feat_slot(2), Pick::feat(FeatKey::new("rapid-shot")));
    slayer.picks.insert(
        selection_slot(6),
        pick("slayer/slayer-talents/combat-style-feat-6"),
    );
    let sheet = engine().evaluate(&slayer);
    assert!(
        sheet
            .slot(&SlotId::OptionSelection { level: 6, index: 0 })
            .is_none()
    );
    assert!(feats_offered(&slayer, &feat_slot(6)).contains("manyshot"));
    slayer
        .picks
        .insert(feat_slot(6), Pick::feat(FeatKey::new("manyshot")));
    assert!(possible(&slayer));
}

// ---- second follow-ups (ticket 35) ----------------------------------------------------------------------------

#[test]
fn a_dual_heritage_or_kindred_raised_half_elf_has_no_adaptability_pick() {
    let half_elf = |subrace: Option<&str>| {
        let mut build = build("fighter", 1);
        build.start.race = Some(rules::RaceKey::new("half-elf"));
        build.start.subrace = subrace.map(rules::SubraceKey::new);
        build
    };
    let race_slots = |build: &Build| {
        engine().evaluate(build).levels[0]
            .slots
            .iter()
            .filter(|slot| slot.id == race_slot())
            .count()
    };
    // Game: HalfElfHeritageSelection 9df7b68d60544bcf8e5b56c0a4688e04: only the basic heritage gives
    // BaseRaseHalfElfSelection a9d54aab4e9243ca867341257aaa253e (Adaptability).
    assert_eq!(race_slots(&half_elf(None)), 1);
    for subrace in ["half-elf/dual-heritage", "half-elf/kindred-raised"] {
        assert_eq!(race_slots(&half_elf(Some(subrace))), 0, "{subrace}");
    }
    // No subrace in the data has slots of its own: the guide lists none.
    assert!(
        DATA.races
            .iter()
            .flat_map(|race| &race.subraces)
            .all(|subrace| subrace.feat_slots.is_empty())
    );
}

/// A Living Deity with the Battle mystery and the Plagued curse, arcanist `levels` (below 20: no domains of its
/// own yet), with Impossible Domain's Nobility domain at MR 1 (character level 6).
fn living_deity_with_impossible_domain(levels: usize) -> Build {
    let build = archetype_build("arcanist", "arcanist/magic-deceiver", levels);
    let build = with_option(
        build,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let build = with_pick(build, option_slot(1, 0), "oracle/mystery/battle");
    let build = with_pick(build, option_slot(1, 2), "oracle/oracles-curse/plagued");
    with_mythic_selection(build, "impossible-domain", "cleric/domain/nobility")
}

#[test]
fn a_living_deity_with_impossible_domain_holds_the_domains_level_1_power_before_arcanist_20() {
    let build = living_deity_with_impossible_domain(8);
    assert!(possible(&build));
    // Arcanist levels from the pick: its 1st-level power at 6, its 8th-level one at 8.
    // Game: NobilityDomainProgression 8480f2d1ca764774895ee6fd610a568e lists the Magic Deceiver
    // (Blueprints/Classes/BlueprintProgression.CalcLevel, WrathDecompiled 6ce4a6b).
    from_level(&build, 6, "cleric/domain/nobility/inspiring-word");
    from_level(&build, 8, "cleric/domain/nobility/inspiring-command");
    assert_eq!(
        at(&build, 8).option_level(&SelectionOptionKey::new("cleric/domain/nobility")),
        8
    );
}

/// Fighter 6, then Hellknight 3 (lawful neutral, with the entry's skill ranks): the Order of the Godclaw at
/// Hellknight 1, Pentamic Faith's Law domain at 3 (character level 9, Mythic rank 3).
fn pentamic_hellknight() -> Build {
    let mut build = classes(&[
        "fighter",
        "fighter",
        "fighter",
        "fighter",
        "fighter",
        "fighter",
        "hellknight",
        "hellknight",
        "hellknight",
    ]);
    align(&mut build, rules::Alignment::LawfulNeutral);
    for (level, ranks) in build.levels.iter_mut().zip([
        [("knowledge-arcana", 1), ("persuasion", 1)],
        [("knowledge-arcana", 1), ("persuasion", 1)],
        [("persuasion", 1), ("perception", 1)],
        [("persuasion", 1), ("perception", 1)],
        [("persuasion", 1), ("perception", 1)],
    ]) {
        level.skill_ranks = ranks
            .into_iter()
            .map(|(skill, ranks)| (SkillKey::new(skill), ranks))
            .collect();
    }
    let build = with_option(build, 7, 0, "fighter/armiger/order/order-of-the-godclaw");
    let build = with_option(build, 9, 0, "hellknight/disciplines/pentamic-faith");
    with_pick(build, option_slot(9, 0), "cleric/domain/law")
}

/// Pentamic Faith's domain doesn't open Impossible Domain: a Hellknight 3 holding one can't take it.
#[test]
#[ignore = "waits on ticket 265: the game gives Impossible Domain no Pentamic Faith"]
fn a_hellknight_3_with_pentamic_faiths_domain_cant_take_impossible_domain() {
    let build = pentamic_hellknight();
    assert!(possible(&build));
    let status = feat_status(&build, &SlotId::Mythic { rank: 3 }, "impossible-domain");
    // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf needs (Any) DomainsSelection 48525e5da45c9c243a343fc6545dbdb9,
    // DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b or FakeDivineSparkSelection
    // e6d337e709b4478cb6e5389d3f85b601, not HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894.
    assert!(status.is_err(), "{status:?}");
}

#[test]
fn a_hellknight_who_becomes_a_signifer_gets_no_second_order() {
    let order_slots = |build: &Build, level: u8| {
        engine().evaluate(build).levels[usize::from(level) - 1]
            .slots
            .iter()
            .filter(|slot| {
                matches!(&slot.kind, SlotKind::Selection(s) if s.as_str() == "hellknight-signifer/order")
            })
            .count()
    };
    let mut hellknight = with_option(
        classes(&[
            "fighter",
            "fighter",
            "fighter",
            "fighter",
            "fighter",
            "fighter",
            "hellknight",
            "hellknight-signifer",
        ]),
        7,
        0,
        "fighter/armiger/order/order-of-the-chain",
    );
    align(&mut hellknight, rules::Alignment::LawfulNeutral);
    // Game: the Hellknight and the Signifer share HellKnightOrderSelection e31d849e4eb2578458419c9df622270f, whose
    // PrerequisiteNoFeature of itself is checked in the progression (UnitLogic/Class/LevelUp/Actions/
    // LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b).
    assert_eq!(order_slots(&hellknight, 8), 0);
    // Anyone holding no order picks one.
    let mut wizard = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "hellknight-signifer",
    ]);
    align(&mut wizard, rules::Alignment::LawfulNeutral);
    assert_eq!(order_slots(&wizard, 6), 1);
}

/// Whether a Separatist of `alignment` may take `domain` as his second domain. The first domain one that a deity of
/// the alignment offers (ticket 86): Protection (Abadar, Shelyn, Nethys), Trickery for a chaotic evil (Calistria).
/// Then a deity that doesn't offer the second, if any.
fn separatist_takes(alignment: rules::Alignment, domain: &str) -> bool {
    let first = if alignment == rules::Alignment::ChaoticEvil {
        "trickery"
    } else {
        "protection"
    };
    let mut build = archetype_build("cleric", "cleric/separatist", 1);
    build
        .picks
        .insert(selection_slot(1), pick(&format!("cleric/domain/{first}")));
    build.picks.insert(
        SlotId::Selection { level: 1, index: 1 },
        pick(&format!("cleric/domain/{domain}")),
    );
    possible(&realigned(build, alignment))
}

#[test]
fn a_lawful_good_separatist_cant_take_chaos_or_evil_as_his_second_domain() {
    let lawful_good = rules::Alignment::LawfulGood;
    // Game: ChaosDomainProgressionSeparatist 714706ca6958440cbbb217c2d167e6ec, EvilDomainProgressionSeparatist
    // bd2a6afae68a445da4d17b32116798fc, GoodDomainProgressionSeparatist 9370c9720dd84695bac921b711b44c66 and
    // LawDomainProgressionSeparatist 94ea125ce3cd42a19bb804e267ab127c each need their alignment
    // (Blueprints/Classes/Prerequisites/PrerequisiteAlignment.CheckInternal, WrathDecompiled 6ce4a6b). A true
    // neutral: `a_true_neutral_separatist_takes_no_alignment_domain`.
    assert!(!separatist_takes(lawful_good, "chaos"));
    assert!(!separatist_takes(lawful_good, "evil"));
    assert!(separatist_takes(lawful_good, "good"));
    assert!(separatist_takes(lawful_good, "law"));
    // A chaotic evil can't take Law or Good.
    assert!(!separatist_takes(rules::Alignment::ChaoticEvil, "law"));
    assert!(!separatist_takes(rules::Alignment::ChaoticEvil, "good"));
    // His first domain, a cleric's, isn't checked against it here; since ticket 86 a cleric's alignment domain needs
    // its alignment (question 7), so a lawful good one can't take Chaos at all.
    // Game: ChaosDomainProgression 5a5d19c246961484a97e1e5dded98ab2's PrerequisiteAlignment: chaotic.
    let mut first = archetype_build("cleric", "cleric/separatist", 1);
    first
        .picks
        .insert(selection_slot(1), pick("cleric/domain/chaos"));
    assert!(!possible(&realigned(first.clone(), lawful_good)));
    assert!(possible(&realigned(
        first,
        rules::Alignment::ChaoticNeutral
    )));
}

/// A ranger 10 / slayer 2 with Archery: the ranger's slot at 10 offers the style's 10th-level list.
#[test]
fn a_ranger_10s_style_slot_offers_the_10th_level_list() {
    let mut keys = vec!["ranger"; 10];
    keys.extend(["slayer", "slayer"]);
    let build = with_option(
        classes(&keys),
        2,
        0,
        "ranger/combat-style-feat/archery-feats",
    );
    // The slayer's 2nd-level combat style talent there: `a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style`.
    // Game: RangerProgression 97261d609529d834eba4fd4da1bc44dc at 10: RangerStyleSelection10
    // 78177315fc63b474ea3cbb8df38fafcd, Archery's 10th-level list (RangerStyleArcherySelection10
    // 7ef950ca681955d47bc4efbe77073e2c).
    assert!(
        feats_offered(
            &build,
            &SlotId::Feat {
                level: 10,
                source: FeatSource::Class,
                index: 0,
            }
        )
        .contains("improved-critical")
    );
}

/// A ranger 10 / slayer 2 can't take the slayer's 2nd-level combat style talent: he holds the ranger's
/// RangerStyleSelection2 already.
#[test]
#[ignore = "waits on ticket 276: the game refuses the talent to a ranger who holds the stage-2 style"]
fn a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style() {
    let mut keys = vec!["ranger"; 10];
    keys.extend(["slayer", "slayer"]);
    let build = with_option(
        classes(&keys),
        2,
        0,
        "ranger/combat-style-feat/archery-feats",
    );
    let build = with_option(build, 12, 0, "slayer/slayer-talents/combat-style-feat-2");
    // Game: RangerProgression 97261d609529d834eba4fd4da1bc44dc gives RangerStyleSelection2
    // c6d0da9124735a44f93ac31df803b9a9 at ranger 2, and SlayerTalentSelection2 04430ad24988baa4daa0bcd4f1c7d118's
    // option needs it not held (Blueprints/Classes/BlueprintFeature.MeetsPrerequisites, WrathDecompiled 6ce4a6b).
    assert!(!possible(&build));
}

#[test]
fn a_bloodrager_with_a_dragon_bloodline_cant_take_a_second_dragon_through_second_bloodline() {
    let second = |first: &str| {
        let mut build = with_option(
            build("bloodrager", 6),
            1,
            0,
            &format!("bloodrager/bloodline/{first}"),
        );
        build.picks.insert(
            SlotId::Mythic { rank: 1 },
            Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodrager-bloodline")),
        );
        let options: HashMap<String, Result<(), Issue>> = engine()
            .options(&build, &SlotId::MythicSelection { rank: 1, index: 0 })
            .unwrap()
            .into_iter()
            .map(|SlotOption { pick, status, .. }| (pick.key().to_owned(), status))
            .collect();
        options
    };
    // Game: BloodragerDragonBlueSecondBloodline 4c495ac785134983b99f77eb559a81a0 and
    // BloodragerElementalFireSecondBloodline 5151e4d7512e41a2a35c1c2707281a64 test the bloodrager's own dragon or
    // element (equivalent model: one option group each).
    let red = second("draconic-red");
    assert_eq!(
        red["bloodrager/bloodline/draconic-blue"],
        Err(one_of_group(rules::OptionGroup::BloodragerDraconic))
    );
    assert_eq!(red["bloodrager/bloodline/abyssal"], Ok(()));
    assert_eq!(red["bloodrager/bloodline/elemental-fire"], Ok(()));
    let air = second("elemental-air");
    assert_eq!(
        air["bloodrager/bloodline/elemental-fire"],
        Err(one_of_group(rules::OptionGroup::BloodragerElemental))
    );
    assert_eq!(air["bloodrager/bloodline/draconic-red"], Ok(()));
    // Apart from the sorcerer's: a sorcerer's dragon doesn't stop a bloodrager's.
    let mut both = with_option(
        build("sorcerer", 1),
        1,
        0,
        "sorcerer/bloodline/draconic-red",
    );
    both.levels.push(LevelUp::new(ClassKey::new("bloodrager")));
    let both = with_option(both, 2, 0, "bloodrager/bloodline/draconic-blue");
    assert!(possible(&both));
}

/// A true neutral Separatist can take none of the alignment domains: each needs its alignment.
#[test]
#[ignore = "waits on ticket 267: the game gives a Separatist's alignment domain only to that alignment"]
fn a_true_neutral_separatist_takes_no_alignment_domain() {
    // Game: ChaosDomainProgressionSeparatist 714706ca6958440cbbb217c2d167e6ec, EvilDomainProgressionSeparatist
    // bd2a6afae68a445da4d17b32116798fc, GoodDomainProgressionSeparatist 9370c9720dd84695bac921b711b44c66 and
    // LawDomainProgressionSeparatist 94ea125ce3cd42a19bb804e267ab127c each need their alignment.
    for domain in ["chaos", "evil", "good", "law"] {
        assert!(
            !separatist_takes(rules::Alignment::TrueNeutral, domain),
            "{domain}"
        );
    }
}
