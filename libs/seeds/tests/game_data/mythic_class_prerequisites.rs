//! The mythic abilities', classes' and archetypes' prerequisites as the game's (ticket 308d, written by
//! `game-check --write prerequisites --only <its entries>`): one real-data test per pattern the run changed.

use rules::{
    Alignment, ArchetypeKey, ClassKey, DeityKey, FeatKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, Prerequisite, SelectionOptionKey,
};

use crate::support::*;

/// A class's or prestige class's prerequisite.
fn class_prerequisite(key: &str) -> &'static Prerequisite {
    class(key)
        .prerequisite
        .as_ref()
        .unwrap_or_else(|| panic!("{key}: no prerequisite"))
}

/// An archetype's prerequisite.
fn archetype_prerequisite(key: &str) -> &'static Prerequisite {
    let key = ArchetypeKey::new(key);
    (DATA.archetypes.iter())
        .find(|archetype| archetype.key == key)
        .unwrap_or_else(|| panic!("no archetype {key:?}"))
        .prerequisite
        .as_ref()
        .unwrap_or_else(|| panic!("{key:?}: no prerequisite"))
}

/// A mythic ability's prerequisite.
fn ability_prerequisite(key: &str) -> &'static Prerequisite {
    let key = MythicAbilityKey::new(key);
    (DATA.mythic_abilities.iter())
        .find(|ability| ability.key == key)
        .unwrap_or_else(|| panic!("no mythic ability {key:?}"))
        .prerequisite
        .as_ref()
        .unwrap_or_else(|| panic!("{key:?}: no prerequisite"))
}

/// The conditions an `All` prerequisite holds directly (a single condition is its own).
fn all_of(prerequisite: &Prerequisite) -> &[Prerequisite] {
    match prerequisite {
        Prerequisite::All(children) => children,
        single => std::slice::from_ref(single),
    }
}

fn has_feat(key: &str) -> Prerequisite {
    Prerequisite::Feat(FeatKey::new(key))
}

fn has_feature(key: &str) -> Prerequisite {
    Prerequisite::Feature {
        feature: FeatureKey::new(key),
        count: 1,
    }
}

fn class_level(class: &str, level: u8) -> Prerequisite {
    Prerequisite::ClassLevel {
        class: ClassKey::new(class),
        level,
    }
}

fn not(inner: Prerequisite) -> Prerequisite {
    Prerequisite::Not(Box::new(inner))
}

/// The game's prestige classes name no character level: their skill ranks, base attack bonus and spell levels gate
/// them.
#[test]
fn the_prestige_classes_name_no_character_level() {
    // Game: AssassinClass eb284ea8d40a2d045911f525eb96c43d, DuelistClass 4e0ea99612ae87a499c7fb0588e31828,
    // EldritchKnightClass de52b73972f0ed74c87f8f6a8e20b542, HellknightClass ed246f1680e667b47b7427d51e651059,
    // HellknightSigniferClass ee6425d6392101843af35f756ce7fefd, MysticTheurgeClass 0920ea7e4fd7a404282e3d8b0ac41838
    // and WinterWitchClass eb24ca44debf6714aabe1af1fd905a07 hold no PrerequisiteCharacterLevel. The Stalwart Defender
    // isn't written (its Endurance has a carrier the planner lacks, 307).
    for key in [
        "assassin",
        "duelist",
        "eldritch-knight",
        "hellknight",
        "hellknight-signifer",
        "mystic-theurge",
        "winter-witch",
    ] {
        assert!(
            !class_prerequisite(key)
                .leaves()
                .any(|leaf| matches!(leaf, Prerequisite::CharacterLevel(_))),
            "{key}"
        );
    }
}

