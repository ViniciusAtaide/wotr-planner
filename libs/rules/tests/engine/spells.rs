//! The spell catalogue in `GameData::check` (ticket 121): spells and spell lists are reference data, so the check
//! sees to their keys, the casters and spells a list names, and its levels.

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, CastingTime, ClassKey, DataAt, DataIssue, DataProblem, GameData, GuideRef,
    MythicPathKey, Spell, SpellCaster, SpellDescriptor, SpellKey, SpellList, SpellListEntry,
    SpellListKey, SpellListOnly, SpellSchool, Story,
};

use crate::support::*;

fn issues(data: &GameData) -> Vec<(DataAt, DataIssue)> {
    data.check()
        .into_iter()
        .map(|DataProblem { at, issue }| (at, issue))
        .collect()
}

fn spell(key: &str) -> Spell {
    Spell {
        key: SpellKey::new(key),
        name: key.to_owned(),
        school: Some(SpellSchool::Enchantment),
        guide: GuideRef::new(format!("wizard-spells#{key}")),
        summary: String::new(),
        components: None,
        target: "One enemy creature within medium range".to_owned(),
        duration: Some("1 round / level".to_owned()),
        saving_throw: "Will negates".to_owned(),
        spell_resistance: true,
        descriptors: BTreeSet::from([SpellDescriptor::MindAffecting, SpellDescriptor::Compulsion]),
        casting_time: CastingTime::Standard,
    }
}

fn entry(level: u8, spell: &str) -> SpellListEntry {
    SpellListEntry {
        level,
        spell: SpellKey::new(spell),
        only: None,
    }
}

fn list(key: &str, owner: SpellCaster, entries: Vec<SpellListEntry>) -> SpellList {
    SpellList {
        key: SpellListKey::new(key),
        name: key.to_owned(),
        guide: GuideRef::new(key),
        owner,
        used_by: BTreeSet::new(),
        entries: entries.into_iter().collect(),
    }
}

fn class(key: &str) -> SpellCaster {
    SpellCaster::Class(ClassKey::new(key))
}

fn path(key: &str) -> SpellCaster {
    SpellCaster::Path(MythicPathKey::new(key))
}

fn at_list(key: &str) -> DataAt {
    DataAt::SpellList(SpellListKey::new(key))
}

fn at_spell(key: &str) -> DataAt {
    DataAt::Spell(SpellKey::new(key))
}

/// The test world with two spells: Hold Monster on the wizard's list (also the sorcerer's) at 5, and Dismiss Spell
/// (Universalist) at 0, and in the Aeon spellbook at 10.
fn world_with_spells() -> GameData {
    let mut data = world();
    data.spells = vec![
        spell("hold-monster"),
        Spell {
            school: None,
            ..spell("dismiss-spell")
        },
    ];
    let mut wizard = list(
        "wizard",
        class("wizard"),
        vec![entry(5, "hold-monster"), entry(0, "dismiss-spell")],
    );
    wizard.used_by.insert(class("sorcerer"));
    data.spell_lists = vec![
        wizard,
        list("aeon", path("aeon"), vec![entry(10, "dismiss-spell")]),
    ];
    data
}

#[test]
fn spells_on_lists_of_classes_and_paths_have_no_problems() {
    assert_eq!(issues(&world_with_spells()), []);
}

#[test]
fn a_spell_key_and_a_list_key_are_each_used_once() {
    let mut data = world_with_spells();
    data.spells.push(spell("hold-monster"));
    let aeon = data.spell_lists[1].clone();
    data.spell_lists.push(aeon);
    assert_eq!(
        issues(&data),
        [
            (at_spell("hold-monster"), DataIssue::DuplicateKey),
            (at_list("aeon"), DataIssue::DuplicateKey),
        ]
    );
}

