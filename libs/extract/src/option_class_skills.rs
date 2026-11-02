//! Ticket 37: options' class skills (the user's decision on ticket 34's first open question, 2026-11-01), applied to
//! finished data.
//!
//! - **Options:** a feature that only makes skills class skills, on an option's row ("Class Skill: Athletics" on a
//!   bloodline's, a mystery's, a domain's or an element's; a cavalier order's "Skills"), becomes that row's
//!   `class_skills`, so the skills are class skills while the row is held.
//! - **Archetypes:** the same feature on an archetype's row (Empyreal and Sylvan Sorcerer) only repeats a skill that
//!   the archetype's own list already holds, so it goes.
//! - **Left as features:** the Arcane bloodline's and the Sage Sorcerer's "Class Skill" ("one of Knowledge (World),
//!   Lore (Nature), Lore (Religion)"): a choice, shown on the sheet (the user's decision on ticket 37's first open
//!   question).
//!
//! Ticket 38's [`cleanup`] then builds the user's decisions on ticket 37's open questions 2, 3 and 5 (2026-11-01;
//! skills questions take the recommended option):
//!
//! - **Overwhelming Mage and Seeker:** their "Class Skill" (their tables say only that) goes. Each holds its chosen
//!   standard bloodline's 1st row, so it holds that bloodline's class skill already.
//! - **Sohei:** its own class skill list, the monk's plus Lore (Nature) ("A Sohei gains Lore (Nature) as a class
//!   skill"), as the Empyreal and Sylvan Sorcerers have one; its "Skills" goes.
//! - **Animal domain:** its Lore (Nature) moves from the domain's 1st row to its 4th, the level of the sentence it
//!   follows ("At 4th level, you gain an Animal Companion. … Lore (Nature) becomes a Class Skill."). The 1st row,
//!   left empty, goes.
//!
//! Each entry asserts the guide phrase that names its skills and the feature it replaces, so the rule fails loudly
//! on data it doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 37's one-off
//! rewrite, on `libs/game-data/`; ticket 38's one-off rewrite ran [`cleanup`] alone on `libs/game-data/`.

use std::collections::BTreeSet;

use rules::{
    Feature, FeatureKey, FeatureSelectionKey, GameData, OptionLevelRow, SelectionOption, SkillKey,
};

use crate::follow_ups::{archetype_mut, cell_at, class_mut, says};
use crate::guide::Guide;
use crate::option_levels::Log;

/// Where the guide names the skills.
enum Says {
    /// A cell of the section's table at a level ("Class Skill: Athletics" at the bloodline's level 0).
    Cell(&'static str, &'static str),
    /// A phrase in the section's text.
    Prose(&'static str),
}

/// Who holds the feature's rows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Holder {
    /// An option's rows: the skills move onto them.
    Option,
    /// An archetype's own rows, whose class skill list already holds the skills: the feature goes.
    Archetype(&'static str),
}

/// One guide statement and the features standing for it, each with its skills.
struct Entry {
    page: &'static str,
    anchor: &'static str,
    says: Says,
    holder: Holder,
    features: &'static [(&'static str, &'static [&'static str])],
}

const fn bloodline(
    anchor: &'static str,
    cell: &'static str,
    feature: &'static [(&'static str, &'static [&'static str])],
) -> Entry {
    Entry {
        page: "sorcerer",
        anchor,
        says: Says::Cell("0", cell),
        holder: Holder::Option,
        features: feature,
    }
}

const fn prose(
    page: &'static str,
    anchor: &'static str,
    phrase: &'static str,
    holder: Holder,
    features: &'static [(&'static str, &'static [&'static str])],
) -> Entry {
    Entry {
        page,
        anchor,
        says: Says::Prose(phrase),
        holder,
        features,
    }
}

