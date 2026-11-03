//! Options with rows by option level (ticket 30): sorcerer and bloodrager bloodlines, wizard schools, cavalier
//! orders, shaman spirits, the Magic Deceiver's ways and the ranger's combat styles, plus Dragon Disciple, Eldritch
//! Scion and Dragonheir Scion.
//!
//! Each option's rows come from the guide: from its progression table where it has one (the cells every option's
//! table shares at a level are the class's, the rest are the option's; bloodline spells stay out), or from prose
//! where it doesn't (spirits, combat styles). Then each class and archetype takes the family:
//! - the class's (and each archetype's) placeholder cells go: the choice ("Bloodline", "Order", "Spirit", "Ways of
//!   the Deceiver") becomes the selection, and the ability placeholders ("Bloodline Ability", "Order Ability") are
//!   the option's rows now;
//! - an option level whose row an archetype's table has no placeholder for (or, for an archetype whose option is
//!   fixed, whose features it doesn't list) is skipped, and the archetype's own row stays;
//! - an archetype that skips levels, limits the options or takes only the feat list gets its own selection that
//!   shares the options it allows; one option is granted with no pick;
//! - the bonus slots that took the class's merged bloodline list pick from the held option's list instead.
//!
//! [`apply`] runs on full rows (game data as extracted or as hand-fixed) and asserts what it finds.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    AbilityBonus, ArchetypeLevelRow, Class, ClassKey, ClassLevelRow, FeatKey, FeatList,
    FeatListKey, FeatSlot, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData,
    GuideRef, OptionLevelRow, OptionRow, Prerequisite, Repeat, SelectionOption, SelectionOptionKey,
    SpellType,
};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::{norm, slug};
use crate::tables::parse_table;

/// What [`apply`] changed, one line each, for the rewrite's log.
pub type Log = Vec<String>;

/// Reads every family from the guide and gives it to the classes that take it.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let feats = FeatNames::new(data);
    spontaneous(guide, data, &mut log);

    let sorcerer = sorcerer_bloodlines(guide, &feats);
    take_family(data, &sorcerer, &SORCERER, &mut log);
    let bloodrager = bloodrager_bloodlines(guide, &feats);
    primalist(guide, data, &mut log);
    take_family(data, &bloodrager, &BLOODRAGER, &mut log);
    let wizard = wizard_schools(guide, &feats);
    take_family(data, &wizard, &WIZARD, &mut log);
    let cavalier = cavalier_orders(guide, &feats);
    take_family(data, &cavalier, &CAVALIER, &mut log);
    let ways = magic_deceiver_ways(guide, &feats);
    take_family(data, &ways, &MAGIC_DECEIVER, &mut log);
    let spirits = shaman_spirits(guide);
    take_family(data, &spirits, &SHAMAN, &mut log);
    let styles = ranger_styles(guide, &feats);
    take_family(data, &styles, &RANGER, &mut log);

    other_classes(data, &sorcerer, &mut log);
    mythic(data, &mut log);
    prerequisites(data, &mut log);
    drop_unused(data, &TOUCHED, &mut log);
    log
}

/// "Spells are cast spontaneously" in a class's or an archetype's characteristics (the others' say "Spells must be
/// prepared in advance"): Dragon Disciple's entry checks it.
fn spontaneous(guide: &Guide, data: &mut GameData, log: &mut Log) {
    const SPONTANEOUS: &str = "Spells are cast spontaneously";
    for class in &mut data.classes {
        let page = guide.page(class.key.as_str());
        for section in page.root.walk() {
            if !section.text().contains(SPONTANEOUS) {
                continue;
            }
            match section.level {
                2 => {
                    class.spontaneous = true;
                    log.push(format!("{}: spontaneous", class.key));
                }
                3 => {
                    let key = format!("{}/{}", class.key, section.anchor);
                    let archetype = class
                        .archetypes
                        .iter_mut()
                        .find(|a| a.key.as_str() == key)
                        .unwrap_or_else(|| panic!("no archetype {key}"));
                    archetype.spontaneous = Some(true);
                    log.push(format!("{key}: spontaneous"));
                }
                _ => panic!(
                    "{}#{}: {SPONTANEOUS} below an archetype",
                    page.slug, section.anchor
                ),
            }
        }
    }
}

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
struct Takes {
    /// The class's level that grants the selection.
    level: u8,
    /// The table cell that is the choice itself ("Bloodline"); it becomes the selection.
    choices: &'static [&'static str],
    /// The table cells the option's rows replace ("Bloodline Ability").
    placeholders: &'static [&'static str],
    /// Whether an archetype that takes the family skips the option levels its table has no placeholder for.
    skips_without_placeholder: bool,
    /// The lists the class's bonus slots took before; their slots now pick from the held option's list.
    merged_lists: &'static [&'static str],
    /// The base class takes the family (the Magic Deceiver's ways are the archetype's alone).
    base_takes: bool,
    archetypes: &'static [(&'static str, Take)],
}

