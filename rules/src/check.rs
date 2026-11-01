//! [`GameData::check`]: mistakes in the game data that its types can't rule out.
//!
//! The seed refuses data with problems, so the database and the engine can rely on these rules. References to
//! missing keys are left to the database's foreign keys.

use std::collections::HashSet;
use std::fmt;

use crate::engine::{BASE_LEVEL_CAP, PATH_SWITCH_RANK};
use crate::game_data::{Class, ClassLevelRow, FeatureSelection, GameData, MythicPath};
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
    /// The key of the entity at fault (`class:wizard`), or `progression` for the mythic ascension table.
    pub at: String,
    pub issue: DataIssue,
}

/// What's wrong.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataIssue {
    /// Two entities share a key.
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
        }
    }
}

impl GameData {
    /// Every problem in the data, ordered by where it is. Empty when the data can be seeded.
    pub fn check(&self) -> Vec<DataProblem> {
        let mut check = Checker::default();
        check.progression(self);
        for race in &self.races {
            check.key(race.key.as_str());
            for subrace in &race.subraces {
                check.key(subrace.key.as_str());
            }
        }
        for skill in &self.skills {
            check.key(skill.key.as_str());
        }
        for feat in &self.feats {
            check.key(feat.key.as_str());
            check.prerequisite(feat.key.as_str(), feat.prerequisite.as_ref());
        }
        for class in &self.classes {
            check.class(class, false);
        }
        for class in &self.prestige_classes {
            check.class(class, true);
        }
        for ability in &self.mythic_abilities {
            check.key(ability.key.as_str());
            check.prerequisite(ability.key.as_str(), ability.prerequisite.as_ref());
        }
        for feat in &self.mythic_feats {
            check.key(feat.key.as_str());
            check.prerequisite(feat.key.as_str(), feat.prerequisite.as_ref());
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
    keys: HashSet<String>,
    problems: Vec<DataProblem>,
}

impl Checker {
    fn problem(&mut self, at: &str, issue: DataIssue) {
        self.problems.push(DataProblem {
            at: at.to_owned(),
            issue,
        });
    }

    fn key(&mut self, key: &str) {
        if !self.keys.insert(key.to_owned()) {
            self.problem(key, DataIssue::DuplicateKey);
        }
    }

    fn level(&mut self, at: &str, level: u8) {
        if !(1..=MAX_CHARACTER_LEVEL).contains(&level) {
            self.problem(at, DataIssue::LevelOutOfRange(level));
        }
    }

    fn rank(&mut self, at: &str, rank: u8) {
        if !(1..=MAX_MYTHIC_RANK).contains(&rank) {
            self.problem(at, DataIssue::RankOutOfRange(rank));
        }
    }

    fn progression(&mut self, data: &GameData) {
        let mut ranks = HashSet::new();
        for ascension in &data.progression.mythic_ascensions {
            self.rank("progression", ascension.rank);
            self.level("progression", ascension.character_level);
            if !ranks.insert(ascension.rank) {
                self.problem("progression", DataIssue::RankOutOfRange(ascension.rank));
            }
        }
    }

    fn prerequisite(&mut self, at: &str, prerequisite: Option<&Prerequisite>) {
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
            self.key(selection.key.as_str());
            for option in &selection.options {
                let key = option.key.as_str();
                self.key(key);
                let inside = key
                    .strip_prefix(selection.key.as_str())
                    .is_some_and(|rest| rest.starts_with('/'));
                if !inside {
                    self.problem(key, DataIssue::OptionOutsideItsSelection);
                }
                self.prerequisite(key, option.prerequisite.as_ref());
            }
        }
    }

    fn rows(&mut self, at: &str, rows: &[ClassLevelRow], prestige: bool) {
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

    fn class(&mut self, class: &Class, prestige: bool) {
        let at = class.key.as_str();
        self.key(at);
        self.prerequisite(at, class.prerequisite.as_ref());
        for feature in &class.features {
            self.key(feature.key.as_str());
        }
        self.selections(&class.selections);
        self.rows(at, &class.levels, prestige);
        if prestige && !class.archetypes.is_empty() {
            self.problem(at, DataIssue::ArchetypeOnPrestigeClass);
        }
        for archetype in &class.archetypes {
            let at = archetype.key.as_str();
            self.key(at);
            self.prerequisite(at, archetype.prerequisite.as_ref());
            for feature in &archetype.features {
                self.key(feature.key.as_str());
            }
            self.selections(&archetype.selections);
            self.rows(at, &archetype.levels, false);
        }
    }

    fn path(&mut self, path: &MythicPath) {
        let at = path.key.as_str();
        self.key(at);
        for feature in &path.features {
            self.key(feature.key.as_str());
        }
        self.selections(&path.selections);
        let mut ranks = HashSet::new();
        for row in &path.ranks {
            self.rank(at, row.rank);
            if !ranks.insert(row.rank) {
                self.problem(at, DataIssue::RankOutOfRange(row.rank));
            }
        }
        if let Some(switch) = &path.switch {
            for &rank in &switch.kept_ranks {
                if !(1..PATH_SWITCH_RANK).contains(&rank) {
                    self.problem(at, DataIssue::RankOutOfRange(rank));
                }
            }
            if let Some(cap) = switch.level_cap
                && !(BASE_LEVEL_CAP..=MAX_CHARACTER_LEVEL).contains(&cap)
            {
                self.problem(at, DataIssue::LevelOutOfRange(cap));
            }
        }
    }
}
