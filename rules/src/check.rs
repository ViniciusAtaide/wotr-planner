//! [`GameData::check`]: mistakes in the game data that its types can't rule out.
//!
//! The seed refuses data with problems, so the database and the engine can rely on these rules. References to
//! missing keys are left to the database's foreign keys.
//!
//! Keys carry no type prefix, so two kinds may share one (a feat and a feature both `power-attack`): each kind is
//! its own table, and a [`DataAt`] names the kind as well as the key.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::engine::{BASE_LEVEL_CAP, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::game_data::{
    Advancement, Archetype, Class, ClassLevelRow, CountsToward, FeatList, FeatSlot, Feature,
    FeatureSelection, GameData, MythicPath, Save, SelectionOption, WeaponProficiency,
};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey, WeaponKey,
};
use crate::parameter::{Parameter, ParameterKind, Repeat};
use crate::prerequisite::Prerequisite;

/// The highest Character level on any path (Legend).
const MAX_CHARACTER_LEVEL: u8 = 40;
/// The highest Mythic rank.
const MAX_MYTHIC_RANK: u8 = 10;
/// The highest spell level.
const MAX_SPELL_LEVEL: u8 = 9;

/// One mistake in the game data.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataProblem {
    pub at: DataAt,
    pub issue: DataIssue,
}

/// Where a data problem is: an entity, by its kind and key, or the mythic ascension table.
///
/// Uniqueness is per kind, so this is also what a duplicate key is a duplicate of. Base and prestige classes are
/// one kind.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataAt {
    /// The mythic ascension table (`progression.ron`).
    Progression,
    Race(RaceKey),
    Subrace(SubraceKey),
    Skill(SkillKey),
    Feat(FeatKey),
    Class(ClassKey),
    Archetype(ArchetypeKey),
    Feature(FeatureKey),
    FeatureSelection(FeatureSelectionKey),
    SelectionOption(SelectionOptionKey),
    MythicAbility(MythicAbilityKey),
    MythicFeat(MythicFeatKey),
    MythicPath(MythicPathKey),
    FeatList(FeatListKey),
}

impl DataAt {
    /// The kind of entity, as a reader would name it: `feat`, `selection option`.
    pub fn kind(&self) -> &'static str {
        match self {
            DataAt::Progression => "progression",
            DataAt::Race(_) => "race",
            DataAt::Subrace(_) => "subrace",
            DataAt::Skill(_) => "skill",
            DataAt::Feat(_) => "feat",
            DataAt::Class(_) => "class",
            DataAt::Archetype(_) => "archetype",
            DataAt::Feature(_) => "feature",
            DataAt::FeatureSelection(_) => "selection",
            DataAt::SelectionOption(_) => "selection option",
            DataAt::MythicAbility(_) => "mythic ability",
            DataAt::MythicFeat(_) => "mythic feat",
            DataAt::MythicPath(_) => "mythic path",
            DataAt::FeatList(_) => "feat list",
        }
    }

    /// The entity's key; `None` for the mythic ascension table.
    pub fn key(&self) -> Option<&str> {
        Some(match self {
            DataAt::Progression => return None,
            DataAt::Race(key) => key.as_str(),
            DataAt::Subrace(key) => key.as_str(),
            DataAt::Skill(key) => key.as_str(),
            DataAt::Feat(key) => key.as_str(),
            DataAt::Class(key) => key.as_str(),
            DataAt::Archetype(key) => key.as_str(),
            DataAt::Feature(key) => key.as_str(),
            DataAt::FeatureSelection(key) => key.as_str(),
            DataAt::SelectionOption(key) => key.as_str(),
            DataAt::MythicAbility(key) => key.as_str(),
            DataAt::MythicFeat(key) => key.as_str(),
            DataAt::MythicPath(key) => key.as_str(),
            DataAt::FeatList(key) => key.as_str(),
        })
    }
}

/// `feat power-attack`, `selection option rogue/rogue-talents/fast-stealth`, `progression`.
impl fmt::Display for DataAt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.kind())?;
        match self.key() {
            Some(key) => write!(f, " {key}"),
            None => Ok(()),
        }
    }
}

