//! The component vocabulary (ticket 295): the game builds its features from a small set of generic components wired
//! together in blueprints, and an entry of the data may carry them in our own form, beside its hand-shaped fields.
//!
//! Each kind is named for what it does and holds keys, never GUIDs. Its doc names the game component it stands for
//! and the code that gives it its meaning, by file and method at WrathDecompiled 6ce4a6b (`Kingmaker/…` under
//! `Assembly-CSharp`). The engine reads them in the structures it already has:
//! - [`Requires`](Component::Requires) compiles at load into the entry's [`Prerequisite`] tree ([`compile_requires`]):
//!   the checked data holds that tree in place of them ([`GameData::checked`](crate::GameData::checked)), so the walk,
//!   the pickers, the reasons, `allows` and every other reader of a prerequisite read one form;
//! - the grants hold their facts through the walk's holds, as a level row's do;
//! - the stat bonuses feed the ability scores, the base attack bonus and the bonuses in force
//!   ([`BonusInForce`](crate::BonusInForce)), with their values worked out from the state at that level ([`Value`]);
//!   every source's bonuses to one stat stack together by type, a gear item's ([`GearStat`](Component::GearStat))
//!   included ([`game_stacking`]);
//! - the caster line's kinds (a spell DC, Spell Focus and Expanded Arsenal, spell penetration, concentration, Heighten
//!   Spell) join the bonuses in force, and the attack line reads the kinds of the feats held for its weapon (ticket
//!   297: the numbers the planner held in hand-shaped fields and named feats);
//! - the prerequisite modifiers ([`PrerequisiteClassLevels`](Component::PrerequisiteClassLevels),
//!   [`PrerequisiteStat`](Component::PrerequisiteStat)) change what a prerequisite reads off the state.
//!
//! [`GameData::check`](crate::GameData::check) refuses an entry that holds a hand field and the component kind that
//! replaces it, and a component on an entry it has no meaning on.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::bonus::SpellScope;
use crate::build::AbilityScores;
use crate::game_data::{Ability, Alignment, GameData, Save, SpellType, WeaponProficiency};
use crate::gear::GearTarget;
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, RaceKey, SelectionOptionKey, SkillKey, SubraceKey, WeaponKey,
};
use crate::parameter::Parameter;
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

