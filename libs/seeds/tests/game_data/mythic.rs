//! Mythic: the paths' rules (ticket 26), Mythic Extra Feat and Extra Mythic Ability's picks (ticket 55), and Witch
//! Wandering Hex and Beneficial Curse's picks (ticket 62).

use std::collections::BTreeSet;

use rules::{
    Ability, Build, FeatKey, FeatSlot, FeatureKey, Issue, MythicPathKey, Parameter, Pick,
    Prerequisite, RankStatus, Repeat, SelectionOptionKey, SlotId, SlotKind, SlotOption, Unmet,
    WeaponProficiency,
};

use crate::support::*;

// ---- Mythic path rules (ticket 26) ------------------------------------------------------------------------------

#[test]
fn the_mr_3_path_gives_its_mr_1_and_2_rows_without_a_first_ascension_twelve_rows_and_thirty_features()
 {
    let rows: Vec<_> = DATA
        .mythic_paths
        .iter()
        .flat_map(|path| &path.ranks)
        .filter(|row| row.rank < 3)
        .collect();
    assert_eq!(rows.len(), 12);
    assert_eq!(rows.iter().map(|row| row.features.len()).sum::<usize>(), 30);

    let sheet = engine().evaluate(&mythic("aeon", 9));
    let force_reality = FeatureKey::new("aeon/force-reality");
    // Without a first ascension of its own, the Build takes its MR 3 path's (ticket 91): Force Reality at MR 1
    // (level 6) and again, improved, at MR 2 (level 7).
    assert_eq!(sheet.at(5).unwrap().feature_count(&force_reality), 0);
    assert_eq!(sheet.at(6).unwrap().feature_count(&force_reality), 1);
    assert_eq!(sheet.at(7).unwrap().feature_count(&force_reality), 2);
    assert_eq!(sheet.at(9).unwrap().feature_count(&force_reality), 2);
    assert_eq!(sheet.ranks[0].path, Some(MythicPathKey::new("aeon")));
}

#[test]
fn an_aeon_gaze_needing_rank_5_opens_at_mr_5() {
    let build = mythic("aeon", 14);
    let area_effects = pick("aeon/enforcing-gaze/enforcing-gaze-area-effects");
    let status = |rank| {
        engine()
            .options(&build, &path_selection(rank))
            .unwrap()
            .into_iter()
            .find(|option| option.pick == area_effects)
            .unwrap()
            .status
    };
    assert_eq!(
        status(4),
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::MythicRank(5),
            has: Some(4)
        }))
    );
    assert_eq!(status(5), Ok(()));
}

