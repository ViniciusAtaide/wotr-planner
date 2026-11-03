//! Ticket 64: ticket 60's decisions 2 and 3 (user, 2026-11-02), applied to finished data. Two archetypes pick, in
//! place of their class's own selection, from a selection of their own (ticket 60), so ticket 49's rule gave their
//! Extra feat no slot:
//!
//! - the **Vivisectionist**'s Medical Discovery (`alchemist/vivisectionist/medical-discovery`) stands where the
//!   alchemist's table has Discovery (2, 4, …, 18), and Extra Discovery ("You gain one additional discovery") may
//!   now give one;
//! - the **Hexcrafter**'s Hex Arcana (`magus/hexcrafter/hex-arcana`: "Besides the usual Magus Arcana abilities, a
//!   Hexcrafter can select a hex") stands where the magus's table has Magus Arcana but the first ("Loses Spell Recall
//!   and one Magus Arcana"), and Extra Arcana ("You gain one additional magus arcana") may now give one.
//!
//! Each feat's "Prerequisites" names the class feature ("Dicovery", so spelt; "Magus Arcana"), which the data holds as
//! "has taken one of its options". The archetype's feature is its version of it, so the archetype from its first slot
//! meets the prerequisite whatever it picked there (an `All` of the archetype and its class level), as a Sanctified
//! Slayer 8 meets Extra Slayer Talent's (ticket 60).
//!
//! Each rule asserts the guide phrases it rests on and what it finds in the data, so it fails loudly on data it doesn't
//! expect. [`apply`] runs on the finished data of an extraction (after ticket 63's `archetype_alignments`) and, through
//! ticket 64's one-off rewrite, on `libs/game-data/`.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, Class, ClassKey, FeatureSelectionKey, GameData, Prerequisite, effective_row,
};

use crate::follow_ups::says;
use crate::guide::Guide;
use crate::mercies_and_exploits::levels_in_tables;
use crate::option_levels::{Log, page};

/// One archetype an Extra feat now reaches.
pub struct Reach {
    /// The Extra feat.
    pub feat: &'static str,
    /// Its entry's "Prerequisites" line and the sentence that gives the pick.
    pub prerequisites: &'static str,
    pub gains: &'static str,
    /// The selections its extra pick may be of before.
    pub before: &'static [&'static str],
    /// The class and its archetype.
    pub class: &'static str,
    pub archetype: &'static str,
    /// The archetype's selection, its table's cell, and the class's own cell it stands for.
    pub selection: &'static str,
    pub cell: &'static str,
    pub class_cell: &'static str,
    /// The archetype's sections and the phrase each says.
    pub says: &'static [(&'static str, &'static str)],
    /// How many of the class's slots the archetype's table leaves out, from the first.
    pub loses: usize,
}

/// The two archetypes.
pub const REACHES: [Reach; 2] = [
    Reach {
        feat: "extra-discovery",
        prerequisites: "Prerequisites: Dicovery",
        gains: "You gain one additional discovery for which you must meet the prerequisites.",
        before: &["alchemist/discovery", "rogue/underground-chemist/discovery"],
        class: "alchemist",
        archetype: "alchemist/vivisectionist",
        selection: "alchemist/vivisectionist/medical-discovery",
        cell: "Medical Discovery",
        class_cell: "Discovery",
        says: &[(
            "medical-discovery",
            "The Vivisectionist is able to select certain Rogue Talents (including Advanced Talents from level 10) \
             and Alchemist discoveries",
        )],
        loses: 0,
    },
    Reach {
        feat: "extra-arcana",
        prerequisites: "Prerequisites: Magus Arcana",
        gains: "You gain one additional magus arcana for which you must meet the prerequisites.",
        before: &["magus/magus-arcana"],
        class: "magus",
        archetype: "magus/hexcrafter",
        selection: "magus/hexcrafter/hex-arcana",
        cell: "Hex Arcana",
        class_cell: "Magus Arcana",
        says: &[
            ("hexcrafter", "Loses Spell Recall and one Magus Arcana"),
            (
                "hex-arcana",
                "Besides the usual Magus Arcana abilities, a Hexcrafter can select a hex.",
            ),
        ],
        loses: 1,
    },
];

