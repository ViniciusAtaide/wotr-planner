//! The lookups the engine reads from indexes built once with it (ticket 83): a class level's row, whether a class
//! casts, which options a selection offers, the one option a selection grants, a pool's options and an
//! option-list slot's lists. Each test pins an answer an index could get wrong where a scan of the game data
//! wouldn't.

use rules::{
    ArchetypeKey, Build, ClassChoice, Engine, FeatureSelectionKey, GameData, SlotId, SlotKind,
    SpellLevels,
};

use crate::support::*;

fn class_mut<'a>(data: &'a mut GameData, key: &str) -> &'a mut rules::Class {
    data.all_classes_mut()
        .find(|c| c.key.as_str() == key)
        .unwrap()
}

fn with_archetype(mut build: Build, of: &str, archetype: &str) -> Build {
    build.class_choices.insert(
        class(of),
        ClassChoice {
            archetype: Some(ArchetypeKey::new(archetype)),
            ..ClassChoice::default()
        },
    );
    build
}

fn slot_ids(engine: &Engine, build: &Build, level: usize) -> Vec<SlotId> {
    engine.evaluate(build).levels[level - 1]
        .slots
        .iter()
        .map(|slot| slot.id.clone())
        .collect()
}

/// The keys offered, each once (an option with a chosen feat is offered once per value).
fn offered(engine: &Engine, build: &Build, slot: &SlotId) -> Vec<String> {
    let mut keys: Vec<String> = engine
        .options(build, slot)
        .unwrap()
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect();
    keys.dedup();
    keys
}

// ---- class rows -----------------------------------------------------------------------------------------------

#[test]
fn a_class_level_gets_the_row_of_its_own_number_when_the_table_has_a_gap() {
    // Without its row 2, the fighter's row 3 is second in the table and row 4 (a Combat feat slot) third.
    let mut data = world();
    class_mut(&mut data, "fighter").levels.remove(1);
    let engine = Engine::new(data);
    let build = build("elf", &times("fighter", 3));

    assert_eq!(slot_ids(&engine, &build, 2), []);
    assert_eq!(slot_ids(&engine, &build, 3), [general_slot(3)]);
    let at_2 = engine.evaluate(&build).at(2).cloned().unwrap();
    assert_eq!(at_2.feature_count(&feature("fighter/bravery")), 0);
}

#[test]
fn an_archetypes_rows_stored_out_of_level_order_apply_at_their_own_levels() {
    let mut data = world();
    class_mut(&mut data, "fighter").archetypes[0]
        .levels
        .reverse();
    let build = with_archetype(
        build("elf", &times("fighter", 4)),
        "fighter",
        "fighter/armiger",
    );

    let reversed = Engine::new(data);
    assert_eq!(slot_ids(&reversed, &build, 2), [class_slot(2, 0)]);
    assert_eq!(slot_ids(&reversed, &build, 3), [general_slot(3)]);
    assert_eq!(slot_ids(&reversed, &build, 4), []);
}

#[test]
fn whether_a_class_casts_follows_its_archetype() {
    // An armiger whose 2nd-level row casts 1st-level arcane spells: the fighter casts arcane spells only with it,
    // so only then can an Eldritch Knight advance it.
    let mut data = world();
    let armiger = &mut class_mut(&mut data, "fighter").archetypes[0];
    armiger
        .levels
        .iter_mut()
        .find(|row| row.level == 2)
        .unwrap()
        .spell_levels = Some(SpellLevels {
        arcane: 1,
        divine: 0,
    });
    let casting = Engine::new(data);
    let mut classes = times("wizard", 3);
    classes.extend(times("fighter", 2));
    classes.extend(times("eldritch-knight", 2));
    let mut build = with_archetype(build("elf", &classes), "fighter", "fighter/armiger");
    build.class_choices.insert(
        class("eldritch-knight"),
        ClassChoice {
            archetype: None,
            advances: [class("fighter")].into(),
        },
    );

    assert!(casting.is_possible(&build));
    assert!(!engine().is_possible(&build));
    let mut plain_fighter = build.clone();
    plain_fighter.class_choices.remove(&class("fighter"));
    assert!(!casting.is_possible(&plain_fighter));
}

// ---- what a selection offers ----------------------------------------------------------------------------------

