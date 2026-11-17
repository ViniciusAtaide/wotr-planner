//! Repeats from the header (ticket 303): how often an entry may be taken is derived from what the data holds of its
//! blueprint's header, as the game's `BlueprintFeatureSelection.CanSelect` derives it
//! (`Blueprints/Classes/Selection/BlueprintFeatureSelection.cs`, WrathDecompiled 6ce4a6b): a feature that isn't a
//! selection stops at its `Ranks`; a selection (a `BlueprintParametrizedFeature` is one too) skips that check and is
//! refused only once none of its options can be picked (`NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable`);
//! its own `PrerequisiteNoFeature` makes it once-only, and its own `PrerequisiteCondition` on its rank caps it.

use rules::{
    FeatKey, FeatureSelectionKey, Issue, MythicAbilityKey, MythicFeatKey, Pick, Repeat, SlotId,
    SlotKind,
};

use crate::support::*;

/// A fighter 20 on the Aeon path, with `picks` at its Mythic ranks.
fn aeon(picks: &[(SlotId, Pick)]) -> rules::Build {
    let mut build = mythic("aeon", 20);
    build.picks.extend(picks.iter().cloned());
    build
}

/// Asserts a Build possible, naming its failures when it isn't.
fn assert_possible(build: &rules::Build) {
    assert!(possible(build), "{:?}", engine().failures(build));
}

fn mythic_feat_pick(key: &str) -> Pick {
    Pick::mythic_feat(MythicFeatKey::new(key))
}

fn ability_pick(key: &str) -> Pick {
    Pick::mythic_ability(MythicAbilityKey::new(key))
}

/// Extra Feat (mythic) is taken twice, each a general feat.
#[test]
fn extra_feat_mythic_is_taken_twice() {
    // Game: ExtraFeatMythicFeat e10c4f18a6c8b4342afe6954bde0587b is a BlueprintFeatureSelection offered by
    // MythicFeatSelection 9ee0f6745f555484299b0a1563b99d81 (Mode Default), with no PrerequisiteNoFeature of itself
    // nor a PrerequisiteCondition: CanSelect skips the rank check for a selection, so it repeats while a feat is left.
    let build = aeon(&[
        (SlotId::Mythic { rank: 2 }, mythic_feat_pick("extra-feat")),
        (
            SlotId::MythicExtra { rank: 2 },
            feat_pick_plain("toughness"),
        ),
        (SlotId::Mythic { rank: 4 }, mythic_feat_pick("extra-feat")),
        (
            SlotId::MythicExtra { rank: 4 },
            feat_pick_plain("iron-will"),
        ),
    ]);
    assert_possible(&build);
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet
            .slot(&SlotId::MythicExtra { rank: 4 })
            .map(|slot| &slot.kind),
        Some(&SlotKind::Feat(FeatureSelectionKey::new(
            "mythic/extra-feat"
        )))
    );
    assert_eq!(at(&build, 20).feat_count(&FeatKey::new("iron-will")), 1);
}

/// Extra Mythic Ability is taken twice, each a mythic ability.
#[test]
fn extra_mythic_ability_is_taken_twice() {
    // Game: ExtraMythicAbilityMythicFeat 8a6a511c55e67d04db328cc49aaad2b8 is a BlueprintFeatureSelection offered by
    // MythicFeatSelection 9ee0f6745f555484299b0a1563b99d81 (Mode Default), with no PrerequisiteNoFeature of itself
    // nor a PrerequisiteCondition (CanSelect).
    let build = aeon(&[
        (
            SlotId::Mythic { rank: 2 },
            mythic_feat_pick("extra-mythic-ability"),
        ),
        (
            SlotId::MythicExtra { rank: 2 },
            ability_pick("always-a-chance"),
        ),
        (
            SlotId::Mythic { rank: 4 },
            mythic_feat_pick("extra-mythic-ability"),
        ),
        (SlotId::MythicExtra { rank: 4 }, ability_pick("best-jokes")),
    ]);
    assert_possible(&build);
    assert!(at(&build, 20).mythic_ability_count(&MythicAbilityKey::new("best-jokes")) > 0);
}