/// The levels (1 to 20) at which `class` taken with `archetype` (or none) has a slot of `selection` in its rows in
/// force.
pub fn slot_levels(class: &Class, archetype: Option<&str>, selection: &str) -> Vec<u8> {
    let archetype = archetype.map(|key| {
        class
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == key)
            .unwrap_or_else(|| panic!("{}: no archetype {key}", class.key))
    });
    let selection = FeatureSelectionKey::new(selection);
    (1..=20)
        .filter(|level| {
            effective_row(class, archetype, *level)
                .is_some_and(|row| row.selections.contains(&selection))
        })
        .collect()
}

/// "The archetype from class level `level`": its feature's version of the class feature, held from its first slot.
pub fn from_its_first_slot(class: &str, archetype: &str, level: u8) -> Prerequisite {
    Prerequisite::All(vec![
        Prerequisite::Archetype(ArchetypeKey::new(archetype)),
        Prerequisite::ClassLevel {
            class: ClassKey::new(class),
            level,
        },
    ])
}

/// Applies ticket 64's rule.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    for reach in &REACHES {
        apply_one(guide, data, reach, &mut log);
    }
    log
}

fn apply_one(guide: &Guide, data: &mut GameData, reach: &Reach, log: &mut Log) {
    let Reach {
        feat: feat_key,
        class,
        archetype,
        selection,
        ..
    } = *reach;
    says(guide, "feats", feat_key, reach.prerequisites);
    says(guide, "feats", feat_key, reach.gains);
    for (anchor, phrase) in reach.says {
        says(guide, class, anchor, phrase);
    }

    // The tables: the archetype's cell where the class's own table (the page's first section, anchor "") has the
    // class's, but the ones it loses; the data's rows in force agree.
    let class_page = page(guide, class);
    let anchor = archetype
        .strip_prefix(&format!("{class}/"))
        .expect("the archetype's key starts with its class's");
    let class_levels = levels_in_tables(class_page, reach.class_cell)
        .remove("")
        .unwrap_or_else(|| panic!("{class}: no {:?} in its table", reach.class_cell));
    let levels = class_levels[reach.loses..].to_vec();
    assert_eq!(
        levels_in_tables(class_page, reach.cell),
        BTreeMap::from([(anchor.to_owned(), levels.clone())]),
        "{class}: the tables' {:?}",
        reach.cell
    );
    let class_data = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == class)
        .unwrap_or_else(|| panic!("no class {class}"));
    assert_eq!(
        slot_levels(class_data, Some(archetype), selection),
        levels,
        "{archetype}: its {selection} slots"
    );
    let own = reach
        .before
        .iter()
        .find(|s| s.starts_with(&format!("{class}/")));
    let own = own.unwrap_or_else(|| panic!("{feat_key}: no selection of {class}"));
    assert_eq!(
        slot_levels(class_data, None, own),
        class_levels,
        "{class}: its {own} slots"
    );
    assert_eq!(
        slot_levels(class_data, Some(archetype), own),
        Vec::<u8>::new(),
        "{archetype}: a slot of {own}"
    );
    let first = levels[0];

    // The feat: its extra pick may be of the archetype's selection, and the archetype meets it from its first slot.
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == feat_key)
        .unwrap_or_else(|| panic!("no feat {feat_key}"));
    let before: BTreeSet<FeatureSelectionKey> = reach
        .before
        .iter()
        .copied()
        .map(FeatureSelectionKey::new)
        .collect();
    assert_eq!(feat.extra_pick_of, before, "{feat_key}'s extra pick");
    feat.extra_pick_of
        .insert(FeatureSelectionKey::new(selection));
    log.push(format!("{feat_key}: its extra pick may be of {selection}"));

    let Some(Prerequisite::Any(children)) = &mut feat.prerequisite else {
        panic!("{feat_key}: its prerequisite isn't an Any");
    };
    let reaches = from_its_first_slot(class, archetype, first);
    assert!(
        !children
            .iter()
            .any(|child| names_archetype(child, archetype)),
        "{feat_key}: names {archetype} already"
    );
    children.push(reaches);
    feat.prerequisite.as_mut().unwrap().canonicalize();
    log.push(format!(
        "{feat_key}: {archetype} from {class} level {first} meets it"
    ));
}

fn names_archetype(prerequisite: &Prerequisite, archetype: &str) -> bool {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => {
            children.iter().any(|c| names_archetype(c, archetype))
        }
        Prerequisite::Not(child) => names_archetype(child, archetype),
        Prerequisite::Archetype(key) => key.as_str() == archetype,
        _ => false,
    }
}
