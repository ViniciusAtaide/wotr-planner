//! How `rules` values are stored in columns: enums as snake_case text, `u8`/`i8` as INTEGER.
//!
//! The text values match the named column types and CHECK constraints in `migrations/` (the first,
//! `20261031000000_game_data.sql`, and the ones after it).

use rules::{
    Ability, Advancement, Alignment, ArmourType, BabProgression, BackgroundGroup, CastingTime,
    DeityDomains, Energy, FeatCategory, Handedness, LevelFrom, MythicAbilityGroup, MythicExtraPick,
    MythicFeatGroup, OptionGroup, ParameterKind, Ranged, Repeat, Save, SaveProgression,
    SpellDescriptor, SpellSchool, SpellType, Story, WeaponProficiency,
};

/// A row that doesn't fit the `rules` types (should be impossible given the constraints).
pub(crate) fn corrupt(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Decode(message.into().into())
}

pub(crate) fn to_u8(value: i64, column: &str) -> Result<u8, sqlx::Error> {
    u8::try_from(value).map_err(|_| corrupt(format!("{column}: {value} is outside 0..=255")))
}

pub(crate) fn to_i8(value: i64, column: &str) -> Result<i8, sqlx::Error> {
    i8::try_from(value).map_err(|_| corrupt(format!("{column}: {value} is outside -128..=127")))
}

/// A two-way mapping between an enum and its column text.
macro_rules! text_column {
    ($type:ty, $from:ident, $to:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        pub(crate) fn $to(value: $type) -> &'static str {
            match value {
                $(<$type>::$variant => $text,)+
            }
        }

        pub(crate) fn $from(text: &str) -> Result<$type, sqlx::Error> {
            match text {
                $($text => Ok(<$type>::$variant),)+
                other => Err(corrupt(format!(
                    "{other:?} is not a {}",
                    stringify!($type)
                ))),
            }
        }
    };
}

text_column!(Ability, ability_from, ability_text {
    Strength => "strength",
    Dexterity => "dexterity",
    Constitution => "constitution",
    Intelligence => "intelligence",
    Wisdom => "wisdom",
    Charisma => "charisma",
});

text_column!(SpellType, spell_type_from, spell_type_text {
    Arcane => "arcane",
    Divine => "divine",
});

text_column!(Alignment, alignment_from, alignment_text {
    LawfulGood => "lawful_good",
    NeutralGood => "neutral_good",
    ChaoticGood => "chaotic_good",
    LawfulNeutral => "lawful_neutral",
    TrueNeutral => "true_neutral",
    ChaoticNeutral => "chaotic_neutral",
    LawfulEvil => "lawful_evil",
    NeutralEvil => "neutral_evil",
    ChaoticEvil => "chaotic_evil",
});

text_column!(BabProgression, bab_progression_from, bab_progression_text {
    Full => "full",
    ThreeQuarters => "three_quarters",
    Half => "half",
});

text_column!(SaveProgression, save_progression_from, save_progression_text {
    Good => "good",
    Poor => "poor",
});

text_column!(Save, save_from, save_text {
    Fortitude => "fortitude",
    Reflex => "reflex",
    Will => "will",
});

text_column!(FeatCategory, feat_category_from, feat_category_text {
    Weapon => "weapon",
    Ranged => "ranged",
    UnarmedCombat => "unarmed_combat",
    ArmourAndShield => "armour_and_shield",
    Magic => "magic",
    Miscellaneous => "miscellaneous",
    MountedCombat => "mounted_combat",
    CombatManoeuvres => "combat_manoeuvres",
    Teamwork => "teamwork",
    Skill => "skill",
    Class => "class",
    Racial => "racial",
});

text_column!(MythicAbilityGroup, mythic_ability_group_from, mythic_ability_group_text {
    Melee => "melee",
    Ranged => "ranged",
    Magic => "magic",
    Healing => "healing",
    Miscellaneous => "miscellaneous",
    Class => "class",
});

text_column!(MythicFeatGroup, mythic_feat_group_from, mythic_feat_group_text {
    Melee => "melee",
    Ranged => "ranged",
    ArmourAndShield => "armour_and_shield",
    Magic => "magic",
    Miscellaneous => "miscellaneous",
    CombatManoeuvre => "combat_manoeuvre",
});

text_column!(Story, story_from, story_text {
    VowOfDahak => "vow_of_dahak",
    Uncorrupted => "uncorrupted",
});

text_column!(OptionGroup, option_group_from, option_group_text {
    Draconic => "draconic",
    Elemental => "elemental",
    BloodragerDraconic => "bloodrager_draconic",
    BloodragerElemental => "bloodrager_elemental",
});

text_column!(BackgroundGroup, background_group_from, background_group_text {
    Craftsman => "craftsman",
    Noble => "noble",
    Oblate => "oblate",
    Regional => "regional",
    Scholar => "scholar",
    StreetUrchin => "street_urchin",
    Wanderer => "wanderer",
    Warrior => "warrior",
});