/// How one archetype takes a family, where its table doesn't say by itself.
#[derive(Clone, Copy, Debug)]
enum Take {
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
    level: 1,
    choices: &[],
    placeholders: &["Bloodline Ability", "Bloodline Arcana"],
    skips_without_placeholder: true,
    merged_lists: &[
        "sorcerer/bonus-feats",
        "sorcerer/empyreal-sorcerer/bonus-feats",
        "sorcerer/sage-sorcerer/bonus-feats",
        "sorcerer/sylvan-sorcerer/bonus-feats",
    ],
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
};

const BLOODRAGER: Takes = Takes {
    level: 1,
    choices: &["Bloodline"],
    // A Primalist's "Bloodline Ability or Rage Powers" is the bloodline row unless its choice replaces it.
    placeholders: &["Bloodline Ability", "Bloodline Ability or Rage Powers"],
    skips_without_placeholder: true,
    merged_lists: &[
        "bloodrager/bonus-feats",
        "bloodrager/hag-riven/bonus-feats",
        "bloodrager/reformed-fiend/bonus-feats",
    ],
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
};

const WIZARD: Takes = Takes {
    level: 1,
    choices: &[],
    placeholders: &["Specialisation abilities", "Specialisation ability"],
    skips_without_placeholder: false,
    merged_lists: &[],
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
};

/// Notes for an archetype's own selection: the Thassilonian's spellbooks, by the school each specialises in.
const SUMMARIES: [(&str, &str); 1] = [(
    "wizard/thassilonian-specialist",
    "Spellbooks: Envy (Abjurer), Gluttony (Necromancer), Greed (Transmuter), Lust (Enchanter), Pride \
     (Illusionist), Sloth (Conjurer), Wrath (Invoker).",
)];

const CAVALIER: Takes = Takes {
    level: 1,
    choices: &["Order"],
    placeholders: &["Order Ability"],
    skips_without_placeholder: true,
    merged_lists: &[],
    base_takes: true,
    archetypes: &[],
};

const MAGIC_DECEIVER: Takes = Takes {
    level: 1,
    choices: &["Ways of the Deceiver"],
    placeholders: &[],
    skips_without_placeholder: false,
    merged_lists: &[],
    base_takes: false,
    archetypes: &[("arcanist/magic-deceiver", Take::Yes)],
};

const SHAMAN: Takes = Takes {
    level: 1,
    choices: &["Spirit"],
    placeholders: &[
        "Spirit Ability",
        "Greater Spirit Ability",
        "True Spirit Ability",
        "Manifestation",
    ],
    skips_without_placeholder: true,
    merged_lists: &[],
    base_takes: true,
    // "Guiding spirit is the Battle spirit" (Wildland Shaman).
    archetypes: &[(
        "shaman/wildland-shaman",
        Take::Fixed("shaman/spirit/battle"),
    )],
};

const RANGER: Takes = Takes {
    level: 2,
    choices: &[],
    placeholders: &[],
    skips_without_placeholder: false,
    merged_lists: &[
        "ranger/bonus-feats-2",
        "ranger/bonus-feats-6",
        "ranger/bonus-feats-10",
    ],
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
};

/// A class's or an archetype's level row, for its feat slots.
trait Row {
    fn feat_slots(&mut self) -> &mut Vec<FeatSlot>;
}

impl Row for ClassLevelRow {
    fn feat_slots(&mut self) -> &mut Vec<FeatSlot> {
        &mut self.feat_slots
    }
}

impl Row for ArchetypeLevelRow {
    fn feat_slots(&mut self) -> &mut Vec<FeatSlot> {
        &mut self.feat_slots
    }
}

/// Every feature name the class and its archetypes define, by key.
fn feature_names(class: &Class) -> BTreeMap<FeatureKey, String> {
    class
        .features
        .iter()
        .chain(class.archetypes.iter().flat_map(|a| &a.features))
        .map(|feature| (feature.key.clone(), feature.name.clone()))
        .collect()
}

/// Turns the bonus slots that took a merged list into slots picking from the held option's list.
fn option_list_slots(row: &mut dyn Row, merged: &[&str], selection: &FeatureSelectionKey) -> bool {
    let mut changed = false;
    for slot in row.feat_slots().iter_mut() {
        if let FeatSlot::OneOf { list } = slot
            && merged.contains(&list.as_str())
        {
            *slot = FeatSlot::OptionList {
                selection: selection.clone(),
                at_level: None,
                label: None,
            };
            changed = true;
        }
    }
    changed
}

