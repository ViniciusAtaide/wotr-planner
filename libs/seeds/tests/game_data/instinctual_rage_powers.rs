//! The Instinctual Warrior's monk rage powers (ticket 66).

use std::collections::BTreeSet;

use rules::{
    Ability, ArchetypeKey, Build, ClassKey, FeatKey, Prerequisite, Repeat, SelectionOptionKey,
    SlotId,
};

use crate::support::*;

const RAGE_POWER: &str = "barbarian/rage-power";
const INSTINCTUAL_WARRIOR: &str = "barbarian/instinctual-warrior";
/// The Instinctual Warrior's own rage power selection (ticket 306h).
const INSTINCTUAL_RAGE_POWER: &str = "barbarian/instinctual-warrior/rage-power";

/// The guide's monk abilities and ki powers an Instinctual Warrior may take as rage powers
/// (`barbarian#rage-power-instinctual-warrior`): each one's key in the rage power selection, its name, and the
/// Instinctual Warrior level its blueprint's `PrerequisiteArchetypeLevel` names (`None`: Deflect Arrows, whose level 4
/// is the archetype's first rage power, and which needs Improved Unarmed Strike and Dex 13 too).
const MONK_RAGE_POWERS: [(&str, &str, Option<u8>); 5] = [
    // Game: RageDeflectArrowsFeature d971bbb15f354621ac37fd51d0ab9044
    (
        "barbarian/rage-power/deflect-arrows-instinctual-warrior",
        "Deflect Arrows (Instinctual Warrior)",
        None,
    ),
    // Game: RagePurityOfBodyFeature a4418aa778cb4ebd965d78af19becced: level 5
    (
        "barbarian/rage-power/purity-of-body-instinctual-warrior",
        "Purity of Body (Instinctual Warrior)",
        Some(5),
    ),
    // Game: RageDiamondBodyFeature 39d6ef0cb9794afbb5af3cae7288029d: level 12
    (
        "barbarian/rage-power/diamond-body-instinctual-warrior",
        "Diamond Body (Instinctual Warrior)",
        Some(12),
    ),
    // Game: RageDiamondSoulFeature d4c929bffb80464bbe6daf1471088bf8: level 14
    (
        "barbarian/rage-power/diamond-soul-instinctual-warrior",
        "Diamond Soul (Instinctual Warrior)",
        Some(14),
    ),
    // Game: RagePerfectSelfFeature 07777880a89d434c896660e4a521df92: level 20
    (
        "barbarian/rage-power/perfect-self-instinctual-warrior",
        "Perfect Self (Instinctual Warrior)",
        Some(20),
    ),
];

/// What a monk power needs: the archetype and the level (equivalent model: the game's `PrerequisiteArchetypeLevel` is
/// the archetype taken and the barbarian level), or for Deflect Arrows the archetype at barbarian 4, Improved Unarmed
/// Strike and Dex 13 (the game's Int 3 always holds: no score goes below 3).
fn monk_power_prerequisite(level: Option<u8>) -> Prerequisite {
    let archetype = Prerequisite::Archetype(ArchetypeKey::new(INSTINCTUAL_WARRIOR));
    let barbarian = |level| Prerequisite::ClassLevel {
        class: ClassKey::new("barbarian"),
        level,
    };
    let mut all = Vec::new();
    match level {
        Some(level) => all.extend([archetype, barbarian(level)]),
        None => {
            all.push(Prerequisite::All(vec![barbarian(4), archetype]));
            all.push(Prerequisite::Feat(FeatKey::new("improved-unarmed-strike")));
            all.push(Prerequisite::AbilityScore {
                ability: Ability::Dexterity,
                score: 13,
            });
        }
    }
    let mut expected = Prerequisite::All(all);
    expected.canonicalize();
    expected
}

/// The Instinctual Warrior's rage power levels, by its table.
const INSTINCTUAL_RAGE_POWER_LEVELS: [u8; 7] = [4, 8, 10, 12, 16, 18, 20];

/// A chaotic neutral Instinctual Warrior of `levels` levels with Dex 13 (the game's Deflect Arrows needs it), with
/// Improved Unarmed Strike at 1 when `unarmed`.
fn instinctual_warrior(levels: usize, unarmed: bool) -> Build {
    let mut build = chaotic(archetype_build("barbarian", INSTINCTUAL_WARRIOR, levels));
    build.start.ability_scores = build
        .start
        .ability_scores
        .with(Ability::Dexterity, 13)
        .expect("Dex 13 fits the point buy");
    if unarmed {
        build
            .picks
            .insert(general_slot(1), feat_pick_plain("improved-unarmed-strike"));
    }
    build
}

