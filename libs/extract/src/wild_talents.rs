//! Ticket 45: the kineticist's wild talents are a selection (ticket 43's decision 2, user, 2026-11-02), which the class
//! stage builds where it reads the kineticist's tables ([`provide`]).
//!
//! - **The pick** (`kineticist#wild-talents`): "At 2nd level and every subsequent even-numbered level, the
//!   Kineticist selects a Utility Wild Talent." Each "Wild Talents" cell (the base class's 2nd to 20th, and every
//!   archetype's table at each even level, as the base's) is a slot of the selection `kineticist/wild-talents`.
//! - **The talents** (the "Wild Talents" section, "Level N Talents"): the guide's 26, less the four defensive ones
//!   (`Type | Defensive`), which stay each element's 2nd-level grant. Each needs the kineticist level twice its own
//!   ("The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it": a
//!   `ClassLevel`), one of its elements (`Element | Fire, Water`: an `Any` of the elements held; Universal and All:
//!   none) and what its "Prerequisites" names: a defensive talent is that feature (Flame Shield: Searing Flesh), a
//!   talent that option (Healing Burst: Kinetic Healer), and "Fire blast" is the Fire element, which the talent's
//!   element asks already. The elements and their defensive talents are built with the other option families
//!   (ticket 31), so these prerequisites are set once they are ([`Picks::late`]).
//! - **Kineticist Bonus Feat** (Universal, level 1): "Select a bonus feat associated with an Elemental Focus. You do
//!   not need to meet the normal prerequisites for the feat you select." Its row 1 is a bonus feat slot from the
//!   held elements' lists (`OptionList(selection: "kineticist/element")`, so Expanded Element's element offers its
//!   list too), whose lists ignore prerequisites. Taken once: the guide doesn't say it repeats. The slot reads "Wild
//!   talent feat" (ticket 47, ticket 45's decision 3), not "Elemental Focus feat": the slot's own label; the
//!   Elemental Focus selection keeps its name.
//! - **Skill Focus** (Universal, level 1): "Identical to the feat": Skill Focus in a skill of the player's choosing
//!   (`chosen_feat`), as the rogue talent, once per skill (ticket 47, a user fact: "Skill Focus can be taken once per
//!   wild talent as long as you select another skill").
//! - **Expanded Defense** (Universal, level 4): "You unlock the Defensive Wild Talent for any element you selected
//!   with Expanded Element." Its row 1 holds a feature that grants Expanded Element's row 2 (`option_row`), which
//!   the engine gives a lists-only holding when the row is of the selection it's held through.
//!
//! - **Extra Wild Talent** (`feats#extra-wild-talent`): "Prerequisites: Wild Talents" names the pick; the feat needs
//!   kineticist level 2 instead, the level of the first pick, as Extra Slayer Talent needs slayer level 2. Its extra
//!   pick is ticket 49's.
//!
//! The extraction report's item `kineticist#kineticist-bonus-feat` ("a selection option can't grant a slot") is
//! settled ([`settled`]).
//!
//! Each rule asserts the guide phrase it rests on where the pick is read, and [`check`] what it finds in the finished
//! data, so it fails loudly on data it doesn't expect.

use std::collections::BTreeSet;

use rules::{
    ClassKey, DataAt, FeatKey, FeatSlot, FeatureKey, FeatureSelectionKey, GameData, OptionLevelRow,
    OptionRow, ParameterKind, Prerequisite, Repeat, SelectionOptionKey, effective_row,
};

use crate::follow_ups::says;
use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::option_levels::{empty_row, feature, option, page, selection};
use crate::picks::{Pick, Picks, Rule};