/// One component of an entry: what the entry does, in the game's terms and our own words.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Component {
    /// A prerequisite: the game's `Prerequisite*` family. The entry may be picked when every condition of the `All`
    /// group holds and, when the `Any` group has any, one of them does (`Blueprints/Classes/BlueprintFeature.cs`
    /// `MeetsPrerequisites`; each condition's `CheckInternal`). Compiled into the entry's [`Prerequisite`].
    Requires {
        condition: Condition,
        #[serde(default, skip_serializing_if = "Group::is_all")]
        group: Group,
    },
    /// While held, the class level a `ClassLevel` prerequisite reads of `class` counts `share` of the levels of
    /// `from` and `plus` more, for the picks of `only_for` alone when it is set: the game's `ClassLevelsForPrerequisites`
    /// (`Designers/Mechanics/Facts/ClassLevelsForPrerequisites.cs`), read by
    /// `Blueprints/Classes/Prerequisites/PrerequisiteClassLevel.cs` `GetClassLevel`, which adds
    /// `(int)(Modifier × level + Summand)` (Eldritch Scion's Fighter Training: half the magus levels as fighter levels
    /// for feats).
    PrerequisiteClassLevels {
        class: ClassKey,
        from: ClassKey,
        #[serde(default, skip_serializing_if = "Share::is_full")]
        share: Share,
        #[serde(default, skip_serializing_if = "is_zero_i8")]
        plus: i8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        only_for: Option<Pool>,
    },
    /// While held, a prerequisite on `stat` reads it by `rule`: the game's `ReplaceStatForPrerequisites`
    /// (`Designers/Mechanics/Facts/ReplaceStatForPrerequisites.cs` `ResultStat`, read by
    /// `PrerequisiteStatValue.GetStatValue`; the Kinetic Warrior's Constitution for Intelligence).
    PrerequisiteStat { stat: StatRef, rule: StatRule },
    /// On a selection: no pick is offered while one of `facts` is held, or with `any_option`, one of the selection's
    /// own options: the game's `NoSelectionIfAlreadyHasFeature` (`Blueprints/Classes/Spells/…`), read by
    /// `Blueprints/Classes/Selection/BlueprintFeatureSelection.cs` `IsSelectionProhibited`.
    NoSelectionIf {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        facts: BTreeSet<Fact>,
        #[serde(default, skip_serializing_if = "is_false")]
        any_option: bool,
    },
    /// While held, these facts are held too: the game's `AddFacts` (`UnitLogic/FactLogic/AddFacts.cs` `OnActivate`,
    /// `UpdateFacts`; `OnDeactivate` clears them). A granted selection is held and offers no pick: the game offers one
    /// only for a progression's row (`LevelUpHelper.AddFeaturesFromProgression`) or a selection picked as another's
    /// option (`SelectFeature.Apply`), never for `AddFacts`.
    Grant(BTreeSet<Fact>),
    /// While held, `fact` is held from `level` of the classes on (or, `before`, only below it): the game's
    /// `AddFeatureOnClassLevel` (`Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs` `Apply`,
    /// `IsFeatureShouldBeApplied`), applied on activation and on every level gained. The level is the classes' levels
    /// added up, a class counting only taken with one of `archetypes` when they are its own
    /// (`ReplaceCasterLevelOfAbility.CalculateClassLevel`, `CheckArchetype`); no classes: the character level.
    GrantAtClassLevel {
        fact: Fact,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        classes: BTreeSet<ClassKey>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        archetypes: BTreeSet<ArchetypeKey>,
        level: u8,
        #[serde(default, skip_serializing_if = "is_false")]
        before: bool,
    },
    /// While held, `fact` is held from the first moment `has` is held (or, `not`, isn't), and stays held while the
    /// source is, whatever `has` does later: the game's `AddFeatureIfHasFact`
    /// (`Designers/Mechanics/Facts/AddFeatureIfHasFact.cs` `Apply`, run on activation and on every level gained, adds
    /// once and never removes; only `OnDeactivate` does).
    GrantIf {
        has: Fact,
        #[serde(default, skip_serializing_if = "is_false")]
        not: bool,
        fact: Fact,
    },
    /// When it is first held, `fact` is held for good, whatever later happens to this one: the game's
    /// `AddFeatureOnApply` (`Designers/Mechanics/Facts/AddFeatureOnApply.cs` `OnActivate`; it has no `OnDeactivate`).
    GrantOnApply(Fact),
    /// When it is first held, `fact` stops being held: the game's `RemoveFeatureOnApply`
    /// (`Designers/Mechanics/Facts/RemoveFeatureOnApply.cs` `OnActivate`, `RemoveFact`; a stage replacing the one
    /// before it).
    RemoveOnApply(Fact),
    /// While held, a parameterised feat is held for one value: the game's `AddParametrizedFeatures`
    /// (`UnitLogic/FactLogic/AddParametrizedFeatures.cs`; the Disenchanter's Spell Focus (Abjuration)).
    GrantWithValue { feat: FeatKey, value: Parameter },
    /// While held, a bonus to a stat: the game's `AddStatBonus`, `AddContextStatBonus` and `AddStatBonusIfHasFact`
    /// (`UnitLogic/FactLogic/AddStatBonus.cs` `OnTurnOn`: `Value × rank`; `AddContextStatBonus.cs` `OnTurnOn`,
    /// `CalculateBaseValue`: `Value × Multiplier`, at least `Minimal` with `HasMinimal`, `at_least` here;
    /// `Designers/Mechanics/Buffs/AddStatBonusIfHasFact.cs`, `if` here: [`IfHeld`]). Bonuses of one descriptor to one stat
    /// stack as `EntitySystem/Stats/ModifiableValue.cs` `ApplyModifiersFiltered` stacks them ([`Descriptor::stacks`]),
    /// and an ability prerequisite counts only the permanent ones ([`Descriptor::is_permanent`]).
    StatBonus {
        stat: StatTarget,
        value: Value,
        #[serde(default, skip_serializing_if = "Descriptor::is_untyped")]
        descriptor: Descriptor,
        #[serde(default = "one", skip_serializing_if = "is_one")]
        multiplier: i8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at_least: Option<i16>,
        #[serde(default, rename = "if", skip_serializing_if = "Option::is_none")]
        only_if: Option<IfHeld>,
    },
    /// While held, a skill is a class skill: the game's `AddClassSkill` and `AddBackgroundClassSkill`
    /// (`UnitLogic/FactLogic/AddClassSkill.cs` `OnTurnOn`).
    ClassSkill(SkillKey),
    /// While held, this many more skill points at each level: the game's `AddSkillPointPerCharacterLevel`
    /// (`UnitLogic/FactLogic/AddSkillPointPerCharacterLevel.cs` `HandleUnitCalculateSkillPointsOnLevelup`, which adds
    /// the character level to the points `UnitLogic/Class/LevelUp/Actions/ApplySkillPoints.cs` `Apply` counts over all
    /// levels: one a level).
    SkillPoints(u8),
    /// While held, proficiency with weapons and armour: the game's `AddProficiencies` and the background ones
    /// (`UnitLogic/FactLogic/AddProficiencies.cs` `OnTurnOn`). A whole weapon list or an armour group is the planner's
    /// proficiency feat for it.
    Proficiency {
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        weapons: BTreeSet<WeaponKey>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        lists: BTreeSet<WeaponProficiency>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        armour: BTreeSet<ArmourProficiency>,
    },
    /// While held, a mechanics flag is set: the game's `AddMechanicsFeature`
    /// (`UnitLogic/FactLogic/AddMechanicsFeature.cs` `OnTurnOn`).
    Flag(MechanicsFlag),
    /// While held, a bonus to the save DC of the spells of `scope`: the game's `IncreaseSpellSchoolDC` (a school),
    /// `IncreaseSpellDescriptorDC` (an energy or a subschool, the game's spell descriptors), `IncreaseAllSpellsDC`, and
    /// `ArcaneBloodlineArcana` (a spell with metamagic but Heighten), each adding its bonus through
    /// `RuleCalculateAbilityParams.AddBonusDC` in `OnEventAboutToTrigger` (`Designers/Mechanics/Facts/…`). A scope of
    /// [`Value`](crate::SpellScope::Value) is the school or energy its entry is held for (Elemental Focus (Fire)).
    SpellDc { scope: SpellScope, value: Value },
    /// Spell Focus: `bonus` to the DC of the spells of the school its entry is held for, doubled while `mythic_focus` is
    /// held for that school (or for any school while an Expanded Arsenal names this one), and none for a school an
    /// Expanded Arsenal names, whose spells take [`ExpandedArsenal`](Component::ExpandedArsenal)'s bonus instead: the
    /// game's `SpellFocusParametrized` (`Designers/Mechanics/Facts/SpellFocusParametrized.cs` `OnEventAboutToTrigger`,
    /// `GetBonusDC`; `UntypedStackable`).
    SpellFocus { bonus: i8, mythic_focus: Fact },
    /// Expanded Arsenal held for a school: a spell of any school an Expanded Arsenal is held for gets, from each
    /// Expanded Arsenal held, the most the `focuses` held for one other school add up to (each its
    /// [`SpellFocus`](Component::SpellFocus) bonus, doubled or not): the game's `ExpandedArsenalMagicSchools`
    /// (`Designers/Mechanics/Facts/ExpandedArsenalMagicSchools.cs` `OnTurnOn`, `OnEventAboutToTrigger`: each fact's handler
    /// checks the shared list of schools, so two add it twice). Its `m_SchoolMastery` caster level is left out with
    /// School Mastery.
    ExpandedArsenal { focuses: BTreeSet<Fact> },
    /// While held, and while `only_while` holds when it is set, a bonus to caster level checks against spell
    /// resistance: the game's `SpellPenetrationBonus` (`Designers/Mechanics/Facts/SpellPenetrationBonus.cs`
    /// `OnEventAboutToTrigger`, its `CheckFact` and `m_RequiredFact` a held fact). Spell Penetration (Mythic)'s two
    /// stages, which the game holds as two features its `AddFeatureIfHasFact` grants with and without Greater Spell
    /// Penetration and `RecalculateOnFactsChange` swaps (`SpellPenetrationMythicFeat`
    /// 51b6b22ff184eef46a675449e837365d), are two of these with a `Has` and a `HasNot`.
    SpellPenetration {
        value: Value,
        #[serde(default, rename = "while", skip_serializing_if = "Option::is_none")]
        only_while: Option<Condition>,
    },
    /// While held, a bonus to concentration checks: the game's `ConcentrationBonus`
    /// (`Designers/Mechanics/Facts/ConcentrationBonus.cs` `GetStaticConcentrationBonus`).
    Concentration(Value),
    /// While held, the character may apply a metamagic: the game's `AddMetamagicFeat`
    /// (`UnitLogic/FactLogic/AddMetamagicFeat.cs`). The caster line reads Heighten Spell's.
    Metamagic(MetamagicKind),
    /// +1 to attack rolls with the weapon its entry is held for: the game's `WeaponFocusParametrized`
    /// (`Designers/Mechanics/Facts/WeaponFocusParametrized.cs` `OnEventAboutToTrigger`, `UntypedStackable`). Its
    /// `m_MythicFocus` doubling is a mythic feat's, which the attack line leaves out.
    WeaponFocus,
    /// +2 to damage with the weapon its entry is held for: the game's `WeaponSpecializationParametrized` with `Mythic`
    /// unset (`Designers/Mechanics/Facts/WeaponSpecializationParametrized.cs` `OnEventAboutToTrigger`).
    WeaponSpecialization,
    /// The weapon its entry is held for threatens on twice the rolls: the game's `ImprovedCriticalParametrized`
    /// (`Designers/Mechanics/Facts/ImprovedCriticalParametrized.cs`, `DoubleCriticalEdge`).
    ImprovedCritical,
    /// The weapon its entry is held for threatens on one roll more, after Improved Critical's doubling: the game's
    /// `ImprovedCriticalEdgeParametrized` (`Designers/Mechanics/Facts/ImprovedCriticalEdgeParametrized.cs`,
    /// `CriticalEdgeBonus`).
    CriticalEdge,
    /// +1 to the critical multiplier of the weapon its entry is held for: the game's
    /// `ImprovedCriticalMythicParametrized` (`Designers/Mechanics/Facts/ImprovedCriticalMythicParametrized.cs`,
    /// `AdditionalCriticalMultiplier`).
    CriticalMultiplier,
    /// While held, Power Attack's trade: the game's `AddFacts` of `PowerAttackToggleAbility` with `PowerAttackWatcher`
    /// (`PowerAttackFeature` 9972f33f977fc724c838e59641b2fca5); the attack line gives the trade as the guide has it.
    PowerAttack,
    /// One always-on bonus of a gear item, the only kind a gear item holds: the game's `AddStatBonusEquipment`,
    /// `AllSavesBonusEquipment` and the flat bonuses of the facts an item gives its wearer (`AddUnitFeatureEquipment`;
    /// `Designers/Mechanics/EquipmentEnchants/AddStatBonusEquipment.cs` `OnTurnOn`, with the item as the modifier's
    /// source). Its bonuses stack with every other source's by type ([`game_stacking`]), and an enhancement one counts
    /// toward ability prerequisites ([`Descriptor::is_permanent`]). Never 0; a penalty is untyped.
    GearStat {
        stat: GearTarget,
        value: i8,
        #[serde(default, skip_serializing_if = "Descriptor::is_untyped")]
        descriptor: Descriptor,
    },
    /// While held, the highest ability score (after every stat bonus; on a tie the first in Str, Dex, Con, Int, Wis, Cha
    /// order) gets this for each rank of its entry: the Gold Dragon's Perfect Body, Mind and Soul. The game applies one
    /// buff per score through a tree of conditions (`AddFactContextActions` on `DragonLevel1MaxAbilityStrengthFeature`
    /// 6a7f8ee85c7f42228c83bb9d2f1711f2 and its siblings); this is that pattern.
    HighestAbilityBonus(i8),
    /// While held, every ability score is at least this, after every stat bonus and before
    /// [`HighestAbilityBonus`](Component::HighestAbilityBonus): Perfect Body's 14 and Perfect Mind's 18. The game's
    /// per-score overrides (`DragonLevel2StrengthOverride` 9ed55389d2079714eab8495957964af2: a shared value over the
    /// score's base value) read the score before the gear's and the other bonuses; this reads it after them, as the
    /// planner always has (the data note lists the difference).
    AbilityAtLeast(u8),
    /// While held, the base attack bonus is at least this plus the Mythic rank: Perfect Mind's
    /// `DragonLevel2BABOverride` f4af14e9f4408d44c830d891fee8f4a5 (a property of the base attack bonus less this and the
    /// Mythic rank, the game's code; its text says 15).
    BaseAttackAtLeast(u8),
    /// While held, every skill is a class skill and a skill's total counts at least the Mythic rank for its ranks:
    /// Perfect Mind's `AddGoldenDragonSkillBonus` (`UnitLogic/FactLogic/AddGoldenDragonSkillBonus.cs` `OnTurnOn`: a
    /// modifier of the Mythic rank less the skill's base value, at least 0) with its class skills. A skill-rank
    /// prerequisite still reads the ranks spent.
    GoldenDragonSkills,
    /// While held, the feature rises one step each time the rank's value rises past one of its bands: a
    /// `ContextRankConfig` on the level its numbers read (`ContextRankConfig.GetValue`), so its steps follow that level
    /// whatever rows are traded (ticket 254: Infernal Resistance on bloodrager levels). The step is the value
    /// ([`Engine::steps_reached`](crate::Engine::steps_reached)); the levels it rises at are worked out from it.
    Scales(RankConfig),
}

