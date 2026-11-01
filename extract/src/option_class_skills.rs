//! Ticket 37: options' class skills (the user's decision on ticket 34's first open question, 2026-11-01), applied to
//! finished data.
//!
//! - **Options:** a feature that only makes skills class skills, on an option's row ("Class Skill: Athletics" on a
//!   bloodline's, a mystery's, a domain's or an element's; a cavalier order's "Skills"), becomes that row's
//!   `class_skills`, so the skills are class skills while the row is held.
//! - **Archetypes:** the same feature on an archetype's row (Empyreal and Sylvan Sorcerer) only repeats a skill that
//!   the archetype's own list already holds, so it goes.
//! - **Left as features:** the four named just "Class Skill", which name no skill: the Arcane bloodline's and the Sage
//!   Sorcerer's ("one of Knowledge (World), Lore (Nature), Lore (Religion)": a choice), the Overwhelming Mage's and
//!   the Seeker's (their tables say only "Class Skill"); and the Sohei's "Skills" (Lore (Nature)), an archetype's
//!   whose class skills are the monk's, which lack it. They wait on the user (ticket 37's open questions).
//!
//! Each entry asserts the guide phrase that names its skills and the feature it replaces, so the rule fails loudly
//! on data it doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 37's one-off
//! rewrite, on `game-data/`.

use std::collections::BTreeSet;

use rules::{Feature, FeatureKey, GameData, SkillKey};

use crate::follow_ups::{cell_at, says};
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

/// The class-skill features that stay (ticket 37's open questions): the four named just "Class Skill", which name no
/// skill, and the Sohei's "Skills", which its archetype's list (the monk's) lacks.
pub const LEFT_AS_FEATURES: [&str; 5] = [
    "monk/sohei/skills",
    "sorcerer/bloodline/arcane/class-skill",
    "sorcerer/overwhelming-mage/class-skill",
    "sorcerer/sage-sorcerer/class-skill",
    "sorcerer/seeker-sorcerer/class-skill",
];

/// Applies every entry, in the order above.
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
    check_left(data);
    log
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
/// Skill" or "Skills" are the five left.
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
