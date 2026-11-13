//! Mythic ranks taken later than the rank table's level (ticket 171). The table's level is the earliest a rank
//! comes at; a Build may place a rank at any later level it has, and ranks keep their order, so a rank placed later
//! takes the ranks above it along. A rank's picks are checked at the level it is taken at, after that level's own
//! picks, and what it gives is held from there. A Build that moves no rank is the Build it always was.

use std::collections::BTreeMap;

use rules::agreement::judge;
use rules::{Build, Choice, Edit, Issue, RankStatus, SlotId, decode_build, encode_build};

use crate::support::*;

/// A human fighter of `levels` levels on the Trickster's path.
fn trickster(levels: usize) -> Build {
    let mut build = build("human", &times("fighter", levels));
    if levels >= 9 {
        build.mythic_path = Some(path("trickster"));
    }
    build
}

/// The gold Enforcer's shape: Toughness at level 13 and Toughness (Mythic) at MR 4, whose table level is 11.
fn toughness_after_its_rank() -> Build {
    let mut build = trickster(13);
    build.picks.insert(general_slot(13), pick_feat("toughness"));
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat("toughness-mythic"),
    );
    build
}

/// The Build with `rank` taken at `level`, as the sheet's edit makes it.
fn moved(build: &Build, rank: u8, level: u8) -> Build {
    let mut build = build.clone();
    engine().apply(
        &mut build,
        &Edit::RankLevel {
            rank,
            level: Some(level),
        },
    );
    build
}

/// Each rank's character level on the sheet, in rank order.
fn levels_of(build: &Build) -> Vec<u8> {
    (engine().evaluate(build).ranks.iter())
        .map(|rank| rank.character_level)
        .collect()
}

const TABLE: [u8; 10] = [6, 7, 9, 11, 14, 15, 16, 17, 17, 19];

#[test]
fn a_build_that_moves_no_rank_takes_each_at_the_tables_level() {
    let build = trickster(20);
    assert!(build.rank_levels.is_empty());
    assert_eq!(levels_of(&build), TABLE);
    for (rank, level) in (1..).zip(TABLE) {
        assert_eq!(engine().rank_level(&build, rank), Some(level));
        assert_eq!(engine().rank_table_level(rank), Some(level));
        assert!(!engine().rank_is_moved(rank, level));
    }
    assert_eq!(engine().rank_level(&build, 11), None);
}

#[test]
fn a_mythic_pick_that_needs_a_later_levels_feat_opens_once_its_rank_is_taken_there() {
    let engine = engine();
    let build = toughness_after_its_rank();
    // At the table's level the mythic feat comes two levels before the feat it needs.
    assert!(!engine.is_possible(&build));
    assert!(matches!(
        why_not(&build, &SlotId::Mythic { rank: 4 }),
        Some(Issue::Unmet(_))
    ));

    let later = moved(&build, 4, 13);
    assert_eq!(later.rank_levels, BTreeMap::from([(4, 13)]));
    assert!(engine.is_possible(&later));
    let sheet = engine.evaluate(&later);
    assert_eq!(sheet.ranks[3].character_level, 13);
    assert_eq!(sheet.ranks[3].status, RankStatus::Reached);
    assert!(engine.rank_is_moved(4, 13));
    // The level shows the rank where it is taken, and the levels before hold nothing of it.
    assert!(sheet.levels[10].mythic_ranks.is_empty());
    assert_eq!(sheet.levels[12].mythic_ranks, [4]);
    assert_eq!(sheet.levels[11].state.mythic_rank, 3);
    assert_eq!(sheet.levels[12].state.mythic_rank, 4);
    // The rank's picks come after the level's own: the level's general slot can't lean on the rank's pick.
    let before = engine
        .state_before(&later, &SlotId::Mythic { rank: 4 })
        .unwrap();
    assert_eq!(before.feat_count(&feat("toughness")), 1);
    assert_eq!(
        engine
            .state_before(&later, &general_slot(13))
            .unwrap()
            .mythic_rank,
        3
    );
}

#[test]
fn a_rank_taken_later_takes_the_ranks_above_it_along() {
    let engine = engine();
    let build = trickster(20);
    let later = moved(&build, 3, 12);
    // MR 4 (table: 11) follows to 12; MR 5 (14) stays.
    assert_eq!(later.rank_levels, BTreeMap::from([(3, 12)]));
    assert_eq!(levels_of(&later), [6, 7, 12, 12, 14, 15, 16, 17, 17, 19]);
    assert!(engine.is_possible(&later));
    let sheet = engine.evaluate(&later);
    // Two ranks at one level come in rank order, and the path is chosen where MR 3 is taken.
    assert_eq!(sheet.levels[11].mythic_ranks, [3, 4]);
    assert_eq!(sheet.levels[10].state.mythic_path, None);
    assert_eq!(sheet.levels[11].state.mythic_path, Some(path("trickster")));
    assert_eq!(
        sheet.levels[10]
            .state
            .feature_count(&feature("trickster/trickster-tricks")),
        0
    );
    assert_eq!(
        sheet.levels[11]
            .state
            .feature_count(&feature("trickster/trickster-tricks")),
        1
    );

    // A rank placed for itself keeps its place while the rank below is no later, and follows it when it is.
    let both = moved(&later, 4, 13);
    assert_eq!(levels_of(&both)[..5], [6, 7, 12, 13, 14]);
    let pushed = moved(&both, 3, 15);
    assert_eq!(pushed.rank_levels, BTreeMap::from([(3, 15), (4, 13)]));
    assert_eq!(levels_of(&pushed), [6, 7, 15, 15, 15, 15, 16, 17, 17, 19]);
    // Back again: MR 4 is where it was placed, and the link is the one it was.
    let back = moved(&pushed, 3, 12);
    assert_eq!(back, both);
    assert_eq!(encode_build(&back), encode_build(&both));
}

