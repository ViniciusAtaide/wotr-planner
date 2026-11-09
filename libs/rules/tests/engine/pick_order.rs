//! A refusal names the pick order within a level (ticket 126): a prerequisite that a pick in a later slot of the same
//! character level would meet says so, names that slot, and says to swap the two picks when each slot takes the
//! other's.

use rules::{FeatSlot, Issue, LaterPick, Pick, Prerequisite, SlotId, SlotKind, Unmet};

use crate::support::*;

/// A slot's option's status for `pick`.
fn status(build: &rules::Build, slot: &SlotId, pick: &Pick) -> Result<(), Issue> {
    engine()
        .options(build, slot)
        .expect("the slot")
        .into_iter()
        .find(|option| &option.pick == pick)
        .expect("offered")
        .status
}

fn reason(issue: &Issue) -> String {
    engine().issue_text(issue).expect("a reason")
}

fn needs_feat(key: &str) -> Unmet {
    Unmet::Condition {
        needs: Prerequisite::Feat(feat(key)),
        has: None,
    }
}

#[test]
fn a_feat_whose_prerequisite_a_later_slot_of_the_level_holds_names_that_slot_and_says_to_swap() {
    // The fighter's level-1 Combat feat slot comes before the general slot.
    let fighter = build("human", &["fighter"]);
    let mut later = fighter.clone();
    later
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    let status = status(&later, &class_slot(1, 0), &pick_feat("cleave"));
    assert_eq!(
        status,
        Err(Issue::Unmet(Unmet::MetLater {
            unmet: Box::new(needs_feat("power-attack")),
            later: Box::new(LaterPick {
                slot: general_slot(1),
                kind: SlotKind::Feat(FeatSlot::General),
                pick: pick_feat("power-attack"),
                level: 1,
                swap: true,
            }),
        }))
    );
    assert_eq!(
        reason(&status.unwrap_err()),
        // The test world names a feat by its key.
        "Needs power-attack: power-attack is in level 1's Feat slot, which comes after this slot in the level's \
         pick order; swap them"
    );

    // In the earlier slot it holds for the later one.
    let mut swapped = fighter;
    swapped
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert_eq!(status_of_general(&swapped, &pick_feat("cleave")), Ok(()));
}

fn status_of_general(build: &rules::Build, pick: &Pick) -> Result<(), Issue> {
    status(build, &general_slot(1), pick)
}

#[test]
fn a_later_pick_that_meets_only_part_of_a_prerequisite_leaves_the_plain_reason() {
    // Great Cleave needs Cleave and BAB +4: Cleave in the general slot doesn't open it.
    let mut build = build("human", &["fighter"]);
    build
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    build.picks.insert(general_slot(1), pick_feat("cleave"));
    let Err(Issue::Unmet(unmet)) = status(&build, &race_slot(0), &pick_feat("great-cleave")) else {
        panic!("Great Cleave is open");
    };
    assert!(!matches!(unmet, Unmet::MetLater { .. }), "{unmet:?}");
}

#[test]
fn a_mythic_ranks_pick_comes_after_its_levels_slots() {
    // MR 4 comes at level 11, after the level's own slots: Dodge in the Trickster's MR 4 bonus feat slot is too late
    // for Crane Style in the level-11 general slot. The bonus slot takes only Dodge and Toughness, so no swap.
    let mut build = build("human", &times("fighter", 11));
    build.mythic_path = Some(path("trickster"));
    let path_feat = SlotId::PathFeat { rank: 4, index: 0 };
    build.picks.insert(path_feat.clone(), pick_feat("dodge"));
    let status = status(&build, &general_slot(11), &pick_feat("crane-style"));
    let Err(Issue::Unmet(Unmet::MetLater { unmet, later })) = &status else {
        panic!("{status:?}");
    };
    assert_eq!(**unmet, needs_feat("dodge"));
    assert_eq!(
        (&later.slot, later.level, later.swap),
        (&path_feat, 11, false)
    );
    assert_eq!(
        reason(&status.clone().unwrap_err()),
        "Needs dodge: dodge is in Mythic rank 4's trickster/bonus-feats slot, which comes after this slot in level \
         11's pick order"
    );
}