const ENTRIES: &[Entry] = &[
    // The sorcerer's bloodlines: the cell at level 0 of each bloodline's table.
    bloodline(
        "abyssal",
        "Class Skill: Athletics",
        &[(
            "sorcerer/bloodline/abyssal/class-skill-athletics",
            &["athletics"],
        )],
    ),
    bloodline(
        "celestial",
        "Class Skill: Lore (Religion)",
        &[(
            "sorcerer/bloodline/celestial/class-skill-lore-religion",
            &["lore-religion"],
        )],
    ),
    bloodline(
        "draconic",
        "Class Skill: Perception",
        &[(
            "sorcerer/bloodline/draconic/class-skill-perception",
            &["perception"],
        )],
    ),
    bloodline(
        "elemental",
        "Class Skill: Mobility",
        &[(
            "sorcerer/bloodline/elemental/class-skill-mobility",
            &["mobility"],
        )],
    ),
    bloodline(
        "fey",
        "Class Skill: Lore (Nature)",
        &[(
            "sorcerer/bloodline/fey/class-skill-lore-nature",
            &["lore-nature"],
        )],
    ),
    bloodline(
        "infernal",
        "Class Skill: Knowledge (World)",
        &[(
            "sorcerer/bloodline/infernal/class-skill-knowledge-world",
            &["knowledge-world"],
        )],
    ),
    bloodline(
        "serpentine",
        "Class Skill: Stealth",
        &[(
            "sorcerer/bloodline/serpentine/class-skill-stealth",
            &["stealth"],
        )],
    ),
    bloodline(
        "undead",
        "Class Skill: Lore (Religion)",
        &[(
            "sorcerer/bloodline/undead/class-skill-lore-religion",
            &["lore-religion"],
        )],
    ),
    // The oracle's mysteries.
    prose(
        "oracle",
        "battle",
        "The Battle Mystery adds Perception to the list of class skills",
        Holder::Option,
        &[(
            "oracle/mystery/battle/class-skill-perception",
            &["perception"],
        )],
    ),
    prose(
        "oracle",
        "bones",
        "The Bones Mystery adds Stealth to the list of class skills",
        Holder::Option,
        &[("oracle/mystery/bones/class-skill-stealth", &["stealth"])],
    ),
    prose(
        "oracle",
        "flame",
        "The Flame Mystery adds Athletics and Mobility to the list of class skills",
        Holder::Option,
        &[
            ("oracle/mystery/flame/class-skill-athletics", &["athletics"]),
            ("oracle/mystery/flame/class-skill-mobility", &["mobility"]),
        ],
    ),
    prose(
        "oracle",
        "nature",
        "The Nature Mystery adds Athletics to the list of class skills",
        Holder::Option,
        &[(
            "oracle/mystery/nature/class-skill-athletics",
            &["athletics"],
        )],
    ),
    prose(
        "oracle",
        "stone",
        "The Stone Mystery adds Athletics to the list of class skills",
        Holder::Option,
        &[("oracle/mystery/stone/class-skill-athletics", &["athletics"])],
    ),
    prose(
        "oracle",
        "waves",
        "The Waves Mystery adds Athletics and Mobility to the list of class skills",
        Holder::Option,
        &[
            ("oracle/mystery/waves/class-skill-athletics", &["athletics"]),
            ("oracle/mystery/waves/class-skill-mobility", &["mobility"]),
        ],
    ),
    prose(
        "oracle",
        "wind",
        "The Wind Mystery adds Mobility and Stealth to the list of class skills",
        Holder::Option,
        &[
            ("oracle/mystery/wind/class-skill-mobility", &["mobility"]),
            ("oracle/mystery/wind/class-skill-stealth", &["stealth"]),
        ],
    ),
    // The domains (`cleric/domain`, shared by every domain holder).
    prose(
        "character-creation",
        "animal-domain",
        "Lore (Nature) becomes a Class Skill",
        Holder::Option,
        &[(
            "cleric/domain/animal/class-skill-lore-nature",
            &["lore-nature"],
        )],
    ),
    prose(
        "character-creation",
        "knowledge-domain",
        "All Knowledge and Lore skills become class skills",
        Holder::Option,
        &[
            (
                "cleric/domain/knowledge/class-skill-knowledge-arcana",
                &["knowledge-arcana"],
            ),
            (
                "cleric/domain/knowledge/class-skill-knowledge-world",
                &["knowledge-world"],
            ),
            (
                "cleric/domain/knowledge/class-skill-lore-nature",
                &["lore-nature"],
            ),
            (
                "cleric/domain/knowledge/class-skill-lore-religion",
                &["lore-religion"],
            ),
        ],
    ),
    prose(
        "character-creation",
        "trickery-domain",
        "Stealth and Trickery become Class Skills",
        Holder::Option,
        &[
            ("cleric/domain/trickery/class-skill-stealth", &["stealth"]),
            ("cleric/domain/trickery/class-skill-trickery", &["trickery"]),
        ],
    ),
    // The kineticist's elements.
    prose(
        "kineticist",
        "elemental-focus-air",
        "Additional class skills: Lore (Nature)",
        Holder::Option,
        &[(
            "kineticist/element/air/class-skill-lore-nature",
            &["lore-nature"],
        )],
    ),
    prose(
        "kineticist",
        "elemental-focus-earth",
        "Additional class skills: Athletics, Knowledge (World)",
        Holder::Option,
        &[
            (
                "kineticist/element/earth/class-skill-athletics",
                &["athletics"],
            ),
            (
                "kineticist/element/earth/class-skill-knowledge-world",
                &["knowledge-world"],
            ),
        ],
    ),
    prose(
        "kineticist",
        "elemental-focus-fire",
        "Additional class skills: Lore (Nature)",
        Holder::Option,
        &[(
            "kineticist/element/fire/class-skill-lore-nature",
            &["lore-nature"],
        )],
    ),
    prose(
        "kineticist",
        "elemental-focus-water",
        "Additional class skills: Athletics, Lore (Nature)",
        Holder::Option,
        &[
            (
                "kineticist/element/water/class-skill-athletics",
                &["athletics"],
            ),
            (
                "kineticist/element/water/class-skill-lore-nature",
                &["lore-nature"],
            ),
        ],
    ),
    // The cavalier's orders: a "Skills" feature each.
    prose(
        "cavalier",
        "skills-order-of-the-lion",
        "The Cavalier adds Knowledge (World) to her list of class skills.",
        Holder::Option,
        &[(
            "cavalier/order/order-of-the-lion/skills",
            &["knowledge-world"],
        )],
    ),
    prose(
        "cavalier",
        "skills-order-of-the-shroud",
        "The Cavalier adds Lore (Religion) to her list of class skills.",
        Holder::Option,
        &[(
            "cavalier/order/order-of-the-shroud/skills",
            &["lore-religion"],
        )],
    ),
    prose(
        "cavalier",
        "skills-order-of-the-star",
        "The Cavalier adds Lore (Religion) to her list of class skills.",
        Holder::Option,
        &[(
            "cavalier/order/order-of-the-star/skills",
            &["lore-religion"],
        )],
    ),
    prose(
        "cavalier",
        "skills-order-of-the-sword",
        "The Cavalier adds Lore (Religion) to her list of class skills.",
        Holder::Option,
        &[(
            "cavalier/order/order-of-the-sword/skills",
            &["lore-religion"],
        )],
    ),
    // Archetypes whose own class skill list already holds the skill.
    prose(
        "sorcerer",
        "empyreal-sorcerer",
        "Class Skill: Lore (Religion).",
        Holder::Archetype("sorcerer/empyreal-sorcerer"),
        &[(
            "sorcerer/empyreal-sorcerer/class-skill-lore-religion",
            &["lore-religion"],
        )],
    ),
    prose(
        "sorcerer",
        "sylvan-sorcerer",
        "Class Skill: Lore (Nature).",
        Holder::Archetype("sorcerer/sylvan-sorcerer"),
        &[(
            "sorcerer/sylvan-sorcerer/class-skill-lore-nature",
            &["lore-nature"],
        )],
    ),
];