#[test]
fn the_edit_keeps_only_a_level_later_than_the_tables_that_the_build_has() {
    let engine = engine();
    let build = trickster(13);
    // The table's level, or an earlier one, is no move.
    assert_eq!(moved(&build, 4, 11), build);
    assert_eq!(moved(&build, 4, 3), build);
    // Past the last level, or of a rank the data lacks.
    assert_eq!(moved(&build, 4, 14), build);
    assert_eq!(moved(&build, 11, 12), build);
    // A level past the last leaves the rank, its picks and the path chosen at it as they are.
    let mut filled = build.clone();
    filled.picks.insert(
        SlotId::PathSelection { rank: 3, index: 0 },
        pick_option("trickster/tricks/perception-1"),
    );
    assert!(engine.is_possible(&filled));
    assert_eq!(moved(&filled, 3, 99), filled);
    assert_eq!(moved(&filled, 3, 14), filled);
    // Taking it back as early as it comes.
    let later = moved(&build, 4, 13);
    let mut back = later.clone();
    engine.apply(
        &mut back,
        &Edit::RankLevel {
            rank: 4,
            level: None,
        },
    );
    assert_eq!(back, build);
    assert_eq!(encode_build(&back), encode_build(&build));
}

#[test]
fn a_hand_made_rank_level_that_is_none_fails_in_the_walk() {
    let engine = engine();
    let failure = |rank: u8, level: u8| {
        let mut build = trickster(13);
        build.rank_levels.insert(rank, level);
        assert!(!engine.is_possible(&build));
        assert_eq!(
            engine.open_link(&encode_build(&build)),
            Err(rules::StaleLink)
        );
        let failures = engine.failures(&build);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert_eq!(failures[0].choice, Choice::RankLevel { rank });
        failures[0].issue.clone()
    };
    assert_eq!(failure(4, 11), Issue::NotAllowed);
    assert_eq!(failure(4, 10), Issue::NotAllowed);
    assert_eq!(failure(4, 14), Issue::NotReached);
    assert_eq!(failure(11, 12), Issue::NotReached);
    // After a switch that ends the ascensions, a rank past it is never taken.
    let mut legend = trickster(20);
    legend.path_switch = Some(path("legend"));
    legend.rank_levels.insert(9, 18);
    assert_eq!(
        engine.failures(&legend),
        [rules::Failure {
            choice: Choice::RankLevel { rank: 9 },
            issue: Issue::NotReached
        }]
    );
}

#[test]
fn the_level_picker_lists_the_levels_from_the_earliest_to_the_last() {
    let engine = engine();
    let build = trickster(13);
    let options = engine.rank_level_options(&build, 4);
    let levels: Vec<u8> = options.iter().map(|option| option.level).collect();
    assert_eq!(levels, [11, 12, 13]);
    assert!(options[0].held && options[0].earliest);
    assert!(options.iter().all(|option| option.status.is_ok()));
    assert_eq!(
        options[0].edit,
        Edit::RankLevel {
            rank: 4,
            level: None
        }
    );
    assert_eq!(
        options[2].edit,
        Edit::RankLevel {
            rank: 4,
            level: Some(13)
        }
    );
    // MR 3's picker says which ranks a move takes along: MR 4 from level 12 on.
    let options = engine.rank_level_options(&build, 3);
    let moves: Vec<(u8, &[(u8, u8)])> = options
        .iter()
        .map(|option| (option.level, option.moves.as_slice()))
        .collect();
    assert_eq!(
        moves,
        [
            (9, &[][..]),
            (10, &[]),
            (11, &[]),
            (12, &[(4, 12)]),
            (13, &[(4, 13)])
        ]
    );
    // With MR 3 at 12, MR 4's earliest is 12: it can't come before the rank below.
    let later = moved(&build, 3, 12);
    let options = engine.rank_level_options(&later, 4);
    let levels: Vec<u8> = options.iter().map(|option| option.level).collect();
    assert_eq!(levels, [12, 13]);
    assert!(options[0].held && options[0].earliest);
    // Moving MR 3 back says MR 4 follows.
    let back = &engine.rank_level_options(&later, 3)[0];
    assert_eq!((back.level, back.moves.as_slice()), (9, &[(4, 11)][..]));

    // A rank the Build doesn't reach, or reaches only at its last level, has nowhere to go.
    assert!(engine.rank_level_options(&build, 5).is_empty());
    assert!(engine.rank_level_options(&build, 11).is_empty());
    assert_eq!(engine.rank_level_options(&trickster(11), 4).len(), 1);
    let sheet = engine.evaluate(&build);
    assert!(engine.rank_can_move(&build, 4, sheet.ranks[3].status));
    assert!(!engine.rank_can_move(&build, 5, sheet.ranks[4].status));
    let short = trickster(11);
    let status = engine.evaluate(&short).ranks[3].status;
    assert!(!engine.rank_can_move(&short, 4, status));
}

