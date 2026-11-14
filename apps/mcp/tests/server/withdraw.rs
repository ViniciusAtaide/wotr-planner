//! `withdraw`: a choice comes off only by the engine's own Withdrawal, and only when nothing later depends on it.

use rmcp::model::CallToolResult;
use rmcp::serde_json::json;
use rules::Build;
use rules::test_world::{
    build, class_slot, general_slot, pick_feat, pick_option, selection_slot, times,
};

use crate::support::*;

/// `withdraw` on the test world.
async fn withdraw(build: &Build, choice: &str) -> CallToolResult {
    Client::on_the_test_world()
        .await
        .call(
            "withdraw",
            json!({ "build": code(build), "choice": choice }),
        )
        .await
}

/// A successful `withdraw`'s Build.
async fn withdrawn(build: &Build, choice: &str) -> Build {
    let result = withdraw(build, choice).await;
    assert_ne!(result.is_error, Some(true), "{choice}: {result:?}");
    let content = result.structured_content.expect("structured content");
    assert!(content["todo"].is_array());
    decode(&text(&content["build"]))
}

#[tokio::test]
async fn a_leaf_comes_off() {
    let mut fighter = build("human", &["fighter"]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    let after = withdrawn(&fighter, "slot.feat.1.class.0").await;
    assert_eq!(after, build("human", &["fighter"]));
}

#[tokio::test]
async fn a_choice_something_depends_on_is_not_removable() {
    let mut fighter = build("human", &["fighter"]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(general_slot(1), pick_feat("cleave"));
    let refused = withdraw(&fighter, "slot.feat.1.class.0").await;
    assert_eq!(error_code(&refused), "not_removable");
    // Ticket 192: it names the choice that depends on it.
    assert_eq!(
        error_text(&refused),
        "not_removable: cleave depends on `slot.feat.1.class.0`, so it can't be withdrawn now: withdraw cleave \
         first."
    );
    // Cleave, the leaf, comes off.
    let after = withdrawn(&fighter, "slot.feat.1.general.0").await;
    assert!(!after.picks.contains_key(&general_slot(1)));
}

#[tokio::test]
async fn the_last_level_comes_off() {
    let fighter = build("human", &times("fighter", 3));
    let after = withdrawn(&fighter, "level.3").await;
    assert_eq!(after, build("human", &times("fighter", 2)));
    // Only the last level is a choice.
    let refused = withdraw(&fighter, "level.2").await;
    assert_eq!(error_code(&refused), "no_such_choice");
    assert!(error_text(&refused).contains("level.3"));
}

#[tokio::test]
async fn a_levels_class_and_the_racial_ability_are_refused_with_what_to_do() {
    let fighter = build("human", &times("fighter", 2));
    let class = withdraw(&fighter, "level.1.class").await;
    assert_eq!(error_code(&class), "not_removable");
    let reason = error_text(&class);
    assert!(reason.contains("`choose`"), "{reason}");
    assert!(reason.contains("`level.2`"), "{reason}");

    let mut human = build("human", &[]);
    human.start.racial_ability = Some(rules::Ability::Intelligence);
    let racial = withdraw(&human, "start.racial-ability").await;
    assert_eq!(error_code(&racial), "not_removable");
    let reason = error_text(&racial);
    assert!(reason.contains("start.race"), "{reason}");
}

#[tokio::test]
async fn a_choice_the_build_lacks_and_a_stale_code_are_refused() {
    let fighter = build("human", &["fighter"]);
    let lacking = withdraw(&fighter, "slot.feat.1.general.0").await;
    assert_eq!(error_code(&lacking), "no_such_choice");
    let not_an_address = withdraw(&fighter, "level.one").await;
    assert_eq!(error_code(&not_an_address), "no_such_choice");

    let mut forged = build("human", &["fighter"]);
    forged.picks.insert(general_slot(1), pick_feat("cleave"));
    let stale = withdraw(&forged, "slot.feat.1.general.0").await;
    assert_eq!(error_code(&stale), "stale_link");
}

#[tokio::test]
async fn the_build_start_choices_come_off() {
    let fighter = build("human", &["fighter"]);
    // A point-buy score withdrawn is 7.
    let after = withdrawn(&fighter, "start.score.cha").await;
    assert_eq!(after.start.ability_scores.get(rules::Ability::Charisma), 7);
    // The alignment needs the faith: the faith can't come off first.
    let refused = withdraw(&fighter, "start.faith").await;
    assert_eq!(error_code(&refused), "not_removable");
    let after = withdrawn(&fighter, "start.alignment").await;
    assert_eq!(after.start.alignment, None);
}

#[tokio::test]
async fn withdraw_names_one_dependent_and_says_them_for_several() {
    // Ticket 192 (QA): "withdraw that choice first" read as the slot just named.
    let mut fighter = build("human", &times("fighter", 5));
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(general_slot(1), pick_feat("cleave"));
    fighter
        .picks
        .insert(general_slot(5), pick_feat("great-cleave"));
    let refused = withdraw(&fighter, "slot.feat.1.class.0").await;
    assert_eq!(error_code(&refused), "not_removable");
    assert_eq!(
        error_text(&refused),
        "not_removable: cleave and great-cleave depend on `slot.feat.1.class.0`, so it can't be withdrawn now: \
         withdraw them first."
    );
}

#[tokio::test]
async fn withdraw_says_them_first_when_the_names_end_in_and_n_more() {
    // Ticket 192 (QA): four choices need Power Attack; three are named, the rest counted.
    let mut fighter = build("human", &times("fighter", 8));
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(general_slot(1), pick_feat("cleave"));
    fighter
        .picks
        .insert(class_slot(2, 0), pick_feat("critical-mastery"));
    fighter
        .picks
        .insert(general_slot(5), pick_feat("great-cleave"));
    fighter
        .picks
        .insert(general_slot(7), pick_feat("cleaving-finish"));
    let refused = withdraw(&fighter, "slot.feat.1.class.0").await;
    assert_eq!(
        error_text(&refused),
        "not_removable: cleave, critical-mastery, great-cleave and 1 more depend on `slot.feat.1.class.0`, so it \
         can't be withdrawn now: withdraw them first."
    );
}

#[tokio::test]
async fn withdraw_names_the_one_choice_it_would_break_when_no_need_is_unmet() {
    // Ticket 192 (QA): the bloodline's feat is no longer offered once the bloodline goes; the address is named once,
    // by the second clause.
    let mut sorcerer = build("human", &times("sorcerer", 7));
    sorcerer.picks.insert(
        selection_slot(1, 0),
        pick_option("sorcerer/bloodline/draconic-red"),
    );
    sorcerer
        .picks
        .insert(class_slot(7, 0), pick_feat("toughness"));
    let refused = withdraw(&sorcerer, "slot.selection.1.0").await;
    assert_eq!(
        error_text(&refused),
        "not_removable: Would break toughness, so `slot.selection.1.0` can't be withdrawn now: withdraw toughness \
         first."
    );
}
