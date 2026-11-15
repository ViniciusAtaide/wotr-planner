//! The splits ticket's three shapes, as the game models them: a ranked feature (one feature granted again, its rank
//! the number of grants), a repeated selection (one selection named by several rows, each taking another option),
//! and a filtered pool (one selection whose options its `filtered_by` selection's held option admits); with the
//! option level that counts half the other classes (an oracle's curse) and an option's old keys in a link.

use std::collections::BTreeSet;

use rules::{
    Build, ClassChoice, DataAt, DataIssue, FeatureKey, FeatureSelectionKey, GameData, Issue,
    LinkKeys, OptionLevelRow, Prerequisite, SelectionOptionKey, decode_build, encode_build,
};

use crate::support::*;

fn status(
    engine: &rules::Engine,
    build: &Build,
    slot: &rules::SlotId,
    key: &str,
) -> Result<(), Issue> {
    engine
        .options(build, slot)
        .unwrap()
        .into_iter()
        .find(|o| o.pick.key() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
        .status
}

// ---- a ranked feature -----------------------------------------------------------------------------------------

/// Sneak Attack at rogue 1, 3 and 5 is one feature granted three times: its rank at each level is the grants so
/// far, and a prerequisite reads it (the game: `Feature.AddRank` on each grant of a held feature,
/// `FeatureCollection.PrepareFactForAttach`, WrathDecompiled 6ce4a6b).
#[test]
fn a_feature_granted_again_gains_a_rank_that_a_prerequisite_reads() {
    let mut data = world();
    let sneak = feature("rogue/sneak-attack");
    data.feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "dodge")
        .unwrap()
        .prerequisite = Some(Prerequisite::Feature {
        feature: sneak.clone(),
        count: 2,
    });
    let engine = engine_on(data);
    let rogue = build("elf", &times("rogue", 5));
    let sheet = engine.evaluate(&rogue);
    let ranks: Vec<u8> = (1..=5)
        .map(|level| sheet.at(level).unwrap().feature_count(&sneak))
        .collect();
    assert_eq!(ranks, [1, 1, 2, 2, 3]);
    assert!(matches!(
        status(&engine, &rogue, &general_slot(1), "dodge"),
        Err(Issue::Unmet(_))
    ));
    assert_eq!(status(&engine, &rogue, &general_slot(3), "dodge"), Ok(()));
}

// ---- a repeated selection -------------------------------------------------------------------------------------

/// The rogue's talents at 2 and 4 are one selection named by two rows: each row is a slot of its own, and an option
/// the first took is held at the second.
#[test]
fn a_selection_named_by_two_rows_takes_another_option_at_each() {
    let engine = engine();
    let mut rogue = build("elf", &times("rogue", 4));
    rogue.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    assert!(engine.is_possible(&rogue));
    let second = selection_slot(4, 0);
    assert!(
        offered_on(&engine, &rogue, &second)
            .contains(&"rogue/rogue-talents/fast-stealth".to_owned())
    );
    assert!(matches!(
        status(&engine, &rogue, &second, "rogue/rogue-talents/fast-stealth"),
        Err(Issue::AlreadyHeld)
    ));
    assert_eq!(
        status(&engine, &rogue, &second, "rogue/rogue-talents/combat-trick"),
        Ok(())
    );
}

// ---- a filtered pool ------------------------------------------------------------------------------------------

/// Three clerics' level-3 revelation (`cleric/revelation`, filtered by `cleric/domain`): a revelation its domain
/// admits is offered only with that domain, one admitted by either of two domains with both, and once admitted
/// the rest of its prerequisite still decides (the game: `BlueprintFeatureSelection.CanSelect` checks each option's
/// `MeetsPrerequisites`, WrathDecompiled 6ce4a6b; the game lists the others greyed, the planner leaves them out).
#[test]
fn a_filtered_pool_offers_what_the_held_option_admits_and_checks_the_rest() {
    let mut data = world();
    let fire = || Prerequisite::SelectionOption(option("cleric/domain/fire"));
    let water = || Prerequisite::SelectionOption(option("cleric/domain/water"));
    let revelations = &mut class_mut(&mut data, "cleric")
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "cleric/fire-revelations")
        .unwrap()
        .options;
    // Fire Breath: the fire domain and character level 5. Burning Hands: either domain.
    revelations[1].prerequisite = Some(Prerequisite::All(vec![
        fire(),
        Prerequisite::CharacterLevel(5),
    ]));
    revelations[0].prerequisite = Some(Prerequisite::Any(vec![fire(), water()]));
    let engine = engine_on(data);
    let with_domain = |domain: &str| {
        let mut cleric = build("elf", &times("cleric", 3));
        cleric
            .picks
            .insert(selection_slot(1, 0), pick_option(domain));
        cleric
    };
    let slot = selection_slot(3, 0);
    let fire = with_domain("cleric/domain/fire");
    assert_eq!(
        offered_on(&engine, &fire, &slot),
        [
            "cleric/fire-revelations/burning-hands",
            "cleric/fire-revelations/fire-breath",
            "cleric/fire-revelations/maneuver-mastery",
            "cleric/fire-revelations/weapon-mastery"
        ]
    );
    assert!(matches!(
        status(&engine, &fire, &slot, "cleric/fire-revelations/fire-breath"),
        Err(Issue::Unmet(_))
    ));
    let water = with_domain("cleric/domain/water");
    assert_eq!(
        offered_on(&engine, &water, &slot),
        [
            "cleric/fire-revelations/burning-hands",
            "cleric/water-revelations/icy-skin",
            "cleric/water-revelations/wave"
        ]
    );
    // The War domain admits none of them.
    assert!(offered_on(&engine, &with_domain("cleric/domain/war"), &slot).is_empty());
    // A pick the picker leaves out is refused by its prerequisite, which says why.
    let mut stray = fire;
    stray.picks.insert(
        slot.clone(),
        pick_option("cleric/water-revelations/icy-skin"),
    );
    let failures = engine.failures(&stray);
    assert_eq!(failures.len(), 1);
    assert!(matches!(failures[0].issue, Issue::Unmet(_)));
}

