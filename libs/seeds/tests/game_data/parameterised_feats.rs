//! Parameterised and repeatable feats (ticket 27).

use std::collections::{BTreeSet, HashMap};

use rules::{
    ArmourType, FeatKey, Issue, Parameter, ParameterKind, Pick, Prerequisite, Repeat,
    SelectionOptionKey, SlotKind, SpellSchool, Unmet, WeaponProficiency,
};

use crate::support::*;

fn feat_for(feat: &str, value: Parameter) -> Prerequisite {
    Prerequisite::FeatFor {
        feat: FeatKey::new(feat),
        value,
    }
}

#[test]
fn the_weapons_are_the_guides_proficiency_lists_and_what_weapon_focus_names() {
    let count = |proficiency| {
        DATA.weapons
            .iter()
            .filter(|w| w.proficiency == proficiency)
            .count()
    };
    // Game: Enums/WeaponCategoryExtension.Data, WrathDecompiled 6ce4a6b: 15 categories with the Simple subcategory.
    assert_eq!(count(Some(WeaponProficiency::Simple)), 15);
    // The Exotic Weapon Proficiency list and the Sawtooth Saber (Weapon Finesse's exotic list; ticket 32).
    // Game: Enums/WeaponCategoryExtension.Data, WrathDecompiled 6ce4a6b: 17 categories with the Exotic subcategory.
    assert_eq!(count(Some(WeaponProficiency::Exotic)), 17);
}

/// The game's martial weapons are 29: the 25, the two shield bashes and the two spiked shields. Its natural attacks
/// also include the sting and the tentacle.
#[test]
#[ignore = "waits on ticket 289: the game gives 29 martial weapons, and the sting and tentacle natural attacks"]
fn the_martial_weapons_include_the_spiked_shields_and_the_natural_attacks_include_sting_and_tentacle()
 {
    let count = |proficiency| {
        DATA.weapons
            .iter()
            .filter(|w| w.proficiency == proficiency)
            .count()
    };
    // Game: Enums/WeaponCategoryExtension.Data, WrathDecompiled 6ce4a6b: 29 categories with the Martial subcategory,
    // SpikedLightShield and SpikedHeavyShield among them.
    assert_eq!(count(Some(WeaponProficiency::Martial)), 29);
    // Unarmed strike, touch attacks and rays, the natural attacks, Kinetic Blast and Bomb.
    // Game: Enums/WeaponCategoryExtension.Data, WrathDecompiled 6ce4a6b: the Natural categories include Sting and
    // Tentacle (OtherNaturalWeapons is Disabled).
    let others: BTreeSet<&str> = DATA
        .weapons
        .iter()
        .filter(|w| w.proficiency.is_none())
        .map(|w| w.key.as_str())
        .collect();
    assert_eq!(
        others,
        BTreeSet::from([
            "bite",
            "bomb",
            "claw",
            "gore",
            "hoof",
            "kinetic-blast",
            "rays",
            "slam",
            "spike",
            "sting",
            "tail",
            "talon",
            "tentacle",
            "touch-attacks",
            "unarmed-strike",
            "wing"
        ])
    );
}

