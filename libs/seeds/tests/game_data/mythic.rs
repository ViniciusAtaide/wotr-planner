//! Mythic: the paths' rules (ticket 26), Mythic Extra Feat and Extra Mythic Ability's picks (ticket 55), and Witch
//! Wandering Hex and Beneficial Curse's picks (ticket 62).

use std::collections::BTreeSet;

use rules::{
    Ability, Build, FeatKey, FeatureKey, Issue, MythicPathKey, Parameter, Pick, Prerequisite,
    RankStatus, Repeat, SelectionOptionKey, SlotId, SlotKind, SlotOption, Unmet, WeaponProficiency,
};

use crate::support::*;

// ---- Mythic path rules (ticket 26) ------------------------------------------------------------------------------

#[test]
fn the_mr_3_path_gives_its_mr_1_and_2_rows_without_a_first_ascension_twelve_rows_and_thirty_features()
 {
    let rows: Vec<_> = DATA
        .mythic_paths
        .iter()
        .flat_map(|path| &path.ranks)
        .filter(|row| row.rank < 3)
        .collect();
    // Game: MythicStartingProgression af4ee0acb9114e544bf02f39027966b0 (MR 1: the bypass of epic DR and the path's
    // first ascension, FirstAscensionSelection 1421e0034a3afac458de08648d06faf0, six paths); each path's
    // FirstLevelsProgression (AeonFirstLevelsProgression 46df0532714a9454eb5fbad64ce6c14f) gives two features at
    // MR 2, with MythicStartingClassFinished bef12729fa3f9b34488634f61751e295: 6 x (2 + 3) features.
    assert_eq!(rows.len(), 12);
    assert_eq!(rows.iter().map(|row| row.features.len()).sum::<usize>(), 30);

    let sheet = engine().evaluate(&mythic("aeon", 9));
    let force_reality = FeatureKey::new("aeon/force-reality");
    // Without a first ascension of its own, the Build takes its MR 3 path's (ticket 91): Force Reality at MR 1
    // (level 6). MR 2's second grant: `the_aeons_mr_2_row_holds_its_second_ability_not_force_reality_again`.
    // Game: AeonFirstAscension 11744000c915c6144ac6be7ceca03521 at MR 1.
    assert_eq!(sheet.at(5).unwrap().feature_count(&force_reality), 0);
    assert_eq!(sheet.at(6).unwrap().feature_count(&force_reality), 1);
    assert_eq!(sheet.ranks[0].path, Some(MythicPathKey::new("aeon")));
}

/// The Aeon's MR 2 row holds a feature of its own (the ability that dispels negative effects on allies), not Force
/// Reality a second time.
#[test]
#[ignore = "waits on ticket 266: the game gives the Aeon's MR 2 its own second ability, Force Reality once"]
fn the_aeons_mr_2_row_holds_its_second_ability_not_force_reality_again() {
    let sheet = engine().evaluate(&mythic("aeon", 9));
    let force_reality = FeatureKey::new("aeon/force-reality");
    // Game: AeonFirstLevelsProgression 46df0532714a9454eb5fbad64ce6c14f at MR 2 grants AeonSecondAbilityFeature
    // 56201ba48e08def4a9180075485c5bac (AeonSecondLevelAbility d87bd4bfc3e50af4ea1d9be5b4735ea1), not
    // AeonFirstAscension 11744000c915c6144ac6be7ceca03521 again.
    assert_eq!(sheet.at(7).unwrap().feature_count(&force_reality), 1);
    assert_eq!(sheet.at(9).unwrap().feature_count(&force_reality), 1);
}

#[test]
fn an_aeon_gaze_needing_rank_5_opens_at_mr_5() {
    let build = mythic("aeon", 14);
    let area_effects = pick("aeon/enforcing-gaze/enforcing-gaze-area-effects");
    let status = |rank| {
        engine()
            .options(&build, &path_selection(rank))
            .unwrap()
            .into_iter()
            .find(|option| option.pick == area_effects)
            .unwrap()
            .status
    };
    // Game: AeonAreaEffectsGazeFeature 39d8ae51c0034adbb8394210c51901e2 is only in AeonGazeSecondSelection
    // 7d498e034d18ca94baea19d7edee7403, first at Aeon level 3 (MR 5) of AeonProgression
    // 34b9484b0d5ce9340ae51d2bf9518bbe (equivalent model: a tiered selection, not a MythicRank(5) prerequisite).
    assert_eq!(
        status(4),
        Err(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::MythicRank(5),
            has: Some(4)
        }))
    );
    assert_eq!(status(5), Ok(()));
}

