//! Ticket 59: ticket 54's decision 2 (user, 2026-11-02), applied to finished data. The kineticist's infusions, a
//! feature in the data, become a pick, as ticket 45 made the wild talents.
//!
//! - **The pick** (`kineticist#infusion`): "The Kineticist may select any Infusion for which she qualifies. She gains
//!   new Infusions at 3rd, 5th, 9th, 11th, 13th, 17th and 19th levels." The "Infusion" feature on those rows becomes a
//!   slot of the selection `kineticist/infusions`. The 1st-level "Infusion" cell stays the feature: the sentence
//!   names no pick there, and by the level rule below no infusion is open at 1.
//! - **The options** are the guide's list (`kineticist#infusions`, "Level N" groups): 32 infusions. Each needs the
//!   kineticist level twice its own ("The minimum Kineticist level to use a given Infusion is twice the Infusion's
//!   level": a `ClassLevel`), one of its elements (`Element | Air, Earth, Water`: an `Any` of the elements held;
//!   Universal: none; an option held through any selection counts, so Expanded Element's element opens its
//!   infusions) and one of the options its "Prerequisites" names (Eruption: Extended Range; Grappling Infusion:
//!   "Cloud, Deadly Earth, Wall", whose text reads "Cloud, Deadly Earth or Wall"; Foxfire Infusion: the Foxfire wild
//!   talent). The burn cost, blasts and save aren't planning facts. The guide's "Dazziing Infusion" is named
//!   "Dazzling Infusion" (a typo, not a fact); its link keeps the guide's anchor.
//! - **The Kinetic Sharpshooter** (`kineticist#infusion-kinetic-sharpshooter`): "A number of infusions are exclusive
//!   to the Sharpshooter" (each of those entries says "This infusion may be only be used by a Kinetic Sharpshooter"):
//!   they need `Archetype(kinetic-sharpshooter)`. "There are a number of infusions that the Sharpshooter cannot use":
//!   they need `Not(Archetype(kinetic-sharpshooter))`. Its own "Infusion (Kinetic Sharpshooter)" feature on rows 3 to
//!   19 is its pick, so those rows take the selection's slot and the feature goes; its rows 17 and 19 are then the
//!   base class's, so they go too (the data leaves such rows out).
//! - **The other archetypes** pick where their own tables have "Infusion" after 1st level (the Blood Kineticist's
//!   Bleeding, Gut-Wrenching and Vampiric Infusions at 5, 9 and 11 are its features already; the Kinetic Knight has
//!   none at 3 or 9); the rows already say so, and this asserts it on the rows in force.
//!
//! No feat or mythic pick gives an infusion (Over-Infused Blasts applies two at once), and nothing needs the
//! features that go: Throw Elixir's "Infusion" is the alchemist's discovery, as ticket 20 fixed in `libs/game-data/`
//! (an extraction's names the kineticist's features too, and gets that fix here). Each rule asserts the guide phrase
//! it rests on and what it finds in the data, so it fails loudly
//! on data it doesn't expect. [`apply`] runs on the finished data of an extraction (after ticket 58's `ki_powers`)
//! and, through ticket 59's one-off rewrite, on `libs/game-data/`.

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, ArchetypeLevelRow, ClassKey, ClassLevelRow, FeatureKey, FeatureSelectionKey,
    GameData, Prerequisite, SelectionOptionKey, effective_row,
};

use crate::follow_ups::{class_mut, says};
use crate::guide::Guide;
use crate::html::{Block, Page, Section};
use crate::mercies_and_exploits::levels_in_tables;
use crate::names::slug;
use crate::option_levels::{Log, option, page, section, selection};
use crate::wild_talents::{needing, take};

const KINETICIST: &str = "kineticist";
/// The feature the rows held, which the pick replaces after 1st level.
const INFUSION_FEATURE: &str = "kineticist/infusion";
/// The selection. Not keyed as the feature, which stays on the 1st-level rows.
const INFUSIONS: &str = "kineticist/infusions";
const SHARPSHOOTER: &str = "kineticist/kinetic-sharpshooter";
/// The Sharpshooter's own feature for its pick.
const SHARPSHOOTER_FEATURE: &str = "kineticist/kinetic-sharpshooter/infusion-kinetic-sharpshooter";
const WILD_TALENTS: &str = "kineticist/wild-talents";
const ELEMENT: &str = "kineticist/element";
const ELEMENTS: [&str; 4] = ["air", "earth", "fire", "water"];
/// "She gains new Infusions at 3rd, 5th, 9th, 11th, 13th, 17th and 19th levels."
const PICKS: [u8; 7] = [3, 5, 9, 11, 13, 17, 19];
/// The guide's misspelt names, and the names they mean.
const TYPOS: [(&str, &str); 1] = [("Dazziing Infusion", "Dazzling Infusion")];
const ONLY_SHARPSHOOTER: &str = "This infusion may be only be used by a Kinetic Sharpshooter";

