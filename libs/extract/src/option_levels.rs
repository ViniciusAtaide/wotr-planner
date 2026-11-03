//! Options with rows by option level (ticket 30): sorcerer and bloodrager bloodlines, wizard schools, cavalier
//! orders, shaman spirits, the Magic Deceiver's ways and the ranger's combat styles, plus Dragon Disciple, Eldritch
//! Scion and Dragonheir Scion.
//!
//! Each option's rows come from the guide: from its progression table where it has one (the cells every option's
//! table shares at a level are the class's, the rest are the option's; bloodline spells stay out), or from prose
//! where it doesn't (spirits, combat styles). [`provide`] reads every family before any class is built and gives it
//! to the class stage through [`crate::picks`]: its selection, features, lists and nested selections are the owner's
//! from the start, and the class's tables take it as they're read ([`Taking`]):
//! - before a table's rows are read, its cells are planned ([`Taking::plan`]): the choice ("Bloodline", "Order",
//!   "Spirit", "Ways of the Deceiver") and the ability placeholders ("Bloodline Ability", "Order Ability") are
//!   markers, never features (the held option's rows stand there), and so is what a fixed option's row gives where
//!   the archetype's own table lists it; an archetype that takes no family keeps its cells;
//! - once every table of the class is read ([`Taking::take`], on the full rows): the class's row grants the
//!   selection; an option level whose row an archetype's table has no placeholder for (a level its row differs from
//!   its class's at, the markers counted as what they'd be) is skipped, and the archetype's own row stays; an
//!   archetype that skips levels, limits the options or takes only the feat list gets its own selection sharing the
//!   options it allows (one option is granted with no pick); the bonus slots that took the class's merged bloodline
//!   list pick from the held option's list (the lists still count when the class's lists are keyed:
//!   [`crate::feat_lists::PendingLists::stand_in`]); the rows an archetype that takes none shares with its class
//!   follow the class's.
//!
//! The archetype rows are then made sparse as every class's are. Making the decisions on the full rows, before that,
//! gives what deciding on the sparse rows did: a row that matches its class's (its markers included) holds what the
//! class's holds, and the archetype's skipped levels are the ones whose row differs from its class's and has no
//! placeholder.
//!
//! The facts of the families' owners that don't hang on those rows are given with them too: the mythic abilities
//! that pick another option and Bloodline Ascendance, the daily choices, the Primalist's choices, Dragon Disciple,
//! the Eldritch Scion, the Dragonheir Scion, the Elemental Specialist's element and Disruptive's line
//! ([`provide_others`]). [`check`] asserts what only the finished data shows.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    AbilityBonus, ArchetypeLevelRow, Class, ClassKey, DataAt, FeatKey, FeatList, FeatListKey,
    FeatSlot, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, GuideRef,
    OptionLevelRow, OptionRow, Prerequisite, Repeat, SelectionOption, SelectionOptionKey,
    SpellType,
};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::{norm, slug};
use crate::picks::{FeatureFact, Pick, Picks, Rule};
use crate::tables::parse_table;

/// What a pass changed, one line each, for a one-off rewrite's log.
pub type Log = Vec<String>;

/// The families built with their classes, given to the class stage.
fn provide_families(guide: &Guide, feats: &FeatNames, skills: &[rules::Skill], picks: &mut Picks) {
    let mut orders = cavalier_orders(guide, feats);
    crate::option_class_skills::option_rows(guide, &mut orders, skills);
    give(picks, orders, &CAVALIER);
    give(picks, wizard_schools(guide, feats), &WIZARD);
    let mut ways = magic_deceiver_ways(guide, feats);
    crate::option_families::living_deity_rows(&mut ways);
    crate::follow_ups::razmiri_infiltrator(guide, &mut ways);
    give(picks, ways, &MAGIC_DECEIVER);
    give(picks, shaman_spirits(guide), &SHAMAN);
    give(picks, ranger_styles(guide, feats), &RANGER);
    let mut bloodrager = bloodrager_bloodlines(guide, feats);
    crate::second_follow_ups::bloodrager_groups(&mut bloodrager.selection);
    give(picks, bloodrager, &BLOODRAGER);
    picks.selection("bloodrager/primalist", primal_choices(guide));
    picks.unnamed(PRIMAL_CHOICES);
    let mut sorcerer = sorcerer_bloodlines(guide, feats);
    crate::option_follow_ups::sorcerer_bloodlines(guide, &mut sorcerer, picks);
    crate::option_class_skills::option_rows(guide, &mut sorcerer, skills);
    provide_others(&sorcerer, picks);
    give(picks, sorcerer, &SORCERER);
}

/// Gives a family's definitions to its owner (its selection, or the rows of the guide's selection its headings
/// define; its features, lists and nested selections), and how its class's tables take it.
fn give(picks: &mut Picks, family: Family, takes: &'static Takes) {
    let owner = match family.owner {
        Owner::Class(class) => class,
        Owner::Archetype(_, archetype) => archetype,
    };
    let key = family.selection.key.clone();
    // The ranger's combat styles are the guide's selection already: they get their rows.
    if HEADING_SELECTIONS.contains(&key.as_str()) {
        let classes: Vec<&str> = family
            .selection
            .option_levels
            .iter()
            .map(ClassKey::as_str)
            .collect();
        picks.option_levels(key.as_str(), &classes);
        for option in &family.selection.options {
            picks.option_rows(option.key.as_str(), option.levels.clone());
        }
    } else {
        picks.selection(owner, family.selection.clone());
        // Second Spirit's "Prerequisites: Spirit" means a spirit held; the other families' names are no line's.
        if key.as_str() != "shaman/spirit" {
            picks.unnamed(key.as_str());
        }
    }
    for feature in &family.features {
        picks.feature(owner, feature.clone());
    }
    for list in &family.lists {
        picks.feat_list(owner, list.clone());
    }
    for nested in &family.nested {
        picks.selection(owner, nested.clone());
        picks.unnamed(nested.key.as_str());
    }
    picks.family(Taking { family, takes });
}

/// The families' selections the guide's headings define.
const HEADING_SELECTIONS: [&str; 1] = ["ranger/combat-style-feat"];

// ---- families read from the guide ----------------------------------------------------------------------------

/// One family of options: the selection a class grants and the options, features and lists it defines.
#[derive(Clone, Debug)]
pub struct Family {
    /// The class (or, for the Magic Deceiver, the archetype) that defines it.
    pub owner: Owner,
    pub selection: FeatureSelection,
    pub features: Vec<Feature>,
    pub lists: Vec<FeatList>,
    /// Rows from prose that belong with a table row (Strength of the Abyss at 13 and 17 with 9): skipped with it.
    pub ties: BTreeMap<u8, u8>,
    /// Selections the options' rows grant (the Arcane bloodline's school at 15).
    pub nested: Vec<FeatureSelection>,
}

/// Who defines a family.
#[derive(Clone, Debug)]
pub enum Owner {
    Class(&'static str),
    Archetype(&'static str, &'static str),
}

impl Family {
    fn option(&self, key: &str) -> &SelectionOption {
        self.selection
            .options
            .iter()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no option {key}"))
    }

    fn option_keys(&self) -> Vec<SelectionOptionKey> {
        self.selection
            .options
            .iter()
            .map(|option| option.key.clone())
            .collect()
    }

    fn feature_name(&self, key: &FeatureKey) -> Option<&str> {
        self.features
            .iter()
            .find(|feature| &feature.key == key)
            .map(|feature| feature.name.as_str())
    }
}

/// Feat names as the guide writes them, to feat keys.
pub(crate) struct FeatNames(BTreeMap<String, FeatKey>);

impl FeatNames {
    pub(crate) fn new(data: &GameData) -> Self {
        Self::of(&data.feats)
    }

    /// The names of `feats`.
    pub(crate) fn of(feats: &[rules::Feat]) -> Self {
        let mut names = BTreeMap::new();
        for feat in feats {
            names.insert(norm(&feat.name), feat.key.clone());
            names.insert(norm(feat.key.as_str()), feat.key.clone());
            // "Metamagic (Empower Spell)" is written "Empower Spell".
            if let Some(inner) = feat
                .name
                .strip_prefix("Metamagic (")
                .and_then(|rest| rest.strip_suffix(')'))
            {
                names.insert(norm(inner), feat.key.clone());
            }
        }
        Self(names)
    }

    /// The feat a guide name means: "Skill Focus (Lore: Religion)" is Skill Focus, "Point Blank Shot" is
    /// Point-Blank Shot.
    pub(crate) fn get(&self, name: &str) -> Option<FeatKey> {
        let bare = name.split(" (").next().unwrap_or(name).trim();
        let bare = bare.trim_end_matches('.');
        self.0
            .get(&norm(bare))
            .or_else(|| {
                self.0
                    .get(&norm(&bare.replace("Point Blank", "Point-Blank")))
            })
            .cloned()
    }

    /// Every name of a list, each resolved, or a panic naming the ones that aren't.
    pub(crate) fn all(&self, names: &[String], what: &str) -> BTreeSet<FeatKey> {
        let missing: Vec<&String> = names.iter().filter(|n| self.get(n).is_none()).collect();
        assert!(missing.is_empty(), "{what}: no feat named {missing:?}");
        names.iter().filter_map(|name| self.get(name)).collect()
    }
}

/// "A, B, C and D." as names.
pub fn name_list(text: &str) -> Vec<String> {
    let text = text.trim().trim_end_matches('.');
    text.split(", ")
        .flat_map(|part| part.split(" and "))
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}

pub(crate) fn page<'g>(guide: &'g Guide, slug: &str) -> &'g Page {
    guide.page(slug)
}

