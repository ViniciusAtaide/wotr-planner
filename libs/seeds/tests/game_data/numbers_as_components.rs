//! Numbers as components (ticket 297): `bonuses.ron`'s rows, a feature's ability bonuses, floors and steps, a gear
//! item's bonuses and the attack and caster lines' named feats are the vocabulary's components on the real data, in the
//! game's terms where the planner's numbers already were the game's.

use rules::{
    Ability, Component, Descriptor, FeatKey, FeatureKey, GearTarget, MechanicsFlag, MetamagicKind,
    RankBase, RankConfig, RankProgression, Save, StatTarget, Value,
};

use crate::support::*;

fn feat_components(key: &str) -> &'static [Component] {
    &(DATA.feats.iter())
        .find(|feat| feat.key == FeatKey::new(key))
        .unwrap_or_else(|| panic!("no feat {key}"))
        .components
}

fn feature_components(key: &str) -> &'static [Component] {
    &(DATA.features.iter())
        .find(|feature| feature.key == FeatureKey::new(key))
        .unwrap_or_else(|| panic!("no feature {key}"))
        .components
}

/// The data's own folder holds no `bonuses.ron`: its rows are components, so no number comes from a file of its own.
#[test]
fn bonuses_ron_is_gone() {
    assert!(!seeds::game_data_dir().join("bonuses.ron").exists());
}

/// The attack line's feats hold the kinds the game's components give them.
#[test]
fn the_attack_and_caster_lines_feats_hold_their_kinds() {
    for (feat, kind) in [
        // Game: WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e, WeaponFocusGreater 09c9e82965fb4334b984a1e9df3bd088
        // (`WeaponFocusParametrized`)
        ("weapon-focus", Component::WeaponFocus),
        ("greater-weapon-focus", Component::WeaponFocus),
        // Game: WeaponSpecialization 31470b17e8446ae4ea0dacd6c5817d86, WeaponSpecializationGreater
        // 7cf5edc65e785a24f9cf93af987d66b3 (`WeaponSpecializationParametrized`, `Mythic` false)
        ("weapon-specialization", Component::WeaponSpecialization),
        (
            "greater-weapon-specialization",
            Component::WeaponSpecialization,
        ),
        // Game: ImprovedCritical f4201c85a991369408740c6888362e20 (`ImprovedCriticalParametrized`)
        ("improved-critical", Component::ImprovedCritical),
        // Game: TricksterImprovedImprovedCritical 56f94badbba018b4b8277ce6e2e79e72,
        // TricksterImprovedImprovedImprovedCritical 006a966007802a0478c9e21007207aac
        // (`ImprovedCriticalEdgeParametrized`)
        ("improved-improved-critical", Component::CriticalEdge),
        (
            "improved-improved-improved-critical",
            Component::CriticalEdge,
        ),
        // Game: TricksterImprovedImprovedImprovedCriticalImproved 319c882ab3cc51544ad2f3f43633d5b1
        // (`ImprovedCriticalMythicParametrized`)
        (
            "improved-improved-improved-critical-improved",
            Component::CriticalMultiplier,
        ),
        // Game: WeaponFinesse 90e54424d682d104ab36436bd527af09 (`AddMechanicsFeature` WeaponFinesse)
        (
            "weapon-finesse",
            Component::Flag(MechanicsFlag::WeaponFinesse),
        ),
        // Game: PowerAttackFeature 9972f33f977fc724c838e59641b2fca5 (`AddFacts` PowerAttackToggleAbility,
        // `PowerAttackWatcher`)
        ("power-attack", Component::PowerAttack),
        // Game: HeightenSpellFeat 2f5d1e705c7967546b72ad8218ccf99c (`AddMetamagicFeat` Heighten)
        (
            "metamagic-heighten-spell",
            Component::Metamagic(MetamagicKind::Heighten),
        ),
    ] {
        assert!(
            feat_components(feat).contains(&kind),
            "{feat}: {:?}",
            feat_components(feat)
        );
    }
}

