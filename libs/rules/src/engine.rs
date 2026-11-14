//! The rules engine: walks a Build level by level against the game data.
//!
//! One walk produces everything: the state at each level, the slots and their picks, the Mythic track, and the
//! choices that don't hold up. Picks are checked in Build order against the state just before their slot, and a
//! pick that fails doesn't count as held, so a broken pick also breaks whatever depended on it (removing Cleave
//! breaks Great Cleave, which breaks anything needing Great Cleave).
//!
//! The sheet only ever holds a possible Build: it offers no edit that would break another choice
//! ([`Engine::allows`]), so choices come off from the leaves inward ([`Engine::removable`]) and the pickers lock
//! what would break a later choice, an item of the gear that a choice needs included ([`Engine::gear_options`]).
//! A link to a Build that isn't possible opens nothing ([`Engine::open_link`]): it says only that it doesn't open,
//! never why. [`Engine::failures`] lists every choice that fails and why, for a
//! Build planned whole (the MCP server's `validate`); it never makes a Build possible.

use std::collections::BTreeSet;

use crate::build::{AbilityScores, BuildStart, Edit, Faith, Pick, SlotId};
use crate::check::CheckedGameData;
use crate::choice::{Choice, Issue};
use crate::codec::LinkKeys;
use crate::game_data::{
    Ability, AbilityBonus, AbilityModifier, Advancement, Archetype, ArchetypeLevelRow, Background,
    Class, ClassLevelRow, Deity, FeatCategory, FeatList, FeatSlot, FeatValue, Race, Skill,
    SpellLevels,
};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey, MythicPathKey,
    RaceKey, SubraceKey,
};
use crate::parameter::{Parameter, ParameterKind};
use crate::state::CharacterState;

mod address;
mod attack;
mod build_sheet;
mod caster;
mod catalog;
mod controls;
mod gear;
mod labels;
mod left_out;
mod pickers;
mod ranks;
mod removable;
mod slots;
mod switch;
mod totals;
mod walk;

pub use address::{Address, NotAnAddress, Place};
pub use attack::{ATTACK_LINE_LEAVES_OUT, AttackLine, AttackPart, PowerAttack};
pub use caster::{CASTER_LINE_LEAVES_OUT, CasterLine, CasterPart, SpellDc};
use catalog::Catalog;
pub use gear::{GearDependent, GearNeed, GearOption};
pub use left_out::LeftOut;
pub use ranks::{LaterRankLevels, LaterRankPick, RankLevelOption, ShiftedPick};
pub use totals::{CLASS_SKILL_BONUS, SkillTotal, TOTALS_LEAVE_OUT, Total, TotalPart, Totals};

/// The highest character level without a path that raises it.
pub const BASE_LEVEL_CAP: u8 = 20;
/// The Mythic rank of the first ascension, where the path-linked ability is chosen.
pub const FIRST_ASCENSION_RANK: u8 = 1;
/// The Mythic rank at which the path is chosen.
pub const PATH_CHOICE_RANK: u8 = 3;
/// The Mythic rank at which a Path switch can happen.
pub const PATH_SWITCH_RANK: u8 = 8;

/// The game data plus lookups by key. Build it once when the game data arrives, then ask it about Builds. It takes
/// only [`CheckedGameData`]: every engine is on data that passed [`GameData::check`](crate::GameData::check).
#[derive(Clone, Debug)]
pub struct Engine {
    /// The game data and its lookups by key.
    catalog: Catalog,
    /// The game data's keys by hash, to read links.
    link_keys: LinkKeys,
}

impl Engine {
    /// Indexes the game data, after putting it in canonical order.
    pub fn new(data: CheckedGameData) -> Self {
        let catalog = Catalog::new(data);
        Self {
            link_keys: LinkKeys::new(catalog.data()),
            catalog,
        }
    }

    /// Every race, with its subraces, in key order: the race picker's list.
    pub fn races(&self) -> &[Race] {
        &self.catalog.data().races
    }