#[test]
fn a_legend_has_4_more_in_every_score_from_the_switch() {
    let mut build = mythic("trickster", 18);
    build.path_switch = Some(MythicPathKey::new("legend"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let (before, after) = (sheet.at(16).unwrap(), sheet.at(17).unwrap());
    for ability in [Ability::Strength, Ability::Charisma] {
        assert_eq!(
            after.ability_scores.get(ability),
            before.ability_scores.get(ability) + 4
        );
    }
    assert_eq!(after.mythic_rank, 3);
    // The Trickster's MR 1-2 rows stay, its MR 3-7 ones go.
    assert_eq!(
        after.feature_count(&FeatureKey::new("trickster/bit-of-fun")),
        1
    );
    assert_eq!(
        after.feature_count(&FeatureKey::new("trickster/sneaky-quack")),
        0
    );
}

#[test]
fn the_devil_switch_is_open_from_aeon_or_azata_only() {
    let open = |path: &str| {
        engine()
            .switch_options(&mythic(path, 0))
            .into_iter()
            .find(|option| option.path.as_str() == "devil")
            .unwrap()
            .status
            .is_ok()
    };
    assert!(open("aeon") && open("azata"));
    assert!(!open("trickster") && !open("lich"));
}

#[test]
fn a_devil_from_aeon_loses_the_gaze_picks_and_keeps_the_rest() {
    let mut build = mythic("aeon", 17);
    build.picks.insert(
        path_selection(3),
        pick("aeon/enforcing-gaze/enforcing-gaze-attack"),
    );
    build.path_switch = Some(MythicPathKey::new("devil"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let at_17 = sheet.at(17).unwrap();
    assert!(at_17.selection_options.is_empty());
    assert_eq!(
        at_17.feature_count(&FeatureKey::new("aeon/enforcing-gaze-extra-gaze")),
        0
    );
    assert_eq!(at_17.feature_count(&FeatureKey::new("aeon/aeon-bane")), 1);
    assert_eq!(sheet.ranks[2].status, RankStatus::PartlyDropped);
    assert_eq!(sheet.ranks[2].dropped_slots, [path_selection(3)]);
}

#[test]
fn a_gold_dragon_takes_three_dragon_feats_ignoring_prerequisites_and_rises_in_every_score() {
    let mut build = mythic("trickster", 19);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    // Improved Critical is taken for a weapon.
    for (index, pick) in [
        Pick::feat_for(FeatKey::new("improved-critical"), weapon("longsword")),
        Pick::feat(FeatKey::new("great-cleave")),
        Pick::feat(FeatKey::new("stat-focus")),
    ]
    .into_iter()
    .enumerate()
    {
        build.picks.insert(
            SlotId::PathFeat {
                rank: 9,
                index: index as u8,
            },
            pick,
        );
    }
    // Annihilating Breath needs MR 9: open at the MR 9 pick, not at MR 8's, though both come at 17.
    let annihilating = "gold-dragon/improved-gold-dragon-breath/annihilating-breath";
    build.picks.insert(path_selection(9), pick(annihilating));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let status = engine()
        .options(&build, &path_selection(8))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == pick(annihilating))
        .unwrap()
        .status;
    assert!(status.is_err());
    // Perfect Body and Mind at 17 (every score at least 18), Perfect Soul at 19.
    let at_19 = sheet.at(19).unwrap();
    assert!(
        rules::AbilityScores::ABILITIES
            .iter()
            .all(|ability| at_19.ability_scores.get(*ability) >= 18)
    );
}

#[test]
fn all_skilled_grants_every_weapon_armour_and_shield_proficiency() {
    let mut build = mythic("azata", 11);
    build
        .picks
        .insert(path_selection(4), pick("azata/superpowers/all-skilled"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let feats = &sheet.at(11).unwrap().feats;
    for feat in [
        "simple-weapon-proficiency",
        "martial-weapons-proficiency",
        "exotic-weapon-proficiency",
        "armor-proficiency-light-armor",
        "armor-proficiency-medium-armor",
        "armor-proficiency-heavy-armor",
        "shield-proficiency",
        "tower-shield-proficiency",
    ] {
        assert!(feats.contains_key(&FeatKey::new(feat)), "{feat}");
    }
    assert!(!feats.contains_key(&FeatKey::new("armor-proficiency-light-barding")));
    // "Proficient with all types of weapons": Exotic Weapon Proficiency in every exotic weapon.
    let exotic = DATA
        .weapons
        .iter()
        .filter(|w| w.proficiency == Some(WeaponProficiency::Exotic))
        .map(|w| Parameter::Weapon(w.key.clone()))
        .collect::<BTreeSet<_>>();
    // The guide's 16 and the Sawtooth Saber (ticket 32).
    assert_eq!(exotic.len(), 17);
    assert_eq!(values_at(&build, 11, "exotic-weapon-proficiency"), exotic);
}

// ---- Mythic Extra Feat and Extra Mythic Ability give their pick (ticket 55) ----------------------------------------

/// The slot a rank's general mythic feat gives.
fn mythic_extra(rank: u8) -> SlotId {
    SlotId::MythicExtra { rank }
}

/// A Build of `levels` levels of `class` with `mythic_feat` as MR 2's general pick (MR 2 is at level 7).
fn with_mythic_feat(class: &str, levels: usize, mythic_feat: &str) -> Build {
    let mut build = build(class, levels);
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new(mythic_feat)),
    );
    build
}

/// Every option a slot's picker offers, with its status.
fn slot_options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
}

/// "You can select any normal feat." / "You can select a mythic ability." (mythic-paths#extra-feat,
/// mythic-paths#extra-mythic-ability): each gives one pick, and only these two; taken once, as the guide says no
/// more.
#[test]
fn extra_feat_and_extra_mythic_ability_are_the_mythic_feats_that_give_a_pick() {
    let giving: Vec<(String, rules::MythicExtraPick, Repeat)> = DATA
        .mythic_feats
        .iter()
        .filter_map(|feat| Some((feat.key.to_string(), feat.extra_pick?, feat.repeat)))
        .collect();
    assert_eq!(
        giving,
        [
            (
                "extra-feat".to_owned(),
                rules::MythicExtraPick::Feat,
                Repeat::Once
            ),
            (
                "extra-mythic-ability".to_owned(),
                rules::MythicExtraPick::MythicAbility,
                Repeat::Once
            ),
        ]
    );
}

#[test]
fn extra_feat_at_mr_2_gives_a_feat_slot_at_level_7_offering_what_the_general_slot_there_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", 7, "extra-feat");
    let sheet = engine.evaluate(&build);
    let rank_2: Vec<(SlotId, SlotKind)> = sheet.ranks[1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect();
    assert_eq!(
        rank_2,
        [
            (SlotId::Mythic { rank: 2 }, SlotKind::MythicFeat),
            (mythic_extra(2), SlotKind::Feat(FeatSlot::General)),
        ]
    );
    assert_eq!(sheet.ranks[1].character_level, 7);
    assert_eq!(engine.slot_label(&sheet.ranks[1].slots[1]), "Feat");
    // Every feat, each open or locked as in level 7's general slot.
    let extra = slot_options(&build, &mythic_extra(2));
    assert_eq!(extra.len(), slot_options(&build, &general_slot(7)).len());
    assert_eq!(extra, slot_options(&build, &general_slot(7)));
    assert!(extra.iter().any(|option| option.status.is_ok()));

    // A pick there is held from level 7; its prerequisites are checked.
    let mut with_pick = build.clone();
    with_pick
        .picks
        .insert(mythic_extra(2), feat_pick_plain("toughness"));
    assert!(possible(&with_pick));
    assert!(
        engine
            .evaluate(&with_pick)
            .at(7)
            .unwrap()
            .feats
            .contains_key(&FeatKey::new("toughness"))
    );
    let mut unmet = build;
    unmet
        .picks
        .insert(mythic_extra(2), feat_pick_plain("great-cleave"));
    assert!(matches!(
        why_not(&unmet, &mythic_extra(2)),
        Some(Issue::Unmet(_))
    ));
}

#[test]
fn extra_mythic_ability_gives_a_mythic_ability_slot_offering_what_a_ranks_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", 7, "extra-mythic-ability");
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&mythic_extra(2)).expect("the extra slot");
    assert_eq!(slot.kind, SlotKind::MythicAbility);
    assert_eq!(engine.slot_label(slot), "Mythic ability");
    let picks = |slot: &SlotId| -> BTreeSet<Pick> {
        slot_options(&build, slot)
            .into_iter()
            .map(|option| option.pick)
            .collect()
    };
    assert_eq!(picks(&mythic_extra(2)), picks(&SlotId::Mythic { rank: 1 }));
    assert_eq!(picks(&mythic_extra(2)).len(), DATA.mythic_abilities.len());

    // A sorcerer's Second Bloodline there gives another bloodline at MR 2.
    let mut sorcerer = with_mythic_feat("sorcerer", 7, "extra-mythic-ability");
    sorcerer.picks.insert(
        mythic_extra(2),
        Pick::MythicAbility(rules::MythicAbilityKey::new("second-bloodline")),
    );
    let second = SlotId::MythicSelection { rank: 2, index: 0 };
    assert_eq!(
        kind_at(&sorcerer, &second),
        Some(selection_kind("sorcerer/bloodline"))
    );
    let open = slot_options(&sorcerer, &second)
        .into_iter()
        .find(|option| option.status.is_ok())
        .expect("an open bloodline");
    sorcerer.picks.insert(second, open.pick);
    assert!(possible(&with_selections_filled(sorcerer)));
}

