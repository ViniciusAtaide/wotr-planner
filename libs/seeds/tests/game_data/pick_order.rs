//! A refusal names the pick order within a level (ticket 126), on the real data: a Mythic rank's picks come after
//! its level's slots (ticket 131, item 6). The game keeps no order among one level's feat picks (ticket 248).

use rules::{Issue, Parameter, SlotId, SpellSchool, Unmet};

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
#[ignore = "waits on ticket 248: the game gives no order among one level's picks"]
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
