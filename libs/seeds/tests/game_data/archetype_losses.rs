//! Ticket 195: an archetype row replaces the base row's features, feats, feat slots and selections in full, so a row
//! written for one reason can drop something else the base row gives at that level. For every archetype row, what the
//! base row gives and the row lacks must be on the archetype's list in `archetype_losses.ron`, with the guide's
//! reason; anything else is a carry-over the row forgot.
//!
//! What is compared, per level where the archetype has a row (a level without one is the base row, which drops
//! nothing):
//! - the multiset of features, of feat slots and of selection slots, and the sets of granted feats and of granted
//!   feats-for-a-value: the shortfall is how many of each the base row has more than the row;
//! - the spell levels, when the row sets them below the base row's (a row that raises them is a grant).
//!
//! A list entry is `(item, at, count, why)`. With no `at` the archetype loses the item at every level where the base
//! row gives it, so a level where the archetype has no row (it keeps the item) fails the entry, which is how a step
//! added to a base class later is caught. With `at`, exactly those levels. `count` (default 1) is the shortfall at
//! each of them. A listed loss the rows don't make fails as well as an unlisted one the rows do.
//!
//! Not here, because other tests own them: an archetype's replaced class skills (`class_skills.rs`), its replaced
//! saves (`archetype_saves.rs`), and the option rows a selection skips with `skipped_levels` (`scaling_features.rs`,
//! which checks the skipped rows of every selection against every option it offers).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use rules::{
    Archetype, ArchetypeKey, ArchetypeLevelRow, Class, ClassLevelRow, FeatKey, FeatSlot, FeatValue,
    FeatureKey, FeatureSelectionKey,
};
use serde::Deserialize;

use crate::support::DATA;

/// The lists, as written.
const LOSSES: &str = include_str!("archetype_losses.ron");

/// Where a message sends the reader to carry an item on the row, and where it sends them to list it as lost.
const LIST_FILE: &str = "libs/seeds/tests/game_data/archetype_losses.ron";

/// One thing a row can lose.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
enum Item {
    Feature(FeatureKey),
    Feat(FeatKey),
    FeatFor(FeatValue),
    Slot(FeatSlot),
    Selection(FeatureSelectionKey),
    SpellLevels,
}

/// One entry of an archetype's list.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Loss {
    item: Item,
    /// The levels it is lost at; none: every level where the base row gives it.
    #[serde(default)]
    at: BTreeSet<u8>,
    #[serde(default = "one")]
    count: u8,
    /// Required with `at`: what the guide's table shows at those levels.
    #[serde(default)]
    why: String,
}

fn one() -> u8 {
    1
}

/// What one archetype loses, with the guide's reason.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Losses {
    archetype: ArchetypeKey,
    why: String,
    loses: Vec<Loss>,
}

/// How many of each item a row (or the base row) has, by level.
type ByLevel = BTreeMap<Item, BTreeMap<u8, u8>>;

fn lists() -> Vec<Losses> {
    ron::from_str(LOSSES).unwrap_or_else(|error| panic!("{LIST_FILE} doesn't parse: {error}"))
}

/// Adds the multiset `items` to `into` as `item(each)` at `level`.
fn tally<'a, T: 'a>(
    into: &mut ByLevel,
    level: u8,
    items: impl IntoIterator<Item = &'a T>,
    item: impl Fn(&T) -> Item,
) {
    for each in items {
        *into
            .entry(item(each))
            .or_default()
            .entry(level)
            .or_default() += 1;
    }
}

/// A row's features, granted feats, granted feats for a value, feat slots and selections.
type Grants<'a> = (
    &'a [FeatureKey],
    &'a BTreeSet<FeatKey>,
    &'a BTreeSet<FeatValue>,
    &'a [FeatSlot],
    &'a [FeatureSelectionKey],
);