/// The Duelist needs Dodge and the Mystic Theurge 3 ranks in each knowledge, as the game's classes say.
#[test]
fn the_prestige_classes_need_the_games_feats_and_ranks() {
    // Game: DuelistClass 4e0ea99612ae87a499c7fb0588e31828: PrerequisiteFeature Dodge 97e216dbb46ae3c4faef90cf6bbe6fd5.
    assert!(all_of(class_prerequisite("duelist")).contains(&has_feat("dodge")));
    // Game: MysticTheurgeClass 0920ea7e4fd7a404282e3d8b0ac41838: PrerequisiteStatValue SkillKnowledgeArcana 3 and
    // SkillLoreReligion 3.
    let ranks: Vec<(&str, u8)> = all_of(class_prerequisite("mystic-theurge"))
        .iter()
        .filter_map(|leaf| match leaf {
            Prerequisite::SkillRanks { skill, ranks } => Some((skill.as_str(), *ranks)),
            _ => None,
        })
        .collect();
    assert_eq!(ranks, [("knowledge-arcana", 3), ("lore-religion", 3)]);
}

/// The Magic Deceiver and the Mystic Theurge refuse each other, and the oracle refuses Apsu.
#[test]
fn the_magic_deceiver_and_the_mystic_theurge_refuse_each_other_and_the_oracle_refuses_apsu() {
    // Game: MagicDeceiver 5c77110cd0414e7eb4c2e485659c9a46: PrerequisiteNoClassLevel LoremasterClass
    // 4a7c05adfbaf05446a6bf664d28fb103 and MysticTheurgeClass 0920ea7e4fd7a404282e3d8b0ac41838; MysticTheurgeClass:
    // PrerequisiteNoArchetype MagicDeceiver.
    assert_eq!(
        archetype_prerequisite("arcanist/magic-deceiver"),
        &Prerequisite::All(vec![
            not(class_level("loremaster", 1)),
            not(class_level("mystic-theurge", 1)),
        ])
    );
    assert!(
        all_of(class_prerequisite("mystic-theurge")).contains(&not(Prerequisite::Archetype(
            ArchetypeKey::new("arcanist/magic-deceiver")
        )))
    );
    // Game: OracleClass 20ce9bf8af32bee4c8557a045ab499b1: PrerequisiteNoFeature ApsuFeature
    // 772e2673945e4583a804ae01f67efea0.
    assert_eq!(
        class_prerequisite("oracle"),
        &not(Prerequisite::Deity(DeityKey::new("apsu")))
    );
}

/// The Dark Elementalist is evil, and the Rageshaper anything but lawful.
#[test]
fn the_dark_elementalist_is_evil_and_the_rageshaper_isnt_lawful() {
    let alignments = |key: &str| -> Vec<Alignment> {
        match archetype_prerequisite(key) {
            Prerequisite::Any(members) => members
                .iter()
                .map(|member| match member {
                    Prerequisite::Alignment(alignment) => *alignment,
                    other => panic!("{other:?}"),
                })
                .collect(),
            other => panic!("{other:?}"),
        }
    };
    // Game: DarkElementalistArchetype f12f18ae8842425489d29f302e69134c: PrerequisiteAlignment LE, NE, CE.
    assert_eq!(
        alignments("kineticist/dark-elementalist"),
        [
            Alignment::LawfulEvil,
            Alignment::NeutralEvil,
            Alignment::ChaoticEvil
        ]
    );
    // Game: RageshaperShifterArchetype 41d5b44e421048d09d32623689173e11: PrerequisiteAlignment NG, CG, N, CN, NE, CE.
    assert_eq!(
        alignments("shifter/rageshaper"),
        [
            Alignment::NeutralGood,
            Alignment::ChaoticGood,
            Alignment::TrueNeutral,
            Alignment::ChaoticNeutral,
            Alignment::NeutralEvil,
            Alignment::ChaoticEvil
        ]
    );
}

/// Any Means Necessary needs Hellknight 7; Kinetic Overcharge and Over-Infused Blasts kineticist 1.
#[test]
fn the_class_mythic_abilities_need_the_games_class_levels() {
    // Game: FreeFromEvil a0a814897cd33ed4ea6ffca4b0e1197a: PrerequisiteClassLevel HellknightClass
    // ed246f1680e667b47b7427d51e651059 7.
    assert_eq!(
        ability_prerequisite("any-means-necessary"),
        &class_level("hellknight", 7)
    );
    // Game: KineticOvercharge 4236ca275c6fa9b4a9945b2645262616 and DoubleKineticistInfusions
    // 7410cc35ae6aa9d4a82f1d23c081a1ac: PrerequisiteClassLevel KineticistClass 42a455d9ec1ad924d889272429eb8391 1.
    for key in ["kinetic-overcharge", "over-infused-blasts"] {
        assert_eq!(
            ability_prerequisite(key),
            &class_level("kineticist", 1),
            "{key}"
        );
    }
}