    /// Every background, in key order: the background picker's list.
    pub fn backgrounds(&self) -> &[Background] {
        &self.catalog.data().backgrounds
    }

    /// Every deity, in key order: the deity picker's list.
    pub fn deities(&self) -> &[Deity] {
        &self.catalog.data().deities
    }

    /// Whether a Build start's faith and alignment don't fit: the one rule both the walk and the start pickers ask.
    /// The alignment comes after the faith, so an alignment with no faith yet fails [`Choice::Alignment`] with
    /// [`Issue::NeedsFaith`]; one the deity doesn't allow fails [`Choice::Deity`] with [`Issue::DeityAlignment`].
    /// `None` for a start without an alignment yet, for atheism, and for a deity the data lacks (which the walk
    /// fails on its own: a stale link, which the sheet never holds).
    fn faith_misfit(&self, start: &BuildStart) -> Option<(Choice, Issue)> {
        if start.deity.is_none() && start.alignment.is_some() {
            return Some((Choice::Alignment, Issue::NeedsFaith));
        }
        let key = start.deity.as_ref().and_then(Faith::deity)?;
        let alignment = start.alignment?;
        let deity = self.catalog.deity(key)?;
        (!deity.alignments.contains(&alignment))
            .then(|| (Choice::Deity, Issue::DeityAlignment { deity: key.clone() }))
    }

    /// The note the sheet shows for the favoured weapons a character is proficient with ("Proficient with Longbow
    /// (Erastil)"): `None` without a feature granting them, a deity, or a weapon.
    pub fn favoured_weapons_note(&self, state: &CharacterState) -> Option<String> {
        let deity = self.catalog.deity(state.deity()?)?;
        if deity.favoured_weapons.is_empty()
            || !state
                .features
                .keys()
                .any(|feature| self.catalog.grants_favoured_weapons(feature))
        {
            return None;
        }
        let weapons: Vec<&str> = deity
            .favoured_weapons
            .iter()
            .map(|weapon| self.weapon_name(weapon))
            .collect();
        Some(format!(
            "Proficient with {} ({})",
            weapons.join(" and "),
            deity.name
        ))
    }

    /// Every skill, in key order: the skill ranks panel's rows.
    pub fn skills(&self) -> &[Skill] {
        &self.catalog.data().skills
    }

    /// Every value of a kind, in key (or declaration) order: the skills, weapons, exotic weapons, schools, energy
    /// types, armour types, metamagic feats, stats or elements.
    pub fn parameter_values(&self, kind: ParameterKind) -> Vec<Parameter> {
        self.catalog.parameter_values(kind)
    }

    /// The kind of value a pick has still to name: what its item takes, when the pick names nothing (Favorite
    /// Metamagic from a link made before it took its metamagic: [`ParameterKind::may_stay_open`]). `None` for a pick
    /// that names its value or takes none.
    pub fn value_left_open(&self, pick: &Pick) -> Option<ParameterKind> {
        if pick.parameter().is_some() {
            return None;
        }
        self.catalog.pick_item(pick)?.parameter
    }

    /// The kind of value a pick's item takes, whatever the pick names: `Some(None)` for an item that takes none, and
    /// `None` for a key the game data lacks among items of the pick's kind (a walk fails it with
    /// [`Issue::UnknownKey`]).
    pub fn value_kind(&self, pick: &Pick) -> Option<Option<ParameterKind>> {
        Some(self.catalog.pick_item(pick)?.parameter)
    }

    /// A feat list by key, wherever it's defined.
    pub fn feat_list(&self, key: &FeatListKey) -> Option<&FeatList> {
        self.catalog.feat_list(key)
    }

    /// The daily choices a character holds (Mixed Bloodline): the sheet shows each as a note, since the game asks
    /// for it again every day and a Build doesn't record it.
    pub fn daily_choices(&self, state: &CharacterState) -> Vec<FeatureKey> {
        state
            .features
            .keys()
            .filter(|feature| self.catalog.is_daily_choice(feature))
            .cloned()
            .collect()
    }

