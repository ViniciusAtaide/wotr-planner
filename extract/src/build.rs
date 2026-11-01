//! Building [`GameData`] from the parsed guide pages.
//!
//! Generic rules do most of the work: anchors give keys, progression tables give level rows, `Prerequisites:`
//! paragraphs give prerequisites, a heading whose name a table grants and whose children it doesn't is a Feature
//! selection. The few rules the guide states only in prose are in [`crate::prose`]. Anything that can't be placed
//! goes to the [`Report`].

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Ability, AbilityModifier, Archetype, ArchetypeKey, Class, ClassKey, ClassLevelRow, Feat,
    FeatCategory, FeatKey, FeatSlot, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey,
    GameData, GuideRef, MythicAbility, MythicAbilityKey, MythicAscension, MythicFeat,
    MythicFeatKey, MythicPath, MythicPathKey, MythicRankRow, Prerequisite, Progression, Race,
    RaceKey, SelectionOption, SelectionOptionKey, Skill, SkillKey, SpellType, Subrace, SubraceKey,
};

use crate::grammar::{self, Cond, Unparsed};
use crate::guide::{BASE_CLASSES, Guide, OTHER_PATHS, PATH_PAGES};
use crate::html::{Block, Page, Section};
use crate::names::{norm, number_forms, slug};
use crate::prose;
use crate::report::{Kind, Report};
use crate::resolve::{Catalog, Context, all};
use crate::tables::{self, Row};

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

/// The skills table's column abbreviations.
const SKILL_COLUMNS: [(&str, &str); 11] = [
    ("ATH", "Athletics"),
    ("MOB", "Mobility"),
    ("TRI", "Trickery"),
    ("STE", "Stealth"),
    ("KNA", "Knowledge (Arcana)"),
    ("KNW", "Knowledge (World)"),
    ("LON", "Lore (Nature)"),
    ("LOR", "Lore (Religion)"),
    ("PCP", "Perception"),
    ("PER", "Persuasion"),
    ("UMD", "Use Magic Device"),
];

/// The feats page's category headings.
const FEAT_CATEGORIES: [(&str, FeatCategory); 12] = [
    ("weapon-feats", FeatCategory::Weapon),
    ("ranged-feats", FeatCategory::Ranged),
    ("unarmed-combat-feats", FeatCategory::UnarmedCombat),
    ("armour-and-shield-feats", FeatCategory::ArmourAndShield),
    ("magic-feats", FeatCategory::Magic),
    ("miscellaneous-feats", FeatCategory::Miscellaneous),
    ("mounted-combat-feats", FeatCategory::MountedCombat),
    ("combat-manoeuvres", FeatCategory::CombatManoeuvres),
    ("teamwork-feats", FeatCategory::Teamwork),
    ("skill-feats", FeatCategory::Skill),
    ("class-feats", FeatCategory::Class),
    ("racial-feats", FeatCategory::Racial),
];

/// What the extractor produced.
pub struct Extraction {
    pub data: GameData,
    pub report: Report,
}

/// Builds the game data from the guide.
pub fn extract(guide: &Guide) -> Extraction {
    let mut builder = Builder::new(guide);
    let progression = Progression {
        mythic_ascensions: MYTHIC_ASCENSIONS
            .iter()
            .map(|&(rank, character_level)| MythicAscension {
                rank,
                character_level,
            })
            .collect(),
    };
    builder.catalog.rank_levels = MYTHIC_ASCENSIONS.iter().copied().collect();
    let skills = builder.skills();
    let races = builder.races();
    let mut feats = builder.feats();
    let mut classes: Vec<Class> = BASE_CLASSES
        .iter()
        .map(|slug| builder.base_class(slug))
        .collect();
    let mut prestige_classes = builder.prestige_classes();
    let mythic_abilities = builder.mythic_abilities();
    let mythic_feats = builder.mythic_feats();
    let mut mythic_paths = builder.mythic_paths();
    feats.extend(builder.trickster_feats());

    prose::apply(
        &mut builder,
        &mut classes,
        &mut prestige_classes,
        &mut mythic_paths,
    );
    builder.place_features(&mut classes, &mut prestige_classes, &mut mythic_paths);

    let mut data = GameData {
        progression,
        races,
        skills,
        feats,
        classes,
        prestige_classes,
        mythic_abilities,
        mythic_feats,
        mythic_paths,
    };
    builder.resolve_prerequisites(&mut data);
    crate::references::drop_dangling(&mut data, &mut builder.report);
    for problem in data.check() {
        builder.report.add(
            Kind::Cell,
            problem.at.clone(),
            "",
            format!("fails the data check: {problem}"),
        );
    }
    Extraction {
        data: data.canonical(),
        report: builder.report,
    }
}

/// Prerequisite lines waiting for the whole catalog.
struct Pending {
    owner: String,
    at: String,
    conds: Vec<(String, Result<Cond, Unparsed>)>,
    context: Context,
    kind: Kind,
    /// Conditions already known, ANDed with the parsed ones.
    extra: Vec<Prerequisite>,
}

/// Where a feature is defined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeatureOwner {
    Class(ClassKey),
    Archetype(ClassKey, ArchetypeKey),
    Path(MythicPathKey),
    /// A feature from the `mythic-paths` page that any path's table grants: defined on the first path that does.
    Mythic,
}

/// A heading a table cell can name.
#[derive(Clone)]
struct Heading<'g> {
    section: &'g Section,
    page: &'g Page,
    /// The key prefix of what it defines: `rogue`, `rogue/dark-lurker`, `trickster`.
    prefix: String,
    owner: FeatureOwner,
}

