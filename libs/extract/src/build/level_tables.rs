//! A class's or archetype's progression table: the option family's plan for it, what each cell names, and its level
//! rows with their BAB, saves and spells.

use std::collections::{BTreeMap, BTreeSet};

use rules::{Class, ClassLevelRow, FeatKey, FeatureKey, SpellType};

use crate::html::{Page, Section};
use crate::names::{norm, number_forms, slug};
use crate::progressions::{self, Numbers};
use crate::report::Kind;
use crate::tables::{self, Row};

use super::sections::find_heading;
use super::{Builder, FeatureOwner, Granted, Scope};

impl<'g> Builder<'g> {
    /// How the table of `class` (its archetype `archetype`'s, or its own) takes the class's option family, from the
    /// features and feats its cells name.
    pub(super) fn plan(
        &mut self,
        class: &str,
        archetype: Option<&str>,
        rows: &[Row],
        scope: &Scope<'g>,
    ) {
        let Some(taking) = self.picks.family_of(class) else {
            return;
        };
        let cells: Vec<crate::option_levels::Cell> = rows
            .iter()
            .flat_map(|row| row.grants.iter().map(move |grant| (row.level, &grant.name)))
            .map(|(level, text)| {
                let (feature, feat) = self.cell_of(scope, text);
                crate::option_levels::Cell {
                    level,
                    text: text.clone(),
                    feature: feature.map(|(key, _, name, _)| (key, name)),
                    feat,
                }
            })
            .collect();
        let plan = taking.plan(archetype, &cells);
        self.plans
            .insert(archetype.unwrap_or(class).to_owned(), plan);
    }

    /// The feature a table cell names (its key, owner, name and guide link), or the feat it grants, as [`Self::grant`]
    /// reads it, without defining anything: for planning a table before its rows are read.
    #[allow(clippy::type_complexity)]
    pub(super) fn cell_of(
        &self,
        scope: &Scope<'g>,
        name: &str,
    ) -> (
        Option<(FeatureKey, FeatureOwner, String, String)>,
        Option<FeatKey>,
    ) {
        let cell_norm = norm(name);
        if cell_norm.ends_with("-feat") || cell_norm.ends_with("-feats") {
            return (None, None);
        }
        if number_forms(&cell_norm).iter().any(|form| {
            scope
                .selections
                .iter()
                .any(|(names, _)| names.contains(form))
        }) {
            return (None, None);
        }
        if let Some(heading) = find_heading(scope, name) {
            return (
                Some((
                    FeatureKey::new(format!("{}/{}", heading.prefix, heading.section.anchor)),
                    heading.owner.clone(),
                    heading.section.name.clone(),
                    heading.page.guide(&heading.section.anchor),
                )),
                None,
            );
        }
        if !scope.path
            && let Some(feat) = self.feat_named(name)
        {
            return (None, Some(feat));
        }
        let Some(owner) = scope.owner.clone() else {
            return (None, None);
        };
        (
            Some((
                FeatureKey::new(format!("{}/{}", scope.prefix, slug(name))),
                owner,
                name.to_owned(),
                scope.guide.clone(),
            )),
            None,
        )
    }

    /// The tables directly in a class or archetype section, merged by level: numbers, spells and grants.
    pub(super) fn level_table(&mut self, page: &Page, section: &Section, anchor: &str) -> Vec<Row> {
        let mut merged: BTreeMap<u8, Row> = BTreeMap::new();
        for rows in section.tables() {
            if !is_level_table(rows) {
                continue;
            }
            match tables::parse_table(&rows[0], &rows[1..]) {
                Ok(parsed) => {
                    for row in parsed {
                        match merged.get_mut(&row.level) {
                            None => {
                                merged.insert(row.level, row);
                            }
                            Some(existing) => {
                                existing.bab = existing.bab.or(row.bab);
                                existing.fort = existing.fort.or(row.fort);
                                existing.reflex = existing.reflex.or(row.reflex);
                                existing.will = existing.will.or(row.will);
                                existing.advances_spellcasting |= row.advances_spellcasting;
                                existing.spell_level = existing.spell_level.max(row.spell_level);
                                existing.grants.extend(row.grants);
                            }
                        }
                    }
                }
                Err(error) => self.report.add(
                    Kind::Cell,
                    page.guide(anchor),
                    rows[0].join(" | "),
                    format!("table not understood: {error}"),
                ),
            }
        }
        merged.into_values().collect()
    }

    /// A class's BAB and save progressions from its table's numbers. A save cell they don't give is kept as an
    /// exception; a BAB cell they don't give is reported.
    pub(super) fn set_progressions(
        &mut self,
        class: &mut Class,
        levels: &[(Numbers, ClassLevelRow)],
        prestige: bool,
        guide: &str,
    ) {
        let numbers: Vec<(u8, Numbers)> = levels
            .iter()
            .map(|(numbers, row)| (row.level, *numbers))
            .collect();
        let found = progressions::progressions(&numbers, prestige);
        for level in found.bab_misfits {
            self.report.add(
                Kind::Cell,
                guide.to_owned(),
                format!("level {level}"),
                format!(
                    "BAB doesn't follow the class's {:?} progression, whose value is used",
                    found.bab
                ),
            );
        }
        class.bab = found.bab;
        class.fort = found.fort;
        class.reflex = found.reflex;
        class.will = found.will;
        class.save_exceptions = found.save_exceptions;
    }