fn take_family(data: &mut GameData, family: &Family, takes: &Takes, log: &mut Log) {
    let (class_key, owner_archetype) = match family.owner {
        Owner::Class(class) => (class, None),
        Owner::Archetype(class, archetype) => (class, Some(archetype)),
    };
    let class = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == class_key)
        .unwrap_or_else(|| panic!("no class {class_key}"));
    let names = feature_names(class);
    let name_of = |key: &FeatureKey| -> String {
        names
            .get(key)
            .cloned()
            .or_else(|| family.feature_name(key).map(str::to_owned))
            .unwrap_or_default()
    };
    let selection_key = family.selection.key.clone();

    // The family's definitions: on the class, or on the archetype that owns it. An existing selection of that key
    // (the shaman's spirits, the ranger's styles) keeps its options' prerequisites and gets the rows.
    let (features, selections, lists) = match owner_archetype {
        None => (
            &mut class.features,
            &mut class.selections,
            &mut class.feat_lists,
        ),
        Some(archetype) => {
            let archetype = class
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == archetype)
                .unwrap_or_else(|| panic!("no archetype {archetype}"));
            (
                &mut archetype.features,
                &mut archetype.selections,
                &mut archetype.feat_lists,
            )
        }
    };
    for feature in &family.features {
        assert!(
            !features.iter().any(|f| f.key == feature.key),
            "{} is defined already",
            feature.key
        );
        features.push(feature.clone());
    }
    lists.extend(family.lists.iter().cloned());
    selections.extend(family.nested.iter().cloned());
    match selections.iter_mut().find(|s| s.key == selection_key) {
        Some(existing) => {
            let options: BTreeSet<&SelectionOptionKey> =
                existing.options.iter().map(|o| &o.key).collect();
            let new: BTreeSet<&SelectionOptionKey> =
                family.selection.options.iter().map(|o| &o.key).collect();
            assert_eq!(options, new, "{selection_key}: the options differ");
            for option in &mut existing.options {
                option.levels = family.option(option.key.as_str()).levels.clone();
            }
            existing.option_levels = family.selection.option_levels.clone();
            log.push(format!("{selection_key}: rows on its existing options"));
        }
        None => {
            selections.push(family.selection.clone());
            log.push(format!(
                "{selection_key}: new, {} options",
                family.selection.options.len()
            ));
        }
    }

    // "Bloodline (Hag-Riven)" is the Hag-Riven's Bloodline cell.
    let is_placeholder = |key: &FeatureKey| {
        let name = bare(&name_of(key));
        takes.placeholders.iter().any(|p| bare(p) == name)
    };
    let is_choice = |key: &FeatureKey| {
        let name = bare(&name_of(key));
        takes.choices.iter().any(|c| bare(c) == name)
    };
    let option_levels: BTreeSet<u8> = family
        .selection
        .options
        .iter()
        .flat_map(|o| o.levels.iter().map(|row| row.level))
        .collect();

    // The base class: placeholders go, the selection is granted, bonus slots take the option's list.
    let base_rows_before = class.levels.clone();
    if takes.base_takes {
        for row in &mut class.levels {
            let before = row.features.len();
            row.features.retain(|f| !is_placeholder(f) && !is_choice(f));
            if row.features.len() != before {
                log.push(format!("{class_key} {}: placeholders removed", row.level));
            }
            if row.level == takes.level && !row.selections.contains(&selection_key) {
                row.selections.push(selection_key.clone());
                log.push(format!("{class_key} {}: grants {selection_key}", row.level));
            }
            if option_list_slots(row, takes.merged_lists, &selection_key) {
                log.push(format!("{class_key} {}: bloodline slots", row.level));
            }
        }
    }

    // Archetypes.
    let base_rows = class.levels.clone();
    for archetype in &mut class.archetypes {
        let take = takes
            .archetypes
            .iter()
            .find(|(key, _)| *key == archetype.key.as_str())
            .map(|(_, take)| *take);
        // An archetype not listed takes the family when its rows (or the base rows it keeps) name a choice or a
        // placeholder, and the base class takes it.
        let effective = |level: u8| -> Vec<FeatureKey> {
            match archetype.levels.iter().find(|row| row.level == level) {
                Some(own) => own.features.clone(),
                None => base_rows_before
                    .iter()
                    .find(|row| row.level == level)
                    .map(|row| row.features.clone())
                    .unwrap_or_default(),
            }
        };
        let names_any = (1..=20).any(|level| {
            effective(level)
                .iter()
                .any(|f| is_placeholder(f) || is_choice(f))
        });
        let take = match take {
            Some(take) => take,
            None if names_any && takes.base_takes => Take::Yes,
            None => Take::No,
        };
        if let Take::No = take {
            // It keeps the base row at the grant level only without the grant.
            if takes.base_takes && !archetype.levels.iter().any(|row| row.level == takes.level) {
                let base = base_rows
                    .iter()
                    .find(|row| row.level == takes.level)
                    .expect("the grant level's row");
                archetype.levels.push(ArchetypeLevelRow {
                    level: takes.level,
                    spell_levels: None,
                    features: base.features.clone(),
                    feats: base.feats.clone(),
                    feat_slots: base.feat_slots.clone(),
                    selections: base
                        .selections
                        .iter()
                        .filter(|s| **s != selection_key)
                        .cloned()
                        .collect(),
                    feats_for: Default::default(),
                });
                archetype.levels.sort_by_key(|row| row.level);
                log.push(format!("{}: takes no {selection_key}", archetype.key));
            }
            continue;
        }
        let fixed = match take {
            Take::Fixed(option) => Some(family.option(option)),
            _ => None,
        };
        // Which option levels the archetype's own table covers.
        let mut skipped = BTreeSet::new();
        for &level in &option_levels {
            let tied = family.ties.get(&level).copied();
            let row_level = tied.unwrap_or(level);
            let has_own = archetype.levels.iter().any(|row| row.level == row_level);
            let features = effective(row_level);
            let covered = match fixed {
                Some(option) => option
                    .levels
                    .iter()
                    .find(|row| row.level == row_level)
                    .is_none_or(|row| {
                        row_names(family, row)
                            .iter()
                            .all(|name| features.iter().any(|own| bare(&name_of(own)) == *name))
                    }),
                None if !takes.skips_without_placeholder => true,
                // The base sorcerer has no table: its rows cover every level.
                None if !has_own
                    && takes.placeholders.iter().all(|p| {
                        !base_rows_before
                            .iter()
                            .any(|row| row.features.iter().any(|f| name_of(f) == *p))
                    }) =>
                {
                    true
                }
                // The choice's own level gets the option's row too (an order's level-1 skills).
                None => features.iter().any(|f| is_placeholder(f) || is_choice(f)),
            };
            if !covered {
                skipped.insert(level);
            }
        }
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
            let mut own = FeatureSelection {
                key: key.clone(),
                name: format!("{} ({})", family.selection.name, archetype.name),
                guide: archetype.guide.clone(),
                options: Vec::new(),
                shared_options: shared
                    .unwrap_or_else(|| family.option_keys().into_iter().collect()),
                skipped_levels: skipped.clone(),
                ..family.selection.clone()
            };
            own.summary = SUMMARIES
                .iter()
                .find(|(of, _)| *of == archetype.key.as_str())
                .map(|(_, summary)| (*summary).to_owned())
                .unwrap_or_default();
            archetype.selections.push(own);
            log.push(format!(
                "{}: own selection {key}, skips {skipped:?}",
                archetype.key
            ));
            key
        } else {
            selection_key.clone()
        };
        // The second pick's choice cell, and its list-only selection.
        let second = match take {
            Take::SecondListOnly(choice) => {
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
                Some((choice, key))
            }
            _ => None,
        };

        // Its rows: the choice becomes the grant; placeholders (and a fixed option's features) go where covered.
        let fixed_names: BTreeMap<u8, Vec<String>> = fixed
            .map(|option| {
                option
                    .levels
                    .iter()
                    .filter(|row| !skipped.contains(&row.level))
                    .map(|row| (row.level, row_names(family, row)))
                    .collect()
            })
            .unwrap_or_default();
        let grant_level = takes.level;
        if !archetype.levels.iter().any(|row| row.level == grant_level) && grant != selection_key {
            let base = base_rows
                .iter()
                .find(|row| row.level == grant_level)
                .expect("the grant level's row");
            archetype.levels.push(ArchetypeLevelRow {
                level: grant_level,
                spell_levels: None,
                features: base.features.clone(),
                feats: base.feats.clone(),
                feat_slots: base.feat_slots.clone(),
                selections: base.selections.clone(),
                feats_for: Default::default(),
            });
            archetype.levels.sort_by_key(|row| row.level);
        }
        for row in &mut archetype.levels {
            let level = row.level;
            row.features.retain(|f| {
                if is_choice(f) {
                    return false;
                }
                if let Some((choice, _)) = &second
                    && name_of(f) == *choice
                {
                    return false;
                }
                if is_placeholder(f) {
                    return false;
                }
                !fixed_names
                    .get(&level)
                    .is_some_and(|names| names.contains(&bare(&name_of(f))))
            });
            // A fixed option's granted feats where it covers the level (Wings at 15).
            if let Some(option) = fixed
                && fixed_names.contains_key(&level)
                && let Some(option_row) = option.levels.iter().find(|r| r.level == level)
            {
                row.feats.retain(|feat| !option_row.feats.contains(feat));
            }
            if level == grant_level {
                row.selections.retain(|s| *s != selection_key);
                row.selections.push(grant.clone());
                if let Some((_, second)) = &second {
                    row.selections.push(second.clone());
                }
            }
            option_list_slots(row, takes.merged_lists, &selection_key);
        }
        log.push(format!("{}: takes {grant}", archetype.key));
    }
    // A row the edits made the same as the base row's is the base row.
    for archetype in &mut class.archetypes {
        archetype.levels.retain(|row| {
            let Some(base) = base_rows.iter().find(|base| base.level == row.level) else {
                return true;
            };
            let sorted = |mut items: Vec<String>| {
                items.sort();
                items
            };
            let keys = |items: &[FeatureKey]| sorted(items.iter().map(|k| k.to_string()).collect());
            let slots =
                |items: &[FeatSlot]| sorted(items.iter().map(|s| format!("{s:?}")).collect());
            let picks = |items: &[FeatureSelectionKey]| {
                sorted(items.iter().map(|k| k.to_string()).collect())
            };
            let same = row
                .spell_levels
                .is_none_or(|spells| spells == base.spell_levels())
                && keys(&row.features) == keys(&base.features)
                && row.feats == base.feats
                && slots(&row.feat_slots) == slots(&base.feat_slots)
                && picks(&row.selections) == picks(&base.selections);
            if same {
                log.push(format!(
                    "{} {}: same as the base row now",
                    archetype.key, row.level
                ));
            }
            !same
        });
    }
}