text_column!(LevelFrom, level_from_from, level_from_text {
    OtherOptions => "other_options",
    GrantingOption => "granting_option",
    MythicRanks => "mythic_ranks",
});

text_column!(DeityDomains, deity_domains_from, deity_domains_text {
    Offered => "offered",
    NotOffered => "not_offered",
});

text_column!(MythicExtraPick, mythic_extra_pick_from, mythic_extra_pick_text {
    Feat => "feat",
    MythicAbility => "mythic_ability",
});

text_column!(Advancement, advancement_from, advancement_text {
    Each => "each",
    Either => "either",
});

text_column!(WeaponProficiency, weapon_proficiency_from, weapon_proficiency_text {
    Simple => "simple",
    Martial => "martial",
    Exotic => "exotic",
});

text_column!(Handedness, handedness_from, handedness_text {
    Light => "light",
    OneHanded => "one_handed",
    TwoHanded => "two_handed",
});

text_column!(Ranged, ranged_from, ranged_text {
    Thrown => "thrown",
    Projectile => "projectile",
    ProjectileWithStrength => "projectile_with_strength",
});

text_column!(ParameterKind, parameter_kind_from, parameter_kind_text {
    Skill => "skill",
    Weapon => "weapon",
    ExoticWeapon => "exotic_weapon",
    SpellSchool => "spell_school",
    Energy => "energy",
    ArmourType => "armour_type",
});

text_column!(CastingTime, casting_time_from, casting_time_text {
    Standard => "standard",
    FullRound => "full_round",
    Move => "move",
    Swift => "swift",
    Free => "free",
});

text_column!(SpellDescriptor, spell_descriptor_from, spell_descriptor_text {
    Acid => "acid",
    ChannelNegativeEnergyHarm => "channel_negative_energy_harm",
    Chaos => "chaos",
    Charmed => "charmed",
    Cold => "cold",
    Compulsion => "compulsion",
    Cure => "cure",
    Curse => "curse",
    Death => "death",
    Disease => "disease",
    Divine => "divine",
    Electricity => "electricity",
    Emotion => "emotion",
    Evil => "evil",
    Fear => "fear",
    Fire => "fire",
    Force => "force",
    Good => "good",
    Law => "law",
    MindAffecting => "mind_affecting",
    MovementImpairing => "movement_impairing",
    NegativeEmotion => "negative_emotion",
    NegativeLevel => "negative_level",
    Poison => "poison",
    Polymorph => "polymorph",
    Sight => "sight",
    Sleep => "sleep",
    Sonic => "sonic",
    Stunned => "stunned",
    Summoning => "summoning",
});

text_column!(SpellSchool, spell_school_from, spell_school_text {
    Abjuration => "abjuration",
    Conjuration => "conjuration",
    Divination => "divination",
    Enchantment => "enchantment",
    Evocation => "evocation",
    Illusion => "illusion",
    Necromancy => "necromancy",
    Transmutation => "transmutation",
});

text_column!(Energy, energy_from, energy_text {
    Acid => "acid",
    Cold => "cold",
    Electricity => "electricity",
    Fire => "fire",
});

text_column!(ArmourType, armour_type_from, armour_type_text {
    Light => "light",
    Medium => "medium",
    Heavy => "heavy",
});

/// A [`Repeat`] as its `repeat` and `repeat_times` columns.
pub(crate) fn repeat_columns(repeat: Repeat) -> (&'static str, Option<i16>) {
    match repeat {
        Repeat::Once => ("once", None),
        Repeat::Times(times) => ("times", Some(times.into())),
        Repeat::Unlimited => ("unlimited", None),
        Repeat::PerValue => ("per_value", None),
    }
}

/// The [`Repeat`] in a `repeat` and a `repeat_times` column.
pub(crate) fn repeat_from(repeat: &str, times: Option<i64>) -> Result<Repeat, sqlx::Error> {
    Ok(match (repeat, times) {
        ("once", None) => Repeat::Once,
        ("times", Some(times)) => Repeat::Times(to_u8(times, "repeat_times")?),
        ("unlimited", None) => Repeat::Unlimited,
        ("per_value", None) => Repeat::PerValue,
        (other, times) => {
            return Err(corrupt(format!(
                "repeat {other:?} with repeat_times {times:?}"
            )));
        }
    })
}

/// The value columns of a `rules::Parameter` (`skill_key`, `weapon_key`, `spell_school`, `energy`,
/// `armour_type`): exactly one is set.
#[derive(Default)]
pub(crate) struct ValueColumns<'a> {
    pub(crate) skill_key: Option<&'a str>,
    pub(crate) weapon_key: Option<&'a str>,
    pub(crate) spell_school: Option<&'static str>,
    pub(crate) energy: Option<&'static str>,
    pub(crate) armour_type: Option<&'static str>,
}