/// Makes the infusions a pick.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let kineticist_page = page(guide, KINETICIST);
    says(
        guide,
        KINETICIST,
        "infusion",
        "The Kineticist may select any Infusion for which she qualifies. She gains new Infusions at 3rd, 5th, 9th, \
         11th, 13th, 17th and 19th levels.",
    );
    says(
        guide,
        KINETICIST,
        "infusions",
        "The minimum Kineticist level to use a given Infusion is twice the Infusion's level",
    );
    let tables = levels_in_tables(kineticist_page, "Infusion");
    assert_eq!(
        tables.keys().collect::<Vec<_>>(),
        [
            "",
            "blood-kineticist",
            "dark-elementalist",
            "elemental-engine",
            "kinetic-knight",
            "kinetic-sharpshooter",
            "overwhelming-soul",
            "psychokineticist",
        ],
        "the kineticist's tables with Infusion"
    );
    let mut base_cells = vec![1];
    base_cells.extend(PICKS);
    assert_eq!(tables[""], base_cells, "the kineticist's Infusion levels");
    no_feat_gives_an_infusion(guide);

    let infusions = read_infusions(kineticist_page);
    assert_eq!(infusions.len(), 32, "the guide's infusions");
    let (only, cannot) = sharpshooter_lists(kineticist_page);
    let marked: BTreeSet<&str> = infusions
        .iter()
        .filter(|i| i.only_sharpshooter)
        .map(|i| i.name.as_str())
        .collect();
    assert_eq!(
        marked,
        only.iter().map(String::as_str).collect(),
        "the Sharpshooter's own infusions"
    );
    let names: BTreeSet<&str> = infusions.iter().map(|i| i.name.as_str()).collect();
    for name in &cannot {
        assert!(names.contains(name.as_str()), "{name}: not an infusion");
    }
    throw_elixir(guide, data, &mut log);
    for feature in [INFUSION_FEATURE, SHARPSHOOTER_FEATURE] {
        let key = FeatureKey::new(feature);
        assert_eq!(
            needing(data, &key),
            Vec::<String>::new(),
            "what needs {key}"
        );
    }

    let kineticist = class_mut(data, KINETICIST);
    let selection_key = FeatureSelectionKey::new(INFUSIONS);
    assert!(
        kineticist.selections.iter().all(|s| s.key != selection_key),
        "{INFUSIONS}: a selection already"
    );
    let talent_key = |name: &str| -> SelectionOptionKey {
        let talents = kineticist
            .selections
            .iter()
            .find(|s| s.key.as_str() == WILD_TALENTS)
            .unwrap_or_else(|| panic!("no {WILD_TALENTS}"));
        talents
            .options
            .iter()
            .find(|o| o.name == name)
            .unwrap_or_else(|| panic!("no wild talent {name:?}"))
            .key
            .clone()
    };
    let infusion_key = |name: &str| -> SelectionOptionKey {
        let found = infusions
            .iter()
            .find(|i| i.name == name)
            .unwrap_or_else(|| panic!("no infusion {name:?}"));
        SelectionOptionKey::new(found.key())
    };
    let archetype = || Prerequisite::Archetype(ArchetypeKey::new(SHARPSHOOTER));
    let mut chosen = selection(
        INFUSIONS,
        "Infusion",
        kineticist_page.guide("infusion"),
        &[],
    );
    for infusion in &infusions {
        let key = infusion.key();
        let mut all = vec![Prerequisite::ClassLevel {
            class: ClassKey::new(KINETICIST),
            level: 2 * infusion.level,
        }];
        let element_of = |e: &String| {
            Prerequisite::SelectionOption(SelectionOptionKey::new(format!("{ELEMENT}/{e}")))
        };
        match infusion.elements.as_slice() {
            [] => {}
            [one] => all.push(element_of(one)),
            several => all.push(Prerequisite::Any(several.iter().map(element_of).collect())),
        }
        let needs: Vec<Prerequisite> = infusion
            .prerequisites
            .iter()
            .map(|name| {
                Prerequisite::SelectionOption(match name.strip_suffix(" wild talent") {
                    Some(talent) => talent_key(talent),
                    None => infusion_key(name),
                })
            })
            .collect();
        match needs.len() {
            0 => {}
            1 => all.extend(needs),
            _ => all.push(Prerequisite::Any(needs)),
        }
        if infusion.only_sharpshooter {
            all.push(archetype());
        }
        if cannot.contains(&infusion.name) {
            all.push(Prerequisite::Not(Box::new(archetype())));
        }
        let mut prerequisite = match all.len() {
            1 => all.pop().expect("the level"),
            _ => Prerequisite::All(all),
        };
        prerequisite.canonicalize();
        let mut one = option(
            &key,
            &infusion.name,
            kineticist_page.guide(&infusion.anchor),
            Vec::new(),
        );
        one.prerequisite = Some(prerequisite);
        log.push(format!(
            "{key}: level {}, {}{}{}",
            infusion.level,
            match infusion.elements.as_slice() {
                [] => "any element".to_owned(),
                elements => elements.join(" or "),
            },
            match infusion.prerequisites.as_slice() {
                [] => String::new(),
                names => format!(", after {}", names.join(" or ")),
            },
            if infusion.only_sharpshooter {
                ", the Kinetic Sharpshooter only"
            } else if cannot.contains(&infusion.name) {
                ", not the Kinetic Sharpshooter"
            } else {
                ""
            }
        ));
        chosen.options.push(one);
    }
    kineticist.selections.push(chosen);

    // The feature goes from every row after 1st level; each picks an infusion instead.
    let feature_key = FeatureKey::new(INFUSION_FEATURE);
    let mut class_levels = Vec::new();
    for row in &mut kineticist.levels {
        if row.level > 1 && take(&mut row.features, &feature_key) {
            row.selections.push(selection_key.clone());
            class_levels.push(row.level);
        }
    }
    assert_eq!(class_levels, PICKS, "the base class's rows");
    log.push(format!("{KINETICIST} {class_levels:?}: an infusion"));
    let sharpshooter_feature = FeatureKey::new(SHARPSHOOTER_FEATURE);
    let base: Vec<ClassLevelRow> = kineticist.levels.clone();
    for archetype in &mut kineticist.archetypes {
        let own = archetype.key.as_str() == SHARPSHOOTER;
        let mut touched = Vec::new();
        for row in &mut archetype.levels {
            let held = match own {
                true => take(&mut row.features, &sharpshooter_feature),
                false => row.level > 1 && take(&mut row.features, &feature_key),
            };
            if held {
                assert!(
                    !row.selections.contains(&selection_key),
                    "{} {}: an infusion slot already",
                    archetype.key,
                    row.level
                );
                row.selections.push(selection_key.clone());
                log.push(format!("{} {}: an infusion", archetype.key, row.level));
                touched.push(row.level);
            }
        }
        // A row that is now the base class's goes, as the data leaves such rows out (the Sharpshooter's 17 and 19).
        archetype.levels.retain(|row| {
            let same = touched.contains(&row.level)
                && base
                    .iter()
                    .find(|b| b.level == row.level)
                    .is_some_and(|b| same_as_base(row, b));
            if same {
                log.push(format!(
                    "{} {}: the base class's row now, left out",
                    archetype.key, row.level
                ));
            }
            !same
        });
        if own {
            let before = archetype.features.len();
            archetype.features.retain(|f| f.key != sharpshooter_feature);
            assert_eq!(
                archetype.features.len(),
                before - 1,
                "{sharpshooter_feature}"
            );
            log.push(format!(
                "{SHARPSHOOTER}: {sharpshooter_feature} is its pick"
            ));
        }
    }

    // Each table's levels are the rows in force: the archetype's where it differs, the base's elsewhere.
    let kineticist = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == KINETICIST)
        .expect("the kineticist");
    let held = |archetype| -> (Vec<u8>, bool) {
        let picks = (1..=20)
            .filter(|level| {
                effective_row(kineticist, archetype, *level).is_some_and(|row| {
                    row.selections
                        .iter()
                        .filter(|s| **s == selection_key)
                        .count()
                        == 1
                })
            })
            .collect();
        let feature = effective_row(kineticist, archetype, 1)
            .is_some_and(|row| row.features.contains(&feature_key));
        (picks, feature)
    };
    let expected = |anchor: &str| -> (Vec<u8>, bool) {
        let cells = &tables[anchor];
        (
            cells.iter().copied().filter(|level| *level > 1).collect(),
            cells.contains(&1),
        )
    };
    assert_eq!(held(None), expected(""), "the kineticist's infusions");
    for archetype in &kineticist.archetypes {
        let anchor = archetype
            .guide
            .0
            .strip_prefix("kineticist#")
            .unwrap_or_default();
        assert_eq!(
            held(Some(archetype)),
            expected(anchor),
            "{}'s infusions",
            archetype.key
        );
    }
    assert_eq!(
        kineticist.archetypes.len(),
        tables.len() - 1,
        "the archetypes"
    );
    log
}

