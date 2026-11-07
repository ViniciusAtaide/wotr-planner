//! Options levelled by Mythic rank (ticket 89): an option held through a path's selection with
//! `level_from: MythicRanks` gets its row 1 at the rank it's picked, row 2 at the next, and so on (Azata's
//! Life-Bonding Friendship: a teamwork feat this rank and the next). The rows' feat slots are the rank's
//! [`SlotId::PathFeat`]s after the path row's own. The test world's Trickster tricks stand in for the superpowers,
//! with an invented Friendship trick.

use rules::{
    Alignment, Build, Choice, Edit, FeatCategory, FeatSlot, FeatureSelectionKey, Pick, RankStatus,
    SlotId, SlotKind,
};

use crate::support::*;

fn path_feat(rank: u8, index: u8) -> SlotId {
    SlotId::PathFeat { rank, index }
}

fn tricks(rank: u8) -> SlotId {
    SlotId::PathSelection { rank, index: 0 }
}

const TEAMWORK: SlotKind = SlotKind::Feat(FeatSlot::Category(FeatCategory::Teamwork));

/// A lawful good human fighter of `levels` levels on the Trickster path (tricks at MR 3, 5, 7 and 9; the bonus
/// feat at MR 4), with Friendship as the trick at `rank`.
fn friendship_at(rank: u8, levels: usize) -> Build {
    let mut build = build("human", &times("fighter", levels));
    build.start.alignment = Some(Alignment::LawfulGood);
    build.mythic_path = Some(path("trickster"));
    build
        .picks
        .insert(tricks(rank), pick_option("trickster/tricks/friendship"));
    build
}

