//! [`GameData::check`]: mistakes in the game data that its types can't rule out.
//!
//! The servers refuse data with problems, and the engine takes only data that passed ([`CheckedGameData`], which only
//! [`GameData::checked`] makes), so the engine and the browser can rely on these rules. Every key an entity names is
//! one of an entity of its kind ([`DataIssue::UnknownKey`]), and every Mythic rank a path names is one of the
//! progression's ([`DataIssue::UnknownRank`]). This is the only check of references: there is no database behind it
//! (ticket 187).
//!
//! Keys carry no type prefix, so two kinds may share one (a feat and a feature both `power-attack`): each kind is
//! its own table, and a [`DataAt`] names the kind as well as the key.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

use crate::bonus::{BonusItem, SpellScope};
use crate::component::{
    Component, Condition, Descriptor, Fact, Pool, RankBase, RankProgression, StatRef, StatRule,
    StatTarget, Value, compile_requires,
};
use crate::engine::{BASE_LEVEL_CAP, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::game_data::{
    Advancement, Archetype, Class, ClassLevelRow, CountsToward, DOMAIN_SELECTION, Feat, FeatList,
    FeatSlot, FeatValue, Feature, FeatureSelection, GameData, LevelFrom, MythicPath, Save,
    SelectionOption, SpellLevels, WeaponProficiency,
};
use crate::gear::GearTarget;
use crate::keys::{
    ArchetypeKey, BackgroundKey, BlueprintGuid, ClassKey, DeityKey, FeatKey, FeatListKey,
    FeatureKey, FeatureSelectionKey, GearKey, MythicAbilityKey, MythicFeatKey, MythicPathKey,
    RaceKey, SelectionOptionKey, SkillKey, SpellKey, SpellListKey, SubraceKey, WeaponKey,
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
    /// A selection option that no selection defines ([`FeatureSelection::options`]), or that more than one does: the
    /// count of selections defining it.
    OptionNotDefinedOnce(usize),
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
    /// A selection that shares one of its own options: an own option is offered already.
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
    /// A selection's `replaced_by` names a level it doesn't skip.
    ReplacementAtUnskippedLevel(u8),
    /// A selection's `replaced_by` feature that a row of its class's table grants too, which would hold it twice.
    ReplacementOnClassRow(FeatureKey),
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
    /// A component reading the school or energy its entry is held for on an entry not taken for one: a
    /// [`SpellDc`](crate::Component::SpellDc) of scope [`Value`](crate::SpellScope::Value) off a feat or mythic feat
    /// taken for a school or an energy type, a [`SpellFocus`](crate::Component::SpellFocus) or an
    /// [`ExpandedArsenal`](crate::Component::ExpandedArsenal) off one taken for a school.
    BonusValueWithoutParameter,
    /// A stat bonus to the stat its entry is held for ([`StatTarget::Parameter`](crate::StatTarget::Parameter)), or a
    /// value or a prerequisite reading it ([`StatRef::Parameter`](crate::StatRef::Parameter)), where it has no meaning:
    /// on an entry not taken for a skill or a stat (a value: a skill), or in a prerequisite.
    HeldBonusWithoutSkillOrStat,
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
    /// A gear always-on or other-effects line that is empty, appears twice, or is in both lists.
    GearEffectInvalid,
    /// A key that isn't lowercase slugs: only `a-z`, `0-9`, `-` and `/`, never starting or ending with `-` or `/`,
    /// and no two of them in a row (`rogue/rogue-talents/fast-stealth`).
    MalformedKey,
    /// A blueprint GUID that isn't 32 lowercase hex digits, the game's own spelling (ticket 237).
    MalformedBlueprint,
    /// An [`ExpandedArsenal`](crate::Component::ExpandedArsenal) naming an entry with no
    /// [`SpellFocus`](crate::Component::SpellFocus), whose feats it can't add up (ticket 298).
    NotASpellFocus(DataAt),
    /// A [`Scales`](crate::Component::Scales) whose base names no class level nor the character level: its steps
    /// follow a level (ticket 298).
    ScalesWithoutLevel,
    /// A `Custom` rank progression with no band, which gives 0 whatever it reads (ticket 298).
    EmptyCustom,
    /// A filtered pool filtered by itself.
    FilteredByItself,
    /// A selection levelled by the Mythic ranks reached ([`LevelFrom::MythicRanks`]) that no path's rank row grants,
    /// directly or through the rows of an option it offers.
    MythicRanksOffAPath,
    /// A text that is there but empty: what it is (`slot label`, `interactions note`, `name`, `guide reference`).
    EmptyText(&'static str),
    /// An entry holding a hand field and the component kind that replaces it (ticket 295): the field. One fact has one
    /// source (`prerequisite` beside a `Requires`, a race's `ability_modifiers` beside an ability `StatBonus`, …).
    FieldAndComponent(&'static str),
    /// A component on an entry it has no meaning on: its kind (a `Requires` on a feature, which is never picked; a
    /// `NoSelectionIf` off a selection).
    ComponentOnWrongEntry(&'static str),
    /// A component granting or removing what is chosen, never granted: a class, an archetype, a race or a deity.
    NotGrantable,
    /// A rank progression dividing by a step of 0.
    ZeroStep,
    /// A `Requires` counting facts from a list of deities only (ticket 295): the game's `PrerequisiteFeaturesFromList`
    /// with `IsDeity`, which needs one of them whatever its `Amount` and counts as met at the first character level
    /// while no deity is held (`ConsiderFulfilled`), a rule the planner doesn't model.
    DeityList,
    /// A `Stat` condition on a skill's full value, which no prerequisite of the planner reads: a skill's prerequisite
    /// reads its ranks.
    FullSkillValue,
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
            DataIssue::OptionNotDefinedOnce(count) => write!(
                f,
                "{count} selections define the option among their own options; one must"
            ),
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
            DataIssue::ReplacementAtUnskippedLevel(level) => write!(
                f,
                "gives a feature in place of option level {level}'s row, which it doesn't skip"
            ),
            DataIssue::ReplacementOnClassRow(feature) => write!(
                f,
                "gives {feature} in place of a skipped option row, and a row of its class's table grants it too"
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
                "a component for the school or energy its entry is held for needs a feat or mythic feat taken for one",
            ),
            DataIssue::HeldBonusWithoutSkillOrStat => f.write_str(
                "a component for the skill or stat its entry is held for needs an entry taken for one",
            ),
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
                f.write_str(
                    "an always-on or other-effects line is empty, appears twice, or is in both lists",
                )
            }
            DataIssue::MalformedBlueprint => {
                f.write_str("the blueprint isn't a GUID of 32 lowercase hex digits")
            }
            DataIssue::NotASpellFocus(focus) => {
                write!(f, "its Expanded Arsenal names {focus}, which holds no Spell Focus")
            }
            DataIssue::ScalesWithoutLevel => {
                f.write_str("its steps read no class level nor the character level")
            }
            DataIssue::EmptyCustom => f.write_str("a custom rank progression has no band"),
            DataIssue::MalformedKey => f.write_str(
                "the key isn't lowercase slugs: a-z, 0-9, and single - or / between them",
            ),
            DataIssue::FilteredByItself => f.write_str("the selection is filtered by itself"),
            DataIssue::MythicRanksOffAPath => f.write_str(
                "levelled by the Mythic ranks reached, but the selection isn't a path's",
            ),
            DataIssue::EmptyText(what) => write!(f, "the {what} is empty"),
            DataIssue::SharesOwnOption(option) => {
                write!(f, "shares {option}, which is one of its own options")
            }
            DataIssue::FieldAndComponent(field) => {
                write!(f, "holds `{field}` and the component that replaces it")
            }
            DataIssue::ComponentOnWrongEntry(kind) => {
                write!(f, "holds a `{kind}` component, which means nothing on it")
            }
            DataIssue::DeityList => f.write_str(
                "a `Requires` counts deities from a list, whose first-level rule the planner doesn't model",
            ),
            DataIssue::NotGrantable => f.write_str(
                "a component grants or removes a class, archetype, race or deity, which are chosen",
            ),
            DataIssue::ZeroStep => f.write_str("a rank progression divides by a step of 0"),
            DataIssue::FullSkillValue => {
                f.write_str("a condition reads a skill's full value: a skill's prerequisite reads its ranks")
            }
        }
    }
}

impl GameData {
    /// Every problem in the data, ordered by where it is. Empty when the data can be seeded.
    pub fn check(&self) -> Vec<DataProblem> {
        let mut check = Checker {
            lists: self
                .feat_lists
                .iter()
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
                .selections
                .iter()
                .filter(|selection| selection.key.as_str() == DOMAIN_SELECTION)
                .flat_map(|selection| selection.options.iter().cloned())
                .collect(),
            archetype_classes: self
                .archetypes
                .iter()
                .map(|archetype| (archetype.key.clone(), archetype.class.clone()))
                .collect(),
            selection_at: (self.selections.iter().enumerate())
                .map(|(i, selection)| (selection.key.clone(), i))
                .collect(),
            ..Checker::default()
        };
        check.progression(self);
        for race in &self.races {
            let at = DataAt::Race(race.key.clone());
            check.key(&at);
            check.no_bonus_list(&at, &race.feat_slots, Some(race.key.as_str()));
            check.names_each(&at, &race.weapons, DataAt::Weapon);
            check.names_each(&at, &race.weapons_as_martial, DataAt::Weapon);
        }
        for subrace in &self.subraces {
            let at = DataAt::Subrace(subrace.key.clone());
            check.key(&at);
            check.names(&at, DataAt::Race(subrace.race.clone()));
            check.names_each(&at, &subrace.weapons, DataAt::Weapon);
            check.names_each(&at, &subrace.weapons_as_martial, DataAt::Weapon);
            // A subrace's slots replace the race's, so they may name the race's own lists.
            check.no_bonus_list(&at, &subrace.feat_slots, Some(subrace.race.as_str()));
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
            check.class(self, class, false);
        }
        for class in &self.prestige_classes {
            check.class(self, class, true);
        }
        for archetype in &self.archetypes {
            check.archetype(self, archetype);
        }
        for feature in &self.features {
            check.feature(feature);
        }
        for list in &self.feat_lists {
            check.feat_list(list);
        }
        check.selections(self);
        for ability in &self.mythic_abilities {
            let at = DataAt::MythicAbility(ability.key.clone());
            check.key(&at);
            check.grants(
                &at,
                &ability.features,
                &BTreeSet::new(),
                &ability.selections,
            );
            check.repeat(&at, ability.repeat, ability.parameter.is_some());
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
            check.names_each(&at, &background.weapons, DataAt::Weapon);
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
        check.interactions(self);
        check.gear(self);
        check.deity_limits(self);
        check.favoured_weapon_features(self);
        check.counts_toward_features(self);
        check.option_levels(self);
        check.entry_components(self);
        check.unknown_keys(self);
        check.blueprints(self);
        for list in &self.feat_lists {
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

    /// Every entry of a kind that can name a game blueprint, with the one it names (ticket 237). Skills have none: the
    /// game's skills are a stat, not a blueprint.
    pub fn blueprints(&self) -> Vec<(DataAt, Option<&BlueprintGuid>)> {
        let mut out = Vec::new();
        macro_rules! each {
            ($entries:expr, $at:path) => {
                out.extend(
                    $entries
                        .into_iter()
                        .map(|entry| ($at(entry.key.clone()), entry.blueprint.as_ref())),
                )
            };
        }
        each!(&self.races, DataAt::Race);
        each!(&self.subraces, DataAt::Subrace);
        each!(&self.weapons, DataAt::Weapon);
        each!(&self.feats, DataAt::Feat);
        each!(self.all_classes(), DataAt::Class);
        each!(&self.archetypes, DataAt::Archetype);
        each!(&self.features, DataAt::Feature);
        each!(&self.selections, DataAt::FeatureSelection);
        each!(&self.options, DataAt::SelectionOption);
        each!(&self.feat_lists, DataAt::FeatList);
        each!(&self.mythic_abilities, DataAt::MythicAbility);
        each!(&self.mythic_feats, DataAt::MythicFeat);
        each!(&self.mythic_paths, DataAt::MythicPath);
        each!(&self.backgrounds, DataAt::Background);
        each!(&self.deities, DataAt::Deity);
        each!(&self.spells, DataAt::Spell);
        each!(&self.spell_lists, DataAt::SpellList);
        each!(&self.gear, DataAt::Gear);
        out
    }

    /// The data as [`CheckedGameData`] when [`check`](Self::check) finds no problem, otherwise every problem. The
    /// checked data holds each entry's `Requires` compiled into its `prerequisite` (ticket 296).
    pub fn checked(mut self) -> Result<CheckedGameData, Vec<DataProblem>> {
        let problems = self.check();
        if problems.is_empty() {
            self.compile_prerequisites();
            Ok(CheckedGameData(self))
        } else {
            Err(problems)
        }
    }
}

/// Game data that [`GameData::check`] found no problem in, the only data [`Engine::new`](crate::Engine::new) takes:
/// the engine relies on the check's rules (every feat list or selection a slot names is one of the data's, so is
/// every selection a filtered pool is filtered by). Only [`GameData::checked`] makes one, with each entry's
/// [`Requires`](crate::Component::Requires) compiled into its `prerequisite` and no longer among its components: the
/// one tree the engine, the MCP server, the game check and the browser (which checks the JSON again) read. Outside this crate it lends the data only for
/// reading: it derefs to `&GameData`, with no `DerefMut` and no way to take the data back out. Inside it,
/// [`Engine::new`](crate::Engine::new) takes the data owned (`into_data`) and puts it in canonical order, which only
/// sorts it, so the check's rules still hold.
#[derive(Clone, Debug)]
pub struct CheckedGameData(GameData);

impl CheckedGameData {
    /// The data, for the engine to own (and sort into canonical order).
    pub(crate) fn into_data(self) -> GameData {
        self.0
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
    /// Every archetype, with its class.
    archetype_classes: HashMap<ArchetypeKey, ClassKey>,
    /// Every selection's position in the data.
    selection_at: HashMap<FeatureSelectionKey, usize>,
    /// The options' old keys, which only the link decoder reads: checked once the walk has seen every key.
    aliases: Vec<SelectionOptionKey>,
    /// Every entry holding a [`SpellFocus`](crate::Component::SpellFocus).
    spell_focuses: HashSet<DataAt>,
    /// Every fact an [`ExpandedArsenal`](crate::Component::ExpandedArsenal) names, with its entry: each checked to
    /// hold a `SpellFocus` once the walk has seen them all.
    arsenal_focuses: Vec<(DataAt, DataAt)>,
    /// Every key an entity names, with the entity: checked against [`keys`](Self::keys) once the walk has seen them
    /// all.
    named: BTreeSet<(DataAt, DataAt)>,
    /// Every Mythic rank in range an entity names, with the entity: checked against the progression's.
    named_ranks: BTreeSet<(DataAt, u8)>,
    problems: Vec<DataProblem>,
}

/// Whether `key` is lowercase slugs joined by `/`, each slug words of `a-z` and `0-9` joined by single `-`s.
fn well_formed_key(key: &str) -> bool {
    key.split('/').all(|slug| {
        slug.split('-').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
    })
}

/// Whether `guid` is spelled as the game spells a blueprint GUID: 32 lowercase hex digits, no dashes or braces.
fn well_formed_blueprint(guid: &str) -> bool {
    guid.len() == 32
        && guid
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
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
        if at.key().is_some_and(|key| !well_formed_key(key)) {
            self.problem(at, DataIssue::MalformedKey);
        }
    }

    /// A text that is there must say something.
    fn not_empty(&mut self, at: &DataAt, text: &str, what: &'static str) {
        if text.trim().is_empty() {
            self.problem(at, DataIssue::EmptyText(what));
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

    /// The blueprint GUIDs that aren't spelled as the game spells them.
    fn blueprints(&mut self, data: &GameData) {
        for (at, blueprint) in data.blueprints() {
            if blueprint.is_some_and(|guid| !well_formed_blueprint(guid.as_str())) {
                self.problem(&at, DataIssue::MalformedBlueprint);
            }
        }
    }

    /// The keys named that no entity of their kind has, and the Mythic ranks named that the progression lacks.
    fn unknown_keys(&mut self, data: &GameData) {
        for (at, named) in std::mem::take(&mut self.named) {
            if !self.keys.contains(&named) {
                self.problem(&at, DataIssue::UnknownKey(named));
            }
        }
        // An old key reads in a link as its option, so no option nor other alias may have it; nothing else names it.
        let mut aliases = HashSet::new();
        for alias in std::mem::take(&mut self.aliases) {
            let at = DataAt::SelectionOption(alias);
            if self.keys.contains(&at) || !aliases.insert(at.clone()) {
                self.problem(&at, DataIssue::DuplicateKey);
            }
            if at.key().is_some_and(|key| !well_formed_key(key)) {
                self.problem(&at, DataIssue::MalformedKey);
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
        for (at, focus) in std::mem::take(&mut self.arsenal_focuses) {
            if self.keys.contains(&focus) && !self.spell_focuses.contains(&focus) {
                self.problem(&at, DataIssue::NotASpellFocus(focus));
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
                FeatSlot::OptionList {
                    selection, label, ..
                } => {
                    self.names(at, DataAt::FeatureSelection(selection.clone()));
                    if let Some(label) = label {
                        self.not_empty(at, label, "slot label");
                    }
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

    /// A feature: the weapons and selection it names.
    fn feature(&mut self, feature: &Feature) {
        let at = DataAt::Feature(feature.key.clone());
        self.key(&at);
        self.names_each(&at, &feature.weapons, DataAt::Weapon);
        if let Some(row) = &feature.option_row {
            self.level(&at, row.level);
            self.names(&at, DataAt::FeatureSelection(row.selection.clone()));
        }
    }

    /// A feat list: the feats it names, and enough of them to pick from.
    fn feat_list(&mut self, list: &FeatList) {
        let at = DataAt::FeatList(list.key.clone());
        self.key(&at);
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
            P::ClassLevel { class, .. } | P::ClassSpellLevel { class, .. } => {
                self.names(at, DataAt::Class(class.clone()));
            }
            P::Archetype(archetype) => self.names(at, DataAt::Archetype(archetype.clone())),
            P::Feature { feature, .. } => self.names(at, DataAt::Feature(feature.clone())),
            P::SelectionOption(option) => self.names(at, DataAt::SelectionOption(option.clone())),
            P::Selection(selection) | P::NoOptionFrom(selection) | P::PickedThrough(selection) => {
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
            | P::FullAbilityScore { .. }
            | P::BaseAttackBonus(_)
            | P::CasterType(_)
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

    /// The selections and the options they offer.
    fn selections(&mut self, data: &GameData) {
        let of_paths = selections_of_paths(data);
        let mut defined: HashMap<&SelectionOptionKey, usize> = HashMap::new();
        for selection in &data.selections {
            let selection_at = DataAt::FeatureSelection(selection.key.clone());
            self.key(&selection_at);
            if selection.filtered_by.as_ref() == Some(&selection.key) {
                self.problem(&selection_at, DataIssue::FilteredByItself);
            }
            if selection.level_from == Some(LevelFrom::MythicRanks)
                && !of_paths.contains(&selection.key)
            {
                self.problem(&selection_at, DataIssue::MythicRanksOffAPath);
            }
            for shared in selection.shared_options.intersection(&selection.options) {
                self.problem(&selection_at, DataIssue::SharesOwnOption(shared.clone()));
            }
            for option in &selection.options {
                *defined.entry(option).or_default() += 1;
            }
            self.names_each(&selection_at, &selection.options, DataAt::SelectionOption);
            self.names_each(
                &selection_at,
                &selection.shared_options,
                DataAt::SelectionOption,
            );
            self.names_each(&selection_at, &selection.option_levels, DataAt::Class);
            self.names_each(
                &selection_at,
                selection.option_level_archetypes.keys(),
                DataAt::Archetype,
            );
            self.names_each(
                &selection_at,
                &selection.filtered_by,
                DataAt::FeatureSelection,
            );
            self.names_each(
                &selection_at,
                &selection.level_also_from,
                DataAt::SelectionOption,
            );
            for level in &selection.skipped_levels {
                self.level(&selection_at, *level);
            }
            for (level, feature) in &selection.replaced_by {
                self.names(&selection_at, DataAt::Feature(feature.clone()));
                if !selection.skipped_levels.contains(level) {
                    self.problem(
                        &selection_at,
                        DataIssue::ReplacementAtUnskippedLevel(*level),
                    );
                }
            }
            if selection.level_from.is_some() && selection.levels_by_class() {
                self.problem(&selection_at, DataIssue::LevelFromWithClasses);
            }
            if selection.level_also_from.is_some()
                && selection.level_from != Some(LevelFrom::OtherOptions)
            {
                self.problem(&selection_at, DataIssue::LevelAlsoFromWithoutOtherOptions);
            }
        }
        for option in &data.options {
            let at = DataAt::SelectionOption(option.key.clone());
            self.key(&at);
            self.aliases.extend(option.aliases.iter().cloned());
            let count = defined.get(&option.key).copied().unwrap_or(0);
            if count != 1 {
                self.problem(&at, DataIssue::OptionNotDefinedOnce(count));
            }
            self.names_each(&at, &option.feats, DataAt::Feat);
            self.names_each(&at, &option.chosen_feat, DataAt::Feat);
            for value in &option.values {
                self.names_value(&at, value);
            }
            self.names_each(&at, &option.replaces_row_of, DataAt::FeatureSelection);
            self.names_each(&at, &option.arcana, DataAt::Feature);
            self.names_each(
                &at,
                option.bab_from_ranks.as_ref().map(|bab| &bab.skill),
                DataAt::Skill,
            );
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
                && !takes
                    .is_some_and(|takes| option.values.iter().all(|value| self.fits(takes, value)))
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
                    if !kind.is_some_and(|kind| takes.is_some_and(|takes| same_values(kind, takes)))
                    {
                        self.problem(&at, DataIssue::RowChosenFeatOfOtherKind(feat.clone()));
                    }
                }
            }
        }
    }

    /// A selection offering an option with rows names the classes whose levels make its option level.
    fn option_levels(&mut self, data: &GameData) {
        let with_rows: HashSet<&SelectionOptionKey> = data
            .options
            .iter()
            .filter(|option| !option.levels.is_empty())
            .map(|option| &option.key)
            .collect();
        for selection in &data.selections {
            if selection.levels_by_class() || selection.level_from.is_some() {
                continue;
            }
            if let Some(option) = selection
                .offered()
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

    /// The replacement feature of a selection a table's rows name (`named`) is on no row of that table: the class's
    /// rows, or the archetype's merged over them (`rows`: the features of every row that can apply).
    fn replacements_off_the_rows<'r>(
        &mut self,
        data: &GameData,
        named: impl Iterator<Item = &'r FeatureSelectionKey>,
        rows: impl Iterator<Item = &'r Vec<FeatureKey>> + Clone,
    ) {
        let named: BTreeSet<&FeatureSelectionKey> = named.collect();
        let named: Vec<&FeatureSelection> = named
            .into_iter()
            .filter_map(|key| self.selection_at.get(key).map(|&i| &data.selections[i]))
            .collect();
        for selection in named {
            for feature in selection.replaced_by.values() {
                if rows.clone().any(|features| features.contains(feature)) {
                    let at = DataAt::FeatureSelection(selection.key.clone());
                    self.problem(&at, DataIssue::ReplacementOnClassRow(feature.clone()));
                }
            }
        }
    }

    /// An archetype stores only the levels where it differs from its base class, each once.
    fn archetype_rows(
        &mut self,
        data: &GameData,
        at: &DataAt,
        class: &Class,
        archetype: &Archetype,
    ) {
        // The selections the class's rows name are checked with the class.
        let class_selections: HashSet<&FeatureSelectionKey> = class
            .levels
            .iter()
            .flat_map(|row| &row.selections)
            .collect();
        self.replacements_off_the_rows(
            data,
            (archetype.levels.iter().flat_map(|row| &row.selections))
                .filter(|selection| !class_selections.contains(selection)),
            class
                .levels
                .iter()
                .map(|row| &row.features)
                .chain(archetype.levels.iter().map(|row| &row.features)),
        );
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
    fn casting_ability(&mut self, data: &GameData, class: &Class, prestige: bool) {
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
            && (prestige || !(own || data.archetypes_of(&class.key).any(archetype_casts)))
        {
            self.problem(&at, DataIssue::CastingAbilityWithoutSpells);
        }
        for archetype in data.archetypes_of(&class.key) {
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

    /// Each interactions note's item exists, and no item has two.
    fn interactions(&mut self, data: &GameData) {
        let mut seen = HashSet::new();
        for note in &data.interactions {
            let at = item_at(&note.item);
            self.names(&at, at.clone());
            self.not_empty(&at, &note.interactions, "interactions note");
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
            self.not_empty(&at, &item.name, "name");
            if let Some(guide) = &item.guide {
                self.not_empty(&at, &guide.0, "guide reference");
            }
            if item.stats().next().is_none() {
                self.problem(&at, DataIssue::GearWithoutBonus);
            }
            for component in &item.components {
                if !matches!(component, Component::GearStat { .. }) {
                    self.problem(&at, DataIssue::ComponentOnWrongEntry(component.kind()));
                }
            }
            let mut seen = HashSet::new();
            for (target, value, descriptor) in item.stats() {
                if value == 0 || (value < 0 && descriptor != Descriptor::Untyped) {
                    self.problem(&at, DataIssue::GearAmountInvalid);
                }
                if !seen.insert((target, descriptor)) {
                    self.problem(&at, DataIssue::GearBonusTwice);
                }
                match target {
                    GearTarget::Skill(skill) => self.names(&at, DataAt::Skill(skill.clone())),
                    GearTarget::SpellDc(SpellScope::Value) => {
                        self.problem(&at, DataIssue::GearSpellDcOfValue);
                    }
                    _ => {}
                }
            }
            let mut lines = HashSet::new();
            for line in item.always_on.iter().chain(&item.other_effects) {
                if line.trim().is_empty() || !lines.insert(line.as_str()) {
                    self.problem(&at, DataIssue::GearEffectInvalid);
                }
            }
        }
    }

    fn class(&mut self, data: &GameData, class: &Class, prestige: bool) {
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
        self.rows(&at, &class.levels, prestige);
        self.replacements_off_the_rows(
            data,
            class.levels.iter().flat_map(|row| &row.selections),
            class.levels.iter().map(|row| &row.features),
        );
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
        if prestige && data.archetypes_of(&class.key).next().is_some() {
            self.problem(&at, DataIssue::ArchetypeOnPrestigeClass);
        }
        self.casting_ability(data, class, prestige);
    }

    /// An archetype: what it names, and its rows and saves against its base class's.
    fn archetype(&mut self, data: &GameData, archetype: &Archetype) {
        let at = DataAt::Archetype(archetype.key.clone());
        self.key(&at);
        self.names(&at, DataAt::Class(archetype.class.clone()));
        self.names_each(&at, archetype.class_skills.iter().flatten(), DataAt::Skill);
        self.prerequisite(&at, archetype.prerequisite.as_ref(), None);
        self.alignment_on_its_own(&at, archetype.prerequisite.as_ref());
        // An archetype of a prestige class is the class's problem; one of no class, an unknown key, whose rows are
        // checked on their own.
        let Some(class) = data
            .classes
            .iter()
            .find(|class| class.key == archetype.class)
        else {
            for row in &archetype.levels {
                self.grants(&at, &row.features, &row.feats, &row.selections);
                self.slots(&at, &row.feat_slots);
                self.feats_for(&at, &row.feats_for);
            }
            return;
        };
        self.archetype_rows(data, &at, class, archetype);
        for save in Save::ALL {
            if archetype.save_progression(save) == Some(class.save_progression(save)) {
                self.problem(&at, DataIssue::SaveProgressionSameAsClass(save));
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
                                self.archetype_classes.get(archetype) == Some(class)
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
            .options
            .iter()
            .map(|option| (&option.key, option.domain.is_some()))
            .collect();
        for selection in &data.selections {
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
            .flat_map(|class| class.levels.iter().flat_map(|row| &row.features))
            .chain(
                data.archetypes
                    .iter()
                    .flat_map(|archetype| &archetype.levels)
                    .flat_map(|row| &row.features),
            )
            .collect();
        let favoured = data
            .features
            .iter()
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
                    || data.archetypes_of(&other.key).any(|archetype| {
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

    /// Every entry's components (ticket 295): the keys they name, a `Requires` checked as the prerequisite it compiles
    /// into, each kind only on an entry it has a meaning on, and never beside the hand field it replaces.
    fn entry_components(&mut self, data: &GameData) {
        use DataAt as At;
        for feat in &data.feats {
            let fields = [(
                "prerequisite",
                feat.prerequisite.is_some(),
                Component::is_requires as Replaces,
            )];
            let at = At::Feat(feat.key.clone());
            self.components(&at, &feat.components, true, feat.parameter, &fields);
        }
        for option in &data.options {
            let takes = (option.chosen_feat.as_ref())
                .and_then(|feat| self.parameters.get(feat).copied().flatten())
                .or(option.parameter);
            let fields = [(
                "prerequisite",
                option.prerequisite.is_some(),
                Component::is_requires as Replaces,
            )];
            let at = At::SelectionOption(option.key.clone());
            self.components(&at, &option.components, true, takes, &fields);
        }
        for ability in &data.mythic_abilities {
            let fields = [(
                "prerequisite",
                ability.prerequisite.is_some(),
                Component::is_requires as Replaces,
            )];
            let at = At::MythicAbility(ability.key.clone());
            self.components(&at, &ability.components, true, ability.parameter, &fields);
        }
        for feat in &data.mythic_feats {
            let fields = [(
                "prerequisite",
                feat.prerequisite.is_some(),
                Component::is_requires as Replaces,
            )];
            let at = At::MythicFeat(feat.key.clone());
            self.components(&at, &feat.components, true, feat.parameter, &fields);
        }
        for class in data.all_classes() {
            let fields = [
                (
                    "prerequisite",
                    class.prerequisite.is_some(),
                    Component::is_requires as Replaces,
                ),
                (
                    "class_skills",
                    !class.class_skills.is_empty(),
                    is_class_skill,
                ),
            ];
            let at = At::Class(class.key.clone());
            if let Some(compiled) = self.components(&at, &class.components, true, None, &fields) {
                self.alignment_on_its_own(&at, Some(&compiled));
            }
        }
        for archetype in &data.archetypes {
            let fields = [
                (
                    "prerequisite",
                    archetype.prerequisite.is_some(),
                    Component::is_requires as Replaces,
                ),
                (
                    "class_skills",
                    archetype.class_skills.is_some(),
                    is_class_skill,
                ),
            ];
            let at = At::Archetype(archetype.key.clone());
            if let Some(compiled) = self.components(&at, &archetype.components, true, None, &fields)
            {
                self.alignment_on_its_own(&at, Some(&compiled));
            }
        }
        for feature in &data.features {
            let fields = [(
                "weapons",
                !feature.weapons.is_empty(),
                is_weapon_proficiency as Replaces,
            )];
            let at = At::Feature(feature.key.clone());
            self.components(&at, &feature.components, false, None, &fields);
        }
        for selection in &data.selections {
            let fields = [(
                "only_without_option",
                selection.only_without_option,
                is_own_options_rule as Replaces,
            )];
            let at = At::FeatureSelection(selection.key.clone());
            self.components(&at, &selection.components, false, None, &fields);
        }
        for race in &data.races {
            let fields = [
                (
                    "ability_modifiers",
                    !race.ability_modifiers.is_empty(),
                    is_ability_bonus as Replaces,
                ),
                ("skill_points", race.skill_points > 0, is_skill_points),
                ("weapons", !race.weapons.is_empty(), is_weapon_proficiency),
            ];
            let at = At::Race(race.key.clone());
            self.components(&at, &race.components, false, None, &fields);
        }
        for subrace in &data.subraces {
            let fields = [
                (
                    "ability_modifiers",
                    !subrace.ability_modifiers.is_empty(),
                    is_ability_bonus as Replaces,
                ),
                ("skill_points", subrace.skill_points > 0, is_skill_points),
                (
                    "weapons",
                    !subrace.weapons.is_empty(),
                    is_weapon_proficiency,
                ),
            ];
            let at = At::Subrace(subrace.key.clone());
            self.components(&at, &subrace.components, false, None, &fields);
        }
        for background in &data.backgrounds {
            let fields = [
                (
                    "class_skills",
                    !background.class_skills.is_empty(),
                    is_class_skill as Replaces,
                ),
                (
                    "feats",
                    !background.feats.is_empty() || !background.feats_for.is_empty(),
                    is_feat_grant,
                ),
                (
                    "weapons",
                    !background.weapons.is_empty(),
                    is_weapon_proficiency,
                ),
            ];
            let at = At::Background(background.key.clone());
            self.components(&at, &background.components, false, None, &fields);
        }
        for deity in &data.deities {
            let at = At::Deity(deity.key.clone());
            self.components(&at, &deity.components, false, None, &[]);
        }
    }

    /// One entry's components. `picked` is set on an entry that has a prerequisite, with what its pick takes, and
    /// `fields` are the hand fields a kind replaces, each with whether the entry holds it. The `Requires` compiled,
    /// when there are any.
    fn components(
        &mut self,
        at: &DataAt,
        components: &[Component],
        picked: bool,
        takes: Option<ParameterKind>,
        fields: &[(&'static str, bool, Replaces)],
    ) -> Option<Prerequisite> {
        if components.is_empty() {
            return None;
        }
        for &(field, held, replaces) in fields {
            if held && components.iter().any(replaces) {
                self.problem(at, DataIssue::FieldAndComponent(field));
            }
        }
        for component in components {
            self.component(at, component, picked, takes);
        }
        let classes = &self.archetype_classes;
        let compiled = compile_requires(components, |archetype| classes.get(archetype))?;
        self.prerequisite(at, Some(&compiled), takes);
        Some(compiled)
    }

    /// One component: the keys it names, and whether it fits its entry.
    fn component(
        &mut self,
        at: &DataAt,
        component: &Component,
        picked: bool,
        takes: Option<ParameterKind>,
    ) {
        let wrong = |kind| DataIssue::ComponentOnWrongEntry(kind);
        let reads_parameter = |stat: &StatRef| *stat == StatRef::Parameter;
        match component {
            Component::Requires { condition, .. } => {
                if !picked {
                    self.problem(at, wrong(component.kind()));
                }
                if condition.any(&|condition| {
                    matches!(condition, Condition::Stat { stat, .. } if reads_parameter(stat))
                }) {
                    self.problem(at, DataIssue::HeldBonusWithoutSkillOrStat);
                }
                if condition.any(&reads_full_skill) {
                    self.problem(at, DataIssue::FullSkillValue);
                }
                if condition.any(&counts_deities) {
                    self.problem(at, DataIssue::DeityList);
                }
            }
            Component::PrerequisiteClassLevels {
                class,
                from,
                only_for,
                ..
            } => {
                self.names(at, DataAt::Class(class.clone()));
                self.names(at, DataAt::Class(from.clone()));
                if let Some(Pool::Options(selection)) = only_for {
                    self.names(at, DataAt::FeatureSelection(selection.clone()));
                }
            }
            Component::PrerequisiteStat { stat, rule } => {
                self.stat_names(at, stat);
                if reads_parameter(stat) {
                    self.problem(at, DataIssue::HeldBonusWithoutSkillOrStat);
                }
                match rule {
                    StatRule::ClassLevel(class)
                    | StatRule::PlusClassLevel(class)
                    | StatRule::MagusBaseAttack(class) => {
                        self.names(at, DataAt::Class(class.clone()));
                    }
                    StatRule::Ability(_) | StatRule::AtLeast(_) | StatRule::Plus(_) => {}
                }
            }
            Component::NoSelectionIf { facts, .. } => {
                if !matches!(at, DataAt::FeatureSelection(_)) {
                    self.problem(at, wrong(component.kind()));
                }
                for fact in facts {
                    self.fact_names(at, fact);
                }
            }
            Component::Grant(facts) => {
                for fact in facts {
                    self.granted(at, fact);
                }
            }
            Component::GrantAtClassLevel {
                fact,
                classes,
                archetypes,
                ..
            } => {
                self.granted(at, fact);
                self.names_each(at, classes, DataAt::Class);
                self.names_each(at, archetypes, DataAt::Archetype);
            }
            Component::GrantIf { has, fact, .. } => {
                self.fact_names(at, has);
                self.granted(at, fact);
            }
            Component::GrantOnApply(fact) | Component::RemoveOnApply(fact) => {
                self.granted(at, fact)
            }
            Component::GrantWithValue { feat, value } => {
                let granted = FeatValue {
                    feat: feat.clone(),
                    value: value.clone(),
                };
                self.feats_for(at, &BTreeSet::from([granted]));
            }
            Component::StatBonus {
                stat,
                value,
                only_if,
                ..
            } => {
                if let StatTarget::Skill(skill) = stat {
                    self.names(at, DataAt::Skill(skill.clone()));
                }
                let for_stat = matches!(takes, Some(ParameterKind::Skill | ParameterKind::Stat));
                if *stat == StatTarget::Parameter && !for_stat {
                    self.problem(at, DataIssue::HeldBonusWithoutSkillOrStat);
                }
                self.parameter_value(at, value, takes);
                self.value(at, value);
                for fact in only_if.iter().flat_map(|only| &only.facts) {
                    self.fact_names(at, fact);
                }
            }
            Component::ClassSkill(skill) => self.names(at, DataAt::Skill(skill.clone())),
            Component::Proficiency {
                weapons,
                lists,
                armour,
            } => {
                self.names_each(at, weapons, DataAt::Weapon);
                // A list or a group is held as the planner's proficiency feat for it.
                let feats = (lists.iter().map(|list| list.feat()))
                    .chain(armour.iter().map(|group| group.feat()));
                for feat in feats {
                    self.names(at, DataAt::Feat(feat));
                }
            }
            Component::SpellDc { scope, value } => {
                let fits = matches!(
                    takes,
                    Some(ParameterKind::SpellSchool | ParameterKind::Energy)
                );
                if *scope == SpellScope::Value && !fits {
                    self.problem(at, DataIssue::BonusValueWithoutParameter);
                }
                self.parameter_value(at, value, takes);
                self.value(at, value);
            }
            Component::SpellFocus { mythic_focus, .. } => {
                self.spell_focuses.insert(at.clone());
                self.fact_names(at, mythic_focus);
                if takes != Some(ParameterKind::SpellSchool) {
                    self.problem(at, DataIssue::BonusValueWithoutParameter);
                }
            }
            Component::ExpandedArsenal { focuses } => {
                for fact in focuses {
                    self.fact_names(at, fact);
                    self.arsenal_focuses.push((at.clone(), fact_at(fact)));
                }
                if takes != Some(ParameterKind::SpellSchool) {
                    self.problem(at, DataIssue::BonusValueWithoutParameter);
                }
            }
            Component::SpellPenetration { value, only_while } => {
                self.parameter_value(at, value, takes);
                self.value(at, value);
                if let Some(condition) = only_while {
                    let classes = &self.archetype_classes;
                    let compiled = condition.compile(&|archetype| classes.get(archetype));
                    self.prerequisite(at, Some(&compiled), None);
                }
            }
            Component::Concentration(value) => {
                self.parameter_value(at, value, takes);
                self.value(at, value);
            }
            Component::WeaponFocus
            | Component::WeaponSpecialization
            | Component::ImprovedCritical
            | Component::CriticalEdge
            | Component::CriticalMultiplier => {
                let weapon = matches!(
                    takes,
                    Some(ParameterKind::Weapon | ParameterKind::ExoticWeapon)
                );
                if !weapon || !matches!(at, DataAt::Feat(_)) {
                    self.problem(at, wrong(component.kind()));
                }
            }
            Component::PowerAttack | Component::Metamagic(_) => {
                if !matches!(at, DataAt::Feat(_)) {
                    self.problem(at, wrong(component.kind()));
                }
            }
            // Only a gear item holds one ([`Checker::gear`]).
            Component::GearStat { .. } => self.problem(at, wrong(component.kind())),
            Component::Scales(config) => {
                if !matches!(at, DataAt::Feature(_)) {
                    self.problem(at, wrong(component.kind()));
                }
                let reads_a_level = match &config.base {
                    RankBase::ClassLevel { classes, except }
                    | RankBase::ClassLevelWithArchetype {
                        classes, except, ..
                    } => *except || !classes.is_empty(),
                    RankBase::CharacterLevel => true,
                    _ => false,
                };
                if !reads_a_level {
                    self.problem(at, DataIssue::ScalesWithoutLevel);
                }
                self.value(at, &Value::Rank(config.clone()));
            }
            // The walk reads them from the features held, as it does `Scales`.
            Component::HighestAbilityBonus(_)
            | Component::AbilityAtLeast(_)
            | Component::BaseAttackAtLeast(_)
            | Component::GoldenDragonSkills => {
                if !matches!(at, DataAt::Feature(_)) {
                    self.problem(at, wrong(component.kind()));
                }
            }
            Component::SkillPoints(_) | Component::Flag(_) => {}
        }
    }

    /// A value reading the skill its entry is held for ([`StatRef::Parameter`]) only on an entry taken for a skill.
    fn parameter_value(&mut self, at: &DataAt, value: &Value, takes: Option<ParameterKind>) {
        let reads = match value {
            Value::Rank(config) => config.base == RankBase::BaseStat(StatRef::Parameter),
            Value::Shared(shared) => [&shared.count, &shared.bonus]
                .into_iter()
                .flatten()
                .any(|part| matches!(&**part, Value::Rank(config) if config.base == RankBase::BaseStat(StatRef::Parameter))),
            Value::Flat(_) | Value::PerRank(_) => false,
        };
        if reads && takes != Some(ParameterKind::Skill) {
            self.problem(at, DataIssue::HeldBonusWithoutSkillOrStat);
        }
    }

    /// A fact a component grants or removes: one of the data, and never one that is chosen.
    fn granted(&mut self, at: &DataAt, fact: &Fact) {
        self.fact_names(at, fact);
        if !fact.is_grantable() {
            self.problem(at, DataIssue::NotGrantable);
        }
    }

    /// The key a fact names.
    fn fact_names(&mut self, at: &DataAt, fact: &Fact) {
        self.names(at, fact_at(fact));
    }

    /// The key a stat names: a skill's.
    fn stat_names(&mut self, at: &DataAt, stat: &StatRef) {
        if let StatRef::Skill(skill) = stat {
            self.names(at, DataAt::Skill(skill.clone()));
        }
    }

    /// The keys a value reads, and its progressions' steps.
    fn value(&mut self, at: &DataAt, value: &Value) {
        match value {
            Value::Flat(_) | Value::PerRank(_) => {}
            Value::Rank(config) => {
                if config.progression.step() == Some(0) {
                    self.problem(at, DataIssue::ZeroStep);
                }
                if config.progression == RankProgression::Custom(Vec::new()) {
                    self.problem(at, DataIssue::EmptyCustom);
                }
                match &config.base {
                    RankBase::ClassLevel { classes, .. } => {
                        self.names_each(at, classes, DataAt::Class);
                    }
                    RankBase::ClassLevelWithArchetype {
                        classes,
                        archetypes,
                        features,
                        ..
                    } => {
                        self.names_each(at, classes, DataAt::Class);
                        self.names_each(at, archetypes, DataAt::Archetype);
                        for fact in features {
                            self.fact_names(at, fact);
                        }
                    }
                    RankBase::FeatureRank(fact) => self.fact_names(at, fact),
                    RankBase::FeatureList(facts) | RankBase::FeatureListRanks(facts) => {
                        for fact in facts {
                            self.fact_names(at, fact);
                        }
                    }
                    RankBase::BaseStat(stat) => self.stat_names(at, stat),
                    RankBase::CharacterLevel
                    | RankBase::MythicLevel
                    | RankBase::StatBonus(_)
                    | RankBase::BaseAttack => {}
                }
            }
            Value::Shared(shared) => {
                for part in [&shared.count, &shared.bonus].into_iter().flatten() {
                    self.value(at, part);
                }
            }
        }
    }
}

/// The selections a Mythic path's rank rows grant, and the selections the rows of the options they offer grant, and
/// so on.
fn selections_of_paths(data: &GameData) -> HashSet<FeatureSelectionKey> {
    let selections: HashMap<&FeatureSelectionKey, &FeatureSelection> =
        data.selections.iter().map(|s| (&s.key, s)).collect();
    let options: HashMap<&SelectionOptionKey, &SelectionOption> =
        data.options.iter().map(|o| (&o.key, o)).collect();
    let mut found: HashSet<FeatureSelectionKey> = HashSet::new();
    let mut next: Vec<&FeatureSelectionKey> = data
        .mythic_paths
        .iter()
        .flat_map(|path| &path.ranks)
        .flat_map(|row| &row.selections)
        .collect();
    while let Some(key) = next.pop() {
        if !found.insert(key.clone()) {
            continue;
        }
        let Some(selection) = selections.get(key) else {
            continue;
        };
        for option in selection.offered() {
            let rows = options.get(option).into_iter().flat_map(|o| &o.levels);
            next.extend(rows.flat_map(|row| &row.selections));
        }
    }
    found
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

/// Whether a component replaces a hand field of its entry ([`DataIssue::FieldAndComponent`]).
type Replaces = fn(&Component) -> bool;

fn is_class_skill(component: &Component) -> bool {
    matches!(component, Component::ClassSkill(_))
}

fn is_ability_bonus(component: &Component) -> bool {
    matches!(
        component,
        Component::StatBonus {
            stat: StatTarget::Ability(_),
            ..
        }
    )
}

fn is_weapon_proficiency(component: &Component) -> bool {
    matches!(component, Component::Proficiency { weapons, .. } if !weapons.is_empty())
}

fn is_skill_points(component: &Component) -> bool {
    matches!(component, Component::SkillPoints(_))
}

fn is_feat_grant(component: &Component) -> bool {
    match component {
        Component::Grant(facts) => facts.iter().any(|fact| matches!(fact, Fact::Feat(_))),
        Component::GrantWithValue { .. } => true,
        _ => false,
    }
}

fn is_own_options_rule(component: &Component) -> bool {
    matches!(
        component,
        Component::NoSelectionIf {
            any_option: true,
            ..
        }
    )
}

/// Whether a condition counts facts from a list of deities only ([`DataIssue::DeityList`]).
fn counts_deities(condition: &Condition) -> bool {
    matches!(condition, Condition::AtLeast(_, facts)
        if !facts.is_empty() && facts.iter().all(|fact| matches!(fact, Fact::Deity(_))))
}

/// Whether a condition reads a skill's full value ([`DataIssue::FullSkillValue`]).
fn reads_full_skill(condition: &Condition) -> bool {
    matches!(
        condition,
        Condition::Stat {
            stat: StatRef::Skill(_),
            full: true,
            ..
        }
    )
}

/// Where a fact's entry is.
fn fact_at(fact: &Fact) -> DataAt {
    match fact {
        Fact::Feat(key) => DataAt::Feat(key.clone()),
        Fact::Feature(key) => DataAt::Feature(key.clone()),
        Fact::Option(key) => DataAt::SelectionOption(key.clone()),
        Fact::Selection(key) => DataAt::FeatureSelection(key.clone()),
        Fact::MythicAbility(key) => DataAt::MythicAbility(key.clone()),
        Fact::MythicFeat(key) => DataAt::MythicFeat(key.clone()),
        Fact::Class(key) => DataAt::Class(key.clone()),
        Fact::Archetype(key) => DataAt::Archetype(key.clone()),
        Fact::Race(key) => DataAt::Race(key.clone()),
        Fact::Deity(key) => DataAt::Deity(key.clone()),
    }
}
