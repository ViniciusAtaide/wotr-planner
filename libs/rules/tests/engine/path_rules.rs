//! The mythic path rules (ticket 26): MR 1–2 from the path chosen at MR 3 unless the first ascension names another
//! (ticket 91, `first_ascension`), what a Path switch keeps and drops, the
//! switch's previous-path condition, ability bonuses and floors from features, Legend and the Trickster's Perception II
//! feats, the Mythic rank prerequisite, Gold Dragon's Dragon Feats and an option granting several feats. A switch that
//! ends the ascensions, and Remove level, drop the picks of the ranks the Build no longer reaches (ticket 40). Remove
//! level also drops the path and the switch once their ranks aren't reached (ticket 41). The picks a switch takes away
//! at earlier ranks stay in the Build, held until the switch (ticket 42).

use std::collections::BTreeSet;

use rules::{
    Ability, Build, Choice, Edit, FeatSlot, Issue, MythicAbilityKey, MythicFeatKey, Prerequisite,
    RankStatus, SkillKey, SlotId, SlotKind, Unmet,
};

use crate::support::*;

/// A fighter of `levels` levels on `path` (chosen at MR 3) with a general mythic pick at every rank it reaches, and
/// the feats the mythic feats need.
fn mythic_build(path_key: &str, levels: usize) -> Build {
    let mut build = build("human", &times("fighter", levels));
    build.mythic_path = Some(path(path_key));
    let abilities = [
        "mythic-charge",
        "limitless-smite",
        "unstoppable",
        "inspired-strike",
        "dimensional-retribution",
    ];
    let feats = [
        "extra-mythic-ability",
        "improved-critical-mythic",
        "dodge-mythic",
        "power-attack-mythic",
        "toughness-mythic",
    ];
    for (i, key) in abilities.iter().enumerate() {
        let rank = 2 * i as u8 + 1;
        build
            .picks
            .insert(SlotId::Mythic { rank }, pick_mythic_ability(key));
    }
    for (i, key) in feats.iter().enumerate() {
        let rank = 2 * i as u8 + 2;
        build
            .picks
            .insert(SlotId::Mythic { rank }, pick_mythic_feat(key));
    }
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    // Only the ranks the Build reaches.
    let reached: Vec<u8> = world()
        .progression
        .mythic_ascensions
        .iter()
        .filter(|ascension| usize::from(ascension.character_level) <= levels)
        .map(|ascension| ascension.rank)
        .collect();
    build
        .picks
        .retain(|slot, _| slot.rank().is_none_or(|rank| reached.contains(&rank)));
    build
}

fn path_selection(rank: u8) -> SlotId {
    SlotId::PathSelection { rank, index: 0 }
}

/// The Trickster's bonus feat slot at MR 4.
fn path_feat_4() -> SlotId {
    SlotId::PathFeat { rank: 4, index: 0 }
}

/// The Build with `key`'s Path switch, as the sheet applies it.
fn switched(mut build: Build, key: &str) -> Build {
    engine().apply(&mut build, &Edit::PathSwitch(Some(path(key))));
    build
}

/// The Path switch picker's status for `key`.
fn switch_status(build: &Build, key: &str) -> Result<(), Issue> {
    engine()
        .switch_options(build)
        .into_iter()
        .find(|option| option.path == path(key))
        .unwrap()
        .status
}

fn statuses(build: &Build) -> Vec<RankStatus> {
    engine()
        .evaluate(build)
        .ranks
        .iter()
        .map(|rank| rank.status)
        .collect()
}

fn needs(ability: Ability, score: u8) -> Prerequisite {
    Prerequisite::AbilityScore { ability, score }
}

/// The scores in the order Str, Dex, Con, Int, Wis, Cha.
fn scores_of(state: &rules::CharacterState) -> [u8; 6] {
    let scores = state.ability_scores;
    [
        scores.strength,
        scores.dexterity,
        scores.constitution,
        scores.intelligence,
        scores.wisdom,
        scores.charisma,
    ]
}

// ---- MR 1–2 from the MR 3 path ----------------------------------------------------------------------------------