/// A ranger with the Archery style at 2, then a sorcerer with the Red Draconic bloodline (character levels 3–9).
fn ranger_then_sorcerer() -> Build {
    let mut classes = times("ranger", 2);
    classes.extend(times("sorcerer", 7));
    let mut build = build("elf", &classes);
    build.picks.insert(
        selection_slot(2, 0),
        pick_option("ranger/combat-style/archery"),
    );
    build.picks.insert(
        selection_slot(3, 0),
        pick_option("sorcerer/bloodline/draconic-red"),
    );
    build
}

#[test]
fn an_option_list_slot_reads_only_the_lists_of_the_options_its_selection_offers() {
    let engine = engine();
    let build = ranger_then_sorcerer();
    assert!(engine.is_possible(&build));
    // The sorcerer's 7th-level bloodline slot: the Draconic list, not the Archery style's.
    assert_eq!(
        offered(&engine, &build, &class_slot(9, 0)),
        ["power-attack", "toughness"]
    );
    // The ranger's style slot: the Archery list, not the bloodline's.
    assert_eq!(
        offered(&engine, &build, &class_slot(2, 0)),
        ["dodge", "toughness"]
    );
}

#[test]
fn an_option_list_pickers_statuses_are_what_the_walk_finds_of_each_pick() {
    // The Archery list ignores prerequisites and the Draconic one checks them; a Crossblooded sorcerer's slot
    // picks from both its bloodlines' lists. The lists a picker reads once decide whether a feat is offered and
    // whether its prerequisites count.
    let engine = engine();
    let mut crossblooded = with_archetype(
        build("elf", &times("sorcerer", 7)),
        "sorcerer",
        "sorcerer/crossblooded",
    );
    crossblooded.picks.insert(
        selection_slot(1, 0),
        pick_option("sorcerer/bloodline/abyssal"),
    );
    crossblooded.picks.insert(
        selection_slot(1, 1),
        pick_option("sorcerer/bloodline/arcane"),
    );
    let pickers = [
        (ranger_then_sorcerer(), class_slot(2, 0)),
        (ranger_then_sorcerer(), class_slot(9, 0)),
        (crossblooded, class_slot(7, 0)),
    ];
    for (build, slot) in pickers {
        let options = engine.options(&build, &slot).unwrap();
        assert!(options.iter().any(|option| option.status.is_ok()));
        for option in options {
            let mut picked = build.clone();
            picked.picks.insert(slot.clone(), option.pick.clone());
            assert_eq!(
                engine.is_possible(&picked),
                option.status.is_ok(),
                "{:?} in {slot:?}",
                option.pick
            );
        }
    }
}

// ---- the one option a selection grants ------------------------------------------------------------------------

#[test]
fn a_selection_sharing_exactly_one_option_grants_it_with_no_pick() {
    let mut data = world();
    class_mut(&mut data, "warpriest").archetypes[0].selections[0]
        .shared_options
        .remove(&option("warpriest/blessings/good"));
    let engine = Engine::new(data);
    let build = with_archetype(
        build("elf", &["warpriest"]),
        "warpriest",
        "warpriest/champion",
    );

    assert!(engine.is_possible(&build));
    let level_1 = &engine.evaluate(&build).levels[0];
    let champion = FeatureSelectionKey::new("warpriest/champion/chosen-alignment");
    assert!(
        !level_1
            .slots
            .iter()
            .any(|slot| slot.kind == SlotKind::Selection(champion.clone()))
    );
    assert_eq!(
        level_1
            .state
            .option_count(&option("warpriest/blessings/war")),
        1
    );
}

// ---- pools ----------------------------------------------------------------------------------------------------

#[test]
fn a_pool_offers_only_the_options_the_data_has() {
    // The Fire revelations share a key no selection defines.
    let mut data = world();
    class_mut(&mut data, "cleric")
        .selections
        .iter_mut()
        .find(|s| s.key.as_str() == "cleric/fire-revelations")
        .unwrap()
        .shared_options
        .insert(option("cleric/fire-revelations/lost"));
    let engine = Engine::new(data);
    let mut build = build("elf", &times("cleric", 3));
    build
        .picks
        .insert(selection_slot(1, 0), pick_option("cleric/domain/fire"));

    assert_eq!(
        offered(&engine, &build, &selection_slot(3, 0)),
        [
            "cleric/fire-revelations/burning-hands",
            "cleric/fire-revelations/fire-breath",
            "cleric/fire-revelations/maneuver-mastery",
            "cleric/fire-revelations/weapon-mastery"
        ]
    );
    build.picks.insert(
        selection_slot(3, 0),
        pick_option("cleric/fire-revelations/fire-breath"),
    );
    assert!(engine.is_possible(&build));
}