/// The picker leaves out only an option its prerequisite can't take without an admitting option: one that names the
/// fire domain beside another way in (`Any`), or as a domain not to hold (`Not`), or among fewer than it needs
/// (`AtLeast`), is offered to a War cleric and its prerequisite decides.
#[test]
fn a_filtered_pool_leaves_out_only_what_needs_an_admitting_option() {
    let mut data = world();
    let fire = || Prerequisite::SelectionOption(option("cleric/domain/fire"));
    let revelations = &mut class_mut(&mut data, "cleric")
        .selections
        .iter_mut()
        .find(|selection| selection.key.as_str() == "cleric/fire-revelations")
        .unwrap()
        .options;
    // Burning Hands: fire or level 1. Fire Breath: not fire. Maneuver Mastery: one of fire and level 1.
    revelations[0].prerequisite = Some(Prerequisite::Any(vec![
        fire(),
        Prerequisite::CharacterLevel(1),
    ]));
    revelations[1].prerequisite = Some(Prerequisite::Not(Box::new(fire())));
    revelations[2].prerequisite = Some(Prerequisite::AtLeast(
        1,
        vec![fire(), Prerequisite::CharacterLevel(1)],
    ));
    let engine = engine_on(data);
    let mut war = build("elf", &times("cleric", 3));
    war.picks
        .insert(selection_slot(1, 0), pick_option("cleric/domain/war"));
    let slot = selection_slot(3, 0);
    assert_eq!(
        offered_on(&engine, &war, &slot),
        [
            "cleric/fire-revelations/burning-hands",
            "cleric/fire-revelations/fire-breath",
            "cleric/fire-revelations/maneuver-mastery",
        ]
    );
    for key in [
        "cleric/fire-revelations/burning-hands",
        "cleric/fire-revelations/fire-breath",
        "cleric/fire-revelations/maneuver-mastery",
    ] {
        assert_eq!(status(&engine, &war, &slot, key), Ok(()), "{key}");
    }
}

// ---- an option level from half the other classes ---------------------------------------------------------------

const CURSE: &str = "cleric/oracles-curse";
const LAME: &str = "cleric/oracles-curse/lame";
const STAGE_2: &str = "cleric/oracles-curse/lame/stage-2";

/// A world whose cleric gets, at cleric 2, a curse levelled as the game levels an oracle's curse: cleric and witch
/// levels, but a witch's only as a False God (an archetype of the witch's), and half the other classes' levels.
/// Its one option, Lame, has a stage at option level 5.
fn curse_world() -> GameData {
    let mut data = world();
    let cleric = class_mut(&mut data, "cleric");
    let mut stage = cleric.features[0].clone();
    stage.key = FeatureKey::new(STAGE_2);
    stage.name = "Lame (stage 2)".into();
    cleric.features.push(stage);
    let mut curse = cleric
        .selections
        .iter()
        .find(|selection| selection.key.as_str() == "cleric/water-revelations")
        .unwrap()
        .clone();
    curse.key = FeatureSelectionKey::new(CURSE);
    curse.option_levels = BTreeSet::from([class("cleric"), class("witch")]);
    curse.option_level_archetypes = BTreeSet::from([rules::ArchetypeKey::new("witch/false-god")]);
    curse.half_other_classes = true;
    let mut lame = curse.options[0].clone();
    lame.key = SelectionOptionKey::new(LAME);
    lame.prerequisite = None;
    lame.levels = vec![OptionLevelRow {
        level: 5,
        features: vec![FeatureKey::new(STAGE_2)],
        ..OptionLevelRow::default()
    }];
    curse.options = vec![lame];
    cleric.selections.push(curse);
    cleric.levels[1]
        .selections
        .push(FeatureSelectionKey::new(CURSE));
    data
}