const KINETICIST: &str = "kineticist";
/// The feature the even rows held, which the pick replaces.
const WILD_TALENTS_FEATURE: &str = "kineticist/wild-talents";
/// The selection, keyed as the feature was (as `rogue/rogue-talents`).
const WILD_TALENTS: &str = "kineticist/wild-talents";
const ELEMENT: &str = "kineticist/element";
const EXPANDED_ELEMENT: &str = "kineticist/expanded-element";
const ELEMENTS: [&str; 4] = ["air", "earth", "fire", "water"];
/// "At 2nd level": the first pick.
const FIRST_PICK: u8 = 2;
/// The bonus-feat wild talent's slot, as the user names it (ticket 47).
const WILD_TALENT_FEAT: &str = "Wild talent feat";

/// The wild talents are a selection, picked where the tables have "Wild Talents".
pub fn provide(guide: &Guide, picks: &mut Picks) {
    let kineticist_page = page(guide, KINETICIST);
    says(
        guide,
        KINETICIST,
        "wild-talents",
        "At 2nd level and every subsequent even-numbered level, the Kineticist selects a Utility Wild Talent.",
    );
    let talents_section = talents_section(kineticist_page);
    assert!(
        talents_section.text().contains(
            "The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it."
        ),
        "kineticist#wild-talents: no level rule"
    );
    even_levels_in_every_table(kineticist_page);

    let talents = read_talents(talents_section);
    assert_eq!(talents.len(), 26, "the guide's wild talents");
    let defensive: Vec<&Talent> = talents.iter().filter(|t| t.defensive).collect();
    assert_eq!(defensive.len(), 4, "the defensive talents");
    for talent in &defensive {
        assert!(
            matches!(talent.elements.as_slice(), [_]),
            "{}: one element",
            talent.name
        );
        assert_eq!(talent.level, 1, "{}: a level-1 talent", talent.name);
    }

    let mut wild = selection(
        WILD_TALENTS,
        "Wild Talents",
        kineticist_page.guide("wild-talents"),
        &[KINETICIST],
    );
    let picked: Vec<&Talent> = talents.iter().filter(|t| !t.defensive).collect();
    let option_key = |name: &str| -> Option<SelectionOptionKey> {
        picked
            .iter()
            .find(|t| t.name == name)
            .map(|t| SelectionOptionKey::new(format!("{WILD_TALENTS}/{}", t.anchor)))
    };
    for talent in &picked {
        let key = format!("{WILD_TALENTS}/{}", talent.anchor);
        let mut all = vec![Prerequisite::ClassLevel {
            class: ClassKey::new(KINETICIST),
            level: 2 * talent.level,
        }];
        let element_of = |e: &String| {
            Prerequisite::SelectionOption(SelectionOptionKey::new(format!("{ELEMENT}/{e}")))
        };
        match talent.elements.as_slice() {
            [] => {}
            [one] => all.push(element_of(one)),
            several => all.push(Prerequisite::Any(several.iter().map(element_of).collect())),
        }
        if let Some(needs) = &talent.prerequisite {
            if let Some(blast) = needs.strip_suffix(" blast") {
                // "Fire blast": every Fire kineticist has it, so the talent's element asks it already.
                assert_eq!(
                    talent.elements,
                    [blast.to_lowercase()],
                    "{}: {needs:?} beside another element",
                    talent.name
                );
            } else if let Some(other) = option_key(needs) {
                all.push(Prerequisite::SelectionOption(other));
            } else {
                // A defensive talent: the 2nd-level feature of its element.
                let found = defensive
                    .iter()
                    .find(|d| &d.name == needs)
                    .unwrap_or_else(|| {
                        panic!("{}: no talent or defensive talent {needs:?}", talent.name)
                    });
                all.push(Prerequisite::Feature {
                    feature: defensive_key(&found.elements[0], &found.anchor),
                    count: 1,
                });
            }
        }
        // A Universal talent needs the level alone.
        let mut prerequisite = match all.len() {
            1 => all.pop().expect("the level"),
            _ => Prerequisite::All(all),
        };
        prerequisite.canonicalize();
        let mut chosen = option(
            &key,
            &talent.name,
            kineticist_page.guide(&talent.anchor),
            Vec::new(),
        );
        chosen.prerequisite = Some(prerequisite);
        match talent.anchor.as_str() {
            "kineticist-bonus-feat" => {
                assert!(
                    talent.text.contains(
                        "Select a bonus feat associated with an Elemental Focus. You do not need to meet the normal \
                         prerequisites for the feat you select."
                    ),
                    "{}: {:?}",
                    talent.name,
                    talent.text
                );
                assert!(talent.elements.is_empty(), "{}: Universal", talent.name);
                chosen.levels = vec![OptionLevelRow {
                    feat_slots: vec![FeatSlot::OptionList {
                        selection: FeatureSelectionKey::new(ELEMENT),
                        at_level: None,
                        label: Some(WILD_TALENT_FEAT.to_owned()),
                    }],
                    ..empty_row(1)
                }];
            }
            "skill-focus" => {
                assert!(
                    talent.text.contains("Identical to the feat."),
                    "{}",
                    talent.name
                );
                chosen.chosen_feat = Some(rules::FeatKey::new("skill-focus"));
                chosen.repeat = Repeat::PerValue;
            }
            "expanded-defense" => {
                assert!(
                    talent.text.contains(
                        "You unlock the Defensive Wild Talent for any element you selected with Expanded Element."
                    ),
                    "{}: {:?}",
                    talent.name,
                    talent.text
                );
                let unlock = FeatureKey::new(format!("{key}/defense"));
                chosen.levels = vec![OptionLevelRow {
                    features: vec![unlock.clone()],
                    ..empty_row(1)
                }];
                let mut defense =
                    feature(&unlock, &talent.name, kineticist_page.guide(&talent.anchor));
                defense.option_row = Some(OptionRow {
                    selection: FeatureSelectionKey::new(EXPANDED_ELEMENT),
                    level: 2,
                });
                picks.feature(KINETICIST, defense);
            }
            _ => {}
        }
        wild.options.push(chosen);
    }
    assert_eq!(wild.options.len(), 22, "the talents picked");
    picks.selection_naming_later_facts(KINETICIST, wild);
    picks.rule(Rule::every(
        WILD_TALENTS_FEATURE,
        Pick::Selection(FeatureSelectionKey::new(WILD_TALENTS)),
    ));

    // Extra Wild Talent's "Wild Talents" is the pick: the feat needs the kineticist level of the first one, as Extra
    // Slayer Talent needs the slayer's (ticket 44).
    says(
        guide,
        "feats",
        "extra-wild-talent",
        "Prerequisites: Wild Talents",
    );
    says(
        guide,
        "feats",
        "extra-wild-talent",
        "You gain one additional wild talent for which you must meet the prerequisites.",
    );
    picks.prerequisite(
        DataAt::Feat(FeatKey::new("extra-wild-talent")),
        "Wild Talents",
        Prerequisite::ClassLevel {
            class: ClassKey::new(KINETICIST),
            level: FIRST_PICK,
        },
    );
}