/// The headings and selections a class, archetype or path table resolves its cells against, nearest first.
#[derive(Clone, Default)]
pub struct Scope<'g> {
    headings: Vec<Heading<'g>>,
    selections: Vec<(Vec<String>, FeatureSelectionKey)>,
    /// The owner of features with no heading of their own.
    prefix: String,
    owner: Option<FeatureOwner>,
    guide: String,
    /// The archetype's anchor, for headings disambiguated with it ("Sneak Attack (Eldritch Scoundrel)").
    archetype_anchor: Option<String>,
    /// A mythic path's table: a feat it names is granted as a feature, since only classes have bonus lists.
    path: bool,
}

/// What a cell grants.
enum Granted {
    Feature(FeatureKey),
    Selection(FeatureSelectionKey),
    Slots(Vec<FeatSlot>),
}

pub struct Builder<'g> {
    pub guide: &'g Guide,
    pub report: Report,
    pub catalog: Catalog,
    pending: Vec<Pending>,
    /// Every feature, where it's defined, and its definition.
    pub features: BTreeMap<FeatureKey, (FeatureOwner, Feature)>,
    /// Feat names, for bonus lists and granted feats.
    feat_names: Vec<(String, FeatKey)>,
}

impl<'g> Builder<'g> {
    fn new(guide: &'g Guide) -> Self {
        Self {
            guide,
            report: Report::default(),
            catalog: Catalog::default(),
            pending: Vec::new(),
            features: BTreeMap::new(),
            feat_names: Vec::new(),
        }
    }

    // Skills and races --------------------------------------------------------------------------------------

    fn skills(&mut self) -> Vec<Skill> {
        let page = self.guide.page("character-creation");
        let Some(section) = page.root.find("skills") else {
            self.report
                .add(Kind::Cell, page.guide("skills"), "", "no skills section");
            return Vec::new();
        };
        let mut skills = Vec::new();
        for rows in section.tables() {
            let Some(header) = rows.first() else { continue };
            if header.first().map(String::as_str) != Some("Skill") {
                continue;
            }
            let abilities = [
                ("STR", Ability::Strength),
                ("DEX", Ability::Dexterity),
                ("CON", Ability::Constitution),
                ("INT", Ability::Intelligence),
                ("WIS", Ability::Wisdom),
                ("CHA", Ability::Charisma),
            ];
            for row in &rows[1..] {
                let name = row[0].clone();
                let ability = header.iter().zip(row).find_map(|(column, cell)| {
                    (cell.contains('✓'))
                        .then(|| abilities.iter().find(|(abbr, _)| abbr == column))
                        .flatten()
                        .map(|(_, ability)| *ability)
                });
                let Some(ability) = ability else {
                    self.report
                        .add(Kind::Cell, page.guide("skills"), name, "no ability ticked");
                    continue;
                };
                let key = SkillKey::new(format!("skill:{}", slug(&name)));
                self.catalog.skills.insert(norm(&name), key.clone());
                skills.push(Skill { key, name, ability });
            }
        }
        skills
    }

    /// The skills table of classes and archetypes: name, skill points, class skills.
    fn class_skill_rows(&self) -> Vec<(String, u8, BTreeSet<SkillKey>)> {
        let page = self.guide.page("character-creation");
        let Some(section) = page.root.find("skills") else {
            return Vec::new();
        };
        let mut rows_out = Vec::new();
        for rows in section.tables() {
            let Some(header) = rows.first() else { continue };
            if header.get(1).map(String::as_str) != Some("/ lvl") {
                continue;
            }
            for row in &rows[1..] {
                let Ok(points) = row[1].parse() else { continue };
                let skills = header
                    .iter()
                    .zip(row)
                    .filter(|(_, cell)| cell.contains('✓'))
                    .filter_map(|(column, _)| {
                        SKILL_COLUMNS
                            .iter()
                            .find(|(abbr, _)| abbr == column)
                            .map(|(_, name)| SkillKey::new(format!("skill:{}", slug(name))))
                    })
                    .collect();
                rows_out.push((row[0].clone(), points, skills));
            }
        }
        rows_out
    }

    fn races(&mut self) -> Vec<Race> {
        let page = self.guide.page("character-creation");
        let Some(section) = page.root.find("race") else {
            return Vec::new();
        };
        let mut races: Vec<Race> = Vec::new();
        for entry in section.children.iter().filter(|s| s.level == 4) {
            let (modifiers, feat_slots) = self.racial_table(page, entry);
            // "Aasimar (Angelkin)" as its own H4 is a subrace of the race before it.
            if let Some((race_name, sub)) = entry.name.split_once(" (")
                && let Some(race) = races
                    .iter_mut()
                    .find(|race| norm(&race.name) == norm(race_name))
            {
                race.subraces
                    .push(subrace(page, race, entry, sub, modifiers));
                continue;
            }
            let key = RaceKey::new(format!("race:{}", entry.anchor));
            self.catalog.races.insert(norm(&entry.name), key.clone());
            let mut race = Race {
                key,
                name: entry.name.clone(),
                summary: String::new(),
                guide: GuideRef::new(page.guide(&entry.anchor)),
                ability_modifiers: modifiers,
                feat_slots,
                subraces: Vec::new(),
            };
            for child in &entry.children {
                let (modifiers, _) = self.racial_table(page, child);
                let sub = child
                    .name
                    .split_once(" (")
                    .map_or(child.name.as_str(), |(_, sub)| sub);
                let made = subrace(page, &race, child, sub, modifiers);
                race.subraces.push(made);
            }
            races.push(race);
        }
        races
    }