/// The five grants of a row (a class's or an archetype's), tallied into `into` at `level`.
fn tally_row(into: &mut ByLevel, level: u8, grants: Grants) {
    let (features, feats, feats_for, feat_slots, selections) = grants;
    tally(into, level, features, |k| Item::Feature(k.clone()));
    tally(into, level, feats, |k| Item::Feat(k.clone()));
    tally(into, level, feats_for, |v| Item::FeatFor(v.clone()));
    tally(into, level, feat_slots, |s| Item::Slot(s.clone()));
    tally(into, level, selections, |k| Item::Selection(k.clone()));
}

/// What a base row gives, by item and level.
fn gives(base: &[ClassLevelRow]) -> ByLevel {
    let mut gives = ByLevel::new();
    for row in base {
        let grants = (
            &row.features[..],
            &row.feats,
            &row.feats_for,
            &row.feat_slots[..],
            &row.selections[..],
        );
        tally_row(&mut gives, row.level, grants);
    }
    gives
}

/// What the base rows give and the archetype's rows lack, by item and level: the shortfall.
fn shortfalls(base: &[ClassLevelRow], rows: &[ArchetypeLevelRow]) -> ByLevel {
    let base_gives = gives(base);
    let mut own = ByLevel::new();
    let mut shortfalls = ByLevel::new();
    for row in rows {
        let grants = (
            &row.features[..],
            &row.feats,
            &row.feats_for,
            &row.feat_slots[..],
            &row.selections[..],
        );
        tally_row(&mut own, row.level, grants);
        let Some(base_row) = base.iter().find(|b| b.level == row.level) else {
            continue;
        };
        if let Some(spell_levels) = row.spell_levels {
            let base_levels = base_row.spell_levels();
            if spell_levels.arcane < base_levels.arcane || spell_levels.divine < base_levels.divine
            {
                shortfalls
                    .entry(Item::SpellLevels)
                    .or_default()
                    .insert(row.level, 1);
            }
        }
    }
    for (item, levels) in &base_gives {
        for (&level, &has) in levels {
            if !rows.iter().any(|row| row.level == level) {
                continue;
            }
            let kept = own
                .get(item)
                .and_then(|levels| levels.get(&level))
                .copied()
                .unwrap_or(0);
            if has > kept {
                shortfalls
                    .entry(item.clone())
                    .or_default()
                    .insert(level, has - kept);
            }
        }
    }
    shortfalls
}

/// The display names of the class's and the archetype's features and selections, by key.
fn names(class: &Class, archetype: &Archetype) -> BTreeMap<String, String> {
    let features = class.features.iter().chain(&archetype.features);
    let selections = class.selections.iter().chain(&archetype.selections);
    features
        .map(|f| (f.key.to_string(), f.name.clone()))
        .chain(selections.map(|s| (s.key.to_string(), s.name.clone())))
        .collect()
}

/// A value in the data files' notation.
fn ron_text<T: serde::Serialize>(value: &T) -> String {
    ron::to_string(value).expect("a data value writes as RON")
}

/// An item as the list file writes it.
fn written(item: &Item) -> String {
    match item {
        Item::Feature(key) => format!("Feature({:?})", key.as_str()),
        Item::Feat(key) => format!("Feat({:?})", key.as_str()),
        Item::FeatFor(value) => format!("FeatFor({})", ron_text(value)),
        Item::Slot(slot) => format!("Slot({})", ron_text(slot)),
        Item::Selection(key) => format!("Selection({:?})", key.as_str()),
        Item::SpellLevels => "SpellLevels".to_owned(),
    }
}

/// An item as a message names it: what it is, its key and its display name.
fn described(item: &Item, names: &BTreeMap<String, String>) -> String {
    let (kind, key) = match item {
        Item::Feature(key) => ("feature", Some(key.as_str())),
        Item::Feat(key) => ("granted feat", Some(key.as_str())),
        Item::Selection(key) => ("selection", Some(key.as_str())),
        Item::FeatFor(value) => return format!("granted feat {}", ron_text(value)),
        Item::Slot(slot) => return format!("feat slot {}", ron_text(slot)),
        Item::SpellLevels => return "spell levels".to_owned(),
    };
    let key = key.unwrap_or_default();
    match names.get(key) {
        Some(name) => format!("{kind} {key} ({name:?})"),
        None => format!("{kind} {key}"),
    }
}

