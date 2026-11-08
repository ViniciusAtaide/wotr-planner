//! A slot's options as its picker shows them: grouped by what the pick takes, each entry with its name, chip,
//! summary, story note and the one reason it shows, each option with its value's name and its own reason.

use rules::{
    Build, Engine, FeatCategory, ParameterKind, Prerequisite, SlotEntry, SlotId, SlotOption, Story,
};

use crate::support::*;

fn entries(engine: &Engine, build: &Build, slot: &SlotId) -> Vec<SlotEntry> {
    engine
        .slot_entries(build, slot)
        .expect("the Build has the slot")
}

fn entry<'a>(entries: &'a [SlotEntry], key: &str) -> &'a SlotEntry {
    entries
        .iter()
        .find(|entry| entry.options[0].option.pick.key() == key)
        .unwrap_or_else(|| panic!("no entry {key}"))
}

#[test]
fn a_slots_entries_group_its_options_by_what_the_pick_takes() {
    let engine = engine();
    let build = build("elf", &["fighter"]);
    let slot = general_slot(1);
    let entries = entries(&engine, &build, &slot);

    // Every option once, in the options' order, each entry's options taking one thing.
    let flat: Vec<SlotOption> = entries
        .iter()
        .flat_map(|entry| entry.options.iter().map(|option| option.option.clone()))
        .collect();
    assert_eq!(flat, engine.options(&build, &slot).unwrap());
    for entry in &entries {
        let key = entry.options[0].option.pick.key();
        assert!(entry.options.iter().all(|o| o.option.pick.key() == key));
    }
    let keys: Vec<&str> = entries
        .iter()
        .map(|entry| entry.options[0].option.pick.key())
        .collect();
    let mut unique = keys.clone();
    unique.dedup();
    assert_eq!(keys, unique, "one entry per feat");

    // A feat that names a value: one option per weapon, each with its name.
    let focus = entry(&entries, "weapon-focus");
    assert_eq!(focus.name, "weapon-focus");
    assert_eq!(focus.category, "Weapon");
    let values: Vec<&str> = focus.options.iter().map(|o| o.value.as_str()).collect();
    assert_eq!(
        values,
        [
            "bastard-sword",
            "dagger",
            "dueling-sword",
            "longsword",
            "unarmed-strike"
        ]
    );

    // A plain feat: one option, naming nothing.
    let toughness = entry(&entries, "toughness");
    assert_eq!(toughness.name, "toughness");
    assert_eq!(toughness.category, "Miscellaneous");
    assert_eq!(toughness.noun, None);
    assert_eq!(toughness.options.len(), 1);
    assert_eq!(toughness.options[0].value, "");
    assert!(toughness.open);
    assert_eq!(toughness.reason, None);
}

#[test]
fn a_shared_option_is_filed_under_the_selection_that_defines_it() {
    let engine = engine();
    let build = build("elf", &["slayer", "slayer"]);
    let entries = entries(&engine, &build, &selection_slot(2, 0));
    assert_eq!(
        entry(&entries, "rogue/rogue-talents/fast-stealth").category,
        "Rogue Talents"
    );
}

/// A feat's type picks its icon: the feat's own category, or, for an option granting a chosen feat, that feat's.
#[test]
fn an_entry_names_the_type_of_feat_it_takes() {
    let engine = engine();
    let fighter = build("elf", &["fighter"]);
    let feats = entries(&engine, &fighter, &general_slot(1));
    assert_eq!(
        entry(&feats, "weapon-focus").feat_category,
        Some(FeatCategory::Weapon)
    );
    assert_eq!(
        entry(&feats, "toughness").feat_category,
        Some(FeatCategory::Miscellaneous)
    );

    // A rogue talent granting Weapon Focus is a Weapon feat; one granting no feat has no type.
    let rogue = build("elf", &["rogue", "rogue"]);
    let talents = entries(&engine, &rogue, &selection_slot(2, 0));
    assert_eq!(
        entry(&talents, "rogue/rogue-talents/weapon-focus").feat_category,
        Some(FeatCategory::Weapon)
    );
    assert_eq!(
        entry(&talents, "rogue/rogue-talents/fast-stealth").feat_category,
        None
    );
}

