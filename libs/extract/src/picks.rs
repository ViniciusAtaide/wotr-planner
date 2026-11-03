//! The picks and the facts that come with them, built where their owner is built.
//!
//! The generic table reader ([`crate::build`]) turns each cell into the feature its heading defines. A cell the game
//! makes a pick is a slot instead: the class stage asks [`Picks::pick`], for the table it reads, the cell, the
//! feature the cell names (or the guide selection it grants) and its level, whether the cell is a pick, of which
//! selection, which slots, or the held option's row ([`Pick::OptionRow`]). The selections those picks are of, with
//! their options, and the features and feat lists that come with them, are the class's or the archetype's from the
//! start ([`Picks::selections_of`], [`Picks::options_of`], [`Picks::features_of`], [`Picks::feat_lists_of`]), as are
//! the option levels and option rows of a selection the guide's headings define ([`Picks::option_levels_of`],
//! [`Picks::option_rows_of`]), a pick a row grants beside its cells ([`Picks::row_selections_of`]) and a feature
//! that is a daily choice ([`Picks::is_daily_choice`]). No feature stands for a pick at any point.
//!
//! A cell may also be feats granted outright, with or without its feature (a proficiency feature: [`Pick::Feats`],
//! [`Pick::FeatureAndFeats`]), feats granted for one value ([`Pick::FeatsFor`]) or nothing ([`Pick::Nothing`]), and a
//! row may grant feats, slots or a feature beside its cells ([`Picks::row_feats_of`], [`Picks::row_slots_of`],
//! [`Picks::row_features_of`]). A feature a table defines may carry facts beside its name ([`Picks::shape`]), an
//! option the guide's headings define a condition, a parameter or a repeat ([`Picks::option_conditions_of`],
//! [`Picks::option_parameter_of`], [`Picks::option_repeat_of`]), and a heading that lists options may be no selection
//! ([`Picks::is_selection`]).
//!
//! Ticket 30's option families are given whole ([`Picks::family_of`]): the class stage plans each of the class's
//! tables before reading its rows, and takes the family once every table is read
//! ([`crate::option_levels::Taking`]).
//!
//! The other stages ask the same providers for their owners' facts: a feat's or a mythic ability's prerequisite
//! stated in place of its line, what a line also accepts beside what it names ([`Picks::alternatives_of`]), or a line
//! the reader passes over ([`Picks::prerequisite_of`], [`Picks::lines_for`]),
//! an archetype's alignment rule or an option's exclusions ([`Picks::stated_for`]), the slot and features a mythic
//! ability gives and the pick a mythic feat gives ([`Picks::mythic_selections_of`], [`Picks::mythic_features_of`],
//! [`Picks::mythic_extra_pick_of`]), an archetype's skills or a prestige class's class skills where the tables don't
//! say ([`Picks::archetype_skills_of`], [`Picks::fix_class_skills`]), and a race's bonus feat list
//! ([`Picks::race_list_of`]).
//!
//! Each provider module reads its facts from the saved pages and asserts the guide's sentences they rest on when the
//! providers are read ([`Picks::read`]), before the classes are built. What can only be asserted on the finished data
//! (the rows in force, after the archetype rows are made sparse) is in [`check`], which changes nothing.

use rules::{
    AbilityBonus, ClassKey, DataAt, FeatKey, FeatList, FeatSlot, FeatValue, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, MythicAbilityKey, MythicExtraPick,
    MythicFeatKey, OptionLevelRow, ParameterKind, Prerequisite, Repeat, SelectionOption,
    SelectionOptionKey, Skill, SkillKey,
};
use std::collections::BTreeSet;

use crate::build::FeatureOwner;
use crate::guide::Guide;
use crate::option_levels::FeatNames;

