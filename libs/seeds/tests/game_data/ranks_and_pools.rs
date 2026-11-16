//! Ticket 243 (ranked features and prerequisite-filtered pools, as the game models them), on the real data.
//! Each expected value comes from the game at 2.7.0x: a blueprint by name and GUID (BubblePrintsData `2.7.0x_0`), or
//! the decompiled code by file and method (WrathDecompiled 6ce4a6b).
//!
//! - **Ranked features:** one feature granted again gains a rank (`FeatureCollection.PrepareFactForAttach`,
//!   `Feature.AddRank`): the Assassin's poison saves, the Feyspeaker's Fey Magic; Physical Enhancement is granted
//!   once, its bonus a number of wizard levels, its 20th-level step a feature of its own.
//! - **The oracle's curse is a progression:** each curse's stages come at curse levels 1, 5, 10 and 15, the curse
//!   level counting oracle levels, a Stigmatized Witch's levels and half the other classes', the Mythic rank among
//!   them (`BlueprintProgression.CalcLevel`); the Dual-Cursed Oracle's second curse and the Living Deity's never advance.
//! - **One revelation selection, filtered by the mystery:** its options carry the mystery that admits them
//!   (`BlueprintFeatureSelection.CanSelect`, `BlueprintFeature.MeetsPrerequisites`).
//! - **Links made before keep opening** with the same picks, an old key read as the option it became.

use std::collections::BTreeSet;

use rules::{
    Address, Build, ClassKey, DataAt, FeatureKey, FeatureSelectionKey, Issue, SelectionOptionKey,
    Sheet, SlotId, Total,
};

use crate::support::*;

/// The links of the fixture: Builds of the corpus (`tools/corpus`) recorded at the commit before the splits, and two
/// made from them with old keys as that commit wrote them: one with Combat Healer (Life), and the Dual-Cursed Oracle's
/// with Fortune moved from oracle 1 to 5, where the game allows it (`a_dual_cursed_oracle_takes_fortune_from_oracle_5_and_misfortune_from_1`).
/// Each line `name<TAB>link<TAB>picks`, the picks as they read now.
const LINKS: &str = include_str!("links-before-the-splits.tsv");

/// A feature's rank, the times it is held, at each level of a Build's sheet.
fn ranks(sheet: &Sheet, feature: &str) -> Vec<u8> {
    let feature = FeatureKey::new(feature);
    sheet
        .levels
        .iter()
        .map(|level| level.state.feature_count(&feature))
        .collect()
}

/// The levels `ranks` would give for one grant at each of `levels`, over `to` levels.
fn ranked_at(levels: &[usize], to: usize) -> Vec<u8> {
    (1..=to)
        .map(|level| u8::try_from(levels.iter().filter(|at| **at <= level).count()).unwrap())
        .collect()
}

fn blueprint(at: DataAt) -> Option<&'static str> {
    BLUEPRINTS.get(&at).copied().flatten()
}

fn feature_blueprint(key: &str) -> Option<&'static str> {
    blueprint(DataAt::Feature(FeatureKey::new(key)))
}

/// Each `(stage, blueprint, level)` of a curse: the feature names the blueprint and comes once, at the curse level.
fn assert_stages(sheet: &Sheet, curse: &str, stages: &[(&str, &str, usize)]) {
    for (feature, guid, level) in stages {
        let key = stage(curse, feature);
        assert_eq!(feature_blueprint(&key), Some(*guid), "{key}");
        assert_eq!(
            ranks(sheet, &key),
            ranked_at(&[*level], sheet.levels.len()),
            "{key}"
        );
    }
}

// ---- ranked features ------------------------------------------------------------------------------------------

