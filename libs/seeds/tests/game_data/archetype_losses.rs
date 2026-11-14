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
//! A list entry is `(item, at, count, replaced_by, why)`. With no `at` the archetype loses the item at every level
//! where the base row gives it (for `SpellLevels`, where the base row sets spell levels), so a level where the
//! archetype has no row (it keeps the item) fails the entry, which is how a step added to a base class later is
//! caught. With `at`, exactly those levels, and a `why` saying what the guide's table shows there. `count` is the
//! shortfall at each of them (default one). A listed loss the rows don't make fails as well as an unlisted one the
//! rows do.
//!
//! `replaced_by` says what the archetype gives in the item's place (its own copy of a feature or selection under the
//! same name, or the same name with the archetype's in brackets; its own bonus-feat list for the class's). The test
//! checks that a row of the archetype grants each one at or after the first level the item is lost at: at a lost
//! level for a substitution, later for a delay (the Standard Bearer's companion, lost at 1, comes at 5). It is there so
//! that nobody takes a substitution for a loss and "restores" the base key. Left off where nothing takes the item's
//! place. An archetype's own `why` is optional: it says only what the list can't (a decision of the user's, a guide
//! table that shows more than its overview).
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
    /// What the archetype's rows grant in its place: each is held by a row at or after the first lost level.
    #[serde(default)]
    replaced_by: Vec<Item>,
    /// Required with `at`: what the guide's table shows at those levels.
    #[serde(default)]
    why: String,
}

fn one() -> u8 {
    1
}

/// What one archetype loses.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Losses {
    archetype: ArchetypeKey,
    /// Optional: what the list itself can't say.
    #[serde(default)]
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
        if row.spell_levels().any() {
            gives
                .entry(Item::SpellLevels)
                .or_default()
                .insert(row.level, 1);
        }
    }
    gives
}

/// What an archetype's rows grant, by item and level.
fn rows_grant(rows: &[ArchetypeLevelRow]) -> ByLevel {
    let mut own = ByLevel::new();
    for row in rows {
        let grants = (
            &row.features[..],
            &row.feats,
            &row.feats_for,
            &row.feat_slots[..],
            &row.selections[..],
        );
        tally_row(&mut own, row.level, grants);
    }
    own
}