/// What a cell is, in place of the feature it names.
#[derive(Clone, Debug)]
pub enum Pick {
    /// One slot of this selection.
    Selection(FeatureSelectionKey),
    /// These feat slots.
    Slots(Vec<FeatSlot>),
    /// A feature of the table's own owner (a class or archetype), defined here.
    Feature(Feature),
    /// The held option's row (a family's placeholder, such as the kineticist's "Defensive Wild Talent" at 2): nothing
    /// on the owner's own row.
    OptionRow,
    /// One slot of this selection, with the cell's feature still defined (and named, for prerequisite lines), though
    /// not on the row: a line elsewhere resolves to it. The Wind Whisperer's "Wind", which the shaman's Air Barrier,
    /// Vortex Spells and Wind Ward resolve their "Wind" line to, and the Divine Hunter's "Domain", which Impossible
    /// Domain's "Domain" line resolves to.
    SelectionKeepingFeature(FeatureSelectionKey),
    /// These feats, granted outright, in place of the cell's feature (a proficiency feature its feats cover in full).
    Feats(BTreeSet<FeatKey>),
    /// These feats granted outright and these granted for one value, in place of the cell's feature (the Mantis
    /// Zealot's Saber Fighting: Weapon Focus in the Sawtooth Saber).
    FeatsFor {
        feats: BTreeSet<FeatKey>,
        feats_for: BTreeSet<FeatValue>,
    },
    /// Nothing: what the cell says is held otherwise (a class skill the archetype's own list or its bloodline's row
    /// holds).
    Nothing,
    /// The cell's feature, and these feats granted outright beside it (a proficiency feature that says more than its
    /// feats: weapons no feat covers, bombs, casting in armour).
    FeatureAndFeats(BTreeSet<FeatKey>),
}

/// Whose rows a row fact is on.
#[derive(Clone, Debug)]
pub enum Rows {
    /// The table of this class or archetype (by its key).
    Of(String),
    /// Every archetype table of this class (by its key).
    EveryArchetypeOf(String),
}

impl Rows {
    fn holds(&self, owner: &FeatureOwner) -> bool {
        match (self, owner) {
            (Self::Of(table), owner) => table == owner.key(),
            (Self::EveryArchetypeOf(class), FeatureOwner::Archetype(of, _)) => class == of.as_str(),
            _ => false,
        }
    }
}

/// Which cells are a pick.
#[derive(Clone, Debug)]
pub struct Rule {
    /// The feature the generic reader makes of the cell, or the selection it grants (a cell naming a selection the
    /// guide's headings define: the warpriest's "Blessings").
    pub feature: FeatureKey,
    /// Only in this table (a class's or an archetype's key); `None`, in every table whose cell names the feature.
    pub table: Option<String>,
    /// Only the cell of this text; `None`, every cell naming the feature.
    pub cell: Option<String>,
    /// Only at these levels; `None`, at every level.
    pub levels: Option<Vec<u8>>,
    pub pick: Pick,
}

impl Rule {
    /// Every cell naming `feature`, at every level, is `pick`.
    pub fn every(feature: &str, pick: Pick) -> Self {
        Self {
            feature: FeatureKey::new(feature),
            table: None,
            cell: None,
            levels: None,
            pick,
        }
    }

    fn matches(&self, table: &str, cell: &str, feature: &FeatureKey, level: u8) -> bool {
        &self.feature == feature
            && self.table.as_deref().is_none_or(|t| t == table)
            && self.cell.as_deref().is_none_or(|c| c == cell)
            && self.levels.as_ref().is_none_or(|l| l.contains(&level))
    }
}