/// A metamagic a [`Metamagic`](Component::Metamagic) component lets the character apply: the game's `Metamagic` the
/// planner reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MetamagicKind {
    /// A spell cast as one of a higher level (the caster line's heightened DC).
    Heighten,
}

/// The two prerequisite groups of `BlueprintFeature.MeetsPrerequisites`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Group {
    /// Every condition of the group holds.
    #[default]
    All,
    /// One condition of the group holds.
    Any,
}

impl Group {
    fn is_all(&self) -> bool {
        *self == Self::All
    }
}

/// Something a character holds, by its kind and key: what a component grants, checks or reads the rank of.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Fact {
    Feat(FeatKey),
    Feature(FeatureKey),
    Option(SelectionOptionKey),
    Selection(FeatureSelectionKey),
    MythicAbility(MythicAbilityKey),
    MythicFeat(MythicFeatKey),
    Class(ClassKey),
    Archetype(ArchetypeKey),
    Race(RaceKey),
    Deity(DeityKey),
}

impl Fact {
    /// Whether a grant can hold it: a class, an archetype, a race and a deity are chosen, never granted.
    pub fn is_grantable(&self) -> bool {
        !matches!(
            self,
            Self::Class(_) | Self::Archetype(_) | Self::Race(_) | Self::Deity(_)
        )
    }

    /// How many times it is held (a feature's grants, a feat's picks and grants, a mythic ability's or a mythic feat's
    /// picks), or 1 for what is held once (the race, a class taken); 0 when it isn't. The rank `Features.GetRank`
    /// gives.
    pub fn rank(&self, state: &CharacterState) -> u8 {
        match self {
            Self::Feat(key) => state.feat_count(key),
            Self::Feature(key) => state.feature_count(key),
            Self::Option(key) => state.option_count(key),
            Self::Selection(key) => state.selection_count(key),
            Self::MythicAbility(key) => state.mythic_ability_count(key),
            Self::MythicFeat(key) => state.mythic_feat_count(key),
            Self::Class(key) => u8::from(state.class_level(key) > 0),
            Self::Archetype(key) => u8::from(
                state
                    .classes
                    .iter()
                    .any(|class| class.archetype.as_ref() == Some(key)),
            ),
            Self::Race(key) => u8::from(state.race.as_ref() == Some(key)),
            Self::Deity(key) => u8::from(state.deity() == Some(key)),
        }
    }

    /// The prerequisite leaf that holds while it is held.
    pub(crate) fn leaf(&self) -> Prerequisite {
        match self {
            Self::Feat(key) => Prerequisite::Feat(key.clone()),
            Self::Feature(key) => Prerequisite::Feature {
                feature: key.clone(),
                count: 1,
            },
            Self::Option(key) => Prerequisite::SelectionOption(key.clone()),
            Self::Selection(key) => Prerequisite::Selection(key.clone()),
            Self::MythicAbility(key) => Prerequisite::MythicAbility(key.clone()),
            Self::MythicFeat(key) => Prerequisite::MythicFeat(key.clone()),
            Self::Class(key) => Prerequisite::ClassLevel {
                class: key.clone(),
                level: 1,
            },
            Self::Archetype(key) => Prerequisite::Archetype(key.clone()),
            Self::Race(key) => Prerequisite::Race(key.clone()),
            Self::Deity(key) => Prerequisite::Deity(key.clone()),
        }
    }
}

/// What holds components while the walk holds it: a fact, or the Build start's subrace or background, which no
/// condition names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Holder {
    Fact(Fact),
    Subrace(SubraceKey),
    Background(BackgroundKey),
}

/// One condition of a [`Requires`](Component::Requires), in the game's terms; each compiles into the planner's
/// [`Prerequisite`] ([`compile_requires`]).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Condition {
    /// The fact is held: `PrerequisiteFeature`, `ContextConditionHasFact`. (`PrerequisiteFeature.CheckInternal` also
    /// fails while the fact is being picked in the same selection's children, `IsSelectedInChildren`: a nested
    /// selection's rule, 298's.)
    Has(Fact),
    /// The fact isn't held: `PrerequisiteNoFeature` (`CheckInternal`: `!unit.HasFact`).
    HasNot(Fact),
    /// The fact is held at least this many times: `ContextConditionBuffRank`'s and `ContextConditionHasFact`'s rank,
    /// `Features.GetRank`. `Not` of it on an entry's own fact, in the `All` group, caps how often a selection is taken
    /// (Extra Hex: `ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35 at 3; [`Repeat`](crate::Repeat), ticket
    /// 303).
    FactRank(Fact, u8),
    /// At least `n` of the facts are held: `PrerequisiteFeaturesFromList` (`CheckInternal` returns on the first fact
    /// held that brings the count to `Amount`, so an `Amount` of 0 needs one, as 1 does). Its `m_RestrictIfNot`
    /// (fewer than `Amount` held) is `Not` of this. Over deities only (`IsDeity`: one is enough, and at the first
    /// character level none held counts as met, `ConsiderFulfilled`) the check refuses it
    /// ([`DataIssue::DeityList`](crate::DataIssue::DeityList)).
    AtLeast(u8, BTreeSet<Fact>),
    /// A class level at least this: `PrerequisiteClassLevel` (`GetClassLevel`, with the levels a held
    /// [`PrerequisiteClassLevels`](Component::PrerequisiteClassLevels) adds), `ContextConditionCharacterClass`.
    ClassLevel(ClassKey, u8),
    /// No level in the class: `PrerequisiteNoClassLevel`.
    NoClassLevel(ClassKey),
    /// The class of the archetype taken with it, at least this level: `PrerequisiteArchetypeLevel`.
    ArchetypeLevel(ArchetypeKey, u8),
    /// The archetype's class not taken with it: `PrerequisiteNoArchetype` (`CheckInternal`: a class not taken passes).
    NoArchetype(ArchetypeKey),
    CharacterLevel(u8),
    /// The Mythic rank: `PrerequisiteMythicLevel`.
    MythicLevel(u8),
    /// A stat at least `value`: `PrerequisiteStatValue` (`GetStatValue`: a skill's base value, its ranks; another
    /// stat's permanent value) or, `full`, `PrerequisiteFullStatValue` (`GetFullStatValue`: the permanent value plus
    /// the modifiers of descriptor `Feat`). Either then goes through each held
    /// [`PrerequisiteStat`](Component::PrerequisiteStat) of the stat (`ReplaceStatForPrerequisites.ResultStat`, with
    /// `fullValue` for the full one).
    Stat {
        stat: StatRef,
        value: u8,
        #[serde(default, skip_serializing_if = "is_false")]
        full: bool,
    },
    /// Spells of this level from the class's spellbook: `PrerequisiteClassSpellLevel` (`GetClassSpellLevel`).
    ClassSpellLevel(ClassKey, u8),
    /// Spells of this level and type, spontaneously when `spontaneous`: `PrerequisiteCasterTypeSpellLevel`.
    CasterTypeSpellLevel {
        caster: SpellType,
        level: u8,
        #[serde(default, skip_serializing_if = "is_false")]
        spontaneous: bool,
    },
    /// A spellbook of this type, whatever level it reaches: `PrerequisiteCasterType` (`CheckInternal`).
    CasterType(SpellType),
    /// One of these alignments: `PrerequisiteAlignment` (`CheckInternal`: the alignment's mask against the
    /// prerequisite's).
    Alignment(BTreeSet<Alignment>),
    /// Proficient with one of the armour groups: `PrerequisiteProficiency` (`CheckInternal`: any one holds).
    Proficient(BTreeSet<ArmourProficiency>),
    /// Not proficient with one of the armour groups: `PrerequisiteNotProficient` (`CheckInternal`: any one lacking).
    NotProficient(BTreeSet<ArmourProficiency>),
    /// A parameterised feat held for one value: `PrerequisiteParametrizedFeature`.
    ParametrizedFeature(FeatKey, Parameter),
    /// Every condition holds: `PrerequisiteCondition`'s `ConditionsChecker` with `Operation.And`.
    All(Vec<Condition>),
    /// One condition holds: `ConditionsChecker` with `Operation.Or`.
    Any(Vec<Condition>),
    /// The condition doesn't hold: a condition's `Not`.
    Not(Box<Condition>),
}