/// Primalist: "At 4th level and every four levels thereafter, a Primalist can choose to take his bloodline ability
/// or two Rage Powers." A choice at each "Bloodline Ability or Rage Powers" cell: the bloodline's row, or two rage
/// powers that skip it.
fn primalist(guide: &Guide, data: &mut GameData, log: &mut Log) {
    let text = section_text(&page(guide, "bloodrager").root, "primal-choices");
    assert!(text.contains("choose to take his bloodline ability or two Rage Powers"));
    let bloodrager = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "bloodrager")
        .expect("bloodrager");
    let primalist = bloodrager
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "bloodrager/primalist")
        .expect("Primalist");
    let choices = FeatureSelectionKey::new("bloodrager/primalist/primal-choices");
    let rage_power = FeatureSelectionKey::new("barbarian/rage-power");
    primalist.selections.push(FeatureSelection {
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
            choices.as_str(),
            "Primal Choices",
            "bloodrager#primal-choices".into(),
            &["bloodrager"],
        )
    });
    let cell = FeatureKey::new("bloodrager/primalist/bloodline-ability-or-rage-powers");
    let mut levels = Vec::new();
    for row in &mut primalist.levels {
        if row.features.contains(&cell) {
            row.selections.push(choices.clone());
            levels.push(row.level);
        }
    }
    assert_eq!(levels, [4, 8, 12, 16, 20], "the Primalist's choices");
    log.push(format!("bloodrager/primalist: {choices} at {levels:?}"));
}

