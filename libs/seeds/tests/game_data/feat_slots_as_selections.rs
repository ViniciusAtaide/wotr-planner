//! Feat slots and feat lists as selections (ticket 301): every feat slot is a row's selection that lists its feats, as
//! the game's `BlueprintFeatureSelection`s do, and a feat list is such a selection.

use std::collections::BTreeSet;
use std::path::Path;

use rules::{BASIC_FEAT_SELECTION, FeatKey, FeatSource, FeatureSelectionKey, SlotId, SlotKind};

use crate::support::*;

fn feats(keys: &[&str]) -> BTreeSet<FeatKey> {
    keys.iter().map(|key| FeatKey::new(*key)).collect()
}

/// Every `.ron` file under `dir`, recursively.
fn ron_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("a readable folder") {
        let path = entry.expect("a folder entry").path();
        if path.is_dir() {
            ron_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "ron") {
            out.push(path);
        }
    }
}

/// The data has no feat lists and no feat slots left: each is a selection a row names.
#[test]
fn no_feat_lists_or_feat_slots_remain() {
    let dir = seeds::game_data_dir();
    assert!(!dir.join("feat-lists").exists());
    let mut files = Vec::new();
    ron_files(&dir, &mut files);
    let naming: Vec<_> = files
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .expect("a readable file")
                .contains("feat_slots")
        })
        .collect();
    assert!(naming.is_empty(), "{naming:?}");
}

/// The general feat at odd levels and the human's bonus feat are the game's `BasicFeatSelection`.
#[test]
fn the_general_slot_is_basic_feat_selection() {
    // Game: BasicFeatSelection 247a4068296e8be42890143f451b4b45, which BasicFeatsProgression
    // 5b72dd2ca2cb73b49903806ee8986325 grants at levels 1, 3, 5, … 19 and HumanRace 0a5d473ead98b0646b94495af250fdc4
    // in its `m_Features`.
    let basic = any_selection(BASIC_FEAT_SELECTION);
    assert_eq!(
        basic.blueprint.as_ref().map(|guid| guid.as_str()),
        Some("247a4068296e8be42890143f451b4b45")
    );
    // The planner's list: every feat it has (the game's 276 are 293's).
    let every: BTreeSet<FeatKey> = DATA.feats.iter().map(|feat| feat.key.clone()).collect();
    assert_eq!(basic.feats, every);
    assert!(!basic.ignores_prerequisites);
    let human = DATA
        .races
        .iter()
        .find(|race| race.key.as_str() == "human")
        .expect("humans");
    assert_eq!(
        human.selections,
        [FeatureSelectionKey::new(BASIC_FEAT_SELECTION)]
    );
    let sheet = engine().evaluate(&build("fighter", 3));
    for level in [1u8, 3] {
        let slot = sheet.levels[usize::from(level) - 1]
            .slots
            .iter()
            .find(|slot| {
                slot.id
                    == SlotId::Feat {
                        level,
                        source: FeatSource::General,
                        index: 0,
                    }
            })
            .expect("the general slot");
        assert_eq!(
            slot.kind,
            SlotKind::Feat(FeatureSelectionKey::new(BASIC_FEAT_SELECTION))
        );
    }
}