#[test]
fn feats_repeat_and_take_values_as_the_guide_says() {
    use ParameterKind as K;
    let shape = |key| (feat(key).parameter, feat(key).repeat);
    // Game: WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e, SpellFocus 16fa59cc9a72a6043b566b49184f53fe and
    // ImprovedCritical f4201c85a991369408740c6888362e20 are parametrised, once per value
    // (Blueprints/Classes/Selection/BlueprintParametrizedFeature.CanSelect, WrathDecompiled 6ce4a6b);
    // SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a, ExoticWeaponProficiencySelection
    // 9a01b6815d6c3684cb25f30b8bf20932, ElementalFocusSelection bb24cc01319528849b09a3ae8eec0b31 and
    // ArmorFocusSelection 76d4885a395976547a13c5d6bf95b482 offer one feature per value (equivalent model: a feat with a
    // value).
    assert_eq!(shape("weapon-focus"), (Some(K::Weapon), Repeat::PerValue));
    assert_eq!(shape("skill-focus"), (Some(K::Skill), Repeat::PerValue));
    assert_eq!(
        shape("exotic-weapon-proficiency"),
        (Some(K::ExoticWeapon), Repeat::PerValue)
    );
    assert_eq!(
        shape("spell-focus"),
        (Some(K::SpellSchool), Repeat::PerValue)
    );
    assert_eq!(
        shape("elemental-focus"),
        (Some(K::Energy), Repeat::PerValue)
    );
    assert_eq!(
        shape("armor-focus"),
        (Some(K::ArmourType), Repeat::PerValue)
    );
    assert_eq!(
        shape("improved-critical"),
        (Some(K::Weapon), Repeat::PerValue)
    );
    // Game: ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35, capped at 3 by its own PrerequisiteCondition
    // (UnitLogic/Mechanics/Conditions/ContextConditionBuffRank.CheckCondition, WrathDecompiled 6ce4a6b).
    assert_eq!(shape("extra-hex"), (None, Repeat::Times(3)));
    // Game: MagicalTail febb8fe9a2d142fb80c1be6b0b539d9d, Ranks 8.
    assert_eq!(shape("magical-tail"), (None, Repeat::Times(8)));
    let repeating: Vec<&str> = DATA
        .feats
        .iter()
        .filter(|f| f.repeat != Repeat::Once)
        .map(|f| f.key.as_str())
        .collect();
    // The 16 parameterised feats once per value (ticket 32; Stat Focus since ticket 125), Magical Tail and the 18
    // Extra feats that repeat (not Extra Bane Weapon).
    // Game: each of the 35 is a BlueprintParametrizedFeature, a selection (BlueprintFeatureSelection.CanSelect skips
    // the rank check for one, WrathDecompiled 6ce4a6b) or a feature with Ranks above 1; every other feat's blueprint
    // has Ranks 1.
    assert_eq!(repeating.len(), 35, "{repeating:?}");
}

/// The game caps the Extra feats that are plain features at their blueprint's ranks: Extra Ki and Extra Reservoir
/// at 20, the other seven at 10.
#[test]
#[ignore = "waits on ticket 250: the game gives Extra Ki 20 ranks, and Extra Rage and six others 10"]
fn the_extra_feats_repeat_up_to_their_blueprints_ranks() {
    let repeat = |key| feat(key).repeat;
    // Game: ExtraKi 231a2a603d0b437e939553e6da3e7247, Ranks 20.
    assert_eq!(repeat("extra-ki"), Repeat::Times(20));
    // Game: ExtraReservoir 29afc53efa4d469b9476760fa09576de, Ranks 20.
    assert_eq!(repeat("extra-reservoir"), Repeat::Times(20));
    // Game: ExtraRage 1a54bbbafab728348a015cf9ffcf50a7, ExtraChannel cd9f19775bd9d3343a31a065e93f0c47,
    // ExtraLayOnHands a2b2f20dfb4d3ed40b9198e22be82030, ExtraPerformance 0d3651b2cb0d89448b112e23214e744e,
    // ExtraBombs 54c57ce67fa1d9044b1b3edc459e05e2, ExtraBloodRage 7624c6e46551720459cf5292da67d2f9 and
    // ExtraArcanePool 42f96fc8d6c80784194262e51b0a1d25: Ranks 10 each.
    for key in [
        "extra-rage",
        "extra-channel",
        "extra-lay-on-hands",
        "extra-performance",
        "extra-bombs",
        "extra-bloodrage",
        "extra-arcane-pool",
    ] {
        assert_eq!(repeat(key), Repeat::Times(10), "{key}");
    }
}

