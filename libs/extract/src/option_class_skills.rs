//! Ticket 37: options' class skills (the user's decision on ticket 34's first open question, 2026-11-01), built where
//! their owners are.
//!
//! - **Options:** a feature that only makes skills class skills, on an option's row ("Class Skill: Athletics" on a
//!   bloodline's; a cavalier order's "Skills"), becomes that row's `class_skills`, so the skills are class skills
//!   while the row is held. A mystery's, an element's and a domain's rows are built with their class skills
//!   ([`option_families`](crate::option_families)' provider), so they have no such feature.
//! - **Archetypes:** the same feature on an archetype's row (Empyreal and Sylvan Sorcerer) only repeats a skill that
//!   the archetype's own list already holds, so it goes.
//! - **Left as features:** the Arcane bloodline's and the Sage Sorcerer's "Class Skill" ("one of Knowledge (World),
//!   Lore (Nature), Lore (Religion)"): a choice, shown on the sheet (the user's decision on ticket 37's first open
//!   question).
//!
//! Ticket 38 then builds the user's decisions on ticket 37's open questions 2, 3 and 5 (2026-11-01;
//! skills questions take the recommended option):
//!
//! - **Overwhelming Mage and Seeker:** their "Class Skill" (their tables say only that) goes. Each holds its chosen
//!   standard bloodline's 1st row, so it holds that bloodline's class skill already.
//! - **Sohei:** its own class skill list, the monk's plus Lore (Nature) ("A Sohei gains Lore (Nature) as a class
//!   skill"), as the Empyreal and Sylvan Sorcerers have one; its "Skills" goes.
//! - **Animal domain:** its Lore (Nature) is on the domain's 4th row, the level of the sentence it follows ("At 4th
//!   level, you gain an Animal Companion. … Lore (Nature) becomes a Class Skill."): built with the domains.
//!
//! Each entry asserts the guide phrase that names its skills and the feature it stands for. The option entries are
//! on the bloodlines and orders before they're given to the class stage ([`option_rows`]), so no "Class Skill: …" or
//! "Skills" feature is made for them; the archetypes' cells are nothing ([`provide`]); [`check`] asserts what holds
//! those skills on the finished data.

use std::collections::BTreeSet;

use rules::{FeatureKey, FeatureSelectionKey, GameData, SelectionOption, Skill, SkillKey};

use crate::follow_ups::{cell_at, says};
use crate::guide::Guide;
use crate::option_levels::Family;
use crate::picks::{ArchetypeSkills, Pick, Picks, Rule};

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

/// The options' class skills, on a family before it's given to the class stage ([`crate::option_levels`]): each
/// option entry's feature is its options' rows' class skills, and is never a feature.
pub(crate) fn option_rows(guide: &Guide, family: &mut Family, skill_names: &[Skill]) {
    for entry in ENTRIES
        .iter()
        .filter(|entry| entry.holder == Holder::Option)
    {
        for (key, skills) in entry.features {
            let key = FeatureKey::new(*key);
            let Some(at) = family.features.iter().position(|f| f.key == key) else {
                continue;
            };
            entry.says(guide);
            let skills: BTreeSet<SkillKey> = skills.iter().map(|s| SkillKey::new(*s)).collect();
            let feature = family.features.remove(at);
            assert_eq!(
                feature.name,
                expected_name(skill_names, &key, &skills),
                "{key}"
            );
            let mut rows = 0;
            for option in &mut family.selection.options {
                for row in &mut option.levels {
                    let before = row.features.len();
                    row.features.retain(|feature| *feature != key);
                    if row.features.len() == before {
                        continue;
                    }
                    assert_eq!(before - row.features.len(), 1, "{key}: twice on a row");
                    assert!(
                        row.class_skills.is_disjoint(&skills),
                        "{key}: {} already",
                        option.key
                    );
                    row.class_skills.extend(skills.iter().cloned());
                    rows += 1;
                }
            }
            assert!(rows > 0, "{key}: on no option row");
        }
    }
}

impl Entry {
    fn says(&self, guide: &Guide) {
        match self.says {
            Says::Cell(level, cell) => cell_at(guide, self.page, self.anchor, level, cell),
            Says::Prose(phrase) => says(guide, self.page, self.anchor, phrase),
        }
    }
}