#[test]
fn a_legend_has_4_more_in_every_score_from_the_switch() {
    let mut build = mythic("trickster", 18);
    build.path_switch = Some(MythicPathKey::new("legend"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let (before, after) = (sheet.at(16).unwrap(), sheet.at(17).unwrap());
    // Game: LegendFeature 7f99961610390044391f300c7ef5e0c8 (LegendProgression 905383229aaf79e4b8d7e2d316b68715 at 1):
    // +4 in each of the six scores.
    for ability in [Ability::Strength, Ability::Charisma] {
        assert_eq!(
            after.ability_scores.get(ability),
            before.ability_scores.get(ability) + 4
        );
    }
    assert_eq!(after.mythic_rank, 3);
    // The Trickster's MR 1-2 rows stay, its MR 3-7 ones go.
    // Game: the MR 1-2 rows come from MythicStartingProgression af4ee0acb9114e544bf02f39027966b0, which no switch
    // removes; LegendClass 3d420403f3e7340499931324640efe96 needs MythicStartingClass 2; the path's later levels
    // go by the story's Designers/EventConditionActionSystem/Actions/RemoveMythicLevels.RunAction, WrathDecompiled
    // 6ce4a6b (an inference: how many levels the story removes is its cue's).
    assert_eq!(
        after.feature_count(&FeatureKey::new("trickster/bit-of-fun")),
        1
    );
    assert_eq!(
        after.feature_count(&FeatureKey::new("trickster/sneaky-quack")),
        0
    );
}

#[test]
fn the_devil_switch_is_open_from_aeon_or_azata_only() {
    let open = |path: &str| {
        engine()
            .switch_options(&mythic(path, 0))
            .into_iter()
            .find(|option| option.path.as_str() == "devil")
            .unwrap()
            .status
            .is_ok()
    };
    // Game: DevilMythicClass 211f49705f478b3468db6daa802452a2's PrerequisiteCondition: Aeon or Azata levels.
    assert!(open("aeon") && open("azata"));
    assert!(!open("trickster") && !open("lich"));
}

#[test]
fn a_devil_from_aeon_loses_the_gaze_picks_and_keeps_the_rest() {
    let mut build = mythic("aeon", 17);
    build.picks.insert(
        path_selection(3),
        pick("aeon/enforcing-gaze/enforcing-gaze-attack"),
    );
    build.path_switch = Some(MythicPathKey::new("devil"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let at_17 = sheet.at(17).unwrap();
    assert!(at_17.options().next().is_none());
    // Game: DevilProgression 87bc9abf00b240a44bb344fea50ec9bc removes AeonGazeFeature
    // 2689d57a889951241912f4bc7d6bb77e and keeps AeonBaneFeature 0b25e8d8b0488c84c9b5714e9ca0a204 (equivalent
    // model: the game leaves the gaze picks held with no gaze to act on).
    assert_eq!(
        at_17.feature_count(&FeatureKey::new("aeon/enforcing-gaze-extra-gaze")),
        0
    );
    assert_eq!(at_17.feature_count(&FeatureKey::new("aeon/aeon-bane")), 1);
    assert_eq!(sheet.ranks[2].status, RankStatus::PartlyDropped);
    assert_eq!(sheet.ranks[2].dropped_slots, [path_selection(3)]);
}

#[test]
fn a_gold_dragon_takes_three_dragon_feats_ignoring_prerequisites_and_rises_in_every_score() {
    let mut build = mythic("trickster", 19);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    // Improved Critical is taken for a weapon.
    // Game: GoldenDragonProgression a6fbca43902c6194c947546e89af64bd at 2 (MR 9): DragonLevel2FeatSelection
    // a21acdafc0169f5488a9bd3256e2e65b three times, IgnorePrerequisites (Blueprints/Classes/Selection/
    // BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b).
    for (index, pick) in [
        Pick::feat_for(FeatKey::new("improved-critical"), weapon("longsword")),
        Pick::feat(FeatKey::new("great-cleave")),
        Pick::feat(FeatKey::new("dodge")),
    ]
    .into_iter()
    .enumerate()
    {
        build.picks.insert(
            SlotId::PathFeat {
                rank: 9,
                index: index as u8,
            },
            pick,
        );
    }
    // Annihilating Breath needs MR 9: open at the MR 9 pick, not at MR 8's, though both come at 17.
    // Game: GoldDragonBreathAnnihilating e8b9aad5295c49539f1bbf901d314964 needs GoldenDragonClass
    // daf1235b6217787499c14e4e32142523 level 2 (MR 9); its story condition, the corrupted branch, isn't modelled.
    let annihilating = "gold-dragon/improved-gold-dragon-breath/annihilating-breath";
    build.picks.insert(path_selection(9), pick(annihilating));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let status = engine()
        .options(&build, &path_selection(8))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == pick(annihilating))
        .unwrap()
        .status;
    assert!(status.is_err());
    // Perfect Body and Mind at 17 (every score at least 18), Perfect Soul at 19.
    // Game: DragonLevel1StrengthOverride b7524715218c4c14ea4f75881797fc6c (Gold Dragon 1: each base score at least
    // 14) and DragonLevel2StrengthOverride 9ed55389d2079714eab8495957964af2 (Gold Dragon 2: at least 18), and their
    // five siblings for the other scores.
    let at_19 = sheet.at(19).unwrap();
    assert!(
        rules::AbilityScores::ABILITIES
            .iter()
            .all(|ability| at_19.ability_scores.get(*ability) >= 18)
    );
}

/// The Gold Dragon's dragon feats are the basic feats but eight: Advanced Weapon Training II to IV and the
/// Trickster's five (Stat Focus, Completely Normal Spell and the three Improved Critical tiers).
#[test]
#[ignore = "waits on ticket 266: the game gives the dragon feats as the basic feats but eight, no Stat Focus"]
fn a_gold_dragons_dragon_feats_leave_out_stat_focus() {
    let mut build = mythic("trickster", 19);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    let slot = SlotId::PathFeat { rank: 9, index: 0 };
    // Game: DragonLevel2FeatSelection a21acdafc0169f5488a9bd3256e2e65b lists BasicFeatSelection
    // 247a4068296e8be42890143f451b4b45's feats but AdvancedWeaponTraining2-4, CompletelyNormalSpellFeat
    // 094b6278f7b570f42aeaa98379f07cf2, TricksterImprovedImprovedCritical 56f94badbba018b4b8277ce6e2e79e72, its two
    // higher tiers and TricksterStatFocusFeatSelection 0d1d80bd3820a78488412581da3ad9c7.
    let offered = offered_at(&build, &slot);
    for left_out in [
        "stat-focus",
        "metamagic-completely-normal-spell",
        "improved-improved-critical",
    ] {
        assert!(!offered.contains(left_out), "{left_out}");
    }
}

#[test]
fn all_skilled_grants_every_weapon_armour_and_shield_proficiency() {
    // Game: AzataSuperpowersSelection 8a30e92cd04ff5b459ba7cb03584fda0 at Azata 2 (MR 4) of AzataProgression
    // 9db53de4bf21b564ca1a90ff5bd16586 offers AllSkilledFeature f3bc6f9c855b2fb4e9aea364b8163aca.
    let mut build = mythic("azata", 11);
    build
        .picks
        .insert(path_selection(4), pick("azata/superpowers/all-skilled"));
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    let held = sheet.at(11).unwrap();
    // Game: AllSkilledFeature f3bc6f9c855b2fb4e9aea364b8163aca's AddProficiencies: light, medium and heavy armour,
    // buckler, light, heavy and tower shields, and every simple and martial weapon category (equivalent model:
    // proficiencies the game adds, the planner holds as the feats).
    for feat in [
        "simple-weapon-proficiency",
        "martial-weapons-proficiency",
        "exotic-weapon-proficiency",
        "armor-proficiency-light-armor",
        "armor-proficiency-medium-armor",
        "armor-proficiency-heavy-armor",
        "shield-proficiency",
        "tower-shield-proficiency",
    ] {
        assert!(held.has_feat(&FeatKey::new(feat)), "{feat}");
    }
    assert!(!held.has_feat(&FeatKey::new("armor-proficiency-light-barding")));
    // The guide's 16 and the Sawtooth Saber (ticket 32); which of them All-Skilled gives:
    // `all_skilled_grants_every_exotic_weapon_but_the_sawtooth_saber`.
    assert_eq!(exotic_weapons().len(), 17);
}

/// The data's exotic weapons, as parameters.
fn exotic_weapons() -> BTreeSet<Parameter> {
    DATA.weapons
        .iter()
        .filter(|w| w.proficiency == Some(WeaponProficiency::Exotic))
        .map(|w| Parameter::Weapon(w.key.clone()))
        .collect()
}

/// All-Skilled gives every exotic weapon category but the Sawtooth Sabre's.
#[test]
#[ignore = "waits on ticket 265: the game gives All-Skilled 16 exotic weapons, not the sawtooth saber"]
fn all_skilled_grants_every_exotic_weapon_but_the_sawtooth_saber() {
    let mut build = mythic("azata", 11);
    build
        .picks
        .insert(path_selection(4), pick("azata/superpowers/all-skilled"));
    let mut exotic = exotic_weapons();
    // Game: AllSkilledFeature f3bc6f9c855b2fb4e9aea364b8163aca's WeaponProficiencies leave out SawtoothSabre
    // ef50f200e4ab429b8bcb66a56f886c2f's category.
    assert!(exotic.remove(&weapon("sawtooth-saber")));
    assert_eq!(values_at(&build, 11, "exotic-weapon-proficiency"), exotic);
}

// ---- Mythic Extra Feat and Extra Mythic Ability give their pick (ticket 55) ----------------------------------------

/// The slot a rank's general mythic feat gives.
fn mythic_extra(rank: u8) -> SlotId {
    SlotId::MythicExtra { rank }
}

/// A Build of `levels` levels of `class` with `mythic_feat` as MR 2's general pick (MR 2 is at level 7).
fn with_mythic_feat(class: &str, levels: usize, mythic_feat: &str) -> Build {
    let mut build = build(class, levels);
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new(mythic_feat)),
    );
    build
}

/// Every option a slot's picker offers, with its status.
fn slot_options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
}

