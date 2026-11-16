//! Life-Bonding Friendship's teamwork feats (ticket 89): "You choose a teamwork feat this rank and the next rank"
//! (azata#life-bonding-friendship). The Azata's superpowers are levelled by Mythic rank, and the superpower's rows
//! 1 and 2 each give a Teamwork feat slot: the rank's path feat slot, at the rank it's taken and the next. Taken
//! at MR 10, the last rank, it gives the one slot.

use std::collections::BTreeSet;

use rules::{
    Build, Choice, Edit, FeatCategory, FeatureSelectionKey, LevelFrom, MythicPathKey, SlotId,
    SlotKind,
};

use crate::support::*;

const LBF: &str = "azata/superpowers/life-bonding-friendship";

/// The teamwork feat slot (ticket 301: the teamwork feats' selection).
fn teamwork_slot() -> SlotKind {
    feat_kind("teamwork-feats")
}

fn path_feat(rank: u8) -> SlotId {
    SlotId::PathFeat { rank, index: 0 }
}

/// `levels` levels of `class` on the Azata path, with Life-Bonding Friendship as the superpower at `rank`.
fn azata(class: &str, levels: usize, rank: u8) -> Build {
    let mut build = build(class, levels);
    build.mythic_path = Some(MythicPathKey::new("azata"));
    build.picks.insert(path_selection(rank), pick(LBF));
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

fn superpowers() -> SlotKind {
    selection_kind("azata/superpowers")
}

#[test]
fn life_bonding_friendship_gives_a_teamwork_feat_slot_at_its_rows_1_and_2_and_no_other_superpower_has_rows()
 {
    let selection = DATA
        .selection(&rules::FeatureSelectionKey::new("azata/superpowers"))
        .expect("the superpowers");
    assert_eq!(selection.level_from, Some(LevelFrom::MythicRanks));
    assert!(selection.option_levels.is_empty());
    let with_rows: Vec<&str> = DATA
        .options_of(selection)
        .filter(|option| !option.levels.is_empty())
        .map(|option| option.key.as_str())
        .collect();
    assert_eq!(with_rows, [LBF]);
    let rows: Vec<(u8, Vec<FeatureSelectionKey>)> = selection_option(LBF)
        .levels
        .iter()
        .map(|row| (row.level, row.selections.clone()))
        .collect();
    let teamwork = || vec![FeatureSelectionKey::new("teamwork-feats")];
    // Game: AzataSuperpowersSelection 8a30e92cd04ff5b459ba7cb03584fda0's option LifeBondingFriendshipSelection
    // cfad18f581584ac4ba066df067956477 is itself a teamwork selection granting LifeBondingFriendshipFeature
    // 5b63a6690ebbaa4459df3e04fe094cbd; LifeBondingFriendshipSelection1 69a33d6ced23446e819667149d088898, at each
    // Azata level of AzataProgression 9db53de4bf21b564ca1a90ff5bd16586, needs that feature and not its own, so it
    // gives one more teamwork feat, once; no other superpower has more (equivalent model: rows 1 and 2).
    assert_eq!(rows, [(1, teamwork()), (2, teamwork())]);
    // The rows give nothing else (their one selection is the teamwork feat's, above).
    for row in &selection_option(LBF).levels {
        assert!(row.features.is_empty() && row.feats.is_empty());
    }
    // No other path selection is levelled by Mythic rank.
    let by_rank: Vec<&str> = DATA
        .selections
        .iter()
        .filter(|selection| selection.level_from == Some(LevelFrom::MythicRanks))
        .map(|selection| selection.key.as_str())
        .collect();
    assert_eq!(by_rank, ["azata/superpowers"]);
}

#[test]
fn taken_at_mr_6_it_gives_a_teamwork_feat_slot_at_mr_6_and_mr_7() {
    // MR 6 at level 15, MR 7 at 16.
    let build = azata("sorcerer", 16, 6);
    assert!(possible(&build));
    // Game: the first pick at the superpower's own Azata level, the second (LifeBondingFriendshipSelection1
    // 69a33d6ced23446e819667149d088898) at the next, its prerequisite checked before that level's picks
    // (UnitLogic/Class/LevelUp/Actions/LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b).
    assert_eq!(
        rank_slots(&build, 6),
        [
            (SlotId::Mythic { rank: 6 }, SlotKind::MythicFeat),
            (path_selection(6), superpowers()),
            (path_feat(6), teamwork_slot()),
        ]
    );
    assert_eq!(
        rank_slots(&build, 7),
        [
            (SlotId::Mythic { rank: 7 }, SlotKind::MythicAbility),
            (path_feat(7), teamwork_slot()),
        ]
    );
    let sheet = engine().evaluate(&build);
    assert_eq!(
        engine().slot_label(sheet.slot(&path_feat(6)).unwrap()),
        "Teamwork feat"
    );

    // Another superpower gives none.
    let mut other = build;
    other
        .picks
        .insert(path_selection(6), pick("azata/superpowers/zippy-magic"));
    assert_eq!(rank_slots(&other, 6).len(), 2);
    assert_eq!(rank_slots(&other, 7).len(), 1);
}

#[test]
fn the_slots_offer_the_teamwork_feats_and_no_other_feat() {
    let teamwork: BTreeSet<String> = DATA
        .feats
        .iter()
        .filter(|feat| feat.category == FeatCategory::Teamwork)
        .map(|feat| feat.key.to_string())
        .collect();
    assert_eq!(teamwork.len(), 13);
    let sorcerer = azata("sorcerer", 16, 6);
    // Which teamwork feats: `the_slots_offer_every_teamwork_feat_but_spell_chain`.
    // Game: LifeBondingFriendshipSelection cfad18f581584ac4ba066df067956477 and LifeBondingFriendshipSelection1
    // 69a33d6ced23446e819667149d088898 list only teamwork feats.
    // Every teamwork feat but Spell Chain, whose place waits in that test.
    let but_spell_chain: BTreeSet<String> = teamwork
        .iter()
        .filter(|feat| *feat != "spell-chain")
        .cloned()
        .collect();
    for rank in [6, 7] {
        let mut offered = feats_offered(&sorcerer, &path_feat(rank));
        offered.remove("spell-chain");
        assert_eq!(offered, but_spell_chain, "MR {rank}");
    }
    // The ticket 88 write-ups' picks: Shake It Off and Allied Spellcaster open; Shield Wall for a sorcerer:
    // `the_slots_ignore_prerequisites`.
    // Game: ShakeItOff 6337b37f2a7c11b4ab0831d6780bce2a and AlliedSpellcaster 9093ceeefe9b84746a5993d619d7c86f are
    // in both lists.
    assert_eq!(
        feat_status(&sorcerer, &path_feat(6), "shake-it-off"),
        Ok(())
    );
    assert_eq!(
        feat_status(&sorcerer, &path_feat(7), "allied-spellcaster"),
        Ok(())
    );
    // Game: ShieldWall 8976de442862f82488a4b138a0a89907 is in both lists.
    let fighter = azata("fighter", 16, 6);
    assert_eq!(feat_status(&fighter, &path_feat(6), "shield-wall"), Ok(()));

    // Jack-in-the-Green's: Shake It Off at MR 6 (level 15), Allied Spellcaster at MR 7 (level 16).
    let mut jack = sorcerer.clone();
    jack.picks
        .insert(path_feat(6), feat_pick_plain("shake-it-off"));
    jack.picks
        .insert(path_feat(7), feat_pick_plain("allied-spellcaster"));
    assert!(possible(&jack));
    let sheet = engine().evaluate(&jack);
    let holds = |level: u8, key: &str| {
        sheet
            .at(level)
            .unwrap()
            .feats
            .contains_key(&rules::FeatKey::new(key))
    };
    assert!(!holds(14, "shake-it-off") && holds(15, "shake-it-off"));
    assert!(!holds(15, "allied-spellcaster") && holds(16, "allied-spellcaster"));
    // The Enforcer's (a fighter here): Shield Wall at MR 6, Shake It Off at MR 7.
    let mut enforcer = fighter;
    enforcer
        .picks
        .insert(path_feat(6), feat_pick_plain("shield-wall"));
    enforcer
        .picks
        .insert(path_feat(7), feat_pick_plain("shake-it-off"));
    assert!(possible(&enforcer));

    // Not a non-teamwork feat.
    // Game: Dodge 97e216dbb46ae3c4faef90cf6bbe6fd5 is in neither list.
    let mut dodge = sorcerer;
    dodge.picks.insert(path_feat(6), feat_pick_plain("dodge"));
    assert!(!possible(&dodge));
}

/// Both slots offer the teamwork feats but Spell Chain.
#[test]
#[ignore = "waits on ticket 266: the game gives Life-Bonding Friendship 12 teamwork feats, no Spell Chain"]
fn the_slots_offer_every_teamwork_feat_but_spell_chain() {
    let mut teamwork: BTreeSet<String> = DATA
        .feats
        .iter()
        .filter(|feat| feat.category == FeatCategory::Teamwork)
        .map(|feat| feat.key.to_string())
        .collect();
    // Game: LifeBondingFriendshipSelection cfad18f581584ac4ba066df067956477 and LifeBondingFriendshipSelection1
    // 69a33d6ced23446e819667149d088898 list twelve teamwork feats; SpellChain d8d641edc9a74a64809f56feff9495b6 isn't
    // one of them.
    assert!(teamwork.remove("spell-chain"));
    let sorcerer = azata("sorcerer", 16, 6);
    for rank in [6, 7] {
        assert_eq!(
            feats_offered(&sorcerer, &path_feat(rank)),
            teamwork,
            "MR {rank}"
        );
    }
}

/// Both slots ignore the feats' prerequisites: a sorcerer takes Shield Wall without Shield Proficiency.
#[test]
#[ignore = "waits on ticket 266: the game gives a sorcerer Shield Wall there"]
fn the_slots_ignore_prerequisites() {
    let sorcerer = azata("sorcerer", 16, 6);
    // Game: LifeBondingFriendshipSelection cfad18f581584ac4ba066df067956477 and LifeBondingFriendshipSelection1
    // 69a33d6ced23446e819667149d088898 have IgnorePrerequisites (Blueprints/Classes/Selection/
    // BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b).
    assert_eq!(feat_status(&sorcerer, &path_feat(6), "shield-wall"), Ok(()));
    let mut shield_wall = sorcerer;
    shield_wall
        .picks
        .insert(path_feat(6), feat_pick_plain("shield-wall"));
    assert!(possible(&shield_wall));
}

#[test]
fn taken_at_mr_4_or_8_the_second_slot_comes_at_the_next_rank() {
    // MR 4 at level 11, MR 5 at 14; MR 8 and 9 both at 17.
    // Game: LifeBondingFriendshipSelection1 69a33d6ced23446e819667149d088898 at the next Azata level, its
    // prerequisite checked before that level's picks (UnitLogic/Class/LevelUp/Actions/
    // LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b).
    for (rank, levels) in [(4, 14), (8, 17)] {
        let build = azata("fighter", levels, rank);
        assert!(possible(&build), "MR {rank}");
        assert_eq!(
            rank_slots(&build, rank).last(),
            Some(&(path_feat(rank), teamwork_slot()))
        );
        assert_eq!(
            rank_slots(&build, rank + 1).last(),
            Some(&(path_feat(rank + 1), teamwork_slot())),
            "MR {rank}"
        );
    }
}

/// The ticket's decision: taken at MR 10, the last rank, it gives one slot; the second has no rank to land on.
#[test]
fn taken_at_mr_10_it_gives_one_slot() {
    let build = azata("fighter", 20, 10);
    assert!(possible(&build));
    // Game: AzataProgression 9db53de4bf21b564ca1a90ff5bd16586's last level (MR 10) holds both the superpower
    // selection and LifeBondingFriendshipSelection1 69a33d6ced23446e819667149d088898; the latter is checked before
    // the superpower is picked (UnitLogic/Class/LevelUp/Actions/LevelUpHelper.AddFeaturesFromProgression,
    // WrathDecompiled 6ce4a6b), so one teamwork feat.
    assert_eq!(
        rank_slots(&build, 10),
        [
            (SlotId::Mythic { rank: 10 }, SlotKind::MythicFeat),
            (path_selection(10), superpowers()),
            (path_feat(10), teamwork_slot()),
        ]
    );
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.ranks.len(), 10);
    let teamwork_slots = sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
        .filter(|slot| slot.kind == teamwork_slot())
        .count();
    assert_eq!(teamwork_slots, 1);
}

#[test]
fn life_bonding_friendship_isnt_removable_while_a_slot_of_it_holds_a_pick_and_takes_both_slots_with_it()
 {
    let engine = engine();
    let mut build = azata("sorcerer", 16, 6);
    build
        .picks
        .insert(path_feat(7), feat_pick_plain("shake-it-off"));
    assert!(possible(&build));
    let superpower = Choice::Slot(path_selection(6));
    assert!(!engine.is_removable(&build, &superpower));
    assert!(engine.is_removable(&build, &Choice::Slot(path_feat(7))));

    build.apply(&Edit::Clear(path_feat(7)));
    assert!(engine.is_removable(&build, &superpower));
    build.apply(&Edit::Clear(path_selection(6)));
    assert!(possible(&build));
    assert_eq!(kind_at(&build, &path_feat(6)), None);
    assert_eq!(kind_at(&build, &path_feat(7)), None);
}
