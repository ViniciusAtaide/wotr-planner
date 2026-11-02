//! Ticket 45: the kineticist's wild talents become a selection (ticket 43's decision 2, user, 2026-11-02),
//! applied to finished data.
//!
//! - **The pick** (`kineticist#wild-talents`): "At 2nd level and every subsequent even-numbered level, the
//!   Kineticist selects a Utility Wild Talent." The "Wild Talents" feature on those rows (the base class's 2nd to
//!   20th; the Kinetic Knight's 4th and the Blood Kineticist's 20th, the archetype rows that differ from the base)
//!   becomes a slot of the selection `kineticist/wild-talents`, and the feature goes. Every archetype's table has
//!   "Wild Talents" at each even level, as the base's: none changes the pick.
//! - **The talents** (the "Wild Talents" section, "Level N Talents"): the guide's 26, less the four defensive ones
//!   (`Type | Defensive`), which stay each element's 2nd-level grant. Each needs the kineticist level twice its own
//!   ("The Kineticist's class level must be at least twice the level of the Wild Talent to unlock it": a
//!   `ClassLevel`), one of its elements (`Element | Fire, Water`: an `Any` of the elements held; Universal and All:
//!   none) and what its "Prerequisites" names: a defensive talent is that feature (Flame Shield: Searing Flesh), a
//!   talent that option (Healing Burst: Kinetic Healer), and "Fire blast" is the Fire element, which the talent's
//!   element asks already.
//! - **Kineticist Bonus Feat** (Universal, level 1): "Select a bonus feat associated with an Elemental Focus. You do
//!   not need to meet the normal prerequisites for the feat you select." Its row 1 is a bonus feat slot from the
//!   held elements' lists (`OptionList(selection: "kineticist/element")`, so Expanded Element's element offers its
//!   list too), whose lists ignore prerequisites. Taken once: the guide doesn't say it repeats.
//! - **Skill Focus** (Universal, level 1): "Identical to the feat": Skill Focus in a skill of the player's choosing
//!   (`chosen_feat`), as the rogue talent.
//! - **Expanded Defense** (Universal, level 4): "You unlock the Defensive Wild Talent for any element you selected
//!   with Expanded Element." Its row 1 holds a feature that grants Expanded Element's row 2 (`option_row`), which
//!   the engine gives a lists-only holding when the row is of the selection it's held through.
//!
//! - **Extra Wild Talent** (`feats#extra-wild-talent`): "Prerequisites: Wild Talents" named the feature that goes; it
//!   needs kineticist level 2 instead, the level of the first pick, as Extra Slayer Talent needs slayer level 2.
//!   Its extra pick isn't modelled, as no Extra feat's is.
//!
//! The extraction report's item `kineticist#kineticist-bonus-feat` ("a selection option can't grant a slot") is
//! settled ([`settled`]).
//!
//! Each rule asserts the guide phrase it rests on and what it finds in the data, so it fails loudly on data it
//! doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 45's one-off rewrite,
//! on `libs/game-data/`.

use std::collections::BTreeSet;

use rules::{
    ClassKey, FeatSlot, FeatureKey, FeatureSelectionKey, GameData, OptionLevelRow, OptionRow,
    ParameterKind, Prerequisite, SelectionOptionKey,
};

use crate::follow_ups::{class_mut, says};
use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::option_levels::{Log, empty_row, feature, option, page, selection};

const KINETICIST: &str = "kineticist";
/// The feature the even rows held, which the pick replaces.
const WILD_TALENTS_FEATURE: &str = "kineticist/wild-talents";
/// The selection, keyed as the feature was (as `rogue/rogue-talents`).
const WILD_TALENTS: &str = "kineticist/wild-talents";
const ELEMENT: &str = "kineticist/element";
const EXPANDED_ELEMENT: &str = "kineticist/expanded-element";
const ELEMENTS: [&str; 4] = ["air", "earth", "fire", "water"];