/// A stat a prerequisite reads.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StatRef {
    Ability(Ability),
    /// A skill's ranks.
    Skill(SkillKey),
    BaseAttack,
    /// The skill a parameterised entry is held for (Skill Focus (Persuasion)): only in a value
    /// ([`RankBase::BaseStat`]), on an entry taken for a skill.
    Parameter,
}

/// How a held [`PrerequisiteStat`](Component::PrerequisiteStat) changes the value a prerequisite reads
/// (`ReplaceStatForPrerequisites.ResultStat`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StatRule {
    /// The higher of the stat and this ability's score (`NewStat`).
    Ability(Ability),
    /// The higher of the stat and the class's level (`ClassLevel`).
    ClassLevel(ClassKey),
    /// The stat plus the class's level (`ClassLevelStacking`).
    PlusClassLevel(ClassKey),
    /// The higher of the stat and this number (`SpecificNumber`).
    AtLeast(u8),
    /// The stat plus this number (`Summand`).
    Plus(i8),
    /// The stat plus a quarter of the class's levels, plus ¾, rounded down (`MagusBaseAttack`: a magus's base attack
    /// bonus as a full one).
    MagusBaseAttack(ClassKey),
}

/// The picks a [`PrerequisiteClassLevels`](Component::PrerequisiteClassLevels) counts for: the game's
/// `ForSelection`, whose own options are those picks.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Pool {
    /// Feats (`BasicFeatSelection`).
    Feats,
    /// The options of this selection (`OracleRevelationSelection`: the revelations).
    Options(FeatureSelectionKey),
}

/// The share of a class's levels a [`PrerequisiteClassLevels`](Component::PrerequisiteClassLevels) counts: the game's
/// `Modifier`, 1 or ½ in every blueprint.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Share {
    #[default]
    Full,
    Half,
}

impl Share {
    fn is_full(&self) -> bool {
        *self == Self::Full
    }

    /// The levels counted of `levels`, plus `plus`: `(int)(Modifier × levels + Summand)`
    /// (`PrerequisiteClassLevel.GetClassLevel`), C#'s cast truncating toward zero.
    pub(crate) fn levels(self, levels: u8, plus: i8) -> i16 {
        let levels = i16::from(levels);
        let plus = i16::from(plus);
        match self {
            Self::Full => levels + plus,
            // `(int)(0.5 × levels + plus)`: twice the value, halved toward zero.
            Self::Half => (levels + 2 * plus) / 2,
        }
    }
}

/// The planner's proficiency feat for a whole weapon list (`AddProficiencies`' weapon categories of one list): Simple
/// and Martial Weapon Proficiency, and Exotic Weapon Proficiency held for every exotic weapon.
impl WeaponProficiency {
    pub(crate) fn feat(self) -> FeatKey {
        FeatKey::new(match self {
            Self::Simple => crate::game_data::SIMPLE_WEAPON_PROFICIENCY,
            Self::Martial => crate::game_data::MARTIAL_WEAPONS_PROFICIENCY,
            Self::Exotic => crate::game_data::EXOTIC_WEAPON_PROFICIENCY,
        })
    }
}

/// An armour group a proficiency names, held as the planner's proficiency feat for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArmourProficiency {
    Light,
    Medium,
    Heavy,
    Shield,
    TowerShield,
}

impl ArmourProficiency {
    /// The planner's proficiency feat for the group.
    pub fn feat(self) -> FeatKey {
        FeatKey::new(match self {
            Self::Light => "armor-proficiency-light-armor",
            Self::Medium => "armor-proficiency-medium-armor",
            Self::Heavy => "armor-proficiency-heavy-armor",
            Self::Shield => "shield-proficiency",
            Self::TowerShield => "tower-shield-proficiency",
        })
    }
}

/// A mechanics flag a planner number reads (`AddMechanicsFeature.MechanicsFeatureType`); the others are combat-time
/// and left out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MechanicsFlag {
    /// Dexterity to attack rolls with a finesse weapon: the attack line's.
    WeaponFinesse,
}

/// The stat a [`StatBonus`](Component::StatBonus) adds to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StatTarget {
    Ability(Ability),
    Skill(SkillKey),
    BaseAttack,
    Save(Save),
    Initiative,
    /// The stat a parameterised entry is held for, on an entry taken for a skill or a stat: Skill Focus (Persuasion)'s
    /// skill, Stat Focus (Will)'s save. The planner holds those feats parameterised; the game has a feature per skill
    /// (`SkillFocusAcrobatics` 52dd89af385466c499338b7297896ded), its selections' shape (298). A stat with no line
    /// (Stat Focus (Strength)) counts in no total.
    Parameter,
    /// The stats the planner has no line for yet: kept in the data and counted in no total.
    ArmourClass,
    CombatManeuverBonus,
    CombatManeuverDefense,
    HitPoints,
    Speed,
}

impl StatTarget {
    /// The stat a bonus adds to for an entry held for `value`: a [`Parameter`](StatTarget::Parameter) one is the
    /// value's skill, initiative or save (`None` for a stat with no line, or no value); any other is itself.
    pub(crate) fn resolve(
        &self,
        value: Option<&crate::parameter::Parameter>,
    ) -> Option<StatTarget> {
        use crate::parameter::{Parameter as P, Stat};
        match (self, value) {
            (Self::Parameter, Some(P::Skill(skill))) => Some(Self::Skill(skill.clone())),
            (Self::Parameter, Some(P::Stat(stat))) => match stat {
                Stat::Initiative => Some(Self::Initiative),
                Stat::Fortitude => Some(Self::Save(Save::Fortitude)),
                Stat::Reflex => Some(Self::Save(Save::Reflex)),
                Stat::Will => Some(Self::Save(Save::Will)),
                _ => None,
            },
            (Self::Parameter, _) => None,
            (stat, _) => Some(stat.clone()),
        }
    }
}

/// The amount a [`StatBonus`](Component::StatBonus) adds at the state of `context`: its value times its multiplier, at
/// least its minimum when it has one (`AddContextStatBonus.CalculateBaseValue`), kept within `i16`.
pub(crate) fn stat_bonus_amount(
    value: &Value,
    multiplier: i8,
    at_least: Option<i16>,
    context: &ValueContext<'_>,
) -> i16 {
    let mut amount = value.at(context) * i32::from(multiplier);
    if let Some(minimum) = at_least {
        amount = amount.max(i32::from(minimum));
    }
    i16::try_from(amount).unwrap_or(if amount < 0 { i16::MIN } else { i16::MAX })
}