#[test]
fn the_assassins_poison_saves_are_one_feature_ranked_at_assassin_5_and_9() {
    // Game: AssassinCreatePoisonIncreasedSavingThrows 0cbb2201a65cb374fb4d73d6c9830b01 (Ranks 5), named by
    // AssassinProgression a02e1f0e13baa8c43b758425eda9e973 at levels 5 and 9; the poison's buffs read its rank
    // (ContextRankConfig FeatureRank on AssassinCreatePoisonAbilityStrBuffEffect
    // 285290cc80941bc4c97461d6f50aaad9).
    let feature = "assassin/poison-use-saves-required";
    assert_eq!(
        feature_blueprint(feature),
        Some("0cbb2201a65cb374fb4d73d6c9830b01")
    );
    let mut assassin = build("rogue", 15);
    for level in &mut assassin.levels[..5] {
        level.skill_ranks.insert(rules::SkillKey::new("stealth"), 1);
    }
    for level in &mut assassin.levels[6..] {
        level.class = ClassKey::new("assassin");
    }
    let assassin = realigned(assassin, rules::Alignment::NeutralEvil);
    assert!(possible(&assassin));
    // Assassin levels 5 and 9 are Build levels 11 and 15.
    assert_eq!(
        ranks(&engine().evaluate(&assassin), feature),
        ranked_at(&[11, 15], 15)
    );
}

#[test]
fn fey_magic_is_one_feature_ranked_at_each_even_feyspeaker_level_from_4_to_18() {
    // Game: FeyspeakerArchetype da69747aa3dd0044ebff5f3d701cdde3 adds FeyspeakerMagicSelection
    // 237efc6a131c8174bbd466ed239f21fc at 4, 6, 8, 10, 12, 14, 16 and 18.
    let feature = "druid/feyspeaker/fey-magic";
    assert_eq!(
        feature_blueprint(feature),
        Some("237efc6a131c8174bbd466ed239f21fc")
    );
    let druid = druid(Some("druid/feyspeaker"), 18);
    assert!(possible(&druid));
    assert_eq!(
        ranks(&engine().evaluate(&druid), feature),
        ranked_at(&[4, 6, 8, 10, 12, 14, 16, 18], 18)
    );
}

#[test]
fn physical_enhancement_is_granted_once_and_its_two_abilities_come_at_wizard_20() {
    // Game: SpecializationSchoolTransmutation c459c8200e666ef4c990873d3e501b91 grants
    // TransmutationPhysicalEnhancementFeature 93919f8ce64dc5a4cbf058a486a44a1b (Ranks 1) once; its bonus scales by
    // wizard level in its buffs (Designers/Mechanics/Buffs/BuffScaling.ScaleBuff, WrathDecompiled 6ce4a6b).
    // SpecialisationSchoolTransmutationProgression b6a604dab356ac34788abf4ad79449ec names
    // TransmutationPhysicalEnhancementCapstone 6aa7d3496cd68e643adcd439a7306caa at 20.
    let capstone = "wizard/school/transmuter/physical-enhancement-capstone";
    assert_eq!(
        feature_blueprint(capstone),
        Some("6aa7d3496cd68e643adcd439a7306caa")
    );
    let wizard = with_option(build("wizard", 20), 1, 0, "wizard/school/transmuter");
    assert!(possible(&wizard));
    let wizard = engine().evaluate(&wizard);
    assert_eq!(
        ranks(&wizard, "wizard/school/transmuter/physical-enhancement"),
        ranked_at(&[1], 20)
    );
    assert_eq!(ranks(&wizard, capstone), ranked_at(&[20], 20));
    for step in 2..=5 {
        let key = format!("wizard/school/transmuter/physical-enhancement-{step}");
        assert_eq!(feature_blueprint(&key), None, "{key}");
        assert!(
            !DATA
                .class_features()
                .any(|feature| feature.key.as_str() == key)
        );
    }
}

// ---- the oracle's curse: a progression ------------------------------------------------------------------------

/// An oracle of `classes` (the first an oracle), with the Battle mystery and `curse`.
fn cursed(classes: &[&str], curse: &str) -> Build {
    let build = classes_build(classes);
    let build = with_option(build, 1, 0, "oracle/mystery/battle");
    with_option(build, 1, 1, &format!("oracle/oracles-curse/{curse}"))
}

fn stage(curse: &str, stage: &str) -> String {
    format!("oracle/oracles-curse/{curse}/{stage}")
}

/// The Build with its Mythic ranks taken at its last level, after that level's class level, so a curse's level
/// counts its classes alone until then (the Mythic rank counts as another class's level:
/// `a_curses_level_adds_half_the_mythic_rank_as_another_class`).
fn unascended(mut build: Build) -> Build {
    let last = u8::try_from(build.levels.len()).unwrap();
    build.rank_levels.insert(1, last);
    build
}