/// A cleric takes Impossible Domain twice, for two more domains, and not for a domain held.
#[test]
fn impossible_domain_is_taken_twice_for_another_domain() {
    // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf is a BlueprintFeatureSelection (Mode OnlyNew: a domain held
    // is refused) offered by MythicAbilitySelection ba0e5a900b775be4a99702f1ed08914d (Mode Default), with no
    // PrerequisiteNoFeature of itself: CanSelect lets it repeat while a domain is left.
    let mut build = cleric(["fire", "glory"], 20);
    build.mythic_path = Some(rules::MythicPathKey::new("aeon"));
    build.picks.extend([
        (
            SlotId::Mythic { rank: 1 },
            ability_pick("impossible-domain"),
        ),
        (
            SlotId::MythicSelection { rank: 1, index: 0 },
            pick("cleric/domain/nobility"),
        ),
        (
            SlotId::Mythic { rank: 3 },
            ability_pick("impossible-domain"),
        ),
        (
            SlotId::MythicSelection { rank: 3, index: 0 },
            pick("cleric/domain/war"),
        ),
    ]);
    let mut build = faithful(build);
    assert_possible(&build);
    let twenty = at(&build, 20);
    for domain in ["nobility", "war"] {
        assert_eq!(
            twenty.option_count(&rules::SelectionOptionKey::new(format!(
                "cleric/domain/{domain}"
            ))),
            1,
            "{domain}"
        );
    }
    // A domain held is refused (`OnlyNew`; the domain's own Ranks 1).
    build.picks.insert(
        SlotId::MythicSelection { rank: 3, index: 0 },
        pick("cleric/domain/fire"),
    );
    assert_eq!(
        why_not(&build, &SlotId::MythicSelection { rank: 3, index: 0 }),
        Some(Issue::AlreadyHeld)
    );
}

/// Second Bloodline is still taken once.
#[test]
fn second_bloodline_is_still_taken_once() {
    // Game: SecondBloodline 3cf2ab2c320b73347a7c21cf0d0995bd has a PrerequisiteNoFeature of itself (Group All):
    // BlueprintFeature.MeetsPrerequisites refuses it once held.
    let mut build = build("sorcerer", 20);
    build.mythic_path = Some(rules::MythicPathKey::new("aeon"));
    let mut build = with_selections_filled(build);
    build
        .picks
        .insert(SlotId::Mythic { rank: 1 }, ability_pick("second-bloodline"));
    build
        .picks
        .insert(SlotId::Mythic { rank: 3 }, ability_pick("second-bloodline"));
    assert_eq!(
        why_not(&build, &SlotId::Mythic { rank: 3 }),
        Some(Issue::AlreadyHeld)
    );
}

/// The mythic abilities and mythic feats that aren't taken once, each as its header says.
#[test]
fn the_mythic_entries_repeat_as_their_headers_say() {
    use std::collections::BTreeMap;
    let abilities: BTreeMap<&str, Repeat> = (DATA.mythic_abilities.iter())
        .filter(|ability| ability.repeat() != Repeat::Once)
        .map(|ability| (ability.key.as_str(), ability.repeat()))
        .collect();
    assert_eq!(
        abilities,
        BTreeMap::from([
            // Game: AscendantElementSelection 535d4e5d8a5664f4797560c280941782 and FavoriteMetamagicSelection
            // 503fb196aa222b24cb6cfdc9a284e838, selections of one-rank features (ticket 249), taken as a value.
            ("ascendant-element", Repeat::PerValue),
            ("favorite-metamagic", Repeat::PerValue),
            // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf, a selection with no PrerequisiteNoFeature of itself.
            ("impossible-domain", Repeat::WhileOptionLeft),
            // Not Witch Wandering Hex, whose selection is the planner's: `witch_wandering_hex_is_taken_once`.
        ])
    );
    let feats: BTreeMap<&str, Repeat> = (DATA.mythic_feats.iter())
        .filter(|feat| feat.repeat() != Repeat::Once)
        .map(|feat| (feat.key.as_str(), feat.repeat()))
        .collect();
    assert_eq!(
        feats,
        BTreeMap::from([
            // Game: ExpandedArsenalSchool f137089c48364014aa3ec3b92ccaf2e2, ImprovedCriticalMythicFeat
            // 8bc0190a4ec04bd489eec290aeaa6d07, SchoolMasteryMythicFeat ac830015569352b458efcdfae00a948c and
            // SpellFocusMythicFeat 41fa2470ab50ff441b4cfbb2fc725109 are parametrised; SkillFocusMythicFeatSelection
            // a224819b9492019428c220754208ba3a a selection of one feature per skill (BlueprintParametrizedFeature.
            // CanSelect: a held value refused).
            ("expanded-arsenal", Repeat::PerValue),
            ("improved-critical-mythic", Repeat::PerValue),
            ("school-mastery", Repeat::PerValue),
            ("skill-focus-mythic", Repeat::PerValue),
            ("spell-focus-mythic", Repeat::PerValue),
            // Game: ExtraFeatMythicFeat e10c4f18a6c8b4342afe6954bde0587b and ExtraMythicAbilityMythicFeat
            // 8a6a511c55e67d04db328cc49aaad2b8, selections with no condition of their own.
            ("extra-feat", Repeat::WhileOptionLeft),
            ("extra-mythic-ability", Repeat::WhileOptionLeft),
        ])
    );
}