/// Impossible Domain needs a domain selection, so a Faith Hunter inquisitor, who has none, is refused it.
#[test]
fn impossible_domain_is_closed_to_a_faith_hunter() {
    // Game: ExtraDomain 213a8480d22206b45acbfa0619ca5aaf needs (Any) DomainsSelection 48525e5da45c9c243a343fc6545dbdb9,
    // DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b or FakeDivineSparkSelection
    // e6d337e709b4478cb6e5389d3f85b601; FaithHunterArchetype ac87f40932cfd324c90b441f93857c40 removes DomainsSelection
    // at 1 (the user's answer: as the game, research/308/decisions.md item 1).
    let impossible_domain = ability_prerequisite("impossible-domain");
    let inquisitor = build("inquisitor", 1);
    assert_eq!(at(&inquisitor, 1).check(impossible_domain), Ok(()));
    let faith_hunter = archetype_build("inquisitor", "inquisitor/faith-hunter", 1);
    assert!(at(&faith_hunter, 1).check(impossible_domain).is_err());
}

/// A Sanctified Slayer and a Tandem Executioner may take Defensive Study at 1st level: each one's Studied Target grants
/// the slayer's.
#[test]
fn a_sanctified_slayer_and_a_tandem_executioner_may_take_defensive_study() {
    // Game: DefensiveStudy ec041312142ba1f48a0044521f68fc0a: PrerequisiteFeature SlayerStudyTargetFeature
    // 09bdd9445ac38044389476689ae8d5a1, which passes on HasFact (PrerequisiteFeature.CheckInternal, WrathDecompiled
    // 6ce4a6b). SanctifiedSlayerStudyTargetFeature 6d9d3fbc30564e64d966dba27cd6357a: AddFacts SlayerStudyTargetFeature;
    // TandemExecutionerStudyTargetFeature 3dac0ae65dec4b619c035df07751b59d: AddFeatureOnClassLevel HunterClass 1
    // SlayerStudyTargetFeature.
    let defensive_study = ability_prerequisite("defensive-study");
    for (class, archetype) in [
        ("inquisitor", "inquisitor/sanctified-slayer"),
        ("hunter", "hunter/tandem-executioner"),
    ] {
        let build = archetype_build(class, archetype, 1);
        assert_eq!(at(&build, 1).check(defensive_study), Ok(()), "{archetype}");
    }
}

/// An Unstoppable Dwarf may enter the Stalwart Defender without taking Endurance: the heritage grants it.
#[test]
fn an_unstoppable_dwarf_may_be_a_stalwart_defender_without_taking_endurance() {
    // Game: StalwartDefenderClass d5917881586ff1d4d96d5b7cebda9464: PrerequisiteFeature Endurance
    // 54ee847996c25cd4ba8773d7b8555174; UnstoppableDwarf 109dca82e31a4b1093c501f8914ca1a8: AddFacts Endurance.
    let mut dwarf = build("fighter", 8);
    dwarf.start.race = Some(rules::RaceKey::new("dwarf"));
    dwarf.start.subrace = Some(rules::SubraceKey::new("dwarf/unstoppable-dwarf"));
    let mut scores = rules::AbilityScores::default();
    *scores.get_mut(rules::Ability::Dexterity) = 13;
    dwarf.start.ability_scores = rules::PointBuy::try_from(scores).expect("in budget");
    dwarf
        .picks
        .insert(general_slot(1), feat_pick_plain("toughness"));
    dwarf.picks.insert(class_slot(1), feat_pick_plain("dodge"));
    let state = at(&dwarf, 8);
    assert!(!state.has_feat(&FeatKey::new("endurance")));
    assert_eq!(state.check(class_prerequisite("stalwart-defender")), Ok(()));
}