/// The elements' defensive talents and lists, the Skill Focus feat the talent takes, and the rows in force.
pub fn check(_guide: &Guide, data: &GameData) {
    let kineticist = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == KINETICIST)
        .expect("the kineticist");
    let element = kineticist
        .selections
        .iter()
        .find(|s| s.key.as_str() == ELEMENT)
        .unwrap_or_else(|| panic!("no {ELEMENT}"));
    let wild = kineticist
        .selections
        .iter()
        .find(|s| s.key.as_str() == WILD_TALENTS)
        .unwrap_or_else(|| panic!("no {WILD_TALENTS}"));
    // The defensive talents are each element's 2nd-level row, and what a talent needs of them.
    for one in ELEMENTS {
        let option = element
            .options
            .iter()
            .find(|o| o.key.as_str() == format!("{ELEMENT}/{one}"))
            .unwrap_or_else(|| panic!("no element {one}"));
        let row_2 = option
            .levels
            .iter()
            .find(|row| row.level == 2)
            .unwrap_or_else(|| panic!("{one} has no row 2"));
        let [talent] = row_2.features.as_slice() else {
            panic!("{one}'s row 2: {:?}", row_2.features)
        };
        assert!(
            talent.as_str().starts_with(&format!("{ELEMENT}/{one}/")),
            "{one}'s row 2: {talent}"
        );
        assert!(
            kineticist.features.iter().any(|f| &f.key == talent),
            "{talent}: not a feature"
        );
        // Each element's list ignores prerequisites, as the bonus feat does.
        let list = option
            .levels
            .iter()
            .find(|row| row.level == 1)
            .and_then(|row| row.feat_list.clone())
            .unwrap_or_else(|| panic!("{one} has no list at 1"));
        let list = kineticist
            .feat_lists
            .iter()
            .find(|l| l.key == list)
            .unwrap_or_else(|| panic!("no list {list}"));
        assert!(
            list.ignores_prerequisites,
            "{}: checks prerequisites",
            list.key
        );
    }
    for option in &wild.options {
        let needed = option.prerequisite.iter().flat_map(features_named);
        for needed in needed {
            assert!(
                kineticist.features.iter().any(|f| f.key == needed),
                "{}: needs {needed}, not a feature",
                option.key
            );
        }
    }
    // The Skill Focus talent takes the feat's value, once per skill.
    let skill_focus = data
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "skill-focus")
        .expect("Skill Focus");
    assert_eq!(
        (skill_focus.parameter, skill_focus.repeat),
        (Some(ParameterKind::Skill), Repeat::PerValue),
        "skill-focus: once per skill"
    );
    assert!(
        kineticist
            .selections
            .iter()
            .any(|s| s.key.as_str() == EXPANDED_ELEMENT && s.feat_lists_only),
        "{EXPANDED_ELEMENT} gives only feat lists"
    );

    let selection_key = FeatureSelectionKey::new(WILD_TALENTS);
    let class_levels: Vec<u8> = kineticist
        .levels
        .iter()
        .filter(|row| row.selections.contains(&selection_key))
        .map(|row| row.level)
        .collect();
    assert_eq!(
        class_levels,
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
        "the base class's rows"
    );
    assert_eq!(class_levels[0], FIRST_PICK, "the first pick");
    for archetype in &kineticist.archetypes {
        for level in 1..=20 {
            let row = effective_row(kineticist, Some(archetype), level).expect("a row");
            assert_eq!(
                row.selections.contains(&selection_key),
                level % 2 == 0,
                "{} {level}",
                archetype.key
            );
        }
    }
    let feat = data
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "extra-wild-talent")
        .expect("Extra Wild Talent");
    assert_eq!(
        feat.prerequisite,
        Some(Prerequisite::ClassLevel {
            class: ClassKey::new(KINETICIST),
            level: FIRST_PICK,
        }),
        "extra-wild-talent's prerequisite"
    );
}