/// "You can select any normal feat." / "You can select a mythic ability." (mythic-paths#extra-feat,
/// mythic-paths#extra-mythic-ability): each gives one pick, and only these two; each a selection, taken again while
/// it has an option left (ticket 303: the game over ticket 55's reading of the guide's silence).
#[test]
fn extra_feat_and_extra_mythic_ability_are_the_mythic_feats_that_give_a_pick() {
    let giving: Vec<(String, String, Repeat)> = DATA
        .mythic_feats
        .iter()
        .filter_map(|feat| {
            let selection = feat.selection.as_ref()?.to_string();
            Some((feat.key.to_string(), selection, feat.repeat()))
        })
        .collect();
    // Game: ExtraFeatMythicFeat e10c4f18a6c8b4342afe6954bde0587b and ExtraMythicAbilityMythicFeat
    // 8a6a511c55e67d04db328cc49aaad2b8 are selections of Ranks 1, with no PrerequisiteNoFeature of themselves nor a
    // PrerequisiteCondition, so BlueprintFeatureSelection.CanSelect lets them repeat (WrathDecompiled 6ce4a6b);
    // MythicFeatSelection
    // 9ee0f6745f555484299b0a1563b99d81's other selections pick a parameter (equivalent model: those are the
    // planner's parameterised mythic feats).
    assert_eq!(
        giving,
        [
            (
                "extra-feat".to_owned(),
                rules::BASIC_FEAT_SELECTION.to_owned(),
                Repeat::WhileOptionLeft
            ),
            (
                "extra-mythic-ability".to_owned(),
                "mythic/extra-mythic-ability".to_owned(),
                Repeat::WhileOptionLeft
            ),
        ]
    );
}