#[test]
fn moving_a_rank_back_before_a_feat_its_pick_needs_is_locked_and_says_what_needs_it() {
    let engine = engine();
    let later = moved(&toughness_after_its_rank(), 4, 13);
    let options = engine.rank_level_options(&later, 4);
    assert_eq!(options.len(), 3);
    for option in &options[..2] {
        assert_eq!(option.status, Err(Issue::BreaksLater));
        assert_eq!(option.needed_by, [Choice::Slot(SlotId::Mythic { rank: 4 })]);
        assert_eq!(
            option.reason,
            Some(format!(
                "toughness-mythic needs MR 4 later than level {}",
                option.level
            ))
        );
        assert!(!engine.allows(&later, &option.edit));
    }
    assert!(options[2].held && options[2].status.is_ok());
    assert_eq!(options[2].reason, None);
    // Its withdrawal is the move back to the earliest level, refused as the picker refuses it.
    let choice = Choice::RankLevel { rank: 4 };
    assert_eq!(
        engine.withdrawal(&later, &choice),
        Some((
            Edit::RankLevel {
                rank: 4,
                level: None
            },
            false
        ))
    );
    assert!(!engine.removable(&later).contains(&choice));
    // Once the mythic feat is cleared the rank comes back.
    let mut cleared = later.clone();
    engine.apply(&mut cleared, &Edit::Clear(SlotId::Mythic { rank: 4 }));
    assert!(engine.is_removable(&cleared, &choice));
    // A Build that moves no rank holds no such choice.
    assert_eq!(engine.withdrawal(&trickster(13), &choice), None);
}

/// A Legend at MR 8: +4 to every score from the switch, which Dodge (Dex 13) at level 19 needs with Dex 10.
fn legend_with_dodge() -> Build {
    let mut build = trickster(20);
    build.start.ability_scores = point_buy([16, 10, 12, 13, 10, 8]);
    build.path_switch = Some(path("legend"));
    build.picks.insert(general_slot(19), pick_feat("dodge"));
    build
}

#[test]
fn moving_a_rank_later_past_a_level_whose_pick_needs_what_it_gives_is_locked() {
    let engine = engine();
    let build = legend_with_dodge();
    assert!(engine.is_possible(&build));
    let options = engine.rank_level_options(&build, 8);
    let statuses: Vec<(u8, bool)> = options
        .iter()
        .map(|option| (option.level, option.status.is_ok()))
        .collect();
    // At 18 the switch still comes before level 19; at 19 it comes after the level's own picks.
    assert_eq!(statuses, [(17, true), (18, true), (19, false), (20, false)]);
    assert_eq!(options[2].needed_by, [Choice::Slot(general_slot(19))]);
    assert_eq!(
        options[2].reason.as_deref(),
        Some("dodge needs MR 8 earlier than level 19")
    );
    assert!(!engine.allows(&build, &options[3].edit));
    // The same from the rank below: MR 7 at 19 would take the switch along.
    let options = engine.rank_level_options(&build, 7);
    let at_19 = options.iter().find(|option| option.level == 19).unwrap();
    assert_eq!(at_19.status, Err(Issue::BreaksLater));
    assert_eq!(at_19.moves, [(8, 19)]);
    assert_eq!(
        at_19.reason.as_deref(),
        Some("dodge needs MR 7 earlier than level 19")
    );
}

/// A Legend of 22 levels: 20 of fighter (the class's cap) and two of rogue past the base level cap.
fn long_legend() -> Build {
    let mut build = build("human", &[times("fighter", 20), times("rogue", 2)].concat());
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    build
}