#[test]
fn weapon_focus_is_taken_once_per_weapon_and_specialization_needs_the_same_weapon() {
    let mut build = build("fighter", 4);
    build.picks.insert(
        class_slot(1),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
    );
    build.picks.insert(
        general_slot(1),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("dagger")),
    );
    build.picks.insert(
        class_slot(2),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("dagger")),
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("rapier")),
    );
    // Game: Blueprints/Classes/Selection/BlueprintParametrizedFeature.CanSelect, WrathDecompiled 6ce4a6b: a held
    // parameter is refused.
    assert_eq!(
        why_not(&without(&build, &class_slot(4)), &class_slot(2)),
        Some(Issue::AlreadyHeld)
    );
    // Game: WeaponSpecialization 31470b17e8446ae4ea0dacd6c5817d86's m_Prerequisite is WeaponFocus, for the same
    // parameter.
    assert_eq!(
        why_not(&without(&build, &class_slot(2)), &class_slot(4)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::FeatForSameValue(FeatKey::new("weapon-focus")),
            has: None,
        }))
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword")),
    );
    build.picks.remove(&class_slot(2));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 4, "weapon-focus"),
        BTreeSet::from([weapon("dagger"), weapon("longsword")])
    );
}

#[test]
fn prerequisites_name_the_values_the_guide_names() {
    let has = |key: &str, needs: &Prerequisite| {
        feat(key)
            .prerequisite
            .iter()
            .flat_map(Prerequisite::leaves)
            .any(|leaf| leaf == needs)
    };
    let same = |key: &str| Prerequisite::FeatForSameValue(FeatKey::new(key));
    // Game: DuelingMastery c3a66c1bbd2fb65498b130802d5f183a: Weapon Focus (dueling sword).
    assert!(has(
        "aldori-dueling-mastery",
        &feat_for("weapon-focus", weapon("dueling-sword"))
    ));
    // Game: AcrobaticSpellcaster 7ba4b4023a7748669098c479dd9e8de6: SkillFocusAcrobatics (the game's Mobility).
    assert!(has(
        "acrobatic-spellcaster",
        &feat_for("skill-focus", skill("mobility"))
    ));
    // Game: SlayersFeint 39425a13df904e3cb0e3f6debfe65cab: SkillFocusAcrobatics, one way in of two.
    assert!(has(
        "slayers-feint",
        &feat_for("skill-focus", skill("mobility"))
    ));
    // Game: AugmentSummoning 38155ca9e4055bb48a89240a2055dcc3: Spell Focus (Conjuration).
    assert!(has(
        "augment-summoning",
        &feat_for(
            "spell-focus",
            Parameter::SpellSchool(SpellSchool::Conjuration)
        )
    ));
    // Game: FencingGrace 47b352ea0f73c354aba777945760b441 (PrerequisiteParametrizedWeaponSubcategory
    // OneHandedPiercing): these six are among its weapons; the rest wait in
    // fencing_grace_takes_weapon_focus_in_any_one_handed_piercing_weapon.
    for weapon_ in [
        "dagger",
        "estoc",
        "punching-dagger",
        "rapier",
        "sai",
        "starknife",
    ] {
        assert!(has(
            "fencing-grace",
            &feat_for("weapon-focus", weapon(weapon_))
        ));
    }
    // Game: WeaponFocusGreater 09c9e82965fb4334b984a1e9df3bd088, WeaponSpecialization 31470b17e8446ae4ea0dacd6c5817d86,
    // WeaponSpecializationGreater 7cf5edc65e785a24f9cf93af987d66b3, SpellFocusGreater 5b04b45b228461c43bad768eb0f7c7bf:
    // each m_Prerequisite is the lesser feat at the same parameter; GreaterElementalFocusFire
    // 7a722c3e782aa5349a867c3516a2a4cf needs ElementalFocusFire (equivalent model: one feature per element).
    assert!(has("greater-weapon-focus", &same("weapon-focus")));
    assert!(has("weapon-specialization", &same("weapon-focus")));
    assert!(has(
        "greater-weapon-specialization",
        &same("weapon-specialization")
    ));
    assert!(has("greater-spell-focus", &same("spell-focus")));
    assert!(has("greater-elemental-focus", &same("elemental-focus")));

    // The mythic Armor Focus feats name their armour type.
    // Game: ArmorFocusLightMythicOffenceFeature 48168449f8ba465fab2843ba2dada063: ArmorFocusLight.
    let light = DATA
        .mythic_feats
        .iter()
        .find(|f| f.key.as_str() == "mythic-armor-focus-light-armor-assault")
        .unwrap();
    assert_eq!(
        light.prerequisite,
        Some(feat_for(
            "armor-focus",
            Parameter::ArmourType(ArmourType::Light)
        ))
    );
}