pub(crate) fn section<'p>(page: &'p Page, anchor: &str) -> &'p Section {
    page.root
        .find(anchor)
        .unwrap_or_else(|| panic!("{}#{anchor} is missing", page.slug))
}

/// The option's table, parsed (row 0 folded into 1).
fn table(page: &Page, anchor: &str) -> Vec<crate::tables::Row> {
    let table = section(page, anchor)
        .tables()
        .next()
        .unwrap_or_else(|| panic!("{}#{anchor} has no table", page.slug));
    parse_table(&table[0], &table[1..]).unwrap_or_else(|error| panic!("{anchor}: {error}"))
}

/// Each option's own cells by level: every table's cells minus those all the family's tables share at that level
/// (a multiset), minus spell cells.
pub fn own_cells(tables: &[Vec<crate::tables::Row>]) -> Vec<BTreeMap<u8, Vec<String>>> {
    let cells = |rows: &Vec<crate::tables::Row>| -> BTreeMap<u8, Vec<String>> {
        rows.iter()
            .map(|row| {
                (
                    row.level,
                    row.grants.iter().map(|grant| grant.name.clone()).collect(),
                )
            })
            .collect()
    };
    let all: Vec<BTreeMap<u8, Vec<String>>> = tables.iter().map(cells).collect();
    let levels: BTreeSet<u8> = all.iter().flat_map(|table| table.keys().copied()).collect();
    let mut common: BTreeMap<u8, Vec<String>> = BTreeMap::new();
    for level in levels {
        let mut shared: Option<Vec<String>> = None;
        for table in &all {
            let here = table.get(&level).cloned().unwrap_or_default();
            shared = Some(match shared {
                None => here,
                Some(shared) => {
                    let mut left = here;
                    shared
                        .into_iter()
                        .filter(|cell| match left.iter().position(|c| c == cell) {
                            Some(at) => {
                                left.remove(at);
                                true
                            }
                            None => false,
                        })
                        .collect()
                }
            });
        }
        common.insert(level, shared.unwrap_or_default());
    }
    all.into_iter()
        .map(|table| {
            table
                .into_iter()
                .map(|(level, cells)| {
                    let mut shared = common.get(&level).cloned().unwrap_or_default();
                    let own = cells
                        .into_iter()
                        .filter(|cell| match shared.iter().position(|c| c == cell) {
                            Some(at) => {
                                shared.remove(at);
                                false
                            }
                            None => true,
                        })
                        .filter(|cell| {
                            let cell = cell.to_ascii_lowercase();
                            // Spells stay out (ticket 29): bloodline spells and the Arcane bloodline's bonus spells.
                            !cell.starts_with("bloodline spell") && cell != "bonus spell"
                        })
                        .collect();
                    (level, own)
                })
                .filter(|(_, cells): &(u8, Vec<String>)| !cells.is_empty())
                .collect()
        })
        .collect()
}

/// The anchor of the heading under `within` that a cell names: "Grow Claws" is "Grow Claws (Abyssal)".
fn anchor_for(within: &Section, name: &str) -> Option<String> {
    let wanted = norm(name);
    let found = within.walk().into_iter().skip(1).find(|section| {
        norm(&section.name) == wanted
            || norm(section.name.split(" (").next().unwrap_or_default()) == wanted
    });
    // "Demonic Arcana" is described under "Abyssal Bloodline Arcana" (the Infernal's spelt "Aracana").
    let arcana = || {
        name.ends_with(" Arcana").then(|| {
            within.walk().into_iter().skip(1).find(|section| {
                section.name.ends_with("Bloodline Arcana")
                    || section.name.ends_with("Bloodline Aracana")
            })
        })?
    };
    found.or_else(arcana).map(|section| section.anchor.clone())
}

/// The names an option row grants, for comparing with an archetype's table: its features' and its selections'.
fn row_names(family: &Family, row: &OptionLevelRow) -> Vec<String> {
    row.features
        .iter()
        .map(|f| bare(family.feature_name(f).unwrap_or_default()))
        .chain(row.selections.iter().map(|s| {
            bare(
                family
                    .nested
                    .iter()
                    .find(|nested| &nested.key == s)
                    .map_or("", |nested| nested.name.as_str()),
            )
        }))
        .collect()
}

/// Builds an option's rows from its cells: a cell naming a feat is a granted feat, anything else a feature keyed
/// under `features_under` (shared by the options of one group, such as the ten dragons).
struct RowBuilder<'a> {
    feats: &'a FeatNames,
    features: &'a mut Vec<Feature>,
}

impl RowBuilder<'_> {
    fn rows(
        &mut self,
        cells: &BTreeMap<u8, Vec<String>>,
        features_under: &str,
        guide_for: &dyn Fn(&str) -> String,
    ) -> Vec<OptionLevelRow> {
        cells
            .iter()
            .map(|(level, cells)| {
                let mut row = empty_row(*level);
                for cell in cells {
                    // Only Wings is a feat among these cells.
                    if cell == "Wings"
                        && let Some(feat) = self.feats.get(cell)
                    {
                        row.feats.insert(feat);
                        continue;
                    }
                    let key = FeatureKey::new(format!("{features_under}/{}", slug(cell)));
                    if !self.features.iter().any(|feature| feature.key == key) {
                        self.features.push(feature(&key, cell, guide_for(cell)));
                    }
                    row.features.push(key);
                }
                row
            })
            .collect()
    }
}

pub(crate) fn empty_row(level: u8) -> OptionLevelRow {
    OptionLevelRow {
        level,
        ..OptionLevelRow::default()
    }
}

pub(crate) fn feature(key: &FeatureKey, name: &str, guide: String) -> Feature {
    Feature {
        key: key.clone(),
        name: name.to_owned(),
        summary: String::new(),
        guide: GuideRef::new(guide),
        ability_bonuses: Vec::new(),
        option_row: None,
        daily_choice: false,
        bab_floor: None,
        skill_ranks_at_least_mythic_rank: false,
    }
}

pub(crate) fn option(
    key: &str,
    name: &str,
    guide: String,
    levels: Vec<OptionLevelRow>,
) -> SelectionOption {
    SelectionOption {
        key: SelectionOptionKey::new(key),
        name: name.to_owned(),
        summary: String::new(),
        guide: GuideRef::new(guide),
        feats: BTreeSet::new(),
        chosen_feat: None,
        repeat: Repeat::Once,
        prerequisite: None,
        levels,
        pool: None,
        replaces_row_of: None,
        group: None,
        arcana: None,
        parameter: None,
        values: Default::default(),
    }
}