/// The class-skill features that stay: the two whose skill is a choice (ticket 37's first open question).
pub const LEFT_AS_FEATURES: [&str; 2] = [
    "sorcerer/bloodline/arcane/class-skill",
    "sorcerer/sage-sorcerer/class-skill",
];

/// Applies every entry, in the order above, then ticket 38's [`cleanup`].
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    for entry in ENTRIES {
        match entry.says {
            Says::Cell(level, cell) => cell_at(guide, entry.page, entry.anchor, level, cell),
            Says::Prose(phrase) => says(guide, entry.page, entry.anchor, phrase),
        }
        for (key, skills) in entry.features {
            let key = FeatureKey::new(*key);
            let skills: BTreeSet<SkillKey> = skills.iter().map(|s| SkillKey::new(*s)).collect();
            let feature = take_feature(data, &key);
            assert_eq!(feature.name, expected_name(data, &key, &skills), "{key}");
            match entry.holder {
                Holder::Option => {
                    let rows = move_to_option_rows(data, &key, &skills);
                    assert!(!rows.is_empty(), "{key}: on no option row");
                    log.push(format!(
                        "{key}: {} class skills of {}",
                        names(&skills),
                        rows.join(", ")
                    ));
                }
                Holder::Archetype(archetype) => {
                    drop_from_archetype_rows(data, archetype, &key, &skills);
                    log.push(format!(
                        "{key}: gone, {archetype}'s own list holds {}",
                        names(&skills)
                    ));
                }
            }
        }
    }
    log.extend(cleanup(guide, data));
    log
}

