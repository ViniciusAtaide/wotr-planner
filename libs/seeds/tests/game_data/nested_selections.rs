//! Nested selections (ticket 302): a feat, a mythic feat or a mythic ability that grants a pick names the selection it
//! grants, as the game's Extra feats and mythic abilities are themselves selections, and that selection's pick is
//! the slot it opens. Also the category selections of ticket 301 against the planner's feat categories.

use std::collections::BTreeSet;

use rules::{
    BASIC_FEAT_SELECTION, Band, FeatCategory, FeatKey, FeatureSelectionKey, MythicAbilityKey,
    SlotId, SlotKind,
};

use crate::support::*;

/// A blueprint GUID as text.
fn guid_of(blueprint: &Option<rules::BlueprintGuid>) -> Option<&str> {
    blueprint.as_ref().map(|guid| guid.as_str())
}

/// Each Extra feat that gives a pick grants a selection carrying the feat's own blueprint: the game's Extra feat is a
/// `BlueprintFeatureSelection`, so one GUID names both entries.
#[test]
fn each_extra_feat_grants_a_selection_with_its_blueprint() {
    let extra: [(&str, &str, &str); 11] = [
        // Game: ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 (BlueprintFeatureSelection)
        (
            "extra-arcana",
            "magus/extra-arcana",
            "00727883edf145e2a6bce9ad176ecfd8",
        ),
        // Game: ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a
        (
            "extra-arcanist-exploit",
            "arcanist/extra-arcanist-exploit",
            "0a7065e13f4449fa81bfb33199bf7f6a",
        ),
        // Game: ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66
        (
            "extra-discovery",
            "alchemist/extra-discovery",
            "537965879fc24ad3948aaffa7a1a3a66",
        ),
        // Game: ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae (306k)
        (
            "extra-discovery-vivisectionist",
            "alchemist/vivisectionist/extra-discovery",
            "10287e7b8cee479e82ea88bd6d2d4dae",
        ),
        // Game: ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35
        (
            "extra-hex",
            "witch/extra-hex",
            "b6054088b4ab4be286724127cbf48b35",
        ),
        // Game: ExtraShamanHexSelection d0b4c8245d504b8c9c6d3fccc1f8c5b6 (ticket 306j)
        (
            "extra-hex-shaman",
            "shaman/extra-hex",
            "d0b4c8245d504b8c9c6d3fccc1f8c5b6",
        ),
        // Game: ExtraMercySelection 8a49abed5be9473da1e1fd1e2457562e
        (
            "extra-mercy",
            "paladin/extra-mercy",
            "8a49abed5be9473da1e1fd1e2457562e",
        ),
        // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a
        (
            "extra-rage-power",
            "barbarian/extra-rage-power",
            "0c7f01fbbe687bb4baff8195cb02fe6a",
        ),
        // Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12
        (
            "extra-rogue-talent",
            "rogue/extra-rogue-talent",
            "14d9089df87a43b696fa9451ca2f0a12",
        ),
        // Game: ExtraSlayerTalentSelection 53f7237f5b1447bb851ba68045a00e41
        (
            "extra-slayer-talent",
            "slayer/extra-slayer-talent",
            "53f7237f5b1447bb851ba68045a00e41",
        ),
        // Game: ExtraWildTalentSelection 0f73730cf1b44ee882671e55d5f6e471
        (
            "extra-wild-talent",
            "kineticist/extra-wild-talent",
            "0f73730cf1b44ee882671e55d5f6e471",
        ),
    ];
    for (key, selection, guid) in extra {
        let feat = feat(key);
        assert_eq!(
            feat.selection.as_ref().map(|s| s.as_str()),
            Some(selection),
            "{key}"
        );
        assert_eq!(guid_of(&feat.blueprint), Some(guid), "{key}");
        assert_eq!(
            guid_of(&any_selection(selection).blueprint),
            Some(guid),
            "{selection}"
        );
    }
    let granting: BTreeSet<&str> = DATA
        .feats
        .iter()
        .filter(|feat| feat.selection.is_some())
        .map(|feat| feat.key.as_str())
        .collect();
    assert_eq!(granting, extra.iter().map(|(key, ..)| *key).collect());
}