/// What the providers give the class stage.
#[derive(Default)]
pub struct Picks {
    rules: Vec<Rule>,
    /// Selections of a class or archetype (by its key) that its tables don't define.
    selections: Vec<(String, FeatureSelection)>,
    /// Options of a selection read from the guide, beside the ones its heading lists (by the selection's key).
    options: Vec<(FeatureSelectionKey, SelectionOption)>,
    /// Features of a class or archetype that no cell names.
    features: Vec<(String, Feature)>,
    /// Feat lists of a class or archetype that its slots name by key.
    feat_lists: Vec<(String, FeatList)>,
    /// Prerequisites of a feat or mythic pick stated in place of its "Prerequisites" line, with the line.
    prerequisites: Vec<(DataAt, String, Prerequisite)>,
    /// Prerequisite lines of an owner that the reader passes over (a misspelt "Prequisites:").
    lines: Vec<(DataAt, String)>,
    /// What an owner's one prerequisite line, naming any of several things, also accepts, with the line.
    alternatives: Vec<(DataAt, String, Vec<Prerequisite>)>,
    /// The slot a mythic ability gives when it's picked.
    mythic_selections: Vec<(MythicAbilityKey, FeatureSelectionKey)>,
    /// The features a mythic ability grants (Bloodline Ascendance: the held bloodline's row 20).
    mythic_features: Vec<(MythicAbilityKey, FeatureKey)>,
    /// The features a table defines that are a choice made each day, not a pick.
    daily_choices: BTreeSet<FeatureKey>,
    /// The pick a mythic feat gives when it's taken.
    mythic_extra_picks: Vec<(MythicFeatKey, MythicExtraPick)>,
    /// Prerequisites stated for an owner beside what its own lines name (an archetype's alignment rule).
    stated: Vec<(DataAt, Prerequisite)>,
    /// The classes whose levels level a selection the guide's headings define (by the selection's key).
    option_levels: Vec<(FeatureSelectionKey, BTreeSet<ClassKey>)>,
    /// The rows of an option the guide's headings define (by the option's key).
    option_rows: Vec<(SelectionOptionKey, Vec<OptionLevelRow>)>,
    /// Selections a row of a class's or archetype's table (by its key) grants beside its cells, by level.
    row_selections: Vec<(String, u8, FeatureSelectionKey)>,
    /// Feats a row grants outright beside its cells, by level (a class level's proficiencies no feature carries).
    row_feats: Vec<(Rows, u8, BTreeSet<FeatKey>)>,
    /// Provided selections whose name a prerequisite line doesn't take as holding one of their options.
    unnamed: BTreeSet<FeatureSelectionKey>,
    /// Ticket 30's option families, each with how its class and archetypes take it.
    families: Vec<crate::option_levels::Taking>,
    /// Facts of a feature a table defines beside its name and link.
    feature_facts: Vec<(FeatureKey, FeatureFact)>,
    /// An archetype's skill points and class skills where the skills table doesn't give them.
    archetype_skills: Vec<(String, ArchetypeSkills)>,
    /// A prestige class's class skills beside its characteristics' list: the ones it lacks and the ones it has.
    class_skill_fixes: Vec<(String, Vec<SkillKey>, Vec<SkillKey>)>,
    /// A race's bonus feat at 1st level, a pick of this list.
    race_lists: Vec<(String, FeatList)>,
    /// Conditions of an option the guide's headings define, ANDed with what its lines name.
    option_conditions: Vec<(SelectionOptionKey, Prerequisite)>,
    /// The parameter an option the guide's headings define is taken for.
    option_parameters: Vec<(SelectionOptionKey, ParameterKind)>,
    /// How often an option the guide's headings define can be taken, when not once.
    option_repeats: Vec<(SelectionOptionKey, Repeat)>,
    /// Headings that read as a selection but aren't one: a cell naming them is their feature.
    not_selections: BTreeSet<FeatureSelectionKey>,
    /// Features a row of a table (by its key) holds once at least, beside what its cells name, by level.
    row_features: Vec<(String, u8, FeatureKey)>,
    /// Feat slots a row of a table (by its key) grants beside its cells, by level.
    row_slots: Vec<(String, u8, FeatSlot)>,
}