// ---- Dragon Disciple, Eldritch Scion, Dragonheir Scion; Winter Witch; mythic --------------------------------------

fn other_classes(data: &mut GameData, sorcerer: &Family, log: &mut Log) {
    let draconic: BTreeSet<SelectionOptionKey> = sorcerer
        .option_keys()
        .into_iter()
        .filter(|key| key.as_str().starts_with("sorcerer/bloodline/draconic-"))
        .collect();
    let every: BTreeSet<SelectionOptionKey> = sorcerer.option_keys().into_iter().collect();

    // Dragon Disciple: "Must have a Draconic bloodline or no known bloodline", "Able to cast 1st level arcane
    // spells spontaneously"; "If the precursor class is other than Sorcerer, you must select a Draconic bloodline.
    // Otherwise, the Dragon Disciple continues your bloodline progression uninterrupted. If you selected a bloodline
    // when you became a Dragon Disciple, your bloodline level is your class level." The 2/5/8 "Bloodline Feat" is
    // the bloodline's; Strength, Constitution and Intelligence +2 are ability bonuses.
    let disciple = data
        .prestige_classes
        .iter_mut()
        .find(|c| c.key.as_str() == "dragon-disciple")
        .expect("Dragon Disciple");
    let bloodline = FeatureSelectionKey::new("dragon-disciple/bloodline");
    disciple.selections.push(FeatureSelection {
        key: bloodline.clone(),
        name: "Blood of Dragons".into(),
        summary: String::new(),
        guide: GuideRef::new("prestige-classes#blood-of-dragons"),
        options: Vec::new(),
        shared_options: draconic.clone(),
        option_levels: BTreeSet::from([ClassKey::new("dragon-disciple")]),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: true,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    });
    let row = disciple
        .levels
        .iter_mut()
        .find(|row| row.level == 1)
        .expect("level 1");
    row.selections.push(bloodline);
    let sorcerer_bloodline = FeatureSelectionKey::new("sorcerer/bloodline");
    for row in &mut disciple.levels {
        option_list_slots(row, &["dragon-disciple/bonus-feats"], &sorcerer_bloodline);
    }
    for feature in &mut disciple.features {
        let ability = match feature.name.as_str() {
            "Strength +2" => rules::Ability::Strength,
            "Constitution +2" => rules::Ability::Constitution,
            "Intelligence +2" => rules::Ability::Intelligence,
            _ => continue,
        };
        feature.ability_bonuses = vec![AbilityBonus::Fixed { ability, amount: 2 }];
    }
    let mut entry = vec![Prerequisite::SpontaneousSpellLevel {
        spell_type: SpellType::Arcane,
        level: 1,
    }];
    let mut either: Vec<Prerequisite> = draconic
        .iter()
        .cloned()
        .map(Prerequisite::SelectionOption)
        .collect();
    either.push(Prerequisite::All(vec![
        Prerequisite::NoOptionFrom(sorcerer_bloodline.clone()),
        Prerequisite::NoOptionFrom(FeatureSelectionKey::new("bloodrager/bloodline")),
    ]));
    entry.push(Prerequisite::Any(either));
    // The stand-in "can cast 1st-level arcane spells" goes: the spontaneous one says more.
    disciple.prerequisite = Some(match disciple.prerequisite.take() {
        Some(Prerequisite::All(mut children)) => {
            let before = children.len();
            children.retain(|c| {
                *c != Prerequisite::SpellLevel {
                    spell_type: SpellType::Arcane,
                    level: 1,
                }
            });
            // The hand-fixed data has it; a fresh extraction leaves the unparsed line out.
            assert!(children.len() + 1 >= before, "the stand-in spell level");
            children.extend(entry);
            Prerequisite::All(children)
        }
        other => panic!("Dragon Disciple's entry: {other:?}"),
    });
    log.push("dragon-disciple: bloodline, entry, bloodline feats, ability bonuses".into());

    // Eldritch Scion: "Spells are cast spontaneously", "Must select a bloodline which provides additional spells and
    // abilities", "but not the feats that Sorcerers would receive"; Dragon Disciple levels "do not hinder bloodline
    // progression" (magus).
    let magus = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "magus")
        .expect("magus");
    let scion = magus
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "magus/eldritch-scion")
        .expect("Eldritch Scion");
    scion.spontaneous = Some(true);
    let scion_bloodline = FeatureSelectionKey::new("magus/eldritch-scion/bloodline");
    scion.selections.push(FeatureSelection {
        key: scion_bloodline.clone(),
        name: "Bloodline (Eldritch Scion)".into(),
        summary: String::new(),
        guide: GuideRef::new("magus#eldritch-scion"),
        options: Vec::new(),
        shared_options: every.clone(),
        option_levels: BTreeSet::from([ClassKey::new("magus"), ClassKey::new("dragon-disciple")]),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    });
    scion
        .levels
        .iter_mut()
        .find(|row| row.level == 1)
        .expect("level 1")
        .selections
        .push(scion_bloodline);
    log.push("magus/eldritch-scion: spontaneous, bloodline".into());

    // Dragonheir Scion: "Gains a Draconic Bloodline" with its own rows (Draconic Defences, Strike, Presence, Wings,
    // Might of Wyrms), so it skips every Draconic row.
    let fighter = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "fighter")
        .expect("fighter");
    let dragonheir = fighter
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "fighter/dragonheir-scion")
        .expect("Dragonheir Scion");
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
    let heir_bloodline = FeatureSelectionKey::new("fighter/dragonheir-scion/bloodline");
    dragonheir.selections.push(FeatureSelection {
        key: heir_bloodline.clone(),
        name: "Draconic Bloodline".into(),
        summary: String::new(),
        guide: GuideRef::new("fighter#draconic-bloodline"),
        options: Vec::new(),
        shared_options: draconic,
        option_levels: BTreeSet::from([ClassKey::new("fighter")]),
        skipped_levels: rows,
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    });
    let row = dragonheir
        .levels
        .iter_mut()
        .find(|row| row.level == 1)
        .expect("level 1");
    let before = row.features.len();
    row.features
        .retain(|f| f.as_str() != "fighter/dragonheir-scion/draconic-bloodline");
    assert_eq!(
        row.features.len() + 1,
        before,
        "the Draconic Bloodline cell"
    );
    row.selections.push(heir_bloodline);
    log.push("fighter/dragonheir-scion: draconic bloodline, own rows".into());

    // Elemental Specialist: "At 1st level, an Elemental Specialist selects one element (from Acid, Cold,
    // Electricity and Fire)": the choice is a selection.
    let wizard = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "wizard")
        .expect("wizard");
    let specialist = wizard
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "wizard/elemental-specialist")
        .expect("Elemental Specialist");
    let focused = FeatureSelectionKey::new("wizard/elemental-specialist/focused-element");
    let mut choice = selection(
        focused.as_str(),
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
    specialist.selections.push(choice);
    let row = specialist
        .levels
        .iter_mut()
        .find(|row| row.level == 1)
        .expect("level 1");
    let feature = FeatureKey::new(focused.as_str());
    let before = row.features.len();
    row.features.retain(|f| *f != feature);
    assert_eq!(row.features.len() + 1, before, "the Focused Element cell");
    row.selections.push(focused);
    log.push("wizard/elemental-specialist: Focused Element is a selection".into());

    // Winter Witch: "Shaman (Frost Spirit)".
    let witch = data
        .prestige_classes
        .iter_mut()
        .find(|c| c.key.as_str() == "winter-witch")
        .expect("Winter Witch");
    let stand_in = Prerequisite::ClassLevel {
        class: ClassKey::new("shaman"),
        level: 1,
    };
    let frost = Prerequisite::SelectionOption(SelectionOptionKey::new("shaman/spirit/frost"));
    let replaced = replace_leaf(
        witch.prerequisite.as_mut().expect("entry"),
        &stand_in,
        &frost,
    );
    // The stand-in is a hand fix; a fresh extraction leaves the whole "Required classes" line out.
    assert!(replaced <= 1, "the Winter Witch's shaman stand-in");
    if replaced == 1 {
        log.push("winter-witch: needs the Frost spirit".into());
    }
}