/// The Extra feats whose game list is the list of the one selection they stand for list it as their own (the game's
/// `m_AllFeatures` of each Extra selection equals that selection's, as the planner holds it), with that selection's
/// header, so an option taken through them has the level and the label it had. Each opens its slot whenever it is
/// taken (306aa).
#[test]
fn the_own_list_extra_feats_list_the_selection_they_stand_for() {
    // Game: ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a and ArcanistExploitSelection
    // b8bf3d5023f2d8c428fdf6438cecaea7 list the same 22; ExtraMercySelection 8a49abed5be9473da1e1fd1e2457562e and
    // SelectionMercy 02b187038a8dce545bb34bbfb346428d the same 15; ExtraSlayerTalentSelection
    // 53f7237f5b1447bb851ba68045a00e41 and ExtraWildTalentSelection 0f73730cf1b44ee882671e55d5f6e471 their own (41, 40),
    // which the planner's wild talents stand for (293 writes the game's); the slayer's own list is written (306g) and
    // `the_slayers_extra_talent_list_is_the_games_41` pins it.
    for (extra, base) in [
        (
            "arcanist/extra-arcanist-exploit",
            "arcanist/arcanist-exploits",
        ),
        ("paladin/extra-mercy", "paladin/mercy"),
        ("kineticist/extra-wild-talent", "kineticist/wild-talents"),
    ] {
        let (extra_selection, base_selection) = (any_selection(extra), any_selection(base));
        assert_eq!(
            extra_selection.offered(),
            base_selection.offered(),
            "{extra}"
        );
        assert!(
            extra_selection.options.is_empty(),
            "{extra}: shares, defines none"
        );
        assert_eq!(extra_selection.name, base_selection.name, "{extra}");
        assert_eq!(extra_selection.band, base_selection.band, "{extra}");
        assert_eq!(
            extra_selection.option_levels, base_selection.option_levels,
            "{extra}"
        );
    }
}

/// The Extra feats' own lists are the game's: Extra Rogue Talent's is the rogue's 38 talents and Evasion, Extra
/// Discovery's 28, the Vivisectionist's 25, the witch's 27 (306aa; the others are pinned by their own tests).
#[test]
fn the_extra_feats_list_the_games_own_selections() {
    // Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 (39: RogueTalentSelection
    // c074a5d615200494b8f2a9c845799d93's 38 and Evasion 576933720c440aa4d8d42b0c54b77e80)
    let rogue = any_selection("rogue/extra-rogue-talent");
    let mut talents: BTreeSet<_> = any_selection("rogue/rogue-talents").offered();
    let evasion = rules::SelectionOptionKey::new("rogue/rogue-talents/evasion");
    talents.insert(&evasion);
    assert_eq!(rogue.offered(), talents);
    assert_eq!(rogue.offered().len(), 39);
    assert!(rogue.feats.is_empty());
    // ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66 (28) and ExtraVivsectionistDiscoverySelection
    // 10287e7b8cee479e82ea88bd6d2d4dae (25): an Underground Chemist's discovery is in neither.
    assert_eq!(
        any_selection("alchemist/extra-discovery").offered().len(),
        28
    );
    assert_eq!(
        any_selection("alchemist/vivisectionist/extra-discovery")
            .offered()
            .len(),
        25
    );
    // ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35 (27)
    assert_eq!(any_selection("witch/extra-hex").offered().len(), 27);
}

