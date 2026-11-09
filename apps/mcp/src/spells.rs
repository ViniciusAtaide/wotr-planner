//! The spell catalogue (ticket 121) as `search` and `describe` give it: a spell's facts and the lists it's on, a
//! spell list's spells by level, and the lists a class, an archetype or a path casts from.

use std::collections::{BTreeMap, HashMap};

use rules::{Engine, GameData, Spell, SpellCaster, SpellList, SpellListOnly, SpellSchool};

use crate::shapes::{Caster, Kind, ListedSpell, Named, OnList, SpellFacts, SpellLevelRow};

/// A spell's school as the guide prints it: "Universalist" for none (Dismiss Spell).
pub(crate) fn school_name(school: Option<SpellSchool>) -> &'static str {
    school.map_or("Universalist", SpellSchool::name)
}

/// A spell's descriptors as the guide prints them, in alphabetical order.
pub(crate) fn descriptor_names(spell: &Spell) -> Vec<String> {
    spell
        .descriptors
        .iter()
        .map(|descriptor| descriptor.name().to_owned())
        .collect()
}

/// A spell list's category in `search`: by its owner's kind.
pub(crate) fn list_category(owner: &SpellCaster) -> &'static str {
    match owner {
        SpellCaster::Class(_) => "Class list",
        SpellCaster::Archetype(_) => "Archetype list",
        SpellCaster::Path(_) => "Mythic spellbook",
    }
}

/// A spell's facts, from its block.
pub(crate) fn facts(spell: &Spell) -> SpellFacts {
    SpellFacts {
        school: school_name(spell.school).to_owned(),
        descriptors: descriptor_names(spell),
        target: spell.target.clone(),
        duration: spell.duration.clone(),
        saving_throw: spell.saving_throw.clone(),
        spell_resistance: spell.spell_resistance,
        casting_time: spell.casting_time.name().to_owned(),
        components: spell.components.clone(),
    }
}

/// Every list a spell is on, at each level it has there, in the lists' order.
pub(crate) fn lists_of(engine: &Engine, data: &GameData, spell: &Spell) -> Vec<OnList> {
    data.spell_lists
        .iter()
        .flat_map(|list| {
            list.entries
                .iter()
                .filter(|entry| entry.spell == spell.key)
                .map(move |entry| OnList {
                    key: list.key.as_str().to_owned(),
                    name: list.name.clone(),
                    level: entry.level,
                    owner: caster(engine, &list.owner),
                    used_by: list
                        .used_by
                        .iter()
                        .map(|user| caster(engine, user))
                        .collect(),
                    only: entry.only.as_ref().map(|only| only_text(engine, only)),
                })
        })
        .collect()
}

/// A list's spells by level, lowest first, each level's by name.
pub(crate) fn spells_by_level(
    engine: &Engine,
    data: &GameData,
    list: &SpellList,
) -> Vec<SpellLevelRow> {
    let spells: HashMap<&str, &Spell> = data
        .spells
        .iter()
        .map(|spell| (spell.key.as_str(), spell))
        .collect();
    let mut levels: BTreeMap<u8, Vec<ListedSpell>> = BTreeMap::new();
    for entry in &list.entries {
        let spell = spells
            .get(entry.spell.as_str())
            .expect("a checked list names known spells");
        levels.entry(entry.level).or_default().push(ListedSpell {
            key: spell.key.as_str().to_owned(),
            name: spell.name.clone(),
            school: school_name(spell.school).to_owned(),
            descriptors: descriptor_names(spell),
            only: entry.only.as_ref().map(|only| only_text(engine, only)),
        });
    }
    levels
        .into_iter()
        .map(|(level, mut spells)| {
            spells.sort_by(|a, b| (&a.name, &a.key).cmp(&(&b.name, &b.key)));
            SpellLevelRow { level, spells }
        })
        .collect()
}

/// The lists a caster owns or uses, in the lists' order.
pub(crate) fn lists_for(data: &GameData, caster: &SpellCaster) -> Vec<Named> {
    data.spell_lists
        .iter()
        .filter(|list| &list.owner == caster || list.used_by.contains(caster))
        .map(|list| Named {
            key: list.key.as_str().to_owned(),
            name: list.name.clone(),
        })
        .collect()
}

/// A class, an archetype or a path, with its kind and name.
pub(crate) fn caster(engine: &Engine, caster: &SpellCaster) -> Caster {
    let (kind, name) = match caster {
        SpellCaster::Class(key) => (Kind::Class, engine.class_name(key)),
        SpellCaster::Archetype(key) => (Kind::Archetype, engine.archetype_name(key)),
        SpellCaster::Path(key) => (Kind::Path, engine.path_name(key)),
    };
    Caster {
        kind,
        key: caster.key().to_owned(),
        name: name.to_owned(),
    }
}

/// A footnote as "Flame Dancer only", or "Uncorrupted only" for a Gold Dragon story branch.
fn only_text(engine: &Engine, only: &SpellListOnly) -> String {
    match only {
        SpellListOnly::Archetype(key) => format!("{} only", engine.archetype_name(key)),
        SpellListOnly::Story(story) => format!("{} only", story.name()),
    }
}
