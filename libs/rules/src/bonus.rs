//! The bonuses in force at a point of a Build (ticket 151): what the caster line and the totals add up. Since ticket 297
//! every one comes from a component ([`Component`](crate::Component)): an entry's the character holds (a feat's Spell
//! Focus, a race's spell DC, a feature's stat bonus to a save or a skill), or a gear item's the Build holds
//! ([`GearStat`](crate::Component::GearStat), ticket 166). The walk records them on the state at the end of each level
//! ([`BonusInForce`]), with each amount worked out there, and the engine sums those that count ([`counted`]: the game's
//! stacking rule, by type, over every source at once; [`Engine::caster_line`], [`Engine::totals`]).
//!
//! `interactions.ron` (ticket 124) names items by [`BonusItem`]: [`ItemInteractions`], what the guide says (or
//! doesn't) about how an item combines with another.
//!
//! [`Engine::caster_line`]: crate::Engine::caster_line
//! [`Engine::totals`]: crate::Engine::totals

use serde::{Deserialize, Serialize};

use crate::component::{Descriptor, Holder, game_stacking};
use crate::game_data::Save;
use crate::keys::{
    BackgroundKey, FeatKey, FeatureKey, GearKey, MythicAbilityKey, MythicFeatKey, RaceKey,
    SelectionOptionKey, SkillKey, SubraceKey,
};
use crate::parameter::{Energy, Parameter, SpellSchool};

/// A bonus in force at a point of a Build: one a component of an entry the character holds gives there (once for each
/// value a parameterised entry is held for), or one of a gear item the Build holds (other than to an ability score,
/// which goes through the scores). The walk records them on the state at the end of each level
/// ([`CharacterState::bonuses_in_force`]), in a fixed order: by what gives them (the race or subrace, the background,
/// feats, mythic feats, mythic abilities, features, options, the other entries, each kind by key), then the gear's in
/// the order the Build holds its items. The caster line and the totals pick out the ones for their numbers and add up
/// those that count ([`counted`]).
///
/// [`CharacterState::bonuses_in_force`]: crate::CharacterState::bonuses_in_force
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BonusInForce {
    /// What gives it.
    pub source: BonusSource,
    /// The value its entry is held for (Spell Focus (Evocation): Evocation); `None` for an entry held for none, and
    /// for gear.
    pub value: Option<Parameter>,
    pub target: InForceTarget,
    /// What it adds there, worked out at that point. Negative for a gear item's penalty.
    pub amount: i16,
    /// Its type, which says whether it counts beside another to the same number ([`counted`]).
    pub descriptor: Descriptor,
    /// The part's name: the entry's, with the value it's held for ("Spell Focus (Evocation)"), a selection option's
    /// with its selection's ("School Power (Evocation)"), an Expanded Arsenal's with the school it adds up ("Expanded
    /// Arsenal (Necromancy): Evocation's Spell Focus feats"). A gear item's is its name.
    pub name: String,
}

/// What gives a [`BonusInForce`]: an entry the character holds, or a gear item the Build holds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BonusSource {
    /// A component of an entry held (ticket 295).
    Component(Holder),
    Gear(GearKey),
}

/// What a [`BonusInForce`] adds to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InForceTarget {
    Number(BonusTarget),
    /// The caster level the caster line reports, and with it caster level checks and concentration: only gear raises
    /// it.
    CasterLevel,
}

impl InForceTarget {
    /// The number it adds to, for those a total adds up.
    pub fn number(&self) -> Option<&BonusTarget> {
        match self {
            Self::Number(target) => Some(target),
            Self::CasterLevel => None,
        }
    }
}

/// Of the bonuses in force that add to one number, those that count, in their order: the game's stacking rule
/// ([`game_stacking`]: every bonus of a type that stacks, the largest of any other type and the lowest penalty), over
/// every source at once, the game keeping one list of modifiers per stat (`ModifiableValue.ModifierList`,
/// `ApplyModifiersFiltered`, WrathDecompiled 6ce4a6b).
pub fn counted<'a>(adding: impl IntoIterator<Item = &'a BonusInForce>) -> Vec<&'a BonusInForce> {
    let adding: Vec<&BonusInForce> = adding.into_iter().collect();
    let amounts: Vec<(Descriptor, i16)> = (adding.iter())
        .map(|bonus| (bonus.descriptor, bonus.amount))
        .collect();
    adding
        .into_iter()
        .zip(game_stacking(&amounts))
        .filter_map(|(bonus, counts)| counts.then_some(bonus))
        .collect()
}

/// The item an `interactions.ron` note is on ([`ItemInteractions`]).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusItem {
    Race(RaceKey),
    Subrace(SubraceKey),
    /// The Build start's background.
    Background(BackgroundKey),
    Feat(FeatKey),
    MythicFeat(MythicFeatKey),
    MythicAbility(MythicAbilityKey),
    /// A class, archetype or path feature.
    Feature(FeatureKey),
    /// A selection option.
    SelectionOption(SelectionOptionKey),
}

/// A number a bonus in force adds to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusTarget {
    /// The save DC of the character's spells of a scope (never [`SpellScope::Value`]: the walk records the value's).
    SpellDc(SpellScope),
    /// Caster level checks to overcome spell resistance.
    SpellPenetration,
    /// Concentration checks.
    Concentration,
    /// Initiative.
    Initiative,
    /// One saving throw.
    Save(Save),
    /// One skill's checks.
    Skill(SkillKey),
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
    /// Whether a bonus of this scope applies to a spell. A bonus in force never has the scope `Value`: the walk records
    /// the school or energy its entry is held for in its place.
    pub fn applies(self, spell: &SpellTraits) -> bool {
        match self {
            Self::All => true,
            Self::School(school) => spell.school == school,
            Self::Subschool(subschool) => spell.subschool == Some(subschool),
            Self::Energy(energy) => spell.energy == Some(energy),
            Self::Metamagic => spell.metamagic,
            Self::Value => false,
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