/// Replaces every leaf equal to `old`; returns how many.
pub(crate) fn replace_leaf(
    prerequisite: &mut Prerequisite,
    old: &Prerequisite,
    new: &Prerequisite,
) -> usize {
    if prerequisite == old {
        *prerequisite = new.clone();
        return 1;
    }
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children
            .iter_mut()
            .map(|child| replace_leaf(child, old, new))
            .sum(),
        _ => 0,
    }
}

/// Second Bloodline ("You can select a second bloodline, gaining all its benefits"), Second Bloodrager Bloodline,
/// Second Spirit ("You gain a second Shaman Spirit"), and Bloodline Ascendance ("You gain the 20th level ability of
/// your bloodline").
fn mythic(data: &mut GameData, log: &mut Log) {
    // Second Spirit's "Prerequisites: Spirit": a spirit held (the Spirit cell is the selection now).
    let spirits: Vec<Prerequisite> = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "shaman")
        .and_then(|c| {
            c.selections
                .iter()
                .find(|s| s.key.as_str() == "shaman/spirit")
        })
        .expect("the shaman's spirits")
        .options
        .iter()
        .map(|option| Prerequisite::SelectionOption(option.key.clone()))
        .collect();
    let second = data
        .mythic_abilities
        .iter_mut()
        .find(|a| a.key.as_str() == "second-spirit")
        .expect("Second Spirit");
    let spirit = Prerequisite::Feature {
        feature: FeatureKey::new("shaman/spirit"),
        count: 1,
    };
    let replaced = replace_leaf(
        second
            .prerequisite
            .as_mut()
            .expect("Second Spirit's prerequisite"),
        &spirit,
        &Prerequisite::Any(spirits),
    );
    assert_eq!(replaced, 1, "Second Spirit's Spirit");
    let grants = [
        ("second-bloodline", "sorcerer/bloodline"),
        ("second-bloodrager-bloodline", "bloodrager/bloodline"),
        ("second-spirit", "shaman/spirit"),
    ];
    for (ability, selection) in grants {
        let ability = data
            .mythic_abilities
            .iter_mut()
            .find(|a| a.key.as_str() == ability)
            .unwrap_or_else(|| panic!("no {ability}"));
        ability.selections.push(FeatureSelectionKey::new(selection));
        log.push(format!("{}: another {selection}", ability.key));
    }
    let key = FeatureKey::new("sorcerer/bloodline-ascendance");
    let sorcerer = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == "sorcerer")
        .expect("sorcerer");
    sorcerer.features.push(Feature {
        option_row: Some(OptionRow {
            selection: FeatureSelectionKey::new("sorcerer/bloodline"),
            level: 20,
        }),
        ..feature(
            &key,
            "Bloodline Ascendance",
            "mythic-paths#bloodline-ascendance".into(),
        )
    });
    let ascendance = data
        .mythic_abilities
        .iter_mut()
        .find(|a| a.key.as_str() == "bloodline-ascendance")
        .expect("Bloodline Ascendance");
    ascendance.features.push(key);
    log.push("bloodline-ascendance: the bloodline's 20th-level row".into());

    // Mixed Bloodline: "Once per day, a Mixed-Blood Rager chooses a bloodline"; the shaman's Wandering Spirit "can
    // be changed each time you rest": daily choices.
    for class in &mut data.classes {
        for feature in class
            .features
            .iter_mut()
            .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.features))
        {
            if [
                "Mixed Bloodline",
                "Second Mixed Bloodline",
                "Wandering Spirit",
            ]
            .contains(&feature.name.as_str())
            {
                feature.daily_choice = true;
                log.push(format!("{}: a daily choice", feature.key));
            }
        }
    }
}