/// Whether a monk power needing barbarian level `needs` is open in a slot at `level`: Deflect Arrows (`None`) from its
/// first rage power, the others from the level they need.
fn opens_at(needs: Option<u8>, level: u8) -> bool {
    needs.is_none_or(|needs| needs <= level)
}

/// "Besides the normal Barbarian rage powers, an Instinctual Warrior can select a number of Monk abilities and Ki
/// powers": five options of the barbarian's rage power selection, each needing the archetype and the level (or, for
/// Deflect Arrows, the feat) the guide gives. Extra Rage Power's "Rage Power" is met by holding any of them too (`option_lists.rs` checks its list).
#[test]
fn the_instinctual_warriors_monk_powers_are_rage_power_options_for_it_alone() {
    for (key, name, level) in MONK_RAGE_POWERS {
        let option = selection_option(key);
        assert_eq!(option.name, name);
        assert_eq!(
            option.guide.as_ref().unwrap().0,
            "barbarian#rage-power-instinctual-warrior"
        );
        // Game: each row's blueprint in MONK_RAGE_POWERS: Ranks 1.
        assert_eq!(option.repeat(), Repeat::Once, "{name}");
        // Game: each row's blueprint.
        assert_eq!(
            option.prerequisite,
            Some(monk_power_prerequisite(level)),
            "{name}"
        );
    }
    let selection = DATA
        .selection(&rules::FeatureSelectionKey::new(INSTINCTUAL_RAGE_POWER))
        .expect("the Instinctual Warrior's rage power selection");
    // Game: InstinctualWarriorRagePowerSelection 609f0e5336084442a0dafa3abd4d31c5 (the 33 of RagePowerSelection
    // 28710502f46848d48b3f0d6132817c4e and the five): the five are its own options, the 33 shared with the barbarian's.
    assert_eq!(selection.options.len(), 5);
    assert_eq!(selection.shared_options.len(), 33);
}

/// The game's Purity of Body needs Instinctual Warrior level 5 and Diamond Soul 14, and Deflect Arrows Dex 13 as well
/// as Improved Unarmed Strike: Purity of Body opens in an Extra Rage Power slot at 5, and the reasons name 5 and 14.
#[test]
fn the_instinctual_warriors_monk_powers_need_the_levels_and_dex_their_blueprints_name() {
    // Game: RageDeflectArrowsFeature d971bbb15f354621ac37fd51d0ab9044 (ImprovedUnarmedStrike
    // 7812ad3672a4b9a4fb894ea402095167, Dex 13, PrerequisiteArchetypeLevel 4); RagePurityOfBodyFeature
    // a4418aa778cb4ebd965d78af19becced (level 5); RageDiamondSoulFeature d4c929bffb80464bbe6daf1471088bf8 (level 14).
    for (key, name, level) in MONK_RAGE_POWERS {
        assert_eq!(
            selection_option(key).prerequisite,
            Some(monk_power_prerequisite(level)),
            "{name}"
        );
    }
    let build = instinctual_warrior(20, true);
    assert_eq!(
        lock_reason(&build, 4, MONK_RAGE_POWERS[1].0),
        "Needs Barbarian level 5 (have 4)"
    );
    assert_eq!(
        lock_reason(&build, 12, MONK_RAGE_POWERS[3].0),
        "Needs Barbarian level 14 (have 12)"
    );
    // Without Dex 13, Deflect Arrows is locked.
    let mut clumsy = instinctual_warrior(4, true);
    clumsy.start.ability_scores = clumsy
        .start
        .ability_scores
        .with(Ability::Dexterity, 10)
        .unwrap();
    assert!(!open_at(&clumsy, 4).contains(MONK_RAGE_POWERS[0].0));
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a: its slot at 5 offers Purity of Body.
    let mut extra = instinctual_warrior(5, true);
    extra
        .picks
        .insert(selection_slot(4), pick(MONK_RAGE_POWERS[0].0));
    extra
        .picks
        .insert(general_slot(5), feat_pick_plain("extra-rage-power"));
    assert!(open_at_slot(&extra, &extra_slot(5, 0)).contains(MONK_RAGE_POWERS[1].0));
}