#[test]
fn a_legend_switch_moves_with_its_rank_and_the_ranks_it_ends_have_no_level() {
    let engine = engine();
    let build = long_legend();
    assert!(engine.is_possible(&build));
    // The switch can come as late as level 20: the levels past 20 need the cap it raises.
    let options = engine.rank_level_options(&build, 8);
    let open: Vec<u8> = options
        .iter()
        .filter(|option| option.status.is_ok())
        .map(|option| option.level)
        .collect();
    assert_eq!(open, [17, 18, 19, 20]);
    let at_21 = options.iter().find(|option| option.level == 21).unwrap();
    assert_eq!(
        at_21.needed_by.first(),
        Some(&Choice::CharacterLevel { level: 21 })
    );
    assert_eq!(
        at_21.reason.as_deref(),
        Some("Level 21 needs MR 8 earlier than level 21")
    );
    // A reason names the first three choices and counts the rest.
    let rank_1 = engine.rank_level_options(&build, 1);
    let at_22 = rank_1.iter().find(|option| option.level == 22).unwrap();
    assert_eq!(at_22.needed_by.len(), 2);
    assert_eq!(
        at_22.reason.as_deref(),
        Some("Level 21 and Level 22 need MR 1 earlier than level 22")
    );
    // The text export gives the switch's level, though a Legend never reaches MR 8 as a rank.
    let text = engine.build_sheet(&moved(&build, 8, 20), "https://x/");
    assert!(
        text.ends_with("\n\nMythic track\nMR 8 (level 20, taken later than level 17)\n"),
        "{text}"
    );
    let later = moved(&build, 8, 20);
    assert!(engine.is_possible(&later));
    let sheet = engine.evaluate(&later);
    assert_eq!(sheet.level_cap, 40);
    assert_eq!(sheet.levels[18].state.mythic_path, Some(path("trickster")));
    assert_eq!(sheet.levels[19].state.mythic_path, Some(path("legend")));
    assert_eq!(sheet.levels[18].state.ability_scores.strength, 16);
    assert_eq!(sheet.levels[19].state.ability_scores.strength, 16 + 4);
    // MR 9 and 10 never come, so they show the table's levels, wherever the switch is taken; MR 8 shows where.
    assert_eq!(levels_of(&later)[7..], [20, 17, 19]);
    assert_eq!(levels_of(&build)[7..], [17, 17, 19]);
    assert_eq!(sheet.ranks[8].status, RankStatus::NotReached);
    // MR 9 and 10 never come: no picker, and a switch to Legend drops a level they were placed at.
    assert!(engine.rank_level_options(&build, 9).is_empty());
    assert!(engine.rank_level_options(&build, 10).is_empty());
    let mut plain = moved(&trickster(20), 10, 20);
    assert_eq!(plain.rank_levels, BTreeMap::from([(10, 20)]));
    engine.apply(&mut plain, &Edit::PathSwitch(Some(path("legend"))));
    assert!(plain.rank_levels.is_empty());
}

#[test]
fn a_switch_at_a_moved_rank_keeps_and_drops_by_rank_as_at_the_tables_level() {
    let engine = engine();
    // The Devil from the Aeon, and the Gold Dragon and the Swarm from the Trickster, each with a path pick at MR 3
    // and general picks, with MR 8 at 18, 19 and 20.
    for (from, switch, pick) in [
        ("aeon", "devil", "aeon/enforcing-gaze/attack"),
        ("trickster", "gold-dragon", "trickster/tricks/perception-1"),
        (
            "trickster",
            "swarm-that-walks",
            "trickster/tricks/perception-1",
        ),
    ] {
        let mut build = build("human", &times("fighter", 20));
        build.mythic_path = Some(path(from));
        build.path_switch = Some(path(switch));
        build.picks.insert(
            SlotId::PathSelection { rank: 3, index: 0 },
            pick_option(pick),
        );
        build.picks.insert(
            SlotId::Mythic { rank: 5 },
            pick_mythic_ability("mythic-charge"),
        );
        build
            .picks
            .insert(SlotId::Mythic { rank: 6 }, pick_mythic_feat("dodge-mythic"));
        assert!(engine.is_possible(&build), "{switch}");
        let at_table = engine.evaluate(&build);
        for level in [18, 19, 20] {
            let later = moved(&build, 8, level);
            assert!(engine.is_possible(&later), "{switch} at {level}");
            let sheet = engine.evaluate(&later);
            assert_eq!(
                levels_of(&later),
                [6, 7, 9, 11, 14, 15, 16, level, level, level.max(19)]
            );
            // The old path holds until the switch's level, and what the switch drops and keeps is the same by rank.
            let index = usize::from(level) - 1;
            assert_eq!(sheet.levels[index - 1].state.mythic_path, Some(path(from)));
            assert_eq!(sheet.levels[index].state.mythic_path, Some(path(switch)));
            for (moved, table) in sheet.ranks.iter().zip(&at_table.ranks) {
                assert_eq!(moved.status, table.status, "{switch}: MR {}", moved.rank);
                assert_eq!(moved.dropped_slots, table.dropped_slots);
                assert_eq!(moved.path, table.path);
                assert_eq!(moved.slots, table.slots);
            }
            assert_eq!(
                sheet.levels[19].state, at_table.levels[19].state,
                "{switch} at {level}"
            );
        }
    }
}

