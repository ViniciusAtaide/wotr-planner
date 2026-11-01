//! How `rules` values are stored in columns: enums as snake_case text, `u8`/`i8` as smallint.
//!
//! The text values match the domains and CHECK constraints in `migrations/20261031000000_game_data.sql`.

use rules::{
    Ability, Alignment, BabProgression, FeatCategory, MythicAbilityGroup, MythicFeatGroup, Save,
    SaveProgression, SpellType,
};

/// A row that doesn't fit the `rules` types (should be impossible given the constraints).
pub(crate) fn corrupt(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Decode(message.into().into())
}

pub(crate) fn to_u8(value: i16, column: &str) -> Result<u8, sqlx::Error> {
    u8::try_from(value).map_err(|_| corrupt(format!("{column}: {value} is outside 0..=255")))
}

pub(crate) fn to_i8(value: i16, column: &str) -> Result<i8, sqlx::Error> {
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

/// `prerequisite_node.kind`: a group or the kind of a single condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeKind {
    All,
    Any,
    AtLeast,
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
    Alignment,
    Race,
}

impl NodeKind {
    pub(crate) fn is_group(self) -> bool {
        matches!(self, Self::All | Self::Any | Self::AtLeast)
    }
}

text_column!(NodeKind, node_kind_from, node_kind_text {
    All => "all",
    Any => "any",
    AtLeast => "at_least",
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
    Alignment => "alignment",
    Race => "race",
});

/// `feat_slot.kind`: which `rules::FeatSlot` variant a row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FeatSlotKind {
    General,
    CombatFeat,
    Category,
    OneOf,
}

text_column!(FeatSlotKind, feat_slot_kind_from, feat_slot_kind_text {
    General => "general",
    CombatFeat => "combat_feat",
    Category => "category",
    OneOf => "one_of",
});