/// An archetype's skill points and class skills, beside the skills table.
#[derive(Clone, Debug)]
pub enum ArchetypeSkills {
    /// Its own points and list.
    Own {
        points: u8,
        skills: BTreeSet<SkillKey>,
    },
    /// Its class's list and these skills.
    ClassAnd(BTreeSet<SkillKey>),
}

/// A fact of a feature a table defines, beside its name and link.
#[derive(Clone, Debug)]
pub enum FeatureFact {
    /// It raises an ability score (Dragon Disciple's "Strength +2").
    AbilityBonus(AbilityBonus),
    /// The character's BAB is at least this for prerequisites (Perfect Mind).
    BabFloor(u8),
    /// Every skill's rank is at least the Mythic rank for prerequisites (Perfect Mind).
    SkillRanksAtLeastMythicRank,
}

impl Picks {
    /// Every provider's picks, each asserting the guide's sentences it rests on. The option families come first:
    /// the wild talents' and infusions' prerequisites name the kineticist's elements.
    pub(crate) fn read(guide: &Guide, feats: &FeatNames, skills: &[Skill]) -> Self {
        let mut picks = Self::default();
        crate::option_families::provide(guide, feats, skills, &mut picks);
        crate::option_levels::provide(guide, feats, skills, &mut picks);
        crate::option_class_skills::provide(guide, &mut picks);
        crate::follow_ups::provide(guide, &mut picks);
        crate::talent_follow_ups::provide(guide, &mut picks);
        crate::option_follow_ups::provide(guide, &mut picks);
        crate::report_gaps::provide(guide, &mut picks);
        crate::archetype_alignments::provide(guide, &mut picks);
        crate::mythic_extra_picks::provide(guide, &mut picks);
        crate::mythic_ability_picks::provide(guide, &mut picks);
        crate::mercies_and_exploits::provide(guide, &mut picks);
        crate::exploits::provide(guide, &mut picks);
        crate::wild_talents::provide(guide, &mut picks);
        crate::ki_powers::provide(guide, &mut picks);
        crate::infusions::provide(guide, &mut picks);
        crate::borrowed_selections::provide(guide, &mut picks);
        crate::bonus_feat_choices::provide(guide, feats, &mut picks);
        crate::instinctual_rage_powers::provide(guide, feats, &mut picks);
        crate::second_curse::provide(guide, &mut picks);
        crate::proficiencies::provide(&mut picks);
        crate::class_rules::provide(&mut picks);
        picks
    }

    pub fn rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn selection(&mut self, owner: &str, selection: FeatureSelection) {
        self.selections.push((owner.to_owned(), selection));
    }

    pub fn option(&mut self, selection: &str, option: SelectionOption) {
        self.options
            .push((FeatureSelectionKey::new(selection), option));
    }

    pub fn feature(&mut self, owner: &str, feature: Feature) {
        self.features.push((owner.to_owned(), feature));
    }

    pub fn feat_list(&mut self, owner: &str, list: FeatList) {
        self.feat_lists.push((owner.to_owned(), list));
    }

    /// The guide's selection `selection` has rows by option level, levelled by `classes`' levels.
    pub fn option_levels(&mut self, selection: &str, classes: &[&str]) {
        self.option_levels.push((
            FeatureSelectionKey::new(selection),
            classes.iter().map(|class| ClassKey::new(*class)).collect(),
        ));
    }

    pub fn option_levels_of(&self, selection: &FeatureSelectionKey) -> Option<BTreeSet<ClassKey>> {
        self.option_levels
            .iter()
            .find(|(s, _)| s == selection)
            .map(|(_, classes)| classes.clone())
    }

    /// The guide's option `option` has `rows`.
    pub fn option_rows(&mut self, option: &str, rows: Vec<OptionLevelRow>) {
        self.option_rows
            .push((SelectionOptionKey::new(option), rows));
    }

    pub fn option_rows_of(&self, option: &SelectionOptionKey) -> Vec<OptionLevelRow> {
        self.option_rows
            .iter()
            .find(|(o, _)| o == option)
            .map(|(_, rows)| rows.clone())
            .unwrap_or_default()
    }