/// Neither the feats nor the mythic picks give an infusion: no section of their pages names one but Over-Infused
/// Blasts, which applies two at once, and Throw Elixir, whose "Infusion" is the alchemist's discovery (ticket 20).
fn no_feat_gives_an_infusion(guide: &Guide) {
    says(
        guide,
        "mythic-paths",
        "over-infused-blasts",
        "You can simultaneously apply two substance infusions to your kinetic blasts.",
    );
    says(
        guide,
        "mythic-paths",
        "throw-elixir",
        "Increases the range of an Alchemist's extracts",
    );
    for (slug, expected) in [
        ("feats", Vec::<&str>::new()),
        ("mythic-paths", vec!["over-infused-blasts", "throw-elixir"]),
    ] {
        let naming: Vec<&str> = page(guide, slug)
            .root
            .walk()
            .into_iter()
            .filter(|section| section.text().to_lowercase().contains("infusion"))
            .map(|section| section.anchor.as_str())
            .collect();
        assert_eq!(naming, expected, "the {slug} sections naming infusions");
    }
}

/// Throw Elixir's "Prerequisites: Infusion" is the alchemist's Infusion discovery: it "Increases the range of an
/// Alchemist's extracts". An extraction reads the name as every "Infusion" (the kineticist's two features too), as
/// ticket 20 fixed in `libs/game-data/` by hand; the Sharpshooter's feature goes here, so an extraction's Throw Elixir
/// gets ticket 20's prerequisite. `libs/game-data/`'s has it already.
fn throw_elixir(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "mythic-paths",
        "throw-elixir",
        "Increases the range of an Alchemist's extracts from touch to close.",
    );
    let discovery =
        Prerequisite::SelectionOption(SelectionOptionKey::new("alchemist/discovery/infusion"));
    let ability = data
        .mythic_abilities
        .iter_mut()
        .find(|a| a.key.as_str() == "throw-elixir")
        .expect("Throw Elixir");
    if ability.prerequisite.as_ref() == Some(&discovery) {
        return;
    }
    let feature = |key: &str| Prerequisite::Feature {
        feature: FeatureKey::new(key),
        count: 1,
    };
    let mut every = Prerequisite::Any(vec![
        feature(INFUSION_FEATURE),
        feature(SHARPSHOOTER_FEATURE),
        discovery.clone(),
    ]);
    every.canonicalize();
    assert_eq!(
        ability.prerequisite,
        Some(every),
        "throw-elixir's prerequisite"
    );
    ability.prerequisite = Some(discovery);
    log.push("throw-elixir: the alchemist's Infusion discovery only (ticket 20's fix)".to_owned());
}

