//! Ticket 244: a feature the game has once is one entry, whichever classes grant it. Where a class, archetype and
//! path each had its own copy, a prerequisite now reads the one feature's rank across them, as the game reads the one
//! blueprint's.

use rules::{ArchetypeKey, ClassChoice, ClassKey, FeatureKey, Issue, Prerequisite};

use crate::support::{at, build, classes_build, conditions, engine, faithful};

/// The Arcane Trickster's entry for a Build of `classes`, one per level, an alchemist as a Vivisectionist.
fn arcane_trickster_status(classes: &[&str]) -> Result<(), Issue> {
    let mut build = classes_build(classes);
    build.class_choices.insert(
        ClassKey::new("alchemist"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("alchemist/vivisectionist")),
            ..ClassChoice::default()
        },
    );
    let build = faithful(build);
    let level = u8::try_from(classes.len() + 1).unwrap();
    engine()
        .class_options(&build, level)
        .expect("the next level")
        .into_iter()
        .find(|option| option.class == ClassKey::new("arcane-trickster"))
        .expect("the class is offered")
        .status
}

/// Whether a failing condition of the entry is about Sneak Attack (the feature, or Accomplished Sneak Attacker).
fn sneak_attack_unmet(status: &Result<(), Issue>) -> bool {
    let Err(Issue::Unmet(unmet)) = status else {
        return false;
    };
    conditions(unmet).into_iter().any(|(needs, _)| {
        matches!(needs, Prerequisite::Feature { feature, .. } if feature.as_str() == "sneak-attack")
            || matches!(needs, Prerequisite::Feat(feat) if feat.as_str() == "accomplished-sneak-attacker")
    })
}

/// Sneak Attack from two classes adds up: a rogue 1 and Vivisectionist 1 has the 2d6 the Arcane Trickster asks for,
/// where the planner's copies once counted each class's alone. Its other conditions (skill ranks) still fail here.
#[test]
fn the_arcane_trickster_reads_sneak_attack_summed_across_classes() {
    // Game: ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3 asks `PrerequisiteFullStatValue` SneakAttack 2;
    // SneakAttack 9b9eac6709e1c084cb18c3a366e0ec87 (`Ranks` 20, granted by RogueProgression at 1 and by
    // VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 at 1) adds `AddStatBonus` SneakAttack 1 times its rank
    // (`UnitLogic/FactLogic/AddStatBonus.cs` `OnTurnOn`, Value * Fact.GetRank(), WrathDecompiled 6ce4a6b), and a grant
    // of a feature held adds a rank (`UnitLogic/Feature.cs` `AddRank`).
    let two_classes =
        arcane_trickster_status(&["rogue", "alchemist", "wizard", "wizard", "wizard"]);
    assert!(!sneak_attack_unmet(&two_classes), "{two_classes:?}");
    // One class's 1d6 is not enough.
    let one_class = arcane_trickster_status(&["rogue", "wizard", "wizard", "wizard", "wizard"]);
    assert!(sneak_attack_unmet(&one_class), "{one_class:?}");
}

/// Uncanny Dodge, the one `UncannyDodgeChecker`, for a Build of one class per level, at its last level.
fn uncanny_dodge_for(classes: &[&str]) -> String {
    let build = classes_build(classes);
    let state = at(&build, u8::try_from(classes.len()).unwrap());
    engine().feature_name_for(&FeatureKey::new(UNCANNY_DODGE), &state)
}

const UNCANNY_DODGE: &str = "uncanny-dodge";

/// A collapsed feature several classes grant is named for the Build's own class, the one whose rows reached it: a
/// Rogue's Uncanny Dodge is the Rogue's and a Barbarian's the Barbarian's, where the first holder in key order once
/// named the Barbarian for both. Its name is told apart because War Sight's Uncanny Dodge shares it.
#[test]
fn a_collapsed_feature_is_named_for_the_builds_own_class() {
    // Game: UncannyDodgeChecker 8f800ed6ce8c42e8a01fd8f3e990c459 is granted by RogueProgression
    // b57b2a75a5abcaf47a01cf84672b50e9 at 4 and BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12 at 2;
    // OracleRevelationWarSight7 374008ffbe3d8884aa79077766fa3f3f is another blueprint of the same name.
    assert_eq!(uncanny_dodge_for(&["rogue"; 4]), "Uncanny Dodge (Rogue)");
    assert_eq!(
        uncanny_dodge_for(&["barbarian"; 2]),
        "Uncanny Dodge (Barbarian)"
    );
    // A barbarian 1 has no Uncanny Dodge yet: the rogue's 4th level gave it.
    assert_eq!(
        uncanny_dodge_for(&["barbarian", "rogue", "rogue", "rogue", "rogue"]),
        "Uncanny Dodge (Rogue)"
    );
}

/// Without a Build (search, describe, a prerequisite's text), a feature several classes grant names none of them,
/// never a wrong one, and one only one holder grants still names it.
#[test]
fn without_a_build_a_feature_several_classes_grant_names_no_holder() {
    let engine = engine();
    let uncanny_dodge = FeatureKey::new(UNCANNY_DODGE);
    assert_eq!(engine.feature_name_apart(&uncanny_dodge), "Uncanny Dodge");
    assert_eq!(
        engine.prerequisite_text(
            &Prerequisite::Feature {
                feature: uncanny_dodge,
                count: 1
            },
            None
        ),
        "Uncanny Dodge"
    );
    assert_eq!(
        engine.feature_name_apart(&FeatureKey::new(
            "oracle/revelations/battle/war-sight/uncanny-dodge"
        )),
        "Uncanny Dodge (War Sight)"
    );
    // A class and its archetypes grant the Barbarian's Damage Reduction: the class is its one holder.
    assert_eq!(
        engine.feature_name_apart(&FeatureKey::new("barbarian/damage-reduction")),
        "Damage Reduction (Barbarian)"
    );
    // A Build that holds none of the holders reads as without a Build.
    let wizard = at(&build("wizard", 1), 1);
    assert_eq!(
        engine.feature_name_for(&FeatureKey::new(UNCANNY_DODGE), &wizard),
        "Uncanny Dodge"
    );
}