#[test]
fn removing_extra_feat_or_extra_mythic_ability_is_refused_while_its_slot_holds_a_pick() {
    let engine = engine();
    for (mythic_feat, pick) in [
        ("extra-feat", feat_pick_plain("toughness")),
        (
            "extra-mythic-ability",
            Pick::MythicAbility(rules::MythicAbilityKey::new("unstoppable")),
        ),
    ] {
        let mut build = with_mythic_feat("fighter", 7, mythic_feat);
        build.picks.insert(mythic_extra(2), pick);
        assert!(possible(&build), "{mythic_feat}");
        let feat = rules::Choice::Slot(SlotId::Mythic { rank: 2 });
        assert!(!engine.is_removable(&build, &feat), "{mythic_feat}");
        assert!(!engine.removable(&build).contains(&feat));
        assert!(engine.is_removable(&build, &rules::Choice::Slot(mythic_extra(2))));

        build.apply(&rules::Edit::Clear(mythic_extra(2)));
        assert!(engine.is_removable(&build, &feat), "{mythic_feat}");
        build.apply(&rules::Edit::Clear(SlotId::Mythic { rank: 2 }));
        assert!(possible(&build));
        assert_eq!(kind_at(&build, &mythic_extra(2)), None);
    }
}

#[test]
fn extra_rage_power_through_extra_feat_gives_a_rage_power_slot_at_the_ranks_level() {
    let mut build = with_selections_filled(barbarian(7));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("extra-feat")),
    );
    build
        .picks
        .insert(mythic_extra(2), feat_pick_plain("extra-rage-power"));
    assert_eq!(
        kind_at(&build, &extra_slot(7, 0)),
        Some(selection_kind("barbarian/rage-power"))
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(7, 0)));
    assert!(possible(&filled));
}