/// Ticket 91: a Build without a first ascension of its own takes its MR 3 path's, so the path's MR 1 and 2 rows are
/// held from their own ranks (ticket 26 held them from MR 3).
#[test]
fn the_mr_3_path_gives_its_mr_1_and_2_rows_at_their_ranks_without_a_first_ascension() {
    let build = mythic_build("trickster", 11);
    let sheet = engine().evaluate(&build);

    // MR 1 and 2 are reached at 6 and 7: each row is held from its rank.
    let at_5 = sheet.at(5).unwrap();
    assert_eq!(at_5.feature_count(&feature("trickster/bit-of-fun")), 0);
    let at_6 = sheet.at(6).unwrap();
    assert_eq!(at_6.feature_count(&feature("trickster/bit-of-fun")), 1);
    assert_eq!(at_6.feature_count(&feature("trickster/hard-to-kill")), 0);
    let at_8 = sheet.at(8).unwrap();
    assert_eq!(at_8.feature_count(&feature("trickster/hard-to-kill")), 1);
    let before_9 = engine().state_before(&build, &general_slot(9)).unwrap();
    assert_eq!(before_9.feature_count(&feature("trickster/bit-of-fun")), 1);

    // Both rows stay held, once each, and MR 1 and 2 show the path.
    let at_9 = sheet.at(9).unwrap();
    assert_eq!(at_9.feature_count(&feature("trickster/bit-of-fun")), 1);
    assert_eq!(at_9.feature_count(&feature("trickster/hard-to-kill")), 1);
    let before_11 = engine().state_before(&build, &general_slot(11)).unwrap();
    assert_eq!(before_11.feature_count(&feature("trickster/bit-of-fun")), 1);
    assert_eq!(sheet.ranks[0].path, Some(path("trickster")));
    assert_eq!(sheet.ranks[1].path, Some(path("trickster")));
    // The general picks of MR 1 and 2 are where they were.
    assert_eq!(
        sheet.ranks[0].slots[0].kind,
        SlotKind::MythicAbility,
        "MR 1 has its general pick only"
    );
    assert_eq!(sheet.ranks[0].slots.len(), 1);
}

#[test]
fn a_build_that_stops_before_mr_3_without_a_first_ascension_shows_no_path_at_mr_1_and_2() {
    let sheet = engine().evaluate(&mythic_build("trickster", 8));
    assert_eq!(sheet.ranks[0].path, None);
    assert_eq!(sheet.ranks[1].path, None);
}

// ---- Partial keeps on a switch ----------------------------------------------------------------------------------

/// A Trickster of 20 levels with a trick at MR 3 and 5 and a bonus feat at MR 4 besides its general picks.
fn trickster_with_path_picks() -> Build {
    let mut build = mythic_build("trickster", 20);
    build.picks.insert(
        path_selection(3),
        pick_option("trickster/tricks/perception-1"),
    );
    build.picks.insert(path_feat_4(), pick_feat("dodge"));
    build.picks.insert(
        path_selection(5),
        pick_option("trickster/tricks/mobility-1"),
    );
    build
}

#[test]
fn gold_dragon_and_the_swarm_keep_the_old_paths_picks_of_mr_3_to_7_in_the_build() {
    let build = trickster_with_path_picks();
    assert!(engine().is_possible(&build));
    for key in ["gold-dragon", "swarm-that-walks"] {
        // Not locked: no choice from the switch on needs the picks it takes away.
        assert_eq!(switch_status(&build, key), Ok(()), "{key}");
        // They were held from their ranks until the switch: the Build keeps them.
        let switched = switched(build.clone(), key);
        assert_eq!(switched.picks, build.picks, "{key}");
        assert!(engine().is_possible(&switched), "{key}");
    }
}

#[test]
fn gold_dragon_keeps_the_general_picks_of_mr_1_to_7_but_the_old_paths_grants_of_mr_1_and_2_only() {
    let build = switched(trickster_with_path_picks(), "gold-dragon");
    let sheet = engine().evaluate(&build);
    assert!(engine().is_possible(&build));

    // The tricks and the bonus feat are held until the switch.
    let at_16 = sheet.at(16).unwrap();
    assert_eq!(at_16.selection_options.len(), 2);
    assert!(at_16.feats.contains_key(&feat("dodge")));
    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.mythic_path, Some(path("gold-dragon")));
    // The general picks of MR 1-7 stay (with MR 8's and 9's, both at 17).
    assert_eq!(at_17.mythic_abilities.len(), 5);
    assert_eq!(
        at_17.mythic_feats,
        [
            MythicFeatKey::new("extra-mythic-ability"),
            MythicFeatKey::new("improved-critical-mythic"),
            MythicFeatKey::new("dodge-mythic"),
            MythicFeatKey::new("power-attack-mythic"),
        ]
        .into()
    );
    // The Trickster's MR 1-2 rows stay; its MR 3-7 grants and picks go.
    assert_eq!(at_17.feature_count(&feature("trickster/bit-of-fun")), 1);
    assert_eq!(at_17.feature_count(&feature("trickster/hard-to-kill")), 1);
    assert_eq!(
        at_17.feature_count(&feature("trickster/trickster-tricks")),
        0
    );
    assert!(at_17.selection_options.is_empty());
    assert_eq!(at_17.feature_count(&feature("gold-dragon/breath")), 1);

    use RankStatus::*;
    assert_eq!(
        statuses(&build),
        [
            Reached,
            Reached,
            PartlyDropped,
            PartlyDropped,
            PartlyDropped,
            PartlyDropped,
            PartlyDropped,
            Reached,
            Reached,
            Reached
        ]
    );
    // The track marks the old path's picks of MR 3-7 dropped; they stay in their slots.
    assert_eq!(sheet.ranks[2].dropped_slots, [path_selection(3)]);
    assert_eq!(sheet.ranks[3].dropped_slots, [path_feat_4()]);
    assert_eq!(sheet.ranks[4].dropped_slots, [path_selection(5)]);
    assert_eq!(sheet.ranks[1].dropped_slots, []);
    assert_eq!(
        sheet.ranks[2].slots[1].pick,
        build.picks.get(&path_selection(3)).cloned()
    );
    // An empty slot of the old path is filled before the switch: it isn't marked, and it has a picker.
    assert_eq!(sheet.ranks[6].dropped_slots, []);
    assert!(engine().options(&build, &path_selection(7)).is_some());
}