#[test]
fn the_first_ascension_moves_with_its_ranks() {
    let engine = engine();
    let mut build = build("human", &times("fighter", 9));
    build.first_ascension = Some(path("trickster"));
    build.mythic_path = Some(path("aeon"));
    let later = moved(&build, 1, 8);
    assert!(engine.is_possible(&later));
    assert_eq!(levels_of(&later)[..3], [8, 8, 9]);
    let sheet = engine.evaluate(&later);
    let bit_of_fun = feature("trickster/bit-of-fun");
    assert_eq!(sheet.levels[6].state.feature_count(&bit_of_fun), 0);
    assert_eq!(sheet.levels[7].state.feature_count(&bit_of_fun), 1);
    assert_eq!(sheet.levels[7].mythic_ranks, [1, 2]);
    assert_eq!(sheet.ranks[0].path, Some(path("trickster")));
    // Moved to the last level, MR 1 and 2 take MR 3 and its path choice along.
    let last = moved(&build, 1, 9);
    assert!(engine.is_possible(&last));
    assert_eq!(engine.evaluate(&last).levels[8].mythic_ranks, [1, 2, 3]);
}

#[test]
fn removing_the_level_a_rank_was_moved_to_takes_its_picks_and_puts_it_back() {
    let engine = engine();
    let later = moved(&toughness_after_its_rank(), 4, 13);
    let last = Choice::CharacterLevel { level: 13 };
    assert_eq!(
        engine.withdrawal(&later, &last),
        Some((Edit::RemoveLevel, true))
    );
    let mut shorter = later.clone();
    engine.apply(&mut shorter, &Edit::RemoveLevel);
    assert!(shorter.rank_levels.is_empty());
    assert!(!shorter.picks.contains_key(&SlotId::Mythic { rank: 4 }));
    assert!(!shorter.picks.contains_key(&general_slot(13)));
    assert!(engine.is_possible(&shorter));
    // MR 4 is back at its table level, reached and empty.
    let sheet = engine.evaluate(&shorter);
    assert_eq!(sheet.ranks[3].character_level, 11);
    assert_eq!(sheet.ranks[3].status, RankStatus::Reached);
    assert!(sheet.ranks[3].slots.iter().all(|slot| slot.pick.is_none()));

    // A rank moved to the last level still comes once the level is removed, at its table level, so the path chosen
    // at it stays: only what has no place goes.
    let path_last = moved(&trickster(12), 3, 12);
    let mut shorter = path_last.clone();
    engine.apply(&mut shorter, &Edit::RemoveLevel);
    assert!(shorter.rank_levels.is_empty());
    assert_eq!(shorter.mythic_path, Some(path("trickster")));
    assert!(engine.is_possible(&shorter));
    assert_eq!(levels_of(&shorter)[2], 9);
    // Below its table level the rank is gone, and the path with it.
    for _ in 0..3 {
        engine.apply(&mut shorter, &Edit::RemoveLevel);
    }
    assert_eq!(shorter.levels.len(), 8);
    assert_eq!(shorter.mythic_path, None);
    // A level below a moved rank's is never the last, so it is never removed from under it.
    let mut twice = moved(&trickster(13), 4, 12);
    engine.apply(&mut twice, &Edit::RemoveLevel);
    assert_eq!(twice.rank_levels, BTreeMap::from([(4, 12)]));
    assert!(engine.is_possible(&twice));
}