/// What's wrong.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataIssue {
    /// Two entities of one kind share a key.
    DuplicateKey,
    /// A class or archetype table doesn't run 1, 2, … n with none missing (or has no rows, or more than 40).
    LevelRowsNotContiguous,
    ArchetypeOnPrestigeClass,
    /// A base class or archetype row that advances another class's spellcasting.
    AdvancesOnBaseClass {
        level: u8,
    },
    /// An `All`, `Any` or `AtLeast` without children.
    EmptyPrerequisiteGroup,
    /// An `AtLeast(n, ..)` with `n` of 0 or more than its children.
    AtLeastOutOfRange {
        n: u8,
        children: usize,
    },
    /// A selection option whose key doesn't start with its selection's key and a `/`.
    OptionOutsideItsSelection,
    /// A Character level outside 1..=40.
    LevelOutOfRange(u8),
    /// A Mythic rank outside 1..=10, or a kept rank at or after the switch rank.
    RankOutOfRange(u8),
    /// A spell level above 9.
    SpellLevelOutOfRange(u8),
    /// An archetype row at a level its base class doesn't have.
    ArchetypeRowOutsideClass(u8),
    /// Two archetype rows at one level.
    DuplicateArchetypeRow(u8),
    /// An archetype row that changes nothing: the base row's grants and no spell levels of its own (or the base
    /// row's). Leave the level out instead.
    ArchetypeRowSameAsBase(u8),
    /// An archetype row whose spell levels are the base row's anyway. Leave them out instead.
    SpellOverrideSameAsBase(u8),
    /// A save exception at a level the class doesn't have.
    SaveExceptionOutsideClass {
        level: u8,
        save: Save,
    },
    /// Two save exceptions for one cell.
    DuplicateSaveException {
        level: u8,
        save: Save,
    },
    /// A save exception the class's progression gives anyway.
    SaveExceptionSameAsFormula {
        level: u8,
        save: Save,
    },
    /// A feat slot ignoring prerequisites where its list isn't the grantor's own: on a race, or on a path rank
    /// with another owner's list. Class and archetype level rows may name any bonus list; a path's rank rows only
    /// the path's own (Gold Dragon's Dragon Feats).
    MisplacedBonusList,
    /// A feature, selection or feat list whose key doesn't start with its owner's key (the class, archetype or
    /// path that defines it) and a `/`.
    KeyOutsideItsOwner,
    /// A feat list of fewer than two feats: a lone feat is granted, not picked.
    FeatListTooShort,
    /// A list of every feat that also names feats of its own.
    EveryFeatListWithFeats,
    /// A path rank row before the path is chosen (MR 1-2) with a slot: choosing the path grants those rows'
    /// features only.
    SlotBeforePathChoice(u8),
    /// A Path switch open from a path that is itself a switch: no Build can be on it at the switch rank.
    SwitchFromASwitchPath(MythicPathKey),
    /// A feat list no slot names.
    UnusedFeatList,
    /// A selection that shares an option keyed under its own key: an own option is offered already, and
    /// anything else under its key isn't another selection's.
    SharesOwnOption(SelectionOptionKey),
    /// A feat or option taken once per value that takes none: an option, or an unparameterised feat.
    PerValueWithoutParameter,
    /// `Times(n)` with `n` below 2: once is the default.
    TimesOutOfRange(u8),
    /// An option's chosen feat that takes no value.
    ChosenFeatNotParameterised,
    /// A prerequisite naming a value the feat isn't taken for: a feat that takes none, a value of another kind,
    /// or "the same value" when the pick's value is of another kind.
    ValueOfWrongKind(FeatKey),
    /// "The same value" in a prerequisite whose pick takes no value: a class's, an archetype's, an unparameterised
    /// feat's or an option's without a chosen feat.
    SameValueWithoutValue,
    /// A class that advances one class, arcane or divine, whose rows don't advance both spell types: `Each` says
    /// the same.
    EitherWithOneSpellType,
    /// A class that counts as itself.
    CountsAsItself,
    /// A "counts as" toward a feature that the other class's rows (and its archetypes') never grant.
    CountsTowardUngrantedFeature(FeatureKey),
    /// A story condition under `Any` or `AtLeast`, where it would hold the group by itself: it never blocks.
    StoryNotRequired,
    /// Two rows of an option at one option level.
    DuplicateOptionRow(u8),
    /// A selection offering an option with rows by option level, but naming no classes whose levels make one.
    OptionRowsWithoutLevels(SelectionOptionKey),
    /// A pool selection (it offers the pools of the options held in another) with options of its own or shared.
    PoolSelectionWithOptions,
    /// An option row's chosen feat that doesn't take the kind of value the option's pick names (or the option's
    /// pick names none).
    RowChosenFeatOfOtherKind(FeatKey),
}