/// The features a prerequisite names anywhere.
fn features_named(prerequisite: &Prerequisite) -> Vec<FeatureKey> {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children.iter().flat_map(features_named).collect(),
        Prerequisite::Not(child) => features_named(child),
        Prerequisite::Feature { feature, .. } => vec![feature.clone()],
        _ => Vec::new(),
    }
}

/// Whether an extraction's report item is one the wild talents settle: the bonus-feat wild talent, which can be picked
/// now and grants its slot.
pub fn settled(at: &str, text: &str) -> bool {
    at == "kineticist#kineticist-bonus-feat" && text.is_empty()
}

/// One wild talent as the guide lists it.
struct Talent {
    anchor: String,
    name: String,
    level: u8,
    /// Lowercase element names; empty for Universal and All.
    elements: Vec<String>,
    prerequisite: Option<String>,
    defensive: bool,
    /// Its table's description.
    text: String,
}

/// The "Wild Talents" section of talents by level (the page has a class feature of the same anchor first).
pub(crate) fn talents_section(page: &Page) -> &Section {
    let found: Vec<&Section> = page
        .root
        .walk()
        .into_iter()
        .filter(|s| s.anchor == "wild-talents" && !s.children.is_empty())
        .collect();
    let [section] = found.as_slice() else {
        panic!(
            "kineticist: {} wild talent sections with talents",
            found.len()
        )
    };
    section
}

