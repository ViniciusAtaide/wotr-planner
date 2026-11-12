//! [`GameData::check`]: mistakes in the game data that its types can't rule out.
//!
//! The seed refuses data with problems, and the engine takes only data that passed ([`CheckedGameData`], which only
//! [`GameData::checked`] makes, the behaviour proof's `unchecked` feature aside), so the database and the engine can rely on these rules. Every key an entity
//! names is one of an entity of its kind ([`DataIssue::UnknownKey`]), and every Mythic rank a path names is one of the
//! progression's ([`DataIssue::UnknownRank`]); the database's foreign keys check the same again, as a backstop.
//!
//! Keys carry no type prefix, so two kinds may share one (a feat and a feature both `power-attack`): each kind is
//! its own table, and a [`DataAt`] names the kind as well as the key.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

use crate::bonus::{BonusAmount, BonusItem, BonusTarget, BonusType, SpellScope};
use crate::engine::{BASE_LEVEL_CAP, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::game_data::{
    Advancement, Archetype, Class, ClassLevelRow, CountsToward, DOMAIN_SELECTION, Feat, FeatList,
    FeatSlot, FeatValue, Feature, FeatureSelection, GameData, LevelFrom, MythicPath, Save,
    SelectionOption, SpellLevels, WeaponProficiency,
};
use crate::gear::GearTarget;
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, FeatListKey, FeatureKey,
    FeatureSelectionKey, GearKey, MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey,
    SelectionOptionKey, SkillKey, SpellKey, SpellListKey, SubraceKey, WeaponKey,
};
use crate::parameter::{Parameter, ParameterKind, Repeat};
use crate::prerequisite::Prerequisite;
use crate::spells::{
    MAX_CLASS_SPELL_LEVEL, MAX_MYTHIC_SPELL_LEVEL, SpellCaster, SpellList, SpellListOnly,
};

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
/// Uniqueness is per kind, so this is also what a duplicate key is a duplicate of, and in an
/// [`UnknownKey`](DataIssue::UnknownKey) it is the kind and the key named. Base and prestige classes are one kind.
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
    Background(BackgroundKey),
    Deity(DeityKey),
    Weapon(WeaponKey),
    Spell(SpellKey),
    SpellList(SpellListKey),
    Gear(GearKey),
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
            DataAt::Background(_) => "background",
            DataAt::Deity(_) => "deity",
            DataAt::Weapon(_) => "weapon",
            DataAt::Spell(_) => "spell",
            DataAt::SpellList(_) => "spell list",
            DataAt::Gear(_) => "gear item",
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
            DataAt::Background(key) => key.as_str(),
            DataAt::Deity(key) => key.as_str(),
            DataAt::Weapon(key) => key.as_str(),
            DataAt::Spell(key) => key.as_str(),
            DataAt::SpellList(key) => key.as_str(),
            DataAt::Gear(key) => key.as_str(),
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
    /// A key the entity names that no entity of that kind has: the kind and the missing key.
    UnknownKey(DataAt),
    /// A Mythic rank a path's rank row or its switch names that the progression doesn't have.
    UnknownRank(u8),
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
    /// An archetype's save progression that is its class's anyway. Leave it out instead.
    SaveProgressionSameAsClass(Save),
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
    /// A feat slot ignoring prerequisites where its list isn't the grantor's own: on a race or a path rank with
    /// another owner's list. Class and archetype level rows may name any bonus list; a race only its own (the
    /// half-elf's Adaptability), a path's rank rows only the path's own (Gold Dragon's Dragon Feats).
    MisplacedBonusList,
    /// A feature, selection or feat list whose key doesn't start with its owner's key (the class, archetype or
    /// path that defines it) and a `/`.
    KeyOutsideItsOwner,
    /// A feat list of fewer than two feats: a lone feat is granted, not picked.
    FeatListTooShort,
    /// A list of every feat that also names feats of its own.
    EveryFeatListWithFeats,
    /// A path rank row before the path is chosen (MR 1-2) with a slot: those rows are the first ascension's
    /// path-linked ability and its follow-ups, features only.
    SlotBeforePathChoice(u8),
    /// A Path switch open from a path that is itself a switch: no Build can be on it at the switch rank.
    SwitchFromASwitchPath(MythicPathKey),
    /// A feat list no slot names.
    UnusedFeatList,
    /// A selection that shares an option keyed under its own key: an own option is offered already, and
    /// anything else under its key isn't another selection's.
    SharesOwnOption(SelectionOptionKey),
    /// A feat taken once per value that takes none.
    PerValueWithoutParameter,
    /// An option taken once per value that takes no value to count: neither a chosen feat's (ticket 47: the Skill
    /// Focus talents, once per skill) nor one of its own (ticket 172: Finesse Training, once per weapon).
    PerValueOptionWithoutValue,
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
    /// A selection naming both classes whose levels make the option level and another source of it.
    LevelFromWithClasses,
    /// An option whose level counts beside the other options held (`level_also_from`) on a selection not levelled
    /// by the other options held.
    LevelAlsoFromWithoutOtherOptions,
    /// An option's arcana that none of its rows grants.
    ArcanaNotInRows(FeatureKey),
    /// A prerequisite that looks at the pick (`NoOtherOptionOfGroup`, `PickedThrough`) outside a selection
    /// option's.
    PickLeafOutsideOption,
    /// An option row's chosen feat that doesn't take the kind of value the option's pick names (or the option's
    /// pick names none).
    RowChosenFeatOfOtherKind(FeatKey),
    /// A prerequisite asking about the pick's value (`FeatIfValue`) with a value of a kind the pick doesn't take.
    IfValueOfWrongKind,
    /// An option with a chosen feat that also names what its pick takes: the chosen feat's kind says it.
    ParameterBesideChosenFeat,
    /// An option listing values its pick doesn't take: it takes none, or they're of another kind.
    OptionValueOfWrongKind,
    /// A prerequisite, an option's values or a feat granted for one value naming a metamagic, stat or element:
    /// only a pick names those (ticket 125), and the tables keep no value columns for them.
    PickOnlyValue,
    /// A class's or an archetype's prerequisite naming an alignment inside a group with other conditions. Its
    /// alignment requirement must be a condition of its own (the whole prerequisite, or one of a top-level `All`'s
    /// conditions), since an archetype's replaces its class's (ticket 63).
    AlignmentNotOnItsOwn,
    /// A deity allowing no alignment.
    DeityWithoutAlignment,
    /// A deity with no favoured weapon.
    DeityWithoutFavouredWeapon,
    /// A deity's domain, a blessing's domain or a `DeityOffers` naming an option that isn't one of the domain
    /// selection's ([`DOMAIN_SELECTION`](crate::DOMAIN_SELECTION)).
    NotADomain(SelectionOptionKey),
    /// A deity-limited selection offering an option that is no domain and names none, which no deity could offer.
    DeityLimitWithoutDomain(SelectionOptionKey),
    /// A feature granting the deity's favoured weapons that no class or archetype row grants: it would never apply.
    FavouredWeaponsNotGranted,
    /// A weapon's stats no weapon has: dice other than 1–4 dice of 2–12 sides, a threat range starting below 15 or
    /// above 20, or a multiplier outside ×2–×4.
    WeaponStatsOutOfRange,
    /// A ranged weapon on the Weapon Finesse list, which is for melee weapons.
    FinesseOnRangedWeapon,
    /// A spell list entry above the list's highest level: 9 on a class's or an archetype's list, 10 in a Mythic
    /// spellbook.
    SpellLevelAboveList {
        level: u8,
        max: u8,
    },
    /// A spell on no list: its block sits under a list that has it, so every spell is on one.
    SpellOnNoList,
    /// A list entry for an archetype of no class that casts from the list, or for a story branch on a list that
    /// isn't a path's.
    OnlyOutsideTheList(SpellKey),
    /// A class or archetype whose rows give spell levels, with no casting ability of its own or its class's.
    NoCastingAbility,
    /// A casting ability on a class that casts no spells of its own (none of its or its archetypes' rows give spell
    /// levels; a prestige class only advances another's).
    CastingAbilityWithoutSpells,
    /// A bonus scoped to the value its item is held for ([`SpellScope::Value`](crate::SpellScope::Value), Expanded
    /// Arsenal's amount) on an item that isn't a feat or mythic feat taken for a school (or, for a scope alone, an
    /// energy type); or Expanded Arsenal's amount without that scope or on another target.
    BonusValueWithoutParameter,
    /// A bonus for what its item is held for ([`BonusTarget::Held`](crate::BonusTarget::Held)) on an item that isn't
    /// a feat or mythic feat taken for a skill or a stat.
    HeldBonusWithoutSkillOrStat,
    /// A bonus that rises with 10 ranks ([`BonusAmount::TenRanks`](crate::BonusAmount::TenRanks)) on a target that
    /// isn't one skill (every skill's included: the walk works out a bonus's amount once, for the skill it's for).
    TenRanksWithoutSkill,
    /// A second `interactions.ron` note on the same item.
    InteractionsTwice,
    /// A gear item with no bonus.
    GearWithoutBonus,
    /// A gear bonus of 0, or a negative one that isn't untyped (every penalty is).
    GearAmountInvalid,
    /// A gear item with two bonuses of one target and type.
    GearBonusTwice,
    /// A gear item named like another.
    GearNameTwice,
    /// A gear bonus on a spell DC of the value its item is held for: an item is held for none.
    GearSpellDcOfValue,
    /// A gear other-effects line that is empty or appears twice.
    GearEffectInvalid,
}