/// Ticket 38: the Overwhelming Mage's and the Seeker's "Class Skill" go, the Sohei gets its own list, and the Animal
/// domain's Lore (Nature) moves to its 4th row. Runs on data whose class-skill features the entries have moved.
pub fn cleanup(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    for (archetype, selection) in BLOODLINE_ARCHETYPES {
        bloodline_archetype(guide, data, archetype, selection, &mut log);
    }
    sohei(guide, data, &mut log);
    animal_domain(guide, data, &mut log);
    check_left(data);
    log
}

/// The sorcerer archetypes that take a standard bloodline and hold its 1st row, with the selection that gives it.
const BLOODLINE_ARCHETYPES: [(&str, &str); 2] = [
    ("sorcerer/overwhelming-mage", "sorcerer/bloodline"),
    (
        "sorcerer/seeker-sorcerer",
        "sorcerer/seeker-sorcerer/bloodline",
    ),
];

/// The bloodline whose 1st row has a "Class Skill" choice instead of a class skill.
const ARCANE: &str = "sorcerer/bloodline/arcane";

/// Drops the archetype's "Class Skill" (its table's, at level 0), after checking that its 1st row grants a bloodline
/// selection that doesn't skip the bloodline's 1st row, and that every bloodline offered there holds a class skill
/// (or, for the Arcane, its own "Class Skill").
fn bloodline_archetype(
    guide: &Guide,
    data: &mut GameData,
    archetype: &str,
    selection: &str,
    log: &mut Log,
) {
    let anchor = archetype.strip_prefix("sorcerer/").unwrap();
    cell_at(guide, "sorcerer", anchor, "0", "Class Skill");
    says(guide, "sorcerer", anchor, "chooses a standard bloodline");
    let key = FeatureKey::new(format!("{archetype}/class-skill"));
    let selection_key = FeatureSelectionKey::new(selection);
    let found = archetype_mut(data, "sorcerer", archetype);
    let first = found
        .levels
        .iter()
        .find(|row| row.level == 1)
        .unwrap_or_else(|| panic!("{archetype}: no 1st row"));
    assert!(
        first.features.contains(&key) && first.selections.contains(&selection_key),
        "{archetype}: {first:?}"
    );
    let selection = data
        .classes
        .iter()
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .find(|s| s.key == selection_key)
        .unwrap_or_else(|| panic!("no selection {selection}"));
    assert!(!selection.skipped_levels.contains(&1), "{selection:?}");
    let offered: Vec<_> = selection
        .options
        .iter()
        .map(|o| o.key.clone())
        .chain(selection.shared_options.iter().cloned())
        .collect();
    assert!(offered.len() > 1, "{selection_key}: {offered:?}");
    let arcane_choice = FeatureKey::new(format!("{ARCANE}/class-skill"));
    for bloodline in &offered {
        let first = option(data, bloodline.as_str())
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap_or_else(|| panic!("{bloodline}: no 1st row"));
        if bloodline.as_str() == ARCANE {
            assert!(first.features.contains(&arcane_choice), "{first:?}");
        } else {
            assert!(
                !first.class_skills.is_empty(),
                "{bloodline}: no class skill"
            );
        }
    }
    let feature = take_feature(data, &key);
    assert_eq!(feature.name, "Class Skill", "{key}");
    let rows = &mut archetype_mut(data, "sorcerer", archetype).levels;
    let mut dropped = 0;
    for row in rows {
        let before = row.features.len();
        row.features.retain(|feature| feature != &key);
        dropped += before - row.features.len();
    }
    assert_eq!(dropped, 1, "{key}: on {dropped} rows");
    log.push(format!(
        "{key}: gone, {archetype} holds the 1st row of its bloodline ({selection_key}) and its class skill"
    ));
}

