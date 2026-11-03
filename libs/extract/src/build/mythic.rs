//! The mythic ranks' Character levels, the mythic abilities, the mythic feats and the mythic paths.

use std::collections::BTreeSet;

use rules::{
    DataAt, FeatureSelection, FeatureSelectionKey, GuideRef, MythicAbility, MythicAbilityGroup,
    MythicAbilityKey, MythicFeat, MythicFeatGroup, MythicFeatKey, MythicPath, MythicPathKey,
    MythicRankRow,
};

use crate::guide::{OTHER_PATHS, PATH_PAGES};
use crate::html::{Page, Section};
use crate::names::{norm, number_forms};
use crate::prose;
use crate::report::Kind;
use crate::resolve::Context;
use crate::tables;

use super::sections::{headings, selection_names, without_options};
use super::selections::mostly_granted;
use super::{Builder, FeatureOwner, Granted, Scope};

/// The Character level of each Mythic rank (facts from the user, not the guide).
pub const MYTHIC_ASCENSIONS: [(u8, u8); 10] = [
    (1, 6),
    (2, 7),
    (3, 9),
    (4, 11),
    (5, 14),
    (6, 15),
    (7, 16),
    (8, 17),
    (9, 17),
    (10, 19),
];

impl<'g> Builder<'g> {
    pub(super) fn mythic_abilities(&mut self) -> Vec<MythicAbility> {
        let page = self.guide.page("mythic-paths");
        let Some(section) = page.root.find("mythic-abilities") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for group in section.children.iter().filter(|s| s.level == 4) {
            let heading = group.name.trim_end_matches(" Abilities");
            let Some(group_name) = MythicAbilityGroup::ALL
                .into_iter()
                .find(|g| g.name() == heading)
            else {
                self.report.add(
                    Kind::Cell,
                    page.guide(&group.anchor),
                    &group.name,
                    "not one of the mythic ability groups; its abilities are left out",
                );
                continue;
            };
            for entry in &group.children {
                let key = MythicAbilityKey::new(entry.anchor.clone());
                self.catalog
                    .mythic_abilities
                    .insert(norm(&entry.name), key.clone());
                self.prerequisites_of(
                    page,
                    entry,
                    &DataAt::MythicAbility(key.clone()),
                    Context::default(),
                    Vec::new(),
                );
                let selections = self.picks.mythic_selections_of(&key);
                let features = self.picks.mythic_features_of(&key);
                out.push(MythicAbility {
                    key,
                    name: entry.name.clone(),
                    group: group_name,
                    summary: String::new(),
                    guide: GuideRef::new(page.guide(&entry.anchor)),
                    prerequisite: None,
                    features,
                    selections,
                });
            }
        }
        out
    }

    pub(super) fn mythic_feats(&mut self) -> Vec<MythicFeat> {
        let page = self.guide.page("mythic-paths");
        let Some(section) = page.root.find("mythic-feats") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for group in section.children.iter().filter(|s| s.level == 4) {
            let heading = group.name.trim_end_matches(" Feats");
            let Some(group_name) = MythicFeatGroup::ALL
                .into_iter()
                .find(|g| g.name() == heading)
            else {
                self.report.add(
                    Kind::Cell,
                    page.guide(&group.anchor),
                    &group.name,
                    "not one of the mythic feat groups; its feats are left out",
                );
                continue;
            };
            for entry in &group.children {
                let key = MythicFeatKey::new(entry.anchor.clone());
                self.catalog
                    .mythic_feats
                    .insert(norm(&entry.name), key.clone());
                self.prerequisites_of(
                    page,
                    entry,
                    &DataAt::MythicFeat(key.clone()),
                    Context::default(),
                    Vec::new(),
                );
                let extra_pick = self.picks.mythic_extra_pick_of(&key);
                let removes_racial_penalty = self.picks.removes_racial_penalty_of(&key);
                out.push(MythicFeat {
                    key,
                    name: entry.name.clone(),
                    group: group_name,
                    summary: String::new(),
                    guide: GuideRef::new(page.guide(&entry.anchor)),
                    prerequisite: None,
                    parameter: None,
                    removes_racial_penalty,
                    extra_pick,
                    repeat: Default::default(),
                });
            }
        }
        out
    }