/// Disruptive: "Fighter Level 6 or Arcane bloodline".
fn prerequisites(data: &mut GameData, log: &mut Log) {
    let Some(disruptive) = data
        .feats
        .iter_mut()
        .find(|f| f.key.as_str() == "disruptive")
    else {
        return;
    };
    let either = Prerequisite::Any(vec![
        Prerequisite::ClassLevel {
            class: ClassKey::new("fighter"),
            level: 6,
        },
        Prerequisite::SelectionOption(SelectionOptionKey::new("sorcerer/bloodline/arcane")),
        Prerequisite::SelectionOption(SelectionOptionKey::new("bloodrager/bloodline/arcane")),
    ]);
    // The "or" line was left out whole while "Arcane bloodline" named nothing.
    assert_eq!(disruptive.prerequisite, None, "Disruptive's prerequisite");
    disruptive.prerequisite = Some(either);
    log.push("disruptive: fighter 6 or an Arcane bloodline".into());
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

/// Features no row, option row, prerequisite or grant names any more, and lists no slot or option names, go.
pub(crate) fn drop_unused(data: &mut GameData, touched: &[&str], log: &mut Log) {
    let mut features: BTreeSet<FeatureKey> = BTreeSet::new();
    let mut lists: BTreeSet<FeatListKey> = BTreeSet::new();
    let slot_lists = |slots: &[FeatSlot], lists: &mut BTreeSet<FeatListKey>| {
        for slot in slots {
            if let FeatSlot::OneOf { list } = slot {
                lists.insert(list.clone());
            }
        }
    };
    for class in data.classes.iter().chain(&data.prestige_classes) {
        for row in &class.levels {
            features.extend(row.features.iter().cloned());
            slot_lists(&row.feat_slots, &mut lists);
        }
        for archetype in &class.archetypes {
            for row in &archetype.levels {
                features.extend(row.features.iter().cloned());
                slot_lists(&row.feat_slots, &mut lists);
            }
        }
        for counts in &class.counts_as {
            if let rules::CountsToward::Feature(feature) = &counts.toward {
                features.insert(feature.clone());
            }
        }
        for selection in class
            .selections
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        {
            for option in &selection.options {
                for row in &option.levels {
                    features.extend(row.features.iter().cloned());
                    lists.extend(row.feat_list.iter().cloned());
                }
            }
        }
    }
    for path in &data.mythic_paths {
        for row in &path.ranks {
            features.extend(row.features.iter().cloned());
            slot_lists(&row.feat_slots, &mut lists);
        }
        if let Some(switch) = &path.switch {
            features.extend(switch.features.iter().cloned());
            features.extend(switch.dropped_features.iter().cloned());
        }
    }
    for ability in &data.mythic_abilities {
        features.extend(ability.features.iter().cloned());
    }
    let mut named = |prerequisite: &Option<Prerequisite>| {
        if let Some(prerequisite) = prerequisite {
            collect_features(prerequisite, &mut features);
        }
    };
    for feat in &data.feats {
        named(&feat.prerequisite);
    }
    for ability in &data.mythic_abilities {
        named(&ability.prerequisite);
    }
    for feat in &data.mythic_feats {
        named(&feat.prerequisite);
    }
    for class in data.classes.iter().chain(&data.prestige_classes) {
        named(&class.prerequisite);
        for archetype in &class.archetypes {
            named(&archetype.prerequisite);
        }
        for selection in class
            .selections
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        {
            for option in &selection.options {
                named(&option.prerequisite);
            }
        }
    }
    for path in &data.mythic_paths {
        for selection in &path.selections {
            for option in &selection.options {
                named(&option.prerequisite);
            }
        }
    }
    // Only the classes these families touch.
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        if !touched.contains(&class.key.as_str()) {
            continue;
        }
        let keep =
            |feature: &Feature| features.contains(&feature.key) || feature.option_row.is_some();
        let before = class.features.len();
        class.features.retain(keep);
        let mut dropped = before - class.features.len();
        for archetype in &mut class.archetypes {
            let before = archetype.features.len();
            archetype.features.retain(keep);
            dropped += before - archetype.features.len();
        }
        let before = class.feat_lists.len();
        class.feat_lists.retain(|list| lists.contains(&list.key));
        let mut lists_dropped = before - class.feat_lists.len();
        for archetype in &mut class.archetypes {
            let before = archetype.feat_lists.len();
            archetype
                .feat_lists
                .retain(|list| lists.contains(&list.key));
            lists_dropped += before - archetype.feat_lists.len();
        }
        if dropped + lists_dropped > 0 {
            log.push(format!(
                "{}: {dropped} unused features and {lists_dropped} unused lists dropped",
                class.key
            ));
        }
    }
}

const TOUCHED: [&str; 10] = [
    "sorcerer",
    "bloodrager",
    "wizard",
    "cavalier",
    "arcanist",
    "shaman",
    "ranger",
    "magus",
    "fighter",
    "dragon-disciple",
];

fn collect_features(prerequisite: &Prerequisite, features: &mut BTreeSet<FeatureKey>) {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => {
            for child in children {
                collect_features(child, features);
            }
        }
        Prerequisite::Feature { feature, .. } => {
            features.insert(feature.clone());
        }
        _ => {}
    }
}

/// A name without its parenthesised tail, for comparing: "Manifestation (Battle)" is "Manifestation".
pub(crate) fn bare(name: &str) -> String {
    norm(name.split(" (").next().unwrap_or(name))
}