    /// The modifier a Build start's race (or its subrace, whose modifiers replace the race's) gives an ability:
    /// the chosen bonus counts toward the ability the start puts it on, and toward none until it does.
    pub fn racial_modifier(&self, start: &BuildStart, ability: Ability) -> i8 {
        racial_amounts(self.catalog.racial_modifiers(start), start)
            .filter(|(to, _)| *to == ability)
            .map(|(_, amount)| amount)
            .sum()
    }

    /// Whether a Build start's race (or subrace) lets it choose where a bonus goes ("+2 to one ability score").
    pub fn chooses_racial_ability(&self, start: &BuildStart) -> bool {
        self.catalog
            .racial_modifiers(start)
            .iter()
            .any(|modifier| matches!(modifier, AbilityModifier::Chosen { .. }))
    }
}

/// Each racial modifier's ability and amount: a chosen one on the start's racial ability, and left out while the
/// start puts it nowhere.
fn racial_amounts<'a>(
    modifiers: &'a [AbilityModifier],
    start: &'a BuildStart,
) -> impl Iterator<Item = (Ability, i8)> + 'a {
    modifiers.iter().filter_map(|modifier| match *modifier {
        AbilityModifier::Fixed { ability, amount } => Some((ability, amount)),
        AbilityModifier::Chosen { amount } => start.racial_ability.map(|ability| (ability, amount)),
    })
}

/// What a Mythic rank's general pick is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MythicPickKind {
    Ability,
    Feat,
}

/// Odd ranks give a mythic ability and even ranks a mythic feat (MR 1 an ability, MR 2 a feat, then alternating).
pub fn mythic_pick_kind(rank: u8) -> MythicPickKind {
    if rank % 2 == 1 {
        MythicPickKind::Ability
    } else {
        MythicPickKind::Feat
    }
}

/// What one Class level gives a character of a class, with or without an archetype: the base class's row, or
/// the archetype's row at that level when it has one. BAB and saves come from the progressions instead
/// ([`Class::base_attack_bonus`], [`Class::save_bonus`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveRow<'a> {
    pub level: u8,
    pub spell_levels: SpellLevels,
    /// Prestige classes only: this level advances the chosen arcane class's spellcasting.
    pub advances_arcane: bool,
    /// Prestige classes only: this level advances the chosen divine class's spellcasting.
    pub advances_divine: bool,
    pub features: &'a [FeatureKey],
    pub feats: &'a BTreeSet<FeatKey>,
    pub feats_for: &'a BTreeSet<FeatValue>,
    pub feat_slots: &'a [FeatSlot],
    pub selections: &'a [FeatureSelectionKey],
}

/// The row in force at one level of `class`, taken with `archetype` (which must be one of the class's): the
/// archetype's row at that level replaces the base row's grants in full, and its spell levels when it sets them;
/// a level the archetype leaves out is the base row. `None` past the class's table. The one place rows merge.
pub fn effective_row<'a>(
    class: &'a Class,
    archetype: Option<&'a Archetype>,
    level: u8,
) -> Option<EffectiveRow<'a>> {
    let base = class_row(class, level)?;
    let own =
        archetype.and_then(|archetype| archetype.levels.iter().find(|row| row.level == level));
    Some(merged_row(base, own))
}

/// A class's row at a level: the row at that position, since the data check requires a table to run 1, 2, … n;
/// a table that doesn't is searched.
fn class_row(class: &Class, level: u8) -> Option<&ClassLevelRow> {
    usize::from(level)
        .checked_sub(1)
        .and_then(|index| class.levels.get(index))
        .filter(|row| row.level == level)
        .or_else(|| class.levels.iter().find(|row| row.level == level))
}

/// An archetype's row at a level, its rows being in level order (as [`GameData::canonical`] puts them): found by
/// halving, the first of any at that level as a search from the start would find.
fn canonical_archetype_row(archetype: &Archetype, level: u8) -> Option<&ArchetypeLevelRow> {
    let rows = &archetype.levels;
    rows.get(rows.partition_point(|row| row.level < level))
        .filter(|row| row.level == level)
}