/// Limitless Rage needs one of five rages, the Elemental Rampager's rampage and the Rageshaper's devastating form
/// among them.
#[test]
fn limitless_rage_counts_the_rampage_and_the_devastating_form() {
    // Game: LimitlessRage 5cb58e6e406525342842a073fb70d068: PrerequisiteFeaturesFromList, Amount 0, of five features,
    // which passes on the first one held (PrerequisiteFeaturesFromList.CheckInternal, WrathDecompiled 6ce4a6b): at
    // least one.
    assert_eq!(
        ability_prerequisite("limitless-rage"),
        &Prerequisite::AtLeast(
            1,
            vec![
                has_feature("barbarian/instinctual-warrior/focused-rage"),
                has_feature("barbarian/rage"),
                has_feature("bloodrager/bloodrage"),
                has_feature("druid/elemental-rampager/rampage"),
                has_feature("shifter/rageshaper/devastating-form"),
            ]
        )
    );
}

/// Second Bloodrager Bloodline and Second Mystery need the bloodline or mystery selection, not a class level or a
/// mystery named one by one; Beneficial Curse the curse selection, the Stigmatized Witch's too.
#[test]
fn the_second_abilities_need_their_selection() {
    let selection = |key: &str| Prerequisite::Selection(FeatureSelectionKey::new(key));
    // Game: SecondBloodragerBloodline b7f62628915bdb14d8888c25da3fac56: PrerequisiteFeature
    // BloodragerBloodlineSelection 62b33ac8ceb18dd47ad4c8f06849bc01, which the bloodrager's and the Hag-Riven's
    // selections share.
    assert!(
        all_of(ability_prerequisite("second-bloodrager-bloodline")).contains(&Prerequisite::Any(
            vec![
                selection("bloodrager/bloodline"),
                selection("bloodrager/hag-riven/bloodline"),
            ]
        ))
    );
    // Game: SecondMystery 277b0164740b97945a3f8022bd572f48: OracleMysterySelection 5531b975dcdf0e24c98f1ff7e017e741
    // (the oracle's and two archetypes') or MysteryGiftFeatureMisterySelection e9629fc4de4c4b3ea1c26e9c13c2402d (the
    // Living Deity's).
    assert!(
        all_of(ability_prerequisite("second-mystery")).contains(&Prerequisite::Any(vec![
            Prerequisite::Any(vec![
                selection("oracle/lone-strider/mystery"),
                selection("oracle/mystery"),
                selection("oracle/wind-whisperer/mystery"),
            ]),
            selection("arcanist/magic-deceiver/mystery"),
        ]))
    );
    // Game: BeneficialCurse 2dda67424ee8e0b4d83ef01a73ca6bff: PrerequisiteFeature OracleCurseSelection
    // b0a5118b4fb793241bc7042464b23fab, which the Stigmatized Witch's curse shares.
    assert_eq!(
        ability_prerequisite("beneficial-curse"),
        &Prerequisite::Any(vec![
            selection(CURSES),
            selection("witch/stigmatized-witch/curse"),
        ])
    );
}

/// Weapon Focus (mythic) needs Weapon Focus, the feat or the rogue talent: one blueprint.
#[test]
fn weapon_focus_mythic_counts_the_rogue_talent() {
    // Game: WeaponFocusMythicFeat 74eb201774bccb9428ba5ac8440bf990: PrerequisiteFeature WeaponFocus
    // 1e1f627d26ad36f43bbd26cc2bf8ac7e, which the feat and the rogue talent share.
    let key = rules::MythicFeatKey::new("weapon-focus-mythic");
    let feat = (DATA.mythic_feats.iter())
        .find(|feat| feat.key == key)
        .unwrap();
    assert_eq!(
        feat.prerequisite,
        Some(Prerequisite::Any(vec![
            has_feat("weapon-focus"),
            Prerequisite::SelectionOption(SelectionOptionKey::new(
                "rogue/rogue-talents/weapon-focus"
            )),
        ]))
    );
}