#[test]
fn a_legend_switch_keeps_the_old_paths_mr_1_and_2_rows_and_drops_every_pick_of_mr_3_to_7() {
    let build = switched(trickster_with_path_picks(), "legend");
    assert!(engine().is_possible(&build));
    // MR 3-7's picks stay in the Build (MR 8-10's go: the switch ends the ascensions).
    assert_eq!(pick_ranks(&build), (1..=7).collect::<Vec<_>>());
    let sheet = engine().evaluate(&build);
    // Before the switch, every pick of MR 1-7 is held.
    let at_16 = sheet.at(16).unwrap();
    assert_eq!(at_16.mythic_abilities.len(), 4);
    assert_eq!(at_16.selection_options.len(), 2);
    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.feature_count(&feature("trickster/bit-of-fun")), 1);
    assert_eq!(at_17.feature_count(&feature("trickster/hard-to-kill")), 1);
    assert_eq!(
        at_17.mythic_abilities,
        [MythicAbilityKey::new("mythic-charge")].into()
    );
    assert!(at_17.selection_options.is_empty());

    // The track marks MR 3-7 dropped, with their picks lost from the switch on.
    use RankStatus::*;
    assert_eq!(
        statuses(&build),
        [
            Reached, Reached, Dropped, Dropped, Dropped, Dropped, Dropped, NotReached, NotReached,
            NotReached
        ]
    );
    assert_eq!(
        sheet.ranks[2].dropped_slots,
        [SlotId::Mythic { rank: 3 }, path_selection(3)]
    );
    assert_eq!(
        sheet.ranks[3].dropped_slots,
        [SlotId::Mythic { rank: 4 }, path_feat_4()]
    );
    // MR 7's trick slot is empty: only its general pick is lost. A lost slot keeps its pick.
    assert_eq!(sheet.ranks[6].dropped_slots, [SlotId::Mythic { rank: 7 }]);
    for rank in &sheet.ranks[2..7] {
        for slot in &rank.slots {
            let lost = rank.dropped_slots.contains(&slot.id);
            assert_eq!(lost, slot.pick.is_some(), "{:?}", slot.id);
        }
    }
}

/// The ranks of the Build's mythic picks.
fn pick_ranks(build: &Build) -> Vec<u8> {
    let ranks: BTreeSet<u8> = build.picks.keys().filter_map(SlotId::rank).collect();
    ranks.into_iter().collect()
}

#[test]
fn choosing_the_legend_switch_drops_the_picks_of_the_ranks_it_takes_away() {
    let engine = engine();
    let mut build = mythic_build("trickster", 20);
    build.picks.insert(
        path_selection(3),
        pick_option("trickster/tricks/perception-1"),
    );
    assert!(engine.is_possible(&build));

    // Gold Dragon ascends on: every pick stays.
    let mut gold = build.clone();
    engine.apply(&mut gold, &Edit::PathSwitch(Some(path("gold-dragon"))));
    assert_eq!(gold.picks, build.picks);

    // Legend ends the ascensions at MR 8: the picks of MR 8, 9 and 10 go. MR 3-7's stay, as the picks the switch
    // drops (the Mythic track marks them).
    build.picks.insert(
        path_selection(9),
        pick_option("trickster/tricks/athletics-1"),
    );
    assert!(engine.is_possible(&build));
    assert_eq!(pick_ranks(&build), (1..=10).collect::<Vec<_>>());
    assert_eq!(switch_status(&build, "legend"), Ok(()));
    engine.apply(&mut build, &Edit::PathSwitch(Some(path("legend"))));
    assert_eq!(build.path_switch, Some(path("legend")));
    assert_eq!(pick_ranks(&build), (1..=7).collect::<Vec<_>>());
    assert!(build.picks.contains_key(&path_selection(3)));
    assert!(engine.is_possible(&build));

    // Taking the switch back brings the ranks back, empty.
    engine.apply(&mut build, &Edit::PathSwitch(None));
    assert_eq!(pick_ranks(&build), (1..=7).collect::<Vec<_>>());
    assert_eq!(engine.evaluate(&build).ranks[9].status, RankStatus::Reached);
    assert!(engine.is_possible(&build));
}

