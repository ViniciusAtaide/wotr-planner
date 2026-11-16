//! A refusal names the pick order within a level (ticket 126), on the real data: a Mythic rank's picks come after
//! its level's slots (ticket 131, item 6). The game keeps no order among one level's feat picks (ticket 248).

use std::collections::BTreeMap;

use rules::{Band, Issue, Parameter, SlotId, SpellSchool, Unmet};

use crate::support::*;

/// The reason the picker gives `pick` in `slot`, which must be an unmet prerequisite a later pick meets.
fn met_later_reason(build: &rules::Build, slot: &SlotId, pick: &rules::Pick) -> String {
    let status = engine()
        .options(build, slot)
        .expect("the slot")
        .into_iter()
        .find(|option| &option.pick == pick)
        .expect("offered")
        .status;
    let Err(issue @ Issue::Unmet(Unmet::MetLater { .. })) = &status else {
        panic!("not met by a later pick: {status:?}");
    };
    engine().issue_text(issue).expect("a reason")
}

#[test]
fn greater_spell_focus_in_the_general_slot_leans_on_spell_focus_in_the_sorcerers_magic_slot() {
    // The Max DC run: Spell Focus (Enchantment) in the class Magic slot, Greater Spell Focus in the general slot.
    // Game: SorcererProgression 997665565ca80a649aedd72455c4df1f gives SorcererBonusFeat
    // d6dd06f454b34014ab0903cb1ed2ade3 at 1, beside the level's feat; SpellFocusGreater
    // 5b04b45b228461c43bad768eb0f7c7bf needs SpellFocus 16fa59cc9a72a6043b566b49184f53fe in the same school.
    let enchantment = Parameter::SpellSchool(SpellSchool::Enchantment);
    let mut swapped = build("sorcerer", 1);
    swapped
        .picks
        .insert(class_slot(1), feat_pick("spell-focus", enchantment.clone()));
    swapped.picks.insert(
        general_slot(1),
        feat_pick("greater-spell-focus", enchantment),
    );
    assert!(possible(&swapped));
}

/// The game applies one level's feat picks in the order the player makes them, so Greater Spell Focus in the
/// sorcerer's Magic slot may lean on Spell Focus in the same level's general slot.
#[test]
fn greater_spell_focus_in_the_sorcerers_magic_slot_leans_on_spell_focus_in_the_same_levels_general_slot()
 {
    // Game: UnitLogic/Class/LevelUp/Actions/SelectFeature.CalculatePriority gives every feat selection the priority
    // Features, and UnitLogic/Class/LevelUp/LevelUpController.AddAction applies actions of one priority in the
    // player's order, each checked (SelectFeature.Check) with the earlier ones applied, WrathDecompiled 6ce4a6b.
    let enchantment = Parameter::SpellSchool(SpellSchool::Enchantment);
    let mut later = build("sorcerer", 1);
    later.picks.insert(
        general_slot(1),
        feat_pick("spell-focus", enchantment.clone()),
    );
    later
        .picks
        .insert(class_slot(1), feat_pick("greater-spell-focus", enchantment));
    assert!(possible(&later));
    // The pickers agree with the walk.
    let agreement = rules::agreement::judge(&engine(), &later);
    assert!(
        agreement.disagreements.is_empty(),
        "{:?}",
        agreement.lines("sorcerer 1")
    );
}

#[test]
fn a_level_11_feat_needing_perception_ii_names_the_mr_4_pick_that_comes_after_it() {
    // The second martial run: Perception II at MR 4, which lands at level 11, after the level's general slot.
    // Game: TricksterImprovedImprovedCritical 56f94badbba018b4b8277ce6e2e79e72 needs ImprovedCritical and
    // TricksterPerceptionTier2Feature e9298851786c5334dba1398e9635a83d (PrerequisitePlayerHasFeature). Where a rank
    // falls among a level's slots is the planner's.
    let falchion = weapon("falchion");
    let mut build = mythic("trickster", 11);
    build.picks.insert(
        path_selection(3),
        pick("trickster/mythic-trick/perception-i"),
    );
    build.picks.insert(
        path_selection(4),
        pick("trickster/improved-mythic-trick/perception-ii"),
    );
    build.picks.insert(
        general_slot(9),
        feat_pick("improved-critical", falchion.clone()),
    );
    assert!(possible(&build));
    assert_eq!(
        met_later_reason(
            &build,
            &general_slot(11),
            &feat_pick("improved-improved-critical", falchion)
        ),
        "Needs Perception II: Perception II is in Mythic rank 4's Improved Mythic Trick slot, which comes after this \
         slot in level 11's pick order"
    );
}