/// A base row with the archetype's row at its level, if any, merged over it as [`effective_row`] says.
fn merged_row<'a>(base: &'a ClassLevelRow, own: Option<&'a ArchetypeLevelRow>) -> EffectiveRow<'a> {
    let base_row = EffectiveRow {
        level: base.level,
        spell_levels: base.spell_levels(),
        advances_arcane: base.advances_arcane,
        advances_divine: base.advances_divine,
        features: &base.features,
        feats: &base.feats,
        feats_for: &base.feats_for,
        feat_slots: &base.feat_slots,
        selections: &base.selections,
    };
    let Some(own) = own else {
        return base_row;
    };
    EffectiveRow {
        spell_levels: own.spell_levels.unwrap_or(base_row.spell_levels),
        features: &own.features,
        feats: &own.feats,
        feats_for: &own.feats_for,
        feat_slots: &own.feat_slots,
        selections: &own.selections,
        ..base_row
    }
}

/// Every row of a class's table, in level order, as [`effective_row`] gives it.
fn effective_rows<'a>(
    class: &'a Class,
    archetype: Option<&'a Archetype>,
) -> impl Iterator<Item = EffectiveRow<'a>> {
    class
        .levels
        .iter()
        .filter_map(move |row| effective_row(class, archetype, row.level))
}

/// What a Build gives at each of its levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sheet {
    /// `levels[0]` is character level 1.
    pub levels: Vec<LevelSheet>,
    /// MR 1–10 in rank order, reached or not.
    pub ranks: Vec<RankSheet>,
    /// The character level cap at the end of the Build: 20, or 40 after a Legend switch.
    pub level_cap: u8,
}

/// One row of the sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelSheet {
    pub character_level: u8,
    pub class: ClassKey,
    /// The class level this Level-up reaches in its class.
    pub class_level: u8,
    /// Whether this level has the +1 ability increase.
    pub ability_increase: bool,
    pub skill_points: SkillPoints,
    /// Feat and Feature selection slots, in the order picks are checked: the class's selections, the class's feat
    /// slots, the race's, the general slot, then the slots the options' rows grant and the feats' extra picks
    /// ([`Feat::extra_pick_of`](crate::Feat::extra_pick_of)), in the order the level grants them.
    pub slots: Vec<Slot>,
    /// The Mythic ranks reached at this level (two at 17).
    pub mythic_ranks: Vec<u8>,
    /// The state at the end of this level, after its Level-up and any Mythic ascensions.
    pub state: CharacterState,
}

/// One Mythic rank on the Mythic track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankSheet {
    pub rank: u8,
    /// The character level this rank's Mythic ascension is taken at: the rank table's, or a later one the Build
    /// places it at ([`Engine::rank_level`]).
    pub character_level: u8,
    pub status: RankStatus,
    /// The path whose row this rank gives: the path in force from MR 3. MR 1 and 2 show the first ascension's path
    /// ([`Build::first_ascension`](crate::Build::first_ascension)), or without one, the path chosen at MR 3 once the
    /// Build reaches it; before that, none.
    pub path: Option<MythicPathKey>,
    /// The general mythic pick, then the path's picks. Empty when not reached.
    pub slots: Vec<Slot>,
    /// The slots whose picks a Path switch took away (all of a dropped rank's, the old path's on a partly dropped
    /// one), in slot order.
    pub dropped_slots: Vec<SlotId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RankStatus {
    Reached,
    /// Reached, then partly lost to a Path switch: its general pick or the old path's grants at it, or a grant the
    /// switch drops by name (Devil from Aeon: the gaze picks).
    PartlyDropped,
    /// Reached, then lost to a Path switch that keeps neither its general pick nor the old path's grants.
    Dropped,
    /// Not reached: the Build is too short, or a Legend switch ended the ascensions.
    NotReached,
}

/// One slot and what fills it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: SlotId,
    pub kind: SlotKind,
    pub pick: Option<Pick>,
}

