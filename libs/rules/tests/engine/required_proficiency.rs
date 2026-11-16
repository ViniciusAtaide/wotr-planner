//! A feat that requires proficiency is taken only for a weapon the character is proficient with, in every slot,
//! whatever the slot's list ignores (ticket 251): the game's `RequireProficiency`, which
//! `BlueprintParametrizedFeature.CanSelect` checks on the feature's own flag, not the selection's (WrathDecompiled
//! 6ce4a6b).

use rules::{Engine, GameData, Issue, Pick, WeaponKey, agreement::judge};

use crate::support::*;

/// The test world with Weapon Focus requiring proficiency.
fn requiring() -> Engine {
    let mut data: GameData = world();
    feat_mut(&mut data, "weapon-focus").requires_proficiency = true;
    engine_on(data)
}

fn status(
    engine: &Engine,
    build: &rules::Build,
    slot: &rules::SlotId,
    pick: &Pick,
) -> Result<(), Issue> {
    engine
        .options(build, slot)
        .expect("the slot")
        .into_iter()
        .find(|option| &option.pick == pick)
        .expect("offered")
        .status
}

fn not_proficient(key: &str) -> Issue {
    Issue::NotProficient {
        weapon: WeaponKey::new(key),
    }
}

#[test]
fn a_slot_that_ignores_prerequisites_still_takes_only_weapons_the_character_is_proficient_with() {
    // The test world's warpriest holds no proficiency feat: of its weapons he is proficient with the unarmed strike,
    // which no list names, alone. His level-1 slot's list ignores prerequisites.
    let engine = requiring();
    let mut build = build("human", &["warpriest"]);
    let longsword = pick_feat_for("weapon-focus", weapon("longsword"));
    let refused = status(&engine, &build, &class_slot(1, 0), &longsword);
    assert_eq!(refused, Err(not_proficient("longsword")));
    assert_eq!(
        engine.issue_text(&refused.unwrap_err()).as_deref(),
        Some("Not proficient with longsword")
    );
    assert_eq!(
        status(
            &engine,
            &build,
            &class_slot(1, 0),
            &pick_feat_for("weapon-focus", weapon("unarmed-strike"))
        ),
        Ok(())
    );
    build.picks.insert(class_slot(1, 0), longsword);
    let failures = engine.failures(&build);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].issue, not_proficient("longsword"));

    // Without the flag, the slot takes any weapon, as before.
    assert!(engine_on(world()).is_possible(&build));
}

#[test]
fn proficiency_taken_at_the_same_level_opens_the_weapon() {
    // Exotic Weapon Proficiency (Bastard Sword) in the general slot comes after the fighter's class slot, but both
    // are the Features band (ticket 248): Weapon Focus (Bastard Sword) in the class slot holds.
    let engine = requiring();
    let mut build = build("human", &["fighter"]);
    let bastard_sword = weapon("bastard-sword");
    build.picks.insert(
        general_slot(1),
        pick_feat_for("exotic-weapon-proficiency", bastard_sword.clone()),
    );
    assert_eq!(
        status(
            &engine,
            &build,
            &class_slot(1, 0),
            &pick_feat_for("weapon-focus", bastard_sword.clone())
        ),
        Ok(())
    );
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", bastard_sword),
    );
    assert!(engine.failures(&build).is_empty());
    let agreement = judge(&engine, &build);
    assert!(
        agreement.disagreements.is_empty(),
        "{:?}",
        agreement.lines("fighter 1")
    );
}

#[test]
fn a_list_whose_pick_skips_proficiency_takes_any_weapon() {
    // As the Sword Saint's chosen weapon: the game's pick there is a feature that gives Weapon Focus and proficiency.
    let mut data: GameData = world();
    feat_mut(&mut data, "weapon-focus").requires_proficiency = true;
    class_mut(&mut data, "warpriest").feat_lists[0].skips_proficiency = true;
    let engine = engine_on(data);
    let mut build = build("human", &["warpriest"]);
    build.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    assert!(engine.is_possible(&build));
}
