//! A refusal names the pick order within a level (ticket 126), on the real data: the sorcerer's level-1 Magic slot
//! comes before the general slot, and a Mythic rank's picks come after its level's slots (ticket 131, item 6).

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
fn greater_spell_focus_in_the_sorcerers_magic_slot_names_spell_focus_in_the_later_general_slot() {
    // The Max DC run: Spell Focus (Enchantment) in the general slot, Greater Spell Focus in the class Magic slot.
    let enchantment = Parameter::SpellSchool(SpellSchool::Enchantment);
    let sorcerer = build("sorcerer", 1);
    let mut later = sorcerer.clone();
    later.picks.insert(
        general_slot(1),
        feat_pick("spell-focus", enchantment.clone()),
    );
    assert_eq!(
        met_later_reason(
            &later,
            &class_slot(1),
            &feat_pick("greater-spell-focus", enchantment.clone())
        ),
        "Needs Spell Focus in the same school: Spell Focus (Enchantment) is in level 1's Feat slot, which comes after \
         this slot in the level's pick order; swap them"
    );

    // Swapped, both hold.
    let mut swapped = sorcerer;
    swapped
        .picks
        .insert(class_slot(1), feat_pick("spell-focus", enchantment.clone()));
    swapped.picks.insert(
        general_slot(1),
        feat_pick("greater-spell-focus", enchantment),
    );
    assert!(possible(&swapped));
}

#[test]
fn a_level_11_feat_needing_perception_ii_names_the_mr_4_pick_that_comes_after_it() {
    // The second martial run: Perception II at MR 4, which lands at level 11, after the level's general slot.
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