/// What a slot accepts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotKind {
    Feat(FeatSlot),
    Selection(FeatureSelectionKey),
    MythicAbility,
    MythicFeat,
}

/// Skill points at one level: class skill points plus the Intelligence modifier (at least 1), and how many the
/// Level-up spends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkillPoints {
    pub available: u8,
    pub spent: u8,
}

/// One option for a slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotOption {
    pub pick: Pick,
    /// `Ok` when it can be picked here; otherwise why not (prerequisites with reasons, already held, or
    /// [`Issue::BreaksLater`]). An unmet "or" names only the alternatives the character's classes give, when any
    /// does.
    pub status: Result<(), Issue>,
    /// The edit that picks it: [`Edit::Pick`] in the slot.
    pub edit: Edit,
    /// For an option open only because of the Build's gear (the slot takes it with the gear and not without):
    /// the items it needs. `None` for any other, and always for a Build without gear.
    pub with_gear: Option<GearNeed>,
}

/// One entry of a slot's picker, as the sheet shows it ([`Engine::slot_entries`]): what a pick takes (a feat,
/// selection option, mythic ability or mythic feat) with its options, one for each value it can name or one alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotEntry {
    pub name: String,
    /// The filter chip it's filed under: a feat's category, the Feature selection defining an option, a mythic
    /// pick's guide heading. Not [`Self::feat_category`], which names the category of the feat it takes, if any.
    pub category: String,
    /// The category of the feat it takes, which picks its icon: see [`Engine::feat_category`].
    pub feat_category: Option<FeatCategory>,
    /// Its short summary, in our own words; empty when the data has none.
    pub summary: String,
    /// "(Story: Vow of Dahak)", for the story conditions in its prerequisite, which never block it.
    pub story_note: Option<String>,
    /// "With Belt of Incredible Dexterity +2", for an entry open only because of the Build's gear: every open
    /// option of it needs gear ([`SlotOption::with_gear`]), and the note is the first one's
    /// ([`Engine::gear_note`]).
    pub gear_note: Option<String>,
    /// What its picks name, with the article ("a weapon"), for an entry whose picks name a value: it opens the list
    /// of its options' values.
    pub noun: Option<&'static str>,
    /// Whether any of its options can be taken.
    pub open: bool,
    /// Why none of its options can be taken, when it shows a reason: an option's reason other than already holding
    /// it (Weapon Focus held for a longsword but needing BAB +1 for the rest says the BAB), else the first option's.
    /// `None` when that option would break a later choice, and for an open entry.
    pub reason: Option<String>,
    pub options: Vec<EntryOption>,
}

/// One option of a [`SlotEntry`]: the slot's option, with its value's name and its reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryOption {
    pub option: SlotOption,
    /// The value its pick names, as the sheet shows it ("Longsword"); empty for a pick that names none.
    pub value: String,
    /// Why it can't be taken, when it shows a reason ([`Engine::issue_text`]).
    pub reason: Option<String>,
}

impl SlotEntry {
    /// For a locked entry, the option whose refusal it shows (its reason, and the issue, pick and edit that stand
    /// for the entry): the first not already held, else the first locked. `None` for an open entry.
    pub fn shown(&self) -> Option<&EntryOption> {
        (!self.open)
            .then(|| pickers::shown_option(&self.options))
            .flatten()
    }

    /// The line under its name: "Choose a weapon." for an entry of values, then its summary and story note.
    pub fn detail(&self) -> String {
        let choose = self.noun.map(|noun| format!("Choose {noun}."));
        [
            choose.as_deref(),
            Some(self.summary.as_str()),
            self.story_note.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
    }
}

/// A choice the Build holds and its Withdrawal: the edit that takes it back ([`Engine::held_choices`]).
///
/// Taking a choice back means: a pick cleared; a level taken by no class (what its class row gives goes, and the
/// levels of its class after it each move down one); the last level removed; a class's archetype or advanced classes
/// unchosen; a point-buy score at 7; a level's ability increase or a skill's ranks at a level gone; the race (with
/// its subrace and racial ability), subrace, alignment, background, faith, first ascension, path or Path switch
/// unchosen; an item of the gear taken off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Withdrawal {
    pub choice: Choice,
    pub edit: Edit,
    /// The choices whose failing the edit doesn't count against it: the edit's own (a Build start choice's only
    /// itself, not the other start choices its `Edit::Start` also sets) and any listed with it (a level's skill
    /// points, for its class).
    excused: Vec<Choice>,
}