#[test]
fn an_oracles_curse_reaches_its_stages_at_curse_levels_1_5_10_and_15() {
    // Game: BlackenedCurseProgression ca1e4627a1ad8fc4f83158f6497d5c54, LevelEntries 1 (BlackenedCurseAttackRollFeature
    // 5edc2d4ed4b7c4b499e635fda70adadb, BlackenedCurseFeatureLevel1 0b6eeed955daedd4896ef104cfc2afb9), 5
    // (BlackenedCurseFeatureLevel5 f85139824ffa90a448dc579097ab988c), 10 (BlackenedCurseFeatureLevel10
    // 510d3d8e625df844496173fef12ff326) and 15 (BlackenedCurseFeatureLevel15 4e05304fd4c1aae45a24d73976225697).
    let oracle = unascended(cursed(&["oracle"; 15], "blackened"));
    assert!(possible(&oracle));
    assert_stages(
        &engine().evaluate(&oracle),
        "blackened",
        &[
            ("weapon-penalty", "5edc2d4ed4b7c4b499e635fda70adadb", 1),
            ("level-1", "0b6eeed955daedd4896ef104cfc2afb9", 1),
            ("level-5", "f85139824ffa90a448dc579097ab988c", 5),
            ("level-10", "510d3d8e625df844496173fef12ff326", 10),
            ("level-15", "4e05304fd4c1aae45a24d73976225697", 15),
        ],
    );
    // The 27 placeholders are gone.
    for owner in [
        "oracle",
        "oracle/dual-cursed-oracle",
        "oracle/possessed-oracle",
    ] {
        for step in 2..=4 {
            let key = format!("{owner}/curse-{step}");
            assert!(
                !DATA
                    .class_features()
                    .any(|feature| feature.key.as_str() == key),
                "{key}"
            );
        }
    }
}

/// The initiative total at each level, and the curse's part of it.
fn initiative(build: &Build, part: &str) -> Vec<Option<i16>> {
    let engine = engine();
    engine
        .evaluate(build)
        .levels
        .iter()
        .map(|level| {
            let total: Total = engine.totals(level).initiative;
            total.parts.iter().find(|p| p.name == part).map(|p| p.value)
        })
        .collect()
}

#[test]
fn a_curses_level_adds_half_the_other_classes_but_not_a_plain_witchs() {
    // Game: Kingmaker/Blueprints/Classes/BlueprintProgression.cs `CalcLevel`, WrathDecompiled 6ce4a6b: the classes
    // listed (WitchClass and OracleClass on every curse progression, PowerlessProphecyCurseProgression
    // 88c89ab44a36a67438347e257f70c8fe here) count unless the class has an archetype the progression lists
    // (AccursedWitchArchetype, a witch's), and with ForAllOtherClasses and AlternateProgressionType Div2 half the
    // other classes' levels, rounded down. PowerlessProphecyCurseFeatureLevel5 abfd1deaddbf58140b8ba0e0bbb51e46: +4
    // initiative.
    let part = "Powerless Prophecy, Curse Level 5";
    let fighter = cursed(
        &["oracle", "oracle", "oracle", "oracle", "fighter", "fighter"],
        "powerless-prophecy",
    );
    assert!(possible(&fighter));
    // Oracle 4 and fighter 1: curse level 4. Oracle 4 and fighter 2: 4 + 1.
    assert_eq!(
        initiative(&fighter, part),
        [None, None, None, None, None, Some(4)]
    );
    assert_eq!(
        at(&fighter, 6).option_level(&SelectionOptionKey::new(
            "oracle/oracles-curse/powerless-prophecy"
        )),
        5
    );
    let witch = cursed(
        &["oracle", "oracle", "oracle", "oracle", "witch", "witch"],
        "powerless-prophecy",
    );
    assert!(possible(&witch));
    assert_eq!(initiative(&witch, part)[5], None);
    assert_eq!(
        at(&witch, 6).option_level(&SelectionOptionKey::new(
            "oracle/oracles-curse/powerless-prophecy"
        )),
        4
    );
    // Pranked's -4 is its level-1 stage's (PrankedCurseFeatureLevel1 cfba3d40cfa879f4b9a0b12060790a72).
    let pranked = cursed(&["oracle"], "pranked");
    assert_eq!(initiative(&pranked, "Pranked, Curse Level 1"), [Some(-4)]);
}

