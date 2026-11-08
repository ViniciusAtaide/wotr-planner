//! The test world's classes and prestige classes, and the rows they're built from.

mod arcane;
mod divine;
mod kineticist;
mod martial;
mod prestige;
mod rogues;
mod sorcerer;

pub(super) use self::{
    arcane::{bard, witch, wizard},
    divine::{cleric, druid},
    kineticist::kineticist,
    martial::{fighter, monk, ranger, warpriest},
    prestige::{arcane_trickster, dragon_disciple, eldritch_knight, mystic_theurge, winter_witch},
    rogues::{rogue, slayer},
    sorcerer::sorcerer,
};

use std::collections::BTreeSet;

use crate::*;

use crate::test_world::world::guide;

pub(in crate::test_world::world) fn row(level: u8) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: vec![],
        feats: BTreeSet::new(),
        feats_for: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
    }
}

/// An archetype's row at one level: no grants and the base class's spell levels until filled in.
fn archetype_row(level: u8) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level,
        spell_levels: None,
        features: vec![],
        feats: BTreeSet::new(),
        feats_for: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
    }
}

/// An archetype with nothing of its own yet.
pub(in crate::test_world::world) fn archetype(key: &str) -> Archetype {
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: guide(key),
        prerequisite: None,
        skill_points: None,
        class_skills: None,
        spontaneous: None,
        fort: None,
        reflex: None,
        will: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels: vec![],
    }
}