// ---- Witch Wandering Hex and Beneficial Curse give their pick (ticket 62) -----------------------------------------

fn mythic_ability_data(key: &str) -> &'static rules::MythicAbility {
    DATA.mythic_abilities
        .iter()
        .find(|ability| ability.key.as_str() == key)
        .unwrap_or_else(|| panic!("no mythic ability {key}"))
}

/// What MR 1's mythic ability picker says of `ability`.
fn mr_1_status(build: &Build, ability: &str) -> Result<(), Issue> {
    slot_options(build, &SlotId::Mythic { rank: 1 })
        .into_iter()
        .find(|option| option.pick.key() == ability)
        .unwrap_or_else(|| panic!("MR 1 doesn't offer {ability}"))
        .status
}

/// "Witch Wandering Hex. Prequisites: Hex. You can select any hex for which you meet the prerequisites." and
/// "Beneficial Curse. Prerequisites: Oracle's Curse. You gain an additional Curse with no penalties."
/// (mythic-paths): one slot of the witch's hexes, needing a hex held as Extra Hex does ("Prerequisites: Hex"), and one
/// of the oracle's curses, needing a curse held.
#[test]
fn witch_wandering_hex_and_beneficial_curse_give_a_slot_of_their_selection() {
    let hex = mythic_ability_data(WITCH_WANDERING_HEX);
    assert_eq!(
        hex.selections,
        [rules::FeatureSelectionKey::new("witch/hex")]
    );
    let mut needs = hex.prerequisite.clone().expect("a prerequisite");
    needs.canonicalize();
    let mut extra_hex = feat("extra-hex").prerequisite.clone().unwrap();
    extra_hex.canonicalize();
    assert_eq!(needs, extra_hex);

    let curse = mythic_ability_data(BENEFICIAL_CURSE);
    assert_eq!(curse.selections, [rules::FeatureSelectionKey::new(CURSES)]);
    let held: BTreeSet<String> = match curse.prerequisite.as_ref() {
        Some(Prerequisite::Any(children)) => children
            .iter()
            .map(|child| match child {
                Prerequisite::SelectionOption(option) => option.to_string(),
                other => panic!("{other:?}"),
            })
            .collect(),
        other => panic!("{other:?}"),
    };
    let curses: BTreeSet<String> = class("oracle")
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == CURSES)
        .unwrap()
        .options
        .iter()
        .map(|option| option.key.to_string())
        .collect();
    assert_eq!(held, curses);
    assert_eq!(curses.len(), 9);

    // The mythic abilities that give a pick: these two and ticket 55's five.
    let giving: BTreeSet<&str> = DATA
        .mythic_abilities
        .iter()
        .filter(|ability| !ability.selections.is_empty())
        .map(|ability| ability.key.as_str())
        .collect();
    assert_eq!(
        giving,
        BTreeSet::from([
            BENEFICIAL_CURSE,
            "impossible-domain",
            "second-bloodline",
            "second-bloodrager-bloodline",
            "second-mystery",
            "second-spirit",
            WITCH_WANDERING_HEX,
        ])
    );
}