#[test]
fn a_curses_level_adds_half_the_mythic_rank_as_another_class() {
    // Game: Blueprints/Classes/BlueprintProgression.cs `CalcLevel` (WrathDecompiled 6ce4a6b) adds, for
    // ForAllOtherClasses, every entry of `unit.Progression.Classes` the progression doesn't list, and that list holds
    // the mythic classes too (UnitLogic/UnitProgressionData.cs `SetupLevelsIfNecessary` sums its mythic entries into
    // the Mythic level).
    // Every level-up, a mythic one included, recomputes each progression's level
    // (UnitLogic/Class/LevelUp/Actions/ApplyClassMechanics.cs `ApplyProgressions`). BlackenedCurseProgression
    // ca1e4627a1ad8fc4f83158f6497d5c54 names BlackenedCurseFeatureLevel10 510d3d8e625df844496173fef12ff326 at 10.
    let mut oracle = cursed(&["oracle"; 9], "blackened");
    oracle.mythic_path = Some(rules::MythicPathKey::new("angel"));
    assert!(possible(&oracle));
    let sheet = engine().evaluate(&oracle);
    // Mythic ranks 1, 2 and 3 come at levels 6, 7 and 9: oracle 7 at MR 2 is curse level 8, oracle 9 at MR 3 is 10.
    let curse = SelectionOptionKey::new("oracle/oracles-curse/blackened");
    let levels: Vec<u8> = (sheet.levels.iter())
        .map(|level| level.state.option_level(&curse))
        .collect();
    assert_eq!(levels, [1, 2, 3, 4, 5, 6, 8, 9, 10]);
    assert_eq!(
        ranks(&sheet, &stage("blackened", "level-10")),
        ranked_at(&[9], 9)
    );
}

#[test]
fn a_stigmatized_witchs_curse_advances_with_her_witch_levels() {
    // Game: AccursedWitchArchetype c5f6e53e71059fb4d802ce81a277a12d adds OracleCurseSelection
    // b0a5118b4fb793241bc7042464b23fab at 1; LameCurseProgression 3d319da9614c76a41a095a4467c3233f lists
    // AccursedWitchArchetype, whose class levels count in full (LameCurseFeatureLevel5
    // bf84de8be98f949478d249e5f308e741 at 5).
    let slot = SlotId::Selection { level: 1, index: 1 };
    let witch = archetype_build("witch", "witch/stigmatized-witch", 5);
    let curse = engine().evaluate(&witch).slot(&slot).cloned().unwrap();
    assert_eq!(curse.kind, selection_kind("witch/stigmatized-witch/curse"));
    assert_eq!(
        blueprint(DataAt::FeatureSelection(FeatureSelectionKey::new(
            "witch/stigmatized-witch/curse"
        ))),
        Some("b0a5118b4fb793241bc7042464b23fab")
    );
    let witch = with_pick(witch, slot, "oracle/oracles-curse/lame");
    assert!(possible(&witch));
    let witch = engine().evaluate(&witch);
    assert_eq!(ranks(&witch, &stage("lame", "level-1")), ranked_at(&[1], 5));
    assert_eq!(ranks(&witch, &stage("lame", "level-5")), ranked_at(&[5], 5));
}