impl fmt::Display for DataProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.at)?;
        match &self.issue {
            DataIssue::DuplicateKey => f.write_str("the key is used twice"),
            DataIssue::LevelRowsNotContiguous => {
                f.write_str("the level rows don't run 1, 2, 3, … with none missing")
            }
            DataIssue::ArchetypeOnPrestigeClass => f.write_str("a prestige class has archetypes"),
            DataIssue::AdvancesOnBaseClass { level } => write!(
                f,
                "level {level} advances spellcasting, which only prestige classes do"
            ),
            DataIssue::EmptyPrerequisiteGroup => {
                f.write_str("a prerequisite group has no children")
            }
            DataIssue::AtLeastOutOfRange { n, children } => {
                write!(f, "\"at least {n}\" of {children} prerequisites")
            }
            DataIssue::OptionOutsideItsSelection => {
                f.write_str("the option's key doesn't start with its selection's key")
            }
            DataIssue::LevelOutOfRange(level) => {
                write!(f, "level {level} is outside 1..={MAX_CHARACTER_LEVEL}")
            }
            DataIssue::RankOutOfRange(rank) => write!(f, "Mythic rank {rank} is out of range"),
            DataIssue::SpellLevelOutOfRange(level) => {
                write!(f, "spell level {level} is above {MAX_SPELL_LEVEL}")
            }
            DataIssue::ArchetypeRowOutsideClass(level) => {
                write!(
                    f,
                    "a row at level {level}, which the base class doesn't have"
                )
            }
            DataIssue::DuplicateArchetypeRow(level) => write!(f, "two rows at level {level}"),
            DataIssue::ArchetypeRowSameAsBase(level) => write!(
                f,
                "the row at level {level} is the base class's; leave the level out"
            ),
            DataIssue::SpellOverrideSameAsBase(level) => write!(
                f,
                "the spell levels at level {level} are the base class's; leave them out"
            ),
            DataIssue::SaveExceptionOutsideClass { level, save } => write!(
                f,
                "a {save:?} exception at level {level}, which the class doesn't have"
            ),
            DataIssue::DuplicateSaveException { level, save } => {
                write!(f, "two {save:?} exceptions at level {level}")
            }
            DataIssue::SaveExceptionSameAsFormula { level, save } => write!(
                f,
                "the {save:?} exception at level {level} is what the progression gives anyway"
            ),
            DataIssue::MisplacedBonusList => f.write_str(
                "a feat slot that ignores prerequisites is granted outside a class's level rows or its own \
                 path's ranks",
            ),
            DataIssue::KeyOutsideItsOwner => {
                f.write_str("the key doesn't start with its owner's key")
            }
            DataIssue::FeatListTooShort => {
                f.write_str("a list of fewer than two feats; grant a lone feat instead")
            }
            DataIssue::UnusedFeatList => f.write_str("no feat slot names this list"),
            DataIssue::EveryFeatListWithFeats => {
                f.write_str("a list of every feat that names feats of its own")
            }
            DataIssue::SlotBeforePathChoice(rank) => write!(
                f,
                "the MR {rank} row grants a slot, but the path is chosen at MR {PATH_CHOICE_RANK}"
            ),
            DataIssue::SwitchFromASwitchPath(path) => {
                write!(f, "the switch is open from {path}, which is itself a switch")
            }
            DataIssue::PerValueWithoutParameter => {
                f.write_str("taken once per value, but it takes no value")
            }
            DataIssue::TimesOutOfRange(times) => {
                write!(f, "taken up to {times} times; a repeat is 2 or more")
            }
            DataIssue::ChosenFeatNotParameterised => {
                f.write_str("the chosen feat takes no value")
            }
            DataIssue::ValueOfWrongKind(feat) => {
                write!(f, "a prerequisite names a value {feat} isn't taken for")
            }
            DataIssue::SameValueWithoutValue => f.write_str(
                "\"the same value\" in a prerequisite whose pick takes no value",
            ),
            DataIssue::EitherWithOneSpellType => f.write_str(
                "advances one class, arcane or divine, but its rows advance only one spell type",
            ),
            DataIssue::CountsAsItself => f.write_str("the class counts as itself"),
            DataIssue::CountsTowardUngrantedFeature(feature) => write!(
                f,
                "counts toward {feature}, which the other class's rows never grant"
            ),
            DataIssue::StoryNotRequired => f.write_str(
                "a story condition inside \"any of\" or \"at least\", where it would hold the group by itself",
            ),
            DataIssue::DuplicateOptionRow(level) => write!(f, "two rows at option level {level}"),
            DataIssue::RowChosenFeatOfOtherKind(feat) => write!(
                f,
                "grants {feat} for the pick's value at an option level, but the pick names no value of its kind"
            ),
            DataIssue::OptionRowsWithoutLevels(option) => write!(
                f,
                "offers {option}, which has rows by option level, but names no classes whose levels count"
            ),
            DataIssue::PoolSelectionWithOptions => {
                f.write_str("a pool selection with options of its own or shared")
            }
            DataIssue::SharesOwnOption(option) => {
                write!(
                    f,
                    "shares {option}, which is keyed under the selection itself"
                )
            }
        }
    }
}