/// The sorcerer's bloodline feat at 7, 13 and 19 is the game's `SorcererFeatSelection`, offering the held bloodlines'
/// lists, nested as the game nests them (ticket 304).
#[test]
fn the_bloodline_feat_slot_is_sorcerer_feat_selection() {
    // Game: SorcererProgression grants SorcererFeatSelection 3a60f0c0442acfb419b0c03b584e1394 at levels 7, 13 and 19.
    let sorcerer = class("sorcerer");
    let granting: Vec<u8> = sorcerer
        .levels
        .iter()
        .filter(|row| {
            row.selections.iter().any(|key| {
                any_selection(key.as_str())
                    .blueprint
                    .as_ref()
                    .is_some_and(|guid| guid.as_str() == "3a60f0c0442acfb419b0c03b584e1394")
            })
        })
        .map(|row| row.level)
        .collect();
    assert_eq!(granting, [7, 13, 19]);
    // Ticket 304: it nests the bloodlines' feat selections as the game's `m_AllFeatures` lists them, each an option
    // whose pick is the list's feat.
    let bloodline_feats = any_selection("sorcerer/bloodline-feats");
    assert!(DATA.nests_feat_lists(bloodline_feats) && bloodline_feats.feats.is_empty());
    let nested: Vec<&str> = DATA
        .options_of(bloodline_feats)
        .map(|option| {
            option
                .blueprint
                .as_ref()
                .expect("a nested list's blueprint")
                .as_str()
        })
        .collect();
    // Game: SorcererFeatSelection 3a60f0c0442acfb419b0c03b584e1394 `m_AllFeatures`: BloodlineAbyssalFeatSelection
    // 7781831170d794a4d8f147a1326c35fd, BloodlineArcaneFeatSelection ff4fd877b4c801342ab8e880b734a6b9,
    // BloodlineCelestialFeatSelection 101a87c6744a3f242836e657cd7f643e, BloodlineDraconicFeatSelection
    // f4b011d090e8ae543b1441bd594c7bf7, BloodlineElementalFeatSelection d2a4b74ee7e43a648b51d0f36db2aa34,
    // BloodlineFeyFeatSelection d7f233e417c490545b00f49c3940638c, BloodlineInfernalFeatSelection
    // f19d9bfcbc1e3ea42bda754a03c40843, BloodlineSerpentineFeatSelection 76be9555a33d58a4bb419154540595df,
    // BloodlineUndeadFeatSelection a29b72a804f7cb243b01e99c42452636.
    assert_eq!(
        nested.iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "7781831170d794a4d8f147a1326c35fd",
            "ff4fd877b4c801342ab8e880b734a6b9",
            "101a87c6744a3f242836e657cd7f643e",
            "f4b011d090e8ae543b1441bd594c7bf7",
            "d2a4b74ee7e43a648b51d0f36db2aa34",
            "d7f233e417c490545b00f49c3940638c",
            "f19d9bfcbc1e3ea42bda754a03c40843",
            "76be9555a33d58a4bb419154540595df",
            "a29b72a804f7cb243b01e99c42452636",
        ])
    );
    assert_eq!(nested.len(), 9);
}

/// The monk's 1st-level bonus feats keep their list, which is the game's.
#[test]
fn monk_bonus_feats_keep_their_list() {
    // Game: MonkBonusFeatSelectionLevel1 ac3b7f5c11bce4e44aeb332f66b75bab: `m_AllFeatures` CombatReflexes, Dodge,
    // CraneStyleFeat, BlindFight, Improved Initiative, DeflectArrows; `IgnorePrerequisites` true.
    let list = any_selection("monk/bonus-feats-1");
    assert_eq!(
        list.blueprint.as_ref().map(|guid| guid.as_str()),
        Some("ac3b7f5c11bce4e44aeb332f66b75bab")
    );
    assert!(list.ignores_prerequisites);
    assert_eq!(
        list.feats,
        feats(&[
            "blind-fight",
            "combat-reflexes",
            "crane-style",
            "deflect-arrows",
            "dodge",
            "improved-initiative",
        ])
    );
    assert!(list.options.is_empty() && !DATA.nests_feat_lists(list));
    assert!(
        class("monk").levels[0]
            .selections
            .contains(&FeatureSelectionKey::new("monk/bonus-feats-1"))
    );
}

/// Gold Dragon's Dragon Feats list every feat the planner has, by name now, and still ignore prerequisites.
#[test]
fn dragon_feats_list_every_feat() {
    // The planner's list, unchanged: today's every feat (263, with ticket 306k's Extra Discovery (Vivisectionist)). The game's DragonLevel2FeatSelection
    // a21acdafc0169f5488a9bd3256e2e65b lists 268, which 293 writes.
    let dragon = any_selection("gold-dragon/dragon-feats");
    let every: BTreeSet<FeatKey> = DATA.feats.iter().map(|feat| feat.key.clone()).collect();
    assert_eq!(every.len(), 263);
    assert_eq!(dragon.feats, every);
    assert!(dragon.ignores_prerequisites);
    let path = DATA
        .mythic_paths
        .iter()
        .find(|path| path.key.as_str() == "gold-dragon")
        .expect("Gold Dragon");
    let granted = path
        .ranks
        .iter()
        .flat_map(|row| &row.selections)
        .filter(|key| key.as_str() == "gold-dragon/dragon-feats")
        .count();
    assert_eq!(granted, 3);
}