/// The options a slot leaves open, by key.
fn open_at_slot(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    engine()
        .options(build, slot)
        .unwrap()
        .into_iter()
        .filter(|option| option.status.is_ok())
        .map(|option| option.pick.key().to_owned())
        .collect()
}

/// The ticket's: an Instinctual Warrior's rage power picker offers each monk power from its level and not before,
/// with a reason; Deflect Arrows needs Improved Unarmed Strike. One taken at its level is possible, one taken early
/// isn't, and the sheet shows it picked.
#[test]
fn an_instinctual_warrior_is_offered_each_monk_power_from_its_level_and_not_before() {
    let engine = engine();
    let build = instinctual_warrior(20, true);
    assert!(possible(&build));
    // Game: InstinctualWarriorArchetype adffdd8a99094a89823a79292a503ee9: InstinctualWarriorRagePowerSelection
    // 609f0e5336084442a0dafa3abd4d31c5 at 4, 8, 10, 12, 16, 18, 20.
    assert_eq!(
        levels_with_slot_of(&build, INSTINCTUAL_RAGE_POWER),
        INSTINCTUAL_RAGE_POWER_LEVELS
    );
    // Game: each row's blueprint in MONK_RAGE_POWERS (Purity of Body's 5 and Diamond Soul's 14 fall between slots).
    for level in INSTINCTUAL_RAGE_POWER_LEVELS {
        let open = open_at(&build, level);
        for (key, name, needs) in MONK_RAGE_POWERS {
            assert_eq!(
                open.contains(key),
                opens_at(needs, level),
                "{name} at {level}"
            );
        }
    }
    // Game: RageDiamondBodyFeature 39d6ef0cb9794afbb5af3cae7288029d (12); RagePerfectSelfFeature
    // 07777880a89d434c896660e4a521df92 (20). Purity of Body's and Diamond Soul's reasons are
    // `the_instinctual_warriors_monk_powers_need_the_levels_and_dex_their_blueprints_name`'s.
    assert_eq!(
        lock_reason(&build, 10, MONK_RAGE_POWERS[2].0),
        "Needs Barbarian level 12 (have 10)"
    );
    assert_eq!(
        lock_reason(&build, 18, MONK_RAGE_POWERS[4].0),
        "Needs Barbarian level 20 (have 18)"
    );
    // Without Improved Unarmed Strike, Deflect Arrows is locked. Game: RageDeflectArrowsFeature
    // d971bbb15f354621ac37fd51d0ab9044 needs ImprovedUnarmedStrike 7812ad3672a4b9a4fb894ea402095167.
    let bare = instinctual_warrior(4, false);
    assert_eq!(
        lock_reason(&bare, 4, MONK_RAGE_POWERS[0].0),
        "Needs Improved Unarmed Strike"
    );

    // Each taken at its level: possible, and the sheet shows it in its slot.
    let mut taken = build.clone();
    for (slot, (key, _, _)) in [4, 8, 12, 16, 20].into_iter().zip(MONK_RAGE_POWERS) {
        taken.picks.insert(selection_slot(slot), pick(key));
    }
    assert!(possible(&taken));
    let sheet = engine.evaluate(&taken);
    let slot = sheet.slot(&selection_slot(8)).unwrap();
    assert_eq!(engine.slot_label(slot), "Rage Power");
    assert_eq!(
        engine.option_name(&SelectionOptionKey::new(MONK_RAGE_POWERS[1].0)),
        "Purity of Body (Instinctual Warrior)"
    );
    // A power taken is "Already taken" in a later slot. Game: RageDeflectArrowsFeature
    // d971bbb15f354621ac37fd51d0ab9044 (Ranks 1).
    assert_eq!(
        lock_reason(&taken, 10, MONK_RAGE_POWERS[0].0),
        "Already taken"
    );

    // Taken before its level: impossible, for that reason. Game: RageDiamondBodyFeature
    // 39d6ef0cb9794afbb5af3cae7288029d (12).
    let mut early = instinctual_warrior(8, true);
    early
        .picks
        .insert(selection_slot(8), pick(MONK_RAGE_POWERS[2].0));
    assert_eq!(
        why_not(&early, &selection_slot(8)).and_then(|issue| engine.issue_text(&issue)),
        Some("Needs Barbarian level 12 (have 8)".to_owned())
    );
}