/// The game's Fencing Grace takes Weapon Focus in any weapon of the one-handed piercing subcategory.
#[test]
#[ignore = "waits on ticket 289: the game gives Fencing Grace every one-handed piercing weapon"]
fn fencing_grace_takes_weapon_focus_in_any_one_handed_piercing_weapon() {
    let has = |needs: &Prerequisite| {
        feat("fencing-grace")
            .prerequisite
            .iter()
            .flat_map(Prerequisite::leaves)
            .any(|leaf| leaf == needs)
    };
    // Game: FencingGrace 47b352ea0f73c354aba777945760b441 (PrerequisiteParametrizedWeaponSubcategory
    // OneHandedPiercing); Enums/WeaponCategoryExtension.Data, WrathDecompiled 6ce4a6b: the subcategory's enabled
    // weapons (the siangham is Disabled).
    for weapon_ in [
        "shortspear",
        "trident",
        "light-pick",
        "heavy-pick",
        "tongi",
        "spiked-light-shield",
        "spiked-heavy-shield",
        "bite",
        "claw",
    ] {
        assert!(has(&feat_for("weapon-focus", weapon(weapon_))), "{weapon_}");
    }
}

#[test]
fn an_aldori_swordlord_needs_weapon_focus_in_the_dueling_sword() {
    let swordlord = DATA
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "aldori-swordlord")
        .unwrap();
    // Game: SwordlordClass 90e4d7da3ccd1a8478411e07e91d5750: PrerequisiteParametrizedFeature WeaponFocus
    // (DuelingSword), not plain Weapon Focus.
    let Some(Prerequisite::All(lines)) = &swordlord.prerequisite else {
        panic!("{:?}", swordlord.prerequisite)
    };
    assert!(lines.contains(&feat_for("weapon-focus", weapon("dueling-sword"))));
    assert!(!lines.contains(&Prerequisite::Feat(FeatKey::new("weapon-focus"))));
}

#[test]
fn a_warpriest_takes_weapon_focus_in_a_weapon_of_his_choosing() {
    // Base, Champion of the Faith and Feral Champion: a bonus slot of Weapon Focus alone, prerequisites ignored
    // (a warpriest 1 has BAB 0).
    // Game: WarpriestProgression 062b215705430ad47b7174c1e75d285e, level 1: WarpriestWeaponFocusSelection
    // ac384183dbfbbd7499410a21d749bef1 (IgnorePrerequisites, one option: WeaponFocus); ChampionOfTheFaithArchetype
    // ad56b5819c4072345bff9aff6f1df13c and FeralChampionArchetype f68ca492c9c15e241ab73735fbd0fb9f keep it. Which
    // weapons are open waits in a_warpriests_weapon_focus_slot_opens_only_the_weapons_he_is_proficient_with.
    for archetype in [
        None,
        Some("warpriest/champion-of-the-faith"),
        Some("warpriest/feral-champion"),
    ] {
        let mut build = match archetype {
            None => build("warpriest", 1),
            Some(archetype) => archetype_build("warpriest", archetype, 1),
        };
        let slot = class_slot(1);
        let options = engine().options(&build, &slot).unwrap();
        assert!(
            options
                .iter()
                .all(|option| option.pick.key() == "weapon-focus"),
            "{archetype:?}"
        );
        assert_eq!(options.len(), DATA.weapons.len());
        build.picks.insert(
            slot,
            Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
        );
        assert!(engine().is_possible(&build), "{archetype:?}");
        assert_eq!(
            values_at(&build, 1, "weapon-focus"),
            BTreeSet::from([weapon("longsword")])
        );
    }
    // A Cult Leader doesn't get it.
    // Game: CultLeaderArchetype cd2d419961bb019438b77d4aecc2fc04 removes WarpriestWeaponFocusSelection at 1.
    let build = archetype_build("warpriest", "warpriest/cult-leader", 1);
    assert_eq!(engine().options(&build, &class_slot(1)), None);
}