    pub(super) fn mythic_paths(&mut self) -> Vec<MythicPath> {
        let guide = self.guide;
        let mythic_page = guide.page("mythic-paths");
        // The `mythic-paths` page's own headings (Hard to Kill, Bypass Epic Damage Reduction): each path whose
        // table grants one gets its own copy, keyed under the path.
        let shared = mythic_page.root.find("first-ascension");
        let mut paths = Vec::new();
        for slug_name in PATH_PAGES {
            let page = guide.page(slug_name);
            let Some(section) = page.root.children.first() else {
                continue;
            };
            paths.push(self.mythic_path(page, section, slug_name, page.slug.clone(), shared));
        }
        let other = guide.page("other-mythic-paths");
        for anchor in OTHER_PATHS {
            let Some(section) = other.root.find(anchor) else {
                self.report.add(
                    Kind::Cell,
                    other.guide(anchor),
                    "",
                    "path section not found",
                );
                continue;
            };
            paths.push(self.mythic_path(other, section, anchor, other.guide(anchor), shared));
        }
        paths
    }

    fn mythic_path(
        &mut self,
        page: &'g Page,
        section: &'g Section,
        slug_name: &str,
        guide: String,
        shared: Option<&'g Section>,
    ) -> MythicPath {
        let key = MythicPathKey::new(slug_name.to_owned());
        let mut rows = Vec::new();
        for table in section.tables() {
            let is_progression = table.first().is_some_and(|header| {
                header.first().map(String::as_str) == Some("Rank")
                    && header
                        .iter()
                        .any(|cell| cell.starts_with("Special") || cell == "Abilities")
            });
            if !is_progression {
                continue;
            }
            match tables::parse_table(&table[0], &table[1..]) {
                Ok(parsed) => rows = parsed,
                Err(error) => self.report.add(
                    Kind::Cell,
                    guide.clone(),
                    table[0].join(" | "),
                    format!("table not understood: {error}"),
                ),
            }
        }
        let granted: BTreeSet<String> = rows
            .iter()
            .flat_map(|row| row.grants.iter().map(|grant| norm(&grant.name)))
            .collect();
        let owner = FeatureOwner::Path(key.clone());
        let mut selections = Vec::new();
        for candidate in section.walk() {
            if candidate.level != 4 || candidate.children.is_empty() {
                continue;
            }
            let named = number_forms(&norm(&candidate.name))
                .iter()
                .any(|form| granted.contains(form));
            if named && !mostly_granted(candidate, &granted) {
                let key = FeatureSelectionKey::new(format!("{slug_name}/{}", candidate.anchor));
                let mut selection = FeatureSelection {
                    key,
                    name: candidate.name.clone(),
                    summary: String::new(),
                    guide: GuideRef::new(page.guide(&candidate.anchor)),
                    options: Vec::new(),
                    shared_options: BTreeSet::new(),
                    option_levels: BTreeSet::new(),
                    skipped_levels: BTreeSet::new(),
                    feat_lists_only: false,
                    only_without_option: false,
                    pool_of: None,
                    levels_behind: 0,
                    level_from: None,
                    once_per_option: false,
                    level_also_from: None,
                };
                self.add_options(page, candidate, &mut selection, None, Vec::new());
                selections.push(selection);
            }
        }
        let mut scope = Scope {
            headings: without_options(headings(page, section, slug_name, &owner), &selections),
            selections: selection_names(&selections),
            prefix: slug_name.to_owned(),
            owner: Some(owner),
            guide: guide.clone(),
            archetype_anchor: None,
            path: true,
        };
        if let Some(shared) = shared {
            let mythic_page = self.guide.page("mythic-paths");
            let owner = FeatureOwner::Path(key.clone());
            scope
                .headings
                .extend(headings(mythic_page, shared, slug_name, &owner));
        }
        let mut ranks = Vec::new();
        for row in &rows {
            let mut rank_row = MythicRankRow {
                rank: row.level,
                features: Vec::new(),
                feat_slots: Vec::new(),
                selections: Vec::new(),
            };
            for grant in &row.grants {
                match self.grant(page, &scope, prose::path_cell(&grant.name), row.level) {
                    Granted::Feature(feature) => rank_row.features.push(feature),
                    Granted::Selection(selection) => rank_row.selections.push(selection),
                    Granted::Slots(slots) => rank_row.feat_slots.extend(slots),
                    Granted::Nothing => {}
                    Granted::SelectionKeepingFeature(_) | Granted::FeatureAndFeats(_) => {
                        unreachable!("grant settles it")
                    }
                    Granted::Feat(_) | Granted::Feats(..) | Granted::FeatsFor(..) => {
                        unreachable!("a path's table grants a feat as a feature")
                    }
                }
            }
            ranks.push(rank_row);
        }
        // A switch rule holds what a path without a rank table gives (Legend).
        if rows.is_empty() && prose::path_switch(slug_name).is_none() {
            self.report.add(
                Kind::ProseRule,
                guide.clone(),
                "",
                "no rank table: the path's ranks are prose only",
            );
        }
        MythicPath {
            key: key.clone(),
            name: section.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(guide),
            features: Vec::new(),
            selections,
            feat_lists: Vec::new(),
            ranks,
            switch: prose::path_switch(slug_name),
        }
    }
}
