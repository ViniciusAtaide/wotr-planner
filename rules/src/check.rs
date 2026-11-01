//! [`GameData::check`]: mistakes in the game data that its types can't rule out.
//!
//! The seed refuses data with problems, so the database and the engine can rely on these rules. References to
//! missing keys are left to the database's foreign keys.
//!
//! Keys carry no type prefix, so two kinds may share one (a feat and a feature both `power-attack`): each kind is
//! its own table, and a [`DataAt`] names the kind as well as the key.

use std::collections::HashSet;
use std::fmt;

use crate::engine::{BASE_LEVEL_CAP, PATH_SWITCH_RANK};
use crate::game_data::{
    Archetype, Class, ClassLevelRow, FeatSlot, FeatureSelection, GameData, MythicPath, Save,
};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, MythicAbilityKey,
    MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey, SubraceKey,
};
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
    /// A race or a mythic rank that grants a feat slot ignoring prerequisites. Only a class's (or archetype's)
    /// level rows can: the bonus list is the class's own.
    BonusListOutsideAClass,
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
            DataIssue::BonusListOutsideAClass => f.write_str(
                "a feat slot that ignores prerequisites is granted outside a class's level rows",
            ),
        }
    }
}

impl GameData {
    /// Every problem in the data, ordered by where it is. Empty when the data can be seeded.
    pub fn check(&self) -> Vec<DataProblem> {
        let mut check = Checker::default();
        check.progression(self);
        for race in &self.races {
            let at = DataAt::Race(race.key.clone());
            check.key(&at);
            check.no_bonus_list(&at, &race.feat_slots);
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
            check.prerequisite(&at, feat.prerequisite.as_ref());
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
            check.prerequisite(&at, ability.prerequisite.as_ref());
        }
        for feat in &self.mythic_feats {
            let at = DataAt::MythicFeat(feat.key.clone());
            check.key(&at);
            check.prerequisite(&at, feat.prerequisite.as_ref());
        }
        for path in &self.mythic_paths {
            check.path(path);
        }
        check.problems.sort();
        check.problems
    }
}

#[derive(Default)]
struct Checker {
    /// Every key seen, with its kind: a key may repeat across kinds but not within one.
    keys: HashSet<DataAt>,
    problems: Vec<DataProblem>,
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

    /// Slots granted outside a class's level rows check prerequisites: the bonus list is the class's own.
    fn no_bonus_list(&mut self, at: &DataAt, slots: &[FeatSlot]) {
        if slots.iter().any(FeatSlot::ignores_prerequisites) {
            self.problem(at, DataIssue::BonusListOutsideAClass);
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

    fn prerequisite(&mut self, at: &DataAt, prerequisite: Option<&Prerequisite>) {
        let Some(prerequisite) = prerequisite else {
            return;
        };
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
            _ => return,
        };
        if children.is_empty() {
            self.problem(at, DataIssue::EmptyPrerequisiteGroup);
        }
        for child in children {
            self.prerequisite(at, Some(child));
        }
    }

    fn selections(&mut self, selections: &[FeatureSelection]) {
        for selection in selections {
            self.key(&DataAt::FeatureSelection(selection.key.clone()));
            for option in &selection.options {
                let at = DataAt::SelectionOption(option.key.clone());
                self.key(&at);
                // `rogue/rogue-talents/fast-stealth` extends `rogue/rogue-talents`.
                let inside = option
                    .key
                    .as_str()
                    .strip_prefix(selection.key.as_str())
                    .is_some_and(|rest| rest.starts_with('/'));
                if !inside {
                    self.problem(&at, DataIssue::OptionOutsideItsSelection);
                }
                self.prerequisite(&at, option.prerequisite.as_ref());
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
        self.prerequisite(&at, class.prerequisite.as_ref());
        for feature in &class.features {
            self.key(&DataAt::Feature(feature.key.clone()));
        }
        self.selections(&class.selections);
        self.rows(&at, &class.levels, prestige);
        self.save_exceptions(&at, class, prestige);
        if prestige && !class.archetypes.is_empty() {
            self.problem(&at, DataIssue::ArchetypeOnPrestigeClass);
        }
        for archetype in &class.archetypes {
            let at = DataAt::Archetype(archetype.key.clone());
            self.key(&at);
            self.prerequisite(&at, archetype.prerequisite.as_ref());
            for feature in &archetype.features {
                self.key(&DataAt::Feature(feature.key.clone()));
            }
            self.selections(&archetype.selections);
            self.archetype_rows(&at, class, archetype);
        }
    }

    fn path(&mut self, path: &MythicPath) {
        let at = DataAt::MythicPath(path.key.clone());
        self.key(&at);
        for feature in &path.features {
            self.key(&DataAt::Feature(feature.key.clone()));
        }
        self.selections(&path.selections);
        let mut ranks = HashSet::new();
        for row in &path.ranks {
            self.rank(&at, row.rank);
            if !ranks.insert(row.rank) {
                self.problem(&at, DataIssue::RankOutOfRange(row.rank));
            }
            self.no_bonus_list(&at, &row.feat_slots);
        }
        if let Some(switch) = &path.switch {
            for &rank in &switch.kept_ranks {
                if !(1..PATH_SWITCH_RANK).contains(&rank) {
                    self.problem(&at, DataIssue::RankOutOfRange(rank));
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

/// A multiset's items in order, for comparing two of them.
fn sorted<T: Ord>(items: &[T]) -> Vec<&T> {
    let mut items: Vec<&T> = items.iter().collect();
    items.sort();
    items
}