/// What a check of one archetype found: the shortfalls no entry lists (as messages and as the entries to paste),
/// and the entries the rows don't bear out.
#[derive(Debug, Default)]
struct Findings {
    unlisted: String,
    paste: Vec<String>,
    stale: Vec<String>,
}

/// Checks one archetype's rows against its list.
fn check(
    class_key: &str,
    archetype: &ArchetypeKey,
    base: &[ClassLevelRow],
    rows: &[ArchetypeLevelRow],
    names: &BTreeMap<String, String>,
    listed: &[Loss],
) -> Findings {
    let class_file = format!("libs/game-data/classes/{class_key}.ron");
    let base_gives = gives(base);
    let shortfalls = shortfalls(base, rows);
    let mut found = Findings::default();
    let who = format!("{class_key} / {archetype}");

    for (item, levels) in &shortfalls {
        let entry = listed.iter().find(|loss| &loss.item == item);
        let expected: BTreeMap<u8, u8> = match entry {
            None => BTreeMap::new(),
            Some(loss) if loss.at.is_empty() => base_gives
                .get(item)
                .map(|levels| levels.keys().map(|&l| (l, loss.count)).collect())
                .unwrap_or_default(),
            Some(loss) => loss.at.iter().map(|&l| (l, loss.count)).collect(),
        };
        let mut unlisted = BTreeMap::new();
        for (&level, &count) in levels {
            if expected.get(&level) == Some(&count) {
                continue;
            }
            unlisted.insert(level, count);
            let listing = match entry {
                None => "no entry lists it".to_owned(),
                Some(loss) if expected.contains_key(&level) => format!(
                    "the entry lists {} lost there, the rows lose {count}",
                    loss.count
                ),
                Some(_) => "the entry's levels leave this one out".to_owned(),
            };
            let _ = writeln!(
                found.unlisted,
                "{who}, level {level}: the base row gives {} and the archetype's row at level {level} lacks it \
                 ({listing}). Carry it on that row ({class_file}), or list it as lost under {archetype} in \
                 {LIST_FILE} with the guide's reason.",
                described(item, names),
            );
        }
        if !unlisted.is_empty() {
            let every = base_gives
                .get(item)
                .is_some_and(|base_levels| base_levels.keys().eq(levels.keys()));
            let count = levels.values().copied().max().unwrap_or(1);
            let at = if every {
                String::new()
            } else {
                format!(", at: {:?}", levels.keys().collect::<Vec<_>>())
            };
            let count = if count > 1 {
                format!(", count: {count}")
            } else {
                String::new()
            };
            let why = if every { "" } else { ", why: \"...\"" };
            found
                .paste
                .push(format!("(item: {}{at}{count}{why}),", written(item)));
        }
    }

    for loss in listed {
        let made = shortfalls.get(&loss.item);
        let expected: Vec<u8> = if loss.at.is_empty() {
            base_gives
                .get(&loss.item)
                .map(|levels| levels.keys().copied().collect())
                .unwrap_or_default()
        } else {
            loss.at.iter().copied().collect()
        };
        if expected.is_empty() {
            found.stale.push(format!(
                "{who}: the list says it loses {} but the base rows never give it (an entry for \
                 SpellLevels needs `at`).",
                described(&loss.item, names),
            ));
        }
        for level in expected {
            let shortfall = made.and_then(|levels| levels.get(&level)).copied();
            if shortfall != Some(loss.count) {
                let rows_say = match shortfall {
                    None if rows.iter().any(|row| row.level == level) => {
                        "its row at that level keeps it".to_owned()
                    }
                    None => "it has no row at that level, so it keeps it".to_owned(),
                    Some(n) => format!("its row there loses {n}, not {}", loss.count),
                };
                found.stale.push(format!(
                    "{who}, level {level}: the list says it loses {} there, but {rows_say}. Fix the entry in \
                     {LIST_FILE}, or give the archetype a row at that level without it ({class_file}).",
                    described(&loss.item, names),
                ));
            }
        }
    }
    found
}