#[test]
fn a_slot_a_ranks_pick_gives_at_its_level_locks_the_move_while_it_is_filled() {
    // A rogue takes Extra Rogue Talent through Extra Feat at MR 2: a rogue talent slot at the rank's level, 7.
    let engine = engine();
    let mut build = build("human", &times("rogue", 9));
    build
        .picks
        .insert(SlotId::Mythic { rank: 2 }, pick_mythic_feat("extra-feat"));
    build.picks.insert(
        SlotId::MythicExtra { rank: 2 },
        pick_feat("extra-rogue-talent"),
    );
    let talent = SlotId::FeatSelection { level: 7, index: 0 };
    build.picks.insert(
        talent.clone(),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    assert!(engine.is_possible(&build));
    let options = engine.rank_level_options(&build, 2);
    assert_eq!(options[1].level, 8);
    assert_eq!(options[1].status, Err(Issue::BreaksLater));
    assert_eq!(options[1].needed_by, [Choice::Slot(talent.clone())]);
    assert_eq!(
        options[1].reason.as_deref(),
        Some(
            "fast-stealth fills a slot MR 2's pick gives at level 7, which would move with the \
             rank: clear it first"
        )
    );
    // Cleared, the rank moves and the slot comes at its new level.
    let mut cleared = build.clone();
    engine.apply(&mut cleared, &Edit::Clear(talent));
    let later = moved(&cleared, 2, 8);
    assert!(engine.is_possible(&later));
    let moved_slot = SlotId::FeatSelection { level: 8, index: 0 };
    assert!(engine.evaluate(&later).slot(&moved_slot).is_some());
}

#[test]
fn a_build_with_moved_ranks_round_trips_through_its_link() {
    let engine = engine();
    let later = moved(&moved(&toughness_after_its_rank(), 4, 13), 1, 8);
    let code = encode_build(&later);
    assert_eq!(engine.open_link(&code), Ok(later.clone()));
    assert_eq!(
        decode_build(&code, &rules::LinkKeys::of_build(&later)),
        Ok(later)
    );
}

#[test]
fn every_picker_agrees_with_the_walk_on_builds_with_moved_ranks() {
    let engine = engine();
    let mut devil = build("human", &times("fighter", 20));
    devil.mythic_path = Some(path("aeon"));
    devil.path_switch = Some(path("devil"));
    let long_legend = long_legend();
    let builds = [
        moved(&toughness_after_its_rank(), 4, 13),
        moved(&moved(&trickster(20), 3, 12), 4, 13),
        legend_with_dodge(),
        moved(&legend_with_dodge(), 8, 18),
        moved(&long_legend, 8, 20),
        moved(&devil, 8, 19),
        moved(&trickster(20), 1, 20),
    ];
    for build in builds {
        assert!(engine.is_possible(&build));
        let agreement = judge(&engine, &build);
        assert_eq!(
            agreement.disagreements,
            Vec::new(),
            "{:?}",
            build.rank_levels
        );
    }
}

#[test]
fn a_ranks_slot_says_which_later_level_of_the_rank_opens_a_pick_it_refuses() {
    let engine = engine();
    let mut build = trickster(13);
    build.picks.insert(general_slot(13), pick_feat("toughness"));
    let sheet = engine.evaluate(&build);
    let slot = SlotId::Mythic { rank: 4 };
    let opens = |pick: &rules::Pick| engine.rank_level_that_opens(&build, &sheet, &slot, pick);
    // Toughness comes at level 13, so MR 4 taken there opens its mythic feat.
    assert_eq!(
        opens(&pick_mythic_feat("toughness-mythic")),
        Some((13, vec![]))
    );
    // No level of the Build holds Cleave; an open pick, and a level's slot, have no such level.
    assert_eq!(opens(&pick_mythic_feat("cleave-mythic")), None);
    assert_eq!(opens(&pick_mythic_feat("dodge-mythic")), None);
    assert_eq!(
        engine.rank_level_that_opens(&build, &sheet, &general_slot(13), &pick_feat("dodge")),
        None
    );
    // With the rank at 13 already, nothing later is needed.
    let later = moved(&build, 4, 13);
    let sheet = engine.evaluate(&later);
    assert_eq!(
        engine.rank_level_that_opens(&later, &sheet, &slot, &pick_mythic_feat("toughness-mythic")),
        None
    );
}

/// The reviewer's Build for the removal: Toughness at 13 and its mythic feat at MR 4, with MR 4 taken at 13 and then
/// MR 3 at 15, so MR 4 is held at 15 by the rank below while its own level is 13.
fn pushed_past_its_own_level() -> Build {
    let mut build = trickster(15);
    build.picks.insert(general_slot(13), pick_feat("toughness"));
    let build = moved(&build, 4, 13);
    let mut build = build;
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat("toughness-mythic"),
    );
    moved(&build, 3, 15)
}

#[test]
fn removing_the_last_level_keeps_the_picks_of_a_rank_that_still_comes_where_they_are_legal() {
    let engine = engine();
    let build = pushed_past_its_own_level();
    assert!(engine.is_possible(&build));
    assert_eq!(build.rank_levels, BTreeMap::from([(3, 15), (4, 13)]));
    assert_eq!(levels_of(&build)[2..5], [15, 15, 15]);
    let mut shorter = build.clone();
    engine.apply(&mut shorter, &Edit::RemoveLevel);
    // MR 3 comes back at 9 and MR 4 at 13, its own level, where Toughness (Mythic) is still legal: it stays.
    assert_eq!(shorter.rank_levels, BTreeMap::from([(4, 13)]));
    assert_eq!(levels_of(&shorter)[2..5], [9, 13, 14]);
    assert_eq!(
        shorter.picks.get(&SlotId::Mythic { rank: 4 }),
        Some(&pick_mythic_feat("toughness-mythic"))
    );
    assert_eq!(shorter.mythic_path, Some(path("trickster")));
    assert!(engine.is_possible(&shorter));
    assert_eq!(
        engine.withdrawal(&build, &Choice::CharacterLevel { level: 15 }),
        Some((Edit::RemoveLevel, true))
    );
    // Two levels more and Toughness goes with level 13: MR 4 comes back at 11, and only then loses the feat.
    engine.apply(&mut shorter, &Edit::RemoveLevel);
    assert!(shorter.picks.contains_key(&SlotId::Mythic { rank: 4 }));
    engine.apply(&mut shorter, &Edit::RemoveLevel);
    assert!(shorter.rank_levels.is_empty());
    assert!(!shorter.picks.contains_key(&SlotId::Mythic { rank: 4 }));
    assert_eq!(levels_of(&shorter)[3], 11);
    assert!(engine.is_possible(&shorter));
}