const SOHEI: &str = "monk/sohei";
const LORE_NATURE: &str = "lore-nature";

/// The Sohei's own list, the monk's and Lore (Nature); its "Skills" goes.
fn sohei(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "monk",
        "skills",
        "A Sohei gains Lore (Nature) as a class skill.",
    );
    let mut list = class_mut(data, "monk").class_skills.clone();
    assert!(list.insert(SkillKey::new(LORE_NATURE)), "the monk's list");
    let sohei = archetype_mut(data, "monk", SOHEI);
    assert_eq!(sohei.class_skills, None, "{SOHEI} keeps the monk's list");
    sohei.class_skills = Some(list);
    let key = FeatureKey::new(format!("{SOHEI}/skills"));
    let feature = take_feature(data, &key);
    assert_eq!(feature.name, "Skills", "{key}");
    let skills = BTreeSet::from([SkillKey::new(LORE_NATURE)]);
    drop_from_archetype_rows(data, SOHEI, &key, &skills);
    log.push(format!(
        "{key}: gone, {SOHEI}'s own list is the monk's and {LORE_NATURE}"
    ));
}

const ANIMAL: &str = "cleric/domain/animal";

/// The Animal domain's Lore (Nature) moves from its 1st row to its 4th (made if missing), with the companion; the
/// 1st row goes when nothing else is on it.
fn animal_domain(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "character-creation",
        "animal-domain",
        "At 4th level, you gain an Animal Companion. Your class level - 3 is used to determine your companion's \
         effective level. Lore (Nature) becomes a Class Skill.",
    );
    let skills = BTreeSet::from([SkillKey::new(LORE_NATURE)]);
    let rows = &mut option_mut(data, ANIMAL).levels;
    let first = rows
        .iter()
        .position(|row| row.level == 1)
        .unwrap_or_else(|| panic!("{ANIMAL}: no 1st row"));
    assert_eq!(rows[first].class_skills, skills, "{ANIMAL}@1");
    rows[first].class_skills.clear();
    let emptied = rows[first]
        == OptionLevelRow {
            level: 1,
            ..OptionLevelRow::default()
        };
    if emptied {
        rows.remove(first);
    }
    let fourth = match rows.iter().position(|row| row.level >= 4) {
        Some(index) if rows[index].level == 4 => index,
        found => {
            let index = found.unwrap_or(rows.len());
            rows.insert(
                index,
                OptionLevelRow {
                    level: 4,
                    ..OptionLevelRow::default()
                },
            );
            index
        }
    };
    assert!(rows[fourth].class_skills.is_empty(), "{ANIMAL}@4");
    rows[fourth].class_skills = skills;
    log.push(format!(
        "{ANIMAL}: {LORE_NATURE} class skills of {ANIMAL}@4, not @1{}",
        if emptied {
            " (the 1st row, now empty, goes)"
        } else {
            ""
        }
    ));
}

/// The option defined under `key` (once, in a class's or an archetype's selection).
fn option<'d>(data: &'d GameData, key: &str) -> &'d SelectionOption {
    let found: Vec<&SelectionOption> = data
        .classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .flat_map(|selection| &selection.options)
        .filter(|option| option.key.as_str() == key)
        .collect();
    assert_eq!(found.len(), 1, "{key}: defined {} times", found.len());
    found[0]
}

fn option_mut<'d>(data: &'d mut GameData, key: &str) -> &'d mut SelectionOption {
    option(data, key);
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter_mut()
                .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.selections))
        })
        .flat_map(|selection| &mut selection.options)
        .find(|option| option.key.as_str() == key)
        .unwrap()
}

fn names(skills: &BTreeSet<SkillKey>) -> String {
    skills
        .iter()
        .map(SkillKey::as_str)
        .collect::<Vec<_>>()
        .join(" and ")
}