#[test]
fn a_switch_is_locked_only_while_a_choice_from_its_rank_on_needs_a_pick_it_takes_away() {
    // Perception II (MR 5, level 14) unlocks Stat Focus.
    let mut build = mythic_build("trickster", 20);
    build.picks.insert(
        path_selection(3),
        pick_option("trickster/tricks/perception-1"),
    );
    build.picks.insert(
        path_selection(5),
        pick_option("trickster/tricks/perception-2"),
    );
    // Taken at 15, before the switch, Stat Focus stays: Legend, Gold Dragon and the Swarm are all open.
    let mut before = build.clone();
    before
        .picks
        .insert(general_slot(15), pick_feat("stat-focus"));
    assert!(engine().is_possible(&before));
    for key in ["legend", "gold-dragon", "swarm-that-walks"] {
        assert_eq!(switch_status(&before, key), Ok(()), "{key}");
        let switched = switched(before.clone(), key);
        assert!(switched.picks.contains_key(&general_slot(15)), "{key}");
        assert!(engine().is_possible(&switched), "{key}");
    }

    // Taken at 19, after the switch, it needs Perception II, which all three take away.
    let mut after = build.clone();
    after
        .picks
        .insert(general_slot(19), pick_feat("stat-focus"));
    assert!(engine().is_possible(&after));
    for key in ["legend", "gold-dragon", "swarm-that-walks"] {
        assert_eq!(switch_status(&after, key), Err(Issue::BreaksLater), "{key}");
        assert!(!engine().allows(&after, &Edit::PathSwitch(Some(path(key)))));
    }

    // A switch already held is removable while no later choice needs what it changes.
    let held = switched(build, "legend");
    assert!(engine().removable(&held).contains(&Choice::PathSwitch));
}

#[test]
fn removing_a_level_drops_its_picks_and_the_picks_of_the_ranks_it_no_longer_reaches() {
    let engine = engine();
    let mut build = mythic_build("trickster", 19);
    build.picks.insert(
        path_selection(9),
        pick_option("trickster/tricks/athletics-1"),
    );
    build.picks.insert(general_slot(17), pick_feat("dodge"));
    assert!(engine.is_possible(&build));
    assert_eq!(pick_ranks(&build), (1..=10).collect::<Vec<_>>());

    // Level 19 reaches MR 10.
    engine.apply(&mut build, &Edit::RemoveLevel);
    assert_eq!(pick_ranks(&build), (1..=9).collect::<Vec<_>>());
    engine.apply(&mut build, &Edit::RemoveLevel);
    assert_eq!(pick_ranks(&build), (1..=9).collect::<Vec<_>>());
    // Level 17 reaches MR 8 and 9: its own pick goes with theirs.
    engine.apply(&mut build, &Edit::RemoveLevel);
    assert_eq!(build.levels.len(), 16);
    assert_eq!(pick_ranks(&build), (1..=7).collect::<Vec<_>>());
    assert!(!build.picks.contains_key(&general_slot(17)));
    assert!(engine.is_possible(&build));

    // Adding the levels back brings the ranks back, empty.
    for level in 17..=19 {
        engine.apply(
            &mut build,
            &Edit::Class {
                level,
                class: class("fighter"),
                choice: None,
            },
        );
    }
    assert_eq!(pick_ranks(&build), (1..=7).collect::<Vec<_>>());
    assert!(engine.is_possible(&build));
}