pub(crate) fn selection(key: &str, name: &str, guide: String, levels: &[&str]) -> FeatureSelection {
    FeatureSelection {
        key: FeatureSelectionKey::new(key),
        name: name.to_owned(),
        summary: String::new(),
        guide: GuideRef::new(guide),
        options: Vec::new(),
        shared_options: BTreeSet::new(),
        option_levels: levels.iter().map(|class| ClassKey::new(*class)).collect(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    }
}

pub(crate) fn list(
    key: &str,
    name: &str,
    guide: String,
    ignores: bool,
    feats: BTreeSet<FeatKey>,
) -> FeatList {
    FeatList {
        key: FeatListKey::new(key),
        name: name.to_owned(),
        guide: GuideRef::new(guide),
        ignores_prerequisites: ignores,
        every_feat: false,
        feats,
    }
}

/// Puts the list on the option's first row (creating it if it has none at that level).
fn set_list(option: &mut SelectionOption, level: u8, list: &FeatListKey) {
    match option.levels.iter_mut().find(|row| row.level == level) {
        Some(row) => row.feat_list = Some(list.clone()),
        None => {
            option.levels.push(OptionLevelRow {
                feat_list: Some(list.clone()),
                ..empty_row(level)
            });
            option.levels.sort_by_key(|row| row.level);
        }
    }
}

/// "Bloodline feats: A, B, C." (or "Bonus Feats:") in a character-creation bloodline section.
fn bloodline_feats(section: &Section) -> Vec<String> {
    let line = section
        .paragraphs()
        .flatten()
        .chain(section.list_items())
        .find_map(|line| {
            let lower = line.to_ascii_lowercase();
            ["bloodline feats:", "bonus feats:"]
                .iter()
                .find_map(|prefix| {
                    lower
                        .starts_with(prefix)
                        .then(|| line[prefix.len()..].to_owned())
                })
        })
        .unwrap_or_else(|| panic!("{} lists no bloodline feats", section.anchor));
    name_list(&line)
}

/// A two-column table under a section ("Dragon | Element"), as (left cell, right cell) rows without the header.
fn pairs(section: &Section, header: &str) -> Vec<(String, String)> {
    let table = section
        .walk()
        .into_iter()
        .flat_map(Section::tables)
        .find(|table| table[0][0].trim() == header)
        .unwrap_or_else(|| panic!("{} has no {header} table", section.anchor));
    table[1..]
        .iter()
        .map(|row| (row[0].trim().to_owned(), row[1].trim().to_owned()))
        .collect()
}

// ---- sorcerer bloodlines -----------------------------------------------------------------------------------

/// The guide's bloodline tables on the sorcerer page, in its order.
const SORCERER_TABLES: [&str; 9] = [
    "abyssal",
    "arcane",
    "celestial",
    "draconic",
    "elemental",
    "fey",
    "infernal",
    "serpentine",
    "undead",
];

/// "Every Draconic bloodline has certain characteristics in common" and "The element linked to each specific
/// bloodline is shown in the following table" (character-creation): one option per dragon and per element, rows
/// and list shared.
fn subtypes(guide: &Guide, table: &str) -> Option<Vec<String>> {
    let creation = page(guide, "character-creation");
    match table {
        "draconic" => Some(
            pairs(section(creation, "draconic-bloodline"), "Dragon")
                .into_iter()
                .flat_map(|(dragons, _)| name_list(&dragons))
                .collect(),
        ),
        "elemental" => Some(
            pairs(section(creation, "elemental-bloodline"), "Elemental")
                .into_iter()
                .map(|(element, _)| element)
                .collect(),
        ),
        _ => None,
    }
}

fn sorcerer_bloodlines(guide: &Guide, feats: &FeatNames) -> Family {
    let sorcerer = page(guide, "sorcerer");
    let creation = page(guide, "character-creation");
    let tables: Vec<_> = SORCERER_TABLES.iter().map(|b| table(sorcerer, b)).collect();
    let cells = own_cells(&tables);
    let mut family = Family {
        owner: Owner::Class("sorcerer"),
        selection: selection(
            "sorcerer/bloodline",
            "Bloodline",
            creation.guide("bloodlines"),
            &["sorcerer", "dragon-disciple"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for (bloodline, cells) in SORCERER_TABLES.iter().zip(&cells) {
        let described = section(creation, &format!("{bloodline}-bloodline"));
        let cells = &school_power(creation, bloodline, cells, &mut family.nested);
        let under = format!("sorcerer/bloodline/{bloodline}");
        let guide_for = |cell: &str| match anchor_for(described, cell) {
            Some(anchor) => creation.guide(&anchor),
            None => sorcerer.guide(bloodline),
        };
        let mut builder = RowBuilder {
            feats,
            features: &mut family.features,
        };
        let mut rows = builder.rows(cells, &under, &guide_for);
        if *bloodline == "arcane" {
            let school = FeatureSelectionKey::new("sorcerer/bloodline/arcane/school-power");
            match rows.iter_mut().find(|row| row.level == 15) {
                Some(row) => row.selections.push(school),
                None => rows.push(OptionLevelRow {
                    selections: vec![school],
                    ..empty_row(15)
                }),
            }
            rows.sort_by_key(|row| row.level);
        }
        if *bloodline == "abyssal" {
            strength_of_the_abyss(described, &mut rows, &mut family.features, &mut family.ties);
        }
        let list_key = format!("{under}/feats");
        let name = section(sorcerer, bloodline).name.clone();
        family.lists.push(list(
            &list_key,
            &format!("{name} Bloodline Feats"),
            creation.guide(&described.anchor),
            false,
            feats.all(&bloodline_feats(described), &list_key),
        ));
        let options: Vec<(String, String)> = match subtypes(guide, bloodline) {
            Some(kinds) => kinds
                .iter()
                .map(|kind| {
                    (
                        format!("{bloodline}-{}", slug(kind)),
                        format!("{name} ({kind})"),
                    )
                })
                .collect(),
            None => vec![(bloodline.to_string(), name.clone())],
        };
        for (key, option_name) in options {
            let mut option = option(
                &format!("sorcerer/bloodline/{key}"),
                &option_name,
                sorcerer.guide(bloodline),
                rows.clone(),
            );
            set_list(&mut option, 1, &FeatListKey::new(&list_key));
            family.selection.options.push(option);
        }
    }
    family
}

/// "At 9th level, you gain a +2 inherent bonus to Strength. This increases to +4 at level 13 and +6 at level 17."
/// One feature, +2 Strength, granted at 9, 13 and 17.
fn strength_of_the_abyss(
    described: &Section,
    rows: &mut Vec<OptionLevelRow>,
    features: &mut [Feature],
    ties: &mut BTreeMap<u8, u8>,
) {
    let text = section_text(described, "strength-of-the-abyss");
    assert!(
        text.contains("+2 inherent bonus to Strength")
            && text.contains("+4 at level 13 and +6 at level 17"),
        "Strength of the Abyss reads differently: {text}"
    );
    let key = FeatureKey::new("sorcerer/bloodline/abyssal/strength-of-the-abyss");
    assert!(
        rows.iter()
            .any(|row| row.level == 9 && row.features.contains(&key)),
        "no Strength of the Abyss at 9"
    );
    let feature = features
        .iter_mut()
        .find(|feature| feature.key == key)
        .expect("Strength of the Abyss");
    feature.ability_bonuses = vec![AbilityBonus::Fixed {
        ability: rules::Ability::Strength,
        amount: 2,
    }];
    for level in [13, 17] {
        match rows.iter_mut().find(|row| row.level == level) {
            Some(row) => row.features.push(key.clone()),
            None => rows.push(OptionLevelRow {
                features: vec![key.clone()],
                ..empty_row(level)
            }),
        }
        ties.insert(level, 9);
    }
    rows.sort_by_key(|row| row.level);
}

/// "At 15th level, pick one school of magic. The DC for any spells you cast from this school increases by +2"
/// (School Power, Arcane bloodline): a choice inside the option, so a selection its 15th-level row grants, one
/// option per school.
fn school_power(
    creation: &Page,
    bloodline: &str,
    cells: &BTreeMap<u8, Vec<String>>,
    nested: &mut Vec<FeatureSelection>,
) -> BTreeMap<u8, Vec<String>> {
    if bloodline != "arcane" {
        return cells.clone();
    }
    let text = section_text(&creation.root, "school-power");
    assert!(text.starts_with("At 15th level, pick one school of magic."));
    let key = "sorcerer/bloodline/arcane/school-power";
    let mut selection = selection(key, "School Power", creation.guide("school-power"), &[]);
    for school in rules::SpellSchool::ALL {
        let name = format!("{school:?}");
        selection.options.push(option(
            &format!("{key}/{}", slug(&name)),
            &name,
            creation.guide("school-power"),
            Vec::new(),
        ));
    }
    nested.push(selection);
    let mut cells = cells.clone();
    let at_15 = cells.get_mut(&15).expect("the Arcane bloodline at 15");
    let before = at_15.len();
    at_15.retain(|cell| cell != "School Power");
    assert_eq!(at_15.len() + 1, before, "School Power at 15");
    cells
}

fn section_text(within: &Section, anchor: &str) -> String {
    within
        .find(anchor)
        .unwrap_or_else(|| panic!("no {anchor}"))
        .text()
}

// ---- bloodrager bloodlines -----------------------------------------------------------------------------------

const BLOODRAGER_TABLES: [&str; 10] = [
    "abyssal",
    "arcane",
    "celestial",
    "draconic",
    "elemental",
    "fey",
    "hag",
    "infernal",
    "serpentine",
    "undead",
];

fn bloodrager_bloodlines(guide: &Guide, feats: &FeatNames) -> Family {
    let bloodrager = page(guide, "bloodrager");
    let anchors: Vec<String> = BLOODRAGER_TABLES
        .iter()
        .map(|b| format!("{b}-bloodrager"))
        .collect();
    let tables: Vec<_> = anchors.iter().map(|a| table(bloodrager, a)).collect();
    let cells = own_cells(&tables);
    let mut family = Family {
        owner: Owner::Class("bloodrager"),
        selection: selection(
            "bloodrager/bloodline",
            "Bloodline",
            page(guide, "character-creation").guide("bloodlines"),
            &["bloodrager"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for ((bloodline, anchor), cells) in BLOODRAGER_TABLES.iter().zip(&anchors).zip(&cells) {
        let described = section(bloodrager, anchor);
        let name = described
            .name
            .strip_suffix(" Bloodrager")
            .unwrap_or(&described.name)
            .to_owned();
        let under = format!("bloodrager/bloodline/{bloodline}");
        let guide_for = |cell: &str| {
            bloodrager.guide(&anchor_for(described, cell).unwrap_or_else(|| anchor.clone()))
        };
        let mut builder = RowBuilder {
            feats,
            features: &mut family.features,
        };
        let rows = builder.rows(cells, &under, &guide_for);
        // "Bonus Feats (Abyssal)": "The Bloodrager may choose from the following feats: A, B and C."
        let bonus = described
            .children
            .iter()
            .find(|child| child.name.starts_with("Bonus Feats"))
            .unwrap_or_else(|| panic!("{anchor} has no bonus feats"));
        let text = bonus.text();
        let names = text
            .split("following feats:")
            .nth(1)
            .unwrap_or_else(|| panic!("{}: {text}", bonus.anchor));
        let list_key = format!("{under}/feats");
        family.lists.push(list(
            &list_key,
            &format!("{name} Bloodline Feats"),
            bloodrager.guide(&bonus.anchor),
            false,
            feats.all(&name_list(names), &list_key),
        ));
        let options: Vec<(String, String)> = match subtypes(guide, bloodline) {
            Some(kinds) => kinds
                .iter()
                .map(|kind| {
                    (
                        format!("{bloodline}-{}", slug(kind)),
                        format!("{name} ({kind})"),
                    )
                })
                .collect(),
            None => vec![(bloodline.to_string(), name.clone())],
        };
        for (key, option_name) in options {
            let mut option = option(
                &format!("bloodrager/bloodline/{key}"),
                &option_name,
                bloodrager.guide(anchor),
                rows.clone(),
            );
            set_list(&mut option, 1, &FeatListKey::new(&list_key));
            family.selection.options.push(option);
        }
    }
    family
}

// ---- wizard schools, cavalier orders, Magic Deceiver ways ----------------------------------------------------

/// A family whose every option has a table and its features under headings below it.
fn tabled(
    guide: &Guide,
    feats: &FeatNames,
    page_slug: &str,
    owner: Owner,
    selection: FeatureSelection,
    anchors: &[&str],
) -> Family {
    let page = page(guide, page_slug);
    let tables: Vec<_> = anchors.iter().map(|a| table(page, a)).collect();
    let cells = own_cells(&tables);
    let mut family = Family {
        owner,
        selection,
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for (anchor, cells) in anchors.iter().zip(&cells) {
        let described = section(page, anchor);
        let key = format!("{}/{anchor}", family.selection.key);
        let guide_for = |cell: &str| {
            page.guide(&anchor_for(described, cell).unwrap_or_else(|| anchor.to_string()))
        };
        let mut builder = RowBuilder {
            feats,
            features: &mut family.features,
        };
        let rows = builder.rows(cells, &key, &guide_for);
        family
            .selection
            .options
            .push(option(&key, &described.name, page.guide(anchor), rows));
    }
    family
}

const SCHOOLS: [&str; 9] = [
    "universalist",
    "abjurer",
    "conjurer",
    "diviner",
    "enchanter",
    "illusionist",
    "invoker",
    "necromancer",
    "transmuter",
];

fn wizard_schools(guide: &Guide, feats: &FeatNames) -> Family {
    let wizard = page(guide, "wizard");
    let mut family = tabled(
        guide,
        feats,
        "wizard",
        Owner::Class("wizard"),
        selection(
            "wizard/school",
            "School",
            wizard.guide("wizard"),
            &["wizard"],
        ),
        &SCHOOLS,
    );
    // "When you prepare spells, choose an energy type and gain resistance 5": a daily choice.
    let resistance = section_text(&wizard.root, "resistance");
    assert!(resistance.starts_with("When you prepare spells, choose an energy type"));
    for feature in &mut family.features {
        if feature.key.as_str() == "wizard/school/abjurer/resistance" {
            feature.daily_choice = true;
        }
    }
    family
}

const ORDERS: [&str; 5] = [
    "order-of-the-cockatrice",
    "order-of-the-lion",
    "order-of-the-shroud",
    "order-of-the-star",
    "order-of-the-sword",
];

fn cavalier_orders(guide: &Guide, feats: &FeatNames) -> Family {
    let cavalier = page(guide, "cavalier");
    let mut family = tabled(
        guide,
        feats,
        "cavalier",
        Owner::Class("cavalier"),
        selection(
            "cavalier/order",
            "Order",
            cavalier.guide("order"),
            &["cavalier"],
        ),
        &ORDERS,
    );
    // Braggart: "At 2nd level, the Cavalier receives Dazzling Display as a bonus feat."
    let braggart = section_text(&cavalier.root, "braggart");
    assert!(braggart.contains("receives Dazzling Display as a bonus feat"));
    let feat = feats.get("Dazzling Display").expect("Dazzling Display");
    let cockatrice = family
        .selection
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "cavalier/order/order-of-the-cockatrice")
        .expect("Cockatrice");
    let row = cockatrice
        .levels
        .iter_mut()
        .find(|row| {
            row.features
                .iter()
                .any(|f| f.as_str().ends_with("/braggart"))
        })
        .expect("Braggart's row");
    row.feats.insert(feat);
    family
}

const WAYS: [&str; 6] = [
    "fake-celebrity",
    "living-deity",
    "magic-inventor",
    "razmiri-infiltrator",
    "time-manipulator",
    "way-of-the-charlatan",
];

fn magic_deceiver_ways(guide: &Guide, feats: &FeatNames) -> Family {
    let arcanist = page(guide, "arcanist");
    tabled(
        guide,
        feats,
        "arcanist",
        Owner::Archetype("arcanist", "arcanist/magic-deceiver"),
        selection(
            "arcanist/magic-deceiver/ways-of-the-deceiver",
            "Ways of the Deceiver",
            arcanist.guide("ways-of-the-deceiver"),
            &["arcanist"],
        ),
        &WAYS,
    )
}

// ---- shaman spirits and ranger styles (prose) -----------------------------------------------------------------

/// "A Shaman's choice of Spirit grants him abilities": under each spirit's heading, its ability (1st level), then
/// the ones from 8th and 16th level, and its Manifestation (20th), each stating its level.
fn shaman_spirits(guide: &Guide) -> Family {
    let shaman = page(guide, "shaman");
    let spirits = section(shaman, "spirits");
    let mut family = Family {
        owner: Owner::Class("shaman"),
        selection: selection(
            "shaman/spirit",
            "Spirit",
            shaman.guide("spirit"),
            &["shaman"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for spirit in &spirits.children {
        let key = format!("shaman/spirit/{}", spirit.anchor);
        let mut rows: Vec<OptionLevelRow> = Vec::new();
        for (index, ability) in spirit
            .children
            .iter()
            .filter(|child| !child.name.ends_with("Bonus Spells"))
            .enumerate()
        {
            let text = ability.text();
            // "At 20th level, a Shaman fully unlocks the power of his guardian spirit" (Manifestation).
            let level = if index == 0 {
                1
            } else if ability.name.starts_with("Manifestation") {
                20
            } else {
                [8u8, 16, 20]
                    .into_iter()
                    .find(|level| text.starts_with(&format!("At {level}th level")))
                    .unwrap_or_else(|| panic!("{}: no level in {text:?}", ability.anchor))
            };
            let name = ability.name.clone();
            let feature_key = FeatureKey::new(format!("{key}/{}", slug(&name)));
            family
                .features
                .push(feature(&feature_key, &name, shaman.guide(&ability.anchor)));
            rows.push(OptionLevelRow {
                features: vec![feature_key],
                ..empty_row(level)
            });
        }
        let levels: Vec<u8> = rows.iter().map(|row| row.level).collect();
        assert_eq!(levels, [1, 8, 16, 20], "{}", spirit.anchor);
        family.selection.options.push(option(
            &key,
            &spirit.name,
            shaman.guide(&spirit.anchor),
            rows,
        ));
    }
    assert_eq!(family.selection.options.len(), 9);
    family
}

/// "At 2nd level, the Ranger may choose from A, B and C. At 6th level, D and E are added. At 10th level, F is
/// added." Each style's list grows at 2, 6 and 10 with the style's feats and, as "Besides the style feats, you can
/// also select from Dodge, Great Fortitude, Iron Will, Lightning Reflexes and Toughness. From 10th level, you can
/// select a Skill Focus", those too. A style feat ignores prerequisites ("does not need to meet the
/// prerequisites").
fn ranger_styles(guide: &Guide, feats: &FeatNames) -> Family {
    let ranger = page(guide, "ranger");
    let styles = section(ranger, "combat-style-feat");
    let intro = styles.text();
    assert!(intro.contains("does not need to meet the prerequisites"));
    let besides = intro
        .split("you can also select from ")
        .nth(1)
        .and_then(|rest| rest.split(". ").next())
        .expect("the ranger's extra feats");
    let extras = feats.all(&name_list(besides), "the ranger's extra feats");
    assert!(intro.contains("From 10th level, you can select a Skill Focus"));
    let skill_focus = feats.get("Skill Focus").expect("Skill Focus");
    let mut family = Family {
        owner: Owner::Class("ranger"),
        selection: selection(
            "ranger/combat-style-feat",
            "Combat Style Feat",
            ranger.guide("combat-style-feat"),
            &["ranger"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for style in &styles.children {
        let key = format!("ranger/combat-style-feat/{}", style.anchor);
        let mut held = BTreeSet::new();
        let mut rows = Vec::new();
        for sentence in style.text().split(". ") {
            let Some(rest) = sentence.trim().strip_prefix("At ") else {
                continue;
            };
            let (level, rest) = rest
                .split_once("th level, ")
                .or_else(|| rest.split_once("nd level, "))
                .expect(sentence);
            let level: u8 = level.parse().expect(sentence);
            let names = if let Some(names) = rest.split(" from ").nth(1) {
                names
            } else {
                rest.trim_end_matches('.')
                    .trim_end_matches(" are added")
                    .trim_end_matches(" is added")
            };
            held.extend(feats.all(&name_list(names), &key));
            let mut feats_now = held.clone();
            feats_now.extend(extras.iter().cloned());
            if level >= 10 {
                feats_now.insert(skill_focus.clone());
            }
            let list_key = format!("{key}-{level}");
            family.lists.push(list(
                &list_key,
                &format!("{} (from level {level})", style.name),
                ranger.guide(&style.anchor),
                true,
                feats_now,
            ));
            rows.push(OptionLevelRow {
                feat_list: Some(FeatListKey::new(&list_key)),
                ..empty_row(level)
            });
        }
        let levels: Vec<u8> = rows.iter().map(|row| row.level).collect();
        assert_eq!(levels, [2, 6, 10], "{}", style.anchor);
        family
            .selection
            .options
            .push(option(&key, &style.name, ranger.guide(&style.anchor), rows));
    }
    family
}

// ---- taking a family -----------------------------------------------------------------------------------------

/// How a class and its archetypes take a family.
pub(crate) struct Takes {
    /// The class whose tables take it.
    pub class: &'static str,
    /// The class's level that grants the selection.
    level: u8,
    /// The table cell that is the choice itself ("Bloodline"); it becomes the selection.
    choices: &'static [&'static str],
    /// Whether the choice cells' features stay defined (off the rows): a line elsewhere resolves to them.
    kept_choices: bool,
    /// The table cells the option's rows replace ("Bloodline Ability").
    placeholders: &'static [&'static str],
    /// Whether an archetype that takes the family skips the option levels its table has no placeholder for.
    skips_without_placeholder: bool,
    /// The tables whose bonus slots' lists are those merged lists: a slot naming one picks from the held option's
    /// list instead.
    merged: Merged,
    /// The base class takes the family (the Magic Deceiver's ways are the archetype's alone).
    base_takes: bool,
    archetypes: &'static [(&'static str, Take)],
    /// A placeholder that is also a choice in an archetype's table (archetype, cell, the selection it grants), at
    /// these levels.
    row_choices: &'static [(&'static str, &'static str, &'static str, &'static [u8])],
}

/// Which tables' bonus lists are a family's merged lists.
#[derive(Clone, Copy, Debug)]
enum Merged {
    None,
    /// Every table of the class.
    Every,
    /// These tables.
    Tables(&'static [&'static str]),
}

/// How one archetype takes a family, where its table doesn't say by itself.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Take {
    /// Not at all.
    No,
    /// The family, coverage by placeholders (or every level, for a family that doesn't skip).
    Yes,
    /// Only these options (the guide's list), coverage by placeholders.
    Only(&'static [&'static str]),
    /// One option, fixed: its rows where the archetype's table lists the same features, the archetype's own
    /// elsewhere.
    Fixed(&'static str),
    /// Only the feat list of a second option (Crossblooded), from the archetype's own choice cell.
    SecondListOnly(&'static str),
}

const SORCERER: Takes = Takes {
    class: "sorcerer",
    level: 1,
    choices: &[],
    kept_choices: false,
    placeholders: &["Bloodline Ability", "Bloodline Arcana"],
    skips_without_placeholder: true,
    merged: Merged::Every,
    base_takes: true,
    // "Note that Empyreal Sorcerers, Sage Sorcerers and Sylvan Sorcerers have fixed bloodlines" (character-creation):
    // "identical to a Sorcerer who has selected the inferior Celestial bloodline", "more or less the same as an
    // Arcane Sorcerer", "the very similar Fey Sorcerer" (sorcerer). Crossblooded: "Bloodline 2 ... Arcana, Bloodline
    // spells, Bloodline feats", of which a second pick keeps only the feats (ticket 29).
    archetypes: &[
        ("sorcerer/crossblooded", Take::SecondListOnly("Bloodline 2")),
        (
            "sorcerer/empyreal-sorcerer",
            Take::Fixed("sorcerer/bloodline/celestial"),
        ),
        ("sorcerer/geomancer", Take::Yes),
        ("sorcerer/nine-tailed-heir", Take::Yes),
        ("sorcerer/overwhelming-mage", Take::Yes),
        (
            "sorcerer/sage-sorcerer",
            Take::Fixed("sorcerer/bloodline/arcane"),
        ),
        ("sorcerer/seeker-sorcerer", Take::Yes),
        (
            "sorcerer/sylvan-sorcerer",
            Take::Fixed("sorcerer/bloodline/fey"),
        ),
    ],
    row_choices: &[],
};

const BLOODRAGER: Takes = Takes {
    class: "bloodrager",
    level: 1,
    choices: &["Bloodline"],
    kept_choices: true,
    // A Primalist's "Bloodline Ability or Rage Powers" is the bloodline row unless its choice replaces it.
    placeholders: &["Bloodline Ability", "Bloodline Ability or Rage Powers"],
    skips_without_placeholder: true,
    merged: Merged::Every,
    base_takes: true,
    // "A Hag-Riven can choose from the following bloodlines: Arcane, Elemental, Fey and Hag." "Bloodline must be
    // Abyssal or Infernal" (Reformed Fiend). The Mixed-Blood Rager "Can choose any bloodline each day": a daily
    // choice, no pick (ticket 29). The Primalist's "Bloodline Ability or Rage Powers" is its own choice (below).
    archetypes: &[
        (
            "bloodrager/hag-riven",
            Take::Only(&["arcane", "elemental", "fey", "hag"]),
        ),
        ("bloodrager/mixed-blood-rager", Take::No),
        (
            "bloodrager/reformed-fiend",
            Take::Only(&["abyssal", "infernal"]),
        ),
    ],
    row_choices: &[(
        "bloodrager/primalist",
        "Bloodline Ability or Rage Powers",
        PRIMAL_CHOICES,
        &[4, 8, 12, 16, 20],
    )],
};

const WIZARD: Takes = Takes {
    class: "wizard",
    level: 1,
    choices: &[],
    kept_choices: false,
    placeholders: &["Specialisation abilities", "Specialisation ability"],
    skips_without_placeholder: false,
    merged: Merged::None,
    base_takes: true,
    // "Does not get an Arcane Bond ... Must select four Opposition Schools" (Arcane Bomber: no specialisation in
    // its table), "Always a specialist Necromancer" (Cruromancer), "Always a specialist Invoker" (Elemental
    // Specialist), "Cannot choose a specialisation" (Exploiter), "May select a specialisation or no
    // specialisation like a regular Wizard" (Scroll Savant), "May select a specialisation" (Shadowcaster), "Always a
    // Universalist" (Spell Master), the Thassilonian's seven schools (its table of spellbooks).
    archetypes: &[
        ("wizard/arcane-bomber", Take::No),
        (
            "wizard/cruromancer",
            Take::Fixed("wizard/school/necromancer"),
        ),
        (
            "wizard/elemental-specialist",
            Take::Fixed("wizard/school/invoker"),
        ),
        ("wizard/exploiter-wizard", Take::No),
        ("wizard/scroll-savant", Take::Yes),
        ("wizard/shadowcaster", Take::Yes),
        (
            "wizard/spell-master",
            Take::Fixed("wizard/school/universalist"),
        ),
        (
            "wizard/thassilonian-specialist",
            Take::Only(&[
                "abjurer",
                "necromancer",
                "transmuter",
                "enchanter",
                "illusionist",
                "conjurer",
                "invoker",
            ]),
        ),
    ],
    row_choices: &[],
};

/// Notes for an archetype's own selection: the Thassilonian's spellbooks, by the school each specialises in.
const SUMMARIES: [(&str, &str); 1] = [(
    "wizard/thassilonian-specialist",
    "Spellbooks: Envy (Abjurer), Gluttony (Necromancer), Greed (Transmuter), Lust (Enchanter), Pride \
     (Illusionist), Sloth (Conjurer), Wrath (Invoker).",
)];

const CAVALIER: Takes = Takes {
    class: "cavalier",
    level: 1,
    choices: &["Order"],
    kept_choices: false,
    placeholders: &["Order Ability"],
    skips_without_placeholder: true,
    merged: Merged::None,
    base_takes: true,
    archetypes: &[],
    row_choices: &[],
};

const MAGIC_DECEIVER: Takes = Takes {
    class: "arcanist",
    level: 1,
    choices: &["Ways of the Deceiver"],
    kept_choices: false,
    placeholders: &[],
    skips_without_placeholder: false,
    merged: Merged::None,
    base_takes: false,
    archetypes: &[("arcanist/magic-deceiver", Take::Yes)],
    row_choices: &[],
};

const SHAMAN: Takes = Takes {
    class: "shaman",
    level: 1,
    choices: &["Spirit"],
    kept_choices: false,
    placeholders: &[
        "Spirit Ability",
        "Greater Spirit Ability",
        "True Spirit Ability",
        "Manifestation",
    ],
    skips_without_placeholder: true,
    merged: Merged::None,
    base_takes: true,
    // "Guiding spirit is the Battle spirit" (Wildland Shaman).
    archetypes: &[(
        "shaman/wildland-shaman",
        Take::Fixed("shaman/spirit/battle"),
    )],
    row_choices: &[],
};

const RANGER: Takes = Takes {
    class: "ranger",
    level: 2,
    choices: &[],
    kept_choices: false,
    placeholders: &[],
    skips_without_placeholder: false,
    merged: Merged::Tables(&["ranger"]),
    base_takes: true,
    // Every archetype's table has the "Combat Style Feat" cells but the Nomad's and the Stormwalker's, which take
    // the Archery feats only ("may only select Archery feats").
    archetypes: &[
        ("ranger/demonslayer", Take::Yes),
        ("ranger/espionage-expert", Take::Yes),
        ("ranger/flamewarden", Take::Yes),
        ("ranger/freebooter", Take::Yes),
        ("ranger/nomad", Take::No),
        ("ranger/sable-company-marine", Take::Yes),
        ("ranger/stormwalker", Take::No),
    ],
    row_choices: &[],
};

/// Primalist: "At 4th level and every four levels thereafter, a Primalist can choose to take his bloodline ability
/// or two Rage Powers." A choice at each "Bloodline Ability or Rage Powers" cell (`Takes::row_choices`): the
/// bloodline's row, or two rage powers that skip it.
fn primal_choices(guide: &Guide) -> FeatureSelection {
    let text = section_text(&page(guide, "bloodrager").root, "primal-choices");
    assert!(text.contains("choose to take his bloodline ability or two Rage Powers"));
    let rage_power = FeatureSelectionKey::new("barbarian/rage-power");
    FeatureSelection {
        options: vec![
            option(
                "bloodrager/primalist/primal-choices/bloodline-ability",
                "Bloodline Ability",
                "bloodrager#primal-choices".into(),
                Vec::new(),
            ),
            SelectionOption {
                repeat: Repeat::Unlimited,
                replaces_row_of: Some(FeatureSelectionKey::new("bloodrager/bloodline")),
                ..option(
                    "bloodrager/primalist/primal-choices/rage-powers",
                    "Two Rage Powers",
                    "bloodrager#primal-choices".into(),
                    vec![OptionLevelRow {
                        selections: vec![rage_power.clone(), rage_power],
                        ..empty_row(1)
                    }],
                )
            },
        ],
        ..selection(
            PRIMAL_CHOICES,
            "Primal Choices",
            "bloodrager#primal-choices".into(),
            &["bloodrager"],
        )
    }
}

const PRIMAL_CHOICES: &str = "bloodrager/primalist/primal-choices";

// ---- taken where the class is built ---------------------------------------------------------------------------

/// A family and how its class's tables take it, given to the class stage through [`Picks`].
pub(crate) struct Taking {
    pub family: Family,
    pub takes: &'static Takes,
}

/// A cell of a family's class's table as the class stage reads it: the feature it names (key and name), or the feat
/// it grants.
pub(crate) struct Cell {
    pub level: u8,
    pub text: String,
    pub feature: Option<(FeatureKey, String)>,
    pub feat: Option<FeatKey>,
}

/// What a cell of a family's class's table is, in place of the feature or feat it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Planned {
    /// A placeholder ("Bloodline Ability": the held option's row) or the choice ("Order": the selection). Nothing on
    /// the row. `kept`: its feature is still defined, off the rows, since a line elsewhere resolves to it.
    Marker { kept: bool },
    /// What a fixed option's row gives, listed in the archetype's own table: the option's row stands there.
    Covered,
}

/// How one table of a family's class takes the family, planned from its cells before its rows are built.
#[derive(Default)]
pub(crate) struct TablePlan {
    /// The archetype's way of taking it; `None` for the class's own table.
    take: Option<Take>,
    cells: Vec<(u8, String, Planned)>,
    /// The placeholder and choice cells' features by level: what the rows would hold if they were features, for
    /// telling which of an archetype's rows are its class's.
    markers: BTreeMap<u8, Vec<(FeatureKey, String)>>,
    /// The fixed option's levels the archetype's table doesn't give (`Take::Fixed`).
    fixed_skipped: BTreeSet<u8>,
}

impl TablePlan {
    /// What the cell `cell` at `level` is, if the plan says.
    pub(crate) fn planned(&self, level: u8, cell: &str) -> Option<Planned> {
        self.cells
            .iter()
            .find(|(l, c, _)| *l == level && c == cell)
            .map(|(_, _, planned)| *planned)
    }

    fn markers_at(&self, level: u8) -> &[(FeatureKey, String)] {
        self.markers.get(&level).map_or(&[], Vec::as_slice)
    }
}

impl Taking {
    fn is_placeholder(&self, name: &str) -> bool {
        let name = bare(name);
        self.takes.placeholders.iter().any(|p| bare(p) == name)
    }

    fn is_choice(&self, name: &str) -> bool {
        let name = bare(name);
        self.takes.choices.iter().any(|c| bare(c) == name)
    }

    fn option_levels(&self) -> BTreeSet<u8> {
        self.family
            .selection
            .options
            .iter()
            .flat_map(|o| o.levels.iter().map(|row| row.level))
            .collect()
    }

    /// How a table of the family's class takes the family (the class's own when `archetype` is `None`), from its
    /// cells: its placeholder and choice cells are markers, and a fixed option's features its own table lists where
    /// the option's row stands are that row. An archetype the guide doesn't list takes the family when its table
    /// names a placeholder or the choice; one that takes none keeps its cells as they are.
    pub(crate) fn plan(&self, archetype: Option<&str>, cells: &[Cell]) -> TablePlan {
        let takes = self.takes;
        let marker = |name: &str| self.is_placeholder(name) || self.is_choice(name);
        let mut plan = TablePlan::default();
        let take = match archetype {
            None if !takes.base_takes => return plan,
            None => None,
            Some(archetype) => {
                let listed = takes
                    .archetypes
                    .iter()
                    .find(|(key, _)| *key == archetype)
                    .map(|(_, take)| *take);
                let names_any = cells
                    .iter()
                    .filter(|cell| cell.level <= 20)
                    .any(|cell| cell.feature.as_ref().is_some_and(|(_, name)| marker(name)));
                let take = match listed {
                    Some(take) => take,
                    None if names_any && takes.base_takes => Take::Yes,
                    None => Take::No,
                };
                plan.take = Some(take);
                if let Take::No = take {
                    return plan;
                }
                Some(take)
            }
        };
        let second = match take {
            Some(Take::SecondListOnly(choice)) => Some(choice),
            _ => None,
        };
        // A fixed option's row stands where the archetype's table lists every feature it gives.
        let mut fixed_names: BTreeMap<u8, Vec<String>> = BTreeMap::new();
        let mut fixed_feats: BTreeMap<u8, BTreeSet<FeatKey>> = BTreeMap::new();
        if let Some(Take::Fixed(option)) = take {
            let option = self.family.option(option);
            for level in self.option_levels() {
                let row_level = self.family.ties.get(&level).copied().unwrap_or(level);
                let names: Vec<String> = cells
                    .iter()
                    .filter(|cell| cell.level == row_level)
                    .filter_map(|cell| cell.feature.as_ref())
                    .map(|(_, name)| bare(name))
                    .collect();
                let covered = option
                    .levels
                    .iter()
                    .find(|row| row.level == row_level)
                    .is_none_or(|row| {
                        row_names(&self.family, row)
                            .iter()
                            .all(|name| names.contains(name))
                    });
                if !covered {
                    plan.fixed_skipped.insert(level);
                }
            }
            for row in &option.levels {
                if !plan.fixed_skipped.contains(&row.level) {
                    fixed_names.insert(row.level, row_names(&self.family, row));
                    fixed_feats.insert(row.level, row.feats.clone());
                }
            }
        }
        for cell in cells {
            let planned = match &cell.feature {
                Some((key, name))
                    if marker(name) || second.is_some_and(|choice| name == choice) =>
                {
                    plan.markers
                        .entry(cell.level)
                        .or_default()
                        .push((key.clone(), name.clone()));
                    Some(Planned::Marker {
                        kept: takes.kept_choices && self.is_choice(name),
                    })
                }
                Some((_, name))
                    if fixed_names
                        .get(&cell.level)
                        .is_some_and(|names| names.contains(&bare(name))) =>
                {
                    Some(Planned::Covered)
                }
                Some(_) => None,
                None => cell
                    .feat
                    .as_ref()
                    .filter(|feat| {
                        fixed_feats
                            .get(&cell.level)
                            .is_some_and(|feats| feats.contains(feat))
                    })
                    .map(|_| Planned::Covered),
            };
            if let Some(planned) = planned {
                plan.cells.push((cell.level, cell.text.clone(), planned));
            }
        }
        plan
    }

    /// The family taken once every table of its class is read, on the full rows: the class's row grants the
    /// selection; each archetype that takes it grants the class's selection or, when it skips option levels or limits
    /// the options, its own sharing them (an option level whose row its table has no placeholder for is skipped, and
    /// its own row stays); the bonus slots that took the merged lists pick from the held option's list. An archetype
    /// that takes none keeps its rows, but those that are its class's follow the class's.
    pub(crate) fn take(
        &self,
        class: &mut Class,
        plans: &BTreeMap<String, TablePlan>,
        lists: &mut crate::feat_lists::PendingLists,
    ) {
        let family = &self.family;
        let takes = self.takes;
        let selection_key = family.selection.key.clone();
        let none = TablePlan::default();
        let plan_of = |table: &str| plans.get(table).unwrap_or(&none);
        let base_plan = plan_of(class.key.as_str());
        let option_levels = self.option_levels();

        // The merged lists: the pending lists the merged tables' bonus slots name.
        let merged_in = |table: &str| match takes.merged {
            Merged::None => false,
            Merged::Every => true,
            Merged::Tables(tables) => tables.contains(&table),
        };
        let one_of = |slots: &[FeatSlot]| -> Vec<FeatListKey> {
            slots
                .iter()
                .filter_map(|slot| match slot {
                    FeatSlot::OneOf { list } => Some(list.clone()),
                    _ => None,
                })
                .collect()
        };
        let mut merged: BTreeSet<FeatListKey> = BTreeSet::new();
        if merged_in(class.key.as_str()) {
            for row in &class.levels {
                merged.extend(one_of(&row.feat_slots));
            }
        }
        for archetype in &class.archetypes {
            if merged_in(archetype.key.as_str()) {
                for row in &archetype.levels {
                    merged.extend(one_of(&row.feat_slots));
                }
            }
        }
        let option_list = || FeatSlot::OptionList {
            selection: selection_key.clone(),
            at_level: None,
            label: None,
        };

        // An archetype row is its class's when it holds the same, its markers counted as features, as making the
        // rows sparse tells.
        let base_before = class.levels.clone();
        let same_as_base = |row: &ArchetypeLevelRow, plan: &TablePlan| -> bool {
            let Some(base) = base_before.iter().find(|base| base.level == row.level) else {
                return false;
            };
            let keys = |features: &[FeatureKey], markers: &[(FeatureKey, String)]| {
                let mut keys: Vec<FeatureKey> = features.to_vec();
                keys.extend(markers.iter().map(|(key, _)| key.clone()));
                keys.sort();
                keys
            };
            fn sorted<T: Ord + Clone>(items: &[T]) -> Vec<T> {
                let mut items = items.to_vec();
                items.sort();
                items
            }
            row.spell_levels
                .is_none_or(|spells| spells == base.spell_levels())
                && keys(&row.features, plan.markers_at(row.level))
                    == keys(&base.features, base_plan.markers_at(row.level))
                && row.feats == base.feats
                && sorted(&row.feat_slots) == sorted(&base.feat_slots)
                && sorted(&row.selections) == sorted(&base.selections)
        };
        let base_placeholder = takes.placeholders.iter().any(|p| {
            base_plan
                .markers
                .values()
                .flatten()
                .any(|(_, name)| name == p)
        });

        // The base class: its row grants the selection, its bonus slots take the option's list.
        if takes.base_takes {
            for row in &mut class.levels {
                if row.level == takes.level && !row.selections.contains(&selection_key) {
                    row.selections.push(selection_key.clone());
                }
                for slot in &mut row.feat_slots {
                    if let FeatSlot::OneOf { list } = slot
                        && merged.contains(list)
                    {
                        lists.stand_in(&class.key, row.level, list);
                        *slot = option_list();
                    }
                }
            }
        }
        let base_after = class.levels.clone();

        for archetype in &mut class.archetypes {
            let plan = plan_of(archetype.key.as_str());
            let take = plan.take.unwrap_or(Take::No);
            if let Take::No = take {
                // Its rows that are the class's stay the class's: without the grant at the grant level.
                if takes.base_takes {
                    for row in &mut archetype.levels {
                        if !same_as_base(row, plan) {
                            continue;
                        }
                        let base = base_after
                            .iter()
                            .find(|base| base.level == row.level)
                            .expect("the class's row");
                        row.features = base.features.clone();
                        row.feats = base.feats.clone();
                        row.feat_slots = base.feat_slots.clone();
                        row.selections = base.selections.clone();
                        if row.level == takes.level {
                            row.selections.retain(|s| *s != selection_key);
                        }
                    }
                }
                continue;
            }
            let skipped: BTreeSet<u8> = match take {
                Take::Fixed(_) => plan.fixed_skipped.clone(),
                _ => option_levels
                    .iter()
                    .copied()
                    .filter(|level| {
                        let row_level = family.ties.get(level).copied().unwrap_or(*level);
                        let has_own = archetype
                            .levels
                            .iter()
                            .find(|row| row.level == row_level)
                            .is_some_and(|row| !same_as_base(row, plan));
                        let covered = !takes.skips_without_placeholder
                            // The base sorcerer has no table: its rows cover every level.
                            || (!has_own && !base_placeholder)
                            // The choice's own level gets the option's row too (an order's level-1 skills).
                            || plan
                                .markers_at(row_level)
                                .iter()
                                .any(|(_, name)| self.is_placeholder(name) || self.is_choice(name));
                        !covered
                    })
                    .collect(),
            };
            // Restrict to options or a second pick: an own selection sharing the base's options.
            let own_selection =
                |suffix: &str| FeatureSelectionKey::new(format!("{}/{suffix}", archetype.key));
            let shared: Option<BTreeSet<SelectionOptionKey>> = match take {
                Take::Only(names) => Some(
                    family
                        .option_keys()
                        .into_iter()
                        .filter(|key| {
                            let last = key.as_str().rsplit('/').next().unwrap_or_default();
                            names
                                .iter()
                                .any(|name| last == *name || last.starts_with(&format!("{name}-")))
                        })
                        .collect(),
                ),
                Take::Fixed(option) => Some(BTreeSet::from([SelectionOptionKey::new(option)])),
                _ => None,
            };
            let grant = if shared.is_some() || !skipped.is_empty() {
                let key = own_selection(
                    selection_key
                        .as_str()
                        .rsplit('/')
                        .next()
                        .unwrap_or_default(),
                );
                archetype.selections.push(FeatureSelection {
                    key: key.clone(),
                    name: format!("{} ({})", family.selection.name, archetype.name),
                    summary: SUMMARIES
                        .iter()
                        .find(|(of, _)| *of == archetype.key.as_str())
                        .map(|(_, summary)| (*summary).to_owned())
                        .unwrap_or_default(),
                    guide: archetype.guide.clone(),
                    options: Vec::new(),
                    shared_options: shared
                        .unwrap_or_else(|| family.option_keys().into_iter().collect()),
                    skipped_levels: skipped,
                    ..family.selection.clone()
                });
                key
            } else {
                selection_key.clone()
            };
            // The second pick's list-only selection.
            let second = match take {
                Take::SecondListOnly(_) => {
                    let key = own_selection("second-bloodline");
                    archetype.selections.push(FeatureSelection {
                        key: key.clone(),
                        name: format!("Second {} ({})", family.selection.name, archetype.name),
                        guide: archetype.guide.clone(),
                        options: Vec::new(),
                        shared_options: family.option_keys().into_iter().collect(),
                        feat_lists_only: true,
                        ..family.selection.clone()
                    });
                    Some(key)
                }
                _ => None,
            };
            for row in &mut archetype.levels {
                if row.level == takes.level {
                    row.selections.retain(|s| *s != selection_key);
                    row.selections.push(grant.clone());
                    row.selections.extend(second.clone());
                }
                for slot in &mut row.feat_slots {
                    if let FeatSlot::OneOf { list } = slot
                        && merged.contains(list)
                    {
                        *slot = option_list();
                    }
                }
            }
        }
        for (archetype, cell, choice, expected) in takes.row_choices {
            let plan = plan_of(archetype);
            let levels: Vec<u8> = plan
                .markers
                .iter()
                .filter(|(_, markers)| markers.iter().any(|(_, name)| name == cell))
                .map(|(level, _)| *level)
                .collect();
            assert_eq!(levels, *expected, "{archetype}: {cell}");
            let archetype = class
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == *archetype)
                .unwrap_or_else(|| panic!("no archetype {archetype}"));
            for row in &mut archetype.levels {
                if levels.contains(&row.level) {
                    row.selections.push(FeatureSelectionKey::new(*choice));
                }
            }
        }
    }
}

// ---- Dragon Disciple, Eldritch Scion, Dragonheir Scion; Winter Witch; mythic --------------------------------------

/// The classes that pick a sorcerer bloodline or a wizard's element, given where they're built.
///
/// - Dragon Disciple: "Must have a Draconic bloodline or no known bloodline", "Able to cast 1st level arcane spells
///   spontaneously"; "If the precursor class is other than Sorcerer, you must select a Draconic bloodline. Otherwise,
///   the Dragon Disciple continues your bloodline progression uninterrupted. If you selected a bloodline when you
///   became a Dragon Disciple, your bloodline level is your class level." Its 2/5/8 "Bloodline Feat" is the
///   bloodline's; Strength, Constitution and Intelligence +2 are ability bonuses.
/// - Eldritch Scion: "Spells are cast spontaneously", "Must select a bloodline which provides additional spells and
///   abilities", "but not the feats that Sorcerers would receive"; Dragon Disciple levels "do not hinder bloodline
///   progression" (magus).
/// - Dragonheir Scion: "Gains a Draconic Bloodline" with its own rows (Draconic Defences, Strike, Presence, Wings,
///   Might of Wyrms), so it skips every Draconic row.
/// - Elemental Specialist: "At 1st level, an Elemental Specialist selects one element (from Acid, Cold, Electricity
///   and Fire)": the choice is a selection.
/// - Disruptive: "Fighter Level 6 or Arcane bloodline".
fn provide_others(sorcerer: &Family, picks: &mut Picks) {
    let draconic: BTreeSet<SelectionOptionKey> = sorcerer
        .option_keys()
        .into_iter()
        .filter(|key| key.as_str().starts_with("sorcerer/bloodline/draconic-"))
        .collect();
    let every: BTreeSet<SelectionOptionKey> = sorcerer.option_keys().into_iter().collect();
    let sorcerer_bloodline = FeatureSelectionKey::new("sorcerer/bloodline");

    // Dragon Disciple.
    let disciple = "dragon-disciple";
    let blood = "dragon-disciple/bloodline";
    picks.selection(
        disciple,
        FeatureSelection {
            name: "Blood of Dragons".into(),
            guide: GuideRef::new("prestige-classes#blood-of-dragons"),
            shared_options: draconic.clone(),
            only_without_option: true,
            ..selection(blood, "", String::new(), &[disciple])
        },
    );
    picks.row_selection(disciple, 1, blood);
    picks.rule(Rule {
        feature: FeatureKey::new("dragon-disciple/bloodline-feat"),
        table: Some(disciple.into()),
        cell: None,
        levels: Some(vec![2, 5, 8]),
        pick: Pick::Slots(vec![FeatSlot::OptionList {
            selection: sorcerer_bloodline.clone(),
            at_level: None,
            label: None,
        }]),
    });
    for (feature, ability) in [
        ("dragon-disciple/strength-2", rules::Ability::Strength),
        (
            "dragon-disciple/constitution-2",
            rules::Ability::Constitution,
        ),
        (
            "dragon-disciple/intelligence-2",
            rules::Ability::Intelligence,
        ),
    ] {
        picks.feature_fact(
            feature,
            FeatureFact::AbilityBonus(AbilityBonus::Fixed { ability, amount: 2 }),
        );
    }
    let mut either: Vec<Prerequisite> = draconic
        .iter()
        .cloned()
        .map(Prerequisite::SelectionOption)
        .collect();
    either.push(Prerequisite::All(vec![
        Prerequisite::NoOptionFrom(sorcerer_bloodline.clone()),
        Prerequisite::NoOptionFrom(FeatureSelectionKey::new("bloodrager/bloodline")),
    ]));
    let at = DataAt::Class(ClassKey::new(disciple));
    picks.stated(
        at.clone(),
        Prerequisite::SpontaneousSpellLevel {
            spell_type: SpellType::Arcane,
            level: 1,
        },
    );
    picks.stated(at, Prerequisite::Any(either));

    // Eldritch Scion.
    let scion = "magus/eldritch-scion";
    let scion_bloodline = "magus/eldritch-scion/bloodline";
    picks.selection(
        scion,
        FeatureSelection {
            name: "Bloodline (Eldritch Scion)".into(),
            guide: GuideRef::new("magus#eldritch-scion"),
            shared_options: every,
            ..selection(scion_bloodline, "", String::new(), &["magus", disciple])
        },
    );
    picks.row_selection(scion, 1, scion_bloodline);

    // Dragonheir Scion.
    let heir = "fighter/dragonheir-scion";
    let heir_bloodline = FeatureSelectionKey::new("fighter/dragonheir-scion/bloodline");
    let rows: BTreeSet<u8> = draconic
        .iter()
        .flat_map(|key| {
            sorcerer
                .option(key.as_str())
                .levels
                .iter()
                .map(|row| row.level)
        })
        .collect();
    picks.selection(
        heir,
        FeatureSelection {
            name: "Draconic Bloodline".into(),
            guide: GuideRef::new("fighter#draconic-bloodline"),
            shared_options: draconic,
            skipped_levels: rows,
            ..selection(heir_bloodline.as_str(), "", String::new(), &["fighter"])
        },
    );
    picks.rule(Rule {
        feature: FeatureKey::new("fighter/dragonheir-scion/draconic-bloodline"),
        table: Some(heir.into()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(heir_bloodline),
    });

    // Elemental Specialist.
    let focused = "wizard/elemental-specialist/focused-element";
    let mut choice = selection(
        focused,
        "Focused Element",
        "wizard#focused-element".into(),
        &[],
    );
    for element in ["Acid", "Cold", "Electricity", "Fire"] {
        choice.options.push(option(
            &format!("{focused}/{}", slug(element)),
            element,
            "wizard#focused-element".into(),
            Vec::new(),
        ));
    }
    picks.selection("wizard/elemental-specialist", choice);
    picks.unnamed(focused);
    picks.rule(Rule {
        feature: FeatureKey::new(focused),
        table: Some("wizard/elemental-specialist".into()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(FeatureSelectionKey::new(focused)),
    });

    // Disruptive: the "or" line, whose "Arcane bloodline" names no heading.
    picks.prerequisite(
        DataAt::Feat(rules::FeatKey::new("disruptive")),
        "Fighter Level 6 or Arcane bloodline",
        Prerequisite::Any(vec![
            Prerequisite::ClassLevel {
                class: ClassKey::new("fighter"),
                level: 6,
            },
            Prerequisite::SelectionOption(SelectionOptionKey::new("sorcerer/bloodline/arcane")),
            Prerequisite::SelectionOption(SelectionOptionKey::new("bloodrager/bloodline/arcane")),
        ]),
    );
}

/// What the providers of this module can only assert on the finished data: Dragon Disciple's, the Eldritch Scion's,
/// the Dragonheir Scion's and the Elemental Specialist's picks are on their rows in force, Dragon Disciple's ability
/// bonuses on the features of those names, and the Winter Witch has at most one shaman stand-in (a hand fix names
/// the Frost spirit there; a fresh extraction leaves the whole "Required classes" line out). Changes nothing.
pub(crate) fn check(data: &GameData) {
    let class = |key: &str| {
        data.classes
            .iter()
            .chain(&data.prestige_classes)
            .find(|class| class.key.as_str() == key)
            .unwrap_or_else(|| panic!("no class {key}"))
    };
    let row = |class: &str, archetype: &str, level: u8| {
        class_row(class_of(data, class), archetype, level)
            .unwrap_or_else(|| panic!("{archetype} has no row {level}"))
    };
    let disciple = class("dragon-disciple");
    let first = disciple
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("level 1");
    assert!(
        first
            .selections
            .iter()
            .any(|s| s.as_str() == "dragon-disciple/bloodline")
    );
    for level in [2, 5, 8] {
        let row = disciple
            .levels
            .iter()
            .find(|row| row.level == level)
            .expect("a row");
        assert!(
            row.feat_slots.iter().any(|slot| matches!(slot, FeatSlot::OptionList { selection, .. } if selection.as_str() == "sorcerer/bloodline")),
            "dragon-disciple {level}: the bloodline's feat"
        );
    }
    for feature in &disciple.features {
        let bonus =
            ["Strength +2", "Constitution +2", "Intelligence +2"].contains(&feature.name.as_str());
        assert_eq!(
            bonus,
            !feature.ability_bonuses.is_empty(),
            "{}",
            feature.key
        );
    }
    let (selections, _) = row("magus", "magus/eldritch-scion", 1);
    assert!(
        selections
            .iter()
            .any(|s| s.as_str() == "magus/eldritch-scion/bloodline")
    );
    let (selections, features) = row("fighter", "fighter/dragonheir-scion", 1);
    assert!(
        selections
            .iter()
            .any(|s| s.as_str() == "fighter/dragonheir-scion/bloodline")
    );
    assert!(
        !features
            .iter()
            .any(|f| f.as_str() == "fighter/dragonheir-scion/draconic-bloodline")
    );
    let (selections, features) = row("wizard", "wizard/elemental-specialist", 1);
    assert!(
        selections
            .iter()
            .any(|s| s.as_str() == "wizard/elemental-specialist/focused-element")
    );
    assert!(
        !features
            .iter()
            .any(|f| f.as_str() == "wizard/elemental-specialist/focused-element")
    );

    let stand_in = Prerequisite::ClassLevel {
        class: ClassKey::new("shaman"),
        level: 1,
    };
    let witch = class("winter-witch");
    let mut found = 0;
    if let Some(prerequisite) = &witch.prerequisite {
        count_leaf(prerequisite, &stand_in, &mut found);
    }
    assert!(found <= 1, "the Winter Witch's shaman stand-in");
}

fn class_of<'d>(data: &'d GameData, key: &str) -> &'d Class {
    data.classes
        .iter()
        .find(|class| class.key.as_str() == key)
        .unwrap_or_else(|| panic!("no class {key}"))
}

/// An archetype's row in force at `level`: its own, or its class's.
fn class_row<'c>(
    class: &'c Class,
    archetype: &str,
    level: u8,
) -> Option<(&'c [FeatureSelectionKey], &'c [FeatureKey])> {
    let archetype = class
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)?;
    match archetype.levels.iter().find(|row| row.level == level) {
        Some(row) => Some((&row.selections, &row.features)),
        None => class
            .levels
            .iter()
            .find(|row| row.level == level)
            .map(|row| (&row.selections[..], &row.features[..])),
    }
}

fn count_leaf(prerequisite: &Prerequisite, leaf: &Prerequisite, found: &mut usize) {
    if prerequisite == leaf {
        *found += 1;
    }
    if let Prerequisite::All(children)
    | Prerequisite::Any(children)
    | Prerequisite::AtLeast(_, children) = prerequisite
    {
        for child in children {
            count_leaf(child, leaf, found);
        }
    }
}

/// The facts of the families' mythic abilities and daily choices, built where their owners are: Second Bloodline
/// ("You can select a second bloodline, gaining all its benefits"), Second Bloodrager Bloodline and Second Spirit
/// ("You gain a second Shaman Spirit") pick another option; Bloodline Ascendance ("You gain the 20th level ability of your bloodline") grants the held bloodline's row
/// 20. Mixed Bloodline ("Once per day, a Mixed-Blood Rager chooses a bloodline") and the shaman's Wandering Spirit
/// ("can be changed each time you rest") are daily choices.
pub(crate) fn provide(
    guide: &Guide,
    feats: &FeatNames,
    skills: &[rules::Skill],
    picks: &mut Picks,
) {
    provide_families(guide, feats, skills, picks);
    let grants = [
        ("second-bloodline", "sorcerer/bloodline"),
        ("second-bloodrager-bloodline", "bloodrager/bloodline"),
        ("second-spirit", "shaman/spirit"),
    ];
    for (ability, selection) in grants {
        picks.mythic_selection(ability, FeatureSelectionKey::new(selection));
    }
    let ascendance = "sorcerer/bloodline-ascendance";
    picks.feature(
        "sorcerer",
        Feature {
            option_row: Some(OptionRow {
                selection: FeatureSelectionKey::new("sorcerer/bloodline"),
                level: 20,
            }),
            ..feature(
                &FeatureKey::new(ascendance),
                "Bloodline Ascendance",
                "mythic-paths#bloodline-ascendance".into(),
            )
        },
    );
    picks.mythic_feature("bloodline-ascendance", ascendance);
    for daily in [
        "bloodrager/mixed-blood-rager/mixed-bloodline",
        "bloodrager/mixed-blood-rager/second-mixed-bloodline",
        "shaman/wandering-spirit",
    ] {
        picks.daily_choice(daily);
    }
}

/// The report items this module settles: the option tables it reads, and the lines it can now state.
pub fn settled(at: &str, text: &str) -> bool {
    let tables = SORCERER_TABLES
        .iter()
        .map(|t| format!("sorcerer#{t}"))
        .chain(
            BLOODRAGER_TABLES
                .iter()
                .map(|t| format!("bloodrager#{t}-bloodrager")),
        )
        .chain(SCHOOLS.iter().map(|t| format!("wizard#{t}")))
        .chain(ORDERS.iter().map(|t| format!("cavalier#{t}")))
        .chain(WAYS.iter().map(|t| format!("arcanist#{t}")));
    tables.into_iter().any(|table| table == at)
        || at == "feats#disruptive"
        || at == "character-creation#bloodlines"
        || (at == "prestige-classes#dragon-disciple"
            && (text.contains("Draconic bloodline") || text.contains("spontaneously")))
}

/// A name without its parenthesised tail, for comparing: "Manifestation (Battle)" is "Manifestation".
pub(crate) fn bare(name: &str) -> String {
    norm(name.split(" (").next().unwrap_or(name))
}