/// "Class Skill: Athletics" (the skill's name), or "Skills" for a cavalier order's.
fn expected_name(data: &GameData, key: &FeatureKey, skills: &BTreeSet<SkillKey>) -> String {
    if key.as_str().ends_with("/skills") {
        return "Skills".into();
    }
    let [skill] = skills.iter().collect::<Vec<_>>()[..] else {
        panic!("{key}: one skill per \"Class Skill: …\" feature");
    };
    let name = &data
        .skills
        .iter()
        .find(|s| &s.key == skill)
        .unwrap_or_else(|| panic!("no skill {skill}"))
        .name;
    format!("Class Skill: {name}")
}

/// Removes the feature's one definition (a class's or an archetype's) and returns it.
fn take_feature(data: &mut GameData, key: &FeatureKey) -> Feature {
    let mut found = Vec::new();
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        let lists = std::iter::once(&mut class.features)
            .chain(class.archetypes.iter_mut().map(|a| &mut a.features));
        for features in lists {
            if let Some(index) = features.iter().position(|f| &f.key == key) {
                found.push(features.remove(index));
            }
        }
    }
    assert_eq!(found.len(), 1, "{key}: defined {} times", found.len());
    found.remove(0)
}

/// Moves the feature off every option row that grants it, onto the row's class skills; returns the rows, as
/// `option@level`.
fn move_to_option_rows(
    data: &mut GameData,
    key: &FeatureKey,
    skills: &BTreeSet<SkillKey>,
) -> Vec<String> {
    let mut rows = Vec::new();
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        assert!(
            class.levels.iter().all(|row| !row.features.contains(key)),
            "{key}: on {}'s level rows",
            class.key
        );
        let selections = class
            .selections
            .iter_mut()
            .chain(class.archetypes.iter_mut().flat_map(|a| {
                assert!(
                    a.levels.iter().all(|row| !row.features.contains(key)),
                    "{key}: on {}'s rows",
                    a.key
                );
                a.selections.iter_mut()
            }));
        for selection in selections {
            for option in &mut selection.options {
                for row in &mut option.levels {
                    let before = row.features.len();
                    row.features.retain(|feature| feature != key);
                    if row.features.len() == before {
                        continue;
                    }
                    assert_eq!(before - row.features.len(), 1, "{key}: twice on a row");
                    assert!(
                        row.class_skills.is_disjoint(skills),
                        "{key}: {} already",
                        option.key
                    );
                    row.class_skills.extend(skills.iter().cloned());
                    rows.push(format!("{}@{}", option.key, row.level));
                }
            }
        }
    }
    rows
}

/// Drops the feature from the archetype's rows, after checking its own class skill list holds the skills.
fn drop_from_archetype_rows(
    data: &mut GameData,
    archetype: &str,
    key: &FeatureKey,
    skills: &BTreeSet<SkillKey>,
) {
    let found = data
        .classes
        .iter_mut()
        .flat_map(|class| class.archetypes.iter_mut())
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no archetype {archetype}"));
    let list = found
        .class_skills
        .as_ref()
        .unwrap_or_else(|| panic!("{archetype} keeps its class's skills"));
    assert!(skills.is_subset(list), "{archetype}: {list:?}");
    let mut dropped = 0;
    for row in &mut found.levels {
        let before = row.features.len();
        row.features.retain(|feature| feature != key);
        dropped += before - row.features.len();
    }
    assert_eq!(dropped, 1, "{key}: on {dropped} of {archetype}'s rows");
}

/// No feature stands for a fixed class skill any more: none is named "Class Skill: …", and the ones named "Class
/// Skill" or "Skills" are the two left.
fn check_left(data: &GameData) {
    let mut left = Vec::new();
    for class in data.classes.iter().chain(&data.prestige_classes) {
        let features = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features));
        for feature in features {
            assert!(
                !feature.name.starts_with("Class Skill:"),
                "{} is left",
                feature.key
            );
            if feature.name == "Class Skill" || feature.name == "Skills" {
                left.push(feature.key.as_str());
            }
        }
    }
    left.sort_unstable();
    assert_eq!(left, LEFT_AS_FEATURES);
}