/// The ticket's: a barbarian without the archetype (plain, or another archetype) is offered none of them in any rage power
/// slot (the game's list for the barbarian holds none; ticket 306h), and can't take one.
#[test]
fn a_barbarian_without_the_archetype_is_not_offered_the_monk_powers() {
    // Game: RagePowerSelection 28710502f46848d48b3f0d6132817c4e holds none of them; each blueprint in
    // MONK_RAGE_POWERS needs InstinctualWarriorArchetype adffdd8a99094a89823a79292a503ee9.
    for archetype in [None, Some("barbarian/armored-hulk")] {
        let mut build = match archetype {
            Some(archetype) => chaotic(archetype_build("barbarian", archetype, 20)),
            None => barbarian(20),
        };
        build
            .picks
            .insert(general_slot(1), feat_pick_plain("improved-unarmed-strike"));
        assert!(possible(&build), "{archetype:?}");
        let levels = levels_with_slot_of(&build, RAGE_POWER);
        assert!(!levels.is_empty());
        for level in levels {
            let offered = offered(&build, level);
            for (key, name, _) in MONK_RAGE_POWERS {
                assert!(
                    !offered.contains_key(key),
                    "{name} at {level} ({archetype:?})"
                );
            }
        }
        let mut taken = build.clone();
        taken
            .picks
            .insert(selection_slot(20), pick(MONK_RAGE_POWERS[4].0));
        assert!(!possible(&taken), "{archetype:?}");
    }
}

/// The ticket's: Extra Rage Power gives an Instinctual Warrior a rage power slot offering the monk powers as its own
/// slots do, from their levels. Holding only a monk power meets the feat's "Rage Power".
#[test]
fn extra_rage_power_offers_an_instinctual_warrior_its_monk_powers() {
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a needs a rage power selection held (equivalent
    // model: a monk power held at 4 means the slot at 4 was reached); a selection, so it repeats
    // (Blueprints/Classes/Selection/BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b).
    let engine = engine();
    let mut build = instinctual_warrior(9, true);
    build
        .picks
        .insert(selection_slot(4), pick(MONK_RAGE_POWERS[0].0));
    build
        .picks
        .insert(general_slot(5), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    build
        .picks
        .insert(general_slot(9), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    for level in [5, 9] {
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind("barbarian/extra-rage-power")),
            "at {level}"
        );
    }
    let keys = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a offers the same 38.
    assert_eq!(keys(&extra_slot(9, 0)), keys(&selection_slot(8)));
    // Deflect Arrows is taken at 4; Purity of Body opens at 5 (the slot at 5 is
    // `the_instinctual_warriors_monk_powers_need_the_levels_and_dex_their_blueprints_name`'s). Game: each row's
    // blueprint in MONK_RAGE_POWERS.
    for (slot, level) in [(extra_slot(5, 0), 5), (extra_slot(9, 0), 9)] {
        let open = open_at_slot(&build, &slot);
        assert!(!open.contains(MONK_RAGE_POWERS[0].0), "at {level}");
        for (key, name, needs) in &MONK_RAGE_POWERS[1..] {
            if *key == MONK_RAGE_POWERS[1].0 && level == 5 {
                continue;
            }
            assert_eq!(
                open.contains(*key),
                opens_at(*needs, level),
                "{name} at {level}"
            );
        }
    }
    let mut filled = build.clone();
    filled
        .picks
        .insert(extra_slot(9, 0), pick(MONK_RAGE_POWERS[1].0));
    let filled = with_selections_filled(filled);
    assert!(possible(&filled));

    // A plain barbarian's Extra Rage Power slot has them locked. Game: each blueprint in MONK_RAGE_POWERS needs
    // InstinctualWarriorArchetype adffdd8a99094a89823a79292a503ee9.
    let mut plain = with_selections_filled(barbarian(3));
    plain
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    let locked: BTreeSet<String> = engine
        .options(&plain, &extra_slot(3, 0))
        .unwrap()
        .into_iter()
        .filter(|option| option.status.is_err())
        .map(|option| option.pick.key().to_owned())
        .collect();
    for (key, name, _) in MONK_RAGE_POWERS {
        assert!(locked.contains(key), "{name}");
    }
}