#[test]
fn witch_wandering_hex_is_locked_without_a_hex() {
    let fighter = build("fighter", 6);
    assert!(matches!(
        mr_1_status(&fighter, WITCH_WANDERING_HEX),
        Err(Issue::Unmet(_))
    ));
    assert!(matches!(
        why_not(
            &with_mr_1_ability(fighter, WITCH_WANDERING_HEX),
            &SlotId::Mythic { rank: 1 }
        ),
        Some(Issue::Unmet(_))
    ));
    // A witch's hex or a shaman's opens it.
    assert_eq!(
        mr_1_status(
            &with_selections_filled(build("witch", 6)),
            WITCH_WANDERING_HEX
        ),
        Ok(())
    );
    let shaman = with_selections_filled(build("shaman", 6));
    assert!(
        at(&shaman, 6)
            .selection_options
            .keys()
            .any(|option| option.as_str().starts_with("shaman/hex/"))
    );
    assert_eq!(mr_1_status(&shaman, WITCH_WANDERING_HEX), Ok(()));
}

#[test]
fn witch_wandering_hex_gives_a_witch_hex_slot_offering_hexes_whose_prerequisites_are_met() {
    let engine = engine();
    let witch = with_mr_1_ability(
        with_selections_filled(build("witch", 6)),
        WITCH_WANDERING_HEX,
    );
    assert!(possible(&witch));
    let sheet = engine.evaluate(&witch);
    let slot = sheet.slot(&mr_1_selection()).expect("the hex slot");
    assert_eq!(slot.kind, selection_kind("witch/hex"));
    assert_eq!(engine.slot_label(slot), "Hex");
    // The witch's hexes, each open as the witch's own slot at 6 would leave it, but the ones held.
    let hexes = offered_in(&witch, &mr_1_selection());
    let held: BTreeSet<String> = witch
        .picks
        .values()
        .map(|pick| pick.key().to_owned())
        .filter(|key| key.starts_with("witch/hex/"))
        .collect();
    assert_eq!(held.len(), 4);
    for hex in &held {
        assert_eq!(hexes[hex], Err(Issue::AlreadyHeld), "{hex}");
    }
    let free = first_open_in_mr_1_selection(&witch);
    assert!(free.starts_with("witch/hex/") && !held.contains(&free));
    assert!(matches!(hexes["witch/hex/agony"], Err(Issue::Unmet(_))));
    // At MR 5 (level 14) a witch has her major hexes (from 10).
    let mut witch_14 = with_selections_filled(build("witch", 14));
    witch_14.mythic_path = Some(MythicPathKey::new("trickster"));
    witch_14.picks.insert(
        SlotId::Mythic { rank: 5 },
        Pick::MythicAbility(rules::MythicAbilityKey::new(WITCH_WANDERING_HEX)),
    );
    let rank_5 = SlotId::MythicSelection { rank: 5, index: 0 };
    let major = offered_in(&witch_14, &rank_5);
    assert!(
        !witch_14
            .picks
            .values()
            .any(|pick| pick.key() == "witch/hex/major-healing")
    );
    assert_eq!(major["witch/hex/major-healing"], Ok(()));
    assert!(matches!(
        hexes["witch/hex/major-healing"],
        Err(Issue::Unmet(_))
    ));

    // A pick there is held from MR 1's level; a hex not open there is refused.
    let mut third = witch.clone();
    third.picks.insert(mr_1_selection(), pick(&free));
    assert!(possible(&third));
    assert_eq!(
        at(&third, 6).option_count(&SelectionOptionKey::new(&free)),
        1
    );
    let mut agony = witch;
    agony
        .picks
        .insert(mr_1_selection(), pick("witch/hex/agony"));
    assert!(matches!(
        why_not(&agony, &mr_1_selection()),
        Some(Issue::Unmet(_))
    ));

    // A shaman's slot is a witch hex slot too; a witch hex the shaman took through her own hexes is held.
    let mut shaman = with_selections_filled(build("shaman", 6));
    shaman.picks.insert(
        SlotId::Selection { level: 2, index: 0 },
        pick("witch/hex/evil-eye"),
    );
    let shaman = with_mr_1_ability(shaman, WITCH_WANDERING_HEX);
    assert!(possible(&shaman));
    assert_eq!(
        kind_at(&shaman, &mr_1_selection()),
        Some(selection_kind("witch/hex"))
    );
    let hexes = offered_in(&shaman, &mr_1_selection());
    assert_eq!(hexes["witch/hex/evil-eye"], Err(Issue::AlreadyHeld));
    assert_eq!(hexes["witch/hex/cackle"], Ok(()));
}