/// Makes the wild talents a selection, with its options and slots.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
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

    let kineticist = class_mut(data, KINETICIST);
    // The defensive talents are each element's 2nd-level row already.
    let element = kineticist
        .selections
        .iter()
        .find(|s| s.key.as_str() == ELEMENT)
        .unwrap_or_else(|| panic!("no {ELEMENT}"));
    for talent in &defensive {
        let [one] = talent.elements.as_slice() else {
            panic!("{}: one element", talent.name)
        };
        let key = defensive_key(one, &talent.anchor);
        let row_2 = element
            .options
            .iter()
            .find(|o| o.key.as_str() == format!("{ELEMENT}/{one}"))
            .and_then(|o| o.levels.iter().find(|row| row.level == 2))
            .unwrap_or_else(|| panic!("{one} has no row 2"));
        assert_eq!(row_2.features, std::slice::from_ref(&key), "{one}'s row 2");
        assert_eq!(talent.level, 1, "{}: a level-1 talent", talent.name);
    }
    // Each element's list ignores prerequisites, as the bonus feat does.
    for one in ELEMENTS {
        let list = element
            .options
            .iter()
            .find(|o| o.key.as_str() == format!("{ELEMENT}/{one}"))
            .and_then(|o| o.levels.iter().find(|row| row.level == 1))
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
    assert!(
        kineticist
            .selections
            .iter()
            .all(|s| s.key.as_str() != WILD_TALENTS),
        "{WILD_TALENTS}: a selection already"
    );
    let feature_names: Vec<(FeatureKey, String)> = kineticist
        .features
        .iter()
        .map(|f| (f.key.clone(), f.name.clone()))
        .collect();

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
    let mut expanded_defense = None;
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
                let (feature, _) = feature_names
                    .iter()
                    .find(|(key, name)| {
                        name == needs && key.as_str().starts_with(&format!("{ELEMENT}/"))
                    })
                    .unwrap_or_else(|| {
                        panic!("{}: no talent or defensive talent {needs:?}", talent.name)
                    });
                all.push(Prerequisite::Feature {
                    feature: feature.clone(),
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
                        label: None,
                    }],
                    ..empty_row(1)
                }];
                log.push(format!("{key}: a bonus feat from the held elements' lists"));
            }
            "skill-focus" => {
                assert!(
                    talent.text.contains("Identical to the feat."),
                    "{}",
                    talent.name
                );
                chosen.chosen_feat = Some(rules::FeatKey::new("skill-focus"));
                log.push(format!(
                    "{key}: Skill Focus in a skill of the player's choosing"
                ));
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
                expanded_defense = Some(defense);
                log.push(format!(
                    "{key}: Expanded Element's row 2 (the defensive talents)"
                ));
            }
            _ => {}
        }
        log.push(format!(
            "{key}: level {}, {}",
            talent.level,
            match talent.elements.as_slice() {
                [] => "any element".to_owned(),
                elements => elements.join(" or "),
            }
        ));
        wild.options.push(chosen);
    }
    assert_eq!(wild.options.len(), 22, "the talents picked");

    // The Skill Focus talent takes the feat's value.
    assert_eq!(
        data.feats
            .iter()
            .find(|f| f.key.as_str() == "skill-focus")
            .and_then(|f| f.parameter),
        Some(ParameterKind::Skill),
        "skill-focus takes a skill"
    );
    let kineticist = class_mut(data, KINETICIST);
    assert!(
        kineticist
            .selections
            .iter()
            .any(|s| s.key.as_str() == EXPANDED_ELEMENT && s.feat_lists_only),
        "{EXPANDED_ELEMENT} gives only feat lists"
    );
    kineticist
        .features
        .push(expanded_defense.expect("Expanded Defense"));
    kineticist.selections.push(wild);

    // The feature goes; each row that held it picks a talent instead.
    let before = kineticist.features.len();
    kineticist
        .features
        .retain(|f| f.key.as_str() != WILD_TALENTS_FEATURE);
    assert_eq!(
        kineticist.features.len(),
        before - 1,
        "{WILD_TALENTS_FEATURE}"
    );
    let feature_key = FeatureKey::new(WILD_TALENTS_FEATURE);
    let selection_key = FeatureSelectionKey::new(WILD_TALENTS);
    let mut class_levels = Vec::new();
    for row in &mut kineticist.levels {
        if take(&mut row.features, &feature_key) {
            row.selections.push(selection_key.clone());
            class_levels.push(row.level);
        }
    }
    assert_eq!(
        class_levels,
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
        "the base class's rows"
    );
    log.push(format!("{KINETICIST} {class_levels:?}: a wild talent"));
    let mut archetype_rows = Vec::new();
    for archetype in &mut kineticist.archetypes {
        for row in &mut archetype.levels {
            let held = take(&mut row.features, &feature_key);
            assert_eq!(held, row.level % 2 == 0, "{} {}", archetype.key, row.level);
            if held {
                row.selections.push(selection_key.clone());
                archetype_rows.push(format!("{} {}", archetype.key, row.level));
            }
        }
    }
    assert_eq!(
        archetype_rows,
        [
            "kineticist/blood-kineticist 20",
            "kineticist/kinetic-knight 4"
        ],
        "the archetype rows that differ from the base's"
    );
    for row in archetype_rows {
        log.push(format!("{row}: a wild talent"));
    }
    extra_wild_talent(guide, data, class_levels[0], &mut log);
    log
}

