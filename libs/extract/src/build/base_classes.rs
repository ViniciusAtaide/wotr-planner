//! A base class and its archetypes, built where their pages are read, with the class's alignment and an
//! archetype's race and alignment rules.

use std::collections::BTreeSet;

use rules::{
    Archetype, ArchetypeKey, BabProgression, Class, ClassKey, DataAt, GuideRef, Prerequisite,
    SaveProgression, SkillKey, SpellType,
};

use crate::grammar::{self, Cond};
use crate::guide::BASE_CLASSES;
use crate::html::{Page, Section};
use crate::names::norm;
use crate::picks::ArchetypeSkills;
use crate::progressions;
use crate::report::Kind;
use crate::resolve::Context;
use crate::tables::Row;

use super::sections::{has_progression_table, headings, selection_names, without_options};
use super::{Builder, FeatureOwner, Pending, Scope};

/// A spontaneous caster's characteristics say so.
const SPONTANEOUS: &str = "Spells are cast spontaneously";

impl<'g> Builder<'g> {
    pub(super) fn base_class(&mut self, slug_name: &str) -> Class {
        let guide = self.guide;
        let page = guide.page(slug_name);
        let root = page.root.children.first().expect("a class page has an h2");
        let class_key = ClassKey::new(slug_name.to_owned());
        self.catalog
            .classes
            .insert(norm(&root.name), class_key.clone());
        let skill_rows = self.class_skill_rows();

        let base_rows = self.level_table(page, root, "");
        // Tables one level down (a sorcerer's bloodlines, a wizard's schools, a cavalier's orders) aren't read.
        for section in root.walk() {
            if section.level >= 4 && has_progression_table(section) {
                self.report.add(
                    Kind::Cell,
                    page.guide(&section.anchor),
                    section.name.clone(),
                    "a progression table under a feature heading isn't read, so what it grants is missing",
                );
            }
        }
        let archetype_sections: Vec<&Section> = root
            .children
            .iter()
            .filter(|s| s.level == 3 && has_progression_table(s))
            .collect();
        let archetype_rows: Vec<Vec<Row>> = archetype_sections
            .iter()
            .map(|section| self.level_table(page, section, &section.anchor))
            .collect();

        // Every cell name in the class's tables, for telling selections from features.
        let mut granted_names: BTreeSet<String> = BTreeSet::new();
        for rows in std::iter::once(&base_rows).chain(&archetype_rows) {
            for row in rows {
                for grant in &row.grants {
                    granted_names.insert(norm(&grant.name));
                }
            }
        }

        let base_spell_type = spell_type(root);
        // "Spells are cast spontaneously" in a class's or an archetype's characteristics (the others' say "Spells
        // must be prepared in advance"): Dragon Disciple's entry checks it.
        for section in root.walk() {
            let archetype = archetype_sections.iter().any(|a| std::ptr::eq(*a, section));
            assert!(
                !section.text().contains(SPONTANEOUS) || std::ptr::eq(section, root) || archetype,
                "{}#{}: {SPONTANEOUS} below an archetype",
                page.slug,
                section.anchor
            );
        }
        let mut class = Class {
            key: class_key.clone(),
            name: root.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(page.slug.clone()),
            skill_points: 0,
            // Set from the table below.
            bab: BabProgression::Full,
            fort: SaveProgression::Good,
            reflex: SaveProgression::Good,
            will: SaveProgression::Good,
            save_exceptions: Vec::new(),
            advances: rules::Advancement::Each,
            spontaneous: root.text().contains(SPONTANEOUS),
            counts_as: std::collections::BTreeSet::new(),
            class_skills: BTreeSet::new(),
            prerequisite: None,
            features: Vec::new(),
            selections: Vec::new(),
            feat_lists: Vec::new(),
            levels: Vec::new(),
            archetypes: Vec::new(),
        };
        let base_skills = skill_rows
            .iter()
            .position(|(name, _, _)| norm(name) == norm(&root.name));
        match base_skills {
            Some(index) => {
                class.skill_points = skill_rows[index].1;
                class.class_skills = skill_rows[index].2.clone();
            }
            None => {
                // Not in the skills table: the class page's own characteristics.
                let (points, skills) = self.prestige_skills(page, root);
                class.skill_points = points;
                class.class_skills = skills;
            }
        }
        self.class_alignment(page, root, &DataAt::Class(class_key.clone()));

        let base_selections =
            self.selections(page, root, slug_name, &granted_names, &class_key, None);
        let base_scope = Scope {
            headings: without_options(
                headings(
                    page,
                    root,
                    slug_name,
                    &FeatureOwner::Class(class_key.clone()),
                ),
                &base_selections,
            ),
            selections: selection_names(&base_selections),
            prefix: slug_name.to_owned(),
            owner: Some(FeatureOwner::Class(class_key.clone())),
            guide: page.slug.clone(),
            archetype_anchor: None,
            path: false,
        };
        class.selections = base_selections;
        let (selections, feat_lists) = self.provided(&FeatureOwner::Class(class_key.clone()));
        class.selections.extend(selections);
        class.feat_lists.extend(feat_lists);
        self.plan(slug_name, None, &base_rows, &base_scope);
        let base_levels = self.level_rows(page, &base_rows, None, base_spell_type, &base_scope);
        self.set_progressions(&mut class, &base_levels, false, &base_scope.guide);
        class.levels = base_levels.iter().map(|(_, row)| row.clone()).collect();

        for (section, rows) in archetype_sections.iter().zip(&archetype_rows) {
            let prefix = format!("{slug_name}/{}", section.anchor);
            let archetype_key = ArchetypeKey::new(prefix.to_owned());
            self.catalog
                .archetypes
                .entry(norm(&section.name))
                .or_default()
                .insert(archetype_key.clone());
            let owner = FeatureOwner::Archetype(class_key.clone(), archetype_key.clone());
            let selections = self.selections(
                page,
                section,
                &prefix,
                &granted_names,
                &class_key,
                Some(&archetype_key),
            );
            let (provided, feat_lists) = self.provided(&owner);
            let mut scope = Scope {
                headings: without_options(headings(page, section, &prefix, &owner), &selections),
                selections: selection_names(&selections),
                prefix: prefix.clone(),
                owner: Some(owner),
                guide: page.guide(&section.anchor),
                archetype_anchor: Some(section.anchor.clone()),
                path: false,
            };
            scope.headings.extend(base_scope.headings.iter().cloned());
            scope
                .selections
                .extend(base_scope.selections.iter().cloned());

            let loses_spells = section.list_items().any(|item| {
                let item = item.to_ascii_lowercase();
                let item = item.trim_end_matches('.');
                item == "loses all spellcasting"
                    || item == "loses spellcasting"
                    || item == "cannot cast spells"
            });
            let own_type = spell_type(section);
            let spell_type = if loses_spells {
                None
            } else {
                own_type.or(base_spell_type)
            };
            let mut selections = selections;
            selections.extend(provided);
            self.plan(slug_name, Some(&prefix), rows, &scope);
            let levels = self.level_rows(page, rows, Some(&base_levels), spell_type, &scope);
            for (numbers, row) in &levels {
                let base = base_levels.iter().find(|(_, base)| base.level == row.level);
                if base.is_some_and(|(base, _)| base != numbers) {
                    self.report.add(
                        Kind::Cell,
                        scope.guide.clone(),
                        format!("level {}", row.level),
                        "BAB or saves differ from the base class's; the base class's progressions are used",
                    );
                }
            }
            let levels = levels
                .into_iter()
                .map(|(_, row)| progressions::full_archetype_row(row))
                .collect();

            let (skill_points, class_skills) =
                match archetype_skills(&skill_rows, &root.name, &section.name) {
                    Some((points, skills)) => (
                        (points != class.skill_points).then_some(points),
                        (skills != class.class_skills).then_some(skills),
                    ),
                    None => (None, None),
                };
            // What the skills table doesn't say (the user's skill facts, the Sohei's Lore (Nature)).
            let (skill_points, class_skills) = match self.picks.archetype_skills_of(&prefix) {
                None => (skill_points, class_skills),
                Some(ArchetypeSkills::Own { points, skills }) => {
                    assert_eq!((skill_points, &class_skills), (None, &None), "{prefix}");
                    (Some(*points), Some(skills.clone()))
                }
                Some(ArchetypeSkills::ClassAnd(more)) => {
                    assert_eq!(class_skills, None, "{prefix} keeps its class's skills");
                    let mut skills = class.class_skills.clone();
                    for skill in more {
                        assert!(skills.insert(skill.clone()), "{prefix}: {skill} already");
                    }
                    (skill_points, Some(skills))
                }
            };
            let at = DataAt::Archetype(archetype_key.clone());
            self.archetype_requirements(page, section, &at);
            let stated = self.picks.stated_for(&at);
            if !stated.is_empty() {
                self.add_prerequisite(&at, page.guide(&section.anchor), stated);
            }
            class.archetypes.push(Archetype {
                key: archetype_key,
                name: section.name.clone(),
                summary: String::new(),
                guide: GuideRef::new(page.guide(&section.anchor)),
                prerequisite: None,
                skill_points,
                class_skills,
                spontaneous: section.text().contains(SPONTANEOUS).then_some(true),
                features: Vec::new(),
                selections,
                feat_lists,
                levels,
            });
        }
        // The class's option family, once every table is read: which archetype takes it, how, and the slots that
        // pick from the held option's list.
        if let Some(taking) = self.picks.family_of(slug_name) {
            taking.take(&mut class, &self.plans, &mut self.lists);
        }
        self.plans.clear();
        class
    }

