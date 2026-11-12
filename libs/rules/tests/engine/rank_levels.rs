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
    let mut build = build("human", &times("fighter", 20));
    build.mythic_path = Some(path("aeon"));
    build.path_switch = Some(path("devil"));
    build.picks.insert(
        SlotId::PathSelection { rank: 3, index: 0 },
        pick_option("aeon/enforcing-gaze/attack"),
    );
    assert!(engine.is_possible(&build));
    let at_table = engine.evaluate(&build);
    let later = moved(&build, 8, 19);
    assert!(engine.is_possible(&later));
    let sheet = engine.evaluate(&later);
    assert_eq!(levels_of(&later), [6, 7, 9, 11, 14, 15, 16, 19, 19, 19]);
    // The Aeon holds until the switch's level, and what the Devil drops and keeps is the same by rank.
    assert_eq!(sheet.levels[17].state.mythic_path, Some(path("aeon")));
    assert_eq!(sheet.levels[18].state.mythic_path, Some(path("devil")));
    assert_eq!(sheet.levels[18].mythic_ranks, [8, 9, 10]);
    for (moved, table) in sheet.ranks.iter().zip(&at_table.ranks) {
        assert_eq!(moved.status, table.status, "MR {}", moved.rank);
        assert_eq!(moved.dropped_slots, table.dropped_slots);
        assert_eq!(moved.path, table.path);
    }
    assert_eq!(sheet.levels[19].state, at_table.levels[19].state);
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

    // The path chosen at a rank moved to the last level goes with the level, as it does at the table's level.
    let path_last = moved(&trickster(12), 3, 12);
    let mut shorter = path_last.clone();
    engine.apply(&mut shorter, &Edit::RemoveLevel);
    assert!(shorter.rank_levels.is_empty());
    assert_eq!(shorter.mythic_path, None);
    assert!(engine.is_possible(&shorter));
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
    assert_eq!(opens(&pick_mythic_feat("toughness-mythic")), Some(13));
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