#[test]
fn removing_the_level_of_mr_3_or_mr_8_drops_the_path_or_the_switch() {
    let engine = engine();
    let mut build = switched(mythic_build("aeon", 17), "devil");
    assert!(engine.is_possible(&build));
    assert_eq!(build.path_switch, Some(path("devil")));

    // Level 17 reaches MR 8, where the switch is.
    engine.apply(&mut build, &Edit::RemoveLevel);
    assert_eq!(build.path_switch, None);
    assert_eq!(build.mythic_path, Some(path("aeon")));
    assert!(engine.is_possible(&build));

    // Level 9 reaches MR 3, where the path is chosen.
    while build.levels.len() > 9 {
        engine.apply(&mut build, &Edit::RemoveLevel);
        assert_eq!(build.mythic_path, Some(path("aeon")));
    }
    engine.apply(&mut build, &Edit::RemoveLevel);
    assert_eq!(build.levels.len(), 8);
    assert_eq!(build.mythic_path, None);
    assert!(engine.is_possible(&build));

    // Adding the level back doesn't bring the path back.
    engine.apply(
        &mut build,
        &Edit::Class {
            level: 9,
            class: class("fighter"),
            choice: None,
        },
    );
    assert_eq!(build.mythic_path, None);
    assert!(engine.is_possible(&build));
}

#[test]
fn a_path_or_switch_whose_rank_is_not_reached_is_not_kept() {
    let engine = engine();
    // Set before its rank is reached, a path or switch leaves the Build as it was.
    let mut eight = mythic_build("trickster", 8);
    eight.mythic_path = None;
    let before = eight.clone();
    engine.apply(&mut eight, &Edit::MythicPath(Some(path("trickster"))));
    assert_eq!(eight, before);
    let mut sixteen = mythic_build("trickster", 16);
    engine.apply(&mut sixteen, &Edit::PathSwitch(Some(path("legend"))));
    assert_eq!(sixteen, mythic_build("trickster", 16));
}

#[test]
fn devil_from_aeon_keeps_mr_1_to_7_but_loses_the_gaze_picks_and_extra_gaze() {
    let mut build = mythic_build("aeon", 20);
    for (rank, gaze) in [
        (3, "attack"),
        (4, "damage"),
        (5, "area-effects"),
        (6, "save-dc"),
    ] {
        build.picks.insert(
            path_selection(rank),
            pick_option(&format!("aeon/enforcing-gaze/{gaze}")),
        );
    }
    assert!(engine().is_possible(&build));
    assert_eq!(switch_status(&build, "devil"), Ok(()));
    // The gaze picks stay in the Build, held until the switch.
    let picks = build.picks.clone();
    let build = switched(build, "devil");
    assert_eq!(build.picks, picks);
    let sheet = engine().evaluate(&build);
    assert!(engine().is_possible(&build));

    let at_16 = sheet.at(16).unwrap();
    assert_eq!(at_16.selection_options.len(), 4);
    assert_eq!(
        at_16.feature_count(&feature("aeon/enforcing-gaze-extra-gaze")),
        1
    );

    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.mythic_path, Some(path("devil")));
    assert!(at_17.selection_options.is_empty());
    assert_eq!(
        at_17.feature_count(&feature("aeon/enforcing-gaze-extra-gaze")),
        0
    );
    // The rest of the Aeon's MR 1-7 stays, general picks included (MR 8 and 9 add theirs at 17).
    assert_eq!(at_17.feature_count(&feature("aeon/aeon-bane")), 1);
    assert_eq!(at_17.feature_count(&feature("aeon/force-reality")), 1);
    assert_eq!(at_17.mythic_abilities.len(), 5);
    assert_eq!(at_17.mythic_feats.len(), 4);
    assert_eq!(at_17.feature_count(&feature("devil/hells-dominance")), 1);

    // The ranks with a gaze pick are partly dropped, the gaze lost from the switch on; MR 7's empty gaze slot is
    // filled before the switch and isn't marked.
    use RankStatus::*;
    assert_eq!(
        statuses(&build),
        [
            Reached,
            Reached,
            PartlyDropped,
            PartlyDropped,
            PartlyDropped,
            PartlyDropped,
            Reached,
            Reached,
            Reached,
            Reached
        ]
    );
    for rank in 3..=6 {
        let sheet = &sheet.ranks[usize::from(rank) - 1];
        assert_eq!(sheet.dropped_slots, [path_selection(rank)], "MR {rank}");
        assert!(sheet.slots[1].pick.is_some(), "MR {rank}");
    }
    assert_eq!(sheet.ranks[6].dropped_slots, []);
    assert!(engine().options(&build, &path_selection(7)).is_some());
    assert_eq!(sheet.ranks[1].dropped_slots, []);
}

// ---- The previous path ------------------------------------------------------------------------------------------

