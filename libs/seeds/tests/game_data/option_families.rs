//! Domains, mysteries, blessings and elements (ticket 31), Impossible Domain and the druid (ticket 36), and the
//! Dual-Cursed Oracle's second curse (ticket 67).

use std::collections::BTreeSet;

use rules::{
    Build, ClassChoice, ClassKey, FeatKey, FeatSlot, FeatureKey, Issue, LevelUp, Pick,
    SelectionOptionKey, SlotId, SlotKind,
};

use crate::support::*;

// ---- domains, mysteries, blessings and elements (ticket 31) -------------------------------------------------

#[test]
fn a_cleric_8_holds_both_domains_second_powers() {
    // Fire and Glory: Sarenrae's (ticket 86: the deity offers both).
    let build = cleric(["fire", "glory"], 8);
    assert!(engine().is_possible(&build));
    for power in [
        "cleric/domain/fire/fire-bolt",
        "cleric/domain/glory/touch-of-glory",
    ] {
        assert!(holds(&at(&build, 1), power), "{power}");
    }
    from_level(&build, 6, "cleric/domain/fire/fire-resistance");
    from_level(&build, 8, "cleric/domain/glory/aura-of-heroism");
    // Two picks of one domain aren't two domains.
    let twice = cleric(["fire", "fire"], 1);
    assert!(why_not(&twice, &SlotId::Selection { level: 1, index: 1 }).is_some());
}

#[test]
fn a_cleric_archetype_that_may_only_select_a_single_domain_picks_one() {
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
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.slot(&slot).expect("the War domain's slot").kind,
        SlotKind::Feat(FeatSlot::CombatFeat)
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
            .features
            .keys()
            .all(|f| !f.as_str().starts_with("cleric/domain/"))
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
    assert!(!holds(
        &at(&blight, 4),
        "cleric/domain/animal/animal-companion"
    ));
    // The domain's Lore (Nature) sits on that skipped row too; a druid has it anyway.
    assert!(has_class_skill(&at(&blight, 4), "lore-nature"));

    let winter = druid(Some("druid/winter-child"), 8);
    assert!(engine().is_possible(&winter));
    assert!(holds(&at(&winter, 1), "cleric/domain/ice/icicle"));
    from_level(&winter, 8, "cleric/domain/ice/body-of-ice");
}

#[test]
fn an_inquisitors_domain_runs_on_inquisitor_levels() {
    let build = with_option(build("inquisitor", 6), 1, 0, "cleric/domain/luck");
    assert!(engine().is_possible(&build));
    from_level(&build, 6, "cleric/domain/luck/divine-fortune");
    let huntsmaster = archetype_build("inquisitor", "inquisitor/sacred-huntsmaster", 1);
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
    let battle = offered_at(&oracle("battle", 1), &revelation_slot(1));
    assert!(battle.contains("oracle/revelations/battle/weapon-mastery"));
    assert!(
        battle
            .iter()
            .all(|key| key.starts_with("oracle/revelations/battle/")),
        "{battle:?}"
    );
    let flame = offered_at(&oracle("flame", 1), &revelation_slot(1));
    assert!(
        flame
            .iter()
            .all(|key| key.starts_with("oracle/revelations/flame/")),
        "{flame:?}"
    );
    // "Prerequisites: Level 7"
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
            weapon("longsword"),
        ),
    );
    assert!(engine().is_possible(&build));
    assert!(at(&build, 1).has_feat_for(&FeatKey::new("weapon-focus"), &weapon("longsword")));
    let critical = FeatKey::new("improved-critical");
    assert!(!at(&build, 7).has_feat_for(&critical, &weapon("longsword")));
    assert!(at(&build, 8).has_feat_for(&critical, &weapon("longsword")));
    assert!(!at(&build, 8).has_feat_for(&critical, &weapon("dagger")));
    assert!(
        at(&build, 12).has_feat_for(&FeatKey::new("greater-weapon-focus"), &weapon("longsword"))
    );
}