/// The selections whose picks come before a level's feats and talents: the game's band for each selection's group
/// (rules::Band). The planner's selections with no blueprint (the Possessed Oracle's curse, the Spell Master's
/// school, the slayer's combat style) offer one option and take no pick, or are feats, so stay in the Features band.
#[test]
fn the_selections_outside_the_features_band_are_the_games() {
    // Game: SelectFeature.CalculatePriority gives LevelUpActionPriority.Path to the groups Domain, SpecialistSchool,
    // BloodLine, DruidDomain, BlightDruidDomain, DruidBond, OracleCurse, ShamanSpirit, OracleMystery,
    // WarpriestBlessing and LichSkeletalUpgrade (among others), ReplaceSpellbook to ThassilonianSpellbook (among the
    // spellbooks), and Features to every other group (UnitLogic/Class/LevelUp/Actions/SelectFeature.cs,
    // WrathDecompiled 6ce4a6b). Each selection's group is its blueprint's.
    let expected = [
        // FakeDivineSparkSelection e6d337e709b4478cb6e5389d3f85b601 (Domain)
        ("arcanist/magic-deceiver/domain", Band::Path),
        // DomainsSelection 48525e5da45c9c243a343fc6545dbdb9 (Domain)
        ("cleric/domain", Band::Path),
        // DruidDomainSelection 5edfe84c93823d04f8c40ca2b4e0f039 (DruidDomain)
        ("druid/domain", Band::Path),
        // DruidBondSelection 3830f3630a33eba49b60f511b4c8f2a8 (DruidBond)
        ("druid/nature-bond", Band::Path),
        // BlightDruidBond 096fc02f6cc817a43991c4b437e12b8e (BlightDruidDomain)
        ("druid/blight-druid/domain", Band::Path),
        // WinterChildDomainSelection 79922e274bf4444b93ea77ae0f876bd2 (DruidDomain)
        ("druid/winter-child/domain", Band::Path),
        // DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b (Domain)
        ("hunter/divine-hunter/domain", Band::Path),
        // DomainsSelection 48525e5da45c9c243a343fc6545dbdb9 (Domain)
        ("inquisitor/domain", Band::Path),
        ("inquisitor/sacred-huntsmaster/domain", Band::Path),
        // EldritchScionBloodlineSelection 94c29f69cdc34594a6a4677441ed7375 (BloodLine)
        ("magus/eldritch-scion/bloodline", Band::Path),
        // OracleMysterySelection 5531b975dcdf0e24c98f1ff7e017e741 (OracleMystery)
        ("oracle/mystery", Band::Path),
        ("oracle/lone-strider/mystery", Band::Path),
        ("oracle/wind-whisperer/mystery", Band::Path),
        // OracleCurseSelection b0a5118b4fb793241bc7042464b23fab (OracleCurse)
        ("oracle/oracles-curse", Band::Path),
        ("witch/stigmatized-witch/curse", Band::Path),
        // SecondMystery 277b0164740b97945a3f8022bd572f48 (OracleMystery)
        ("oracle/second-mystery", Band::Path),
        // ShamanSPiritSelection 00c8c566d1825dd4a871250f35285982 (ShamanSpirit)
        ("shaman/spirit", Band::Path),
        ("shaman/wildland-shaman/spirit", Band::Path),
        // SorcererBloodlineSelection 24bef8d1bee12274686f6da6ccbc8914 (BloodLine)
        ("sorcerer/bloodline", Band::Path),
        ("sorcerer/empyreal-sorcerer/bloodline", Band::Path),
        ("sorcerer/sage-sorcerer/bloodline", Band::Path),
        ("sorcerer/seeker-sorcerer/bloodline", Band::Path),
        ("sorcerer/sylvan-sorcerer/bloodline", Band::Path),
        // BlessingSelection 6d9dcc2a59210a14891aeedb09d406aa (WarpriestBlessing)
        ("warpriest/blessings", Band::Path),
        // SpecialistSchoolSelection 5f838049069f1ac4d804ce0862ab5110 (SpecialistSchool)
        ("wizard/school", Band::Path),
        ("wizard/elemental-specialist/school", Band::Path),
        // ThassilonianSchoolSelection f431178ec0e2b4946a34ab504bb46285 (ThassilonianSpellbook)
        ("wizard/thassilonian-specialist/school", Band::Spellbook),
        // LichSkeletalUpgradeSelection a434dddab8026e947bc16eb36d18a783 (LichSkeletalUpgrade)
        ("lich/skeleton-upgrade", Band::Path),
    ];
    let expected: BTreeMap<&str, Band> = expected.into_iter().collect();
    let actual: BTreeMap<&str, Band> = (DATA.selections.iter())
        .filter(|selection| selection.band != Band::Features)
        .map(|selection| (selection.key.as_str(), selection.band))
        .collect();
    assert_eq!(actual, expected);
    // The sorcerer's bonus feat slot and the rogue's talents are the Features band.
    // Game: SorcererBonusFeat d6dd06f454b34014ab0903cb1ed2ade3 (WizardFeat); RogueTalentSelection
    // c074a5d615200494b8f2a9c845799d93 (RogueTalent)
    let band = |key: &str| {
        (DATA.selections.iter())
            .find(|selection| selection.key.as_str() == key)
            .unwrap_or_else(|| panic!("no selection {key}"))
            .band
    };
    assert_eq!(band("rogue/rogue-talents"), Band::Features);
}