impl<'a> ValueColumns<'a> {
    pub(crate) fn of(value: &'a rules::Parameter) -> Self {
        use rules::Parameter as P;
        match value {
            P::Skill(skill) => Self {
                skill_key: Some(skill.as_str()),
                ..Self::default()
            },
            P::Weapon(weapon) => Self {
                weapon_key: Some(weapon.as_str()),
                ..Self::default()
            },
            P::SpellSchool(school) => Self {
                spell_school: Some(spell_school_text(*school)),
                ..Self::default()
            },
            P::Energy(energy) => Self {
                energy: Some(energy_text(*energy)),
                ..Self::default()
            },
            P::ArmourType(armour) => Self {
                armour_type: Some(armour_type_text(*armour)),
                ..Self::default()
            },
        }
    }
}

/// The value the value columns of a row hold: whichever is set, or `None`.
pub(crate) fn value_from(
    skill_key: Option<String>,
    weapon_key: Option<String>,
    spell_school: Option<String>,
    energy: Option<String>,
    armour_type: Option<String>,
) -> Result<Option<rules::Parameter>, sqlx::Error> {
    use rules::Parameter as P;
    Ok(
        match (skill_key, weapon_key, spell_school, energy, armour_type) {
            (None, None, None, None, None) => None,
            (Some(skill), None, None, None, None) => Some(P::Skill(rules::SkillKey(skill))),
            (None, Some(weapon), None, None, None) => Some(P::Weapon(rules::WeaponKey(weapon))),
            (None, None, Some(school), None, None) => {
                Some(P::SpellSchool(spell_school_from(&school)?))
            }
            (None, None, None, Some(energy), None) => Some(P::Energy(energy_from(&energy)?)),
            (None, None, None, None, Some(armour)) => {
                Some(P::ArmourType(armour_type_from(&armour)?))
            }
            _ => return Err(corrupt("a row sets two value columns".to_owned())),
        },
    )
}

/// `prerequisite_node.kind`: a group or the kind of a single condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeKind {
    All,
    Any,
    AtLeast,
    Not,
    Feat,
    MythicFeat,
    MythicAbility,
    ClassLevel,
    Archetype,
    Feature,
    SelectionOption,
    SkillRanks,
    AbilityScore,
    BaseAttackBonus,
    SpellLevel,
    CasterLevel,
    CharacterLevel,
    MythicRank,
    Alignment,
    Race,
    FeatFor,
    FeatForSameValue,
    FeatIfValue,
    Story,
    NoOptionFrom,
    SpontaneousSpellLevel,
    NoOtherOptionOfGroup,
    PickedThrough,
    Deity,
    Atheist,
    Worships,
    DeityOffers,
}

impl NodeKind {
    pub(crate) fn is_group(self) -> bool {
        matches!(self, Self::All | Self::Any | Self::AtLeast | Self::Not)
    }
}

text_column!(NodeKind, node_kind_from, node_kind_text {
    All => "all",
    Any => "any",
    AtLeast => "at_least",
    Not => "not",
    Feat => "feat",
    MythicFeat => "mythic_feat",
    MythicAbility => "mythic_ability",
    ClassLevel => "class_level",
    Archetype => "archetype",
    Feature => "feature",
    SelectionOption => "selection_option",
    SkillRanks => "skill_ranks",
    AbilityScore => "ability_score",
    BaseAttackBonus => "base_attack_bonus",
    SpellLevel => "spell_level",
    CasterLevel => "caster_level",
    CharacterLevel => "character_level",
    MythicRank => "mythic_rank",
    Alignment => "alignment",
    Race => "race",
    FeatFor => "feat_for",
    FeatForSameValue => "feat_for_same_value",
    FeatIfValue => "feat_if_value",
    Story => "story",
    NoOptionFrom => "no_option_from",
    SpontaneousSpellLevel => "spontaneous_spell_level",
    NoOtherOptionOfGroup => "no_other_option_of_group",
    PickedThrough => "picked_through",
    Deity => "deity",
    Atheist => "atheist",
    Worships => "worships",
    DeityOffers => "deity_offers",
});

/// `feat_slot.kind`: which `rules::FeatSlot` variant a row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FeatSlotKind {
    General,
    CombatFeat,
    Category,
    OneOf,
    OptionList,
}

text_column!(FeatSlotKind, feat_slot_kind_from, feat_slot_kind_text {
    General => "general",
    CombatFeat => "combat_feat",
    Category => "category",
    OneOf => "one_of",
    OptionList => "option_list",
});

/// `feature_ability_bonus.kind`: which `rules::AbilityBonus` variant a row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AbilityBonusKind {
    Fixed,
    Every,
    Highest,
    AtLeast,
}

text_column!(AbilityBonusKind, ability_bonus_kind_from, ability_bonus_kind_text {
    Fixed => "fixed",
    Every => "every",
    Highest => "highest",
    AtLeast => "at_least",
});
