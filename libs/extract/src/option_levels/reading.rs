//! Reading the guide for the families: feat names, pages and sections, an option's table and the cells it adds to
//! its family's, the heading a cell names, a bloodline's feats and two-column tables.

use std::collections::{BTreeMap, BTreeSet};

use rules::{FeatKey, GameData};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::norm;
use crate::tables::parse_table;

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
pub(super) fn table(page: &Page, anchor: &str) -> Vec<crate::tables::Row> {
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
pub(super) fn anchor_for(within: &Section, name: &str) -> Option<String> {
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

/// "Bloodline feats: A, B, C." (or "Bonus Feats:") in a character-creation bloodline section.
pub(super) fn bloodline_feats(section: &Section) -> Vec<String> {
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
pub(super) fn pairs(section: &Section, header: &str) -> Vec<(String, String)> {
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
