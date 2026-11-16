//! Ticket 243's three shapes, as the game models them: a ranked feature (one feature granted again, its rank
//! the number of grants), a repeated selection (one selection named by several rows, each taking another option),
//! and a filtered pool (one selection whose options its `filtered_by` selection's held option admits); with the
//! option level that counts half the other classes (an oracle's curse) and an option's old keys in a link.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Build, ClassChoice, DataAt, DataIssue, FeatureKey, FeatureSelectionKey, GameData, Issue,
    LinkKeys, OptionLevelRow, Prerequisite, ProgressionHeader, SelectionOptionKey, decode_build,
    encode_build,
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
    let mut revelations = own_options_mut(&mut data, "cleric/fire-revelations");
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
    let mut revelations = own_options_mut(&mut data, "cleric/fire-revelations");
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
    let mut stage = feature_mut(&mut data, "cleric/domain/war/battle-rage").clone();
    stage.key = FeatureKey::new(STAGE_2);
    stage.name = "Lame (stage 2)".into();
    data.features.push(stage);
    let mut lame = own_options_mut(&mut data, "cleric/water-revelations")[0].clone();
    let mut curse = selection_mut(&mut data, "cleric/water-revelations").clone();
    curse.key = FeatureSelectionKey::new(CURSE);
    // The curse's level is its option's progression header (ticket 304), as an oracle's curse's is.
    curse.option_levels = BTreeSet::new();
    lame.header = Some(ProgressionHeader {
        classes: BTreeMap::from([(class("cleric"), 0), (class("witch"), 0)]),
        archetypes: BTreeMap::from([(rules::ArchetypeKey::new("witch/false-god"), 0)]),
        half_other_classes: true,
        previous_levels: true,
    });
    lame.key = SelectionOptionKey::new(LAME);
    lame.prerequisite = None;
    lame.levels = vec![OptionLevelRow {
        level: 5,
        features: vec![FeatureKey::new(STAGE_2)],
        ..OptionLevelRow::default()
    }];
    curse.options = BTreeSet::from([lame.key.clone()]);
    data.options.push(lame);
    data.selections.push(curse);
    class_mut(&mut data, "cleric").levels[1]
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

const BATTLE_RAGE: &str = "cleric/domain/war/battle-rage";