#[test]
fn the_devil_switch_is_open_only_from_its_previous_paths() {
    let mut from_trickster = mythic_build("trickster", 20);
    from_trickster.path_switch = Some(path("devil"));
    assert!(!engine().is_possible(&from_trickster));
    let expected = Issue::NotFromPath {
        from: [path("aeon")].into(),
    };

    let status = |build: &Build, key: &str| {
        engine()
            .switch_options(build)
            .into_iter()
            .find(|option| option.path == path(key))
            .unwrap()
            .status
    };
    assert_eq!(status(&from_trickster, "devil"), Err(expected.clone()));
    assert_eq!(status(&from_trickster, "legend"), Ok(()));
    assert_eq!(status(&mythic_build("aeon", 20), "devil"), Ok(()));
    // No path chosen yet: a switch with previous paths isn't open.
    assert_eq!(status(&build("human", &[]), "devil"), Err(expected));

    let mut from_aeon = mythic_build("aeon", 20);
    from_aeon.path_switch = Some(path("devil"));
    assert!(engine().is_possible(&from_aeon));
}

// ---- Ability bonuses from features ------------------------------------------------------------------------------

#[test]
fn the_legend_switch_raises_every_score_by_4_from_the_switch_on() {
    let build = switched(mythic_build("trickster", 20), "legend");
    let sheet = engine().evaluate(&build);

    let at_16 = sheet.at(16).unwrap();
    assert_eq!(scores_of(at_16), [16, 14, 12, 13, 10, 8]);
    assert!(at_16.check(&needs(Ability::Charisma, 12)).is_err());

    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.feature_count(&feature("legend/legend")), 1);
    assert_eq!(scores_of(at_17), [20, 18, 16, 17, 14, 12]);
    assert_eq!(
        scores_of(&rules::CharacterState {
            ability_scores: at_17.base_ability_scores,
            ..at_17.clone()
        }),
        [16, 14, 12, 13, 10, 8]
    );
    // They count toward prerequisites, and skill points follow the higher Intelligence (13 -> 17: +2 to the
    // modifier, a point a level).
    assert_eq!(at_17.check(&needs(Ability::Charisma, 12)), Ok(()));
    assert_eq!(
        sheet.levels[17].skill_points.available,
        sheet.levels[15].skill_points.available + 1
    );
}

#[test]
fn gold_dragons_perfect_body_mind_and_soul_set_floors_and_raise_the_highest_score() {
    let mut build = mythic_build("trickster", 20);
    build.path_switch = Some(path("gold-dragon"));
    let sheet = engine().evaluate(&build);

    // MR 8 (Perfect Body): every score at least 14, the highest (Str 16) +4.
    let after_8 = engine().state_before(&build, &path_selection(8)).unwrap();
    assert_eq!(scores_of(&after_8), [20, 14, 14, 14, 14, 14]);
    // MR 9 at the same level (Perfect Mind): at least 18, the highest another +2.
    assert_eq!(scores_of(sheet.at(17).unwrap()), [24, 18, 18, 18, 18, 18]);
    assert_eq!(
        sheet.at(17).unwrap().check(&needs(Ability::Wisdom, 18)),
        Ok(())
    );
    // MR 10 (Perfect Soul): the highest another +2, +8 in all.
    assert_eq!(scores_of(sheet.at(19).unwrap()), [26, 18, 18, 18, 18, 18]);
}

#[test]
fn the_highest_score_is_taken_before_the_floors_and_a_tie_goes_to_the_first_in_str_to_cha_order() {
    // Provisional (ticket 26, open question): the guide doesn't say which of two equal highest scores gets the
    // bonus.
    let mut build = mythic_build("trickster", 17);
    build.start.ability_scores = point_buy([14, 16, 12, 16, 10, 8]);
    build.path_switch = Some(path("gold-dragon"));
    let after_8 = engine().state_before(&build, &path_selection(8)).unwrap();
    assert_eq!(scores_of(&after_8), [14, 20, 14, 16, 14, 14]);
}

#[test]
fn a_fixed_bonus_raises_one_score() {
    let mut build = mythic_build("trickster", 17);
    build.path_switch = Some(path("swarm-that-walks"));
    let sheet = engine().evaluate(&build);
    // MR 8 and 9 both come at 17; Eye of the Swarm is MR 9's.
    assert_eq!(scores_of(sheet.at(16).unwrap()), [16, 14, 12, 13, 10, 8]);
    assert_eq!(scores_of(sheet.at(17).unwrap()), [20, 18, 12, 13, 10, 8]);
}

#[test]
fn a_dropped_feature_takes_its_bonus_with_it_and_level_increases_still_count() {
    // An ability increase after the switch adds to the base score, under the bonus.
    let mut build = switched(mythic_build("trickster", 20), "legend");
    build.levels[19].ability_increase = Some(Ability::Charisma);
    let at_20 = engine().evaluate(&build).at(20).cloned().unwrap();
    assert_eq!(at_20.base_ability_scores.charisma, 9);
    assert_eq!(at_20.ability_scores.charisma, 13);
}