#[test]
fn the_dual_cursed_oracles_second_curse_and_the_living_deitys_never_advance() {
    // Game: SecondCurseSelection cc6fda79e8c340b88c84689414a9abbe and MysteryGiftFeatureCurseSelection
    // 4e7265c0ae1345db90d3375f4ced94cc offer the `<Curse>CurseNoProgression`s, each with a level-1 entry only
    // (LameCurseNoProgression 857c97115ec745d58cc8fa7bdedc794f, PlaguedCurseNoProgression
    // b1f3b639d54540e59eb0643d77f3736c); the Dual-Cursed Oracle's first curse is the class's OracleCurseSelection.
    let dual = unascended(archetype_build("oracle", "oracle/dual-cursed-oracle", 15));
    let dual = with_option(dual, 1, 1, "oracle/mystery/battle");
    let dual = with_option(dual, 1, 2, "oracle/oracles-curse/plagued");
    let dual = with_option(dual, 1, 3, "oracle/oracles-curse/lame");
    assert!(possible(&dual));
    let dual = engine().evaluate(&dual);
    assert_eq!(
        ranks(&dual, &stage("plagued", "level-15")),
        ranked_at(&[15], 15)
    );
    assert_eq!(ranks(&dual, &stage("lame", "level-1")), ranked_at(&[1], 15));
    for level in ["level-5", "level-10", "level-15"] {
        assert_eq!(ranks(&dual, &stage("lame", level)), [0; 15], "{level}");
    }
    assert_eq!(
        blueprint(DataAt::FeatureSelection(FeatureSelectionKey::new(
            "oracle/second-curse"
        ))),
        Some("cc6fda79e8c340b88c84689414a9abbe")
    );

    let deity = archetype_build("arcanist", "arcanist/magic-deceiver", 15);
    let deity = with_option(
        deity,
        1,
        0,
        "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity",
    );
    let deity = with_pick(deity, option_slot(1, 0), "oracle/mystery/battle");
    let deity = with_pick(deity, option_slot(1, 2), "oracle/oracles-curse/plagued");
    assert!(possible(&deity));
    let deity = engine().evaluate(&deity);
    assert_eq!(
        ranks(&deity, &stage("plagued", "level-1")),
        ranked_at(&[1], 15)
    );
    assert_eq!(ranks(&deity, &stage("plagued", "level-5")), [0; 15]);
    assert_eq!(
        blueprint(DataAt::FeatureSelection(FeatureSelectionKey::new(
            "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity/curse"
        ))),
        Some("4e7265c0ae1345db90d3375f4ced94cc")
    );
}

#[test]
fn a_possessed_oracle_holds_powerless_prophecy_as_its_advancing_curse_with_no_pick() {
    // Game: PossessedOracleArchetype 09035bb5dbb714645bb08ea7db47884c adds PowerlessProphecyCurseProgression
    // 88c89ab44a36a67438347e257f70c8fe at 1 (and removes OracleCurseSelection), whose LevelEntries are
    // UncannyDodgeChecker 8f800ed6ce8c42e8a01fd8f3e990c459 and PowerlessProphecyCurseFeatureLevel1
    // e567696843b0da74b8d7e1e7f0649b9e at 1, PowerlessProphecyCurseFeatureLevel5 abfd1deaddbf58140b8ba0e0bbb51e46 at
    // 5, ImprovedUncannyDodge 485a18c05792521459c7d06c63128c79 at 10, PowerlessProphecyCurseFeatureLevel15
    // afa1f5b0540be6543ae1605da6a1a429 at 15. The game has no selection there, so the planner's curse selection names no
    // blueprint; the archetype and the option name theirs.
    assert_eq!(
        blueprint(DataAt::FeatureSelection(FeatureSelectionKey::new(
            "oracle/possessed-oracle/curse"
        ))),
        None
    );
    assert_eq!(
        blueprint(DataAt::SelectionOption(SelectionOptionKey::new(
            "oracle/oracles-curse/powerless-prophecy"
        ))),
        Some("88c89ab44a36a67438347e257f70c8fe")
    );
    assert_eq!(
        blueprint(DataAt::Archetype(rules::ArchetypeKey::new(
            "oracle/possessed-oracle"
        ))),
        Some("09035bb5dbb714645bb08ea7db47884c")
    );
    let possessed = with_option(
        unascended(archetype_build("oracle", "oracle/possessed-oracle", 15)),
        1,
        0,
        "oracle/mystery/battle",
    );
    assert!(possible(&possessed));
    let sheet = engine().evaluate(&possessed);
    assert!(sheet.levels[0].slots.iter().all(|slot| {
        slot.kind != selection_kind("oracle/possessed-oracle/curse")
            && slot.kind != selection_kind(CURSES)
    }));
    assert_eq!(
        sheet.levels[0].state.option_count(&SelectionOptionKey::new(
            "oracle/oracles-curse/powerless-prophecy"
        )),
        1
    );
    assert_stages(
        &sheet,
        "powerless-prophecy",
        &[
            ("uncanny-dodge", "8f800ed6ce8c42e8a01fd8f3e990c459", 1),
            ("level-1", "e567696843b0da74b8d7e1e7f0649b9e", 1),
            ("level-5", "abfd1deaddbf58140b8ba0e0bbb51e46", 5),
            (
                "improved-uncanny-dodge",
                "485a18c05792521459c7d06c63128c79",
                10,
            ),
            ("level-15", "afa1f5b0540be6543ae1605da6a1a429", 15),
        ],
    );
}