/// The type of a stat bonus: the game's `ModifierDescriptor`, the values blueprints the planner reaches use.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Descriptor {
    /// `None`.
    #[default]
    Untyped,
    UntypedStackable,
    Alchemical,
    Armour,
    Circumstance,
    Competence,
    Deflection,
    Dodge,
    Enhancement,
    Feat,
    Focus,
    Inherent,
    Insight,
    Luck,
    Morale,
    Mythic,
    NaturalArmour,
    Penalty,
    Profane,
    Racial,
    Resistance,
    Sacred,
    Shield,
    Size,
    Trait,
}

impl Descriptor {
    fn is_untyped(&self) -> bool {
        *self == Self::Untyped
    }

    /// Whether every bonus of the type to one stat counts: `Enums/ModifierDescriptorHelper.cs`
    /// `DefaultStackingDescriptors` (`None`, `Racial`, `Dodge`, `UntypedStackable`, `Feat`, `Penalty`, `Focus`,
    /// `NaturalArmor`, with `ArmorFocus` and others the planner doesn't meet). Of any other type the highest bonus and
    /// the lowest penalty count (`ModifiableValue.ApplyModifiersFiltered`).
    pub fn stacks(self) -> bool {
        matches!(
            self,
            Self::Untyped
                | Self::UntypedStackable
                | Self::Racial
                | Self::Dodge
                | Self::Feat
                | Self::Penalty
                | Self::Focus
                | Self::NaturalArmour
        )
    }

    /// Whether an ability prerequisite counts a bonus of the type: `ModifiableValue.Modifier.IsPermanent`, a racial
    /// one, or an enhancement one from an item (`from_item`: a gear item's
    /// [`GearStat`](Component::GearStat)), which `ModifiableValueAttributeStat.CalculatePermanentValue` filters by.
    pub fn is_permanent(self, from_item: bool) -> bool {
        self == Self::Racial || (from_item && self == Self::Enhancement)
    }

    /// Lower case, as a summary prints it ("enhancement", "natural armour"); "" for the untyped ones (`None`,
    /// `UntypedStackable`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Untyped | Self::UntypedStackable => "",
            Self::Alchemical => "alchemical",
            Self::Armour => "armour",
            Self::Circumstance => "circumstance",
            Self::Competence => "competence",
            Self::Deflection => "deflection",
            Self::Dodge => "dodge",
            Self::Enhancement => "enhancement",
            Self::Feat => "feat",
            Self::Focus => "focus",
            Self::Inherent => "inherent",
            Self::Insight => "insight",
            Self::Luck => "luck",
            Self::Morale => "morale",
            Self::Mythic => "mythic",
            Self::NaturalArmour => "natural armour",
            Self::Penalty => "penalty",
            Self::Profane => "profane",
            Self::Racial => "racial",
            Self::Resistance => "resistance",
            Self::Sacred => "sacred",
            Self::Shield => "shield",
            Self::Size => "size",
            Self::Trait => "trait",
        }
    }

    /// Whether a full ability prerequisite counts a bonus of the type beside the permanent ones: a `Feat` one
    /// (`PrerequisiteFullStatValue.GetFullStatValue`).
    pub fn counts_in_full_value(self) -> bool {
        self == Self::Feat
    }
}

/// Which of several amounts of one stat count, in step with `amounts`: `ModifiableValue.ApplyModifiersFiltered`. Of
/// a type that stacks ([`Descriptor::stacks`]) every one counts; of any other type the highest bonus (the first of
/// equals) and the lowest penalty. Amounts of different types all count.
pub fn game_stacking(amounts: &[(Descriptor, i16)]) -> Vec<bool> {
    amounts
        .iter()
        .enumerate()
        .map(|(at, &(descriptor, amount))| {
            descriptor.stacks()
                || !amounts.iter().enumerate().any(|(other, &(of, than))| {
                    of == descriptor
                        && ((amount >= 0 && (than > amount || (than == amount && other < at)))
                            || (amount < 0 && (than < amount || (than == amount && other < at))))
                })
        })
        .collect()
}

/// What the amounts of one stat add up to under [`game_stacking`].
pub fn game_stacked(amounts: &[(Descriptor, i16)]) -> i16 {
    amounts
        .iter()
        .zip(game_stacking(amounts))
        .filter(|(_, counts)| *counts)
        .map(|((_, amount), _)| amount)
        .sum()
}

/// The condition of an `AddStatBonusIfHasFact`: its checked facts, whether all are needed, and whether it's inverted.
/// The bonus is decided when its entry is first held (`OnTurnOn` → `Update`) and looked at again only on two events:
/// a checked fact gained while inverted (`HandleUnitGainFact`) and a checked fact lost while not
/// (`HandleUnitLostFact`). So a bonus whose fact comes later stays off, and one turned off stays off.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IfHeld {
    pub facts: BTreeSet<Fact>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub all: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub not: bool,
}

impl IfHeld {
    /// Whether the bonus applies, as `AddStatBonusIfHasFact.ShouldApplyBonus` decides: with every fact needed (or a
    /// single one), one missing gives `not`; otherwise one held gives `!not`; and when the loop ends, `!not`. So with
    /// two facts or more and `all` unset it applies whatever is held: the code's behaviour, modelled as it is.
    pub fn holds(&self, state: &CharacterState) -> bool {
        let every = self.all || self.facts.len() == 1;
        for fact in &self.facts {
            let held = fact.rank(state) > 0;
            if held && !every {
                return !self.not;
            }
            if !held && every {
                return self.not;
            }
        }
        !self.not
    }
}

/// The value a bonus reads: a number, or one worked out from the character at that level.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Value {
    /// A number (`ContextValue` of type `Simple`).
    Flat(i16),
    /// A number for each rank of the entry holding it: `AddStatBonus`'s `Value × Fact.GetRank()`.
    PerRank(i16),
    /// A rank (`ContextValue` of type `Rank`, its `ContextRankConfig`).
    Rank(RankConfig),
    /// A shared value (`ContextValue` of type `Shared`, its `ContextCalculateSharedValue`).
    Shared(SharedValue),
}

/// `ContextRankConfig`: a base value, run through a progression, then kept between `min` and `max`
/// (`UnitLogic/Mechanics/Components/ContextRankConfig.cs` `GetValue` = `ApplyMinMax(ApplyProgression(GetBaseValue))`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RankConfig {
    pub base: RankBase,
    #[serde(default, skip_serializing_if = "RankProgression::is_as_is")]
    pub progression: RankProgression,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i16>,
}

/// A rank's base value (`ContextRankConfig.GetBaseValue`), the bases the planner can know.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RankBase {
    /// The levels of the classes added up, or with `except` of every other class (`ClassLevel`).
    ClassLevel {
        classes: BTreeSet<ClassKey>,
        #[serde(default, skip_serializing_if = "is_false")]
        except: bool,
    },
    /// As `ClassLevel`, a class counting only taken with one of `archetypes` when they are its own, plus the ranks of
    /// `features` (`SummClassLevelWithArchetype`); with `max`, the highest such class's level alone, `features` unread
    /// (`MaxClassLevelWithArchetype`: `CheckAndSetMaxValue` over the classes, no `m_FeatureList`).
    ClassLevelWithArchetype {
        classes: BTreeSet<ClassKey>,
        #[serde(default, skip_serializing_if = "is_false")]
        except: bool,
        archetypes: BTreeSet<ArchetypeKey>,
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        features: BTreeSet<Fact>,
        #[serde(default, skip_serializing_if = "is_false")]
        max: bool,
    },
    CharacterLevel,
    /// The Mythic rank (`MythicLevel`).
    MythicLevel,
    /// The rank of one fact (`FeatureRank`).
    FeatureRank(Fact),
    /// How many of the facts are held (`FeatureList`).
    FeatureList(BTreeSet<Fact>),
    /// The ranks of the facts added up (`FeatureListRanks`).
    FeatureListRanks(BTreeSet<Fact>),
    /// An ability's modifier, with every bonus (`StatBonus`: `ModifiableValueAttributeStat.Bonus`).
    StatBonus(Ability),
    /// A stat's base value: an ability's score before any modifier, the race's included (they are `Racial` modifiers:
    /// `ElfRace`'s `AddStatBonus`, `SelectRaceStat.Apply`), a skill's ranks (`BaseStat`: `ModifiableValue.BaseValue`).
    /// A skill's ranks are those its total counts, with [`GoldenDragonSkills`](Component::GoldenDragonSkills)' floor
    /// (`CharacterState::skill_ranks_in_total`), as the planner's ten-ranks bonuses always read them; the game's floor
    /// is a modifier, which leaves `BaseValue` alone (the data note of ticket 297 lists it).
    BaseStat(StatRef),
    /// The base attack bonus (`BaseAttack`).
    BaseAttack,
}

