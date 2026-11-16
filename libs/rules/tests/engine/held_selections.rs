//! A selection held as a fact of its own (ticket 247): a row that grants a Feature selection (a class's, an
//! archetype's, a mythic path's or an option's) holds the selection from its level, whatever its slot holds, and
//! [`Prerequisite::Selection`] reads it. The game adds each selection a progression grants as a fact
//! (`UnitLogic/Class/LevelUp/Actions/LevelUpHelper.cs`, `AddFeaturesFromProgression`, WrathDecompiled 6ce4a6b), so a
//! `PrerequisiteFeature` naming a selection (Extra Rage Power's `RagePowerSelection`) holds with the slot empty.

use rules::{
    Alignment, BASIC_FEAT_SELECTION, Build, Engine, FeatureSelectionKey, Issue, Prerequisite,
    SlotId,
};

use crate::support::*;

fn holds(key: &str) -> Prerequisite {
    Prerequisite::Selection(FeatureSelectionKey::new(key))
}

/// Whether the state at the end of `level` holds the selection.
fn held_at(build: &Build, level: u8, key: &str) -> bool {
    walked(&engine(), build)
        .at(level)
        .expect("the level")
        .check(&holds(key))
        .is_ok()
}

#[test]
fn a_class_rows_selection_is_held_from_its_level_with_its_slot_empty() {
    // Rogue talents at rogue 2, no talent picked.
    let rogue = build("human", &times("rogue", 2));
    assert!(!held_at(&rogue, 1, "rogue/rogue-talents"));
    assert!(held_at(&rogue, 2, "rogue/rogue-talents"));
}

#[test]
fn an_option_taken_through_another_selection_doesnt_hold_the_selection_that_defines_it() {
    // A slayer's talent slot shares the rogue talents: Fast Stealth taken there is held, the rogue's selection isn't.
    let mut slayer = build("human", &times("slayer", 2));
    slayer.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    let sheet = walked(&engine(), &slayer);
    let state = sheet.at(2).unwrap();
    assert_eq!(
        state.check(&Prerequisite::SelectionOption(option(
            "rogue/rogue-talents/fast-stealth"
        ))),
        Ok(())
    );
    assert!(state.check(&holds("rogue/rogue-talents")).is_err());
    assert_eq!(state.check(&holds("slayer/slayer-talents")), Ok(()));
}

#[test]
fn an_options_row_and_a_mythic_paths_row_hold_their_selections_too() {
    // The combat style feat's row 1 grants the slayer's combat style selection.
    let mut slayer = build("human", &times("slayer", 2));
    slayer.picks.insert(
        selection_slot(2, 0),
        pick_option("slayer/slayer-talents/combat-style-feat-2"),
    );
    assert!(!held_at(&slayer, 1, "slayer/combat-style"));
    assert!(held_at(&slayer, 2, "slayer/combat-style"));

    // The Trickster's row at MR 3 grants its tricks.
    let mut fighter = build("human", &times("fighter", 20));
    fighter.start.alignment = Some(Alignment::LawfulGood);
    fighter.mythic_path = Some(path("trickster"));
    let sheet = walked(&engine(), &fighter);
    let level = sheet.ranks[2].character_level;
    let before = sheet.ranks[1].character_level;
    assert!(before < level);
    let at = |level: u8| sheet.at(level).unwrap().check(&holds("trickster/tricks"));
    assert!(at(before).is_err());
    assert_eq!(at(level), Ok(()));
}

/// The test world with Extra Rogue Talent needing the rogue's talent selection, as the game's needs its rogue
/// talent selections.
fn extra_talent_needs_the_selection() -> Engine {
    let mut data = world();
    data.feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "extra-rogue-talent")
        .unwrap()
        .prerequisite = Some(holds("rogue/rogue-talents"));
    engine_on(data)
}

fn extra_talent_status(engine: &Engine, build: &Build, slot: &SlotId) -> Result<(), Issue> {
    engine
        .options(build, slot)
        .expect("the slot")
        .into_iter()
        .find(|option| option.pick == pick_feat("extra-rogue-talent"))
        .expect("offered")
        .status
}

#[test]
fn a_feat_needing_a_selection_is_open_from_the_rows_level_and_the_walk_agrees() {
    let engine = extra_talent_needs_the_selection();
    // Rogue 3, the talent slot at 2 empty: Extra Rogue Talent in the general slot at 3 holds.
    let mut rogue = build("human", &times("rogue", 3));
    assert_eq!(
        extra_talent_status(&engine, &rogue, &general_slot(3)),
        Ok(())
    );
    assert!(extra_talent_status(&engine, &rogue, &general_slot(1)).is_err());
    rogue
        .picks
        .insert(general_slot(3), pick_feat("extra-rogue-talent"));
    assert!(engine.is_possible(&rogue));
    let agreement = rules::agreement::judge(&engine, &rogue);
    assert!(
        agreement.disagreements.is_empty(),
        "{:?}",
        agreement.lines("rogue 3")
    );

    // A slayer 3 holding a rogue talent through its own slot doesn't meet it, and the reason names the selection.
    let mut slayer = build("human", &times("slayer", 3));
    slayer.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    let status = extra_talent_status(&engine, &slayer, &general_slot(3));
    let Err(Issue::Unmet(unmet)) = status else {
        panic!("unmet: {status:?}");
    };
    assert_eq!(engine.unmet_text(&unmet), "the Rogue Talents selection");
    slayer
        .picks
        .insert(general_slot(3), pick_feat("extra-rogue-talent"));
    assert!(!engine.is_possible(&slayer));
}

#[test]
fn a_selection_of_feats_is_held_as_the_game_holds_it() {
    // The game adds each selection the race or a progression grants as a fact, a selection of feats too
    // (`SelectRace.Apply` and `LevelUpHelper.AddFeaturesFromProgression`, WrathDecompiled 6ce4a6b): the general slot's
    // (BasicFeatsProgression 5b72dd2ca2cb73b49903806ee8986325's levels 1, 3, …: BasicFeatSelection
    // 247a4068296e8be42890143f451b4b45), the race's (HumanRace 0a5d473ead98b0646b94495af250fdc4's features), a class
    // row's and a mythic path rank's.
    let elf = build("elf", &times("wizard", 1));
    assert!(held_at(&elf, 1, BASIC_FEAT_SELECTION));
    let half_elf = build("half-elf", &times("rogue", 1));
    assert!(held_at(&half_elf, 1, "half-elf/adaptability"));
    assert!(!held_at(&elf, 1, "half-elf/adaptability"));
    // The fighter's combat feats from its first row; a wizard's row holds none.
    let fighter = build("elf", &times("fighter", 1));
    assert!(held_at(&fighter, 1, "combat-feats"));
    assert!(!held_at(&elf, 1, "combat-feats"));
    // The Trickster's MR 4 bonus feats.
    let mut trickster = build("human", &times("fighter", 20));
    trickster.start.alignment = Some(Alignment::LawfulGood);
    trickster.mythic_path = Some(path("trickster"));
    let sheet = walked(&engine(), &trickster);
    let at = |level: u8| {
        (sheet.at(level).unwrap())
            .check(&holds("trickster/bonus-feats"))
            .is_ok()
    };
    assert!(!at(sheet.ranks[2].character_level));
    assert!(at(sheet.ranks[3].character_level));
}