/// The game's Weapon Focus keeps its proficiency check where the slot ignores prerequisites: the warpriest's slot
/// opens only the weapons he is proficient with.
#[test]
#[ignore = "waits on ticket 251: the game gives the warpriest's slot only the weapons he is proficient with"]
fn a_warpriests_weapon_focus_slot_opens_only_the_weapons_he_is_proficient_with() {
    // Game: WarpriestWeaponFocusSelection ac384183dbfbbd7499410a21d749bef1 (IgnorePrerequisites); WeaponFocus
    // 1e1f627d26ad36f43bbd26cc2bf8ac7e has RequireProficiency, which
    // Blueprints/Classes/Selection/BlueprintParametrizedFeature.CanSelect (WrathDecompiled 6ce4a6b) checks on the
    // feature's own flag, not the selection's.
    for archetype in [
        None,
        Some("warpriest/champion-of-the-faith"),
        Some("warpriest/feral-champion"),
    ] {
        let build = match archetype {
            None => build("warpriest", 1),
            Some(archetype) => archetype_build("warpriest", archetype, 1),
        };
        let options = engine().options(&build, &class_slot(1)).unwrap();
        let status = |weapon_: &str| {
            &options
                .iter()
                .find(|option| option.pick.parameter() == Some(&weapon(weapon_)))
                .unwrap_or_else(|| panic!("{weapon_} offered"))
                .status
        };
        assert!(status("longsword").is_ok(), "{archetype:?}");
        assert!(status("sawtooth-saber").is_err(), "{archetype:?}");
    }
}

#[test]
fn a_master_of_all_takes_three_skill_focus_in_three_skills() {
    let mut build = archetype_build("rogue", "rogue/master-of-all", 15);
    // "Prerequisites: Half-Elf".
    // Game: MasterOfAllArchetype bd4e70bfb89a452b876713d61b9b8eb2: MasterOfAllSkillFocus
    // f2d2c1702d8a4cc6adfcbd4ebff8eee4, a selection of SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a, whose
    // per-skill features have Ranks 1.
    build.start.race = Some(rules::RaceKey::new("half-elf"));
    for (level, skill_) in [(3, "mobility"), (9, "perception"), (15, "persuasion")] {
        build.picks.insert(
            class_slot(level),
            Pick::feat_for(FeatKey::new("skill-focus"), skill(skill_)),
        );
    }
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 15, "skill-focus"),
        BTreeSet::from([skill("mobility"), skill("perception"), skill("persuasion")])
    );
    // Not twice in one skill.
    build.picks.insert(
        class_slot(15),
        Pick::feat_for(FeatKey::new("skill-focus"), skill("mobility")),
    );
    assert_eq!(why_not(&build, &class_slot(15)), Some(Issue::AlreadyHeld));
}

