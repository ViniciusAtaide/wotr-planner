//! Ticket 61: ticket 54's decision 2 (user, 2026-11-02). Four bonus-feat choices are slots of fixed lists, the shape
//! of the Divine Guardian's bonus feats: lists ignoring prerequisites, and a `OneOf` slot where the table has the
//! cell, which the class stage reads so ([`provide`]). No feature stands for the choice.
//!
//! - **The Titan Fighter's Massive Attacks** (`fighter#massive-attacks`: "At 2nd level and every four subsequent
//!   levels, a Titan fighter selects a feat from the Ranger's Two-Weapon Combat Style feats"). Its table has
//!   "Massive Attacks" at 2, 6, 10, 14 and 18, in place of the fighter's Bonus Combat Feat there. The ranger's
//!   Two-Weapon Combat Feats grow at 2, 6 and 10 (`ranger#two-weapon-combat-feats`), so the Titan Fighter's own lists
//!   `fighter/titan-fighter/bonus-feats-2`, `-6` and `-10` hold the style's feats by then, read by fighter level as a
//!   ranger's are by ranger level. They are the style feats alone: "Besides the style feats" the ranger may take five
//!   more feats (and Skill Focus from 10), which the sentence doesn't name. Like the ranger's ("does not need to meet
//!   the prerequisites"), they ignore prerequisites.
//! - **The hunter's Precise Companion** (`hunter#precise-companion`: "At 2nd level, a Hunter chooses either Precise
//!   Shot or Outflank as a bonus feat. She does not need to meet the prerequisites."): `hunter/bonus-feats-2`.
//! - **The Urban Hunter's Captor** (`hunter#captor`: a bonus feat "At 6th level and every 3 subsequent levels", from
//!   four feats and six more "From 12th level", and "Neither she nor her companion need to meet the prerequisites"):
//!   `hunter/urban-hunter/bonus-feats-6` and `-12`.
//! - **The Stormwalker's Archery** (`ranger#archery`: "May select from the Ranger's list of Archery combat style
//!   feats"): the archery style's feats without the extras, `ranger/bonus-feats-2b`, `-6b` and `-10b`, the lists the
//!   Nomad's "Archery Feat" picks from in `libs/game-data/` (a hand fix of ticket 18, left as it is).
//!
//! Each rule asserts the guide phrase it rests on where the slots are read, and [`check`] what it finds in the finished
//! data (the style lists of ticket 30, the rows in force), so it fails loudly on data it doesn't expect.

use std::collections::{BTreeMap, BTreeSet};

use rules::{Archetype, Class, FeatKey, FeatList, FeatListKey, FeatSlot, GameData, effective_row};

use crate::follow_ups::says;
use crate::guide::Guide;
use crate::html::Page;
use crate::mercies_and_exploits::levels_in_tables;
use crate::option_levels::{FeatNames, list, name_list, page, section};
use crate::picks::{Pick, Picks, Rule};

const FIGHTER: &str = "fighter";
const TITAN: &str = "fighter/titan-fighter";
const MASSIVE_ATTACKS: &str = "fighter/titan-fighter/massive-attacks";
/// 2nd level and every four levels after.
const MASSIVE_ATTACKS_LEVELS: [u8; 5] = [2, 6, 10, 14, 18];

const HUNTER: &str = "hunter";
const PRECISE_COMPANION: &str = "hunter/precise-companion";
const URBAN_HUNTER: &str = "hunter/urban-hunter";
const CAPTOR: &str = "hunter/urban-hunter/captor";
/// "At 6th level and every 3 subsequent levels".
const CAPTOR_LEVELS: [u8; 5] = [6, 9, 12, 15, 18];

const RANGER: &str = "ranger";
const STORMWALKER_ARCHERY: &str = "ranger/stormwalker/archery";
/// The ranger's table levels of a combat style feat, which the Stormwalker's "Archery" keeps.
const STORMWALKER_LEVELS: [u8; 5] = [2, 6, 10, 14, 18];