/// The grid's chip shows the same icon for the pick a slot holds.
#[test]
fn a_picks_feat_type_is_its_feats_category() {
    let engine = engine();
    let build = build("elf", &["fighter"]);
    for entry in entries(&engine, &build, &general_slot(1)) {
        for option in &entry.options {
            assert_eq!(
                engine.feat_category(&option.option.pick),
                entry.feat_category
            );
        }
    }
}

#[test]
fn no_entries_for_a_slot_the_build_doesnt_have() {
    assert_eq!(
        engine().slot_entries(&build("elf", &["fighter"]), &general_slot(3)),
        None
    );
}

#[test]
fn a_locked_entry_shows_a_reason_other_than_already_taken() {
    // A warpriest's bonus Weapon Focus ignores its BAB +1: held for the longsword at level 1, with BAB +0.
    let engine = engine();
    let mut build = build("elf", &["warpriest"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    let entries = entries(&engine, &build, &general_slot(1));
    let focus = entry(&entries, "weapon-focus");
    assert!(!focus.open);
    assert_eq!(focus.reason.as_deref(), Some("Needs BAB +1 (have +0)"));
    // Each value keeps its own.
    let reasons: Vec<(&str, Option<&str>)> = focus
        .options
        .iter()
        .map(|o| (o.value.as_str(), o.reason.as_deref()))
        .collect();
    assert_eq!(
        reasons,
        [
            ("bastard-sword", Some("Needs BAB +1 (have +0)")),
            ("dagger", Some("Needs BAB +1 (have +0)")),
            ("dueling-sword", Some("Needs BAB +1 (have +0)")),
            ("longsword", Some("Already taken")),
            ("unarmed-strike", Some("Needs BAB +1 (have +0)")),
        ]
    );
}

#[test]
fn an_entry_held_already_says_so() {
    let engine = engine();
    let mut build = build("elf", &["fighter"]);
    build.picks.insert(class_slot(1, 0), pick_feat("toughness"));
    let entries = entries(&engine, &build, &general_slot(1));
    let toughness = entry(&entries, "toughness");
    assert!(!toughness.open);
    assert_eq!(toughness.reason.as_deref(), Some("Already taken"));
    assert_eq!(
        toughness.options[0].reason.as_deref(),
        Some("Already taken")
    );
}

#[test]
fn an_entry_that_would_break_a_later_choice_shows_no_reason() {
    let engine = engine();
    let mut build = build("elf", &times("fighter", 3));
    build.picks.insert(general_slot(3), pick_feat("toughness"));
    let entries = entries(&engine, &build, &general_slot(1));
    let toughness = entry(&entries, "toughness");
    assert!(!toughness.open);
    assert_eq!(toughness.reason, None);
    assert_eq!(toughness.options[0].reason, None);
}

#[test]
fn an_entry_with_a_value_open_is_open_and_shows_no_reason() {
    // Weapon Focus held for the longsword, the other weapons open at BAB +1.
    let engine = engine();
    let mut build = build("elf", &["fighter"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    let entries = entries(&engine, &build, &general_slot(1));
    let focus = entry(&entries, "weapon-focus");
    assert!(focus.open);
    assert_eq!(focus.reason, None);
    assert_eq!(focus.options[3].reason.as_deref(), Some("Already taken"));
}

#[test]
fn an_entry_with_a_story_condition_notes_it() {
    let engine = engine();
    let mut build = build("human", &times("fighter", 17));
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("gold-dragon"));
    let entries = entries(
        &engine,
        &build,
        &SlotId::PathSelection { rank: 8, index: 0 },
    );
    let annihilating = entry(&entries, "gold-dragon/breaths/annihilating");
    assert_eq!(
        annihilating.story_note.as_deref(),
        Some("(Story: Vow of Dahak)")
    );
    assert_eq!(annihilating.detail(), "(Story: Vow of Dahak)");
    // Never its reason: it's locked by MR 9 alone.
    assert_eq!(
        annihilating.reason.as_deref(),
        Some("Needs Mythic rank 9 (have 8)")
    );
    let empowered = entry(&entries, "gold-dragon/breaths/empowered");
    assert_eq!(empowered.story_note, None);
    assert_eq!(empowered.detail(), "");
}

#[test]
fn an_entrys_detail_is_its_summary_then_its_story_note() {
    let mut data = world();
    let toughness = data
        .feats
        .iter_mut()
        .find(|row| row.key == feat("toughness"))
        .unwrap();
    toughness.summary = "More hit points.".into();
    toughness.prerequisite = Some(Prerequisite::All(vec![
        Prerequisite::Story(Story::Uncorrupted),
        Prerequisite::Story(Story::VowOfDahak),
    ]));
    let dodge = data
        .feats
        .iter_mut()
        .find(|row| row.key == feat("dodge"))
        .unwrap();
    dodge.summary = "+1 AC.".into();
    let engine = Engine::new(data);
    let entries = entries(&engine, &build("elf", &["fighter"]), &general_slot(1));

    let toughness = entry(&entries, "toughness");
    assert_eq!(toughness.summary, "More hit points.");
    assert_eq!(
        toughness.story_note.as_deref(),
        Some("(Story: Vow of Dahak, Uncorrupted)")
    );
    assert_eq!(
        toughness.detail(),
        "More hit points. (Story: Vow of Dahak, Uncorrupted)"
    );
    assert!(toughness.open, "a story condition never blocks");
    assert_eq!(entry(&entries, "dodge").detail(), "+1 AC.");
}

#[test]
fn an_entry_of_values_names_what_they_are() {
    let mut data = world();
    data.feats
        .iter_mut()
        .find(|row| row.key == feat("weapon-focus"))
        .unwrap()
        .summary = "+1 to hit.".into();
    let engine = Engine::new(data);
    let entries = entries(&engine, &build("elf", &["fighter"]), &general_slot(1));
    let noun = |key: &str| entry(&entries, key).noun;
    assert_eq!(noun("weapon-focus"), Some("a weapon"));
    assert_eq!(noun("exotic-weapon-proficiency"), Some("an exotic weapon"));
    assert_eq!(noun("skill-focus"), Some("a skill"));
    assert_eq!(noun("spell-focus"), Some("a school"));
    assert_eq!(noun("armor-focus"), Some("an armour type"));
    assert_eq!(noun("toughness"), None);
    assert_eq!(
        entry(&entries, "weapon-focus").detail(),
        "Choose a weapon. +1 to hit."
    );
    assert_eq!(entry(&entries, "skill-focus").detail(), "Choose a skill.");
}

#[test]
fn a_kind_of_value_is_named_with_its_article() {
    use ParameterKind::*;
    let named = [Skill, Weapon, ExoticWeapon, SpellSchool, Energy, ArmourType]
        .map(|kind| (kind.noun(), kind.with_article()));
    assert_eq!(
        named,
        [
            ("skill", "a skill"),
            ("weapon", "a weapon"),
            ("exotic weapon", "an exotic weapon"),
            ("school", "a school"),
            ("energy type", "an energy type"),
            ("armour type", "an armour type"),
        ]
    );
}

#[test]
fn an_option_that_names_a_value_opens_its_values_too() {
    // The rogue talent Weapon Focus: its chosen feat's values.
    let engine = engine();
    let entries = entries(
        &engine,
        &build("elf", &["rogue", "rogue"]),
        &selection_slot(2, 0),
    );
    let focus = entry(&entries, "rogue/rogue-talents/weapon-focus");
    assert_eq!(focus.noun, Some("a weapon"));
    assert_eq!(focus.options.len(), 5);
    assert_eq!(focus.options[1].value, "dagger");
}
