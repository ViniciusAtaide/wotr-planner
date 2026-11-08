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
        Pick::MythicAbility(rules::MythicAbilityKey::new("second-bloodline")),
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
    for name in ["air", "fire", "water"] {
        assert!(
            holds(
                &element(name),
                &format!("sorcerer/bloodline/elemental/elemental-movement-{name}")
            ),
            "{name}"
        );
    }
    let earth = element("earth");
    assert!(
        earth
            .features
            .keys()
            .all(|key| !key.as_str().contains("elemental-movement")),
        "{:?}",
        earth.features
    );
}

#[test]
fn a_separatists_second_domain_comes_two_levels_late() {
    let one = separatist(1);
    assert!(possible(&one));
    let at_1 = at(&one, 1);
    assert!(holds(&at_1, "cleric/domain/fire/fire-bolt"));
    assert!(!holds(&at_1, "cleric/domain/luck/bit-of-luck"));
    let three = separatist(3);
    let at_3 = at(&three, 3);
    assert!(holds(&at_3, "cleric/domain/fire/fire-bolt"));
    assert!(holds(&at_3, "cleric/domain/luck/bit-of-luck"));
    // Luck's second power at 6, two levels late: cleric 8.
    from_level(&separatist(8), 8, "cleric/domain/luck/divine-fortune");
    from_level(&separatist(8), 6, "cleric/domain/fire/fire-resistance");
}

#[test]
fn a_divine_hunter_8_holds_its_domains_level_8_power() {
    let build = with_option(
        archetype_build("hunter", "hunter/divine-hunter", 8),
        1,
        0,
        "cleric/domain/nobility",
    );
    assert!(possible(&build));
    from_level(&build, 8, "cleric/domain/nobility/inspiring-command");
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
        Pick::MythicAbility(rules::MythicAbilityKey::new(ability)),
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
    for level in [1, 3, 5, 7, 11, 13] {
        let offered = offered_at(&build, &selection_slot(level));
        assert!(
            offered.contains("oracle/dual-cursed-oracle/revelation/misfortune")
                && offered.contains("oracle/dual-cursed-oracle/revelation/fortune")
                && offered.contains("oracle/revelations/battle/weapon-mastery"),
            "{level}: {offered:?}"
        );
    }
    // A plain oracle isn't.
    assert!(
        !offered_at(&oracle("battle", 1), &revelation_slot(1))
            .contains("oracle/dual-cursed-oracle/revelation/misfortune")
    );
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
    assert!(champion(rules::Alignment::LawfulGood, "good"));
    assert!(champion(rules::Alignment::LawfulGood, "law"));
    assert!(!champion(rules::Alignment::LawfulGood, "evil"));
    assert!(!champion(rules::Alignment::TrueNeutral, "chaos"));
    assert!(champion(rules::Alignment::ChaoticEvil, "chaos"));
    // A warpriest's own blessings checked it too since ticket 86 (question 7): an evil one needs an evil warpriest.
    let blessings = with_option(
        build("warpriest", 1),
        1,
        0,
        "warpriest/blessings/evil-domain",
    );
    assert_eq!(blessings.start.alignment, None);
    assert!(!possible(&blessings));
    assert!(possible(&realigned(
        blessings,
        rules::Alignment::NeutralEvil
    )));
}

#[test]
fn a_signifer_picks_one_of_the_seven_hellknight_orders() {
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

#[test]
fn a_hellknight_3_with_pentamic_faiths_domain_can_take_impossible_domain() {
    let build = pentamic_hellknight();
    assert!(possible(&build));
    let mythic = SlotId::Mythic { rank: 3 };
    let status = engine()
        .options(&build, &mythic)
        .unwrap()
        .into_iter()
        .find(|option| option.pick.key() == "impossible-domain")
        .map(|option| option.status);
    assert_eq!(status, Some(Ok(())));
    // Its domain runs on Hellknight levels, like the Pentamic Faith domain.
    let mut build = build;
    build.picks.insert(
        mythic,
        Pick::MythicAbility(rules::MythicAbilityKey::new("impossible-domain")),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 3, index: 0 },
        pick("cleric/domain/war"),
    );
    assert!(possible(&build));
    let nine = at(&build, 9);
    assert_eq!(
        nine.option_level(&SelectionOptionKey::new("cleric/domain/war")),
        3
    );
    assert!(holds(&nine, "cleric/domain/war/battle-rage"));
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

#[test]
fn a_lawful_good_separatist_cant_take_chaos_or_evil_as_his_second_domain() {
    // The first domain one that a deity of the alignment offers (ticket 86): Protection (Abadar, Shelyn, Nethys),
    // Trickery for a chaotic evil (Calistria). Then a deity that doesn't offer the second, if any.
    let separatist = |alignment: rules::Alignment, domain: &str| {
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
    };
    let lawful_good = rules::Alignment::LawfulGood;
    assert!(!separatist(lawful_good, "chaos"));
    assert!(!separatist(lawful_good, "evil"));
    assert!(separatist(lawful_good, "good"));
    assert!(separatist(lawful_good, "law"));
    // A true neutral has none that conflicts; a chaotic evil can't take Law or Good.
    for domain in ["chaos", "evil", "good", "law"] {
        assert!(
            separatist(rules::Alignment::TrueNeutral, domain),
            "{domain}"
        );
    }
    assert!(!separatist(rules::Alignment::ChaoticEvil, "law"));
    assert!(!separatist(rules::Alignment::ChaoticEvil, "good"));
    // His first domain, a cleric's, isn't checked against it here; since ticket 86 a cleric's alignment domain needs
    // its alignment (question 7), so a lawful good one can't take Chaos at all.
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

#[test]
fn a_ranger_10_slayer_2_takes_the_slayers_2_or_4_feat_from_the_first_stage() {
    let mut keys = vec!["ranger"; 10];
    keys.extend(["slayer", "slayer"]);
    let build = with_option(
        classes(&keys),
        2,
        0,
        "ranger/combat-style-feat/archery-feats",
    );
    let build = with_option(build, 12, 0, "slayer/slayer-talents/combat-style-feat-2");
    // One shared style: the slayer picks none.
    assert!(
        engine()
            .evaluate(&build)
            .slot(&option_slot(12, 0))
            .is_none()
    );
    let feat_slot = SlotId::OptionFeat {
        level: 12,
        index: 0,
    };
    let offered = feats_offered(&build, &feat_slot);
    assert!(offered.contains("rapid-shot"), "{offered:?}");
    assert!(
        !offered.contains("manyshot") && !offered.contains("improved-critical"),
        "{offered:?}"
    );
    // The ranger's own slot at 10 is the style's 10th-level list.
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
            Pick::MythicAbility(rules::MythicAbilityKey::new("second-bloodrager-bloodline")),
        );
        let options: HashMap<String, Result<(), Issue>> = engine()
            .options(&build, &SlotId::MythicSelection { rank: 1, index: 0 })
            .unwrap()
            .into_iter()
            .map(|SlotOption { pick, status, .. }| (pick.key().to_owned(), status))
            .collect();
        options
    };
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