/// After cleric 2 then `then`, in the curse world with Lame's header `header`, a row at option level 3 (Battle
/// Rage) beside its stage at 5, and the curse's own classes `own`: the curse's option level, and whether Battle
/// Rage and the stage are held.
fn header_level(
    header: ProgressionHeader,
    own: &[&str],
    then: &[&'static str],
) -> (u8, bool, bool) {
    let mut data = curse_world();
    let lame = option_mut(&mut data, LAME);
    lame.header = Some(header);
    lame.levels.insert(
        0,
        OptionLevelRow {
            level: 3,
            features: vec![FeatureKey::new(BATTLE_RAGE)],
            ..OptionLevelRow::default()
        },
    );
    selection_mut(&mut data, CURSE).option_levels = own.iter().map(|key| class(key)).collect();
    let engine = engine_on(data);
    let mut classes = vec!["cleric", "cleric"];
    classes.extend_from_slice(then);
    let sheet = engine.evaluate(&build("elf", &classes));
    let state = &sheet.levels.last().unwrap().state;
    (
        state.option_level(&option(LAME)),
        state.feature_count(&feature(BATTLE_RAGE)) > 0,
        state.feature_count(&feature(STAGE_2)) > 0,
    )
}

/// A progression header (ticket 304) as `BlueprintProgression.CalcLevel` reads it (WrathDecompiled 6ce4a6b): with no
/// classes and no archetypes, the character level; a class's offset (`ClassWithLevel.AdditionalLevel`) adds to its
/// levels.
#[test]
fn a_header_with_no_classes_counts_the_character_level_and_a_class_its_offset() {
    // Cleric 2 and fighter 3: character level 5, reached a level at a time (the row at 3 too).
    assert_eq!(
        header_level(ProgressionHeader::default(), &[], &times("fighter", 3)),
        (5, true, true)
    );
    // Cleric 2 with an offset of 3: 5 at once; the header's previous levels give the row at 3 too.
    let offset = ProgressionHeader {
        classes: BTreeMap::from([(class("cleric"), 3)]),
        previous_levels: true,
        ..ProgressionHeader::default()
    };
    assert_eq!(header_level(offset, &[], &[]), (5, true, true));
}

/// `LevelUpHelper.UpdateProgression` (WrathDecompiled 6ce4a6b): without `GiveFeaturesForPreviousLevels`, a progression
/// whose level rises gets only the row of the level it reaches, never those it passes over (a wizard's school, a
/// kineticist's element).
#[test]
fn a_header_without_previous_levels_skips_the_rows_it_passes_over() {
    let offset = ProgressionHeader {
        classes: BTreeMap::from([(class("cleric"), 3)]),
        ..ProgressionHeader::default()
    };
    // Taken at option level 5: the row at 5, not the one at 3.
    assert_eq!(header_level(offset.clone(), &[], &[]), (5, false, true));
    // Nor later.
    assert_eq!(
        header_level(offset, &[], &times("cleric", 2)),
        (7, false, true)
    );
    // One level at a time, it gets each row it reaches.
    let plain = ProgressionHeader {
        classes: BTreeMap::from([(class("cleric"), 0)]),
        ..ProgressionHeader::default()
    };
    assert_eq!(
        header_level(plain, &[], &times("cleric", 3)),
        (5, true, true)
    );
}

/// Until 293 and 294 (ticket 304), a selection naming its own classes gives the level of every option it offers, a
/// header or not (the domains, shared by selections the game keeps apart), unless it lets its options' headers go
/// first (an oracle's revelations: Weapon Mastery's own header).
#[test]
fn a_selections_own_classes_go_before_an_options_header_unless_it_lets_headers_first() {
    let fighter_header = ProgressionHeader {
        classes: BTreeMap::from([(class("fighter"), 0)]),
        previous_levels: true,
        ..ProgressionHeader::default()
    };
    // The curse names cleric: cleric 2 and fighter 3 make 2, whatever Lame's header says.
    assert_eq!(
        header_level(fighter_header.clone(), &["cleric"], &times("fighter", 3)),
        (2, false, false)
    );
    // Letting the headers go first: fighter 3.
    let mut data = curse_world();
    option_mut(&mut data, LAME).header = Some(fighter_header);
    let curse = selection_mut(&mut data, CURSE);
    curse.option_levels = BTreeSet::from([class("cleric")]);
    curse.option_headers_first = true;
    let engine = engine_on(data);
    let sheet = engine.evaluate(&build(
        "elf",
        &["cleric", "cleric", "fighter", "fighter", "fighter"],
    ));
    let state = &sheet.levels.last().unwrap().state;
    assert_eq!(state.option_level(&option(LAME)), 3);
}

// ---- an option's old keys -------------------------------------------------------------------------------------

/// The old keys of the rogue talent Fast Stealth.
fn stealth_aliases(data: &mut GameData) -> &mut BTreeSet<SelectionOptionKey> {
    &mut option_mut(data, "rogue/rogue-talents/fast-stealth").aliases
}

/// A link naming an option's old key opens on the option. An old key is read only by the link decoder and its
/// migration: it may be another option's key or another option's alias too (ticket 298, `link_migration`), but not
/// the option's own key, and nothing else may name it.
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

    // An alias that is another option's key and an alias two options have are fine; the option's own key as an alias
    // and a prerequisite naming an old key are not.
    let trick = SelectionOptionKey::new("rogue/rogue-talents/combat-trick");
    let stealth = SelectionOptionKey::new("rogue/rogue-talents/fast-stealth");
    stealth_aliases(&mut data).insert(trick.clone());
    option_mut(&mut data, trick.as_str())
        .aliases
        .insert(old_key.clone());
    assert_eq!(data.check(), []);
    stealth_aliases(&mut data).insert(stealth.clone());
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
            (DataAt::SelectionOption(stealth), DataIssue::DuplicateKey),
        ]
    );
}