    /// The row at `level` of `table` (a class's or an archetype's key) grants `selection` beside its cells.
    pub fn row_selection(&mut self, table: &str, level: u8, selection: &str) {
        self.row_selections
            .push((table.to_owned(), level, FeatureSelectionKey::new(selection)));
    }

    pub fn row_selections_of<'p>(
        &'p self,
        table: &'p str,
        level: u8,
    ) -> impl Iterator<Item = &'p FeatureSelectionKey> {
        self.row_selections
            .iter()
            .filter(move |(t, l, _)| t == table && *l == level)
            .map(|(_, _, selection)| selection)
    }

    /// The row at `level` of `table` (a class's or an archetype's key) holds `feature` once at least, whether a cell
    /// names it there or not.
    pub fn row_feature(&mut self, table: &str, level: u8, feature: &str) {
        self.row_features
            .push((table.to_owned(), level, FeatureKey::new(feature)));
    }

    pub fn row_features_of<'p>(
        &'p self,
        table: &'p str,
        level: u8,
    ) -> impl Iterator<Item = &'p FeatureKey> {
        self.row_features
            .iter()
            .filter(move |(t, l, _)| t == table && *l == level)
            .map(|(_, _, feature)| feature)
    }

    /// The row at `level` of `table` grants `slot` beside its cells.
    pub fn row_slot(&mut self, table: &str, level: u8, slot: FeatSlot) {
        self.row_slots.push((table.to_owned(), level, slot));
    }

    pub fn row_slots_of<'p>(
        &'p self,
        table: &'p str,
        level: u8,
    ) -> impl Iterator<Item = &'p FeatSlot> {
        self.row_slots
            .iter()
            .filter(move |(t, l, _)| t == table && *l == level)
            .map(|(_, _, slot)| slot)
    }

    /// The rows `rows` at `level` grant `feats` outright beside their cells.
    pub fn row_feats(&mut self, rows: Rows, level: u8, feats: BTreeSet<FeatKey>) {
        self.row_feats.push((rows, level, feats));
    }

    /// The feats the row at `level` of `owner`'s table grants beside its cells.
    pub fn row_feats_of(&self, owner: &FeatureOwner, level: u8) -> BTreeSet<FeatKey> {
        self.row_feats
            .iter()
            .filter(|(rows, l, _)| *l == level && rows.holds(owner))
            .flat_map(|(_, _, feats)| feats.iter().cloned())
            .collect()
    }

    /// A line naming the provided selection `selection` doesn't mean holding one of its options: the domains', whose
    /// "Domain" lines (Impossible Domain's) name the Divine Hunter's "Domain" feature, as when the domains were built
    /// after the lines were resolved.
    pub fn unnamed(&mut self, selection: &str) {
        self.unnamed.insert(FeatureSelectionKey::new(selection));
    }

    /// Whether a line naming the provided selection `selection` means holding one of its options.
    pub fn is_named(&self, selection: &FeatureSelectionKey) -> bool {
        !self.unnamed.contains(selection)
    }

    /// One of ticket 30's option families, taken by its class's tables as `taking` says.
    pub(crate) fn family(&mut self, taking: crate::option_levels::Taking) {
        self.families.push(taking);
    }

    /// The option family the class `class`'s tables take, if any.
    pub(crate) fn family_of(&self, class: &str) -> Option<&crate::option_levels::Taking> {
        self.families
            .iter()
            .find(|taking| taking.takes.class == class)
    }

    /// `owner`'s prerequisite is `prerequisite`, in place of what its "Prerequisites" line, `line`, names.
    pub fn prerequisite(&mut self, owner: DataAt, line: &str, prerequisite: Prerequisite) {
        self.prerequisites
            .push((owner, line.to_owned(), prerequisite));
    }

    /// `owner`'s prerequisite line `line`, which the reader passes over.
    pub fn line(&mut self, owner: DataAt, line: &str) {
        self.lines.push((owner, line.to_owned()));
    }

    pub fn lines_for(&self, owner: &DataAt) -> Vec<String> {
        self.lines
            .iter()
            .filter(|(o, _)| o == owner)
            .map(|(_, line)| line.clone())
            .collect()
    }

    /// `owner`'s prerequisite line `line`, which resolves to any of several things, also accepts `alternatives`:
    /// what the guide's name means beside the features a table names (the Nature Bond's companion option for
    /// "Animal Companion").
    pub fn alternatives(&mut self, owner: DataAt, line: &str, alternatives: Vec<Prerequisite>) {
        self.alternatives
            .push((owner, line.to_owned(), alternatives));
    }

    /// What `owner`'s line also accepts, asserting its lines are the one the provider read.
    pub fn alternatives_of(&self, owner: &DataAt, lines: &[String]) -> Vec<Prerequisite> {
        let mut found = Vec::new();
        for (_, line, alternatives) in self.alternatives.iter().filter(|(o, ..)| o == owner) {
            assert_eq!(
                lines,
                std::slice::from_ref(line),
                "{owner}: its prerequisite line"
            );
            found.extend(alternatives.iter().cloned());
        }
        found
    }

    /// The mythic ability `ability` gives one slot of `selection`.
    pub fn mythic_selection(&mut self, ability: &str, selection: FeatureSelectionKey) {
        self.mythic_selections
            .push((MythicAbilityKey::new(ability), selection));
    }

    pub fn mythic_selections_of(&self, ability: &MythicAbilityKey) -> Vec<FeatureSelectionKey> {
        self.mythic_selections
            .iter()
            .filter(|(a, _)| a == ability)
            .map(|(_, selection)| selection.clone())
            .collect()
    }

    /// The mythic ability `ability` grants `feature`.
    pub fn mythic_feature(&mut self, ability: &str, feature: &str) {
        self.mythic_features
            .push((MythicAbilityKey::new(ability), FeatureKey::new(feature)));
    }

    pub fn mythic_features_of(&self, ability: &MythicAbilityKey) -> Vec<FeatureKey> {
        self.mythic_features
            .iter()
            .filter(|(a, _)| a == ability)
            .map(|(_, feature)| feature.clone())
            .collect()
    }

    /// The feature `feature`, which a table defines, is a choice made each day, not a pick.
    pub fn daily_choice(&mut self, feature: &str) {
        self.daily_choices.insert(FeatureKey::new(feature));
    }

    pub fn is_daily_choice(&self, feature: &FeatureKey) -> bool {
        self.daily_choices.contains(feature)
    }

    /// The archetype `archetype`'s skill points and class skills are `skills`.
    pub fn archetype_skills(&mut self, archetype: &str, skills: ArchetypeSkills) {
        self.archetype_skills.push((archetype.to_owned(), skills));
    }

    pub fn archetype_skills_of(&self, archetype: &str) -> Option<&ArchetypeSkills> {
        self.archetype_skills
            .iter()
            .find(|(a, _)| a == archetype)
            .map(|(_, skills)| skills)
    }

    /// The prestige class `class`'s class skills are its characteristics' list without `without` and with `with`.
    pub fn class_skills(&mut self, class: &str, without: &[&str], with: &[&str]) {
        let keys = |skills: &[&str]| skills.iter().map(|s| SkillKey::new(*s)).collect();
        self.class_skill_fixes
            .push((class.to_owned(), keys(without), keys(with)));
    }

    /// Applies the class skills stated for the prestige class `class`, asserting each changes its list.
    pub fn fix_class_skills(&self, class: &str, skills: &mut BTreeSet<SkillKey>) {
        for (_, without, with) in self.class_skill_fixes.iter().filter(|(c, ..)| c == class) {
            for skill in without {
                assert!(skills.remove(skill), "{class}: no {skill} to take off");
            }
            for skill in with {
                assert!(skills.insert(skill.clone()), "{class}: {skill} already");
            }
        }
    }

    /// The race `race`'s bonus feat at 1st level is a pick of `list`.
    pub fn race_list(&mut self, race: &str, list: FeatList) {
        self.race_lists.push((race.to_owned(), list));
    }

    pub fn race_list_of(&self, race: &str) -> Option<&FeatList> {
        self.race_lists
            .iter()
            .find(|(r, _)| r == race)
            .map(|(_, list)| list)
    }

    /// The option `option`, which the guide's headings define, needs `condition` beside what its lines name.
    pub fn option_condition(&mut self, option: &str, condition: Prerequisite) {
        self.option_conditions
            .push((SelectionOptionKey::new(option), condition));
    }

    pub fn option_conditions_of(&self, option: &SelectionOptionKey) -> Vec<Prerequisite> {
        self.option_conditions
            .iter()
            .filter(|(o, _)| o == option)
            .map(|(_, condition)| condition.clone())
            .collect()
    }

    /// The option `option`, which the guide's headings define, is taken for a value of `kind`.
    pub fn option_parameter(&mut self, option: &str, kind: ParameterKind) {
        self.option_parameters
            .push((SelectionOptionKey::new(option), kind));
    }

    pub fn option_parameter_of(&self, option: &SelectionOptionKey) -> Option<ParameterKind> {
        self.option_parameters
            .iter()
            .find(|(o, _)| o == option)
            .map(|(_, kind)| *kind)
    }

    /// The option `option`, which the guide's headings define, can be taken `repeat` times.
    pub fn option_repeat(&mut self, option: &str, repeat: Repeat) {
        self.option_repeats
            .push((SelectionOptionKey::new(option), repeat));
    }

    pub fn option_repeat_of(&self, option: &SelectionOptionKey) -> Repeat {
        self.option_repeats
            .iter()
            .find(|(o, _)| o == option)
            .map_or(Repeat::Once, |(_, repeat)| *repeat)
    }

    /// The heading `selection` lists options, but is no selection: a cell naming it is its feature.
    pub fn not_selection(&mut self, selection: &str) {
        self.not_selections
            .insert(FeatureSelectionKey::new(selection));
    }

    pub fn is_selection(&self, selection: &FeatureSelectionKey) -> bool {
        !self.not_selections.contains(selection)
    }

    /// The feature `feature`, which a table defines, has `fact`.
    pub fn feature_fact(&mut self, feature: &str, fact: FeatureFact) {
        self.feature_facts.push((FeatureKey::new(feature), fact));
    }

    /// A feature as a table defines it, with the facts the providers give it.
    pub fn shape(&self, feature: &mut Feature) {
        feature.daily_choice = self.is_daily_choice(&feature.key);
        for (_, fact) in self.feature_facts.iter().filter(|(f, _)| *f == feature.key) {
            match fact {
                FeatureFact::AbilityBonus(bonus) => feature.ability_bonuses.push(bonus.clone()),
                FeatureFact::BabFloor(floor) => feature.bab_floor = Some(*floor),
                FeatureFact::SkillRanksAtLeastMythicRank => {
                    feature.skill_ranks_at_least_mythic_rank = true;
                }
            }
        }
    }

    /// The mythic feat `feat` gives `pick` when it's taken.
    pub fn mythic_extra_pick(&mut self, feat: &str, pick: MythicExtraPick) {
        self.mythic_extra_picks
            .push((MythicFeatKey::new(feat), pick));
    }

    pub fn mythic_extra_pick_of(&self, feat: &MythicFeatKey) -> Option<MythicExtraPick> {
        self.mythic_extra_picks
            .iter()
            .find(|(f, _)| f == feat)
            .map(|(_, pick)| *pick)
    }

    /// `owner` needs `prerequisite`, beside what its own lines name.
    pub fn stated(&mut self, owner: DataAt, prerequisite: Prerequisite) {
        self.stated.push((owner, prerequisite));
    }

    /// The prerequisites stated for `owner`.
    pub fn stated_for(&self, owner: &DataAt) -> Vec<Prerequisite> {
        self.stated
            .iter()
            .filter(|(o, _)| o == owner)
            .map(|(_, prerequisite)| prerequisite.clone())
            .collect()
    }

    /// What a cell of `table` (a class's or an archetype's key) naming `feature` at `level` is, if it's a pick. A rule
    /// for the cell's own text comes before one for every cell naming the feature.
    pub fn pick(&self, table: &str, cell: &str, feature: &FeatureKey, level: u8) -> Option<&Pick> {
        let matching = || {
            self.rules
                .iter()
                .filter(|rule| rule.matches(table, cell, feature, level))
        };
        matching()
            .find(|rule| rule.cell.is_some())
            .or_else(|| matching().next())
            .map(|rule| &rule.pick)
    }

    /// The selections of the class or archetype `owner` that its tables don't define.
    pub fn selections_of<'p>(
        &'p self,
        owner: &'p str,
    ) -> impl Iterator<Item = &'p FeatureSelection> {
        self.selections
            .iter()
            .filter(move |(o, _)| o == owner)
            .map(|(_, selection)| selection)
    }

    /// The options a provider adds to the selection `key`.
    pub fn options_of<'p>(
        &'p self,
        key: &'p FeatureSelectionKey,
    ) -> impl Iterator<Item = &'p SelectionOption> {
        self.options
            .iter()
            .filter(move |(s, _)| s == key)
            .map(|(_, option)| option)
    }

    pub fn features_of<'p>(&'p self, owner: &'p str) -> impl Iterator<Item = &'p Feature> {
        self.features
            .iter()
            .filter(move |(o, _)| o == owner)
            .map(|(_, feature)| feature)
    }

    pub fn feat_lists_of<'p>(&'p self, owner: &'p str) -> impl Iterator<Item = &'p FeatList> {
        self.feat_lists
            .iter()
            .filter(move |(o, _)| o == owner)
            .map(|(_, list)| list)
    }

    /// The prerequisite stated for `owner` in place of its line, asserting the line is the one the provider read.
    pub fn prerequisite_of(&self, owner: &DataAt, lines: &[String]) -> Option<Prerequisite> {
        let (_, line, prerequisite) = self.prerequisites.iter().find(|(o, ..)| o == owner)?;
        assert_eq!(
            lines,
            std::slice::from_ref(line),
            "{owner}: its prerequisite line"
        );
        Some(prerequisite.clone())
    }
}

/// Asserts what the providers can only assert on the finished data: the rows in force hold each pick where the guide's
/// tables have its cell. Changes nothing.
pub fn check(guide: &Guide, data: &GameData) {
    crate::option_levels::check(data);
    crate::option_class_skills::check(data);
    crate::follow_ups::check(data);
    crate::option_follow_ups::check(data);
    crate::second_follow_ups::check(guide, data);
    crate::talent_follow_ups::check(data);
    crate::option_families::check(guide, data);
    crate::report_gaps::check(data);
    crate::archetype_alignments::check(guide, data);
    crate::mythic_extra_picks::check(guide, data);
    crate::mythic_ability_picks::check(guide, data);
    crate::mercies_and_exploits::check(guide, data);
    crate::exploits::check(guide, data);
    crate::wild_talents::check(guide, data);
    crate::ki_powers::check(guide, data);
    crate::infusions::check(guide, data);
    crate::borrowed_selections::check(guide, data);
    crate::bonus_feat_choices::check(guide, data);
    crate::instinctual_rage_powers::check(guide, data);
    crate::second_curse::check(guide, data);
    crate::proficiencies::check(data);
    crate::class_rules::check(data);
}