impl GameData {
    /// Every problem in the data, ordered by where it is. Empty when the data can be seeded.
    pub fn check(&self) -> Vec<DataProblem> {
        let mut check = Checker {
            lists: self
                .feat_lists()
                .map(|list| (list.key.clone(), list.ignores_prerequisites))
                .collect(),
            parameters: self
                .feats
                .iter()
                .map(|feat| (feat.key.clone(), feat.parameter))
                .collect(),
            weapons: self
                .weapons
                .iter()
                .map(|weapon| (weapon.key.clone(), weapon.proficiency))
                .collect(),
            ..Checker::default()
        };
        check.progression(self);
        for race in &self.races {
            let at = DataAt::Race(race.key.clone());
            check.key(&at);
            check.no_bonus_list(&at, &race.feat_slots, None);
            for subrace in &race.subraces {
                check.key(&DataAt::Subrace(subrace.key.clone()));
            }
        }
        for skill in &self.skills {
            check.key(&DataAt::Skill(skill.key.clone()));
        }
        for feat in &self.feats {
            let at = DataAt::Feat(feat.key.clone());
            check.key(&at);
            check.repeat(&at, feat.repeat, feat.parameter.is_some());
            check.prerequisite(&at, feat.prerequisite.as_ref(), feat.parameter);
        }
        for class in &self.classes {
            check.class(class, false);
        }
        for class in &self.prestige_classes {
            check.class(class, true);
        }
        for ability in &self.mythic_abilities {
            let at = DataAt::MythicAbility(ability.key.clone());
            check.key(&at);
            check.prerequisite(&at, ability.prerequisite.as_ref(), None);
        }
        for feat in &self.mythic_feats {
            let at = DataAt::MythicFeat(feat.key.clone());
            check.key(&at);
            check.prerequisite(&at, feat.prerequisite.as_ref(), None);
        }
        let switches: HashSet<&MythicPathKey> = self
            .mythic_paths
            .iter()
            .filter(|path| path.switch.is_some())
            .map(|path| &path.key)
            .collect();
        for path in &self.mythic_paths {
            check.path(path, &switches);
        }
        check.counts_toward_features(self);
        check.option_levels(self);
        for list in self.feat_lists() {
            if !check.named_lists.contains(&list.key) {
                check.problem(
                    &DataAt::FeatList(list.key.clone()),
                    DataIssue::UnusedFeatList,
                );
            }
        }
        check.problems.sort();
        check.problems
    }
}

#[derive(Default)]
struct Checker {
    /// Every key seen, with its kind: a key may repeat across kinds but not within one.
    keys: HashSet<DataAt>,
    /// Every feat list, with whether it ignores prerequisites.
    lists: HashMap<FeatListKey, bool>,
    /// The lists some slot names.
    named_lists: HashSet<FeatListKey>,
    /// Every feat, with what it's taken for.
    parameters: HashMap<FeatKey, Option<ParameterKind>>,
    /// Every weapon, with its proficiency list.
    weapons: HashMap<WeaponKey, Option<WeaponProficiency>>,
    problems: Vec<DataProblem>,
}

/// Whether `key` is `owner/…`.
fn owned_by(key: &str, owner: &str) -> bool {
    key.strip_prefix(owner)
        .is_some_and(|rest| rest.starts_with('/'))
}

impl Checker {
    fn problem(&mut self, at: &DataAt, issue: DataIssue) {
        self.problems.push(DataProblem {
            at: at.clone(),
            issue,
        });
    }

    fn key(&mut self, at: &DataAt) {
        if !self.keys.insert(at.clone()) {
            self.problem(at, DataIssue::DuplicateKey);
        }
    }

    /// Notes the lists the slots name.
    fn slots(&mut self, slots: &[FeatSlot]) {
        for slot in slots {
            if let FeatSlot::OneOf { list } = slot {
                self.named_lists.insert(list.clone());
            }
        }
    }

    /// Slots granted outside a class's level rows check prerequisites, unless the bonus list is the grantor's own
    /// (`owner`: a path's key; `None` for a race, which owns no list).
    fn no_bonus_list(&mut self, at: &DataAt, slots: &[FeatSlot], owner: Option<&str>) {
        self.slots(slots);
        let misplaced = slots.iter().any(|slot| match slot {
            FeatSlot::OneOf { list } => {
                self.lists.get(list).copied().unwrap_or(false)
                    && !owner.is_some_and(|owner| owned_by(list.as_str(), owner))
            }
            _ => false,
        });
        if misplaced {
            self.problem(at, DataIssue::MisplacedBonusList);
        }
    }