fn read_talents(section: &Section) -> Vec<Talent> {
    let mut talents = Vec::new();
    for group in &section.children {
        let level: u8 = group
            .name
            .strip_prefix("Level ")
            .and_then(|rest| rest.strip_suffix(" Talents"))
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("kineticist#{}: not \"Level N Talents\"", group.anchor));
        assert_eq!(group.anchor, format!("level-{level}-talents"));
        for entry in &group.children {
            let rows: Vec<&Vec<String>> = entry.tables().flatten().collect();
            let cell = |label: &str| -> Option<String> {
                rows.iter()
                    .find(|row| row.first().map(String::as_str) == Some(label))
                    .map(|row| row.get(1).cloned().unwrap_or_default())
            };
            let element = cell("Element").unwrap_or_else(|| panic!("{}: no Element", entry.anchor));
            let elements = match element.as_str() {
                "Universal" | "All" => Vec::new(),
                list => list
                    .split(", ")
                    .map(|e| {
                        let e = e.to_lowercase();
                        assert!(
                            ELEMENTS.contains(&e.as_str()),
                            "{}: element {e:?}",
                            entry.anchor
                        );
                        e
                    })
                    .collect(),
            };
            let text = rows
                .iter()
                .filter(|row| row.len() == 1)
                .map(|row| row[0].clone())
                .collect::<Vec<_>>()
                .join(" ");
            talents.push(Talent {
                anchor: entry.anchor.clone(),
                name: entry.name.clone(),
                level,
                elements,
                prerequisite: cell("Prerequisites"),
                defensive: cell("Type").as_deref() == Some("Defensive"),
                text,
            });
        }
    }
    talents
}

/// The key of an element's defensive talent: `kineticist/element/fire/searing-flesh`.
fn defensive_key(element: &str, anchor: &str) -> FeatureKey {
    FeatureKey::new(format!("{ELEMENT}/{element}/{anchor}"))
}

/// Every kineticist table (the base's and each archetype's) has "Wild Talents" at each even level, and only there.
fn even_levels_in_every_table(page: &Page) {
    let tables: Vec<(&str, &Vec<Vec<String>>)> = page
        .root
        .walk()
        .into_iter()
        .flat_map(|s| s.tables().map(move |t| (s.anchor.as_str(), t)))
        .filter(|(_, table)| {
            table.first().is_some_and(|header| {
                header.first().map(String::as_str) == Some("Lvl")
                    && header.iter().any(|c| c == "Feat")
            })
        })
        .collect();
    let owners: BTreeSet<&str> = tables.iter().map(|(anchor, _)| *anchor).collect();
    assert_eq!(
        owners,
        BTreeSet::from([
            "",
            "blood-kineticist",
            "dark-elementalist",
            "elemental-engine",
            "kinetic-knight",
            "kinetic-sharpshooter",
            "overwhelming-soul",
            "psychokineticist",
        ]),
        "the kineticist's progression tables"
    );
    for (anchor, table) in tables {
        let levels: Vec<u8> = table
            .iter()
            .skip(1)
            .filter(|row| row.iter().skip(1).any(|cell| cell == "Wild Talents"))
            .filter_map(|row| row.first()?.parse().ok())
            .collect();
        assert_eq!(
            levels,
            [2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
            "kineticist#{anchor}'s Wild Talents"
        );
    }
}