/// The ranger's style sections the choices name.
const TWO_WEAPON: &str = "two-weapon-combat-feats";
const ARCHERY: &str = "archery-feats";
/// The levels each ranger style's list grows at.
const STYLE_LEVELS: [u8; 3] = [2, 6, 10];

/// A list growing at levels: from each level on (until the next), the slot picks from those feats.
type Growing = Vec<(u8, BTreeSet<FeatKey>)>;

/// One bonus-feat choice: who has it, where its tables have it, and its lists.
struct Choice<'a> {
    class: &'static str,
    /// The feature the generic reader would make of the cell.
    feature: &'static str,
    /// The archetype that owns the lists; `None`, the class.
    lists_owner: Option<&'static str>,
    /// The table cell that is the choice.
    cell: &'static str,
    /// The levels the cell stands at, in each table that has it, by the table's anchor.
    tables: BTreeMap<String, Vec<u8>>,
    /// Tables whose rows may hold slots of the lists too, under another cell, left as they are.
    shared_with: &'a [&'static str],
    /// The lists, each with the level it's open from.
    lists: Vec<(u8, FeatList)>,
}

/// The bonus-feat choices are slots of their lists, with no feature standing for them.
pub(crate) fn provide(guide: &Guide, feats: &FeatNames, picks: &mut Picks) {
    for choice in choices(guide, feats) {
        choice_slots(guide, picks, &choice);
    }
}

/// The style lists the choices copy, and the rows in force.
pub fn check(guide: &Guide, data: &GameData) {
    let feats = FeatNames::new(data);
    check_ranger_styles(guide, data, &feats);
    for choice in choices(guide, &feats) {
        check_choice(data, &choice);
    }
}

fn choices<'a>(guide: &Guide, feats: &FeatNames) -> Vec<Choice<'a>> {
    let styles = ranger_styles(guide, feats);
    vec![
        massive_attacks(guide, &styles[TWO_WEAPON]),
        precise_companion(guide, feats),
        captor(guide, feats),
        stormwalker_archery(guide, &styles[ARCHERY]),
    ]
}

/// The feats a ranger style section names, by the level they're open from, each level's names holding the earlier
/// ones': "At 2nd level, the Ranger can select from A, B and C. At 6th level, D is added. At 10th level, E and F are
/// added." Sentences that don't start "At …th level" (a note after the list) are passed over.
pub fn style_feats(text: &str) -> Vec<(u8, Vec<String>)> {
    let mut held: Vec<String> = Vec::new();
    let mut levels = Vec::new();
    for sentence in text.split(". ") {
        let Some(rest) = sentence.trim().strip_prefix("At ") else {
            continue;
        };
        let Some((level, rest)) = ["st", "nd", "rd", "th"]
            .into_iter()
            .find_map(|suffix| rest.split_once(&format!("{suffix} level, ")))
        else {
            continue;
        };
        let Ok(level) = level.parse::<u8>() else {
            continue;
        };
        let names = match rest.split_once(" from ") {
            Some((_, names)) => names,
            None => rest
                .trim_end_matches('.')
                .trim_end_matches(" are added")
                .trim_end_matches(" is added"),
        };
        held.extend(name_list(names));
        levels.push((level, held.clone()));
    }
    levels
}

/// The ranger's two-weapon and archery styles' own feats by level, without the feats every style adds ("Besides the
/// style feats, you can also select from …"), read from their sections.
fn ranger_styles(guide: &Guide, feats: &FeatNames) -> BTreeMap<&'static str, Growing> {
    says(
        guide,
        RANGER,
        "combat-style-feat",
        "does not need to meet the prerequisites",
    );
    says(
        guide,
        RANGER,
        "combat-style-feat",
        "Besides the style feats",
    );
    let ranger_page = page(guide, RANGER);
    let mut styles = BTreeMap::new();
    for anchor in [TWO_WEAPON, ARCHERY] {
        let text = section(ranger_page, anchor).text();
        let growing: Growing = style_feats(&text)
            .into_iter()
            .map(|(level, names)| (level, feats.all(&names, anchor)))
            .collect();
        assert_eq!(
            growing.iter().map(|(level, _)| *level).collect::<Vec<_>>(),
            STYLE_LEVELS,
            "ranger#{anchor}: the levels"
        );
        styles.insert(anchor, growing);
    }
    styles
}