#[test]
fn extra_feat_at_mr_2_gives_a_feat_slot_at_level_7_offering_what_the_general_slot_there_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", 7, "extra-feat");
    let sheet = engine.evaluate(&build);
    let rank_2: Vec<(SlotId, SlotKind)> = sheet.ranks[1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect();
    assert_eq!(
        rank_2,
        [
            (SlotId::Mythic { rank: 2 }, SlotKind::MythicFeat),
            (mythic_extra(2), feat_kind(rules::BASIC_FEAT_SELECTION)),
        ]
    );
    assert_eq!(sheet.ranks[1].character_level, 7);
    assert_eq!(engine.slot_label(&sheet.ranks[1].slots[1]), "Feat");
    // Every feat but the Trickster's five, each open or locked as in level 7's general slot (those five wait in
    // `extra_feat_offers_the_general_slots_feats_but_the_tricksters_five`).
    let but_tricksters = |slot: &SlotId| -> Vec<_> {
        picks_and_statuses(&build, slot)
            .into_iter()
            .filter(|(pick, _)| !TRICKSTERS_FIVE.contains(&pick.key()))
            .collect()
    };
    let extra = but_tricksters(&mythic_extra(2));
    assert_eq!(extra, but_tricksters(&general_slot(7)));
    assert!(extra.iter().any(|(_, status)| status.is_ok()));

    // A pick there is held from level 7; its prerequisites are checked.
    // Game: Toughness d09b20029e9abfe4480b356c92095623 has no prerequisite; GreatCleaveFeature
    // cc9c862ef2e03af4f89be5088851ea35 needs Cleave and Strength 13.
    let mut with_pick = build.clone();
    with_pick
        .picks
        .insert(mythic_extra(2), feat_pick_plain("toughness"));
    assert!(possible(&with_pick));
    assert!(
        engine
            .evaluate(&with_pick)
            .at(7)
            .unwrap()
            .has_feat(&FeatKey::new("toughness"))
    );
    let mut unmet = build;
    unmet
        .picks
        .insert(mythic_extra(2), feat_pick_plain("great-cleave"));
    assert!(matches!(
        why_not(&unmet, &mythic_extra(2)),
        Some(Issue::Unmet(_))
    ));
}

/// The feats a slot offers, each with its status, in the picker's order.
fn picks_and_statuses(build: &Build, slot: &SlotId) -> Vec<(Pick, Result<(), Issue>)> {
    slot_options(build, slot)
        .into_iter()
        .map(|option| (option.pick, option.status))
        .collect()
}