    /// What `owner` defines: features, selections (and their options) and feat lists, each keyed under it.
    fn owned(
        &mut self,
        owner: &str,
        features: &[Feature],
        selections: &[FeatureSelection],
        lists: &[FeatList],
    ) {
        for feature in features {
            let at = DataAt::Feature(feature.key.clone());
            self.key(&at);
            if !owned_by(feature.key.as_str(), owner) {
                self.problem(&at, DataIssue::KeyOutsideItsOwner);
            }
            if let Some(row) = &feature.option_row {
                self.level(&at, row.level);
            }
        }
        for selection in selections {
            if !owned_by(selection.key.as_str(), owner) {
                self.problem(
                    &DataAt::FeatureSelection(selection.key.clone()),
                    DataIssue::KeyOutsideItsOwner,
                );
            }
        }
        self.selections(selections);
        for list in lists {
            let at = DataAt::FeatList(list.key.clone());
            self.key(&at);
            if !owned_by(list.key.as_str(), owner) {
                self.problem(&at, DataIssue::KeyOutsideItsOwner);
            }
            // A list of one parameterised feat is a bonus feat of the player's choosing: the pick names the value.
            let one_choice = list.feats.len() == 1
                && list
                    .feats
                    .iter()
                    .all(|feat| self.parameters.get(feat).is_some_and(Option::is_some));
            if list.every_feat {
                if !list.feats.is_empty() {
                    self.problem(&at, DataIssue::EveryFeatListWithFeats);
                }
            } else if list.feats.len() < 2 && !one_choice {
                self.problem(&at, DataIssue::FeatListTooShort);
            }
        }
    }

    fn level(&mut self, at: &DataAt, level: u8) {
        if !(1..=MAX_CHARACTER_LEVEL).contains(&level) {
            self.problem(at, DataIssue::LevelOutOfRange(level));
        }
    }

    fn rank(&mut self, at: &DataAt, rank: u8) {
        if !(1..=MAX_MYTHIC_RANK).contains(&rank) {
            self.problem(at, DataIssue::RankOutOfRange(rank));
        }
    }

    fn progression(&mut self, data: &GameData) {
        let at = DataAt::Progression;
        let mut ranks = HashSet::new();
        for ascension in &data.progression.mythic_ascensions {
            self.rank(&at, ascension.rank);
            self.level(&at, ascension.character_level);
            if !ranks.insert(ascension.rank) {
                self.problem(&at, DataIssue::RankOutOfRange(ascension.rank));
            }
        }
    }

    /// How many times a feat or option repeats: once per value only with a value, a number of times from 2.
    fn repeat(&mut self, at: &DataAt, repeat: Repeat, takes_value: bool) {
        match repeat {
            Repeat::PerValue if !takes_value => {
                self.problem(at, DataIssue::PerValueWithoutParameter);
            }
            Repeat::Times(times) if times < 2 => {
                self.problem(at, DataIssue::TimesOutOfRange(times));
            }
            _ => {}
        }
    }

    /// Whether a value is one a feat of `kind` is taken for. An unknown weapon is the foreign keys' business.
    fn fits(&self, kind: ParameterKind, value: &Parameter) -> bool {
        use ParameterKind as K;
        match (kind, value) {
            (K::Skill, Parameter::Skill(_))
            | (K::Weapon, Parameter::Weapon(_))
            | (K::SpellSchool, Parameter::SpellSchool(_))
            | (K::Energy, Parameter::Energy(_))
            | (K::ArmourType, Parameter::ArmourType(_)) => true,
            (K::ExoticWeapon, Parameter::Weapon(weapon)) => self
                .weapons
                .get(weapon)
                .is_none_or(|proficiency| *proficiency == Some(WeaponProficiency::Exotic)),
            _ => false,
        }
    }

