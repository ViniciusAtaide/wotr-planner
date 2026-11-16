//! Selections of feats held as facts (ticket 305): the game adds each selection the race or a progression grants as
//! a fact of its own, a selection of feats too (`Kingmaker/UnitLogic/Class/LevelUp/Actions/LevelUpHelper.cs`,
//! `AddFeaturesFromProgression`: `unit.AddFact(item)`; `SelectRace.cs`, `Apply`, through it with the race's features;
//! WrathDecompiled 6ce4a6b), so [`Prerequisite::Selection`] holds for one from the level that grants it.

use rules::{
    BASIC_FEAT_SELECTION, Build, ClassKey, FeatureSelectionKey, LevelUp, MythicPathKey,
    Prerequisite,
};

use crate::support::*;

fn holds(state: &rules::CharacterState, key: &str) -> bool {
    (state.check(&Prerequisite::Selection(FeatureSelectionKey::new(key)))).is_ok()
}

#[test]
fn the_general_feat_selection_is_held_from_level_1() {
    // Game: BasicFeatsProgression 5b72dd2ca2cb73b49903806ee8986325's level 1 (and 3, 5, …) grants BasicFeatSelection
    // 247a4068296e8be42890143f451b4b45; HumanRace 0a5d473ead98b0646b94495af250fdc4's features hold it too.
    let wizard = build("wizard", 1);
    assert!(holds(&at(&wizard, 1), BASIC_FEAT_SELECTION));
    let mut human = wizard.clone();
    human.start.race = Some(rules::RaceKey::new("human"));
    assert!(holds(&at(&human, 1), BASIC_FEAT_SELECTION));
}

#[test]
fn a_class_rows_selection_of_feats_is_held_from_its_level() {
    // Game: FighterProgression's level 1 grants FighterFeatSelection 41c8486641f7d6d4283ca9dae4147a9f (the data's
    // combat feats); a wizard's progression grants none.
    assert!(holds(&at(&build("fighter", 1), 1), "combat-feats"));
    assert!(!holds(&at(&build("wizard", 1), 1), "combat-feats"));
}

/// A wizard 17 on the Trickster path that switched to the Gold Dragon: MR 9 at 17.
fn gold_dragon_wizard_17() -> Build {
    let mut wizard = mythic("trickster", 17);
    wizard.levels = (0..17)
        .map(|_| LevelUp::new(ClassKey::new("wizard")))
        .collect();
    wizard.class_choices.clear();
    wizard.path_switch = Some(MythicPathKey::new("gold-dragon"));
    wizard
}

#[test]
fn a_path_ranks_selection_of_feats_is_held_from_its_rank() {
    // Game: GoldenDragonProgression a6fbca43902c6194c947546e89af64bd's level 2 (the data's MR 9) grants
    // DragonLevel2FeatSelection a21acdafc0169f5488a9bd3256e2e65b three times (the data's dragon feats).
    let sheet = engine().evaluate(&gold_dragon_wizard_17());
    let (before, state) = (sheet.at(16).unwrap(), sheet.at(17).unwrap());
    assert_eq!((before.mythic_rank, state.mythic_rank), (7, 9));
    assert!(!holds(before, "gold-dragon/dragon-feats"));
    assert!(holds(state, "gold-dragon/dragon-feats"));
}