/// The mythic abilities that grant a pick each grant one selection with the ability's blueprint (the game's ability
/// is the selection); Witch Wandering Hex's is the planner's, the game's being a plain feature.
#[test]
fn the_mythic_abilities_that_grant_a_pick_name_their_selection() {
    let cases: [(&str, &str, Option<&str>, &str); 7] = [
        // Game: BeneficialCurse 2dda67424ee8e0b4d83ef01a73ca6bff (BlueprintFeatureSelection)
        (
            "beneficial-curse",
            "oracle/beneficial-curse",
            Some("2dda67424ee8e0b4d83ef01a73ca6bff"),
            "oracle/oracles-curse",
        ),
        // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf (BlueprintFeatureSelection)
        (
            "impossible-domain",
            "cleric/impossible-domain",
            Some("213a8480d22206b45acbfa0619ca5aaf"),
            "cleric/impossible-domain",
        ),
        // Game: SecondBloodline 3cf2ab2c320b73347a7c21cf0d0995bd (BlueprintFeatureSelection; its 21 are
        // SorcererBloodlineSelection 24bef8d1bee12274686f6da6ccbc8914's)
        (
            "second-bloodline",
            "sorcerer/second-bloodline",
            Some("3cf2ab2c320b73347a7c21cf0d0995bd"),
            "sorcerer/bloodline",
        ),
        // Game: SecondBloodragerBloodline b7f62628915bdb14d8888c25da3fac56 (BlueprintFeatureSelection)
        (
            "second-bloodrager-bloodline",
            "bloodrager/second-bloodline",
            Some("b7f62628915bdb14d8888c25da3fac56"),
            "bloodrager/bloodline",
        ),
        // Game: SecondMystery 277b0164740b97945a3f8022bd572f48 (BlueprintFeatureSelection)
        (
            "second-mystery",
            "oracle/second-mystery",
            Some("277b0164740b97945a3f8022bd572f48"),
            "oracle/second-mystery",
        ),
        // Game: SecondSpirit 2faa80662a56ab644aec2f875a68597f (BlueprintFeatureSelection; its 9 are
        // ShamanSpiritSelection 00c8c566d1825dd4a871250f35285982's)
        (
            "second-spirit",
            "shaman/second-spirit",
            Some("2faa80662a56ab644aec2f875a68597f"),
            "shaman/spirit",
        ),
        // Game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 is a BlueprintFeature, no selection: the planner's
        // pick of a witch hex
        (
            "witch-wandering-hex",
            "witch/wandering-hex",
            None,
            "witch/hex",
        ),
    ];
    for (ability, selection, guid, base) in cases {
        let entry = (DATA.mythic_abilities.iter())
            .find(|entry| entry.key.as_str() == ability)
            .unwrap_or_else(|| panic!("no mythic ability {ability}"));
        assert_eq!(
            entry.selection.as_ref().map(|s| s.as_str()),
            Some(selection),
            "{ability}"
        );
        assert_eq!(
            guid_of(&any_selection(selection).blueprint),
            guid,
            "{selection}"
        );
        if guid.is_some() {
            assert_eq!(guid_of(&entry.blueprint), guid, "{ability}");
        }
        // Its list: the selection it stands for (until 293 writes the game's), with that selection's name and option
        // levels; its band is its blueprint's group's (`pick_order`).
        let (nested, base) = (any_selection(selection), any_selection(base));
        assert_eq!(nested.offered(), base.offered(), "{selection}");
        assert_eq!(nested.name, base.name, "{selection}");
        assert_eq!(nested.option_levels, base.option_levels, "{selection}");
        assert_eq!(
            nested.option_level_archetypes, base.option_level_archetypes,
            "{selection}"
        );
    }
    let granting: BTreeSet<&str> = DATA
        .mythic_abilities
        .iter()
        .filter(|ability| ability.selection.is_some())
        .map(|ability| ability.key.as_str())
        .collect();
    assert_eq!(granting, cases.iter().map(|(key, ..)| *key).collect());
}

/// Extra Mythic Ability grants a selection of mythic abilities with its blueprint, listing every mythic ability (the
/// planner's general pick's; the game's list is 293's); Extra Feat grants the general feat selection.
#[test]
fn extra_mythic_ability_and_extra_feat_grant_their_selections() {
    // Game: ExtraMythicAbilityMythicFeat 8a6a511c55e67d04db328cc49aaad2b8 (BlueprintFeatureSelection, Group
    // MythicAbility)
    let extra = mythic_feat("extra-mythic-ability");
    assert_eq!(
        extra.selection.as_ref().map(|s| s.as_str()),
        Some("mythic/extra-mythic-ability")
    );
    let selection = any_selection("mythic/extra-mythic-ability");
    assert_eq!(
        guid_of(&selection.blueprint),
        Some("8a6a511c55e67d04db328cc49aaad2b8")
    );
    let every: BTreeSet<MythicAbilityKey> = (DATA.mythic_abilities.iter())
        .map(|ability| ability.key.clone())
        .collect();
    assert_eq!(selection.mythic_abilities, every);
    assert_eq!(selection.band, Band::MythicAbility);
    assert_eq!(selection.name, "Mythic ability");
    // Game: ExtraFeatMythicFeat e10c4f18a6c8b4342afe6954bde0587b (Group Feat): the planner's general feat selection
    // stands for it (its list is 293's)
    assert_eq!(
        mythic_feat("extra-feat")
            .selection
            .as_ref()
            .map(|s| s.as_str()),
        Some(BASIC_FEAT_SELECTION)
    );
    let granting: BTreeSet<&str> = (DATA.mythic_feats.iter())
        .filter(|feat| feat.selection.is_some())
        .map(|feat| feat.key.as_str())
        .collect();
    assert_eq!(
        granting,
        BTreeSet::from(["extra-feat", "extra-mythic-ability"])
    );
}