    fn class_alignment(&mut self, page: &Page, section: &Section, owner: &DataAt) {
        for item in section.list_items() {
            if item.to_ascii_lowercase().starts_with("alignment must be") {
                self.queue_requirement(page, section, owner, item, None, Vec::new());
            }
        }
    }

    /// An archetype's race and alignment rules from its bullets.
    fn archetype_requirements(&mut self, page: &Page, section: &Section, owner: &DataAt) {
        let mut conds = Vec::new();
        for item in section.list_items() {
            let lower = item.to_ascii_lowercase();
            if let Some(race) = lower.strip_prefix("race must be ") {
                let race = item[item.len() - race.len()..]
                    .trim_end_matches('.')
                    .to_owned();
                conds.push((
                    item.clone(),
                    Ok(Cond::Named {
                        name: race,
                        count: None,
                    }),
                ));
            } else if lower.starts_with("alignment must be") || lower.starts_with("cannot be ") {
                let parsed = grammar::parse_line(item);
                conds.push((item.clone(), parsed));
            }
        }
        if !conds.is_empty() {
            self.pending.push(Pending {
                owner: owner.clone(),
                at: page.guide(&section.anchor),
                conds,
                context: Context::default(),
                kind: Kind::Requirement,
                extra: Vec::new(),
            });
        }
    }