/// The first option, in key order, open in MR 1's selection slot.
fn first_open_in_mr_1_selection(build: &Build) -> String {
    offered_in(build, &mr_1_selection())
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .min()
        .expect("an open option")
}

#[test]
fn beneficial_curse_gives_an_oracle_curse_slot_offering_only_curses_not_held() {
    let engine = engine();
    let oracle = with_selections_filled(build("oracle", 6));
    assert_eq!(mr_1_status(&oracle, BENEFICIAL_CURSE), Ok(()));
    assert!(matches!(
        mr_1_status(&build("fighter", 6), BENEFICIAL_CURSE),
        Err(Issue::Unmet(_))
    ));
    let oracle = with_mr_1_ability(oracle, BENEFICIAL_CURSE);
    assert!(possible(&oracle));
    let sheet = engine.evaluate(&oracle);
    let slot = sheet.slot(&mr_1_selection()).expect("the curse slot");
    assert_eq!(slot.kind, selection_kind(CURSES));
    assert_eq!(engine.slot_label(slot), "Oracle's Curse");
    let held = oracle
        .picks
        .values()
        .map(|pick| pick.key().to_owned())
        .find(|key| key.starts_with("oracle/oracles-curse/"))
        .expect("a curse held");
    let curses = offered_in(&oracle, &mr_1_selection());
    assert_eq!(curses.len(), 9);
    for (curse, status) in &curses {
        if *curse == held {
            assert_eq!(*status, Err(Issue::AlreadyHeld));
        } else {
            assert_eq!(*status, Ok(()), "{curse}");
        }
    }
    let other = curses.keys().find(|curse| **curse != held).unwrap().clone();
    let mut second = oracle.clone();
    second.picks.insert(mr_1_selection(), pick(&other));
    assert!(possible(&second));
    assert_eq!(
        at(&second, 6).option_count(&SelectionOptionKey::new(&other)),
        1
    );
    let mut twice = oracle;
    twice.picks.insert(mr_1_selection(), pick(&held));
    assert_eq!(why_not(&twice, &mr_1_selection()), Some(Issue::AlreadyHeld));
}

#[test]
fn removing_witch_wandering_hex_or_beneficial_curse_is_refused_while_its_slot_holds_a_pick() {
    let engine = engine();
    for (class, ability) in [("witch", WITCH_WANDERING_HEX), ("oracle", BENEFICIAL_CURSE)] {
        let mut build = with_mr_1_ability(with_selections_filled(build(class, 6)), ability);
        let option = first_open_in_mr_1_selection(&build);
        build.picks.insert(mr_1_selection(), pick(&option));
        assert!(possible(&build), "{ability}");
        let choice = rules::Choice::Slot(SlotId::Mythic { rank: 1 });
        assert!(!engine.is_removable(&build, &choice), "{ability}");
        assert!(!engine.removable(&build).contains(&choice));
        assert!(engine.is_removable(&build, &rules::Choice::Slot(mr_1_selection())));

        build.apply(&rules::Edit::Clear(mr_1_selection()));
        assert!(engine.is_removable(&build, &choice), "{ability}");
        build.apply(&rules::Edit::Clear(SlotId::Mythic { rank: 1 }));
        assert!(possible(&build));
        assert_eq!(kind_at(&build, &mr_1_selection()), None);
    }
}