/// The Trickster's five feats, which the game's Extra Feat leaves out.
const TRICKSTERS_FIVE: [&str; 5] = [
    "stat-focus",
    "metamagic-completely-normal-spell",
    "improved-improved-critical",
    "improved-improved-improved-critical",
    "improved-improved-improved-critical-improved",
];

/// Extra Feat offers the basic feats but the Trickster's five: Stat Focus, Completely Normal Spell and the three
/// Improved Critical tiers; each open or locked as in level 7's general slot.
#[test]
#[ignore = "waits on ticket 265: the game gives Extra Feat the basic feats but the Trickster's five"]
fn extra_feat_offers_the_general_slots_feats_but_the_tricksters_five() {
    let build = with_mythic_feat("fighter", 7, "extra-feat");
    // Game: ExtraFeatMythicFeat e10c4f18a6c8b4342afe6954bde0587b lists BasicFeatSelection
    // 247a4068296e8be42890143f451b4b45's feats but TricksterStatFocusFeatSelection 0d1d80bd3820a78488412581da3ad9c7,
    // CompletelyNormalSpellFeat 094b6278f7b570f42aeaa98379f07cf2, TricksterImprovedImprovedCritical
    // 56f94badbba018b4b8277ce6e2e79e72, TricksterImprovedImprovedImprovedCritical 006a966007802a0478c9e21007207aac
    // and TricksterImprovedImprovedImprovedCriticalImproved 319c882ab3cc51544ad2f3f43633d5b1.
    let general: Vec<_> = picks_and_statuses(&build, &general_slot(7))
        .into_iter()
        .filter(|(pick, _)| !TRICKSTERS_FIVE.contains(&pick.key()))
        .collect();
    assert_eq!(picks_and_statuses(&build, &mythic_extra(2)), general);
}

#[test]
fn extra_mythic_ability_gives_a_mythic_ability_slot_offering_what_a_ranks_does() {
    let engine = engine();
    let build = with_mythic_feat("fighter", 7, "extra-mythic-ability");
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&mythic_extra(2)).expect("the extra slot");
    assert_eq!(
        slot.kind,
        SlotKind::MythicAbilities(rules::FeatureSelectionKey::new(
            "mythic/extra-mythic-ability"
        ))
    );
    assert_eq!(engine.slot_label(slot), "Mythic ability");
    let picks = |slot: &SlotId| -> BTreeSet<Pick> {
        slot_options(&build, slot)
            .into_iter()
            .map(|option| option.pick)
            .collect()
    };
    // What a rank's slot offers, but the two the game leaves out (which wait in
    // `extra_mythic_ability_leaves_out_emergency_help_and_any_means_necessary`).
    let but_left_out = |slot: &SlotId| -> BTreeSet<Pick> {
        picks(slot)
            .into_iter()
            .filter(|pick| !LEFT_OUT_OF_EXTRA_MYTHIC_ABILITY.contains(&pick.key()))
            .collect()
    };
    assert_eq!(
        but_left_out(&mythic_extra(2)),
        but_left_out(&SlotId::Mythic { rank: 1 })
    );

    // A sorcerer's Second Bloodline there gives another bloodline at MR 2.
    // Game: SecondBloodline 3cf2ab2c320b73347a7c21cf0d0995bd selects one of the sorcerer's bloodline progressions.
    let mut sorcerer = with_mythic_feat("sorcerer", 7, "extra-mythic-ability");
    sorcerer.picks.insert(
        mythic_extra(2),
        Pick::mythic_ability(rules::MythicAbilityKey::new("second-bloodline")),
    );
    let second = SlotId::MythicSelection { rank: 2, index: 0 };
    assert_eq!(
        kind_at(&sorcerer, &second),
        Some(selection_kind("sorcerer/second-bloodline"))
    );
    let open = slot_options(&sorcerer, &second)
        .into_iter()
        .find(|option| option.status.is_ok())
        .expect("an open bloodline");
    sorcerer.picks.insert(second, open.pick);
    assert!(possible(&with_selections_filled(sorcerer)));
}

/// The mythic abilities the game's Extra Mythic Ability leaves out.
const LEFT_OUT_OF_EXTRA_MYTHIC_ABILITY: [&str; 2] = ["emergency-help", "any-means-necessary"];