/// What is promised of a rank's own level while the rank below holds it later (a dormant level), and what is not.
#[test]
fn a_rank_keeps_its_own_level_under_a_later_rank_below_until_it_is_moved_itself() {
    let engine = engine();
    let base = trickster(20);
    let both = moved(&moved(&base, 4, 13), 3, 15);
    assert_eq!(both.rank_levels, BTreeMap::from([(3, 15), (4, 13)]));
    // The dormant level changes nothing the sheet shows, and is in the link all the same.
    let alone = moved(&base, 3, 15);
    assert_eq!(engine.evaluate(&both), engine.evaluate(&alone));
    assert_ne!(encode_build(&both), encode_build(&alone));
    // Promised: moving the lower rank back restores the Build it was moved from, and its link.
    assert_eq!(moved(&both, 3, 9), moved(&base, 4, 13));
    assert_eq!(
        encode_build(&moved(&both, 3, 9)),
        encode_build(&moved(&base, 4, 13))
    );
    // Not promised: moving the upper rank itself forgets its dormant level. MR 4 to 16 and back to 15, the earliest
    // it can come, leaves MR 3's level alone.
    let up = moved(&both, 4, 16);
    assert_eq!(up.rank_levels, BTreeMap::from([(3, 15), (4, 16)]));
    let back = engine
        .rank_level_options(&up, 4)
        .into_iter()
        .find(|option| option.level == 15)
        .unwrap();
    assert!(back.earliest);
    let mut down = up.clone();
    engine.apply(&mut down, &back.edit);
    assert_eq!(down.rank_levels, BTreeMap::from([(3, 15)]));
    // Nor is a rank that was placed where the rank below then came taken back with it: {3: 15, 4: 15} with MR 3
    // back at 9 leaves MR 4 at 15.
    let level = moved(&moved(&base, 4, 15), 3, 15);
    assert_eq!(level.rank_levels, BTreeMap::from([(3, 15), (4, 15)]));
    let lower_back = moved(&level, 3, 9);
    assert_eq!(lower_back.rank_levels, BTreeMap::from([(4, 15)]));
    assert_eq!(levels_of(&lower_back)[2..5], [9, 15, 15]);
}