/// The curse's option level after cleric 2 and `then`, taken with `archetype` when given.
fn curse_level(then: &[&'static str], archetype: Option<(&str, &str)>) -> (u8, bool) {
    let engine = engine_on(curse_world());
    let mut classes = vec!["cleric", "cleric"];
    classes.extend_from_slice(then);
    let mut build = build("elf", &classes);
    if let Some((class_key, archetype)) = archetype {
        build.class_choices.insert(
            class(class_key),
            ClassChoice {
                archetype: Some(rules::ArchetypeKey::new(archetype)),
                ..ClassChoice::default()
            },
        );
    }
    let sheet = engine.evaluate(&build);
    let last = sheet.levels.last().unwrap();
    (
        last.state.option_level(&option(LAME)),
        last.state.feature_count(&feature(STAGE_2)) > 0,
    )
}

/// `BlueprintProgression.CalcLevel` (WrathDecompiled 6ce4a6b): the classes named, but one that has a named archetype
/// counts only taken with it; each class taken with a named archetype; and half the other classes' levels, the mythic
/// classes' among them (`UnitProgressionData.SetupLevelsIfNecessary`: their levels are the Mythic rank). The test
/// world's ranks come at levels 6 and 7.
#[test]
fn an_option_level_counts_half_the_other_classes_and_a_class_only_through_its_named_archetype() {
    // Cleric 2 and fighter 3: 2 + 3 / 2, rounded down.
    assert_eq!(curse_level(&times("fighter", 3), None), (3, false));
    // Cleric 2, fighter 4 and MR 1: 2 + (4 + 1) / 2.
    assert_eq!(curse_level(&times("fighter", 4), None), (4, false));
    // Cleric 2, fighter 6 and MR 2: 2 + (6 + 2) / 2.
    assert_eq!(curse_level(&times("fighter", 6), None), (6, true));
    // A plain witch's levels count neither in full nor by half; MR 2 by half.
    assert_eq!(curse_level(&times("witch", 6), None), (3, false));
    // A False God's count in full.
    assert_eq!(
        curse_level(&times("witch", 3), Some(("witch", "witch/false-god"))),
        (5, true)
    );
}

// ---- an option's old keys -------------------------------------------------------------------------------------

/// The old keys of the rogue talent Fast Stealth.
fn stealth_aliases(data: &mut GameData) -> &mut BTreeSet<SelectionOptionKey> {
    &mut class_mut(data, "rogue").selections[0]
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "rogue/rogue-talents/fast-stealth")
        .unwrap()
        .aliases
}

/// A link naming an option's old key opens on the option. An old key is read only by the link decoder: no option nor
/// other alias may have it, and nothing else may name it.
#[test]
fn a_link_naming_an_options_old_key_reads_as_the_option() {
    let mut data = world();
    let old_key = SelectionOptionKey::new("rogue/rogue-talents/quick-stealth");
    stealth_aliases(&mut data).insert(old_key.clone());
    let mut old = build("elf", &times("rogue", 2));
    old.picks
        .insert(selection_slot(2, 0), pick_option(old_key.as_str()));
    let mut new = old.clone();
    new.picks.insert(
        selection_slot(2, 0),
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    let keys = LinkKeys::new(&data);
    assert!(keys.collisions().is_empty());
    assert_eq!(decode_build(&encode_build(&old), &keys), Ok(new));
    assert_eq!(data.check(), []);

    // An alias that is another option's key, an alias two options have, and a prerequisite naming an old key.
    let trick = SelectionOptionKey::new("rogue/rogue-talents/combat-trick");
    stealth_aliases(&mut data).insert(trick.clone());
    class_mut(&mut data, "rogue").selections[0]
        .options
        .iter_mut()
        .find(|o| o.key == trick)
        .unwrap()
        .aliases
        .insert(old_key.clone());
    let dodge = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "dodge")
        .unwrap();
    dodge.prerequisite = Some(Prerequisite::SelectionOption(old_key.clone()));
    let problems: Vec<(DataAt, DataIssue)> = data
        .check()
        .into_iter()
        .map(|problem| (problem.at, problem.issue))
        .collect();
    assert_eq!(
        problems,
        [
            (
                DataAt::Feat(rules::FeatKey::new("dodge")),
                DataIssue::UnknownKey(DataAt::SelectionOption(old_key.clone()))
            ),
            (DataAt::SelectionOption(trick), DataIssue::DuplicateKey),
            (DataAt::SelectionOption(old_key), DataIssue::DuplicateKey),
        ]
    );
}