    fn queue_requirement(
        &mut self,
        page: &Page,
        section: &Section,
        owner: &DataAt,
        item: &str,
        context: Option<Context>,
        extra: Vec<Prerequisite>,
    ) {
        let parsed = grammar::parse_line(item);
        self.pending.push(Pending {
            owner: owner.clone(),
            at: page.guide(&section.anchor),
            conds: vec![(item.to_owned(), parsed)],
            context: context.unwrap_or_default(),
            kind: Kind::Requirement,
            extra,
        });
    }
}

/// The spell type a class casts, from its characteristics ("May cast arcane spells up to 9th-level").
fn spell_type(section: &Section) -> Option<SpellType> {
    section.list_items().find_map(|item| {
        let lower = item.to_ascii_lowercase();
        if lower.contains("cast arcane spells") {
            Some(SpellType::Arcane)
        } else if lower.contains("cast divine spells") {
            Some(SpellType::Divine)
        } else {
            None
        }
    })
}

/// The archetype's row in the skills table, matched by name with the base class's rows before it.
fn archetype_skills(
    rows: &[(String, u8, BTreeSet<SkillKey>)],
    class: &str,
    archetype: &str,
) -> Option<(u8, BTreeSet<SkillKey>)> {
    let start = rows
        .iter()
        .position(|(name, _, _)| norm(name) == norm(class))?;
    let wanted = crate::prose::fix_name(archetype);
    rows[start + 1..]
        .iter()
        .take_while(|(name, _, _)| !BASE_CLASSES.contains(&norm(name).as_str()))
        .find(|(name, _, _)| crate::prose::fix_name(name) == wanted)
        .map(|(_, points, skills)| (*points, skills.clone()))
}