/// One option of the class picker at a level: a class, or a base class with one of its archetypes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassOption {
    pub class: ClassKey,
    pub archetype: Option<ArchetypeKey>,
    /// `Ok` when it can be taken at that level; otherwise why not: the class's (and archetype's) entry
    /// prerequisites against the character before the level, the class table's end, or, at a level the Build
    /// has, [`Issue::BreaksLater`]. A prestige class that asks which classes it advances is open when some
    /// advancement keeps the Build ([`Engine::advances_options`] lists those).
    pub status: Result<(), Issue>,
    /// The edit that takes it: a class first taken at that level gets the option's archetype and keeps any
    /// advancement the Build has for it; one taken before keeps its choice.
    ///
    /// For a prestige class that asks which classes it advances ([`Engine::asks_advances`]), this edit alone may
    /// break the Build even when the option is `Ok`: the option is open when *some* advancement keeps the Build,
    /// and this edit carries the Build's own (none, for a class the Build doesn't take). The caller picks the
    /// advancement first and applies that answer's edit instead ([`Engine::advances_options`], each of which keeps
    /// the Build).
    pub edit: Edit,
    /// For an open option at a level the Build has or the next: the choices that, with it, would be allowed only
    /// because of the Build's gear and aren't today. The class itself is among them when only the gear meets its
    /// entry (or its archetype's): [`Choice::Class`] at that level, or [`Choice::Archetype`]. Always empty for a
    /// Build without gear.
    pub leans_on_gear: Vec<Choice>,
}

/// One answer to which classes a prestige class's levels advance ([`Engine::advances_options`]), with the edit that
/// takes the class with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancesOption {
    pub advances: BTreeSet<ClassKey>,
    pub edit: Edit,
}

/// One option of the path picker (MR 3) or the Path switch picker (MR 8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathOption {
    pub path: MythicPathKey,
    /// `Ok` when it can be taken; a switch open only from other paths than the Build's is
    /// [`Issue::NotFromPath`], and one that would break a later choice [`Issue::BreaksLater`].
    pub status: Result<(), Issue>,
    /// The edit that takes it: [`Edit::MythicPath`], [`Edit::PathSwitch`] or, for the first ascension's picker,
    /// [`Edit::FirstAscension`].
    pub edit: Edit,
}

/// One option of a Build start picker (race, faith, alignment, background, the racial ability) or of a level's
/// ability increase: what it chooses, whether it can be chosen now, and the edit that chooses it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceOption<T> {
    pub value: T,
    /// `Ok` when it can be chosen; otherwise why not: a faith and an alignment that don't fit
    /// ([`Issue::NeedsFaith`], [`Issue::DeityAlignment`]), or [`Issue::BreaksLater`]. What the Build holds now is
    /// always `Ok`.
    pub status: Result<(), Issue>,
    /// The edit that chooses it, side effects included (a race without a racial choice also clears the racial
    /// ability).
    pub edit: Edit,
    /// For an open option: the choices that, after it, would be allowed only because of the Build's gear and aren't
    /// today (the racial +2 moved off Dexterity, under a belt, with a feat needing the score). With any, the option
    /// is allowed but not safe. Always empty for a Build without gear.
    pub leans_on_gear: Vec<Choice>,
}

/// What the race picker chooses: a race, or one of its subraces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceChoice {
    pub race: RaceKey,
    pub subrace: Option<SubraceKey>,
}