#[test]
fn the_rogue_talents_weapon_focus_and_skill_focus_name_their_value() {
    let mut build = build("rogue", 2);
    let talent = |key: &str, value| {
        Pick::option_for(
            SelectionOptionKey::new(format!("rogue/rogue-talents/{key}")),
            value,
        )
    };
    // Game: WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e and SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a
    // carry the RogueTalent group.
    build
        .picks
        .insert(selection_slot(2), talent("weapon-focus", weapon("rapier")));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "weapon-focus"),
        BTreeSet::from([weapon("rapier")])
    );
    build
        .picks
        .insert(selection_slot(2), talent("skill-focus", skill("stealth")));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "skill-focus"),
        BTreeSet::from([skill("stealth")])
    );
}

#[test]
fn rage_powers_discoveries_incense_and_mutations_repeat_as_the_guide_says() {
    // The two whose repeat differs from the game's wait in their own tests: New Strain in
    // plague_mutation_new_strain_repeats_four_times, the rogue's Weapon Focus talent in repeating_talents.rs.
    const WAITING: [&str; 2] = [NEW_STRAIN, "rogue/rogue-talents/weapon-focus"];
    let options: HashMap<&str, Repeat> = DATA
        .class_selections()
        .flat_map(|selection| &selection.options)
        .filter(|option| option.repeat != Repeat::Once && !WAITING.contains(&option.key.as_str()))
        .map(|option| (option.key.as_str(), option.repeat))
        .collect();
    let incense =
        |key: &str| format!("alchemist/incense-synthesizer/incense-fog/incense-fog-{key}");
    let mut expected: HashMap<String, Repeat> = [
        // Game: IncreasedDamageReductionFeature ddaee203ee4dcb24c880d633fbd77db6, Ranks 3.
        (
            "barbarian/rage-power/increased-damage-reduction",
            Repeat::Times(3),
        ),
        // Game: SwiftFootFeature 7df32d4e9bd2cdc48b0f69b03a57754a, Ranks 3.
        ("barbarian/rage-power/swift-foot", Repeat::Times(3)),
        // Game: IncreasedDamageReductionDefensivePower d10496e92d0799a40bb3930b8f4fda0d, Ranks 2.
        (
            "stalwart-defender/defensive-powers/increased-damage-reduction",
            Repeat::Times(2),
        ),
        // Game: PreserveOrgans 76b4bb8e54f3f5c418f421684c76ef4e, Ranks 3.
        ("alchemist/discovery/preserve-organs", Repeat::Times(3)),
        // "At 4th level and every four levels thereafter, a Primalist can choose ... two Rage Powers."
        // Game: PrimalistTakeRagePowers4 8eb5c34bb8471a0438e7eb3994de3b92 and its copies to 20, one per level, each
        // once (equivalent model: one option taken at each of those levels).
        (
            "bloodrager/primalist/primal-choices/rage-powers",
            Repeat::Unlimited,
        ),
    ]
    .into_iter()
    .map(|(key, repeat)| (key.to_owned(), repeat))
    .collect();
    // Game: IncenseFogClarifyingIncenseFeature c16dc187fd12ac54f83007ec4c0a676a,
    // IncenseFogNeutralizingIncenseFeature 6dd6caccd12ff6a4d9c12d36980e1906, IncenseFogRallyingIncenseFeature
    // c5f14b5981ec3994c80def79b6f5a77a, IncenseFogToxicIncenseFeature 8241699d20232084380fec8152434f2a,
    // IncenseFogUnsettlingFogFeature 57a418cf996beeb4a9b1b3638cfc000e: Ranks 3; IncenseFogImprovedIncenseFeature
    // ccea52d8cc5f8d34d95196d0a885be06: Ranks 2.
    for key in [
        "clarifying-incense",
        "neutralizing-incense",
        "rallying-incense",
        "toxic-incense",
        "unsettling-fog",
    ] {
        expected.insert(incense(key), Repeat::Times(3));
    }
    expected.insert(incense("improved-incense"), Repeat::Times(2));
    // "the Kineticist may choose a second Elemental Focus or increase her primary focus", "there isn't a third
    // level of specialization" (ticket 31).
    // Game: SecondatyElementalFocusSelection 4204bc10b3d5db440b1f52f0c375848b offers each element again;
    // ThirdElementalFocusSelection e2c1718828fc843479f18ab4d75ded86's ThirdElementAir
    // 651570c873e22b84f893f146ce2de502 (and each element's) has PrerequisiteNoFeature on the secondary one
    // (equivalent model: an element held at most twice).
    for element in ["air", "earth", "fire", "water"] {
        expected.insert(format!("kineticist/element/{element}"), Repeat::Times(2));
    }
    // The user's (ticket 47): the Skill Focus talents once per skill, Combat Trick any number of times.
    // Game: SkillFocusSelection c9629ef9eebb88b479b2fbc5e836656a, one feature per skill (equivalent model);
    // CombatTrick c5158a6622d0b694a99efb1d0025d2c1 is a selection, which skips the rank check
    // (Blueprints/Classes/Selection/BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b).
    expected.insert(
        "kineticist/wild-talents/skill-focus".into(),
        Repeat::PerValue,
    );
    expected.insert("rogue/rogue-talents/skill-focus".into(), Repeat::PerValue);
    expected.insert("rogue/rogue-talents/combat-trick".into(), Repeat::Unlimited);
    // The Battle Scion's teamwork feat in place of a rage power, as often as it likes (ticket 189).
    // Game: BattleScionTeamworkFeat 64960cdba39692243bef11da263ab7f3 is a selection, so it skips the rank check
    // (BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b; equivalent model: each teamwork feat once).
    expected.insert(
        "skald/battle-scion/rage-power-or-teamwork-feat/teamwork-feat".into(),
        Repeat::Unlimited,
    );
    // "She can select a second weapon at 11th level and a third at 19th level" (ticket 172): once per weapon.
    // Game: FinesseTrainingSelection b78d146cea711a84598f0acef69462ea, one feature per weapon (equivalent model).
    expected.insert("rogue/finesse-training/weapon".into(), Repeat::PerValue);
    let expected: HashMap<&str, Repeat> = expected
        .iter()
        .map(|(key, repeat)| (key.as_str(), *repeat))
        .collect();
    assert_eq!(options, expected);

    // A barbarian takes Increased Damage Reduction three times, not four.
    // Game: IncreasedDamageReductionFeature ddaee203ee4dcb24c880d633fbd77db6, Ranks 3.
    let mut build = build("barbarian", 16);
    align(&mut build, rules::Alignment::ChaoticNeutral);
    let rage_power_levels: Vec<u8> = (1..=16)
        .filter(|level| {
            engine()
                .evaluate(&build)
                .levels
                .get(usize::from(*level) - 1)
                .is_some_and(|l| {
                    l.slots.iter().any(|slot| {
                        slot.kind
                            == SlotKind::Selection(rules::FeatureSelectionKey::new(
                                "barbarian/rage-power",
                            ))
                    })
                })
        })
        .collect();
    let reduction = || pick("barbarian/rage-power/increased-damage-reduction");
    let later: Vec<u8> = rage_power_levels
        .iter()
        .copied()
        .filter(|level| *level >= 8)
        .take(4)
        .collect();
    assert_eq!(later.len(), 4, "{rage_power_levels:?}");
    for level in &later {
        build.picks.insert(selection_slot(*level), reduction());
    }
    assert_eq!(
        why_not(&build, &selection_slot(later[3])),
        Some(Issue::MaxTimes { times: 3 })
    );
}

const NEW_STRAIN: &str =
    "shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-new-strain";

/// The game's Plague Mutation: New Strain is taken up to four times.
#[test]
#[ignore = "waits on ticket 278: the game gives New Strain 4 ranks"]
fn plague_mutation_new_strain_repeats_four_times() {
    // Game: PlagueHexIncreaseDamageStepFeature 505648df95f5489fa13ddbd1af9473fd, Ranks 4.
    assert_eq!(selection_option(NEW_STRAIN).repeat, Repeat::Times(4));
}