/// The chain at one rank: Extra Mythic Ability's pick is Second Bloodline, whose pick is a bloodline, each slot the
/// selection the one before grants.
#[test]
fn extra_mythic_ability_then_second_bloodline_nest_at_one_rank() {
    // Game: ExtraMythicAbilityMythicFeat 8a6a511c55e67d04db328cc49aaad2b8 lists SecondBloodline
    // 3cf2ab2c320b73347a7c21cf0d0995bd; SelectFeature.Apply, WrathDecompiled 6ce4a6b (a picked selection is added
    // to the level-up's selections, same source and level)
    let engine = engine();
    let mut sorcerer = with_selections_filled(build("sorcerer", 7));
    sorcerer.picks.insert(
        SlotId::Mythic { rank: 2 },
        rules::Pick::mythic_feat(rules::MythicFeatKey::new("extra-mythic-ability")),
    );
    sorcerer.picks.insert(
        SlotId::MythicExtra { rank: 2 },
        rules::Pick::mythic_ability(MythicAbilityKey::new("second-bloodline")),
    );
    let second = SlotId::MythicSelection { rank: 2, index: 0 };
    let open = offered_in(&sorcerer, &second)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .min()
        .expect("an open bloodline");
    sorcerer.picks.insert(second.clone(), pick(&open));
    assert!(possible(&sorcerer));
    let sheet = engine.evaluate(&sorcerer);
    let extra = sheet.slot(&SlotId::MythicExtra { rank: 2 }).unwrap();
    assert_eq!(
        extra.kind,
        SlotKind::MythicAbilities(FeatureSelectionKey::new("mythic/extra-mythic-ability"))
    );
    assert_eq!(engine.slot_label(extra), "Mythic ability");
    let bloodline = sheet.slot(&second).unwrap();
    assert_eq!(bloodline.kind, selection_kind("sorcerer/second-bloodline"));
    assert_eq!(engine.slot_label(bloodline), "Bloodline");
    // Its option is levelled as the first bloodline's: by sorcerer levels.
    assert_eq!(
        at(&sorcerer, 7)
            .option_levels
            .get(&rules::SelectionOptionKey::new(&open)),
        Some(&7)
    );
}

/// The general Combat, Magic, Teamwork and Mounted Combat selections each list the planner's feats of their category
/// (ticket 301's lists, by `FeatCategory`). Ticket 293 replaces them with the game's lists (`FighterFeatSelection`
/// 41c8486641f7d6d4283ca9dae4147a9f, `WizardFeatSelection` 8c3102c2ff3b69444b139a98521a4899, `TeamworkFeat`
/// d87e2f6a9278ac04caeb0f93eff95fcb, `CavalierBonusFeatSelection` dd17090d14958ef48ba601688b611970), and this test
/// with one against the game's.
#[test]
fn the_category_selections_list_the_feats_of_their_category() {
    let of = |keep: &dyn Fn(FeatCategory) -> bool| -> BTreeSet<FeatKey> {
        (DATA.feats.iter())
            .filter(|feat| keep(feat.category))
            .map(|feat| feat.key.clone())
            .collect()
    };
    assert_eq!(
        any_selection("combat-feats").feats,
        of(&FeatCategory::is_combat)
    );
    assert_eq!(
        any_selection("magic-feats").feats,
        of(&|category| category == FeatCategory::Magic)
    );
    assert_eq!(
        any_selection("teamwork-feats").feats,
        of(&|category| category == FeatCategory::Teamwork)
    );
}