    /// A prerequisite's groups and values. `takes` is what the owner's pick names (`None`: nothing), which "the
    /// same value" compares with.
    fn prerequisite(
        &mut self,
        at: &DataAt,
        prerequisite: Option<&Prerequisite>,
        takes: Option<ParameterKind>,
    ) {
        let Some(prerequisite) = prerequisite else {
            return;
        };
        // A story condition always holds, so it may be required but not be one choice among several.
        if let Prerequisite::Any(children) | Prerequisite::AtLeast(_, children) = prerequisite
            && children
                .iter()
                .any(|child| matches!(child, Prerequisite::Story(_)))
        {
            self.problem(at, DataIssue::StoryNotRequired);
        }
        let children = match prerequisite {
            Prerequisite::All(children) | Prerequisite::Any(children) => children,
            Prerequisite::AtLeast(n, children) => {
                if !children.is_empty() && (*n == 0 || usize::from(*n) > children.len()) {
                    self.problem(
                        at,
                        DataIssue::AtLeastOutOfRange {
                            n: *n,
                            children: children.len(),
                        },
                    );
                }
                children
            }
            Prerequisite::FeatFor { feat, value } => {
                // An unknown feat is the foreign keys' business.
                if let Some(kind) = self.parameters.get(feat).copied()
                    && !kind.is_some_and(|kind| self.fits(kind, value))
                {
                    self.problem(at, DataIssue::ValueOfWrongKind(feat.clone()));
                }
                return;
            }
            Prerequisite::FeatForSameValue(feat) => {
                let Some(takes) = takes else {
                    self.problem(at, DataIssue::SameValueWithoutValue);
                    return;
                };
                if let Some(kind) = self.parameters.get(feat).copied()
                    && !kind.is_some_and(|kind| same_values(kind, takes))
                {
                    self.problem(at, DataIssue::ValueOfWrongKind(feat.clone()));
                }
                return;
            }
            _ => return,
        };
        if children.is_empty() {
            self.problem(at, DataIssue::EmptyPrerequisiteGroup);
        }
        for child in children {
            self.prerequisite(at, Some(child), takes);
        }
    }

    fn selections(&mut self, selections: &[FeatureSelection]) {
        for selection in selections {
            let selection_at = DataAt::FeatureSelection(selection.key.clone());
            self.key(&selection_at);
            for shared in &selection.shared_options {
                if owned_by(shared.as_str(), selection.key.as_str()) {
                    self.problem(&selection_at, DataIssue::SharesOwnOption(shared.clone()));
                }
            }
            for option in &selection.options {
                let at = DataAt::SelectionOption(option.key.clone());
                self.key(&at);
                // `rogue/rogue-talents/fast-stealth` extends `rogue/rogue-talents`.
                if !owned_by(option.key.as_str(), selection.key.as_str()) {
                    self.problem(&at, DataIssue::OptionOutsideItsSelection);
                }
                // An option takes a value only for its chosen feat, which must take one.
                let takes = option.chosen_feat.as_ref().and_then(|feat| {
                    let kind = self.parameters.get(feat).copied();
                    if kind == Some(None) {
                        self.problem(&at, DataIssue::ChosenFeatNotParameterised);
                    }
                    kind.flatten()
                });
                self.repeat(&at, option.repeat, false);
                self.prerequisite(&at, option.prerequisite.as_ref(), takes);
                let mut levels = HashSet::new();
                for row in &option.levels {
                    self.level(&at, row.level);
                    if !levels.insert(row.level) {
                        self.problem(&at, DataIssue::DuplicateOptionRow(row.level));
                    }
                    if let Some(list) = &row.feat_list {
                        self.named_lists.insert(list.clone());
                    }
                    self.slots(&row.feat_slots);
                    for feat in &row.chosen_feats {
                        let kind = self.parameters.get(feat).copied().flatten();
                        if !kind
                            .is_some_and(|kind| takes.is_some_and(|takes| same_values(kind, takes)))
                        {
                            self.problem(&at, DataIssue::RowChosenFeatOfOtherKind(feat.clone()));
                        }
                    }
                }
            }
            for level in &selection.skipped_levels {
                self.level(&selection_at, *level);
            }
            if selection.pool_of.is_some()
                && !(selection.options.is_empty() && selection.shared_options.is_empty())
            {
                self.problem(&selection_at, DataIssue::PoolSelectionWithOptions);
            }
        }
    }