#[test]
fn an_oracle_holds_the_mysterys_class_skill_and_final_revelation() {
    let build = oracle("battle", 20);
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
    let one = at(&build, 1);
    assert!(has_class_skill(&one, "athletics"));
    assert!(has_class_skill(&one, "knowledge-world"));
    assert_eq!(
        one.option_feat_lists
            .get(&SelectionOptionKey::new("kineticist/element/earth")),
        Some(&rules::FeatListKey::new(
            "kineticist/element/earth/wild-talent-feats"
        ))
    );
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
    assert_eq!(
        at(&archetype_build("druid", "druid/winter-child", 1), 1).check(&domain_zealot),
        Ok(())
    );
}

#[test]
fn a_hellknight_picks_an_order_unless_held_and_pentamic_faith_gives_a_domain() {
    let hellknight = class("hellknight");
    let order = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/order")
        .expect("the Hellknight's order");
    assert!(order.only_without_option);
    assert_eq!(order.offered().len(), 7);
    assert!(hellknight.levels[0].selections.contains(&order.key));
    let faith = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/disciplines")
        .unwrap()
        .options
        .iter()
        .find(|o| o.key.as_str() == "hellknight/disciplines/pentamic-faith")
        .unwrap();
    assert_eq!(
        faith.levels[0].selections,
        [rules::FeatureSelectionKey::new("hellknight/pentamic-faith")]
    );
    let domains = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/pentamic-faith")
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

/// The Dual-Cursed Oracle's level-1 slots in key order: its own revelation, the mystery, and the two curses.
const FIRST_CURSE: SlotId = SlotId::Selection { level: 1, index: 2 };
const SECOND_CURSE: SlotId = SlotId::Selection { level: 1, index: 3 };

/// "The Oracle must choose a second curse" (oracle#second-curse): its table's "Second Curse" at 1 is a second slot of
/// the oracle's curses in the rows in force, not a feature; the plain oracle and the other archetypes keep one.
#[test]
fn the_dual_cursed_oracles_second_curse_is_a_slot_of_the_oracles_curses() {
    let oracle = class("oracle");
    let curses = rules::FeatureSelectionKey::new(CURSES);
    let curse_slots = |archetype: Option<&rules::Archetype>, level| {
        rules::effective_row(oracle, archetype, level).map_or(0, |row| {
            row.selections.iter().filter(|s| **s == curses).count()
        })
    };
    let dual = oracle
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == DUAL_CURSED)
        .expect("the Dual-Cursed Oracle");
    assert_eq!(curse_slots(Some(dual), 1), 2);
    assert!((2..=20).all(|level| curse_slots(Some(dual), level) == 0));
    assert_eq!(curse_slots(None, 1), 1);
    for archetype in oracle
        .archetypes
        .iter()
        .filter(|a| a.key.as_str() != DUAL_CURSED)
    {
        assert!(curse_slots(Some(archetype), 1) <= 1, "{}", archetype.key);
    }
    let second = FeatureKey::new("oracle/dual-cursed-oracle/second-curse");
    assert!(dual.features.iter().all(|feature| feature.key != second));
    assert!(
        dual.levels
            .iter()
            .all(|row| !row.features.contains(&second))
    );
}

#[test]
fn a_dual_cursed_oracle_1_picks_two_curses_the_second_not_offering_the_first() {
    let engine = engine();
    let build = archetype_build("oracle", DUAL_CURSED, 1);
    let sheet = engine.evaluate(&build);
    for id in [&FIRST_CURSE, &SECOND_CURSE] {
        let slot = sheet.slot(id).expect("a curse slot");
        assert_eq!(slot.kind, selection_kind(CURSES), "{id:?}");
        assert_eq!(engine.slot_label(slot), "Oracle's Curse");
    }
    let curse_slots = |build: &Build| {
        engine.evaluate(build).levels[0]
            .slots
            .iter()
            .filter(|slot| slot.kind == selection_kind(CURSES))
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
