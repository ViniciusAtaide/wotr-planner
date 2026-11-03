//! The picks and the facts that come with them, built where their owner is built.
//!
//! The generic table reader ([`crate::build`]) turns each cell into the feature its heading defines. A cell the game
//! makes a pick is a slot instead: the class stage asks [`Picks::pick`], for the table it reads, the cell, the
//! feature the cell names and its level, whether the cell is a pick, of which selection, or which slots. The
//! selections those picks are of, with their options, and the features and feat lists that come with them, are the
//! class's or the archetype's from the start ([`Picks::selections_of`], [`Picks::options_of`],
//! [`Picks::features_of`], [`Picks::feat_lists_of`]). No feature stands for a pick at any point.
//!
//! The other stages ask the same providers for their owners' facts: a feat's or a mythic ability's prerequisite
//! stated in place of its line, or a line the reader passes over ([`Picks::prerequisite_of`], [`Picks::lines_for`]),
//! an archetype's alignment rule ([`Picks::stated_for`]), the slot a mythic ability gives and the pick a mythic feat
//! gives ([`Picks::mythic_selections_of`], [`Picks::mythic_extra_pick_of`]).
//!
//! Each provider module reads its facts from the saved pages and asserts the guide's sentences they rest on when the
//! providers are read ([`Picks::read`]), before the classes are built. What can only be asserted on the finished data
//! (the rows in force, after the archetype rows are made sparse) is in [`check`], which changes nothing.

use rules::{
    DataAt, FeatList, FeatSlot, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey,
    GameData, MythicAbilityKey, MythicExtraPick, MythicFeatKey, Prerequisite, SelectionOption,
    SelectionOptionKey,
};

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
}

/// Which cells are a pick.
#[derive(Clone, Debug)]
pub struct Rule {
    /// The feature the generic reader makes of the cell.
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
    /// The slot a mythic ability gives when it's picked.
    mythic_selections: Vec<(MythicAbilityKey, FeatureSelectionKey)>,
    /// The pick a mythic feat gives when it's taken.
    mythic_extra_picks: Vec<(MythicFeatKey, MythicExtraPick)>,
    /// Prerequisites stated for an owner beside what its own lines name (an archetype's alignment rule).
    stated: Vec<(DataAt, Prerequisite)>,
    /// Prerequisites of provided options that name what a later pass builds, set once it has ([`Picks::late`]).
    late: Vec<(SelectionOptionKey, Prerequisite)>,
}

impl Picks {
    /// Every provider's picks, each asserting the guide's sentences it rests on.
    pub(crate) fn read(guide: &Guide, feats: &FeatNames) -> Self {
        let mut picks = Self::default();
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
        picks
    }

    pub fn rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn selection(&mut self, owner: &str, selection: FeatureSelection) {
        self.selections.push((owner.to_owned(), selection));
    }

    /// A selection whose options' prerequisites name what a later pass builds (the kineticist's elements, built with
    /// the other option families on the finished rows): the class stage gives the options none, and [`Picks::late`]
    /// sets them once that pass has run.
    pub fn selection_naming_later_facts(&mut self, owner: &str, mut selection: FeatureSelection) {
        for option in &mut selection.options {
            if let Some(prerequisite) = option.prerequisite.take() {
                self.late.push((option.key.clone(), prerequisite));
            }
        }
        self.selection(owner, selection);
    }

    /// Sets the prerequisites [`Picks::selection_naming_later_facts`] held back, asserting each names only what the
    /// data has by now.
    pub fn late(&self, data: &mut GameData) {
        for (key, prerequisite) in &self.late {
            let option = data
                .classes
                .iter_mut()
                .chain(&mut data.prestige_classes)
                .flat_map(|class| {
                    class
                        .selections
                        .iter_mut()
                        .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.selections))
                })
                .flat_map(|selection| &mut selection.options)
                .find(|option| &option.key == key)
                .unwrap_or_else(|| panic!("no option {key}"));
            assert!(
                option.prerequisite.is_none(),
                "{key}: a prerequisite already"
            );
            option.prerequisite = Some(prerequisite.clone());
        }
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
}