/// Extra Mythic Ability offers what a rank's mythic ability slot does but Emergency Help and Any Means Necessary
/// (and the Divine Herbalist's and Lone Strider's second mysteries, which the planner holds as Second Mystery);
/// a parameterised ability once per value (Favorite Metamagic: each metamagic, ticket 125).
#[test]
#[ignore = "waits on ticket 265: the game gives Extra Mythic Ability no Emergency Help, Any Means Necessary"]
fn extra_mythic_ability_leaves_out_emergency_help_and_any_means_necessary() {
    let build = with_mythic_feat("fighter", 7, "extra-mythic-ability");
    let picks = |slot: &SlotId| -> BTreeSet<Pick> {
        slot_options(&build, slot)
            .into_iter()
            .map(|option| option.pick)
            .collect()
    };
    // Game: ExtraMythicAbilityMythicFeat 8a6a511c55e67d04db328cc49aaad2b8 lists MythicAbilitySelection
    // ba0e5a900b775be4a99702f1ed08914d's options but EmergencyHelpFeature 22ce17c5253183b42845a5bfc54f8afa,
    // FreeFromEvil a0a814897cd33ed4ea6ffca4b0e1197a, SecondMysteryHerbalist cff5c53fe99c48bf863a0005d768f75a and
    // SecondMysteryHermit 324476155d5c4968babfecda37bcc87c.
    let left_out = LEFT_OUT_OF_EXTRA_MYTHIC_ABILITY;
    let rank: BTreeSet<Pick> = picks(&SlotId::Mythic { rank: 1 })
        .into_iter()
        .filter(|pick| !left_out.contains(&pick.key()))
        .collect();
    let offered = picks(&mythic_extra(2));
    assert_eq!(offered, rank);
    let abilities: BTreeSet<&str> = offered.iter().map(Pick::key).collect();
    assert_eq!(
        abilities.len(),
        DATA.mythic_abilities.len() - left_out.len()
    );
}

#[test]
fn removing_extra_feat_or_extra_mythic_ability_is_refused_while_its_slot_holds_a_pick() {
    let engine = engine();
    for (mythic_feat, pick) in [
        ("extra-feat", feat_pick_plain("toughness")),
        (
            "extra-mythic-ability",
            Pick::mythic_ability(rules::MythicAbilityKey::new("unstoppable")),
        ),
    ] {
        let mut build = with_mythic_feat("fighter", 7, mythic_feat);
        build.picks.insert(mythic_extra(2), pick);
        assert!(possible(&build), "{mythic_feat}");
        let feat = rules::Choice::Slot(SlotId::Mythic { rank: 2 });
        assert!(!engine.is_removable(&build, &feat), "{mythic_feat}");
        assert!(!engine.removable(&build).contains(&feat));
        assert!(engine.is_removable(&build, &rules::Choice::Slot(mythic_extra(2))));

        build.apply(&rules::Edit::Clear(mythic_extra(2)));
        assert!(engine.is_removable(&build, &feat), "{mythic_feat}");
        build.apply(&rules::Edit::Clear(SlotId::Mythic { rank: 2 }));
        assert!(possible(&build));
        assert_eq!(kind_at(&build, &mythic_extra(2)), None);
    }
}

#[test]
fn extra_rage_power_through_extra_feat_gives_a_rage_power_slot_at_the_ranks_level() {
    let mut build = with_selections_filled(barbarian(7));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat(rules::MythicFeatKey::new("extra-feat")),
    );
    build
        .picks
        .insert(mythic_extra(2), feat_pick_plain("extra-rage-power"));
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a, a selection, is in ExtraFeatMythicFeat
    // e10c4f18a6c8b4342afe6954bde0587b's list.
    assert_eq!(
        kind_at(&build, &extra_slot(7, 0)),
        Some(selection_kind("barbarian/extra-rage-power"))
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(7, 0)));
    assert!(possible(&filled));
}

// ---- Witch Wandering Hex and Beneficial Curse give their pick (ticket 62) -----------------------------------------

fn mythic_ability_data(key: &str) -> &'static rules::MythicAbility {
    DATA.mythic_abilities
        .iter()
        .find(|ability| ability.key.as_str() == key)
        .unwrap_or_else(|| panic!("no mythic ability {key}"))
}

/// What MR 1's mythic ability picker says of `ability`.
fn mr_1_status(build: &Build, ability: &str) -> Result<(), Issue> {
    slot_options(build, &SlotId::Mythic { rank: 1 })
        .into_iter()
        .find(|option| option.pick.key() == ability)
        .unwrap_or_else(|| panic!("MR 1 doesn't offer {ability}"))
        .status
}