/// Perfect Mind's floors: the base attack bonus at least 15 plus the Mythic rank, and every skill counting at least the
/// Mythic rank for its ranks, with its class skills.
// Game: DragonLevel2BABOverride f4af14e9f4408d44c830d891fee8f4a5 (`AddContextStatBonus` on BaseAttackBonus,
// Multiplier −1, HasMinimal 0, of a property of the base attack bonus less 15 and the Mythic rank: ticket 252);
// GoldenDragonSharedSkillUMD 4e0b919e4fbd85142bce959fae129d1a (`AddGoldenDragonSkillBonus` per skill,
// `UnitLogic/FactLogic/AddGoldenDragonSkillBonus.cs` `OnTurnOn`, WrathDecompiled 6ce4a6b)
#[test]
fn perfect_minds_floors_are_its_components() {
    let mind = feature_components("gold-dragon/perfect-mind");
    assert!(mind.contains(&Component::BaseAttackAtLeast(15)), "{mind:?}");
    assert!(mind.contains(&Component::GoldenDragonSkills), "{mind:?}");
}

/// The Dragon Disciple's ability increases are racial stat bonuses, one for each grant.
// Game: DragonDiscipleConstitution 95de611796547be489e527fae84abbd6 (`AddStatBonus` Constitution +2, `Racial`)
#[test]
fn the_dragon_disciples_constitution_is_a_racial_stat_bonus() {
    assert_eq!(
        feature_components("dragon-disciple/constitution-2"),
        [Component::StatBonus {
            stat: StatTarget::Ability(Ability::Constitution),
            value: Value::PerRank(2),
            descriptor: Descriptor::Racial,
            multiplier: 1,
            at_least: None,
            only_if: None,
        }]
    );
}

/// Infernal Resistance and Abyssal Bloodrage scale on bloodrager levels (ticket 254), a rank whose value is the step:
/// one at 5 and 8, and at 16 and 20.
// Game: BloodragerInfernalResistanceBuff 66c973ccad1293345ae6c5bfeced4a7b (two `ContextRankConfig`s on
// BloodragerClass, rising at 5 and 8); BloodragerAbyssalBloodrageBonusProperty 415f71e5a47f4cccb3dbd10bd7a0f8f8 (a
// class-level getter on BloodragerClass, 1 at 12–15, 2 at 16–19, 3 at 20)
#[test]
fn the_bloodrager_features_scale_on_bloodrager_levels() {
    let on_bloodrager = |bands: Vec<(i16, i16)>| {
        Component::Scales(RankConfig {
            base: RankBase::ClassLevel {
                classes: [rules::ClassKey::new("bloodrager")].into(),
                except: false,
            },
            progression: RankProgression::Custom(bands),
            min: None,
            max: None,
        })
    };
    assert_eq!(
        feature_components("bloodrager/bloodline/infernal/infernal-resistance"),
        [on_bloodrager(vec![(4, 0), (7, 1), (100, 2)])]
    );
    assert_eq!(
        feature_components("bloodrager/bloodline/abyssal/abyssal-bloodrage"),
        [on_bloodrager(vec![(15, 0), (19, 1), (100, 2)])]
    );
}

/// A gear item's bonuses are `GearStat`s with the game's types.
// Game: BeltOfAlacrityItem 4b0b9b95225d6034f84b75a6ad1e76c7: Dexterity4 15008553994dd7e48af2ad1a1426d733
// (`AddStatBonusEquipment` Dexterity +4, Enhancement) and BeltOfAlacrityEnchantment 3d2ebf32e8c47494ea1c1db6485e7d1e,
// whose wearer's feature adds +4 Competence to Reflex (the game check's items area compares them)
#[test]
fn a_gear_items_bonuses_are_gear_stats() {
    let belt = (DATA.gear.iter())
        .find(|item| item.key.as_str() == "belt-of-alacrity")
        .unwrap();
    assert_eq!(
        belt.stats().collect::<Vec<_>>(),
        [
            (
                &GearTarget::Ability(Ability::Dexterity),
                4,
                Descriptor::Enhancement
            ),
            (&GearTarget::Save(Save::Reflex), 4, Descriptor::Competence),
        ]
    );
}