#[test]
fn every_spell_and_caster_a_list_names_is_one_the_data_has() {
    let mut data = world_with_spells();
    let wizard = &mut data.spell_lists[0];
    wizard.entries.insert(entry(3, "no-such-spell"));
    wizard.used_by.insert(class("no-such-class"));
    wizard
        .used_by
        .insert(SpellCaster::Archetype(ArchetypeKey::new("wizard/no-such")));
    data.spell_lists
        .push(list("no-such-path", path("no-such-path"), vec![]));
    let unknown = |list: &str, missing| (at_list(list), DataIssue::UnknownKey(missing));
    assert_eq!(
        issues(&data),
        [
            unknown(
                "no-such-path",
                DataAt::MythicPath(MythicPathKey::new("no-such-path"))
            ),
            unknown("wizard", DataAt::Class(ClassKey::new("no-such-class"))),
            unknown(
                "wizard",
                DataAt::Archetype(ArchetypeKey::new("wizard/no-such"))
            ),
            unknown("wizard", at_spell("no-such-spell")),
        ]
    );
}

#[test]
fn a_class_list_runs_to_level_9_and_a_mythic_spellbook_to_10() {
    let mut data = world_with_spells();
    data.spell_lists[0]
        .entries
        .insert(entry(10, "hold-monster"));
    data.spell_lists[1]
        .entries
        .insert(entry(11, "hold-monster"));
    assert_eq!(
        issues(&data),
        [
            (
                at_list("aeon"),
                DataIssue::SpellLevelAboveList { level: 11, max: 10 }
            ),
            (
                at_list("wizard"),
                DataIssue::SpellLevelAboveList { level: 10, max: 9 }
            ),
        ]
    );
}

#[test]
fn every_spell_is_on_a_list() {
    let mut data = world_with_spells();
    data.spells.push(spell("orphan"));
    assert_eq!(
        issues(&data),
        [(at_spell("orphan"), DataIssue::SpellOnNoList)]
    );
}

#[test]
fn a_list_is_keyed_by_its_owner_or_under_it() {
    let mut data = world_with_spells();
    data.spell_lists.push(list(
        "aeon/merged",
        path("aeon"),
        vec![entry(3, "hold-monster")],
    ));
    data.spell_lists.push(list(
        "arcane",
        class("wizard"),
        vec![entry(3, "hold-monster")],
    ));
    assert_eq!(
        issues(&data),
        [(at_list("arcane"), DataIssue::KeyOutsideItsOwner)]
    );
}

#[test]
fn an_entry_is_only_for_an_archetype_of_a_class_on_the_list_or_a_paths_story_branch() {
    let mut data = world_with_spells();
    let only = |level, spell: &str, only| SpellListEntry {
        only: Some(only),
        ..entry(level, spell)
    };
    let cleric = SpellListOnly::Archetype(ArchetypeKey::new("cleric/separatist"));
    let armiger = SpellListOnly::Archetype(ArchetypeKey::new("fighter/armiger"));
    let mut cleric_list = list("cleric", class("cleric"), vec![]);
    cleric_list
        .entries
        .insert(only(1, "hold-monster", cleric.clone()));
    cleric_list.entries.insert(only(2, "hold-monster", armiger));
    cleric_list.entries.insert(only(
        3,
        "hold-monster",
        SpellListOnly::Story(Story::Uncorrupted),
    ));
    data.spell_lists.push(cleric_list);
    // A class that casts from the list without owning it: its archetypes may have entries too.
    data.spell_lists[0].used_by.insert(class("cleric"));
    data.spell_lists[0]
        .entries
        .insert(only(4, "hold-monster", cleric));
    data.spell_lists[1].entries.insert(only(
        5,
        "hold-monster",
        SpellListOnly::Story(Story::VowOfDahak),
    ));
    let outside = (
        at_list("cleric"),
        DataIssue::OnlyOutsideTheList(SpellKey::new("hold-monster")),
    );
    assert_eq!(issues(&data), [outside.clone(), outside]);
}
