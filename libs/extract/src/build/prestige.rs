//! The prestige classes, with their skills.

use std::collections::BTreeSet;

use rules::{
    BabProgression, Class, ClassKey, ClassLevelRow, DataAt, GuideRef, SaveProgression, SkillKey,
};

use crate::grammar::{self, Unparsed};
use crate::html::{Page, Section};
use crate::names::norm;
use crate::prose;
use crate::report::Kind;
use crate::resolve::Context;

use super::sections::{has_progression_table, headings, selection_names, without_options};
use super::{Builder, FeatureOwner, Pending, Scope};

impl<'g> Builder<'g> {
    pub(super) fn prestige_classes(&mut self) -> Vec<Class> {
        let guide = self.guide;
        let page = guide.page("prestige-classes");
        let Some(root) = page.root.children.first() else {
            return Vec::new();
        };
        let sections: Vec<&Section> = root
            .children
            .iter()
            .filter(|s| s.level == 3 && has_progression_table(s))
            .collect();
        for section in &sections {
            self.catalog
                .classes
                .insert(norm(&section.name), ClassKey::new(section.anchor.clone()));
        }
        let mut classes: Vec<Class> = sections
            .into_iter()
            .map(|section| self.prestige_class(page, section))
            .collect();
        // What the guide's prose says of advancement and "counts as" (ticket 28), on the classes it names.
        crate::class_rules::apply_prestige(&mut classes);
        classes
    }

    fn prestige_class(&mut self, page: &'g Page, section: &'g Section) -> Class {
        let key = ClassKey::new(section.anchor.clone());
        let rows = self.level_table(page, section, &section.anchor);
        let mut granted: BTreeSet<String> = BTreeSet::new();
        for row in &rows {
            for grant in &row.grants {
                granted.insert(norm(&grant.name));
            }
        }
        let mut selections = self.selections(page, section, &section.anchor, &granted, &key, None);
        let owner = FeatureOwner::Class(key.clone());
        let (provided, feat_lists) = self.provided(&owner);
        let scope = Scope {
            headings: without_options(
                headings(page, section, &section.anchor, &owner),
                &selections,
            ),
            selections: selection_names(&selections),
            prefix: section.anchor.clone(),
            owner: Some(owner),
            guide: page.guide(&section.anchor),
            archetype_anchor: None,
            path: false,
        };
        selections.extend(provided);
        let numbered = self.level_rows(page, &rows, None, None, &scope);
        let mut levels: Vec<ClassLevelRow> = numbered.iter().map(|(_, row)| row.clone()).collect();
        // A CL column's "+1" advances the spellcasting the class's requirements name.
        let (arcane, divine) = prose::prestige_advances(section);
        let mut advances = false;
        for row in levels.iter_mut() {
            if rows
                .iter()
                .any(|parsed| parsed.level == row.level && parsed.advances_spellcasting)
            {
                advances = true;
                row.advances_arcane = arcane;
                row.advances_divine = divine;
            }
        }
        if advances && arcane && divine && !section.anchor.contains("mystic-theurge") {
            self.report.add(
                Kind::ProseRule,
                page.guide(&section.anchor),
                "",
                "advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine \
                 reads as one of each",
            );
        }

        // Entry requirements.
        let mut conds = Vec::new();
        for item in section.list_items() {
            match grammar::parse_requirement(item) {
                None => {
                    if item.to_ascii_lowercase().starts_with("must ")
                        || item.to_ascii_lowercase().starts_with("required")
                    {
                        conds.push((item.clone(), Err(Unparsed(item.clone()))));
                    }
                }
                Some(Ok(parsed)) => {
                    for cond in parsed {
                        conds.push((item.clone(), Ok(cond)));
                    }
                }
                Some(Err(error)) => conds.push((item.clone(), Err(error))),
            }
        }
        let stated = self.picks.stated_for(&DataAt::Class(key.clone()));
        if !stated.is_empty() {
            self.add_prerequisite(
                &DataAt::Class(key.clone()),
                page.guide(&section.anchor),
                stated,
            );
        }
        if !conds.is_empty() {
            self.pending.push(Pending {
                owner: DataAt::Class(key.clone()),
                at: page.guide(&section.anchor),
                conds,
                context: Context::default(),
                kind: Kind::Requirement,
                extra: Vec::new(),
            });
        }

        let (skill_points, mut class_skills) = self.prestige_skills(page, section);
        self.picks
            .fix_class_skills(section.anchor.as_str(), &mut class_skills);
        let mut class = Class {
            key,
            name: section.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(page.guide(&section.anchor)),
            skill_points,
            // Set from the table below.
            bab: BabProgression::Full,
            fort: SaveProgression::Good,
            reflex: SaveProgression::Good,
            will: SaveProgression::Good,
            save_exceptions: Vec::new(),
            advances: rules::Advancement::Each,
            spontaneous: false,
            counts_as: std::collections::BTreeSet::new(),
            class_skills,
            prerequisite: None,
            features: Vec::new(),
            selections,
            feat_lists,
            levels,
            archetypes: Vec::new(),
        };
        self.set_progressions(&mut class, &numbered, true, &page.guide(&section.anchor));
        class
    }

    pub(super) fn prestige_skills(
        &mut self,
        page: &Page,
        section: &Section,
    ) -> (u8, BTreeSet<SkillKey>) {
        let mut points = None;
        for item in section.list_items() {
            let lower = item.to_ascii_lowercase();
            if lower.contains("skill points") {
                points = lower
                    .split_whitespace()
                    .next()
                    .and_then(|number| number.parse().ok());
            }
        }
        let mut skills = BTreeSet::new();
        let mut found = false;
        let lines: Vec<String> = section
            .paragraphs()
            .map(|paragraph| paragraph.join(" "))
            .chain(section.list_items().cloned())
            .collect();
        for line in lines {
            let lower = line.to_ascii_lowercase();
            let Some(list) = lower.strip_prefix("class skills:") else {
                continue;
            };
            found = true;
            let list = &line[line.len() - list.len()..];
            if norm(list.trim_end_matches('.')) == "all" {
                skills.extend(self.catalog.skills.values().cloned());
                continue;
            }
            for name in list.split(',') {
                let name = name.trim().trim_end_matches('.');
                match self.catalog.skills.get(&norm(name)) {
                    Some(key) => {
                        skills.insert(key.clone());
                    }
                    None => self.report.add(
                        Kind::Cell,
                        page.guide(&section.anchor),
                        name,
                        "class skill not in the skills table",
                    ),
                }
            }
        }
        if points.is_none() || !found {
            self.report.add(
                Kind::Cell,
                page.guide(&section.anchor),
                "",
                "skill points or class skills not found",
            );
        }
        (points.unwrap_or(0), skills)
    }
}