/// What the base rows give and the archetype's rows lack, by item and level: the shortfall.
fn shortfalls(base: &[ClassLevelRow], rows: &[ArchetypeLevelRow]) -> ByLevel {
    let own = rows_grant(rows);
    let mut shortfalls = ByLevel::new();
    for row in rows {
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
    for (item, levels) in &gives(base) {
        if *item == Item::SpellLevels {
            continue;
        }
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

/// An entry as the list file writes it, with the reasons it already has.
fn entry_text(item: &Item, at: &[u8], count: u8, existing: Option<&Loss>) -> String {
    let mut text = format!("(item: {}", written(item));
    if !at.is_empty() {
        let _ = write!(text, ", at: {at:?}");
    }
    if count > 1 {
        let _ = write!(text, ", count: {count}");
    }
    if let Some(loss) = existing.filter(|loss| !loss.replaced_by.is_empty()) {
        let items: Vec<String> = loss.replaced_by.iter().map(written).collect();
        let _ = write!(text, ", replaced_by: [{}]", items.join(", "));
    }
    match existing {
        Some(loss) if !loss.why.is_empty() => {
            let _ = write!(text, ", why: {:?}", loss.why);
        }
        _ if !at.is_empty() => text.push_str(", why: \"...\""),
        _ => {}
    }
    text.push_str("),");
    text
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
    let own = rows_grant(rows);
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
        let mut unlisted = false;
        for (&level, &count) in levels {
            if expected.get(&level) == Some(&count) {
                continue;
            }
            unlisted = true;
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
        if !unlisted {
            continue;
        }
        let every = base_gives
            .get(item)
            .is_some_and(|base_levels| base_levels.keys().eq(levels.keys()));
        let counts: BTreeSet<u8> = levels.values().copied().collect();
        if counts.len() > 1 {
            found.paste.push(format!(
                "{}: the rows lose different counts by level ({levels:?}), and an entry holds one count per item: \
                 make the rows lose the same count at every level",
                written(item)
            ));
            continue;
        }
        let at: Vec<u8> = if every {
            Vec::new()
        } else {
            levels.keys().copied().collect()
        };
        let count = counts.first().copied().unwrap_or(1);
        let text = entry_text(item, &at, count, entry);
        found.paste.push(match entry {
            Some(_) => format!(
                "replace the existing entry for {} with: {text}",
                written(item)
            ),
            None => text,
        });
    }

    for loss in listed {
        let made = shortfalls.get(&loss.item);
        let base_levels = base_gives.get(&loss.item);
        let named = described(&loss.item, names);
        let expected: Vec<u8> = if loss.at.is_empty() {
            let all: Vec<u8> = base_levels
                .map(|levels| levels.keys().copied().collect())
                .unwrap_or_default();
            if all.is_empty() {
                found.stale.push(format!(
                    "{who}: the list says it loses {named} but the base rows never give it."
                ));
            }
            all
        } else {
            let (given, never): (Vec<u8>, Vec<u8>) = loss
                .at
                .iter()
                .partition(|level| base_levels.is_some_and(|levels| levels.contains_key(level)));
            for level in never {
                found.stale.push(format!(
                    "{who}, level {level}: the entry for {named} names level {level} in `at`, but the base row \
                     gives it nothing there. Fix `at` in {LIST_FILE}."
                ));
            }
            given
        };
        for &level in &expected {
            let shortfall = made.and_then(|levels| levels.get(&level)).copied();
            if shortfall == Some(loss.count) {
                continue;
            }
            let message = match shortfall {
                None if rows.iter().any(|row| row.level == level) => format!(
                    "the list says it loses {named} there, but its row at that level keeps it. Take it off the \
                     row ({class_file}), or fix the entry in {LIST_FILE}."
                ),
                None => format!(
                    "the list says it loses {named} there, but it has no row at that level, so it keeps it. \
                     Either add a row for level {level} that copies the base row less the item (a row replaces \
                     the base row in full, {class_file}), or give the entry `at` with the guide's reason \
                     ({LIST_FILE})."
                ),
                Some(n) => format!(
                    "the list says it loses {named} there ({}), but its row there loses {n}. Fix `count` in \
                     {LIST_FILE}, or the row ({class_file}).",
                    loss.count
                ),
            };
            found.stale.push(format!("{who}, level {level}: {message}"));
        }
        let lost_from = made
            .and_then(|levels| levels.keys().next())
            .or_else(|| expected.first())
            .copied();
        if let Some(first) = lost_from {
            for replacement in &loss.replaced_by {
                let granted = own
                    .get(replacement)
                    .is_some_and(|levels| levels.keys().any(|&level| level >= first));
                if !granted {
                    found.stale.push(format!(
                        "{who}: the entry for {named} says {} replaces it, but no row of the archetype at or \
                         after level {first} (where it first loses it) grants that. Fix `replaced_by` in \
                         {LIST_FILE}, or the rows ({class_file}).",
                        described(replacement, names),
                    ));
                }
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
/// archetype of the data, and each entry with `at` carries its reason.
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
        if list.why.trim() == "TODO" {
            problems.push(format!("{}: a `why` of TODO", list.archetype));
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
            if loss.replaced_by.contains(&loss.item) {
                problems.push(format!(
                    "{}: {} is replaced by itself",
                    list.archetype,
                    written(&loss.item)
                ));
            }
            if !loss.at.is_empty() && (loss.why.trim().is_empty() || loss.why.trim() == "...") {
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

/// An entry's count is exact; the suggestion for an item that already has an entry replaces it, keeping its reasons.
#[test]
fn a_count_of_two_is_exact_and_the_suggestion_replaces_the_existing_entry() {
    let names = BTreeMap::new();
    let key = ArchetypeKey::new("rage/flesh");
    let base = base_rows(r#"[(level: 1, selections: ["rage/blessing", "rage/blessing"])]"#);
    let rows = archetype_rows("[(level: 1)]");

    let one = entries(r#"[(item: Selection("rage/blessing"), why: "kept")]"#);
    let found = check("rage", &key, &base, &rows, &names, &one);
    assert!(
        found
            .unlisted
            .contains("the entry lists 1 lost there, the rows lose 2")
    );
    assert_eq!(
        found.paste,
        vec![
            "replace the existing entry for Selection(\"rage/blessing\") with: (item: Selection(\"rage/blessing\"), \
             count: 2, why: \"kept\"),"
                .to_owned()
        ]
    );

    let two = entries(r#"[(item: Selection("rage/blessing"), count: 2)]"#);
    let found = check("rage", &key, &base, &rows, &names, &two);
    assert!(found.unlisted.is_empty() && found.stale.is_empty());

    // A count the rows don't make is stale.
    let three = entries(r#"[(item: Selection("rage/blessing"), count: 3)]"#);
    let found = check("rage", &key, &base, &rows, &names, &three);
    assert!(
        found.stale[0].contains("(3), but its row there loses 2"),
        "{:?}",
        found.stale
    );
}

/// Spell levels lowered below the base row's are a loss; a bare `SpellLevels` entry means every level where the base
/// row sets them, so a class that starts casting where the archetype has no row is caught.
#[test]
fn lowered_spell_levels_are_a_loss_and_a_bare_entry_covers_every_casting_level() {
    let names = BTreeMap::new();
    let key = ArchetypeKey::new("paladin/guard");
    let base = base_rows(
        r#"[(level: 4, divine_spell_level: 1), (level: 5, divine_spell_level: 1), (level: 6, divine_spell_level: 2)]"#,
    );
    let rows = archetype_rows(
        "[(level: 5, spell_levels: Some((arcane: 0, divine: 0))), (level: 6, spell_levels: Some((arcane: 0, divine: 0)))]",
    );

    let found = check("paladin", &key, &base, &rows, &names, &[]);
    assert!(found.unlisted.contains("level 5") && found.unlisted.contains("spell levels"));
    assert_eq!(
        found.paste,
        vec!["(item: SpellLevels, at: [5, 6], why: \"...\"),".to_owned()]
    );

    // Bare: the base row casts from 4, where the archetype has no row, so it keeps the casting.
    let bare = entries("[(item: SpellLevels)]");
    let found = check("paladin", &key, &base, &rows, &names, &bare);
    assert_eq!(found.stale.len(), 1, "{:?}", found.stale);
    assert!(found.stale[0].contains("level 4") && found.stale[0].contains("copies the base row"));

    let at = entries(r#"[(item: SpellLevels, at: [5, 6], why: "no spells")]"#);
    let found = check("paladin", &key, &base, &rows, &names, &at);
    assert!(
        found.unlisted.is_empty() && found.stale.is_empty(),
        "{found:?}"
    );

    // `at` naming a level where the base row sets no spell levels points at the entry.
    let wrong = entries(r#"[(item: SpellLevels, at: [3, 5, 6], why: "x")]"#);
    let found = check("paladin", &key, &base, &rows, &names, &wrong);
    assert_eq!(found.stale.len(), 1, "{:?}", found.stale);
    assert!(found.stale[0].contains("level 3") && found.stale[0].contains("Fix `at`"));
}

/// A slot of one kind replaced by another at its level is a loss of the base kind, and `replaced_by` is checked
/// against what the rows grant (at or after the first lost level).
#[test]
fn replaced_by_names_what_the_rows_grant_in_the_items_place() {
    let names = BTreeMap::new();
    let key = ArchetypeKey::new("warpriest/dis");
    let base = base_rows(
        r#"[(level: 3, feat_slots: [CombatFeat], features: ["w/channel"]), (level: 9, features: ["w/channel"])]"#,
    );
    let rows = archetype_rows(
        r#"[(level: 3, feat_slots: [OneOf(list: "w/dis/bonus-3")], features: ["w/dis/channel"]),
            (level: 9, features: ["w/dis/channel"])]"#,
    );

    let right = entries(
        r#"[(item: Slot(CombatFeat), replaced_by: [Slot(OneOf(list: "w/dis/bonus-3"))]),
            (item: Feature("w/channel"), replaced_by: [Feature("w/dis/channel")])]"#,
    );
    let found = check("w", &key, &base, &rows, &names, &right);
    assert!(
        found.unlisted.is_empty() && found.stale.is_empty(),
        "{found:?}"
    );

    // Naming something the rows don't grant is stale, and the message says which row to look at.
    let wrong = entries(
        r#"[(item: Slot(CombatFeat), replaced_by: [Slot(OneOf(list: "w/dis/elsewhere"))]),
            (item: Feature("w/channel"))]"#,
    );
    let found = check("w", &key, &base, &rows, &names, &wrong);
    assert_eq!(found.stale.len(), 1, "{:?}", found.stale);
    assert!(
        found.stale[0].contains("w/dis/elsewhere")
            && found.stale[0].contains("at or after level 3"),
        "{:?}",
        found.stale
    );

    // A delay: lost at 3, the copy granted only at 9, still a replacement.
    let delayed_rows = archetype_rows(
        r#"[(level: 3, feat_slots: [CombatFeat]), (level: 9, features: ["w/dis/channel"])]"#,
    );
    let delayed =
        entries(r#"[(item: Feature("w/channel"), replaced_by: [Feature("w/dis/channel")])]"#);
    let found = check("w", &key, &base, &delayed_rows, &names, &delayed);
    assert!(
        found.unlisted.is_empty() && found.stale.is_empty(),
        "{found:?}"
    );
}