// ---- one revelation selection, filtered by the mystery ----------------------------------------------------------

const REVELATIONS: &str = "oracle/revelations";

#[test]
fn the_oracles_revelations_are_the_games_one_selection_of_62() {
    // Game: OracleRevelationSelection 60008a10ad7ad6543b1f63016741a5d2, 62 options in m_AllFeatures (61 revelations,
    // FortuneRevelationFeature and MisfortuneRevelationFeature).
    let selection = DATA
        .selections()
        .find(|selection| selection.key.as_str() == REVELATIONS)
        .unwrap();
    assert_eq!(
        blueprint(DataAt::FeatureSelection(selection.key.clone())),
        Some("60008a10ad7ad6543b1f63016741a5d2")
    );
    assert_eq!(selection.options.len(), 62);
    assert_eq!(
        selection
            .filtered_by
            .as_ref()
            .map(FeatureSelectionKey::as_str),
        Some("oracle/mystery")
    );
    // Nothing else holds the game's selection but the Dual-Cursed Oracle's and the Living Deity's copies.
    let copies: BTreeSet<String> = DATA
        .blueprints()
        .into_iter()
        .filter(|(_, guid)| {
            guid.is_some_and(|guid| guid.as_str() == "60008a10ad7ad6543b1f63016741a5d2")
        })
        .filter_map(|(at, _)| at.key().map(str::to_owned))
        .collect();
    assert_eq!(
        copies,
        BTreeSet::from(
            [
                "arcanist/magic-deceiver/revelation",
                "oracle/dual-cursed-oracle/revelation",
                REVELATIONS,
            ]
            .map(String::from)
        )
    );
}

#[test]
fn the_revelation_slot_offers_the_held_mysterys_revelations_and_those_no_mystery_admits() {
    // Game: each revelation's PrerequisiteFeaturesFromList names its mystery's features (OracleRevelationBattlecry
    // 9a67248ce68bd5f47a36d240ff8196e4: OracleBattleMysteryFeature and its Enlightened Philosopher's and Divine
    // Herbalist's); OracleRevelationCombatHealer db1d9829383e78841a6f1145411a54b4 names the Battle ones and
    // OracleLifeMysteryFeature, and OracleClass level 7; FortuneRevelationFeature 16935c4660ee4c7da976b5f6cafb43dd
    // and MisfortuneRevelationFeature 8121fe70a7974970a8a560bd2267cc24 need the Dual-Cursed Oracle
    // (PrerequisiteArchetypeLevel).
    let battle = offered_in(&oracle("battle", 1), &revelation_slot(1));
    let keys: BTreeSet<&str> = battle.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        BTreeSet::from([
            "oracle/revelations/battle/battlecry",
            "oracle/revelations/battle/battlefield-clarity",
            "oracle/revelations/battle/combat-healer",
            "oracle/revelations/battle/iron-skin",
            "oracle/revelations/battle/maneuver-mastery",
            "oracle/revelations/battle/skill-at-arms",
            "oracle/revelations/battle/war-sight",
            "oracle/revelations/battle/weapon-mastery",
            "oracle/revelations/fortune",
            "oracle/revelations/misfortune",
        ])
    );
    for no_dual_curse in [
        "oracle/revelations/fortune",
        "oracle/revelations/misfortune",
    ] {
        assert!(
            matches!(battle[no_dual_curse], Err(Issue::Unmet(_))),
            "{no_dual_curse}"
        );
    }
    // Combat Healer through Life too, from oracle 7.
    let life = with_option(
        oracle("life", 7),
        7,
        0,
        "oracle/revelations/battle/combat-healer",
    );
    assert!(possible(&life));
    let early = with_option(
        oracle("life", 3),
        3,
        0,
        "oracle/revelations/battle/combat-healer",
    );
    assert!(matches!(
        why_not(&early, &revelation_slot(3)),
        Some(Issue::Unmet(_))
    ));
    // A Dual-Cursed Oracle may take Misfortune at 1.
    let dual = with_option(
        archetype_build("oracle", "oracle/dual-cursed-oracle", 1),
        1,
        1,
        "oracle/mystery/battle",
    );
    let dual = with_option(dual, 1, 0, "oracle/revelations/misfortune");
    assert!(possible(&dual));
}