/// A rank's slots, by id and kind.
fn rank_slots(build: &Build, rank: u8) -> Vec<(SlotId, SlotKind)> {
    engine().evaluate(build).ranks[usize::from(rank) - 1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect()
}

fn tricks_kind() -> SlotKind {
    SlotKind::Selection(FeatureSelectionKey::new("trickster/tricks"))
}

#[test]
fn an_option_levelled_by_mythic_rank_gives_its_rows_slots_at_the_rank_its_picked_and_the_next() {
    let build = friendship_at(3, 11);
    assert!(engine().is_possible(&build));
    assert_eq!(
        rank_slots(&build, 3),
        [
            (SlotId::Mythic { rank: 3 }, SlotKind::MythicAbility),
            (tricks(3), tricks_kind()),
            (path_feat(3, 0), TEAMWORK),
        ]
    );
    // After the path row's own feat slot.
    assert_eq!(
        rank_slots(&build, 4),
        [
            (SlotId::Mythic { rank: 4 }, SlotKind::MythicFeat),
            (
                path_feat(4, 0),
                SlotKind::Feat(FeatSlot::OneOf {
                    list: feat_list("trickster/bonus-feats")
                })
            ),
            (path_feat(4, 1), TEAMWORK),
        ]
    );
    let sheet = engine().evaluate(&build);
    assert_eq!(
        engine().slot_label(sheet.slot(&path_feat(3, 0)).unwrap()),
        "Teamwork feat"
    );
    // No level gets them as option slots.
    assert!(sheet.levels.iter().all(|level| {
        level
            .slots
            .iter()
            .all(|slot| !matches!(slot.id, SlotId::OptionFeat { .. }))
    }));

    // Another trick gives none.
    let mut other = build;
    other
        .picks
        .insert(tricks(3), pick_option("trickster/tricks/perception-1"));
    assert_eq!(rank_slots(&other, 3).len(), 2);
    assert_eq!(rank_slots(&other, 4).len(), 2);
}

#[test]
fn the_slot_offers_only_teamwork_feats_held_from_the_ranks_level() {
    let engine = engine();
    let build = friendship_at(3, 11);
    let offered: Vec<Pick> = engine
        .options(&build, &path_feat(3, 0))
        .unwrap()
        .into_iter()
        .map(|option| option.pick)
        .collect();
    assert_eq!(offered, [pick_feat("outflank")]);

    // MR 3 is at level 9, MR 4 at 11.
    let mut with_pick = build.clone();
    with_pick
        .picks
        .insert(path_feat(3, 0), pick_feat("outflank"));
    assert!(engine.is_possible(&with_pick));
    let sheet = engine.evaluate(&with_pick);
    assert!(!sheet.at(8).unwrap().feats.contains_key(&feat("outflank")));
    assert!(sheet.at(9).unwrap().feats.contains_key(&feat("outflank")));

    let mut next = build.clone();
    next.picks.insert(path_feat(4, 1), pick_feat("outflank"));
    assert!(engine.is_possible(&next));
    let sheet = engine.evaluate(&next);
    assert!(!sheet.at(10).unwrap().feats.contains_key(&feat("outflank")));
    assert!(sheet.at(11).unwrap().feats.contains_key(&feat("outflank")));

    // Not another feat; and prerequisites count (Outflank needs a lawful good character).
    let mut dodge = build.clone();
    dodge.picks.insert(path_feat(3, 0), pick_feat("dodge"));
    assert!(!engine.is_possible(&dodge));
    let mut neutral = with_pick;
    neutral.start.alignment = Some(Alignment::TrueNeutral);
    assert!(!engine.is_possible(&neutral));
}

#[test]
fn picked_at_mr_9_its_second_slot_comes_at_mr_10_and_not_before() {
    // MR 8 and 9 both come at level 17, MR 10 at 19; the Trickster has no row at 10.
    let at_17 = friendship_at(9, 17);
    assert!(engine().is_possible(&at_17));
    assert_eq!(
        rank_slots(&at_17, 9).last(),
        Some(&(path_feat(9, 0), TEAMWORK))
    );
    assert!(rank_slots(&at_17, 10).is_empty());

    let at_19 = friendship_at(9, 19);
    assert_eq!(
        rank_slots(&at_19, 10),
        [
            (SlotId::Mythic { rank: 10 }, SlotKind::MythicFeat),
            (path_feat(10, 0), TEAMWORK),
        ]
    );
    // A pick in a slot the Build doesn't reach is refused.
    let mut early = at_17;
    early.picks.insert(path_feat(10, 0), pick_feat("outflank"));
    assert!(!engine().is_possible(&early));
}

#[test]
fn the_option_isnt_removable_while_a_slot_of_its_rows_holds_a_pick_and_takes_its_slots_with_it() {
    let engine = engine();
    let mut build = friendship_at(3, 11);
    build.picks.insert(path_feat(4, 1), pick_feat("outflank"));
    assert!(engine.is_possible(&build));
    let trick = Choice::Slot(tricks(3));
    assert!(!engine.is_removable(&build, &trick));
    assert!(engine.is_removable(&build, &Choice::Slot(path_feat(4, 1))));

    build.apply(&Edit::Clear(path_feat(4, 1)));
    assert!(engine.is_removable(&build, &trick));
    build.apply(&Edit::Clear(tricks(3)));
    assert!(engine.is_possible(&build));
    let sheet = engine.evaluate(&build);
    assert_eq!(sheet.slot(&path_feat(3, 0)), None);
    assert_eq!(sheet.slot(&path_feat(4, 1)), None);
}

#[test]
fn a_switch_that_drops_the_paths_grants_of_those_ranks_drops_both_picks() {
    // Gold Dragon keeps the old path's grants of MR 1-2 only.
    for (rank, slot) in [(3u8, path_feat(3, 0)), (4, path_feat(4, 1))] {
        let mut build = friendship_at(3, 20);
        build.picks.insert(slot.clone(), pick_feat("outflank"));
        engine().apply(&mut build, &Edit::PathSwitch(Some(path("gold-dragon"))));
        assert!(engine().is_possible(&build), "{slot:?}");
        let sheet = engine().evaluate(&build);
        assert!(sheet.at(16).unwrap().feats.contains_key(&feat("outflank")));
        assert!(!sheet.at(17).unwrap().feats.contains_key(&feat("outflank")));
        let sheet_rank = &sheet.ranks[usize::from(rank) - 1];
        assert_eq!(sheet_rank.status, RankStatus::PartlyDropped);
        assert!(sheet_rank.dropped_slots.contains(&slot));
    }
}