impl fmt::Display for DataProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.at)?;
        match &self.issue {
            DataIssue::DuplicateKey => f.write_str("the key is used twice"),
            DataIssue::UnknownKey(missing) => write!(f, "names {missing}, which the data doesn't have"),
            DataIssue::UnknownRank(rank) => write!(f, "names MR {rank}, which the progression doesn't have"),
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
            DataIssue::SaveProgressionSameAsClass(save) => write!(
                f,
                "the {save:?} progression is the base class's; leave it out"
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
                "a feat slot that ignores prerequisites is granted outside a class's level rows from a list \
                 that isn't the grantor's own",
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
                "the MR {rank} row grants a slot, but the rows before MR {PATH_CHOICE_RANK} are the first ascension's features"
            ),
            DataIssue::SwitchFromASwitchPath(path) => {
                write!(f, "the switch is open from {path}, which is itself a switch")
            }
            DataIssue::PerValueWithoutParameter => {
                f.write_str("taken once per value, but it takes no value")
            }
            DataIssue::PerValueOptionWithoutValue => {
                f.write_str("taken once per value, but it takes no value, of a chosen feat or its own")
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
            DataIssue::IfValueOfWrongKind => f.write_str(
                "a prerequisite asks about a value of a kind the pick doesn't take",
            ),
            DataIssue::ParameterBesideChosenFeat => f.write_str(
                "names what its pick takes beside a chosen feat, whose kind says it already",
            ),
            DataIssue::OptionValueOfWrongKind => {
                f.write_str("lists values of a kind its pick doesn't take")
            }
            DataIssue::PickOnlyValue => {
                f.write_str("names a metamagic, stat or element, which only a pick names")
            }
            DataIssue::AlignmentNotOnItsOwn => f.write_str(
                "an alignment sits in a group with other conditions; the alignment requirement must be a \
                 condition of its own",
            ),
            DataIssue::LevelFromWithClasses => f.write_str(
                "names classes whose levels make the option level and another source of it",
            ),
            DataIssue::LevelAlsoFromWithoutOtherOptions => f.write_str(
                "names an option whose level counts beside the other options held, but isn't levelled by them",
            ),
            DataIssue::ArcanaNotInRows(feature) => {
                write!(f, "its arcana {feature} isn't granted by any of its rows")
            }
            DataIssue::PickLeafOutsideOption => f.write_str(
                "a prerequisite looks at the option picked or its selection outside a selection option's",
            ),
            DataIssue::DeityWithoutAlignment => f.write_str("the deity allows no alignment"),
            DataIssue::DeityWithoutFavouredWeapon => f.write_str("the deity has no favoured weapon"),
            DataIssue::NotADomain(option) => write!(f, "names {option} as a domain, which it isn't"),
            DataIssue::DeityLimitWithoutDomain(option) => write!(
                f,
                "limits its options by the deity's domains, but offers {option}, which is no domain and names none"
            ),
            DataIssue::FavouredWeaponsNotGranted => f.write_str(
                "grants the deity's favoured weapons, but no class or archetype row grants it",
            ),
            DataIssue::WeaponStatsOutOfRange => f.write_str(
                "its stats are out of range: 1–4 dice of 2–12 sides, a threat range from 15–20 up, ×2–×4",
            ),
            DataIssue::FinesseOnRangedWeapon => {
                f.write_str("a ranged weapon can't be on the Weapon Finesse list")
            }
            DataIssue::SpellLevelAboveList { level, max } => {
                write!(f, "lists a spell at level {level}, above the list's highest, {max}")
            }
            DataIssue::SpellOnNoList => f.write_str("the spell is on no spell list"),
            DataIssue::OnlyOutsideTheList(spell) => write!(
                f,
                "limits {spell} to an archetype of no class that casts from the list, or to a story branch on a \
                 list that isn't a path's"
            ),
            DataIssue::NoCastingAbility => {
                f.write_str("its rows give spell levels, but it has no casting ability")
            }
            DataIssue::CastingAbilityWithoutSpells => {
                f.write_str("it has a casting ability but casts no spells of its own")
            }
            DataIssue::BonusValueWithoutParameter => f.write_str(
                "a bonus for the value its item is held for needs a feat or mythic feat taken for a school or an energy type",
            ),
            DataIssue::HeldBonusWithoutSkillOrStat => f.write_str(
                "a bonus for what its item is held for needs a feat or mythic feat taken for a skill or a stat",
            ),
            DataIssue::TenRanksWithoutSkill => {
                f.write_str("a bonus that rises with 10 ranks needs one skill as its target")
            }
            DataIssue::InteractionsTwice => f.write_str("it has two interactions notes"),
            DataIssue::GearWithoutBonus => f.write_str("the item has no bonus"),
            DataIssue::GearAmountInvalid => {
                f.write_str("a bonus is 0, or negative and not untyped")
            }
            DataIssue::GearBonusTwice => {
                f.write_str("two bonuses share a target and a type")
            }
            DataIssue::GearNameTwice => f.write_str("another item has the same name"),
            DataIssue::GearSpellDcOfValue => f.write_str(
                "a spell DC bonus is for the value an item is held for, which gear isn't",
            ),
            DataIssue::GearEffectInvalid => {
                f.write_str("an other-effects line is empty or appears twice")
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
            domains: self
                .selections()
                .filter(|selection| selection.key.as_str() == DOMAIN_SELECTION)
                .flat_map(|selection| &selection.options)
                .map(|option| option.key.clone())
                .collect(),
            ..Checker::default()
        };
        check.progression(self);
        for race in &self.races {
            let at = DataAt::Race(race.key.clone());
            check.key(&at);
            check.owned(race.key.as_str(), &[], &[], &race.feat_lists);
            check.no_bonus_list(&at, &race.feat_slots, Some(race.key.as_str()));
            for subrace in &race.subraces {
                let at = DataAt::Subrace(subrace.key.clone());
                check.key(&at);
                // A subrace's slots replace the race's, so they may name the race's own lists.
                check.no_bonus_list(&at, &subrace.feat_slots, Some(race.key.as_str()));
            }
        }
        for skill in &self.skills {
            check.key(&DataAt::Skill(skill.key.clone()));
        }
        for weapon in &self.weapons {
            let at = DataAt::Weapon(weapon.key.clone());
            check.key(&at);
            if let Some(stats) = &weapon.stats {
                if !(1..=4).contains(&stats.damage.count)
                    || !(2..=12).contains(&stats.damage.sides)
                    || !(15..=20).contains(&stats.threat)
                    || !(2..=4).contains(&stats.multiplier)
                {
                    check.problem(&at, DataIssue::WeaponStatsOutOfRange);
                }
                if stats.finesse && stats.ranged.is_some() {
                    check.problem(&at, DataIssue::FinesseOnRangedWeapon);
                }
            }
        }
        for feat in &self.feats {
            let at = DataAt::Feat(feat.key.clone());
            check.key(&at);
            check.names_each(&at, &feat.extra_pick_of, DataAt::FeatureSelection);
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
            check.grants(
                &at,
                &ability.features,
                &BTreeSet::new(),
                &ability.selections,
            );
            check.prerequisite(&at, ability.prerequisite.as_ref(), ability.parameter);
        }
        for feat in &self.mythic_feats {
            let at = DataAt::MythicFeat(feat.key.clone());
            check.key(&at);
            check.repeat(&at, feat.repeat, feat.parameter.is_some());
            check.prerequisite(&at, feat.prerequisite.as_ref(), feat.parameter);
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
        for background in &self.backgrounds {
            let at = DataAt::Background(background.key.clone());
            check.key(&at);
            check.names_each(&at, &background.class_skills, DataAt::Skill);
            check.names_each(&at, &background.feats, DataAt::Feat);
            check.feats_for(&at, &background.feats_for);
        }
        for deity in &self.deities {
            check.deity(deity);
        }
        for spell in &self.spells {
            check.key(&DataAt::Spell(spell.key.clone()));
        }
        for list in &self.spell_lists {
            check.spell_list(list);
        }
        let listed: HashSet<&SpellKey> = self
            .spell_lists
            .iter()
            .flat_map(|list| &list.entries)
            .map(|entry| &entry.spell)
            .collect();
        for spell in &self.spells {
            if !listed.contains(&spell.key) {
                check.problem(&DataAt::Spell(spell.key.clone()), DataIssue::SpellOnNoList);
            }
        }
        check.bonuses(self);
        check.interactions(self);
        check.gear(self);
        check.deity_limits(self);
        check.favoured_weapon_features(self);
        check.counts_toward_features(self);
        check.option_levels(self);
        check.unknown_keys(self);
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

    /// The data as [`CheckedGameData`] when [`check`](Self::check) finds no problem, otherwise every problem.
    pub fn checked(self) -> Result<CheckedGameData, Vec<DataProblem>> {
        let problems = self.check();
        if problems.is_empty() {
            Ok(CheckedGameData(self))
        } else {
            Err(problems)
        }
    }
}

/// Game data that [`GameData::check`] found no problem in, the only data [`Engine::new`](crate::Engine::new) takes:
/// the engine relies on the check's rules (every feat list or selection a slot names is one of the data's, every
/// option's pool is a selection). Only [`GameData::checked`] makes one (and, for the behaviour proof alone, the
/// `unchecked` feature's [`From`]). Outside this crate it lends the data only for reading: it derefs to
/// `&GameData`, with no `DerefMut` and no way to take the data back out. Inside it, [`Engine::new`](crate::Engine::new)
/// takes the data owned (`into_data`) and puts it in canonical order, which only sorts it, so the check's rules
/// still hold.
#[derive(Clone, Debug)]
pub struct CheckedGameData(GameData);

impl CheckedGameData {
    /// The data, for the engine to own (and sort into canonical order).
    pub(crate) fn into_data(self) -> GameData {
        self.0
    }
}

/// The behaviour proof's way past [`GameData::check`], behind the `unchecked` feature, which only `tools/proof`
/// turns on (never the apps or the tests). A ticket that adds a `check` rule together with the data fix it needs
/// proves the fix with `scripts/proof.sh data`, whose side A is the older data, which fails the new rule: the proof
/// records it anyway, warning with its problems, rather than giving that change no comparison. A `From` rather than
/// a named constructor so the harness's `Engine::new(data.into())` compiles against crates from before this type
/// too, where `Engine::new` takes `GameData` itself.
///
/// A workspace-wide build (`cargo clippy --workspace`) unifies the feature into every crate's `rules`; an app's own
/// build (`cargo leptos`, `-p backend`) doesn't have it, so no app can rely on it.
#[cfg(feature = "unchecked")]
impl From<GameData> for CheckedGameData {
    fn from(data: GameData) -> Self {
        Self(data)
    }
}

impl std::ops::Deref for CheckedGameData {
    type Target = GameData;

    fn deref(&self) -> &GameData {
        &self.0
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
    /// The domains: the options of the domain selection.
    domains: HashSet<SelectionOptionKey>,
    /// Every key an entity names, with the entity: checked against [`keys`](Self::keys) once the walk has seen them
    /// all.
    named: BTreeSet<(DataAt, DataAt)>,
    /// Every Mythic rank in range an entity names, with the entity: checked against the progression's.
    named_ranks: BTreeSet<(DataAt, u8)>,
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

    /// Notes a key `at` names, which must be an entity's of its kind ([`DataIssue::UnknownKey`]).
    fn names(&mut self, at: &DataAt, named: DataAt) {
        self.named.insert((at.clone(), named));
    }

    /// [`names`](Self::names) for keys of one kind.
    fn names_each<'k, K: Clone + 'k>(
        &mut self,
        at: &DataAt,
        keys: impl IntoIterator<Item = &'k K>,
        kind: fn(K) -> DataAt,
    ) {
        for key in keys {
            self.names(at, kind(key.clone()));
        }
    }

    /// Notes the key a value names: a skill or a weapon. A metamagic, stat or element is a pick's alone.
    fn names_value(&mut self, at: &DataAt, value: &Parameter) {
        match value {
            Parameter::Skill(skill) => self.names(at, DataAt::Skill(skill.clone())),
            Parameter::Weapon(weapon) => self.names(at, DataAt::Weapon(weapon.clone())),
            Parameter::SpellSchool(_) | Parameter::Energy(_) | Parameter::ArmourType(_) => {}
            Parameter::Metamagic(_) | Parameter::Stat(_) | Parameter::Element(_) => {
                self.problem(at, DataIssue::PickOnlyValue);
            }
        }
    }

    /// What a level, rank or option row grants: features, feats and selections. Its slots and feats for one value
    /// are checked with them ([`slots`](Self::slots), [`feats_for`](Self::feats_for)).
    fn grants(
        &mut self,
        at: &DataAt,
        features: &[FeatureKey],
        feats: &BTreeSet<FeatKey>,
        selections: &[FeatureSelectionKey],
    ) {
        self.names_each(at, features, DataAt::Feature);
        self.names_each(at, feats, DataAt::Feat);
        self.names_each(at, selections, DataAt::FeatureSelection);
    }

    /// The keys named that no entity of their kind has, and the Mythic ranks named that the progression lacks.
    fn unknown_keys(&mut self, data: &GameData) {
        for (at, named) in std::mem::take(&mut self.named) {
            if !self.keys.contains(&named) {
                self.problem(&at, DataIssue::UnknownKey(named));
            }
        }
        let ranks: HashSet<u8> = data
            .progression
            .mythic_ascensions
            .iter()
            .map(|ascension| ascension.rank)
            .collect();
        for (at, rank) in std::mem::take(&mut self.named_ranks) {
            if !ranks.contains(&rank) {
                self.problem(&at, DataIssue::UnknownRank(rank));
            }
        }
    }

    /// Notes the lists and selections the slots name.
    fn slots(&mut self, at: &DataAt, slots: &[FeatSlot]) {
        for slot in slots {
            match slot {
                FeatSlot::OneOf { list } => {
                    self.named_lists.insert(list.clone());
                    self.names(at, DataAt::FeatList(list.clone()));
                }
                FeatSlot::OptionList { selection, .. } => {
                    self.names(at, DataAt::FeatureSelection(selection.clone()));
                }
                FeatSlot::General | FeatSlot::CombatFeat | FeatSlot::Category(_) => {}
            }
        }
    }

    /// Slots granted outside a class's level rows check prerequisites, unless the bonus list is the grantor's own
    /// (`owner`: a race's or a path's key).
    fn no_bonus_list(&mut self, at: &DataAt, slots: &[FeatSlot], owner: Option<&str>) {
        self.slots(at, slots);
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
                self.names(&at, DataAt::FeatureSelection(row.selection.clone()));
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
            self.names_each(&at, &list.feats, DataAt::Feat);
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

    /// Whether a value is one a feat of `kind` is taken for. An unknown weapon fits: it's an unknown key of its own.
    fn fits(&self, kind: ParameterKind, value: &Parameter) -> bool {
        use ParameterKind as K;
        match (kind, value) {
            (K::Skill, Parameter::Skill(_))
            | (K::Weapon, Parameter::Weapon(_))
            | (K::SpellSchool, Parameter::SpellSchool(_))
            | (K::Energy, Parameter::Energy(_))
            | (K::ArmourType, Parameter::ArmourType(_))
            | (K::Stat, Parameter::Stat(_))
            | (K::Element, Parameter::Element(_)) => true,
            (K::Metamagic, Parameter::Metamagic(feat)) => {
                feat.as_str().starts_with(Feat::METAMAGIC_PREFIX)
            }
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
        self.condition_names(at, prerequisite);
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
                // An unknown feat is an unknown key of its own.
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
            Prerequisite::FeatIfValue { value, .. } => {
                match takes {
                    None => self.problem(at, DataIssue::SameValueWithoutValue),
                    Some(takes) if !self.fits(takes, value) => {
                        self.problem(at, DataIssue::IfValueOfWrongKind);
                    }
                    Some(_) => {}
                }
                return;
            }
            Prerequisite::Not(child) => {
                self.prerequisite(at, Some(child), takes);
                return;
            }
            Prerequisite::DeityOffers(domain) => {
                self.domain(at, domain);
                return;
            }
            Prerequisite::NoOtherOptionOfGroup(_) | Prerequisite::PickedThrough(_) => {
                if !matches!(at, DataAt::SelectionOption(_)) {
                    self.problem(at, DataIssue::PickLeafOutsideOption);
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

    /// The keys one condition names (a group's children are conditions of their own). A domain is checked as one
    /// ([`DataIssue::NotADomain`]).
    fn condition_names(&mut self, at: &DataAt, condition: &Prerequisite) {
        use Prerequisite as P;
        match condition {
            P::Feat(feat) | P::FeatForSameValue(feat) => self.names(at, DataAt::Feat(feat.clone())),
            P::FeatFor { feat, value } | P::FeatIfValue { value, feat } => {
                self.names(at, DataAt::Feat(feat.clone()));
                self.names_value(at, value);
            }
            P::MythicFeat(feat) => self.names(at, DataAt::MythicFeat(feat.clone())),
            P::MythicAbility(ability) => self.names(at, DataAt::MythicAbility(ability.clone())),
            P::ClassLevel { class, .. } => self.names(at, DataAt::Class(class.clone())),
            P::Archetype(archetype) => self.names(at, DataAt::Archetype(archetype.clone())),
            P::Feature { feature, .. } => self.names(at, DataAt::Feature(feature.clone())),
            P::SelectionOption(option) => self.names(at, DataAt::SelectionOption(option.clone())),
            P::NoOptionFrom(selection) | P::PickedThrough(selection) => {
                self.names(at, DataAt::FeatureSelection(selection.clone()));
            }
            P::SkillRanks { skill, .. } => self.names(at, DataAt::Skill(skill.clone())),
            P::Deity(deity) => self.names(at, DataAt::Deity(deity.clone())),
            P::Race(race) => self.names(at, DataAt::Race(race.clone())),
            P::All(_)
            | P::Any(_)
            | P::AtLeast(..)
            | P::Not(_)
            | P::NoOtherOptionOfGroup(_)
            | P::AbilityScore { .. }
            | P::BaseAttackBonus(_)
            | P::SpellLevel { .. }
            | P::SpontaneousSpellLevel { .. }
            | P::CasterLevel(_)
            | P::CharacterLevel(_)
            | P::MythicRank(_)
            | P::Alignment(_)
            | P::Atheist
            | P::Worships
            | P::DeityOffers(_)
            | P::Story(_) => {}
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
            self.names_each(
                &selection_at,
                &selection.shared_options,
                DataAt::SelectionOption,
            );
            self.names_each(&selection_at, &selection.option_levels, DataAt::Class);
            self.names_each(&selection_at, &selection.pool_of, DataAt::FeatureSelection);
            self.names_each(
                &selection_at,
                &selection.level_also_from,
                DataAt::SelectionOption,
            );
            for option in &selection.options {
                let at = DataAt::SelectionOption(option.key.clone());
                self.key(&at);
                // `rogue/rogue-talents/fast-stealth` extends `rogue/rogue-talents`.
                if !owned_by(option.key.as_str(), selection.key.as_str()) {
                    self.problem(&at, DataIssue::OptionOutsideItsSelection);
                }
                self.names_each(&at, &option.feats, DataAt::Feat);
                self.names_each(&at, &option.chosen_feat, DataAt::Feat);
                for value in &option.values {
                    self.names_value(&at, value);
                }
                self.names_each(&at, &option.pool, DataAt::FeatureSelection);
                self.names_each(&at, &option.replaces_row_of, DataAt::FeatureSelection);
                self.names_each(&at, &option.arcana, DataAt::Feature);
                // An option takes a value for its chosen feat, which must take one, or of its own kind.
                let takes = option.chosen_feat.as_ref().and_then(|feat| {
                    let kind = self.parameters.get(feat).copied();
                    if kind == Some(None) {
                        self.problem(&at, DataIssue::ChosenFeatNotParameterised);
                    }
                    kind.flatten()
                });
                if option.chosen_feat.is_some() && option.parameter.is_some() {
                    self.problem(&at, DataIssue::ParameterBesideChosenFeat);
                }
                let takes = takes.or(option.parameter);
                if !option.values.is_empty()
                    && !takes.is_some_and(|takes| {
                        option.values.iter().all(|value| self.fits(takes, value))
                    })
                {
                    self.problem(&at, DataIssue::OptionValueOfWrongKind);
                }
                // Once per value counts the values its chosen feat is held for (the Skill Focus talents), or those
                // its own value is (Finesse Training).
                if option.repeat == Repeat::PerValue {
                    if option.chosen_feat.is_none() && option.parameter.is_none() {
                        self.problem(&at, DataIssue::PerValueOptionWithoutValue);
                    }
                } else {
                    self.repeat(&at, option.repeat, false);
                }
                self.prerequisite(&at, option.prerequisite.as_ref(), takes);
                if let Some(domain) = &option.domain {
                    self.domain(&at, domain);
                }
                if let Some(arcana) = &option.arcana
                    && !option
                        .levels
                        .iter()
                        .any(|row| row.features.contains(arcana))
                {
                    self.problem(&at, DataIssue::ArcanaNotInRows(arcana.clone()));
                }
                let mut levels = HashSet::new();
                for row in &option.levels {
                    self.level(&at, row.level);
                    if !levels.insert(row.level) {
                        self.problem(&at, DataIssue::DuplicateOptionRow(row.level));
                    }
                    if let Some(list) = &row.feat_list {
                        self.named_lists.insert(list.clone());
                        self.names(&at, DataAt::FeatList(list.clone()));
                    }
                    self.grants(&at, &row.features, &row.feats, &row.selections);
                    self.names_each(&at, &row.chosen_feats, DataAt::Feat);
                    self.names_each(&at, &row.class_skills, DataAt::Skill);
                    self.slots(&at, &row.feat_slots);
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
            if selection.level_from.is_some() && !selection.option_levels.is_empty() {
                self.problem(&selection_at, DataIssue::LevelFromWithClasses);
            }
            if selection.level_also_from.is_some()
                && selection.level_from != Some(LevelFrom::OtherOptions)
            {
                self.problem(&selection_at, DataIssue::LevelAlsoFromWithoutOtherOptions);
            }
        }
    }

    /// A selection offering an option with rows names the classes whose levels make its option level.
    fn option_levels<'a>(&mut self, data: &'a GameData) {
        let selections: Vec<&'a FeatureSelection> = data.selections().collect();
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
        // Its own and shared options, and a pool selection's pools.
        let offered = |selection: &'a FeatureSelection| -> Vec<&'a SelectionOptionKey> {
            let pooled = selection
                .pool_of
                .iter()
                .filter_map(|of| by_key.get(of))
                .flat_map(|of| of.offered())
                .filter_map(|option| options.get(option)?.pool.as_ref())
                .filter_map(|pool| by_key.get(pool))
                .flat_map(|pool| pool.offered());
            selection.offered().into_iter().chain(pooled).collect()
        };
        for selection in selections.iter().copied() {
            if !selection.option_levels.is_empty()
                || selection.level_from.is_some()
                || pools.contains(&selection.key)
            {
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
            self.grants(at, &row.features, &row.feats, &row.selections);
            self.slots(at, &row.feat_slots);
            self.feats_for(at, &row.feats_for);
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

    /// A feat granted for one value is one the data has and takes a value of that kind.
    fn feats_for(&mut self, at: &DataAt, granted: &BTreeSet<FeatValue>) {
        for granted in granted {
            let (feat, value) = (&granted.feat, &granted.value);
            self.names(at, DataAt::Feat(feat.clone()));
            self.names_value(at, value);
            if let Some(kind) = self.parameters.get(feat).copied()
                && !kind.is_some_and(|kind| self.fits(kind, value))
            {
                self.problem(at, DataIssue::ValueOfWrongKind(feat.clone()));
            }
        }
    }

    /// An archetype stores only the levels where it differs from its base class, each once.
    fn archetype_rows(&mut self, at: &DataAt, class: &Class, archetype: &Archetype) {
        let mut seen = HashSet::new();
        for row in &archetype.levels {
            self.grants(at, &row.features, &row.feats, &row.selections);
            self.slots(at, &row.feat_slots);
            self.feats_for(at, &row.feats_for);
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
                && row.feats_for == base.feats_for
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

    /// A class or archetype that casts has a casting ability (its own, or its class's), and one that doesn't has
    /// none.
    fn casting_ability(&mut self, class: &Class, prestige: bool) {
        let at = DataAt::Class(class.key.clone());
        let own = class.levels.iter().any(|row| row.spell_levels().any());
        let archetype_casts = |archetype: &Archetype| {
            archetype
                .levels
                .iter()
                .any(|row| row.spell_levels.is_some_and(SpellLevels::any))
        };
        if own && class.casting_ability.is_none() {
            self.problem(&at, DataIssue::NoCastingAbility);
        }
        if class.casting_ability.is_some()
            && (prestige || !(own || class.archetypes.iter().any(archetype_casts)))
        {
            self.problem(&at, DataIssue::CastingAbilityWithoutSpells);
        }
        for archetype in &class.archetypes {
            if archetype_casts(archetype)
                && archetype
                    .casting_ability
                    .or(class.casting_ability)
                    .is_none()
            {
                self.problem(
                    &DataAt::Archetype(archetype.key.clone()),
                    DataIssue::NoCastingAbility,
                );
            }
        }
    }

    /// Each bonus's item exists, its condition is a sound prerequisite, and a bonus for the value its item is held
    /// for is on an item taken for one.
    fn bonuses(&mut self, data: &GameData) {
        let mythic: HashMap<&MythicFeatKey, Option<ParameterKind>> = data
            .mythic_feats
            .iter()
            .map(|feat| (&feat.key, feat.parameter))
            .collect();
        for bonus in &data.bonuses {
            let at = item_at(&bonus.item);
            let parameter = match &bonus.item {
                BonusItem::Feat(key) => self.parameters.get(key).copied().flatten(),
                BonusItem::MythicFeat(key) => mythic.get(key).copied().flatten(),
                _ => None,
            };
            self.names(&at, at.clone());
            self.prerequisite(&at, bonus.requires.as_ref(), None);
            if let BonusTarget::Skill(skill) = &bonus.target {
                self.names(&at, DataAt::Skill(skill.clone()));
            }
            let copies = bonus.amount == BonusAmount::SpellFocusOfAnotherSchool;
            let by_value = bonus.target == BonusTarget::SpellDc(SpellScope::Value);
            let fits = match parameter {
                Some(ParameterKind::SpellSchool) => true,
                Some(ParameterKind::Energy) => !copies,
                _ => false,
            };
            if (by_value || copies) && !(fits && by_value) {
                self.problem(&at, DataIssue::BonusValueWithoutParameter);
            }
            let held = bonus.target == BonusTarget::Held;
            if held && !matches!(parameter, Some(ParameterKind::Skill | ParameterKind::Stat)) {
                self.problem(&at, DataIssue::HeldBonusWithoutSkillOrStat);
            }
            let on_skill = match bonus.target {
                BonusTarget::Skill(_) => true,
                BonusTarget::Held => parameter == Some(ParameterKind::Skill),
                _ => false,
            };
            if matches!(bonus.amount, BonusAmount::TenRanks { .. }) && !on_skill {
                self.problem(&at, DataIssue::TenRanksWithoutSkill);
            }
        }
    }

    /// Each interactions note's item exists, and no item has two.
    fn interactions(&mut self, data: &GameData) {
        let mut seen = HashSet::new();
        for note in &data.interactions {
            let at = item_at(&note.item);
            self.names(&at, at.clone());
            if !seen.insert(&note.item) {
                self.problem(&at, DataIssue::InteractionsTwice);
            }
        }
    }

    /// Each gear item: a unique key and name, at least one bonus, each valid and none repeating a target and type,
    /// each skill named a skill of the data, and sound other-effects lines.
    fn gear(&mut self, data: &GameData) {
        let mut names = HashSet::new();
        for item in &data.gear {
            let at = DataAt::Gear(item.key.clone());
            self.key(&at);
            if !names.insert(item.name.as_str()) {
                self.problem(&at, DataIssue::GearNameTwice);
            }
            if item.bonuses.is_empty() {
                self.problem(&at, DataIssue::GearWithoutBonus);
            }
            let mut seen = HashSet::new();
            for bonus in &item.bonuses {
                if bonus.amount == 0 || (bonus.amount < 0 && bonus.kind != BonusType::Untyped) {
                    self.problem(&at, DataIssue::GearAmountInvalid);
                }
                if !seen.insert((&bonus.target, bonus.kind)) {
                    self.problem(&at, DataIssue::GearBonusTwice);
                }
                match &bonus.target {
                    GearTarget::Skill(skill) => self.names(&at, DataAt::Skill(skill.clone())),
                    GearTarget::SpellDc(SpellScope::Value) => {
                        self.problem(&at, DataIssue::GearSpellDcOfValue);
                    }
                    _ => {}
                }
            }
            let mut lines = HashSet::new();
            for line in &item.other_effects {
                if line.trim().is_empty() || !lines.insert(line.as_str()) {
                    self.problem(&at, DataIssue::GearEffectInvalid);
                }
            }
        }
    }

    fn class(&mut self, class: &Class, prestige: bool) {
        let at = DataAt::Class(class.key.clone());
        self.key(&at);
        self.names_each(&at, &class.class_skills, DataAt::Skill);
        for counts in &class.counts_as {
            self.names(&at, DataAt::Class(counts.class.clone()));
            if let CountsToward::Feature(feature) = &counts.toward {
                self.names(&at, DataAt::Feature(feature.clone()));
            }
        }
        self.prerequisite(&at, class.prerequisite.as_ref(), None);
        self.alignment_on_its_own(&at, class.prerequisite.as_ref());
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
        self.casting_ability(class, prestige);
        for archetype in &class.archetypes {
            let at = DataAt::Archetype(archetype.key.clone());
            self.key(&at);
            self.names_each(&at, archetype.class_skills.iter().flatten(), DataAt::Skill);
            self.prerequisite(&at, archetype.prerequisite.as_ref(), None);
            self.alignment_on_its_own(&at, archetype.prerequisite.as_ref());
            self.owned(
                archetype.key.as_str(),
                &archetype.features,
                &archetype.selections,
                &archetype.feat_lists,
            );
            self.archetype_rows(&at, class, archetype);
            for save in Save::ALL {
                if archetype.save_progression(save) == Some(class.save_progression(save)) {
                    self.problem(&at, DataIssue::SaveProgressionSameAsClass(save));
                }
            }
        }
    }

    /// A class's or an archetype's alignment requirement is a condition of its own, so that an archetype's can
    /// replace its class's (ticket 63): no alignment is left once it's taken away.
    fn alignment_on_its_own(&mut self, at: &DataAt, prerequisite: Option<&Prerequisite>) {
        if prerequisite
            .and_then(Prerequisite::without_alignment)
            .is_some_and(|rest| rest.names_alignment())
        {
            self.problem(at, DataIssue::AlignmentNotOnItsOwn);
        }
    }

    /// A key naming a domain names an option of the domain selection.
    fn domain(&mut self, at: &DataAt, domain: &SelectionOptionKey) {
        if !self.domains.contains(domain) {
            self.problem(at, DataIssue::NotADomain(domain.clone()));
        }
    }

    /// A spell list: its key under its owner's, the casters, spells and archetypes it names, and its levels.
    fn spell_list(&mut self, list: &SpellList) {
        let at = DataAt::SpellList(list.key.clone());
        self.key(&at);
        let owner = list.owner.key();
        if list.key.as_str() != owner && !owned_by(list.key.as_str(), owner) {
            self.problem(&at, DataIssue::KeyOutsideItsOwner);
        }
        let caster_at = |caster: &SpellCaster| match caster {
            SpellCaster::Class(key) => DataAt::Class(key.clone()),
            SpellCaster::Archetype(key) => DataAt::Archetype(key.clone()),
            SpellCaster::Path(key) => DataAt::MythicPath(key.clone()),
        };
        self.names(&at, caster_at(&list.owner));
        for user in &list.used_by {
            self.names(&at, caster_at(user));
        }
        let is_path = matches!(list.owner, SpellCaster::Path(_));
        let max = if is_path {
            MAX_MYTHIC_SPELL_LEVEL
        } else {
            MAX_CLASS_SPELL_LEVEL
        };
        for entry in &list.entries {
            if entry.level > max {
                self.problem(
                    &at,
                    DataIssue::SpellLevelAboveList {
                        level: entry.level,
                        max,
                    },
                );
            }
            self.names(&at, DataAt::Spell(entry.spell.clone()));
            let inside = match &entry.only {
                None => true,
                Some(SpellListOnly::Archetype(archetype)) => {
                    self.names(&at, DataAt::Archetype(archetype.clone()));
                    std::iter::once(&list.owner)
                        .chain(&list.used_by)
                        .any(|caster| match caster {
                            SpellCaster::Class(class) => {
                                owned_by(archetype.as_str(), class.as_str())
                            }
                            SpellCaster::Archetype(_) | SpellCaster::Path(_) => false,
                        })
                }
                Some(SpellListOnly::Story(_)) => is_path,
            };
            if !inside {
                self.problem(&at, DataIssue::OnlyOutsideTheList(entry.spell.clone()));
            }
        }
    }

    /// A deity is a key of its own, allows some alignment, favours some weapon and offers only domains.
    fn deity(&mut self, deity: &crate::game_data::Deity) {
        let at = DataAt::Deity(deity.key.clone());
        self.key(&at);
        if deity.alignments.is_empty() {
            self.problem(&at, DataIssue::DeityWithoutAlignment);
        }
        if deity.favoured_weapons.is_empty() {
            self.problem(&at, DataIssue::DeityWithoutFavouredWeapon);
        }
        for domain in &deity.domains {
            self.domain(&at, domain);
        }
        self.names_each(&at, &deity.favoured_weapons, DataAt::Weapon);
    }

    /// A selection limited by the deity's domains offers only options it can check: domains, and options naming
    /// the domain they stand for. An unknown shared key is an unknown key of its own.
    fn deity_limits(&mut self, data: &GameData) {
        let named: HashMap<&SelectionOptionKey, bool> = data
            .selections()
            .flat_map(|selection| &selection.options)
            .map(|option| (&option.key, option.domain.is_some()))
            .collect();
        for selection in data.selections() {
            if selection.deity_domains.is_none() {
                continue;
            }
            let unchecked = selection.offered().into_iter().find(|option| {
                !self.domains.contains(*option)
                    && named.get(option).is_some_and(|names_one| !names_one)
            });
            if let Some(option) = unchecked {
                self.problem(
                    &DataAt::FeatureSelection(selection.key.clone()),
                    DataIssue::DeityLimitWithoutDomain(option.clone()),
                );
            }
        }
    }

    /// A feature granting the deity's favoured weapons is one some class or archetype row grants.
    fn favoured_weapon_features(&mut self, data: &GameData) {
        let granted: HashSet<&FeatureKey> = data
            .all_classes()
            .flat_map(|class| {
                class.levels.iter().flat_map(|row| &row.features).chain(
                    class
                        .archetypes
                        .iter()
                        .flat_map(|archetype| &archetype.levels)
                        .flat_map(|row| &row.features),
                )
            })
            .collect();
        let favoured = data
            .class_features()
            .chain(data.mythic_paths.iter().flat_map(|path| &path.features))
            .filter(|feature| feature.favoured_weapon_proficiency);
        for feature in favoured {
            if !granted.contains(&feature.key) {
                self.problem(
                    &DataAt::Feature(feature.key.clone()),
                    DataIssue::FavouredWeaponsNotGranted,
                );
            }
        }
    }

    /// A class's levels can count toward another class's feature only when that class's rows (or its
    /// archetypes') grant it. An unknown class is an unknown key of its own.
    fn counts_toward_features(&mut self, data: &GameData) {
        for class in data.all_classes() {
            for counts in &class.counts_as {
                let CountsToward::Feature(feature) = &counts.toward else {
                    continue;
                };
                let Some(other) = data.class(&counts.class) else {
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
            if (1..=MAX_MYTHIC_RANK).contains(&row.rank) {
                self.named_ranks.insert((at.clone(), row.rank));
            }
            self.grants(&at, &row.features, &BTreeSet::new(), &row.selections);
            self.no_bonus_list(&at, &row.feat_slots, Some(path.key.as_str()));
            if row.rank < PATH_CHOICE_RANK
                && !(row.feat_slots.is_empty() && row.selections.is_empty())
            {
                self.problem(&at, DataIssue::SlotBeforePathChoice(row.rank));
            }
        }
        if let Some(switch) = &path.switch {
            for rank in [switch.kept_through_rank, switch.path_kept_through_rank] {
                if (1..PATH_SWITCH_RANK).contains(&rank) {
                    self.named_ranks.insert((at.clone(), rank));
                } else {
                    self.problem(&at, DataIssue::RankOutOfRange(rank));
                }
            }
            self.names_each(&at, &switch.from, DataAt::MythicPath);
            self.names_each(&at, &switch.dropped_features, DataAt::Feature);
            self.names_each(&at, &switch.dropped_selections, DataAt::FeatureSelection);
            self.names_each(&at, &switch.features, DataAt::Feature);
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

/// Where an item a bonus or an interactions note names is in the data.
fn item_at(item: &BonusItem) -> DataAt {
    match item {
        BonusItem::Race(key) => DataAt::Race(key.clone()),
        BonusItem::Subrace(key) => DataAt::Subrace(key.clone()),
        BonusItem::Background(key) => DataAt::Background(key.clone()),
        BonusItem::Feat(key) => DataAt::Feat(key.clone()),
        BonusItem::MythicFeat(key) => DataAt::MythicFeat(key.clone()),
        BonusItem::MythicAbility(key) => DataAt::MythicAbility(key.clone()),
        BonusItem::Feature(key) => DataAt::Feature(key.clone()),
        BonusItem::SelectionOption(key) => DataAt::SelectionOption(key.clone()),
    }
}