/// Every archetype of every base class, with its class and its list (none when it has no entry).
fn archetypes_with_lists(
    lists: &[Losses],
) -> impl Iterator<Item = (&'static Class, &'static Archetype, Option<&Losses>)> {
    DATA.classes.iter().flat_map(move |class| {
        class.archetypes.iter().map(move |archetype| {
            let list = lists.iter().find(|l| l.archetype == archetype.key);
            (class, archetype, list)
        })
    })
}

/// The findings of every archetype that has a list, in class order.
fn all_findings() -> Vec<(String, Findings)> {
    let lists = lists();
    archetypes_with_lists(&lists)
        .map(|(class, archetype, list)| {
            let listed = list.map_or(&[][..], |l| &l.loses[..]);
            let found = check(
                class.key.as_str(),
                &archetype.key,
                &class.levels,
                &archetype.levels,
                &names(class, archetype),
                listed,
            );
            (archetype.key.to_string(), found)
        })
        .collect()
}

/// One kind of finding of every archetype, joined.
fn report(findings: &[(String, Findings)], pick: impl Fn(&Findings) -> &String) -> String {
    let mut out = String::new();
    for (_, found) in findings {
        out.push_str(pick(found));
    }
    out
}

/// A row of an archetype that rewrites what the base row gives must carry what the base row gives, or have it listed as
/// lost.
#[test]
fn an_archetype_row_keeps_what_the_base_row_gives_unless_the_list_says_it_is_lost() {
    let findings = all_findings();
    let message = report(&findings, |found| &found.unlisted);
    if message.is_empty() {
        return;
    }
    let mut paste = String::new();
    for (archetype, found) in &findings {
        if found.paste.is_empty() {
            continue;
        }
        let _ = writeln!(paste, "    loses under {archetype}:");
        for entry in &found.paste {
            let _ = writeln!(paste, "        {entry}");
        }
    }
    panic!(
        "{} lines: an archetype row drops what the base row gives:\n{message}\nentries to paste into {LIST_FILE} \
         once the guide bears each out:\n{paste}",
        message.lines().count()
    );
}

/// A listed loss that the rows don't make is as wrong as an unlisted one they do: the list must say what the rows do.
#[test]
fn a_listed_loss_the_rows_do_not_make_fails() {
    let findings = all_findings();
    let stale: Vec<&String> = findings.iter().flat_map(|(_, f)| &f.stale).collect();
    assert!(stale.is_empty(), "stale entries:\n{}", join(&stale));
}

