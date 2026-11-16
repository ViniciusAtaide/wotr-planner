//! Ticket 244: a feature the game has once is one entry, whichever classes grant it. Where a class, archetype and
//! path each had its own copy, a prerequisite now reads the one feature's rank across them, as the game reads the one
//! blueprint's.

use rules::{ArchetypeKey, ClassChoice, ClassKey, Issue, Prerequisite};

use crate::support::{classes_build, conditions, engine, faithful};

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