/// Whether an archetype row grants what the base row does, with no spell levels of its own (the data check's rule).
fn same_as_base(row: &ArchetypeLevelRow, base: &ClassLevelRow) -> bool {
    fn sorted<T: Ord + Clone>(items: &[T]) -> Vec<T> {
        let mut items = items.to_vec();
        items.sort();
        items
    }
    row.spell_levels.is_none()
        && sorted(&row.features) == sorted(&base.features)
        && row.feats == base.feats
        && row.feats_for == base.feats_for
        && sorted(&row.feat_slots) == sorted(&base.feat_slots)
        && sorted(&row.selections) == sorted(&base.selections)
}

/// The Sharpshooter's section's two lists, each the one item of the list after its sentence: the infusions exclusive
/// to it, and those it cannot use.
fn sharpshooter_lists(page: &Page) -> (Vec<String>, Vec<String>) {
    let own = section(page, "infusion-kinetic-sharpshooter");
    let lines: Vec<&String> = own
        .blocks
        .iter()
        .flat_map(|block| match block {
            Block::Para(lines) | Block::List(lines) => lines.iter().collect(),
            _ => Vec::new(),
        })
        .collect();
    let after = |phrase: &str| -> Vec<String> {
        let at = lines
            .iter()
            .position(|line| line.ends_with(phrase))
            .unwrap_or_else(|| panic!("kineticist#infusion-kinetic-sharpshooter: no {phrase:?}"));
        let list = lines
            .get(at + 1)
            .and_then(|line| line.strip_suffix('.'))
            .unwrap_or_else(|| {
                panic!("kineticist#infusion-kinetic-sharpshooter: no list after {phrase:?}")
            });
        list.split(", ").map(str::to_owned).collect()
    };
    let only = after("A number of infusions are exclusive to the Sharpshooter:");
    let cannot =
        after("However, there are a number of infusions that the Sharpshooter cannot use:");
    assert_eq!(only.len(), 4, "the Sharpshooter's own infusions");
    assert_eq!(cannot.len(), 6, "the infusions the Sharpshooter cannot use");
    (only, cannot)
}