/// `ContextRankConfig.ApplyProgression`: every progression the game has. C#'s integer division truncates toward zero,
/// as Rust's does.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RankProgression {
    #[default]
    AsIs,
    Div2,
    OnePlusDiv2,
    StartPlusDivStep {
        start: i16,
        step: i16,
    },
    DivStep(i16),
    OnePlusDivStep(i16),
    MultiplyBy(i16),
    HalfMore,
    BonusValue(i16),
    DelayedStartPlusDivStep {
        start: i16,
        step: i16,
    },
    StartPlusDoubleDivStep {
        start: i16,
        step: i16,
    },
    Div2PlusStep(i16),
    /// Bands of `(up to, value)`: the first band whose bound is at or above the base value gives its value, the last
    /// band's value past them all (`Custom`: Skill Focus's +3, or +6 from 10 ranks).
    Custom(Vec<(i16, i16)>),
    DoublePlusBonusValue(i16),
    SquaredValue,
}

impl RankProgression {
    fn is_as_is(&self) -> bool {
        *self == Self::AsIs
    }

    /// The step a division reads, for the check (a step of 0 divides by zero).
    pub fn step(&self) -> Option<i16> {
        match self {
            Self::StartPlusDivStep { step, .. }
            | Self::DelayedStartPlusDivStep { step, .. }
            | Self::StartPlusDoubleDivStep { step, .. }
            | Self::DivStep(step)
            | Self::OnePlusDivStep(step) => Some(*step),
            _ => None,
        }
    }

    /// `ApplyProgression(value)`.
    pub fn apply(&self, value: i32) -> i32 {
        let div = |a: i32, b: i16| a.checked_div(i32::from(b)).unwrap_or(0);
        match self {
            Self::AsIs => value,
            Self::Div2 => value / 2,
            Self::OnePlusDiv2 => 1 + (value - 1) / 2,
            Self::StartPlusDivStep { start, step } => {
                1 + div(value - i32::from(*start), *step).max(0)
            }
            Self::DivStep(step) => div(value, *step),
            Self::OnePlusDivStep(step) => 1 + div(value, *step).max(0),
            Self::MultiplyBy(by) => value * i32::from(*by),
            Self::HalfMore => value + value / 2,
            Self::BonusValue(bonus) => value + i32::from(*bonus),
            Self::DelayedStartPlusDivStep { start, step } => {
                if value < i32::from(*start) {
                    0
                } else {
                    1 + div(value - i32::from(*start), *step).max(0)
                }
            }
            Self::StartPlusDoubleDivStep { start, step } => {
                1 + 2 * div(value - i32::from(*start), *step).max(0)
            }
            Self::Div2PlusStep(step) => value / 2 + i32::from(*step),
            Self::Custom(bands) => bands
                .iter()
                .find(|(up_to, _)| value <= i32::from(*up_to))
                .or(bands.last())
                .map_or(0, |(_, value)| i32::from(*value)),
            Self::DoublePlusBonusValue(bonus) => value * 2 + i32::from(*bonus),
            Self::SquaredValue => value * value,
        }
    }
}

/// `ContextCalculateSharedValue`: `(count × 1 + bonus) × modifier` (`Calculate`, its dice of type `One` or `Zero`:
/// no dice is no count).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SharedValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<Box<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonus: Option<Box<Value>>,
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub modifier: i8,
}

/// What a value reads besides the state: the rank of the entry holding it, the value it is held for, the ability
/// scores' base values, and the class of an archetype.
pub(crate) struct ValueContext<'a> {
    pub state: &'a CharacterState,
    /// The rank of the entry whose component it is ([`Value::PerRank`]).
    pub own_rank: u8,
    /// The value the entry is held for, for a parameterised one ([`StatRef::Parameter`]).
    pub value: Option<&'a Parameter>,
    /// The scores bought and raised at levels, without the race's modifiers ([`RankBase::BaseStat`]).
    pub base_scores: AbilityScores,
    pub archetype_class: &'a dyn Fn(&ArchetypeKey) -> Option<&'a ClassKey>,
}

impl Value {
    /// Whether something held besides its own entry may change it: a rank on a fact's rank, a score or the base
    /// attack bonus. A flat value, one per own rank (its entry's hold works it out again) and a rank on a level or the
    /// Mythic rank (worked out again at each level and rank) don't.
    pub(crate) fn reads_state(&self) -> bool {
        match self {
            Self::Flat(_) | Self::PerRank(_) => false,
            Self::Rank(config) => !matches!(
                &config.base,
                RankBase::ClassLevel { .. } | RankBase::CharacterLevel | RankBase::MythicLevel
            ),
            Self::Shared(shared) => [&shared.count, &shared.bonus]
                .into_iter()
                .flatten()
                .any(|part| part.reads_state()),
        }
    }

    /// The value at the state of `context`.
    pub(crate) fn at(&self, context: &ValueContext<'_>) -> i32 {
        match self {
            Self::Flat(value) => i32::from(*value),
            Self::PerRank(value) => i32::from(*value) * i32::from(context.own_rank),
            Self::Rank(config) => config.at(context),
            Self::Shared(shared) => {
                let part = |value: &Option<Box<Value>>| value.as_ref().map_or(0, |v| v.at(context));
                (part(&shared.count) + part(&shared.bonus)) * i32::from(shared.modifier)
            }
        }
    }
}

impl RankConfig {
    /// `ContextRankConfig.GetValue`.
    pub(crate) fn at(&self, context: &ValueContext<'_>) -> i32 {
        let mut value = self.progression.apply(self.base.at(context));
        if let Some(min) = self.min {
            value = value.max(i32::from(min));
        }
        if let Some(max) = self.max {
            value = value.min(i32::from(max));
        }
        value
    }
}

impl RankBase {
    /// `ContextRankConfig.GetBaseValue` for the bases the planner knows.
    fn at(&self, context: &ValueContext<'_>) -> i32 {
        let state = context.state;
        let named = |classes: &BTreeSet<ClassKey>, except: bool, class: &ClassKey| {
            classes.contains(class) != except
        };
        match self {
            Self::ClassLevel { classes, except } => state
                .classes
                .iter()
                .filter(|entry| named(classes, *except, &entry.class))
                .map(|entry| i32::from(entry.level))
                .sum(),
            Self::ClassLevelWithArchetype {
                classes,
                except,
                archetypes,
                features,
                max,
            } => {
                let levels = state
                    .classes
                    .iter()
                    .filter(|entry| named(classes, *except, &entry.class))
                    .filter(|entry| {
                        // `CheckClassForArchetype`: a class none of the archetypes belongs to counts as it is.
                        let own = archetypes.iter().any(|archetype| {
                            (context.archetype_class)(archetype) == Some(&entry.class)
                        });
                        !own || entry
                            .archetype
                            .as_ref()
                            .is_some_and(|taken| archetypes.contains(taken))
                    })
                    .map(|entry| i32::from(entry.level));
                if *max {
                    levels.max().unwrap_or(0)
                } else {
                    levels.sum::<i32>()
                        + features
                            .iter()
                            .map(|fact| i32::from(fact.rank(state)))
                            .sum::<i32>()
                }
            }
            Self::CharacterLevel => i32::from(state.character_level),
            Self::MythicLevel => i32::from(state.mythic_rank),
            Self::FeatureRank(fact) => i32::from(fact.rank(state)),
            Self::FeatureList(facts) => facts
                .iter()
                .filter(|fact| fact.rank(state) > 0)
                .count()
                .try_into()
                .unwrap_or(i32::MAX),
            Self::FeatureListRanks(facts) => {
                facts.iter().map(|fact| i32::from(fact.rank(state))).sum()
            }
            Self::StatBonus(ability) => i32::from(state.ability_scores.modifier(*ability)),
            Self::BaseStat(StatRef::Ability(ability)) => {
                i32::from(context.base_scores.get(*ability))
            }
            Self::BaseStat(StatRef::Skill(skill)) => i32::from(state.skill_ranks_in_total(skill)),
            Self::BaseStat(StatRef::Parameter) => match context.value {
                Some(Parameter::Skill(skill)) => i32::from(state.skill_ranks_in_total(skill)),
                _ => 0,
            },
            Self::BaseStat(StatRef::BaseAttack) | Self::BaseAttack => i32::from(state.bab),
        }
    }
}