/// Each style's feats are exactly the data's style lists (ticket 30's) less one set of extras per level, the same for
/// both styles.
fn check_ranger_styles(guide: &Guide, data: &GameData, feats: &FeatNames) {
    let ranger = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == RANGER)
        .expect("the ranger");
    let mut extras: BTreeMap<u8, BTreeSet<FeatKey>> = BTreeMap::new();
    for (anchor, growing) in ranger_styles(guide, feats) {
        for (level, style) in &growing {
            let key = format!("ranger/combat-style-feat/{anchor}-{level}");
            let data_list = &ranger
                .feat_lists
                .iter()
                .find(|l| l.key.as_str() == key)
                .unwrap_or_else(|| panic!("no {key}"))
                .feats;
            assert!(style.is_subset(data_list), "{key}: the style's feats");
            let rest: BTreeSet<FeatKey> = data_list - style;
            let known = extras.entry(*level).or_insert_with(|| rest.clone());
            assert_eq!(*known, rest, "{key}: the extras");
        }
    }
}

/// The Titan Fighter's Massive Attacks: lists of its own.
fn massive_attacks<'a>(guide: &Guide, two_weapon: &Growing) -> Choice<'a> {
    says(
        guide,
        FIGHTER,
        "massive-attacks",
        "At 2nd level and every four subsequent levels, a Titan fighter selects a feat from the Ranger's Two-Weapon \
         Combat Style feats.",
    );
    let guide_ref = page(guide, FIGHTER).guide("massive-attacks");
    let lists = two_weapon
        .iter()
        .enumerate()
        .map(|(i, (level, feats))| {
            let name = if i == 0 {
                "Massive Attacks".to_owned()
            } else {
                format!("Massive Attacks (from level {level})")
            };
            let key = format!("{TITAN}/bonus-feats-{level}");
            (
                *level,
                list(&key, &name, guide_ref.clone(), true, feats.clone()),
            )
        })
        .collect();
    Choice {
        class: FIGHTER,
        feature: MASSIVE_ATTACKS,
        lists_owner: Some(TITAN),
        cell: "Massive Attacks",
        tables: BTreeMap::from([("titan-fighter".to_owned(), MASSIVE_ATTACKS_LEVELS.to_vec())]),
        shared_with: &[],
        lists,
    }
}

/// The hunter's Precise Companion: one list, at 2, on every table that has the cell (all but the Forester's).
fn precise_companion<'a>(guide: &Guide, feats: &FeatNames) -> Choice<'a> {
    let sentence = "At 2nd level, a Hunter chooses either Precise Shot or Outflank as a bonus feat. She does not \
                    need to meet the prerequisites.";
    says(guide, HUNTER, "precise-companion", sentence);
    let names: Vec<String> = sentence
        .split_once("either ")
        .and_then(|(_, rest)| rest.split_once(" as a bonus feat"))
        .map(|(names, _)| names.split(" or ").map(str::to_owned).collect())
        .expect("the two feats");
    // Every hunter table, the base class's and each archetype's, but the Forester's.
    let mut tables: BTreeMap<String, Vec<u8>> = progression_tables(page(guide, HUNTER))
        .into_iter()
        .map(|anchor| (anchor, vec![2]))
        .collect();
    assert!(
        tables.remove("forester").is_some(),
        "hunter#forester: no table"
    );
    Choice {
        class: HUNTER,
        feature: PRECISE_COMPANION,
        lists_owner: None,
        cell: "Precise Companion",
        tables,
        shared_with: &[],
        lists: vec![(
            2,
            list(
                "hunter/bonus-feats-2",
                "Precise Companion",
                page(guide, HUNTER).guide("precise-companion"),
                true,
                feats.all(&names, PRECISE_COMPANION),
            ),
        )],
    }
}