fn join(lines: &[&String]) -> String {
    lines
        .iter()
        .map(|line| line.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every archetype has exactly one list (an archetype that loses nothing has an empty one), every list names an
/// archetype of the data, and each carries its reasons.
#[test]
fn every_archetype_has_one_list_and_every_list_names_an_archetype() {
    let lists = lists();
    let mut problems = Vec::new();
    let known: BTreeSet<&ArchetypeKey> = DATA
        .classes
        .iter()
        .flat_map(|class| class.archetypes.iter().map(|a| &a.key))
        .collect();
    let mut seen = BTreeSet::new();
    for list in &lists {
        if !known.contains(&list.archetype) {
            problems.push(format!("{}: no such archetype", list.archetype));
        }
        if !seen.insert(&list.archetype) {
            problems.push(format!("{}: listed twice", list.archetype));
        }
        let reason = list.why.trim();
        if reason.is_empty() || reason == "TODO" {
            problems.push(format!("{}: no reason (why)", list.archetype));
        }
        let mut items = BTreeSet::new();
        for loss in &list.loses {
            if !items.insert(&loss.item) {
                problems.push(format!(
                    "{}: {} listed twice",
                    list.archetype,
                    written(&loss.item)
                ));
            }
            if loss.count == 0 {
                problems.push(format!(
                    "{}: {} has count 0",
                    list.archetype,
                    written(&loss.item)
                ));
            }
            let needs_why = !loss.at.is_empty() || loss.item == Item::SpellLevels;
            if needs_why && (loss.why.trim().is_empty() || loss.why.trim() == "...") {
                problems.push(format!(
                    "{}: {} at {:?} has no reason (why: what the guide's table shows there)",
                    list.archetype,
                    written(&loss.item),
                    loss.at
                ));
            }
        }
    }
    for archetype in &known {
        if !seen.contains(archetype) {
            problems.push(format!(
                "{archetype}: no list in {LIST_FILE} (an archetype that loses nothing has `loses: []`)"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// ---- the check itself, on rows made up for it --------------------------------------------------------------------

fn base_rows(text: &str) -> Vec<ClassLevelRow> {
    ron::from_str(text).expect("made-up base rows parse")
}

fn archetype_rows(text: &str) -> Vec<ArchetypeLevelRow> {
    ron::from_str(text).expect("made-up archetype rows parse")
}

fn entries(text: &str) -> Vec<Loss> {
    ron::from_str(text).expect("made-up entries parse")
}

/// The message names the class, the archetype, the level and the item, and offers the two ways out; a base row that
/// gains a feature at a level where the archetype has no row fails an entry that says "lost everywhere".
#[test]
fn the_check_names_the_class_the_archetype_the_level_and_the_item() {
    let names = BTreeMap::from([("rage/rage-power".to_owned(), "Rage Power".to_owned())]);
    let key = ArchetypeKey::new("rage/flesh");
    let base = base_rows(
        r#"[
        (level: 1, features: ["rage/rage"]),
        (level: 2, selections: ["rage/rage-power"]),
        (level: 3, features: ["rage/sense"]),
    ]"#,
    );
    let rows = archetype_rows(
        r#"[
        (level: 2, features: ["rage/rage"]),
    ]"#,
    );

    // Unlisted: the message names everything and offers both ways out.
    let found = check("rage", &key, &base, &rows, &names, &[]);
    for text in [
        "rage / rage/flesh, level 2",
        "selection rage/rage-power (\"Rage Power\")",
        "Carry it on that row",
        "list it as lost under rage/flesh",
        "the guide's reason",
    ] {
        assert!(found.unlisted.contains(text), "{text}: {}", found.unlisted);
    }
    assert_eq!(
        found.paste,
        vec!["(item: Selection(\"rage/rage-power\")),".to_owned()]
    );
    assert!(found.stale.is_empty());

    // Listed: nothing to report.
    let listed = entries(r#"[(item: Selection("rage/rage-power"))]"#);
    let found = check("rage", &key, &base, &rows, &names, &listed);
    assert!(found.unlisted.is_empty() && found.stale.is_empty());

    // A listed loss the rows don't make is stale, at the level the entry names.
    let listed = entries(
        r#"[(item: Selection("rage/rage-power")), (item: Feature("rage/rage"), at: [2], why: "x")]"#,
    );
    let found = check("rage", &key, &base, &rows, &names, &listed);
    assert!(found.unlisted.is_empty());
    assert_eq!(found.stale.len(), 1, "{:?}", found.stale);
    assert!(found.stale[0].contains("level 2") && found.stale[0].contains("rage/rage"));

    // The base class gains a feature at 3, where the archetype has no row: "lost everywhere" no longer holds.
    let listed = entries(r#"[(item: Feature("rage/sense"))]"#);
    let found = check("rage", &key, &base, &rows, &names, &listed);
    assert_eq!(found.stale.len(), 1, "{:?}", found.stale);
    assert!(found.stale[0].contains("level 3") && found.stale[0].contains("keeps it"));
}