    fn racial_table(
        &mut self,
        page: &Page,
        entry: &Section,
    ) -> (Vec<AbilityModifier>, Vec<FeatSlot>) {
        let mut modifiers = Vec::new();
        let mut slots = Vec::new();
        for rows in entry.tables() {
            for row in rows {
                let text = row.join(" ");
                if row.first().map(String::as_str) == Some("Ability Modifiers") {
                    match ability_modifiers(row.get(1).map_or("", String::as_str)) {
                        Some(parsed) => modifiers = parsed,
                        None => self.report.add(
                            Kind::Cell,
                            page.guide(&entry.anchor),
                            text.clone(),
                            "ability modifiers not understood",
                        ),
                    }
                }
                if text
                    .to_ascii_lowercase()
                    .contains("bonus feat at 1st level")
                {
                    slots.push(FeatSlot::General);
                }
            }
        }
        (modifiers, slots)
    }

    // Feats ---------------------------------------------------------------------------------------------------

    fn feats(&mut self) -> Vec<Feat> {
        let page = self.guide.page("feats");
        let mut feats = Vec::new();
        for section in page.root.walk() {
            let Some((_, category)) = FEAT_CATEGORIES
                .iter()
                .find(|(anchor, _)| *anchor == section.anchor)
            else {
                continue;
            };
            for entry in section.children.iter().filter(|s| s.level == 4) {
                feats.push(self.feat(page, entry, *category, Vec::new()));
            }
        }
        feats
    }

    fn feat(
        &mut self,
        page: &Page,
        entry: &Section,
        category: FeatCategory,
        extra: Vec<Prerequisite>,
    ) -> Feat {
        let key = FeatKey::new(format!("feat:{}", entry.anchor));
        self.catalog.feats.insert(norm(&entry.name), key.clone());
        self.feat_names.push((entry.name.clone(), key.clone()));
        self.prerequisites_of(page, entry, key.as_str(), Context::default(), extra);
        Feat {
            key,
            name: entry.name.clone(),
            category,
            summary: String::new(),
            guide: GuideRef::new(page.guide(&entry.anchor)),
            prerequisite: None,
        }
    }

    /// The Trickster's bonus feats: open to a Trickster from Perception II.
    fn trickster_feats(&mut self) -> Vec<Feat> {
        let page = self.guide.page("trickster");
        let Some(section) = page.root.find("trickster-bonus-feats") else {
            return Vec::new();
        };
        let gate = prose::trickster_feat_gate();
        let mut feats = Vec::new();
        for entry in section.children.iter().filter(|s| s.level == 4) {
            let category = prose::trickster_feat_category(&entry.name);
            feats.push(self.feat(page, entry, category, vec![gate.clone()]));
        }
        feats
    }

    /// Queues the prerequisites in `entry`'s paragraphs (and kineticist-style `Prerequisites` table rows).
    fn prerequisites_of(
        &mut self,
        page: &Page,
        entry: &Section,
        owner: &str,
        context: Context,
        extra: Vec<Prerequisite>,
    ) {
        let mut lines = Vec::new();
        for paragraph in entry.paragraphs() {
            if let Some(found) = grammar::prerequisite_lines(paragraph) {
                lines.extend(found);
                break;
            }
        }
        for rows in entry.tables() {
            for row in rows {
                if row.first().map(String::as_str) == Some("Prerequisites")
                    && let Some(line) = row.get(1)
                {
                    lines.push(line.clone());
                }
            }
        }
        if lines.is_empty() && extra.is_empty() {
            return;
        }
        self.pending.push(Pending {
            owner: owner.to_owned(),
            at: page.guide(&entry.anchor),
            conds: lines
                .into_iter()
                .map(|line| {
                    let cond = grammar::parse_line(&line);
                    (line, cond)
                })
                .collect(),
            context,
            kind: Kind::Prerequisite,
            extra,
        });
    }

    // Base classes and archetypes -------------------------------------------------------------------------------