// ---- Legend keeps the Trickster's Perception II feats -----------------------------------------------------------

/// A Trickster with Perception I at MR 3 and Perception II at MR 5 (level 14), switching to Legend at 17.
fn perception_legend() -> Build {
    let mut build = mythic_build("trickster", 20);
    build.picks.insert(
        path_selection(3),
        pick_option("trickster/tricks/perception-1"),
    );
    build.picks.insert(
        path_selection(5),
        pick_option("trickster/tricks/perception-2"),
    );
    switched(build, "legend")
}

#[test]
fn a_trickster_who_becomes_a_legend_keeps_the_feats_perception_ii_unlocked() {
    let mut build = perception_legend();
    build
        .picks
        .insert(general_slot(15), pick_feat("stat-focus"));
    let sheet = engine().evaluate(&build);
    assert!(engine().is_possible(&build));
    let at_20 = sheet.at(20).unwrap();
    // Perception II itself is gone with the Trickster's MR 3-7, the feat it unlocked stays.
    assert!(
        !at_20
            .selection_options
            .contains_key(&option("trickster/tricks/perception-2"))
    );
    assert!(at_20.feats.contains_key(&feat("stat-focus")));
}

#[test]
fn after_the_legend_switch_a_perception_ii_feat_cant_be_taken_again() {
    // "If you subsequently respec, you will not be able to take them again."
    let mut build = perception_legend();
    build
        .picks
        .insert(general_slot(19), pick_feat("stat-focus"));
    assert_eq!(
        why_not(&build, &general_slot(19)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::SelectionOption(option("trickster/tricks/perception-2")),
            has: None,
        }))
    );
}

// ---- The Mythic rank prerequisite -------------------------------------------------------------------------------

#[test]
fn a_mythic_rank_prerequisite_checks_the_rank_reached() {
    let build = mythic_build("aeon", 14);
    let status = |rank: u8, key: &str| {
        engine()
            .options(&build, &path_selection(rank))
            .unwrap()
            .into_iter()
            .find(|o| o.pick == pick_option(key))
            .unwrap()
            .status
    };
    // Area Effects needs MR 5: not at the MR 4 pick (level 11), yes at the MR 5 pick (level 14).
    assert_eq!(
        status(4, "aeon/enforcing-gaze/area-effects"),
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::MythicRank(5),
            has: Some(4),
        }))
    );
    assert_eq!(status(5, "aeon/enforcing-gaze/area-effects"), Ok(()));
}

#[test]
fn a_gold_dragon_breath_needing_mr_9_waits_for_the_mr_9_pick_though_both_come_at_17() {
    let mut build = mythic_build("trickster", 17);
    build.path_switch = Some(path("gold-dragon"));
    let breath = pick_option("gold-dragon/breaths/annihilating");
    let status = |rank: u8| {
        engine()
            .options(&build, &path_selection(rank))
            .unwrap()
            .into_iter()
            .find(|o| o.pick == breath)
            .unwrap()
            .status
    };
    assert!(status(8).is_err());
    assert_eq!(status(9), Ok(()));
}

#[test]
fn after_a_legend_switch_the_mythic_rank_is_3() {
    let build = switched(mythic_build("trickster", 20), "legend");
    let at_17 = engine().evaluate(&build).at(17).cloned().unwrap();
    assert!(at_17.check(&Prerequisite::MythicRank(3)).is_ok());
    assert!(at_17.check(&Prerequisite::MythicRank(4)).is_err());
}

// ---- Gold Dragon's Dragon Feats ---------------------------------------------------------------------------------