/// "Beneficial Curse. Prerequisites: Oracle's Curse. You gain an additional Curse with no penalties."
/// (mythic-paths): one slot of the oracle's curses, needing a curse held.
#[test]
fn beneficial_curse_gives_a_slot_of_the_oracles_curses() {
    let curse = mythic_ability_data(BENEFICIAL_CURSE);
    // Game: BeneficialCurse 2dda67424ee8e0b4d83ef01a73ca6bff selects one of nine `<Curse>CurseNoPenaltyProgression`s
    // (LameCurseNoPenaltyProgression f9dadbcdecb41c6419f9297a2425eabd) and needs OracleCurseSelection
    // b0a5118b4fb793241bc7042464b23fab (equivalent model: the same nine curses; that the game's carry no penalty is
    // the difference ticket 243 recorded).
    // Its selection (ticket 302: the game's ability is one) stands for the oracle's curses.
    let own = curse
        .selection
        .as_ref()
        .expect("Beneficial Curse's selection");
    assert_eq!(own.as_str(), BENEFICIAL_CURSE_SELECTION);
    assert_eq!(
        any_selection(own.as_str()).offered(),
        any_selection(CURSES).offered()
    );
    // Taken once (ticket 303), the planner's: the game's selection repeats while a curse whose progression it lists is
    // left, which the planner's list of the curses themselves can't say (293).
    let once = Prerequisite::Not(Box::new(Prerequisite::MythicAbility(
        rules::MythicAbilityKey::new(BENEFICIAL_CURSE),
    )));
    let any = match curse.prerequisite.as_ref() {
        Some(Prerequisite::All(children)) if children.len() == 2 && children[1] == once => {
            &children[0]
        }
        other => panic!("{other:?}"),
    };
    let held: BTreeSet<String> = match any {
        Prerequisite::Any(children) => children
            .iter()
            .map(|child| match child {
                Prerequisite::SelectionOption(option) => option.to_string(),
                other => panic!("{other:?}"),
            })
            .collect(),
        other => panic!("{other:?}"),
    };
    let curses: BTreeSet<String> = DATA
        .selection(&rules::FeatureSelectionKey::new(CURSES))
        .unwrap()
        .options
        .iter()
        .map(|option| option.to_string())
        .collect();
    assert_eq!(held, curses);
    assert_eq!(curses.len(), 9);
}

/// Witch Wandering Hex gives no slot: it is a feature granting a daily ability that applies one of twelve basic witch
/// hexes the caster lacks, so the mythic abilities that give a pick are Beneficial Curse and ticket 55's five.
#[test]
#[ignore = "waits on ticket 259: the game gives Witch Wandering Hex a daily hex ability, no hex slot"]
fn witch_wandering_hex_gives_a_daily_hex_ability_and_no_slot() {
    // Game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 is a feature, not a selection: it adds
    // WitchWanderingHexAbility b209beab784d93546b40a2fa2a09ffa8 and a daily resource.
    assert!(mythic_ability_data(WITCH_WANDERING_HEX).selection.is_none());
    let giving: BTreeSet<&str> = DATA
        .mythic_abilities
        .iter()
        .filter(|ability| ability.selection.is_some())
        .map(|ability| ability.key.as_str())
        .collect();
    // Game: MythicAbilitySelection ba0e5a900b775be4a99702f1ed08914d's selections: BeneficialCurse
    // 2dda67424ee8e0b4d83ef01a73ca6bff, ExtraDomain 213a8480d22206b45acbfa0619ca5aaf, SecondBloodline
    // 3cf2ab2c320b73347a7c21cf0d0995bd, SecondBloodragerBloodline b7f62628915bdb14d8888c25da3fac56, SecondMystery
    // 277b0164740b97945a3f8022bd572f48 and SecondSpirit 2faa80662a56ab644aec2f875a68597f.
    assert_eq!(
        giving,
        BTreeSet::from([
            BENEFICIAL_CURSE,
            "impossible-domain",
            "second-bloodline",
            "second-bloodrager-bloodline",
            "second-mystery",
            "second-spirit",
        ])
    );
}

#[test]
fn witch_wandering_hex_is_locked_without_a_hex() {
    // Game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 needs (Any) witch 1, a Sylvan Trickster rogue 2 or
    // a Hexcrafter magus 4: a fighter has none, a witch 6 has witch 1.
    let fighter = build("fighter", 6);
    assert!(matches!(
        mr_1_status(&fighter, WITCH_WANDERING_HEX),
        Err(Issue::Unmet(_))
    ));
    assert!(matches!(
        why_not(
            &with_mr_1_ability(fighter, WITCH_WANDERING_HEX),
            &SlotId::Mythic { rank: 1 }
        ),
        Some(Issue::Unmet(_))
    ));
    // A witch's hex opens it; a shaman's: `a_shamans_hex_doesnt_open_witch_wandering_hex`.
    assert_eq!(
        mr_1_status(
            &with_selections_filled(build("witch", 6)),
            WITCH_WANDERING_HEX
        ),
        Ok(())
    );
}

/// A shaman, with hexes of her own, can't take Witch Wandering Hex: it needs witch 1, a Sylvan Trickster rogue 2 or a
/// Hexcrafter magus 4.
#[test]
fn a_shamans_hex_doesnt_open_witch_wandering_hex() {
    let shaman = with_selections_filled(build("shaman", 6));
    assert!(
        at(&shaman, 6)
            .options()
            .any(|(option, _)| option.as_str().starts_with("shaman/hex/"))
    );
    // Game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351's Any prerequisites name WitchClass, RogueClass with
    // SylvanTricksterArchetype and MagusClass with HexcrafterArchetype, no shaman.
    assert!(matches!(
        mr_1_status(&shaman, WITCH_WANDERING_HEX),
        Err(Issue::Unmet(_))
    ));
}

