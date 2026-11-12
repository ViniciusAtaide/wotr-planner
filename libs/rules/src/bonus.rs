//! Flat bonuses that items give while a character holds them: the save DC of spells, caster level against spell
//! resistance and concentration (ticket 119), initiative, the saves and the skills (ticket 120). `bonuses.ron` lists
//! them, one [`ItemBonus`] per bonus, each naming the item that gives it. The walk records the ones a character
//! holds on its state, each a [`BonusInForce`] (ticket 151), with the bonuses of the gear the Build holds (ticket
//! 166), and the engine sums those that count ([`counted`]; [`Engine::caster_line`], [`Engine::totals`]). A bonus
//! has a [`BonusType`], and the stacking rule ([`stacking`]) says which of several to one number count.
//!
//! The shape is meant to grow: a new kind of number is a new [`BonusTarget`], a new kind of giver a new
//! [`BonusItem`], and the amount, the condition and the item stay as they are.
//!
//! `interactions.ron` (ticket 124) names items the same way: [`ItemInteractions`], what the guide says (or doesn't)
//! about how an item combines with another.
//!
//! [`Engine::caster_line`]: crate::Engine::caster_line
//! [`Engine::totals`]: crate::Engine::totals

use serde::{Deserialize, Serialize};

use crate::game_data::{Ability, Save};
use crate::keys::{
    BackgroundKey, FeatKey, FeatureKey, GearKey, MythicAbilityKey, MythicFeatKey, RaceKey,
    SelectionOptionKey, SkillKey, SubraceKey,
};
use crate::parameter::{Energy, Parameter, SpellSchool, Stat};
use crate::prerequisite::Prerequisite;

/// One flat bonus an item gives while it's held: to what, how much, and on what else it depends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemBonus {
    /// The item that gives it.
    pub item: BonusItem,
    /// What it adds to.
    pub target: BonusTarget,
    pub amount: BonusAmount,
    /// What the character must hold besides the item for it to count, checked like a prerequisite (Spell
    /// Penetration (Mythic)'s full Mythic rank: Greater Spell Penetration). `None`: the item alone.
    #[serde(default)]
    pub requires: Option<Prerequisite>,
}

/// A bonus in force at a point of a Build: an [`ItemBonus`] the character holds there, whose `requires` it meets,
/// once for each value its item is held for, with its amount worked out at that point; or a bonus of a gear item the
/// Build holds (ticket 166), other than one to an ability score, which goes through the scores. The walk records
/// them on the state at the end of each level ([`CharacterState::bonuses_in_force`]): the Item bonuses in the data's
/// order, then gear's in the order the Build holds its items. The caster line and the totals pick out the ones for
/// their numbers and add up those that count ([`counted`]).
///
/// [`CharacterState::bonuses_in_force`]: crate::CharacterState::bonuses_in_force
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BonusInForce {
    /// What gives it.
    pub source: BonusSource,
    /// The value its item is held for (Spell Focus (Evocation): Evocation); `None` for an item taken for none, and
    /// for gear.
    pub value: Option<Parameter>,
    pub target: InForceTarget,
    /// What it adds there: its [`BonusAmount`] worked out from the Mythic rank, the character's ability scores
    /// and the ranks of the skill it's for. Expanded Arsenal's is the copy of the other school's Spell Focus feats.
    /// Negative for a gear item's penalty.
    pub amount: i16,
    /// Its type, which says whether it counts beside another to the same number ([`counted`]). Always
    /// [`BonusType::Untyped`] for an Item bonus, until ticket 144 gives those their types.
    pub kind: BonusType,
    /// The part's name: the item's, with the value it's held for ("Spell Focus (Evocation)"), a selection option's
    /// with its selection's ("School Power (Evocation)"), and for Expanded Arsenal the school copied ("Expanded
    /// Arsenal (Necromancy): Evocation's Spell Focus feats"). A gear item's is its name.
    pub name: String,
}

/// What gives a [`BonusInForce`]: an item of `bonuses.ron`, or a gear item the Build holds. Gear is no
/// [`BonusItem`]: that one is what the data's tables name a bonus's item by, and no bonus of `bonuses.ron` is a gear
/// item's.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BonusSource {
    Item(BonusItem),
    Gear(GearKey),
}

/// What a [`BonusInForce`] adds to: a number an Item bonus can add to, or the caster level, which only gear raises
/// ([`GearTarget::CasterLevel`](crate::GearTarget::CasterLevel)).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InForceTarget {
    Number(BonusTarget),
    /// The caster level the caster line reports, and with it caster level checks and concentration.
    CasterLevel,
}