/// A rogue 20 on the Trickster's path with a Gold Dragon switch: MR 4's Extra Feat takes `extra` (a feat that gives
/// a slot at the rank's level, left empty), and MR 9's first Dragon Feat takes Extra Rogue Talent, whose talent slot
/// at level 17 holds Fast Stealth.
fn two_ranks_giving_slots(classes: &[&'static str], extra: &str) -> Build {
    let mut build = build("human", classes);
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("gold-dragon"));
    build
        .picks
        .insert(SlotId::Mythic { rank: 4 }, pick_mythic_feat("extra-feat"));
    build
        .picks
        .insert(SlotId::MythicExtra { rank: 4 }, pick_feat(extra));
    build.picks.insert(
        SlotId::PathFeat { rank: 9, index: 0 },
        pick_feat("extra-rogue-talent"),
    );
    build.picks.insert(
        SlotId::FeatSelection {
            level: 17,
            index: 0,
        },
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    build
}

#[test]
fn a_move_that_would_give_a_filled_slot_to_another_ranks_pick_is_locked_and_says_so() {
    let engine = engine();
    let talent = SlotId::FeatSelection {
        level: 17,
        index: 0,
    };
    // Both ranks give a rogue talent slot: at level 17, MR 4's would come first and take Fast Stealth's place,
    // though the walk of the moved Build fails nothing.
    let build = two_ranks_giving_slots(&times("rogue", 20), "extra-rogue-talent");
    assert!(engine.is_possible(&build));
    let edit = Edit::RankLevel {
        rank: 4,
        level: Some(17),
    };
    let mut edited = build.clone();
    engine.apply(&mut edited, &edit);
    assert!(engine.is_possible(&edited));
    assert_eq!(
        engine.shifted_picks(&build, &edit),
        [rules::ShiftedPick {
            slot: talent.clone(),
            from: 9,
            to: Some(4)
        }]
    );
    assert!(!engine.allows(&build, &edit));
    let options = engine.rank_level_options(&build, 4);
    let at_17 = options.iter().find(|option| option.level == 17).unwrap();
    assert_eq!(at_17.status, Err(Issue::BreaksLater));
    assert_eq!(at_17.needed_by, [Choice::Slot(talent.clone())]);
    assert_eq!(
        at_17.reason.as_deref(),
        Some(
            "fast-stealth fills a slot MR 9's pick gives at level 17, which the move would \
             give to MR 4's: clear it first"
        )
    );
    // Before level 17 nothing shifts, and with the talent cleared the move is open.
    let at_16 = options.iter().find(|option| option.level == 16).unwrap();
    assert!(at_16.status.is_ok());
    let mut cleared = build.clone();
    engine.apply(&mut cleared, &Edit::Clear(talent.clone()));
    assert!(engine.allows(&cleared, &edit));
    assert_eq!(judge(&engine, &build).disagreements, Vec::new());

    // The two slots take different kinds (a hex, a rogue talent): the walk refuses Fast Stealth in the hex slot,
    // and the reason is the shift, not that the talent needs the rank earlier.
    let classes = [vec!["witch"], times("rogue", 19)].concat();
    let build = two_ranks_giving_slots(&classes, "extra-hex");
    assert!(engine.is_possible(&build));
    let options = engine.rank_level_options(&build, 4);
    let at_17 = options.iter().find(|option| option.level == 17).unwrap();
    assert_eq!(at_17.status, Err(Issue::BreaksLater));
    assert_eq!(at_17.needed_by, [Choice::Slot(talent)]);
    assert_eq!(
        at_17.reason.as_deref(),
        Some(
            "fast-stealth fills a slot MR 9's pick gives at level 17, which the move would \
             give to MR 4's: clear it first"
        )
    );
    assert_eq!(judge(&engine, &build).disagreements, Vec::new());
}

#[test]
fn the_level_that_opens_a_ranks_pick_comes_with_the_ranks_the_move_takes_along() {
    let engine = engine();
    let mut build = trickster(13);
    build.picks.insert(general_slot(13), pick_feat("toughness"));
    let sheet = engine.evaluate(&build);
    let pick = pick_mythic_feat("toughness-mythic");
    // MR 4 alone moves; MR 2 at 13 would take MR 3 and 4 along.
    assert_eq!(
        engine.rank_level_that_opens(&build, &sheet, &SlotId::Mythic { rank: 4 }, &pick),
        Some((13, vec![]))
    );
    assert_eq!(
        engine.rank_level_that_opens(&build, &sheet, &SlotId::Mythic { rank: 2 }, &pick),
        Some((13, vec![(3, 13), (4, 13)]))
    );
    assert_eq!(
        rules::Engine::moves_text(&[(3, 13), (4, 13)]),
        "MR 3 to level 13, MR 4 to level 13"
    );
}

/// A Trickster fighter of 13 levels with Dodge in MR 4's bonus feat slot (MR 4 comes at level 11) and Crane Style,
/// which needs Dodge, in level 13's general slot.
fn crane_style_after_its_prerequisite() -> Build {
    let mut build = trickster(13);
    build
        .picks
        .insert(SlotId::PathFeat { rank: 4, index: 0 }, pick_feat("dodge"));
    build
        .picks
        .insert(general_slot(13), pick_feat("crane-style"));
    build
}

#[test]
fn a_move_that_refuses_a_pick_for_a_rank_it_takes_along_names_that_rank_and_its_pick() {
    // Ticket 192: MR 3 at level 13 takes MR 4 there, after Crane Style, which needs MR 4's Dodge.
    let engine = engine();
    let build = crane_style_after_its_prerequisite();
    assert!(engine.is_possible(&build));
    let options = engine.rank_level_options(&build, 3);
    let at_13 = options.iter().find(|option| option.level == 13).unwrap();
    assert_eq!(at_13.moves, [(4, 13)]);
    assert_eq!(at_13.status, Err(Issue::BreaksLater));
    assert_eq!(
        at_13.reason.as_deref(),
        Some("crane-style needs MR 4's dodge earlier than level 13: MR 4 moves with MR 3")
    );
    // MR 4 itself is the rank that holds Dodge: the text names it as it always has.
    let own = engine.rank_level_options(&build, 4);
    let own_13 = own.iter().find(|option| option.level == 13).unwrap();
    assert_eq!(
        own_13.reason.as_deref(),
        Some("crane-style needs MR 4 earlier than level 13")
    );
}

#[test]
fn a_pick_whose_prerequisite_is_a_later_ranks_says_where_and_what_would_open_it() {
    // Ticket 192: MR 4 is taken at level 14 with Dodge, so Crane Style at level 13 is refused.
    let engine = engine();
    let mut build = trickster(14);
    build
        .picks
        .insert(SlotId::PathFeat { rank: 4, index: 0 }, pick_feat("dodge"));
    build = moved(&build, 4, 14);
    let sheet = engine.evaluate(&build);
    let slot = general_slot(13);
    let pick = pick_feat("crane-style");
    let issue = engine
        .options(&build, &slot)
        .expect("the slot")
        .into_iter()
        .find(|option| option.pick == pick)
        .expect("offered")
        .status
        .unwrap_err();
    let later = engine
        .later_rank_pick(&build, &sheet, &slot, &issue)
        .expect("Dodge is in a later rank");
    assert_eq!(
        (later.rank, later.level, later.at, &later.pick),
        (4, 14, 13, &pick_feat("dodge"))
    );
    assert_eq!(
        engine.later_rank_text(&later),
        "dodge is in Mythic rank 4's trickster/bonus-feats slot, taken at level 14, after this slot's level 13"
    );
    // MR 4 at level 12 (the latest below 13) opens it; no rank above moves.
    assert_eq!(
        engine.rank_level_that_lets_in(&build, &slot, &pick, &later),
        Some((12, vec![]))
    );
    // An issue that isn't an unmet prerequisite, and a slot after the rank, have none.
    assert_eq!(
        engine.later_rank_pick(&build, &sheet, &slot, &Issue::AlreadyHeld),
        None
    );
    assert_eq!(
        engine.later_rank_pick(&build, &sheet, &general_slot(14), &issue),
        None
    );
}