impl Component {
    /// The kind's name, as a problem names it.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Requires { .. } => "Requires",
            Self::PrerequisiteClassLevels { .. } => "PrerequisiteClassLevels",
            Self::PrerequisiteStat { .. } => "PrerequisiteStat",
            Self::NoSelectionIf { .. } => "NoSelectionIf",
            Self::Grant(_) => "Grant",
            Self::GrantAtClassLevel { .. } => "GrantAtClassLevel",
            Self::GrantIf { .. } => "GrantIf",
            Self::GrantOnApply(_) => "GrantOnApply",
            Self::RemoveOnApply(_) => "RemoveOnApply",
            Self::GrantWithValue { .. } => "GrantWithValue",
            Self::StatBonus { .. } => "StatBonus",
            Self::ClassSkill(_) => "ClassSkill",
            Self::SkillPoints(_) => "SkillPoints",
            Self::Proficiency { .. } => "Proficiency",
            Self::Flag(_) => "Flag",
            Self::SpellDc { .. } => "SpellDc",
            Self::SpellFocus { .. } => "SpellFocus",
            Self::ExpandedArsenal { .. } => "ExpandedArsenal",
            Self::SpellPenetration { .. } => "SpellPenetration",
            Self::Concentration(_) => "Concentration",
            Self::Metamagic(_) => "Metamagic",
            Self::WeaponFocus => "WeaponFocus",
            Self::WeaponSpecialization => "WeaponSpecialization",
            Self::ImprovedCritical => "ImprovedCritical",
            Self::CriticalEdge => "CriticalEdge",
            Self::CriticalMultiplier => "CriticalMultiplier",
            Self::PowerAttack => "PowerAttack",
            Self::GearStat { .. } => "GearStat",
            Self::HighestAbilityBonus(_) => "HighestAbilityBonus",
            Self::AbilityAtLeast(_) => "AbilityAtLeast",
            Self::BaseAttackAtLeast(_) => "BaseAttackAtLeast",
            Self::GoldenDragonSkills => "GoldenDragonSkills",
            Self::Scales(_) => "Scales",
        }
    }

    /// Whether it changes a number the walk keeps on the state as it goes (an ability score, the base attack bonus,
    /// the skill points, the skill floor, a prerequisite's reading), rather than one the lines and totals read at a
    /// level's end: the walk works those out again when an entry holding one comes or goes.
    pub(crate) fn changes_state_numbers(&self) -> bool {
        match self {
            Self::StatBonus { stat, .. } => {
                matches!(stat, StatTarget::Ability(_) | StatTarget::BaseAttack)
            }
            Self::SkillPoints(_)
            | Self::PrerequisiteClassLevels { .. }
            | Self::PrerequisiteStat { .. }
            | Self::HighestAbilityBonus(_)
            | Self::AbilityAtLeast(_)
            | Self::BaseAttackAtLeast(_)
            | Self::GoldenDragonSkills => true,
            _ => false,
        }
    }

    /// Whether a state number it changes may change when something else is held: a stat bonus whose value reads the
    /// state (a rank, a fact's rank, a score), worked out again at every hold. The others change only with their
    /// entry, the level or the Mythic rank, when the walk works them out anyway.
    pub(crate) fn reads_state(&self) -> bool {
        matches!(self, Self::StatBonus { value, .. } if self.changes_state_numbers() && value.reads_state())
    }

    /// Whether the walk acts on it while its entry is held: every kind the walk holds, works out or records. The
    /// attack line's kinds, Heighten Spell's and a feature's `Scales` are read off what is held instead, so an entry
    /// holding only those needs no activation.
    pub(crate) fn walk_reads(&self) -> bool {
        !matches!(
            self,
            Self::WeaponFocus
                | Self::WeaponSpecialization
                | Self::ImprovedCritical
                | Self::CriticalEdge
                | Self::CriticalMultiplier
                | Self::PowerAttack
                | Self::Metamagic(_)
                | Self::Scales(_)
        )
    }

    /// Whether it is a `Requires`, which the checked data holds compiled into its entry's prerequisite.
    pub fn is_requires(&self) -> bool {
        matches!(self, Self::Requires { .. })
    }

    /// Whether the walk evaluates it while its entry is held: every kind but `Requires` (compiled into the entry's
    /// prerequisite, and gone from the checked data the engine takes) and `NoSelectionIf` (read where its selection is
    /// granted).
    pub(crate) fn applies_while_held(&self) -> bool {
        !matches!(
            self,
            Self::Requires { .. } | Self::NoSelectionIf { .. } | Self::GearStat { .. }
        )
    }
}

/// The `Requires` components of an entry as one [`Prerequisite`]: an `All` of the `All` group's conditions and, when
/// the `Any` group has any, an `Any` of them (`BlueprintFeature.MeetsPrerequisites`), or the lone condition left;
/// `None` without one. `archetype_class` gives the class an archetype belongs to (an `ArchetypeLevel` reads it).
pub fn compile_requires<'a>(
    components: &[Component],
    archetype_class: impl Fn(&ArchetypeKey) -> Option<&'a ClassKey>,
) -> Option<Prerequisite> {
    let (mut all, mut any) = (Vec::new(), Vec::new());
    for component in components {
        if let Component::Requires { condition, group } = component {
            let compiled = condition.compile(&archetype_class);
            match group {
                Group::All => all.push(compiled),
                Group::Any => any.push(compiled),
            }
        }
    }
    if !any.is_empty() {
        all.push(lone_or(any, Prerequisite::Any));
    }
    (!all.is_empty()).then(|| lone_or(all, Prerequisite::All))
}

impl GameData {
    /// Compiles every picked entry's `Requires` into its `prerequisite` ([`compile_requires`]) and takes them out of
    /// its components, and puts every entry's tree, compiled or hand-written, in its normal form
    /// ([`Prerequisite::canonicalize`], ticket 332): the data [`checked`](GameData::checked) holds, so the engine, the
    /// MCP server, the game check and the browser's JSON read one tree in one shape. An entry without `Requires` keeps
    /// its field; the check refuses an entry holding both.
    pub(crate) fn compile_prerequisites(&mut self) {
        let archetype_classes: HashMap<ArchetypeKey, ClassKey> = (self.archetypes.iter())
            .map(|archetype| (archetype.key.clone(), archetype.class.clone()))
            .collect();
        let compile = |prerequisite: &mut Option<Prerequisite>, components: &mut Vec<Component>| {
            if let Some(compiled) =
                compile_requires(components, |archetype| archetype_classes.get(archetype))
            {
                *prerequisite = Some(compiled);
                components.retain(|component| !component.is_requires());
            }
            if let Some(tree) = prerequisite {
                tree.canonicalize();
            }
        };
        for feat in &mut self.feats {
            compile(&mut feat.prerequisite, &mut feat.components);
        }
        for option in &mut self.options {
            compile(&mut option.prerequisite, &mut option.components);
        }
        for ability in &mut self.mythic_abilities {
            compile(&mut ability.prerequisite, &mut ability.components);
        }
        for feat in &mut self.mythic_feats {
            compile(&mut feat.prerequisite, &mut feat.components);
        }
        for class in self.all_classes_mut() {
            compile(&mut class.prerequisite, &mut class.components);
        }
        for archetype in &mut self.archetypes {
            compile(&mut archetype.prerequisite, &mut archetype.components);
        }
    }
}