impl InForceTarget {
    /// The number it adds to; `None` for the caster level.
    pub fn number(&self) -> Option<&BonusTarget> {
        match self {
            Self::Number(target) => Some(target),
            Self::CasterLevel => None,
        }
    }
}

impl BonusType {
    /// Whether two bonuses of this type to one number both count: only an untyped one's do. Of any other type the
    /// highest counts, an inherent one included (the user, 2026-11-11: "inherent bonuses don't stack, keep highest
    /// only"), so the Tome of Ancient Perfection's +4 hides a +2 manual.
    pub fn stacks(self) -> bool {
        self == Self::Untyped
    }
}

/// The stacking rule (ticket 144's, applied to gear first: ticket 166), for typed amounts that add to one number:
/// which of them count, in step with `bonuses`. A penalty (a negative amount) always counts; so does every bonus of
/// a type that stacks ([`BonusType::stacks`]); of any other type only the highest counts, the first of equals.
/// Bonuses of different types all count, so they add.
pub fn stacking(bonuses: &[(BonusType, i16)]) -> Vec<bool> {
    bonuses
        .iter()
        .enumerate()
        .map(|(at, &(kind, amount))| {
            amount < 0
                || kind.stacks()
                || !bonuses.iter().enumerate().any(|(other, &(of, more))| {
                    of == kind && (more > amount || (more == amount && other < at))
                })
        })
        .collect()
}

/// What typed amounts to one number add up to under the stacking rule ([`stacking`]).
pub fn stacked(bonuses: &[(BonusType, i16)]) -> i16 {
    let counts = stacking(bonuses);
    bonuses
        .iter()
        .zip(counts)
        .filter(|(_, counts)| *counts)
        .map(|((_, amount), _)| amount)
        .sum()
}

/// Of the bonuses in force that add to one number, those that count, in their order: every Item bonus (a feat's or
/// feature's still adds as it always did, until ticket 144), and gear's under the stacking rule among themselves
/// ([`stacking`]): two belts' worth of enhancement count as the higher one.
pub fn counted<'a>(adding: impl IntoIterator<Item = &'a BonusInForce>) -> Vec<&'a BonusInForce> {
    let adding: Vec<&BonusInForce> = adding.into_iter().collect();
    let gear: Vec<(BonusType, i16)> = adding
        .iter()
        .filter(|bonus| matches!(bonus.source, BonusSource::Gear(_)))
        .map(|bonus| (bonus.kind, bonus.amount))
        .collect();
    if gear.is_empty() {
        return adding;
    }
    let mut counts = stacking(&gear).into_iter();
    adding
        .into_iter()
        .filter(|bonus| match bonus.source {
            BonusSource::Item(_) => true,
            BonusSource::Gear(_) => counts.next().unwrap_or(true),
        })
        .collect()
}

/// The item that gives a bonus (or that an [`ItemInteractions`] note is on). A race's bonus counts for a character of
/// that race without a subrace, a subrace's for one of that subrace: the guide lists a subrace's racial traits in
/// full, so they replace the race's, as its ability modifiers do.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusItem {
    Race(RaceKey),
    Subrace(SubraceKey),
    /// The Build start's background (the user's list: the Pickpocket's +2 initiative).
    Background(BackgroundKey),
    /// A feat, held however (picked or granted). A parameterised one's bonus counts for each value it's held for.
    Feat(FeatKey),
    /// A mythic feat; a parameterised one's counts for each value, as a feat's.
    MythicFeat(MythicFeatKey),
    MythicAbility(MythicAbilityKey),
    /// A class, archetype or path feature, held however it's granted (a bloodline's arcana through Second
    /// Bloodline or Crossblooded too).
    Feature(FeatureKey),
    /// A selection option held (School Power's Enchantment).
    SelectionOption(SelectionOptionKey),
}

/// A [`BonusItem`]'s kind, without its key. With the key's text it's what the engine indexes each item's bonuses
/// by ([`BonusItem::kind_and_key`]), so the walk finds the bonuses of what it holds without building a `BonusItem`
/// (a copy of the key) for each item it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BonusItemKind {
    Race,
    Subrace,
    Background,
    Feat,
    MythicFeat,
    MythicAbility,
    Feature,
    SelectionOption,
}

impl BonusItem {
    /// The item's kind and its key's text.
    pub(crate) fn kind_and_key(&self) -> (BonusItemKind, &str) {
        match self {
            Self::Race(key) => (BonusItemKind::Race, key.as_str()),
            Self::Subrace(key) => (BonusItemKind::Subrace, key.as_str()),
            Self::Background(key) => (BonusItemKind::Background, key.as_str()),
            Self::Feat(key) => (BonusItemKind::Feat, key.as_str()),
            Self::MythicFeat(key) => (BonusItemKind::MythicFeat, key.as_str()),
            Self::MythicAbility(key) => (BonusItemKind::MythicAbility, key.as_str()),
            Self::Feature(key) => (BonusItemKind::Feature, key.as_str()),
            Self::SelectionOption(key) => (BonusItemKind::SelectionOption, key.as_str()),
        }
    }
}