    fn base_class(&mut self, slug_name: &str) -> Class {
        let guide = self.guide;
        let page = guide.page(slug_name);
        let root = page.root.children.first().expect("a class page has an h2");
        let class_key = ClassKey::new(format!("class:{slug_name}"));
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
        let mut class = Class {
            key: class_key.clone(),
            name: root.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(page.slug.clone()),
            skill_points: 0,
            class_skills: BTreeSet::new(),
            prerequisite: None,
            features: Vec::new(),
            selections: Vec::new(),
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
        self.class_alignment(page, root, class_key.as_str());

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
        class.levels = self.level_rows(page, &base_rows, None, base_spell_type, &base_scope);

        for (section, rows) in archetype_sections.iter().zip(&archetype_rows) {
            let prefix = format!("{slug_name}/{}", section.anchor);
            let archetype_key = ArchetypeKey::new(format!("archetype:{prefix}"));
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
            let levels = self.level_rows(page, rows, Some(&class.levels), spell_type, &scope);

            let (skill_points, class_skills) =
                match archetype_skills(&skill_rows, &root.name, &section.name) {
                    Some((points, skills)) => (
                        (points != class.skill_points).then_some(points),
                        (skills != class.class_skills).then_some(skills),
                    ),
                    None => (None, None),
                };
            self.archetype_requirements(page, section, archetype_key.as_str());
            class.archetypes.push(Archetype {
                key: archetype_key,
                name: section.name.clone(),
                summary: String::new(),
                guide: GuideRef::new(page.guide(&section.anchor)),
                prerequisite: None,
                skill_points,
                class_skills,
                features: Vec::new(),
                selections,
                levels,
            });
        }
        class
    }

    /// The tables directly in a class or archetype section, merged by level: numbers, spells and grants.
    fn level_table(&mut self, page: &Page, section: &Section, anchor: &str) -> Vec<Row> {
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

    /// Level rows from a parsed table. Numbers missing from an archetype's table come from its base class's rows.
    fn level_rows(
        &mut self,
        page: &Page,
        rows: &[Row],
        base: Option<&[ClassLevelRow]>,
        spell_type: Option<SpellType>,
        scope: &Scope<'g>,
    ) -> Vec<ClassLevelRow> {
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
            let base_row = base.and_then(|base| base.iter().find(|r| r.level == row.level));
            let number = |own: Option<u8>, from_base: Option<u8>| own.or(from_base);
            let (bab, fort, reflex, will) = (
                number(row.bab, base_row.map(|r| r.bab)),
                number(row.fort, base_row.map(|r| r.fort)),
                number(row.reflex, base_row.map(|r| r.reflex)),
                number(row.will, base_row.map(|r| r.will)),
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
            let mut level_row = ClassLevelRow {
                level: row.level,
                bab,
                fort,
                reflex,
                will,
                arcane_spell_level: 0,
                divine_spell_level: 0,
                advances_arcane: false,
                advances_divine: false,
                features: Vec::new(),
                feat_slots: Vec::new(),
                selections: Vec::new(),
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
                }
            }
            out.push(level_row);
        }
        out
    }

    /// What a table cell grants.
    fn grant(&mut self, page: &Page, scope: &Scope<'g>, name: &str, level: u8) -> Granted {
        let cell_norm = norm(name);
        if cell_norm.ends_with("-feat") || cell_norm.ends_with("-feats") {
            return Granted::Slots(self.bonus_slots(scope, name, level));
        }
        for form in number_forms(&cell_norm) {
            if let Some((_, key)) = scope
                .selections
                .iter()
                .find(|(names, _)| names.contains(&form))
            {
                return Granted::Selection(key.clone());
            }
        }
        if let Some(heading) = find_heading(scope, name) {
            let key = FeatureKey::new(format!(
                "feature:{}/{}",
                heading.prefix, heading.section.anchor
            ));
            self.define_feature(
                &key,
                heading.owner.clone(),
                &heading.section.name,
                heading.page.guide(&heading.section.anchor),
            );
            return Granted::Feature(key);
        }
        if !scope.path
            && let Some(feat) = self.feat_named(name)
        {
            // A feat the class grants outright: a one-feat list that ignores the feat's prerequisites.
            return Granted::Slots(vec![FeatSlot::OneOf {
                feats: [feat].into(),
                ignores_prerequisites: true,
            }]);
        }
        let key = FeatureKey::new(format!("feature:{}/{}", scope.prefix, slug(name)));
        let owner = scope
            .owner
            .clone()
            .expect("a scope with grants has an owner");
        let _ = page;
        self.define_feature(&key, owner, name, scope.guide.clone());
        Granted::Feature(key)
    }

    pub fn define_feature(
        &mut self,
        key: &FeatureKey,
        owner: FeatureOwner,
        name: &str,
        guide: String,
    ) {
        if self.features.contains_key(key) {
            return;
        }
        let feature = Feature {
            key: key.clone(),
            name: name.to_owned(),
            summary: String::new(),
            guide: GuideRef::new(guide),
        };
        let mut names = vec![norm(name)];
        // "Sneak Attack (Eldritch Scoundrel)" is also a "Sneak Attack".
        if let Some((base, _)) = name.split_once(" (") {
            names.push(norm(base));
        }
        for form in names {
            self.catalog
                .features
                .entry(form)
                .or_default()
                .insert(key.clone());
        }
        self.features.insert(key.clone(), (owner, feature));
    }

    pub fn feat_named(&self, name: &str) -> Option<FeatKey> {
        let wanted = norm(name);
        self.feat_names
            .iter()
            .find(|(feat, _)| norm(feat) == wanted)
            .map(|(_, key)| key.clone())
    }

    /// The bonus feat slots a "… Feat" cell grants.
    fn bonus_slots(&mut self, scope: &Scope<'g>, label: &str, level: u8) -> Vec<FeatSlot> {
        let label_norm = norm(label);
        match label_norm.as_str() {
            "bonus-combat-feat" | "combat-feat" => return vec![FeatSlot::CombatFeat],
            "teamwork-feat" | "bonus-teamwork-feat" => {
                return vec![FeatSlot::Category(FeatCategory::Teamwork)];
            }
            "mounted-combat-feat" => return vec![FeatSlot::Category(FeatCategory::MountedCombat)],
            _ => {}
        }
        if let Some(slots) = prose::special_bonus_slots(self, scope, label, level) {
            return slots;
        }
        // The list: a heading naming the label or "bonus feat(s)", nearest first.
        let label_forms = number_forms(&label_norm);
        let heading = scope
            .headings
            .iter()
            .find(|heading| label_forms.contains(&norm(&heading.section.name)))
            .or_else(|| {
                scope.headings.iter().find(|heading| {
                    let name = norm(&heading.section.name);
                    label_forms.iter().any(|form| name.contains(form.as_str()))
                })
            });
        let heading = heading.or_else(|| {
            scope.headings.iter().find(|heading| {
                let name = norm(&heading.section.name);
                name.contains("bonus-feat")
            })
        });
        let Some(heading) = heading else {
            self.report.add(
                Kind::BonusSlot,
                scope.guide.clone(),
                label,
                "no list found for this slot; stored as a general slot",
            );
            return vec![FeatSlot::General];
        };
        let at = heading.page.guide(&heading.section.anchor);
        let mut text = heading.section.text();
        for child in &heading.section.children {
            text.push(' ');
            text.push_str(&child.text());
        }
        let lower = text.to_ascii_lowercase();
        if lower.contains("bonus magic feat") {
            return vec![FeatSlot::Category(FeatCategory::Magic)];
        }
        if lower.contains("bonus combat feat") {
            return vec![FeatSlot::CombatFeat];
        }
        if lower.contains("any feat for which") {
            return vec![FeatSlot::General];
        }
        let tiers = self.feats_in(&text);
        if tiers.is_empty() {
            self.report.add(
                Kind::BonusSlot,
                at,
                label,
                "the list names no feats; stored as a general slot",
            );
            return vec![FeatSlot::General];
        }
        let feats: BTreeSet<FeatKey> = tiers
            .into_iter()
            .filter(|(tier, _)| *tier <= level)
            .map(|(_, feat)| feat)
            .collect();
        let ignores = if lower.contains("not need to meet")
            || lower.contains("without having to meet")
            || lower.contains("do not need to meet")
        {
            true
        } else if lower.contains("must meet")
            || lower.contains("must satisfy")
            || lower.contains("meets the prerequisites")
        {
            false
        } else {
            // D1: a class's own fixed bonus list ignores prerequisites unless the guide says otherwise.
            true
        };
        vec![FeatSlot::OneOf {
            feats,
            ignores_prerequisites: ignores,
        }]
    }

    /// Feats named in a prose list, each with the Class level its sentence opens at ("At 6th level, …").
    /// "(Improved) Great Fortitude" names both feats.
    pub fn feats_in(&self, text: &str) -> Vec<(u8, FeatKey)> {
        let mut found = Vec::new();
        let mut tier = 0u8;
        for sentence in text.split(". ") {
            if let Some(level) = sentence_level(sentence) {
                tier = level;
            }
            let expanded = expand_improved(sentence);
            let lower = expanded.to_ascii_lowercase();
            let mut names: Vec<&(String, FeatKey)> = self.feat_names.iter().collect();
            names.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
            let mut taken: Vec<(usize, usize)> = Vec::new();
            for (name, key) in names {
                let needle = name.to_ascii_lowercase();
                let mut from = 0;
                while let Some(at) = lower[from..].find(&needle) {
                    let start = from + at;
                    let end = start + needle.len();
                    from = end;
                    let bounded = !lower[..start].ends_with(|c: char| c.is_alphanumeric())
                        && !lower[end..].starts_with(|c: char| c.is_alphanumeric());
                    if !bounded || taken.iter().any(|&(s, e)| start < e && s < end) {
                        continue;
                    }
                    taken.push((start, end));
                    found.push((tier, key.clone()));
                }
            }
        }
        found
    }

    /// The feature selections of a class or archetype section.
    fn selections(
        &mut self,
        page: &Page,
        section: &Section,
        prefix: &str,
        granted: &BTreeSet<String>,
        class: &ClassKey,
        _archetype: Option<&ArchetypeKey>,
    ) -> Vec<FeatureSelection> {
        let mut out = Vec::new();
        for candidate in section
            .children
            .iter()
            .filter(|s| s.level == 4 && !s.children.is_empty())
        {
            let forms = number_forms(&norm(&candidate.name));
            let named = forms.iter().any(|form| granted.contains(form));
            if prose::is_merged_list(prefix, &candidate.anchor) {
                continue;
            }
            if !named || mostly_granted(candidate, granted) {
                continue;
            }
            out.push(self.selection(page, candidate, prefix, class, Vec::new()));
        }
        out
    }

    /// A Feature selection from a heading and its option headings.
    pub fn selection(
        &mut self,
        page: &Page,
        heading: &Section,
        prefix: &str,
        class: &ClassKey,
        extra: Vec<Prerequisite>,
    ) -> FeatureSelection {
        let key = FeatureSelectionKey::new(format!("selection:{prefix}/{}", heading.anchor));
        let mut selection = FeatureSelection {
            key: key.clone(),
            name: heading.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(page.guide(&heading.anchor)),
            options: Vec::new(),
        };
        self.add_options(page, heading, &mut selection, Some(class), extra);
        selection
    }

    /// Adds `heading`'s children as options of `selection`, each with `extra` ANDed into its prerequisites.
    pub fn add_options(
        &mut self,
        page: &Page,
        heading: &Section,
        selection: &mut FeatureSelection,
        class: Option<&ClassKey>,
        extra: Vec<Prerequisite>,
    ) {
        let mut all_options = BTreeSet::new();
        for child in &heading.children {
            let key = SelectionOptionKey::new(format!("{}/{}", selection.key, child.anchor));
            self.catalog
                .options
                .entry(norm(&child.name))
                .or_default()
                .insert(key.clone());
            all_options.insert(key.clone());
            let feat = self.option_feat(child);
            self.prerequisites_of(
                page,
                child,
                key.as_str(),
                Context {
                    own_class: class.cloned(),
                },
                extra.clone(),
            );
            selection.options.push(SelectionOption {
                key,
                name: child.name.clone(),
                summary: String::new(),
                guide: GuideRef::new(page.guide(&child.anchor)),
                feat,
                prerequisite: None,
            });
        }
        self.catalog
            .selections
            .entry(norm(&selection.name))
            .or_default()
            .extend(all_options);
    }

    /// The feat an option grants: "the feat of the same name", or "Gain X as a bonus feat".
    fn option_feat(&self, option: &Section) -> Option<FeatKey> {
        let text = option.text();
        let lower = text.to_ascii_lowercase();
        if lower.contains("feat of the same name") || lower.contains("identical to the feat") {
            return self.feat_named(&option.name);
        }
        let start = lower.find("gain ")? + "gain ".len();
        let end = lower[start..].find(" as a bonus feat")? + start;
        self.feat_named(&text[start..end])
    }

    fn class_alignment(&mut self, page: &Page, section: &Section, owner: &str) {
        for item in section.list_items() {
            if item.to_ascii_lowercase().starts_with("alignment must be") {
                self.queue_requirement(page, section, owner, item, None, Vec::new());
            }
        }
    }

    /// An archetype's race and alignment rules from its bullets.
    fn archetype_requirements(&mut self, page: &Page, section: &Section, owner: &str) {
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
                owner: owner.to_owned(),
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
        owner: &str,
        item: &str,
        context: Option<Context>,
        extra: Vec<Prerequisite>,
    ) {
        let parsed = grammar::parse_line(item);
        self.pending.push(Pending {
            owner: owner.to_owned(),
            at: page.guide(&section.anchor),
            conds: vec![(item.to_owned(), parsed)],
            context: context.unwrap_or_default(),
            kind: Kind::Requirement,
            extra,
        });
    }

    // Prestige classes -----------------------------------------------------------------------------------------

    fn prestige_classes(&mut self) -> Vec<Class> {
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
            self.catalog.classes.insert(
                norm(&section.name),
                ClassKey::new(format!("class:{}", section.anchor)),
            );
        }
        sections
            .into_iter()
            .map(|section| self.prestige_class(page, section))
            .collect()
    }