/// Extra Wild Talent's "Wild Talents" named the feature, which is gone: the feat needs the kineticist level of the
/// first pick instead, as Extra Slayer Talent needs the slayer's (ticket 44). Nothing else needed the feature.
fn extra_wild_talent(guide: &Guide, data: &mut GameData, level: u8, log: &mut Log) {
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
    let gone = FeatureKey::new(WILD_TALENTS_FEATURE);
    let needing: Vec<String> = data
        .feats
        .iter()
        .filter(|feat| feat.prerequisite.as_ref().is_some_and(|p| needs(p, &gone)))
        .map(|feat| feat.key.to_string())
        .chain(
            data.classes
                .iter()
                .chain(&data.prestige_classes)
                .flat_map(|class| {
                    class
                        .selections
                        .iter()
                        .chain(class.archetypes.iter().flat_map(|a| &a.selections))
                })
                .chain(data.mythic_paths.iter().flat_map(|path| &path.selections))
                .flat_map(|selection| &selection.options)
                .filter(|option| {
                    option
                        .prerequisite
                        .as_ref()
                        .is_some_and(|p| needs(p, &gone))
                })
                .map(|option| option.key.to_string()),
        )
        .chain(
            data.mythic_abilities
                .iter()
                .filter(|a| a.prerequisite.as_ref().is_some_and(|p| needs(p, &gone)))
                .map(|a| a.key.to_string()),
        )
        .chain(
            data.mythic_feats
                .iter()
                .filter(|f| f.prerequisite.as_ref().is_some_and(|p| needs(p, &gone)))
                .map(|f| f.key.to_string()),
        )
        .chain(
            data.classes
                .iter()
                .chain(&data.prestige_classes)
                .filter(|c| c.prerequisite.as_ref().is_some_and(|p| needs(p, &gone)))
                .map(|c| c.key.to_string()),
        )
        .chain(
            data.archetypes()
                .filter(|a| a.prerequisite.as_ref().is_some_and(|p| needs(p, &gone)))
                .map(|a| a.key.to_string()),
        )
        .collect();
    assert_eq!(needing, ["extra-wild-talent"], "what needs {gone}");
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "extra-wild-talent")
        .expect("Extra Wild Talent");
    assert_eq!(
        feat.prerequisite,
        Some(Prerequisite::Feature {
            feature: gone,
            count: 1
        }),
        "extra-wild-talent's prerequisite"
    );
    feat.prerequisite = Some(Prerequisite::ClassLevel {
        class: ClassKey::new(KINETICIST),
        level,
    });
    log.push(format!("extra-wild-talent: needs kineticist level {level}"));
}

/// Whether a prerequisite names this feature anywhere.
fn needs(prerequisite: &Prerequisite, feature: &FeatureKey) -> bool {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children.iter().any(|child| needs(child, feature)),
        Prerequisite::Not(child) => needs(child, feature),
        Prerequisite::Feature { feature: key, .. } => key == feature,
        _ => false,
    }
}

/// Whether `items` held `key` (once), which is now gone.
fn take(items: &mut Vec<FeatureKey>, key: &FeatureKey) -> bool {
    let before = items.len();
    items.retain(|item| item != key);
    assert!(before - items.len() <= 1, "{key} twice");
    before != items.len()
}

/// Whether an extraction's report item is one [`apply`] settles: the bonus-feat wild talent, which can be picked now
/// and grants its slot.
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
fn talents_section(page: &Page) -> &Section {
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