#[test]
fn gold_dragons_dragon_feats_are_three_picks_of_any_feat_ignoring_prerequisites() {
    let mut build = mythic_build("trickster", 17);
    build.path_switch = Some(path("gold-dragon"));
    let slot = |index| SlotId::PathFeat { rank: 9, index };
    // Great Cleave needs Cleave and BAB 4, and Crane Wing needs Crane Style: neither is held.
    build.picks.insert(slot(0), pick_feat("great-cleave"));
    build.picks.insert(slot(1), pick_feat("crane-wing"));
    // Cleaving Finish needs Great Cleave, held from the first pick on.
    build.picks.insert(slot(2), pick_feat("cleaving-finish"));
    let sheet = engine().evaluate(&build);
    assert!(engine().is_possible(&build));
    let at_17 = sheet.at(17).unwrap();
    for key in ["great-cleave", "crane-wing", "cleaving-finish"] {
        assert!(at_17.feats.contains_key(&feat(key)), "{key}");
    }

    // Every feat of the data is offered (a parameterised one once per value), and each one not held is open, but
    // for Cleaving Finish, which the third pick takes. (The second slot: nothing depends on Crane Wing.)
    let options = engine().options(&build, &slot(1)).unwrap();
    let offered: std::collections::BTreeSet<_> =
        options.iter().map(|option| option.pick.key()).collect();
    assert_eq!(offered.len(), world().feats.len());
    let held = engine().state_before(&build, &slot(1)).unwrap().feats;
    for option in options {
        let rules::Pick::Feat { feat: key, .. } = &option.pick else {
            panic!("{option:?}")
        };
        let expected = if held.contains_key(key) {
            Err(Issue::AlreadyHeld)
        } else if key == &feat("cleaving-finish") {
            Err(Issue::BreaksLater)
        } else {
            Ok(())
        };
        assert_eq!(option.status, expected, "{key}");
    }
    assert_eq!(
        sheet.ranks[8].slots[1].kind,
        SlotKind::Feat(FeatSlot::OneOf {
            list: feat_list("gold-dragon/dragon-feats"),
        })
    );
}

// ---- An option granting several feats ---------------------------------------------------------------------------

/// `levels` levels of one class on the Trickster path, with All Armour (two feats) as the MR 3 trick.
fn all_armour(class_key: &'static str, levels: usize) -> Build {
    let mut build = build("human", &times(class_key, levels));
    build.mythic_path = Some(path("trickster"));
    build.picks.insert(
        path_selection(3),
        pick_option("trickster/tricks/all-armour"),
    );
    build
}

#[test]
fn an_option_can_grant_several_feats() {
    let mut build = all_armour("rogue", 11);
    // The level-11 general slot picks a feat that needs one of them.
    build
        .picks
        .insert(general_slot(11), pick_feat("shield-focus"));
    let sheet = engine().evaluate(&build);
    assert!(engine().is_possible(&build));
    let at_8 = sheet.at(8).unwrap();
    assert!(!at_8.feats.contains_key(&feat("shield-proficiency")));
    let at_9 = sheet.at(9).unwrap();
    assert!(at_9.feats.contains_key(&feat("shield-proficiency")));
    assert!(at_9.feats.contains_key(&feat("tower-shield-proficiency")));
}

#[test]
fn a_switch_dropping_an_option_drops_its_feats_but_not_the_same_feats_held_otherwise() {
    let switched = |class_key| {
        let mut build = all_armour(class_key, 17);
        build.path_switch = Some(path("gold-dragon"));
        engine().evaluate(&build)
    };
    // A rogue has them only from the trick, which the Gold Dragon switch drops.
    let rogue = switched("rogue");
    assert!(
        rogue
            .at(16)
            .unwrap()
            .feats
            .contains_key(&feat("shield-proficiency"))
    );
    for key in ["shield-proficiency", "tower-shield-proficiency"] {
        assert!(
            !rogue.at(17).unwrap().feats.contains_key(&feat(key)),
            "{key}"
        );
    }
    // A fighter's class grants them at level 1 too: they stay.
    let fighter = switched("fighter");
    for key in ["shield-proficiency", "tower-shield-proficiency"] {
        assert!(
            fighter.at(17).unwrap().feats.contains_key(&feat(key)),
            "{key}"
        );
    }
}

// ---- Perfect Mind's floors ------------------------------------------------------------------------------------

#[test]
fn perfect_mind_raises_bab_and_every_skills_ranks_for_prerequisites() {
    let mut build = build("human", &times("wizard", 17));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("gold-dragon"));
    let sheet = engine().evaluate(&build);
    let bab_15 = Prerequisite::BaseAttackBonus(15);
    let mobility_9 = Prerequisite::SkillRanks {
        skill: SkillKey::new("mobility"),
        ranks: 9,
    };
    let at_16 = sheet.at(16).unwrap();
    assert!(at_16.check(&bab_15).is_err());
    assert!(at_16.check(&mobility_9).is_err());
    // MR 9 at 17: Perfect Mind.
    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.bab, 8);
    assert_eq!(at_17.prerequisite_bab(), 15);
    assert_eq!(at_17.check(&bab_15), Ok(()));
    assert_eq!(at_17.check(&mobility_9), Ok(()));
    assert!(
        at_17
            .check(&Prerequisite::SkillRanks {
                skill: SkillKey::new("mobility"),
                ranks: 10,
            })
            .is_err()
    );
}