    fn prestige_class(&mut self, page: &'g Page, section: &'g Section) -> Class {
        let key = ClassKey::new(format!("class:{}", section.anchor));
        let rows = self.level_table(page, section, &section.anchor);
        let mut granted: BTreeSet<String> = BTreeSet::new();
        for row in &rows {
            for grant in &row.grants {
                granted.insert(norm(&grant.name));
            }
        }
        let selections = self.selections(page, section, &section.anchor, &granted, &key, None);
        let owner = FeatureOwner::Class(key.clone());
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
        let mut levels = self.level_rows(page, &rows, None, None, &scope);
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
        if !conds.is_empty() {
            self.pending.push(Pending {
                owner: key.as_str().to_owned(),
                at: page.guide(&section.anchor),
                conds,
                context: Context::default(),
                kind: Kind::Requirement,
                extra: Vec::new(),
            });
        }

        let (skill_points, class_skills) = self.prestige_skills(page, section);
        Class {
            key,
            name: section.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(page.guide(&section.anchor)),
            skill_points,
            class_skills,
            prerequisite: None,
            features: Vec::new(),
            selections,
            levels,
            archetypes: Vec::new(),
        }
    }

    fn prestige_skills(&mut self, page: &Page, section: &Section) -> (u8, BTreeSet<SkillKey>) {
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

    // Mythic abilities, feats and paths ----------------------------------------------------------------------

    fn mythic_abilities(&mut self) -> Vec<MythicAbility> {
        let page = self.guide.page("mythic-paths");
        let Some(section) = page.root.find("mythic-abilities") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for group in section.children.iter().filter(|s| s.level == 4) {
            let group_name = group.name.trim_end_matches(" Abilities").to_owned();
            for entry in &group.children {
                let key = MythicAbilityKey::new(format!("mythic-ability:{}", entry.anchor));
                self.catalog
                    .mythic_abilities
                    .insert(norm(&entry.name), key.clone());
                self.prerequisites_of(page, entry, key.as_str(), Context::default(), Vec::new());
                out.push(MythicAbility {
                    key,
                    name: entry.name.clone(),
                    group: group_name.clone(),
                    summary: String::new(),
                    guide: GuideRef::new(page.guide(&entry.anchor)),
                    prerequisite: None,
                });
            }
        }
        out
    }

    fn mythic_feats(&mut self) -> Vec<MythicFeat> {
        let page = self.guide.page("mythic-paths");
        let Some(section) = page.root.find("mythic-feats") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for group in section.children.iter().filter(|s| s.level == 4) {
            let group_name = group.name.trim_end_matches(" Feats").to_owned();
            for entry in &group.children {
                let key = MythicFeatKey::new(format!("mythic-feat:{}", entry.anchor));
                self.catalog
                    .mythic_feats
                    .insert(norm(&entry.name), key.clone());
                self.prerequisites_of(page, entry, key.as_str(), Context::default(), Vec::new());
                out.push(MythicFeat {
                    key,
                    name: entry.name.clone(),
                    group: group_name.clone(),
                    summary: String::new(),
                    guide: GuideRef::new(page.guide(&entry.anchor)),
                    prerequisite: None,
                });
            }
        }
        out
    }

    fn mythic_paths(&mut self) -> Vec<MythicPath> {
        let guide = self.guide;
        let mythic_page = guide.page("mythic-paths");
        let shared: Vec<Heading<'g>> = mythic_page
            .root
            .find("first-ascension")
            .map(|section| headings(mythic_page, section, "mythic", &FeatureOwner::Mythic))
            .unwrap_or_default();
        let mut paths = Vec::new();
        for slug_name in PATH_PAGES {
            let page = guide.page(slug_name);
            let Some(section) = page.root.children.first() else {
                continue;
            };
            paths.push(self.mythic_path(page, section, slug_name, page.slug.clone(), &shared));
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
            paths.push(self.mythic_path(other, section, anchor, other.guide(anchor), &shared));
        }
        paths
    }

    fn mythic_path(
        &mut self,
        page: &'g Page,
        section: &'g Section,
        slug_name: &str,
        guide: String,
        shared: &[Heading<'g>],
    ) -> MythicPath {
        let key = MythicPathKey::new(format!("path:{slug_name}"));
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
                let key =
                    FeatureSelectionKey::new(format!("selection:{slug_name}/{}", candidate.anchor));
                let mut selection = FeatureSelection {
                    key,
                    name: candidate.name.clone(),
                    summary: String::new(),
                    guide: GuideRef::new(page.guide(&candidate.anchor)),
                    options: Vec::new(),
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
        scope.headings.extend(shared.iter().cloned());
        let mut ranks = Vec::new();
        for row in &rows {
            let mut rank_row = MythicRankRow {
                rank: row.level,
                features: Vec::new(),
                feat_slots: Vec::new(),
                selections: Vec::new(),
            };
            for grant in &row.grants {
                match self.grant(page, &scope, &grant.name, row.level) {
                    Granted::Feature(feature) => rank_row.features.push(feature),
                    Granted::Selection(selection) => rank_row.selections.push(selection),
                    Granted::Slots(slots) => rank_row.feat_slots.extend(slots),
                }
            }
            ranks.push(rank_row);
        }
        if rows.is_empty() {
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
            ranks,
            switch: prose::path_switch(slug_name),
        }
    }

    // Placing features and resolving prerequisites ---------------------------------------------------------------

    fn place_features(
        &mut self,
        classes: &mut [Class],
        prestige: &mut [Class],
        paths: &mut [MythicPath],
    ) {
        let mut first_path_with: BTreeMap<FeatureKey, MythicPathKey> = BTreeMap::new();
        for path in paths.iter() {
            for row in &path.ranks {
                for feature in &row.features {
                    first_path_with
                        .entry(feature.clone())
                        .or_insert_with(|| path.key.clone());
                }
            }
        }
        for (key, (owner, feature)) in std::mem::take(&mut self.features) {
            let feature = feature.clone();
            match owner {
                FeatureOwner::Class(class) => {
                    if let Some(class) = classes
                        .iter_mut()
                        .chain(prestige.iter_mut())
                        .find(|c| c.key == class)
                    {
                        class.features.push(feature);
                    }
                }
                FeatureOwner::Archetype(class, archetype) => {
                    if let Some(archetype) = classes
                        .iter_mut()
                        .filter(|c| c.key == class)
                        .flat_map(|c| c.archetypes.iter_mut())
                        .find(|a| a.key == archetype)
                    {
                        archetype.features.push(feature);
                    }
                }
                FeatureOwner::Path(path) => {
                    if let Some(path) = paths.iter_mut().find(|p| p.key == path) {
                        path.features.push(feature);
                    }
                }
                FeatureOwner::Mythic => {
                    let Some(path_key) = first_path_with.get(&key) else {
                        continue;
                    };
                    if let Some(path) = paths.iter_mut().find(|p| &p.key == path_key) {
                        path.features.push(feature);
                    }
                }
            }
        }
    }

    /// Queues extra conditions for an owner (prose rules).
    pub fn add_prerequisite(&mut self, owner: &str, at: String, extra: Vec<Prerequisite>) {
        self.pending.push(Pending {
            owner: owner.to_owned(),
            at,
            conds: Vec::new(),
            context: Context::default(),
            kind: Kind::Prerequisite,
            extra,
        });
    }

    fn resolve_prerequisites(&mut self, data: &mut GameData) {
        let mut resolved: BTreeMap<String, Vec<Prerequisite>> = BTreeMap::new();
        for pending in std::mem::take(&mut self.pending) {
            let conditions = resolved.entry(pending.owner.clone()).or_default();
            conditions.extend(pending.extra);
            for (line, cond) in pending.conds {
                match cond {
                    Err(Unparsed(_)) => {
                        self.report
                            .add(pending.kind, pending.at.clone(), line, "doesn't parse");
                    }
                    Ok(cond) => match self.catalog.resolve(&cond, &pending.context) {
                        Ok(prerequisite) => conditions.push(prerequisite),
                        Err(why) => {
                            // An ANDed group keeps the children that resolve.
                            if let Cond::All(children) = &cond {
                                for child in children {
                                    match self.catalog.resolve(child, &pending.context) {
                                        Ok(prerequisite) => conditions.push(prerequisite),
                                        Err(why) => self.report.add(
                                            pending.kind,
                                            pending.at.clone(),
                                            line.clone(),
                                            why,
                                        ),
                                    }
                                }
                            } else {
                                self.report.add(pending.kind, pending.at.clone(), line, why);
                            }
                        }
                    },
                }
            }
        }
        let mut take = |key: &str| -> Option<Prerequisite> {
            let mut conditions = resolved.remove(key)?;
            conditions.sort();
            conditions.dedup();
            all(conditions)
        };
        for feat in &mut data.feats {
            feat.prerequisite = take(feat.key.as_str());
        }
        for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
            class.prerequisite = take(class.key.as_str());
            for selection in &mut class.selections {
                for option in &mut selection.options {
                    option.prerequisite = take(option.key.as_str());
                }
            }
            for archetype in &mut class.archetypes {
                archetype.prerequisite = take(archetype.key.as_str());
                for selection in &mut archetype.selections {
                    for option in &mut selection.options {
                        option.prerequisite = take(option.key.as_str());
                    }
                }
            }
        }
        for ability in &mut data.mythic_abilities {
            ability.prerequisite = take(ability.key.as_str());
        }
        for feat in &mut data.mythic_feats {
            feat.prerequisite = take(feat.key.as_str());
        }
        for path in &mut data.mythic_paths {
            for selection in &mut path.selections {
                for option in &mut selection.options {
                    option.prerequisite = take(option.key.as_str());
                }
            }
        }
        for owner in resolved.keys() {
            self.report.add(
                Kind::Prerequisite,
                owner.clone(),
                "",
                "prerequisites for something the data doesn't have",
            );
        }
    }
}

// Helpers -----------------------------------------------------------------------------------------------------

fn subrace(
    page: &Page,
    race: &Race,
    entry: &Section,
    sub: &str,
    modifiers: Vec<AbilityModifier>,
) -> Subrace {
    let race_slug = race.key.as_str().trim_start_matches("race:");
    let sub_slug = slug(sub.trim_end_matches(')'));
    let sub_slug = sub_slug
        .strip_prefix(&format!("{race_slug}-"))
        .unwrap_or(&sub_slug)
        .to_owned();
    Subrace {
        key: SubraceKey::new(format!("subrace:{race_slug}/{sub_slug}")),
        name: entry.name.clone(),
        summary: String::new(),
        guide: GuideRef::new(page.guide(&entry.anchor)),
        ability_modifiers: modifiers,
    }
}

/// "+2 Dexterity, Intelligence; -2 Constitution" or "+2 any".
fn ability_modifiers(text: &str) -> Option<Vec<AbilityModifier>> {
    let mut out = Vec::new();
    for group in text.split(';') {
        let group = group.trim().trim_end_matches('|').trim();
        if group.is_empty() {
            continue;
        }
        let (amount, abilities) = group.split_once(' ')?;
        let amount: i8 = amount.trim_start_matches('+').parse().ok()?;
        for name in abilities.split(',') {
            let name = name.trim().to_ascii_lowercase();
            if name == "any" {
                out.push(AbilityModifier::Chosen { amount });
                continue;
            }
            let ability = match name.as_str() {
                "strength" => Ability::Strength,
                "dexterity" => Ability::Dexterity,
                "constitution" => Ability::Constitution,
                "intelligence" => Ability::Intelligence,
                "wisdom" => Ability::Wisdom,
                "charisma" => Ability::Charisma,
                _ => return None,
            };
            out.push(AbilityModifier::Fixed { ability, amount });
        }
    }
    (!out.is_empty()).then_some(out)
}

fn is_level_table(rows: &[Vec<String>]) -> bool {
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

fn has_progression_table(section: &Section) -> bool {
    section.tables().any(|rows| {
        is_level_table(rows)
            && rows[0]
                .iter()
                .any(|cell| cell.starts_with("Special") || cell == "Feat" || cell == "Feats")
    })
}

/// Whether a table grants most of a heading's children by name: then they're features listed under a heading
/// (Ki Pool's Ki Strikes), not a list to pick from.
fn mostly_granted(heading: &Section, granted: &BTreeSet<String>) -> bool {
    let named = heading
        .children
        .iter()
        .filter(|child| granted.contains(&norm(&child.name)))
        .count();
    named * 2 >= heading.children.len()
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

/// The headings in `section` that a table can name: its own and its descendants', except archetypes (`h3`)
/// below a class's `h2`.
fn headings<'g>(
    page: &'g Page,
    section: &'g Section,
    prefix: &str,
    owner: &FeatureOwner,
) -> Vec<Heading<'g>> {
    let mut out = Vec::new();
    collect_headings(page, section, prefix, owner, &mut out, true);
    out
}

fn collect_headings<'g>(
    page: &'g Page,
    section: &'g Section,
    prefix: &str,
    owner: &FeatureOwner,
    out: &mut Vec<Heading<'g>>,
    top: bool,
) {
    if !top && !section.anchor.is_empty() {
        out.push(Heading {
            section,
            page,
            prefix: prefix.to_owned(),
            owner: owner.clone(),
        });
    }
    for child in &section.children {
        if section.level == 2 && child.level == 3 {
            continue;
        }
        collect_headings(page, child, prefix, owner, out, false);
    }
}

fn selection_names(selections: &[FeatureSelection]) -> Vec<(Vec<String>, FeatureSelectionKey)> {
    selections
        .iter()
        .map(|selection| (number_forms(&norm(&selection.name)), selection.key.clone()))
        .collect()
}

/// The headings without the options of `selections`: a cell naming an option isn't granting it.
fn without_options<'g>(
    headings: Vec<Heading<'g>>,
    selections: &[FeatureSelection],
) -> Vec<Heading<'g>> {
    let options: BTreeSet<String> = selections
        .iter()
        .flat_map(|selection| &selection.options)
        .map(|option| option.guide.0.clone())
        .collect();
    headings
        .into_iter()
        .filter(|heading| !options.contains(&heading.page.guide(&heading.section.anchor)))
        .collect()
}