/// The cells that say a class skill an archetype holds otherwise: never features. The Empyreal's and the Sylvan
/// Sorcerer's own lists hold theirs; the Overwhelming Mage and the Seeker hold their chosen standard bloodline's 1st
/// row, so its class skill (ticket 38); the Sohei's own list is the monk's and Lore (Nature) (ticket 38). [`check`]
/// asserts what holds them.
pub(crate) fn provide(guide: &Guide, picks: &mut Picks) {
    let nothing = |picks: &mut Picks, table: &str, key: &str, levels: Option<Vec<u8>>| {
        picks.rule(Rule {
            feature: FeatureKey::new(key),
            table: Some(table.to_owned()),
            cell: None,
            levels,
            pick: Pick::Nothing,
        });
    };
    for entry in ENTRIES {
        let Holder::Archetype(archetype) = entry.holder else {
            continue;
        };
        entry.says(guide);
        for (key, _) in entry.features {
            nothing(picks, archetype, key, None);
        }
    }
    for (archetype, _) in BLOODLINE_ARCHETYPES {
        let anchor = archetype.strip_prefix("sorcerer/").unwrap();
        cell_at(guide, "sorcerer", anchor, "0", "Class Skill");
        says(guide, "sorcerer", anchor, "chooses a standard bloodline");
        nothing(
            picks,
            archetype,
            &format!("{archetype}/class-skill"),
            Some(vec![1]),
        );
    }
    says(
        guide,
        "monk",
        "skills",
        "A Sohei gains Lore (Nature) as a class skill.",
    );
    picks.archetype_skills(
        SOHEI,
        ArchetypeSkills::ClassAnd(BTreeSet::from([SkillKey::new(LORE_NATURE)])),
    );
    nothing(picks, SOHEI, &format!("{SOHEI}/skills"), None);
}

/// What holds the class skills [`provide`]'s cells said: each archetype entry's own list; the Overwhelming Mage's and
/// the Seeker's bloodline selection at 1, which doesn't skip the 1st row, whose every bloodline's 1st row holds a
/// class skill (or, for the Arcane, its own "Class Skill" choice). No feature stands for a fixed class skill any more.
/// Changes nothing.
pub(crate) fn check(data: &GameData) {
    let archetype = |key: &str| {
        data.classes
            .iter()
            .flat_map(|class| &class.archetypes)
            .find(|a| a.key.as_str() == key)
            .unwrap_or_else(|| panic!("no archetype {key}"))
    };
    for entry in ENTRIES {
        let Holder::Archetype(key) = entry.holder else {
            continue;
        };
        let list = archetype(key)
            .class_skills
            .as_ref()
            .unwrap_or_else(|| panic!("{key} keeps its class's skills"));
        for (_, skills) in entry.features {
            for skill in *skills {
                assert!(list.contains(&SkillKey::new(*skill)), "{key}: {list:?}");
            }
        }
    }
    for (key, selection) in BLOODLINE_ARCHETYPES {
        let selection_key = FeatureSelectionKey::new(selection);
        let first = archetype(key)
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap_or_else(|| panic!("{key}: no 1st row"));
        assert!(
            first.selections.contains(&selection_key),
            "{key}: {first:?}"
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
    }
    check_left(data);
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

const SOHEI: &str = "monk/sohei";
const LORE_NATURE: &str = "lore-nature";

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

/// "Class Skill: Athletics" (the skill's name), or "Skills" for a cavalier order's.
fn expected_name(skills: &[Skill], key: &FeatureKey, of: &BTreeSet<SkillKey>) -> String {
    if key.as_str().ends_with("/skills") {
        return "Skills".into();
    }
    let [skill] = of.iter().collect::<Vec<_>>()[..] else {
        panic!("{key}: one skill per \"Class Skill: …\" feature");
    };
    let name = &skills
        .iter()
        .find(|s| &s.key == skill)
        .unwrap_or_else(|| panic!("no skill {skill}"))
        .name;
    format!("Class Skill: {name}")
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