/// What a bonus adds to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusTarget {
    /// The save DC of the character's spells of a scope.
    SpellDc(SpellScope),
    /// Caster level checks to overcome spell resistance.
    SpellPenetration,
    /// Concentration checks.
    Concentration,
    /// Initiative.
    Initiative,
    /// One saving throw.
    Save(Save),
    /// Every saving throw (Halfling Luck, Mythic Resolve).
    AllSaves,
    /// One skill's checks.
    Skill(SkillKey),
    /// Every skill's checks (Creative Approach).
    AllSkills,
    /// What the item is held for: the skill of Skill Focus (Persuasion), or the stat of Stat Focus (Initiative) when
    /// it's initiative or a save (Stat Focus's other stats are no number the planner sums). Only on a feat or mythic
    /// feat taken for a skill or a stat.
    Held,
}

impl BonusTarget {
    /// Whether a bonus of this target from an item held for `value` adds to a skill's checks. A target of `Held`
    /// needs the skill as the value.
    pub fn adds_to_skill(&self, skill: &SkillKey, value: Option<&Parameter>) -> bool {
        match self {
            Self::Skill(own) => own == skill,
            Self::AllSkills => true,
            Self::Held => value == Some(&Parameter::Skill(skill.clone())),
            _ => false,
        }
    }

    /// Whether a bonus of this target from an item held for `value` adds to a save.
    pub fn adds_to_save(&self, save: Save, value: Option<&Parameter>) -> bool {
        let stat = match save {
            Save::Fortitude => Stat::Fortitude,
            Save::Reflex => Stat::Reflex,
            Save::Will => Stat::Will,
        };
        match self {
            Self::Save(own) => *own == save,
            Self::AllSaves => true,
            Self::Held => value == Some(&Parameter::Stat(stat)),
            _ => false,
        }
    }

    /// Whether a bonus of this target from an item held for `value` adds to initiative.
    pub fn adds_to_initiative(&self, value: Option<&Parameter>) -> bool {
        match self {
            Self::Initiative => true,
            Self::Held => value == Some(&Parameter::Stat(Stat::Initiative)),
            _ => false,
        }
    }
}

/// Which spells a spell DC bonus applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellScope {
    /// Every spell.
    All,
    School(SpellSchool),
    Subschool(Subschool),
    /// Spells of an energy type.
    Energy(Energy),
    /// Spells cast with a metamagic feat other than Heighten Spell (the Arcane bloodline's arcana).
    Metamagic,
    /// The school or energy type the item is held for: Spell Focus (Enchantment), Elemental Focus (Fire). Only on a
    /// feat or mythic feat taken for a school or an energy type.
    Value,
}

impl SpellScope {
    /// Whether a bonus of this scope from an item held for `value` applies to a spell. A scope of `Value` needs the
    /// value; any other ignores it.
    pub fn applies(self, spell: &SpellTraits, value: Option<&Parameter>) -> bool {
        match self {
            Self::All => true,
            Self::School(school) => spell.school == school,
            Self::Subschool(subschool) => spell.subschool == Some(subschool),
            Self::Energy(energy) => spell.energy == Some(energy),
            Self::Metamagic => spell.metamagic,
            Self::Value => match value {
                Some(Parameter::SpellSchool(school)) => spell.school == *school,
                Some(Parameter::Energy(energy)) => spell.energy == Some(*energy),
                _ => false,
            },
        }
    }
}

/// How much a bonus adds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusAmount {
    /// A fixed number (+1, +2, +4).
    Flat(i8),
    /// Half the Mythic rank, rounded down.
    HalfMythicRank,
    /// The Mythic rank.
    MythicRank,
    /// Expanded Arsenal: the school it's held for gets the bonuses the Spell Focus feats (the feats and mythic feats
    /// whose spell DC bonus is for the school they're held for) give one other school, the one where they add up to
    /// the most; nothing when it has Spell Focus feats of its own (the guide: "this has no effect"). Only with
    /// [`SpellScope::Value`].
    SpellFocusOfAnotherSchool,
    /// A skill bonus that rises with 10 or more ranks in the skill (Skill Focus: +3, +6 with 10 ranks; Persuasive:
    /// +2, +4). Only on one skill's target: [`BonusTarget::Skill`] or a skill [`BonusTarget::Held`].
    TenRanks { amount: i8, with_ten_ranks: i8 },
    /// An ability's modifier, from the character's own score (Divine Grace: Charisma to every save).
    AbilityModifier(Ability),
}