/// The Urban Hunter's Captor: four feats from 6, ten from 12.
fn captor<'a>(guide: &Guide, feats: &FeatNames) -> Choice<'a> {
    for phrase in [
        "At 6th level and every 3 subsequent levels, a hunter may choose a bonus feat",
        "Neither she nor her companion need to meet the prerequisites.",
    ] {
        says(guide, HUNTER, "captor", phrase);
    }
    let text = section(page(guide, HUNTER), "captor").text();
    let after = |lead: &str| -> Vec<String> {
        let rest = text
            .split_once(lead)
            .unwrap_or_else(|| panic!("hunter#captor: no {lead:?}"))
            .1;
        name_list(rest.split(". ").next().expect("a sentence"))
    };
    let at_6 = feats.all(
        &after("At 6th level, she can choose between "),
        "Captor at 6",
    );
    let more = feats.all(
        &after("From 12th level, the list of feats expands and she can additionally choose from "),
        "Captor from 12",
    );
    let guide_ref = page(guide, HUNTER).guide("captor");
    Choice {
        class: HUNTER,
        feature: CAPTOR,
        lists_owner: Some(URBAN_HUNTER),
        cell: "Captor",
        tables: BTreeMap::from([("urban-hunter".to_owned(), CAPTOR_LEVELS.to_vec())]),
        shared_with: &[],
        lists: vec![
            (
                6,
                list(
                    "hunter/urban-hunter/bonus-feats-6",
                    "Captor",
                    guide_ref.clone(),
                    true,
                    at_6.clone(),
                ),
            ),
            (
                12,
                list(
                    "hunter/urban-hunter/bonus-feats-12",
                    "Captor (from level 12)",
                    guide_ref,
                    true,
                    &at_6 | &more,
                ),
            ),
        ],
    }
}

/// The Stormwalker's Archery: the ranger's archery style feats.
fn stormwalker_archery<'a>(guide: &Guide, archery: &Growing) -> Choice<'a> {
    says(
        guide,
        RANGER,
        "archery",
        "May select from the Ranger's list of Archery combat style feats.",
    );
    says(
        guide,
        RANGER,
        "stormwalker",
        "May only select Archery feats as bonus feats.",
    );
    let guide_ref = page(guide, RANGER).guide(ARCHERY);
    let lists = archery
        .iter()
        .enumerate()
        .map(|(i, (level, feats))| {
            let name = if i == 0 {
                "Archery Feats".to_owned()
            } else {
                format!("Archery Feats (from level {level})")
            };
            let key = format!("ranger/bonus-feats-{level}b");
            (
                *level,
                list(&key, &name, guide_ref.clone(), true, feats.clone()),
            )
        })
        .collect();
    Choice {
        class: RANGER,
        feature: STORMWALKER_ARCHERY,
        lists_owner: None,
        cell: "Archery",
        tables: BTreeMap::from([("stormwalker".to_owned(), STORMWALKER_LEVELS.to_vec())]),
        // The Nomad's "Archery Feat" ("Can only select Archery style feats").
        shared_with: &["nomad"],
        lists,
    }
}

/// The anchors of a page's progression tables (the base class's: `""`), whatever cells they have.
fn progression_tables(page: &Page) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for section in page.root.walk() {
        for table in section.tables() {
            let progression = table.first().is_some_and(|header| {
                matches!(header.first().map(String::as_str), Some("Lvl" | "Level"))
                    && header.iter().any(|c| c == "Feat" || c == "Feats")
            });
            if progression {
                found.insert(section.anchor.clone());
            }
        }
    }
    found
}

/// A choice's cells are slots of its lists: each cell naming the choice's feature, at a level a table has it, is a
/// slot of the list open at that level. The page's tables have the cell exactly at `tables`. The lists are the
/// owner's.
fn choice_slots(guide: &Guide, picks: &mut Picks, choice: &Choice) {
    let class_key = choice.class;
    assert_eq!(
        levels_in_tables(page(guide, class_key), choice.cell),
        choice.tables,
        "{class_key}: the tables' {:?} levels",
        choice.cell
    );
    let thresholds = thresholds(choice);
    let levels: BTreeSet<u8> = choice.tables.values().flatten().copied().collect();
    for level in levels {
        picks.rule(Rule {
            cell: Some(choice.cell.to_owned()),
            levels: Some(vec![level]),
            ..Rule::every(
                choice.feature,
                Pick::Slots(vec![FeatSlot::OneOf {
                    list: list_at(&thresholds, level),
                }]),
            )
        });
    }
    let owner = choice.lists_owner.unwrap_or(class_key);
    for (_, list) in &choice.lists {
        picks.feat_list(owner, list.clone());
    }
}