/// The selections the game takes once by their own `PrerequisiteNoFeature` (Group All) hold it as `HasNot` of
/// themselves, so the planner's entry is taken once and the game check compares it (ticket 303).
#[test]
fn the_selections_with_their_own_no_feature_are_taken_once() {
    use rules::SelectionOptionKey;
    let not = |leaf: rules::Prerequisite| rules::Prerequisite::Not(Box::new(leaf));
    let holds = |prerequisite: Option<&rules::Prerequisite>, own: &rules::Prerequisite| {
        match prerequisite {
            Some(rules::Prerequisite::All(children)) => children.contains(own),
            Some(lone) => lone == own,
            None => false,
        }
    };
    // Game: AdvancedWeaponTraining1 3aa4cbdd4af5ba54888b0dc7f07f80c4.
    let feat = feat("advanced-weapon-training");
    let own = not(rules::Prerequisite::Feat(feat.key.clone()));
    assert!(holds(feat.prerequisite.as_ref(), &own));
    assert_eq!(feat.repeat(), Repeat::Once);
    // Game: SecondBloodline 3cf2ab2c320b73347a7c21cf0d0995bd, SecondBloodragerBloodline
    // b7f62628915bdb14d8888c25da3fac56, SecondMystery 277b0164740b97945a3f8022bd572f48, SecondSpirit
    // 2faa80662a56ab644aec2f875a68597f.
    for key in [
        "second-bloodline",
        "second-bloodrager-bloodline",
        "second-mystery",
        "second-spirit",
    ] {
        let ability = (DATA.mythic_abilities.iter())
            .find(|ability| ability.key.as_str() == key)
            .unwrap();
        let own = not(rules::Prerequisite::MythicAbility(ability.key.clone()));
        assert!(holds(ability.prerequisite.as_ref(), &own), "{key}");
        assert_eq!(ability.repeat(), Repeat::Once, "{key}");
    }
    // Game: HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894, ElementalWhispersSelection
    // f02525006521bee4eb90ab26b7b9db24, LoremasterCombatFeatSelection 90f105c8e31a6224ea319e6a810e4af8,
    // LoremasterRogueTalentSelection 4b7018b1ed4b27140a5e7adfacaaf9c6, LoremasterWizardFeatSelection
    // 689959eef3e972e458b52598dcc2c752, OracleRevelationBondedMount 0234d0dd1cead22428e71a2500afa2e1, Familiar
    // 363cab72f77c47745bf3a8807074d183, RangerStyleSelection2 c6d0da9124735a44f93ac31df803b9a9, RangerStyleSelection6
    // 61f82ba786fe05643beb3cd3910233a8, RangerStyleSelection10 78177315fc63b474ea3cbb8df38fafcd.
    for key in [
        "hellknight/disciplines/pentamic-faith",
        "kineticist/wild-talents/elemental-whispers",
        "loremaster/secret/combat-feat",
        "loremaster/secret/rogue-secret",
        "loremaster/secret/wizard-feat",
        "oracle/revelations/nature/animal-companion",
        "rogue/rogue-talents/familiar",
        "slayer/slayer-talents/combat-style-feat-2",
        "slayer/slayer-talents/combat-style-feat-6",
        "slayer/slayer-talents/combat-style-feat-10",
    ] {
        let option = selection_option(key);
        let own = not(rules::Prerequisite::SelectionOption(
            SelectionOptionKey::new(key),
        ));
        assert!(holds(option.prerequisite.as_ref(), &own), "{key}");
        assert_eq!(option.repeat(), Repeat::Once, "{key}");
    }
}

/// Witch Wandering Hex is taken once: its selection is the planner's way to give its hex, not the game's.
#[test]
fn witch_wandering_hex_is_taken_once() {
    // Game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 is a BlueprintFeature of Ranks 1, so
    // BlueprintFeatureSelection.CanSelect refuses it once held (WrathDecompiled 6ce4a6b).
    let ability = (DATA.mythic_abilities.iter())
        .find(|ability| ability.key.as_str() == "witch-wandering-hex")
        .unwrap();
    assert!(ability.plain_feature && ability.selection.is_some());
    assert_eq!(ability.repeat(), Repeat::Once);
    let mut build = classes_build(&["witch"; 20]);
    build.mythic_path = Some(rules::MythicPathKey::new("aeon"));
    let mut build = with_selections_filled(build);
    build.picks.extend([
        (
            SlotId::Mythic { rank: 1 },
            ability_pick("witch-wandering-hex"),
        ),
        (
            SlotId::Mythic { rank: 3 },
            ability_pick("witch-wandering-hex"),
        ),
    ]);
    assert_eq!(
        why_not(&build, &SlotId::Mythic { rank: 3 }),
        Some(Issue::AlreadyHeld)
    );
}