/// A bonus's type, as the guide words it. Two bonuses of one type to one number don't stack: the higher counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusType {
    Alchemical,
    Armour,
    Circumstance,
    Competence,
    Deflection,
    Dodge,
    Enhancement,
    Inherent,
    Insight,
    Luck,
    Morale,
    NaturalArmour,
    Profane,
    Racial,
    Resistance,
    Sacred,
    Shield,
    Size,
    /// The guide gives no type, or says "untyped". Always adds. Every penalty is untyped.
    Untyped,
}

impl BonusType {
    pub const ALL: [Self; 19] = [
        Self::Alchemical,
        Self::Armour,
        Self::Circumstance,
        Self::Competence,
        Self::Deflection,
        Self::Dodge,
        Self::Enhancement,
        Self::Inherent,
        Self::Insight,
        Self::Luck,
        Self::Morale,
        Self::NaturalArmour,
        Self::Profane,
        Self::Racial,
        Self::Resistance,
        Self::Sacred,
        Self::Shield,
        Self::Size,
        Self::Untyped,
    ];

    /// Lower case, as a summary prints it ("enhancement", "natural armour"); "" for [`Untyped`](Self::Untyped).
    pub fn name(self) -> &'static str {
        match self {
            Self::Alchemical => "alchemical",
            Self::Armour => "armour",
            Self::Circumstance => "circumstance",
            Self::Competence => "competence",
            Self::Deflection => "deflection",
            Self::Dodge => "dodge",
            Self::Enhancement => "enhancement",
            Self::Inherent => "inherent",
            Self::Insight => "insight",
            Self::Luck => "luck",
            Self::Morale => "morale",
            Self::NaturalArmour => "natural armour",
            Self::Profane => "profane",
            Self::Racial => "racial",
            Self::Resistance => "resistance",
            Self::Sacred => "sacred",
            Self::Shield => "shield",
            Self::Size => "size",
            Self::Untyped => "",
        }
    }
}

/// What the guide says about how an item combines with another, in the planner's own words (ticket 124): only for
/// items an agent run showed two summaries weren't enough for, and "the guide doesn't say" where it doesn't.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemInteractions {
    pub item: BonusItem,
    pub interactions: String,
}

/// What a spell is, as far as the DC bonuses look: a school, and optionally a subschool, an energy type and whether
/// a metamagic feat (other than Heighten Spell) is applied. No spell catalogue is needed: the caster line takes these
/// as given.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpellTraits {
    pub school: SpellSchool,
    pub subschool: Option<Subschool>,
    pub energy: Option<Energy>,
    pub metamagic: bool,
}

/// The subschools of magic, as the core rules name them. Only charm and compulsion change a DC in the data today
/// (the Infernal and Fey arcana); the rest are here so a caster line can name any spell's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Subschool {
    Calling,
    Creation,
    Healing,
    Summoning,
    Teleportation,
    Scrying,
    Charm,
    Compulsion,
    Figment,
    Glamer,
    Pattern,
    Phantasm,
    Shadow,
    Polymorph,
}

impl Subschool {
    pub const ALL: [Self; 14] = [
        Self::Calling,
        Self::Creation,
        Self::Healing,
        Self::Summoning,
        Self::Teleportation,
        Self::Scrying,
        Self::Charm,
        Self::Compulsion,
        Self::Figment,
        Self::Glamer,
        Self::Pattern,
        Self::Phantasm,
        Self::Shadow,
        Self::Polymorph,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Calling => "Calling",
            Self::Creation => "Creation",
            Self::Healing => "Healing",
            Self::Summoning => "Summoning",
            Self::Teleportation => "Teleportation",
            Self::Scrying => "Scrying",
            Self::Charm => "Charm",
            Self::Compulsion => "Compulsion",
            Self::Figment => "Figment",
            Self::Glamer => "Glamer",
            Self::Pattern => "Pattern",
            Self::Phantasm => "Phantasm",
            Self::Shadow => "Shadow",
            Self::Polymorph => "Polymorph",
        }
    }

    /// The school it belongs to.
    pub fn school(self) -> SpellSchool {
        match self {
            Self::Calling
            | Self::Creation
            | Self::Healing
            | Self::Summoning
            | Self::Teleportation => SpellSchool::Conjuration,
            Self::Scrying => SpellSchool::Divination,
            Self::Charm | Self::Compulsion => SpellSchool::Enchantment,
            Self::Figment | Self::Glamer | Self::Pattern | Self::Phantasm | Self::Shadow => {
                SpellSchool::Illusion
            }
            Self::Polymorph => SpellSchool::Transmutation,
        }
    }
}