/// A group's lone condition itself, or the group of them.
fn lone_or(
    mut children: Vec<Prerequisite>,
    group: fn(Vec<Prerequisite>) -> Prerequisite,
) -> Prerequisite {
    if children.len() == 1 {
        children.remove(0)
    } else {
        group(children)
    }
}

impl Condition {
    /// The planner's prerequisite for the condition.
    pub(crate) fn compile<'a>(
        &self,
        archetype_class: &impl Fn(&ArchetypeKey) -> Option<&'a ClassKey>,
    ) -> Prerequisite {
        use Prerequisite as P;
        let each = |conditions: &[Condition]| -> Vec<Prerequisite> {
            conditions
                .iter()
                .map(|condition| condition.compile(archetype_class))
                .collect()
        };
        let one_of = |leaves| lone_or(leaves, P::Any);
        match self {
            Self::Has(fact) => fact.leaf(),
            Self::HasNot(fact) => P::Not(Box::new(fact.leaf())),
            Self::FactRank(Fact::Feature(feature), rank) => P::Feature {
                feature: feature.clone(),
                count: (*rank).max(1),
            },
            Self::FactRank(fact, rank) if *rank <= 1 => fact.leaf(),
            Self::FactRank(fact, rank) => P::Rank {
                fact: fact.clone(),
                rank: *rank,
            },
            Self::AtLeast(n, facts) => {
                P::AtLeast((*n).max(1), facts.iter().map(Fact::leaf).collect())
            }
            Self::ClassLevel(class, level) => P::ClassLevel {
                class: class.clone(),
                level: *level,
            },
            Self::NoClassLevel(class) => P::Not(Box::new(P::ClassLevel {
                class: class.clone(),
                level: 1,
            })),
            Self::ArchetypeLevel(archetype, level) => {
                let taken = P::Archetype(archetype.clone());
                match archetype_class(archetype) {
                    Some(class) => P::All(vec![
                        taken,
                        P::ClassLevel {
                            class: class.clone(),
                            level: *level,
                        },
                    ]),
                    None => taken,
                }
            }
            Self::NoArchetype(archetype) => P::Not(Box::new(P::Archetype(archetype.clone()))),
            Self::CharacterLevel(level) => P::CharacterLevel(*level),
            Self::MythicLevel(rank) => P::MythicRank(*rank),
            Self::Stat { stat, value, full } => match stat {
                StatRef::Ability(ability) if *full => P::FullAbilityScore {
                    ability: *ability,
                    score: *value,
                },
                StatRef::Ability(ability) => P::AbilityScore {
                    ability: *ability,
                    score: *value,
                },
                // A skill's full value has no prerequisite leaf: the check refuses it (`FullSkillValue`).
                StatRef::Skill(skill) => P::SkillRanks {
                    skill: skill.clone(),
                    ranks: *value,
                },
                // The base attack bonus's permanent value is its full one
                // (`ModifiableValue.CalculatePermanentValue` filters nothing for it).
                StatRef::BaseAttack => P::BaseAttackBonus(*value),
                // No prerequisite reads it (the check refuses one that does): never met.
                StatRef::Parameter => P::Not(Box::new(P::All(Vec::new()))),
            },
            Self::ClassSpellLevel(class, level) => P::ClassSpellLevel {
                class: class.clone(),
                level: *level,
            },
            Self::CasterTypeSpellLevel {
                caster,
                level,
                spontaneous,
            } => {
                if *spontaneous {
                    P::SpontaneousSpellLevel {
                        spell_type: *caster,
                        level: *level,
                    }
                } else {
                    P::SpellLevel {
                        spell_type: *caster,
                        level: *level,
                    }
                }
            }
            Self::CasterType(caster) => P::CasterType(*caster),
            Self::Alignment(alignments) => {
                one_of(alignments.iter().copied().map(P::Alignment).collect())
            }
            Self::Proficient(armour) => {
                one_of(armour.iter().map(|group| P::Feat(group.feat())).collect())
            }
            Self::NotProficient(armour) => one_of(
                armour
                    .iter()
                    .map(|group| P::Not(Box::new(P::Feat(group.feat()))))
                    .collect(),
            ),
            Self::ParametrizedFeature(feat, value) => P::FeatFor {
                feat: feat.clone(),
                value: value.clone(),
            },
            Self::All(conditions) => P::All(each(conditions)),
            Self::Any(conditions) => P::Any(each(conditions)),
            Self::Not(condition) => P::Not(Box::new(condition.compile(archetype_class))),
        }
    }

    /// Whether `test` holds of the condition or of one inside it.
    pub fn any(&self, test: &impl Fn(&Condition) -> bool) -> bool {
        test(self)
            || match self {
                Self::All(conditions) | Self::Any(conditions) => {
                    conditions.iter().any(|condition| condition.any(test))
                }
                Self::Not(condition) => condition.any(test),
                _ => false,
            }
    }

    /// The facts the condition names, anywhere in it.
    pub fn facts(&self) -> Vec<&Fact> {
        match self {
            Self::Has(fact) | Self::HasNot(fact) | Self::FactRank(fact, _) => vec![fact],
            Self::AtLeast(_, facts) => facts.iter().collect(),
            Self::All(conditions) | Self::Any(conditions) => {
                conditions.iter().flat_map(Condition::facts).collect()
            }
            Self::Not(condition) => condition.facts(),
            _ => Vec::new(),
        }
    }
}

fn one() -> i8 {
    1
}

fn is_one(value: &i8) -> bool {
    *value == 1
}

fn is_zero_i8(value: &i8) -> bool {
    *value == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// The level a [`GrantAtClassLevel`](Component::GrantAtClassLevel) reads: the classes' levels added up, a class
/// counting only taken with one of `archetypes` when they are its own (`ReplaceCasterLevelOfAbility.CheckArchetype`);
/// no classes: the character level (`CalculateClassLevel` with no class).
pub(crate) fn grant_level<'a>(
    state: &CharacterState,
    classes: &BTreeSet<ClassKey>,
    archetypes: &BTreeSet<ArchetypeKey>,
    archetype_class: &dyn Fn(&ArchetypeKey) -> Option<&'a ClassKey>,
) -> u8 {
    if classes.is_empty() {
        return state.character_level;
    }
    state
        .classes
        .iter()
        .filter(|entry| classes.contains(&entry.class))
        .filter(|entry| {
            // `CheckArchetype`: no archetypes, or some not of this class, counts the class as it is.
            let all_own = !archetypes.is_empty()
                && archetypes
                    .iter()
                    .all(|archetype| archetype_class(archetype) == Some(&entry.class));
            !all_own
                || entry
                    .archetype
                    .as_ref()
                    .is_some_and(|taken| archetypes.contains(taken))
        })
        .fold(0u8, |sum, entry| sum.saturating_add(entry.level))
}

/// Whether a [`GrantAtClassLevel`](Component::GrantAtClassLevel) holds its fact at `level`:
/// `AddFeatureOnClassLevel.IsFeatureShouldBeApplied` (at `level` or more, or below it with `before`).
pub(crate) fn grants_at(level: u8, at: u8, before: bool) -> bool {
    (level >= at) != before
}

impl StatRule {
    /// `ReplaceStatForPrerequisites.ResultStat` for a stat whose value is `old`; `NewStat` reads the other ability in
    /// `scores`: the sheet's (`ModifiedValue`) for a prerequisite, the permanent ones for a full one (`fullValue`).
    pub(crate) fn apply(&self, old: i16, state: &CharacterState, scores: &AbilityScores) -> i16 {
        let level = |class: &ClassKey| i16::from(state.class_level(class));
        match self {
            Self::Ability(ability) => old.max(i16::from(scores.get(*ability))),
            Self::ClassLevel(class) => old.max(level(class)),
            Self::PlusClassLevel(class) => old + level(class),
            Self::AtLeast(number) => old.max(i16::from(*number)),
            Self::Plus(number) => old + i16::from(*number),
            // `(int)((double)(float)(level × 0.25f) + 0.75)`: a quarter of the levels plus ¾, truncated.
            Self::MagusBaseAttack(class) => old + (level(class) + 3) / 4,
        }
    }
}