    /// A selection offering an option with rows names the classes whose levels make its option level.
    fn option_levels<'a>(&mut self, data: &'a GameData) {
        let selections: Vec<&'a FeatureSelection> = data
            .classes
            .iter()
            .chain(&data.prestige_classes)
            .flat_map(|class| {
                class.selections.iter().chain(
                    class
                        .archetypes
                        .iter()
                        .flat_map(|archetype| &archetype.selections),
                )
            })
            .chain(data.mythic_paths.iter().flat_map(|path| &path.selections))
            .collect();
        let with_rows: HashSet<&SelectionOptionKey> = selections
            .iter()
            .flat_map(|selection| &selection.options)
            .filter(|option| !option.levels.is_empty())
            .map(|option| &option.key)
            .collect();
        // A pool (a mystery's revelations) is offered only through a pool selection, which then names the levels.
        let by_key: HashMap<&FeatureSelectionKey, &FeatureSelection> = selections
            .iter()
            .map(|selection| (&selection.key, *selection))
            .collect();
        let options: HashMap<&SelectionOptionKey, &SelectionOption> = selections
            .iter()
            .flat_map(|selection| &selection.options)
            .map(|option| (&option.key, option))
            .collect();
        let pools: HashSet<&FeatureSelectionKey> = options
            .values()
            .filter_map(|option| option.pool.as_ref())
            .collect();
        let offered = |selection: &'a FeatureSelection| -> Vec<&'a SelectionOptionKey> {
            match &selection.pool_of {
                None => selection.offered().into_iter().collect(),
                Some(of) => by_key
                    .get(of)
                    .map(|of| of.offered())
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|option| options.get(option)?.pool.as_ref())
                    .filter_map(|pool| by_key.get(pool))
                    .flat_map(|pool| pool.offered())
                    .collect(),
            }
        };
        for selection in selections.iter().copied() {
            if !selection.option_levels.is_empty() || pools.contains(&selection.key) {
                continue;
            }
            if let Some(option) = offered(selection)
                .into_iter()
                .find(|option| with_rows.contains(option))
            {
                self.problem(
                    &DataAt::FeatureSelection(selection.key.clone()),
                    DataIssue::OptionRowsWithoutLevels(option.clone()),
                );
            }
        }
    }

    fn rows(&mut self, at: &DataAt, rows: &[ClassLevelRow], prestige: bool) {
        let contiguous = !rows.is_empty()
            && rows.len() <= usize::from(MAX_CHARACTER_LEVEL)
            && rows
                .iter()
                .enumerate()
                .all(|(index, row)| usize::from(row.level) == index + 1);
        if !contiguous {
            self.problem(at, DataIssue::LevelRowsNotContiguous);
        }
        for row in rows {
            self.slots(&row.feat_slots);
            if !prestige && (row.advances_arcane || row.advances_divine) {
                self.problem(at, DataIssue::AdvancesOnBaseClass { level: row.level });
            }
            for spell_level in [row.arcane_spell_level, row.divine_spell_level] {
                if spell_level > MAX_SPELL_LEVEL {
                    self.problem(at, DataIssue::SpellLevelOutOfRange(spell_level));
                }
            }
        }
    }

    /// An archetype stores only the levels where it differs from its base class, each once.
    fn archetype_rows(&mut self, at: &DataAt, class: &Class, archetype: &Archetype) {
        let mut seen = HashSet::new();
        for row in &archetype.levels {
            self.slots(&row.feat_slots);
            if !seen.insert(row.level) {
                self.problem(at, DataIssue::DuplicateArchetypeRow(row.level));
            }
            let Some(base) = class.levels.iter().find(|base| base.level == row.level) else {
                self.problem(at, DataIssue::ArchetypeRowOutsideClass(row.level));
                continue;
            };
            if let Some(spells) = row.spell_levels {
                for spell_level in [spells.arcane, spells.divine] {
                    if spell_level > MAX_SPELL_LEVEL {
                        self.problem(at, DataIssue::SpellLevelOutOfRange(spell_level));
                    }
                }
            }
            let same_spells = row
                .spell_levels
                .is_none_or(|spells| spells == base.spell_levels());
            let same_grants = sorted(&row.features) == sorted(&base.features)
                && row.feats == base.feats
                && sorted(&row.feat_slots) == sorted(&base.feat_slots)
                && sorted(&row.selections) == sorted(&base.selections);
            if same_spells && same_grants {
                self.problem(at, DataIssue::ArchetypeRowSameAsBase(row.level));
            } else if row.spell_levels == Some(base.spell_levels()) {
                self.problem(at, DataIssue::SpellOverrideSameAsBase(row.level));
            }
        }
    }

    /// A save exception is a guide cell the progression doesn't give, at one of the class's levels, once.
    fn save_exceptions(&mut self, at: &DataAt, class: &Class, prestige: bool) {
        let mut seen = HashSet::new();
        for exception in &class.save_exceptions {
            let (level, save) = (exception.level, exception.save);
            if !seen.insert((level, save)) {
                self.problem(at, DataIssue::DuplicateSaveException { level, save });
            }
            if !class.levels.iter().any(|row| row.level == level) {
                self.problem(at, DataIssue::SaveExceptionOutsideClass { level, save });
            } else if class.save_progression(save).save(level, prestige) == exception.value {
                self.problem(at, DataIssue::SaveExceptionSameAsFormula { level, save });
            }
        }
    }

    fn class(&mut self, class: &Class, prestige: bool) {
        let at = DataAt::Class(class.key.clone());
        self.key(&at);
        self.prerequisite(&at, class.prerequisite.as_ref(), None);
        self.owned(
            class.key.as_str(),
            &class.features,
            &class.selections,
            &class.feat_lists,
        );
        self.rows(&at, &class.levels, prestige);
        self.save_exceptions(&at, class, prestige);
        let advances = |spell_type: fn(&ClassLevelRow) -> bool| class.levels.iter().any(spell_type);
        if class.advances == Advancement::Either
            && !(advances(|row| row.advances_arcane) && advances(|row| row.advances_divine))
        {
            self.problem(&at, DataIssue::EitherWithOneSpellType);
        }
        if class
            .counts_as
            .iter()
            .any(|counts| counts.class == class.key)
        {
            self.problem(&at, DataIssue::CountsAsItself);
        }
        if prestige && !class.archetypes.is_empty() {
            self.problem(&at, DataIssue::ArchetypeOnPrestigeClass);
        }
        for archetype in &class.archetypes {
            let at = DataAt::Archetype(archetype.key.clone());
            self.key(&at);
            self.prerequisite(&at, archetype.prerequisite.as_ref(), None);
            self.owned(
                archetype.key.as_str(),
                &archetype.features,
                &archetype.selections,
                &archetype.feat_lists,
            );
            self.archetype_rows(&at, class, archetype);
        }
    }

    /// A class's levels can count toward another class's feature only when that class's rows (or its
    /// archetypes') grant it. An unknown class is the foreign keys' business.
    fn counts_toward_features(&mut self, data: &GameData) {
        let classes = || data.classes.iter().chain(&data.prestige_classes);
        for class in classes() {
            for counts in &class.counts_as {
                let CountsToward::Feature(feature) = &counts.toward else {
                    continue;
                };
                let Some(other) = classes().find(|other| other.key == counts.class) else {
                    continue;
                };
                let granted = other
                    .levels
                    .iter()
                    .any(|row| row.features.contains(feature))
                    || other.archetypes.iter().any(|archetype| {
                        archetype
                            .levels
                            .iter()
                            .any(|row| row.features.contains(feature))
                    });
                if !granted {
                    self.problem(
                        &DataAt::Class(class.key.clone()),
                        DataIssue::CountsTowardUngrantedFeature(feature.clone()),
                    );
                }
            }
        }
    }

    fn path(&mut self, path: &MythicPath, switches: &HashSet<&MythicPathKey>) {
        let at = DataAt::MythicPath(path.key.clone());
        self.key(&at);
        self.owned(
            path.key.as_str(),
            &path.features,
            &path.selections,
            &path.feat_lists,
        );
        let mut ranks = HashSet::new();
        for row in &path.ranks {
            self.rank(&at, row.rank);
            if !ranks.insert(row.rank) {
                self.problem(&at, DataIssue::RankOutOfRange(row.rank));
            }
            self.no_bonus_list(&at, &row.feat_slots, Some(path.key.as_str()));
            if row.rank < PATH_CHOICE_RANK
                && !(row.feat_slots.is_empty() && row.selections.is_empty())
            {
                self.problem(&at, DataIssue::SlotBeforePathChoice(row.rank));
            }
        }
        if let Some(switch) = &path.switch {
            for rank in [switch.kept_through_rank, switch.path_kept_through_rank] {
                if !(1..PATH_SWITCH_RANK).contains(&rank) {
                    self.problem(&at, DataIssue::RankOutOfRange(rank));
                }
            }
            for from in &switch.from {
                if switches.contains(from) {
                    self.problem(&at, DataIssue::SwitchFromASwitchPath(from.clone()));
                }
            }
            if let Some(cap) = switch.level_cap
                && !(BASE_LEVEL_CAP..=MAX_CHARACTER_LEVEL).contains(&cap)
            {
                self.problem(&at, DataIssue::LevelOutOfRange(cap));
            }
        }
    }
}

/// Whether two kinds take the same sort of value: a weapon is a weapon whether the feat takes any or only exotic
/// ones.
fn same_values(a: ParameterKind, b: ParameterKind) -> bool {
    let weapon = |kind| matches!(kind, ParameterKind::Weapon | ParameterKind::ExoticWeapon);
    a == b || (weapon(a) && weapon(b))
}

/// A multiset's items in order, for comparing two of them.
fn sorted<T: Ord>(items: &[T]) -> Vec<&T> {
    let mut items: Vec<&T> = items.iter().collect();
    items.sort();
    items
}