    /// Level rows from a parsed table, each with its BAB and save cells. Numbers missing from an archetype's table
    /// come from its base class's rows.
    pub(super) fn level_rows(
        &mut self,
        page: &Page,
        rows: &[Row],
        base: Option<&[(Numbers, ClassLevelRow)]>,
        spell_type: Option<SpellType>,
        scope: &Scope<'g>,
    ) -> Vec<(Numbers, ClassLevelRow)> {
        let mut out = Vec::new();
        let has_spell_columns = rows.iter().any(|row| row.spell_level > 0);
        // A table leaves out levels that grant nothing; an archetype has every level of its base class.
        let last = rows
            .iter()
            .map(|row| row.level)
            .chain(base.map(|base| base.len() as u8))
            .max()
            .unwrap_or(0);
        let empty: Vec<Row> = (1..=last)
            .map(|level| Row {
                level,
                bab: None,
                fort: None,
                reflex: None,
                will: None,
                advances_spellcasting: false,
                spell_level: 0,
                grants: Vec::new(),
            })
            .collect();
        for blank in &empty {
            let row = rows
                .iter()
                .find(|row| row.level == blank.level)
                .unwrap_or(blank);
            let base_row = base.and_then(|base| base.iter().find(|(_, r)| r.level == row.level));
            let base_numbers = base_row.map(|(numbers, _)| *numbers);
            let base_row = base_row.map(|(_, row)| row);
            let number = |own: Option<u8>, from_base: Option<u8>| own.or(from_base);
            let (bab, fort, reflex, will) = (
                number(row.bab, base_numbers.map(|n| n.bab)),
                number(row.fort, base_numbers.map(|n| n.fort)),
                number(row.reflex, base_numbers.map(|n| n.reflex)),
                number(row.will, base_numbers.map(|n| n.will)),
            );
            let (Some(bab), Some(fort), Some(reflex), Some(will)) = (bab, fort, reflex, will)
            else {
                self.report.add(
                    Kind::Cell,
                    scope.guide.clone(),
                    format!("level {}", row.level),
                    "no BAB or saves for this level; stored as 0",
                );
                continue;
            };
            let spell_level = if has_spell_columns {
                row.spell_level
            } else {
                base_row.map_or(0, |r| r.arcane_spell_level.max(r.divine_spell_level))
            };
            let numbers = Numbers {
                bab,
                fort,
                reflex,
                will,
            };
            let mut level_row = ClassLevelRow {
                level: row.level,
                arcane_spell_level: 0,
                divine_spell_level: 0,
                advances_arcane: false,
                advances_divine: false,
                features: Vec::new(),
                feats: BTreeSet::new(),
                feat_slots: Vec::new(),
                selections: Vec::new(),
                feats_for: Default::default(),
            };
            match spell_type {
                Some(SpellType::Arcane) => level_row.arcane_spell_level = spell_level,
                Some(SpellType::Divine) => level_row.divine_spell_level = spell_level,
                None => {}
            }
            for grant in &row.grants {
                match self.grant(page, scope, &grant.name, row.level) {
                    Granted::Feature(key) => level_row.features.push(key),
                    Granted::Selection(key) => level_row.selections.push(key),
                    Granted::Slots(slots) => level_row.feat_slots.extend(slots),
                    Granted::Feat(feat) => {
                        level_row.feats.insert(feat);
                    }
                    Granted::Nothing => {}
                    Granted::SelectionKeepingFeature(_) => unreachable!("grant settles it"),
                    Granted::Feats(feats, feature) => {
                        level_row.feats.extend(feats);
                        level_row.features.extend(feature);
                    }
                    Granted::FeatureAndFeats(_) => unreachable!("grant settles it"),
                    Granted::FeatsFor(feats, feats_for) => {
                        level_row.feats.extend(feats);
                        level_row.feats_for.extend(feats_for);
                    }
                }
            }
            if let Some(owner) = &scope.owner {
                level_row.selections.extend(
                    self.picks
                        .row_selections_of(owner.key(), row.level)
                        .cloned(),
                );
                level_row
                    .feats
                    .extend(self.picks.row_feats_of(owner, row.level));
                level_row
                    .feat_slots
                    .extend(self.picks.row_slots_of(owner.key(), row.level).cloned());
                for feature in self.picks.row_features_of(owner.key(), row.level) {
                    if !level_row.features.contains(feature) {
                        level_row.features.push(feature.clone());
                    }
                }
            }
            out.push((numbers, level_row));
        }
        out
    }
}

pub(super) fn is_level_table(rows: &[Vec<String>]) -> bool {
    let Some(header) = rows.first() else {
        return false;
    };
    matches!(header.first().map(String::as_str), Some("Lvl" | "Level"))
        && header.iter().skip(1).all(|cell| {
            let cell = cell.trim();
            matches!(
                cell,
                "AB" | "BAB" | "CL" | "Fort" | "Reflex" | "Will" | "Feat" | "Feats" | "AC"
            ) || cell.starts_with("Special")
                || cell.parse::<u8>().is_ok()
        })
}