/// A witch with Witch Wandering Hex holds a daily ability, not a hex slot.
#[test]
#[ignore = "waits on ticket 259: the game gives Witch Wandering Hex a daily hex ability, no hex slot"]
fn witch_wandering_hex_gives_a_witch_no_hex_slot() {
    let witch = with_mr_1_ability(
        with_selections_filled(build("witch", 6)),
        WITCH_WANDERING_HEX,
    );
    assert!(possible(&witch));
    // Game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 adds WitchWanderingHexAbility
    // b209beab784d93546b40a2fa2a09ffa8, which applies one of twelve basic witch hexes the caster lacks for a time.
    assert_eq!(kind_at(&witch, &mr_1_selection()), None);
}

/// The first option, in key order, open in MR 1's selection slot.
fn first_open_in_mr_1_selection(build: &Build) -> String {
    offered_in(build, &mr_1_selection())
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .min()
        .expect("an open option")
}

#[test]
fn beneficial_curse_gives_an_oracle_curse_slot_offering_only_curses_not_held() {
    let engine = engine();
    let oracle = with_selections_filled(build("oracle", 6));
    // Game: BeneficialCurse 2dda67424ee8e0b4d83ef01a73ca6bff needs OracleCurseSelection
    // b0a5118b4fb793241bc7042464b23fab; each of its options (LameCurseNoPenaltyProgression
    // f9dadbcdecb41c6419f9297a2425eabd) has a PrerequisiteNoFeature of its own curse, so the held one is refused.
    assert_eq!(mr_1_status(&oracle, BENEFICIAL_CURSE), Ok(()));
    assert!(matches!(
        mr_1_status(&build("fighter", 6), BENEFICIAL_CURSE),
        Err(Issue::Unmet(_))
    ));
    let oracle = with_mr_1_ability(oracle, BENEFICIAL_CURSE);
    assert!(possible(&oracle));
    let sheet = engine.evaluate(&oracle);
    let slot = sheet.slot(&mr_1_selection()).expect("the curse slot");
    assert_eq!(slot.kind, selection_kind(BENEFICIAL_CURSE_SELECTION));
    assert_eq!(engine.slot_label(slot), "Oracle's Curse");
    let held = oracle
        .picks
        .values()
        .map(|pick| pick.key().to_owned())
        .find(|key| key.starts_with("oracle/oracles-curse/"))
        .expect("a curse held");
    let curses = offered_in(&oracle, &mr_1_selection());
    assert_eq!(curses.len(), 9);
    for (curse, status) in &curses {
        if *curse == held {
            assert_eq!(*status, Err(Issue::AlreadyHeld));
        } else {
            assert_eq!(*status, Ok(()), "{curse}");
        }
    }
    let other = curses.keys().find(|curse| **curse != held).unwrap().clone();
    let mut second = oracle.clone();
    second.picks.insert(mr_1_selection(), pick(&other));
    assert!(possible(&second));
    assert_eq!(
        at(&second, 6).option_count(&SelectionOptionKey::new(&other)),
        1
    );
    let mut twice = oracle;
    twice.picks.insert(mr_1_selection(), pick(&held));
    assert_eq!(why_not(&twice, &mr_1_selection()), Some(Issue::AlreadyHeld));
}

#[test]
fn removing_witch_wandering_hex_or_beneficial_curse_is_refused_while_its_slot_holds_a_pick() {
    let engine = engine();
    for (class, ability) in [("witch", WITCH_WANDERING_HEX), ("oracle", BENEFICIAL_CURSE)] {
        let mut build = with_mr_1_ability(with_selections_filled(build(class, 6)), ability);
        let option = first_open_in_mr_1_selection(&build);
        build.picks.insert(mr_1_selection(), pick(&option));
        assert!(possible(&build), "{ability}");
        let choice = rules::Choice::Slot(SlotId::Mythic { rank: 1 });
        assert!(!engine.is_removable(&build, &choice), "{ability}");
        assert!(!engine.removable(&build).contains(&choice));
        assert!(engine.is_removable(&build, &rules::Choice::Slot(mr_1_selection())));

        build.apply(&rules::Edit::Clear(mr_1_selection()));
        assert!(engine.is_removable(&build, &choice), "{ability}");
        build.apply(&rules::Edit::Clear(SlotId::Mythic { rank: 1 }));
        assert!(possible(&build));
        assert_eq!(kind_at(&build, &mr_1_selection()), None);
    }
}