/// The lists' keys, each with the level it's open from.
fn thresholds(choice: &Choice) -> Vec<(u8, FeatListKey)> {
    choice
        .lists
        .iter()
        .map(|(level, list)| (*level, list.key.clone()))
        .collect()
}

/// The owner has the lists as given, and the rows in force hold the slots exactly where the tables have the cell,
/// one slot on each such row, and none elsewhere but in the tables `shared_with`.
fn check_choice(data: &GameData, choice: &Choice) {
    let class_key = choice.class;
    let thresholds = thresholds(choice);
    let class = class_of(data, class_key);
    let owned = match choice.lists_owner {
        Some(archetype) => &archetype_of(class, archetype).feat_lists,
        None => &class.feat_lists,
    };
    for (_, list) in &choice.lists {
        assert!(
            owned.contains(list),
            "{}: the list {}",
            choice.feature,
            list.key
        );
    }
    for archetype in std::iter::once(None).chain(class.archetypes.iter().map(Some)) {
        let at = archetype.map(|a| anchor(class_key, a)).unwrap_or_default();
        if choice.shared_with.contains(&at.as_str()) {
            continue;
        }
        let levels = choice.tables.get(&at).cloned().unwrap_or_default();
        let expected: Vec<(u8, FeatListKey)> = levels
            .iter()
            .map(|level| (*level, list_at(&thresholds, *level)))
            .collect();
        assert_eq!(
            slots_of(class, archetype, &thresholds),
            expected,
            "{class_key} {at:?}: {}",
            choice.cell
        );
        for level in levels {
            let row = effective_row(class, archetype, level).expect("a row");
            assert_eq!(row.feat_slots.len(), 1, "{class_key} {at:?} {level}: slots");
        }
    }
}

/// An archetype's anchor on its class's page.
fn anchor(class_key: &str, archetype: &Archetype) -> String {
    archetype
        .guide
        .0
        .strip_prefix(&format!("{class_key}#"))
        .unwrap_or_else(|| panic!("{}: guide {}", archetype.key, archetype.guide.0))
        .to_owned()
}

/// The list a slot at `level` picks from: the one open from the highest level reached.
fn list_at(lists: &[(u8, FeatListKey)], level: u8) -> FeatListKey {
    lists
        .iter()
        .rev()
        .find(|(from, _)| *from <= level)
        .map(|(_, key)| key.clone())
        .unwrap_or_else(|| panic!("no list open at {level}"))
}

/// The slots of `lists` the rows in force hold, by level.
fn slots_of(
    class: &Class,
    archetype: Option<&Archetype>,
    lists: &[(u8, FeatListKey)],
) -> Vec<(u8, FeatListKey)> {
    let keys: BTreeSet<&FeatListKey> = lists.iter().map(|(_, key)| key).collect();
    (1..=20)
        .filter_map(|level| effective_row(class, archetype, level))
        .flat_map(|row| {
            row.feat_slots
                .iter()
                .filter_map(|slot| match slot {
                    FeatSlot::OneOf { list } if keys.contains(list) => {
                        Some((row.level, list.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn class_of<'d>(data: &'d GameData, key: &str) -> &'d Class {
    data.classes
        .iter()
        .find(|class| class.key.as_str() == key)
        .unwrap_or_else(|| panic!("no class {key}"))
}

fn archetype_of<'c>(class: &'c Class, key: &str) -> &'c Archetype {
    class
        .archetypes
        .iter()
        .find(|archetype| archetype.key.as_str() == key)
        .unwrap_or_else(|| panic!("no archetype {key}"))
}
