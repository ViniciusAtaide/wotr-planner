//! Mythic ascensions: the general pick at each rank, the path at MR 3, and the Path switch.

use rules::{
    Build, Edit, FeatSlot, FeatureSelectionKey, MythicAbilityKey, MythicFeatKey, MythicPickKind,
    RankStatus, Sheet, SlotId, SlotKind, mythic_pick_kind,
};

use crate::support::*;

#[test]
fn odd_ranks_give_a_mythic_ability_and_even_ranks_a_mythic_feat() {
    let kinds: Vec<_> = (1..=10).map(mythic_pick_kind).collect();
    use MythicPickKind::{Ability, Feat};
    assert_eq!(
        kinds,
        [
            Ability, Feat, Ability, Feat, Ability, Feat, Ability, Feat, Ability, Feat
        ]
    );
}

fn ranks_by_level(sheet: &Sheet) -> Vec<(u8, Vec<u8>)> {
    sheet
        .levels
        .iter()
        .filter(|level| !level.mythic_ranks.is_empty())
        .map(|level| (level.character_level, level.mythic_ranks.clone()))
        .collect()
}

fn mythic_slot_kinds(sheet: &Sheet, rank: u8) -> Vec<(SlotId, SlotKind)> {
    sheet.ranks[usize::from(rank) - 1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect()
}

/// A 20-level fighter on the Trickster path, with a general mythic pick at every rank.
fn trickster_build(levels: usize) -> Build {
    let mut build = build("human", &times("fighter", levels));
    build.mythic_path = Some(path("trickster"));
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
    // What the mythic feats need.
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    build.picks.insert(race_slot(0), pick_feat("toughness"));
    build.picks.insert(
        SlotId::PathSelection { rank: 3, index: 0 },
        pick_option("trickster/tricks/perception-1"),
    );
    build
}

#[test]
fn mythic_ascensions_happen_at_their_fixed_levels_with_two_ranks_at_17() {
    let sheet = engine().evaluate(&trickster_build(20));
    assert_eq!(
        ranks_by_level(&sheet),
        [
            (6, vec![1]),
            (7, vec![2]),
            (9, vec![3]),
            (11, vec![4]),
            (14, vec![5]),
            (15, vec![6]),
            (16, vec![7]),
            (17, vec![8, 9]),
            (19, vec![10]),
        ]
    );
    assert!(
        sheet
            .ranks
            .iter()
            .all(|rank| rank.status == RankStatus::Reached)
    );
    assert_eq!(sheet.at(5).unwrap().mythic_rank, 0);
    assert_eq!(sheet.at(17).unwrap().mythic_rank, 9);
    assert_eq!(sheet.at(20).unwrap().mythic_rank, 10);
    assert_eq!(sheet.at(20).unwrap().mythic_abilities.len(), 5);
    assert_eq!(sheet.at(20).unwrap().mythic_feats.len(), 5);
}

#[test]
fn the_general_mythic_pick_alternates_ability_and_feat() {
    let sheet = engine().evaluate(&trickster_build(7));
    assert_eq!(
        mythic_slot_kinds(&sheet, 1),
        [(SlotId::Mythic { rank: 1 }, SlotKind::MythicAbility)]
    );
    assert_eq!(
        mythic_slot_kinds(&sheet, 2),
        [(SlotId::Mythic { rank: 2 }, SlotKind::MythicFeat)]
    );
    // Ranks not reached yet are listed, with no slots.
    assert_eq!(sheet.ranks.len(), 10);
    assert_eq!(sheet.ranks[2].status, RankStatus::NotReached);
    assert_eq!(sheet.ranks[2].character_level, 9);
}

#[test]
fn the_path_is_chosen_at_mr_3_and_adds_its_own_picks() {
    let sheet = engine().evaluate(&trickster_build(11));
    assert_eq!(sheet.at(8).unwrap().mythic_path, None);
    assert_eq!(sheet.at(9).unwrap().mythic_path, Some(path("trickster")));
    assert_eq!(sheet.ranks[1].path, Some(path("trickster")));
    assert_eq!(sheet.ranks[2].path, Some(path("trickster")));
    assert_eq!(
        mythic_slot_kinds(&sheet, 3),
        [
            (SlotId::Mythic { rank: 3 }, SlotKind::MythicAbility),
            (
                SlotId::PathSelection { rank: 3, index: 0 },
                SlotKind::Selection(FeatureSelectionKey::new("trickster/tricks"))
            ),
        ]
    );
    assert_eq!(
        mythic_slot_kinds(&sheet, 4)[1].1,
        SlotKind::Feat(FeatSlot::OneOf {
            list: feat_list("trickster/bonus-feats"),
        })
    );
    let at_9 = sheet.at(9).unwrap();
    assert_eq!(
        at_9.feature_count(&feature("trickster/trickster-tricks")),
        1
    );
    assert!(
        at_9.selection_options
            .contains_key(&option("trickster/tricks/perception-1"))
    );
}

#[test]
fn a_legend_switch_at_mr_8_keeps_mr_1_and_2_and_stays_at_rank_3() {
    let mut build = trickster_build(40);
    // Fighter 20 then Rogue 20.
    for level in &mut build.levels[20..] {
        level.class = class("rogue");
    }
    // The switch drops the picks of MR 8 to 10, which it ends, and those of MR 3 to 7, which it takes away.
    engine().apply(&mut build, &Edit::PathSwitch(Some(path("legend"))));
    let ranks: std::collections::BTreeSet<u8> =
        build.picks.keys().filter_map(SlotId::rank).collect();
    assert_eq!(ranks, [1, 2].into());
    let sheet = engine().evaluate(&build);

    // Before the switch MR 7 is reached, with the Trickster's rows, but only MR 1-2's picks are held.
    let at_16 = sheet.at(16).unwrap();
    assert_eq!(at_16.mythic_rank, 7);
    assert_eq!(
        at_16.mythic_abilities,
        [MythicAbilityKey::new("mythic-charge")].into()
    );
    assert_eq!(at_16.mythic_path, Some(path("trickster")));
    assert_eq!(
        at_16.feature_count(&feature("trickster/trickster-tricks")),
        1
    );

    // From level 17: only the MR 1 ability and MR 2 feat remain, the rank is 3, and the Trickster's picks are gone.
    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.mythic_rank, 3);
    assert_eq!(at_17.mythic_path, Some(path("legend")));
    assert_eq!(
        at_17.mythic_abilities,
        [MythicAbilityKey::new("mythic-charge")].into()
    );
    assert_eq!(
        at_17.mythic_feats,
        [MythicFeatKey::new("extra-mythic-ability")].into()
    );
    assert_eq!(
        at_17.feature_count(&feature("trickster/trickster-tricks")),
        0
    );
    assert!(at_17.selection_options.is_empty());

    let statuses: Vec<_> = sheet.ranks.iter().map(|rank| rank.status).collect();
    use RankStatus::*;
    assert_eq!(
        statuses,
        [
            Reached, Reached, Dropped, Dropped, Dropped, Dropped, Dropped, NotReached, NotReached,
            NotReached
        ]
    );
    assert_eq!(ranks_by_level(&sheet).last(), Some(&(16, vec![7])));

    // Levels 18-40 carry on: feats at odd levels, an ability increase every 4 levels.
    assert_eq!(sheet.levels.len(), 40);
    assert_eq!(sheet.level_cap, 40);
    let general: Vec<_> = sheet.levels[17..]
        .iter()
        .filter(|level| {
            level
                .slots
                .iter()
                .any(|slot| slot.id == general_slot(level.character_level))
        })
        .map(|level| level.character_level)
        .collect();
    assert_eq!(general, (19..=39).step_by(2).collect::<Vec<_>>());
    let increases: Vec<_> = sheet.levels[17..]
        .iter()
        .filter(|level| level.ability_increase)
        .map(|level| level.character_level)
        .collect();
    assert_eq!(increases, [20, 24, 28, 32, 36, 40]);
    assert_eq!(sheet.at(40).unwrap().class_level(&class("rogue")), 20);
    assert!(engine().is_possible(&build));
}

#[test]
fn a_switch_that_keeps_ascending_takes_over_from_mr_8() {
    let mut build = trickster_build(20);
    engine().apply(&mut build, &Edit::PathSwitch(Some(path("gold-dragon"))));
    let sheet = engine().evaluate(&build);
    assert!(engine().is_possible(&build));

    let at_17 = sheet.at(17).unwrap();
    assert_eq!(at_17.mythic_path, Some(path("gold-dragon")));
    assert_eq!(at_17.mythic_rank, 9);
    assert_eq!(at_17.feature_count(&feature("gold-dragon/breath")), 1);
    // Gold Dragon keeps the general picks of MR 1-7 but drops the Trickster's own grants of MR 3-7.
    assert_eq!(at_17.mythic_abilities.len(), 5);
    assert_eq!(
        at_17.feature_count(&feature("trickster/trickster-tricks")),
        0
    );
    assert_eq!(sheet.at(20).unwrap().mythic_rank, 10);
    assert_eq!(sheet.ranks[7].path, Some(path("gold-dragon")));
    assert_eq!(sheet.level_cap, 20);
}