#[test]
fn a_dual_cursed_oracle_takes_fortune_from_oracle_5_and_misfortune_from_1() {
    // Game: FortuneRevelationFeature 16935c4660ee4c7da976b5f6cafb43dd's PrerequisiteArchetypeLevel asks for
    // DualCursedOracleArchetype at OracleClass level 5, MisfortuneRevelationFeature 8121fe70a7974970a8a560bd2267cc24's
    // at level 1. The Dual-Cursed Oracle's revelation slots are at 1, 3 and 5 (its own selection, the first of each
    // row's).
    let dual = with_option(
        archetype_build("oracle", "oracle/dual-cursed-oracle", 5),
        1,
        1,
        "oracle/mystery/battle",
    );
    for level in [1, 3, 5] {
        let offered = offered_in(&dual, &selection_slot(level));
        let fortune = &offered["oracle/revelations/fortune"];
        if level < 5 {
            assert!(matches!(fortune, Err(Issue::Unmet(_))), "{level}");
        } else {
            assert_eq!(fortune, &Ok(()), "{level}");
        }
        assert_eq!(offered["oracle/revelations/misfortune"], Ok(()), "{level}");
    }
}

// ---- links made before -----------------------------------------------------------------------------------------

/// The picks of a Build as the fixture writes them: `address=key`, in slot order.
fn picks(build: &Build) -> String {
    build
        .picks
        .iter()
        .map(|(slot, pick)| format!("{}={}", Address::Slot(slot.clone()), pick.key()))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn links_made_before_the_splits_open_with_the_same_picks() {
    let engine = engine();
    let keys = rules::LinkKeys::new(&DATA);
    let (mut count, mut stripped) = (0, 0);
    for line in LINKS.lines().filter(|line| !line.is_empty()) {
        let mut fields = line.split('\t');
        let (name, link, expected) = (
            fields.next().unwrap(),
            fields.next().unwrap(),
            fields.next().unwrap(),
        );
        let build = rules::decode_build(link, &keys).unwrap_or_else(|_| panic!("{name}: decodes"));
        // It opens as the same Build, unless it holds weapon feats in weapons the character isn't proficient with,
        // which fail since ticket 251: then it opens without them.
        let kept = without_unproficient_picks(build.clone());
        match engine.open_link(link) {
            Ok(opened) => assert_eq!(opened, build, "{name}"),
            Err(_) => {
                assert_ne!(kept, build, "{name}: stale for another reason");
                stripped += 1;
            }
        }
        assert!(engine.is_possible(&kept), "{name}");
        assert_eq!(picks(&build), expected, "{name}");
        count += 1;
    }
    // Four open unchanged; the Dual-Cursed and Possessed Oracles', the Stigmatized Witch's and the Feyspeaker's
    // hold such feats.
    assert_eq!((count, stripped), (8, 4));
    // Combat Healer (Life)'s old key reads as Combat Healer.
    let life = LINKS
        .lines()
        .find(|line| line.starts_with("life oracle with combat healer"))
        .unwrap();
    assert!(life.ends_with("slot.selection.7.0=oracle/revelations/battle/combat-healer"));
    // And the Dual-Cursed Oracle's Misfortune and Fortune theirs.
    let dual = LINKS
        .lines()
        .find(|line| line.starts_with("full oracle oracle/dual-cursed-oracle, fortune at 5"))
        .unwrap();
    assert!(dual.contains(
        "slot.selection.3.0=oracle/revelations/misfortune slot.selection.5.0=oracle/revelations/fortune "
    ));
}