/// One infusion as the guide's list gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Infusion {
    /// Its section.
    pub anchor: String,
    /// Its name, misspellings fixed.
    pub name: String,
    pub level: u8,
    /// Lowercase element names; empty for Universal.
    pub elements: Vec<String>,
    /// The names its "Prerequisites" row lists (any one of them).
    pub prerequisites: Vec<String>,
    /// Its text says only a Kinetic Sharpshooter may use it.
    pub only_sharpshooter: bool,
}

impl Infusion {
    /// `kineticist/infusions/extended-range`.
    fn key(&self) -> String {
        format!("{INFUSIONS}/{}", slug(&self.name))
    }
}

/// The list's infusions (`kineticist#infusions`: "Level N" groups of one section each), in the guide's order.
pub fn read_infusions(page: &Page) -> Vec<Infusion> {
    let list = section(page, "infusions");
    assert!(
        list.children
            .iter()
            .all(|group| group.children.iter().all(|e| e.children.is_empty())),
        "kineticist#infusions: deeper sections"
    );
    let mut infusions = Vec::new();
    for group in &list.children {
        let level: u8 = group
            .name
            .strip_prefix("Level ")
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("kineticist#{}: not \"Level N\"", group.anchor));
        assert_eq!(group.anchor, format!("level-{level}"));
        for entry in &group.children {
            infusions.push(read_infusion(entry, level));
        }
    }
    infusions
}

fn read_infusion(entry: &Section, level: u8) -> Infusion {
    let rows: Vec<&Vec<String>> = entry.tables().flatten().collect();
    let cell = |label: &str| -> Option<String> {
        rows.iter()
            .find(|row| row.first().map(String::as_str) == Some(label))
            .map(|row| row.get(1).cloned().unwrap_or_default())
    };
    let element = cell("Element").unwrap_or_else(|| panic!("{}: no Element", entry.anchor));
    let elements = match element.as_str() {
        "Universal" => Vec::new(),
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
    let kind = cell("Type").unwrap_or_default();
    assert!(
        matches!(kind.as_str(), "Form infusion" | "Substance infusion"),
        "{}: type {kind:?}",
        entry.anchor
    );
    let text = rows
        .iter()
        .filter(|row| row.len() == 1)
        .map(|row| row[0].replace('\n', " "))
        .collect::<Vec<_>>()
        .join(" ");
    let name = TYPOS
        .iter()
        .find(|(typo, _)| *typo == entry.name)
        .map_or(entry.name.clone(), |(_, fixed)| (*fixed).to_owned());
    let prerequisites: Vec<String> = cell("Prerequisites")
        .map(|names| names.split(", ").map(str::to_owned).collect())
        .unwrap_or_default();
    if prerequisites.len() > 1 {
        // "Cloud, Deadly Earth, Wall": the text says which ("Cloud, Deadly Earth or Wall"), one of them.
        let (last, rest) = prerequisites.split_last().expect("several");
        let either = format!("{} or {last}", rest.join(", "));
        assert!(
            text.contains(&either),
            "{}: {:?} isn't one of them in {text:?}",
            entry.anchor,
            prerequisites
        );
    }
    let infusion = Infusion {
        anchor: entry.anchor.clone(),
        name,
        level,
        elements,
        prerequisites,
        only_sharpshooter: text.contains(ONLY_SHARPSHOOTER),
    };
    // The key is the name's, so the anchor is too, but where the guide misspells it.
    if infusion.name == entry.name {
        assert_eq!(
            slug(&infusion.name),
            infusion.anchor,
            "{}: its anchor",
            infusion.name
        );
    }
    infusion
}