/// The heading a cell names: by name or anchor, in its plural or singular form, disambiguated with the archetype,
/// as a summon, or without a trailing bonus ("Arcane Armor +2") or detail ("Fast Movement: +10'").
fn find_heading<'a, 'g>(scope: &'a Scope<'g>, name: &str) -> Option<&'a Heading<'g>> {
    let mut forms = Vec::new();
    let mut bases = vec![name.to_owned()];
    let trimmed = name
        .split(": ")
        .next()
        .unwrap_or(name)
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == '+' || c == ' ')
        .to_owned();
    if trimmed != name && !trimmed.is_empty() {
        bases.push(trimmed);
    }
    for base in &bases {
        let base_slug = norm(base);
        if let Some(anchor) = &scope.archetype_anchor {
            forms.push(format!("{base_slug}-{anchor}"));
        }
        forms.extend(number_forms(&base_slug));
        forms.push(format!("summon-{base_slug}"));
    }
    for form in &forms {
        if let Some(heading) = scope
            .headings
            .iter()
            .find(|heading| heading.section.anchor == *form || norm(&heading.section.name) == *form)
        {
            return Some(heading);
        }
    }
    None
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

/// "At 6th level, …" or "From 10th level, …" opens a tier.
fn sentence_level(sentence: &str) -> Option<u8> {
    let lower = sentence.trim().to_ascii_lowercase();
    let rest = lower
        .strip_prefix("at ")
        .or_else(|| lower.strip_prefix("from "))?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let after = &rest[digits.len()..];
    if !(after.starts_with("th level")
        || after.starts_with("st level")
        || after.starts_with("nd level")
        || after.starts_with("rd level"))
    {
        return None;
    }
    digits.parse().ok()
}

/// "(Improved) Great Fortitude" names "Great Fortitude" and "Improved Great Fortitude".
fn expand_improved(sentence: &str) -> String {
    let mut out = String::new();
    let mut rest = sentence;
    while let Some(at) = rest.find("(Improved) ") {
        out.push_str(&rest[..at]);
        rest = &rest[at + "(Improved) ".len()..];
        let end = rest.find([',', '.']).unwrap_or(rest.len());
        let name = &rest[..end];
        out.push_str(&format!("{name}, Improved {name}"));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Every block of a section of a given kind, for the prose rules.
pub fn section_text(section: &Section) -> String {
    let mut text = section.text();
    for block in &section.blocks {
        if let Block::Table(rows) = block {
            for row in rows {
                text.push(' ');
                text.push_str(&row.join(" "));
            }
        }
    }
    text
}