/// One step of a stepper: whether it can be taken now (`Ok`, or [`Issue::BreaksLater`]), and its edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub status: Result<(), Issue>,
    pub edit: Edit,
    /// For an open point-buy step: the choices that, after it, would be allowed only because of the Build's gear and
    /// aren't today ([`Engine::newly_gear_dependent`]). With any, the step is allowed but not safe. Always empty
    /// for a Build without gear, and for a skill's step (ranks never see gear).
    pub leans_on_gear: Vec<Choice>,
}

/// A stepper of one point-buy score or one skill's ranks at a level: a step down and a step up, each `None` past the
/// end of its range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stepper<T> {
    pub value: T,
    pub down: Option<Step>,
    pub up: Option<Step>,
}

impl Sheet {
    /// The state at the end of a character level.
    pub fn at(&self, character_level: u8) -> Option<&CharacterState> {
        let index = usize::from(character_level).checked_sub(1)?;
        self.levels.get(index).map(|level| &level.state)
    }

    /// The slot with this id, wherever it is.
    pub fn slot(&self, id: &SlotId) -> Option<&Slot> {
        self.levels
            .iter()
            .flat_map(|level| &level.slots)
            .chain(self.ranks.iter().flat_map(|rank| &rank.slots))
            .find(|slot| &slot.id == id)
    }
}

/// The scores with features' ability bonuses applied, in the guide's terms: fixed and every-score bonuses add
/// first; the highest score is the highest of those (a tie goes to the first in Str, Dex, Con, Int, Wis, Cha
/// order, which the guide doesn't settle); then every score is raised to the highest floor ("at least 14"), and
/// the highest score gets the sum of the highest-score bonuses (Perfect Body, Mind and Soul: +8 in all).
pub fn with_bonuses<'b>(
    base: AbilityScores,
    bonuses: impl IntoIterator<Item = &'b AbilityBonus>,
) -> AbilityScores {
    let mut scores = base;
    let (mut floor, mut highest_bonus) = (0u8, 0u8);
    for bonus in bonuses {
        match *bonus {
            AbilityBonus::Fixed { ability, amount } => {
                let score = scores.get_mut(ability);
                *score = score.saturating_add(amount);
            }
            AbilityBonus::Every { amount } => {
                for ability in AbilityScores::ABILITIES {
                    let score = scores.get_mut(ability);
                    *score = score.saturating_add(amount);
                }
            }
            AbilityBonus::Highest { amount } => {
                highest_bonus = highest_bonus.saturating_add(amount)
            }
            AbilityBonus::AtLeast { score } => floor = floor.max(score),
        }
    }
    // `max_by_key` keeps the last of equal maxima, so walk the abilities backwards to keep the first.
    let highest = AbilityScores::ABILITIES
        .into_iter()
        .rev()
        .max_by_key(|ability| scores.get(*ability));
    for ability in AbilityScores::ABILITIES {
        let score = scores.get_mut(ability);
        *score = (*score).max(floor);
    }
    if let Some(highest) = highest
        && highest_bonus > 0
    {
        let score = scores.get_mut(highest);
        *score = score.saturating_add(highest_bonus);
    }
    scores
}

/// A class's level cap: the length of its table (20 for base classes, 10 for prestige classes). An archetype
/// has its base class's table.
fn class_level_cap(class: &Class) -> u8 {
    u8::try_from(class.levels.len()).unwrap_or(u8::MAX)
}

/// Which spellcasting a prestige class's rows advance: (arcane, divine). Neither for a base class.
fn advanced_spellcasting(class: &Class) -> (bool, bool) {
    let arcane = class.levels.iter().any(|row| row.advances_arcane);
    let divine = class.levels.iter().any(|row| row.advances_divine);
    (arcane, divine)
}

/// How many classes a prestige class may advance: one per spell type its rows advance, or one in all when it
/// advances `Either`.
fn most_advanced(class: &Class) -> usize {
    let (arcane, divine) = advanced_spellcasting(class);
    match class.advances {
        Advancement::Each => usize::from(arcane) + usize::from(divine),
        Advancement::Either => usize::from(arcane || divine),
    }
}
