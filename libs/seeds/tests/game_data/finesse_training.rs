//! Finesse Training takes a weapon (ticket 172, the user's "make finesse training take a weapon", reversing ticket
//! 54's "choices that gate no pick stay labels" for this feature). The guide (`rogue#finesse-training`): "At 3rd
//! level, a Rogue may select a finessable weapon that she has proficiency in. … She can select a second weapon at
//! 11th level and a third at 19th level. The weapons selectable are: Bite, Claw, Dagger, Dueling Sword, Elven Curve
//! Blade, Estoc, Handaxe, Kama, Kukri, Light Hammer, Light Mace, Light Pick, Light Shield, Nunchaku, Punching Dagger,
//! Rapier, Sai, Shortsword, Sickle, Starknife, Unarmed." So it's a selection, `rogue/finesse-training`, at rogue 3,
//! 11 and 19, whose one option takes one of those weapons, once per weapon.

use std::collections::{BTreeMap, BTreeSet};

use rules::{Build, Parameter, Pick, SelectionOptionKey, SlotId, encode_build};

use crate::support::*;

const SELECTION: &str = "rogue/finesse-training";
const OPTION: &str = "rogue/finesse-training/weapon";

/// The guide's list, as the planner's weapons: its "Handaxe", "Light Shield", "Shortsword" and "Unarmed" are the Hand
/// Axe, the Light Shield Bash, the Short Sword and the Unarmed Strike.
const WEAPONS: [&str; 21] = [
    "bite",
    "claw",
    "dagger",
    "dueling-sword",
    "elven-curve-blade",
    "estoc",
    "hand-axe",
    "kama",
    "kukri",
    "light-hammer",
    "light-mace",
    "light-pick",
    "light-shield-bash",
    "nunchaku",
    "punching-dagger",
    "rapier",
    "sai",
    "short-sword",
    "sickle",
    "starknife",
    "unarmed-strike",
];

/// The levels the guide gives a choice at.
const LEVELS: [u8; 3] = [3, 11, 19];

fn trained(weapon: &str) -> Pick {
    Pick::option_for(SelectionOptionKey::new(OPTION), weapon_value(weapon))
}

fn weapon_value(key: &str) -> Parameter {
    crate::support::weapon(key)
}

/// The weapons the Finesse Training slot of `level` offers, each with whether it's open.
fn weapons_offered(build: &Build, level: u8) -> BTreeMap<String, bool> {
    engine()
        .options(build, &selection_slot(level))
        .unwrap_or_else(|| panic!("no selection slot at {level}"))
        .into_iter()
        .map(|option| {
            let Pick::SelectionOption {
                option: key,
                parameter: Some(Parameter::Weapon(weapon)),
            } = &option.pick
            else {
                panic!("not a weapon pick of Finesse Training: {:?}", option.pick)
            };
            assert_eq!(key.as_str(), OPTION);
            (weapon.as_str().to_owned(), option.status.is_ok())
        })
        .collect()
}

#[test]
fn finesse_trainings_weapons_are_the_guides_list() {
    let option = selection_option(OPTION);
    let values: Vec<Parameter> = option.values.iter().cloned().collect();
    assert_eq!(values, WEAPONS.map(weapon_value));
    assert_eq!(option.parameter, Some(rules::ParameterKind::Weapon));
    assert_eq!(option.repeat, rules::Repeat::PerValue);
    assert_eq!(option.name, "Finesse Training");
    // The Light Shield is a weapon: the shield's bash, finessable as the guide's weapon table has it.
    let shield = DATA
        .weapons
        .iter()
        .find(|weapon| weapon.key.as_str() == "light-shield-bash")
        .unwrap();
    assert_eq!(shield.name, "Light Shield Bash");
    assert!(shield.stats.is_some_and(|stats| stats.finesse));
    // No feature is left: the selection is the Finesse Training a level holds.
    assert!(
        DATA.class_features()
            .all(|feature| feature.key.as_str() != SELECTION)
    );
}

#[test]
fn a_rogue_chooses_a_weapon_at_3_11_and_19() {
    let build = build("rogue", 20);
    assert!(possible(&build));
    assert_eq!(levels_with_slot_of(&build, SELECTION), LEVELS);
    for level in LEVELS {
        let offered = weapons_offered(&build, level);
        assert_eq!(
            offered.keys().map(String::as_str).collect::<Vec<_>>(),
            WEAPONS,
            "{level}"
        );
        assert!(offered.values().all(|open| *open), "{level}");
        assert_eq!(
            engine().slot_label(
                engine()
                    .evaluate(&build)
                    .slot(&selection_slot(level))
                    .unwrap()
            ),
            "Finesse Training"
        );
        // The web app's picker: one entry, which opens the list of the 21 weapons.
        let entries = engine()
            .slot_entries(&build, &selection_slot(level))
            .unwrap();
        let [entry] = entries.as_slice() else {
            panic!("{} entries at {level}", entries.len())
        };
        assert_eq!(entry.name, "Finesse Training");
        assert_eq!(entry.noun, Some("a weapon"));
        assert_eq!(entry.options.len(), WEAPONS.len());
    }
    // Filled, with three weapons, the Build is possible and holds the option for each.
    let filled = with_picks(
        build,
        &[
            (3, trained("light-shield-bash")),
            (11, trained("dagger")),
            (19, trained("rapier")),
        ],
    );
    assert!(possible(&filled));
    let last = at(&filled, 20);
    assert_eq!(last.option_count(&SelectionOptionKey::new(OPTION)), 3);
    assert_eq!(
        last.option_values[&SelectionOptionKey::new(OPTION)]
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        ["dagger", "light-shield-bash", "rapier"]
            .map(weapon_value)
            .into_iter()
            .collect()
    );
    assert_eq!(
        engine().pick_name(&trained("light-shield-bash")),
        "Finesse Training (Light Shield Bash)"
    );
}

#[test]
fn a_weapon_trained_once_is_taken_and_a_weapon_off_the_list_is_refused() {
    let build = with_picks(build("rogue", 11), &[(3, trained("dagger"))]);
    assert!(possible(&build));
    let offered = weapons_offered(&build, 11);
    assert!(!offered["dagger"]);
    assert!(offered["rapier"]);
    let dagger = engine()
        .options(&build, &selection_slot(11))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == trained("dagger"))
        .unwrap();
    assert_eq!(
        engine().issue_text(&dagger.status.unwrap_err()).as_deref(),
        Some("Already taken")
    );
    let twice = with_picks(build.clone(), &[(11, trained("dagger"))]);
    assert!(!possible(&twice));
    assert_eq!(
        why_not(&twice, &selection_slot(11)),
        Some(rules::Issue::AlreadyHeld)
    );
    // The longsword isn't on the guide's list: not offered, and a Build naming it is impossible.
    let longsword = with_picks(build, &[(11, trained("longsword"))]);
    assert!(!possible(&longsword));
    assert_eq!(why_not(&longsword, &selection_slot(11)), None);
}

/// Every rogue archetype keeps Finesse Training at 3, 11 and 19 (the guide's tables), but the Rowdy, which "Loses
/// finesse training"; no other class has it.
#[test]
fn every_rogue_archetype_but_the_rowdy_chooses_at_3_11_and_19() {
    let mut kept = Vec::new();
    for archetype in &class("rogue").archetypes {
        let key = archetype.key.as_str();
        let mut build = archetype_build("rogue", key, 20);
        if key == "rogue/master-of-all" {
            build.start.race = Some(rules::RaceKey::new("half-elf"));
        }
        assert!(possible(&build), "{key}");
        let levels = levels_with_slot_of(&build, SELECTION);
        if key == "rogue/rowdy" {
            assert_eq!(levels, [], "{key}");
            continue;
        }
        assert_eq!(levels, LEVELS, "{key}");
        kept.push(key);
    }
    assert_eq!(
        kept,
        [
            "rogue/dark-lurker",
            "rogue/eldritch-scoundrel",
            "rogue/knife-master",
            "rogue/master-of-all",
            "rogue/sylvan-trickster",
            "rogue/thug",
            "rogue/underground-chemist",
        ]
    );
    // Only the rogue's rows name the selection.
    let naming: BTreeSet<&str> = DATA
        .classes
        .iter()
        .chain(&DATA.prestige_classes)
        .filter(|class| {
            let names = |selections: &[rules::FeatureSelectionKey]| {
                selections.iter().any(|s| s.as_str() == SELECTION)
            };
            class.levels.iter().any(|row| names(&row.selections))
                || class
                    .archetypes
                    .iter()
                    .flat_map(|a| &a.levels)
                    .any(|row| names(&row.selections))
        })
        .map(|class| class.key.as_str())
        .collect();
    assert_eq!(naming, BTreeSet::from(["rogue"]));
}

/// A rogue 20 as every link before ticket 172 held it: Finesse Training had no weapon, so its slots are empty. Its
/// link, written by the v1 layout this ticket leaves as it is (the same text on `main` at 9d89e35).
const OLDER_ROGUE_20: &str = "AaXEsQkAAAgDwdha60AO5N6OIcQFhBzPZ08p4TgFnoxf";

/// A link from before ticket 172 still opens: the Build is possible, and each Finesse Training weapon is a choice
/// still to make (an empty slot, open with every weapon). The bench's Assassin Build, a real older link, too: its
/// rogue 3 is at character level 5.
#[test]
fn an_older_link_opens_with_its_finesse_training_weapons_still_to_choose() {
    let engine = engine();
    let rogue = build("rogue", 20);
    assert_eq!(encode_build(&rogue), OLDER_ROGUE_20);
    let opened = engine
        .open_link(OLDER_ROGUE_20)
        .expect("the older link opens");
    assert_eq!(opened, rogue);
    assert!(possible(&opened));
    let sheet = engine.evaluate(&opened);
    for level in LEVELS {
        let slot = sheet.slot(&selection_slot(level)).unwrap();
        assert_eq!(slot.pick, None, "{level}");
        assert!(weapons_offered(&opened, level).values().all(|open| *open));
    }

    let (_, assassin) = bench()
        .into_iter()
        .find(|(name, _)| name == "multi_prestige20")
        .expect("the bench's Assassin Build");
    assert!(possible(&assassin));
    let at_5 = SlotId::Selection { level: 5, index: 0 };
    let slot = engine.evaluate(&assassin).slot(&at_5).cloned().unwrap();
    assert_eq!(slot.pick, None);
    assert_eq!(engine.slot_label(&slot), "Finesse Training");
    let filled = with_pick_value(assassin, at_5, trained("short-sword"));
    assert!(possible(&filled));
}

fn with_pick_value(mut build: Build, slot: SlotId, pick: Pick) -> Build {
    build.picks.insert(slot, pick);
    build
}

// ---- the review's tests: later picks, withdrawals and a multiclass rogue -----------------------------------------

/// The status `slot`'s picker gives Finesse Training for `weapon`.
fn status_of(build: &Build, slot: &SlotId, weapon: &str) -> Result<(), rules::Issue> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .find(|option| option.pick == trained(weapon))
        .unwrap_or_else(|| panic!("{weapon} isn't offered in {slot:?}"))
        .status
}

/// A weapon picked at a later level blocks it in an earlier slot (the later pick would break), and one picked at an
/// earlier level is already held later; a weapon twice in one Build is impossible.
#[test]
fn a_weapon_picked_later_breaks_it_earlier_and_one_picked_earlier_is_held_later() {
    use rules::Issue::{AlreadyHeld, BreaksLater};
    let both = with_picks(
        build("rogue", 20),
        &[(3, trained("dagger")), (11, trained("rapier"))],
    );
    assert!(possible(&both));
    let [at_3, at_11, at_19] = LEVELS.map(selection_slot);
    assert_eq!(status_of(&both, &at_3, "rapier"), Err(BreaksLater));
    assert_eq!(status_of(&both, &at_3, "dagger"), Ok(()));
    assert_eq!(status_of(&both, &at_11, "dagger"), Err(AlreadyHeld));
    assert_eq!(status_of(&both, &at_11, "rapier"), Ok(()));
    assert_eq!(status_of(&both, &at_19, "dagger"), Err(AlreadyHeld));
    assert_eq!(status_of(&both, &at_19, "rapier"), Err(AlreadyHeld));
    assert_eq!(status_of(&both, &at_19, "kukri"), Ok(()));

    let only_11 = with_picks(build("rogue", 20), &[(11, trained("dagger"))]);
    assert!(possible(&only_11));
    assert_eq!(status_of(&only_11, &at_3, "dagger"), Err(BreaksLater));
    assert_eq!(status_of(&only_11, &at_3, "rapier"), Ok(()));

    let again_at_19 = with_picks(
        build("rogue", 20),
        &[
            (3, trained("dagger")),
            (11, trained("rapier")),
            (19, trained("dagger")),
        ],
    );
    assert!(!possible(&again_at_19));
    assert_eq!(why_not(&again_at_19, &at_19), Some(AlreadyHeld));
}

/// Each Finesse Training pick comes off on its own; the Build stays possible, the weapon is no longer held, and it's
/// open again in the other slot.
#[test]
fn a_finesse_training_pick_withdraws_and_frees_its_weapon() {
    let engine = engine();
    let both = with_picks(
        build("rogue", 11),
        &[(3, trained("dagger")), (11, trained("rapier"))],
    );
    assert!(possible(&both));
    let option = SelectionOptionKey::new(OPTION);
    for (level, weapon, other) in [(3, "dagger", 11), (11, "rapier", 3)] {
        let choice = rules::Choice::Slot(selection_slot(level));
        assert!(engine.is_removable(&both, &choice), "{level}");
        let (edit, removable) = engine.withdrawal(&both, &choice).expect("the pick is held");
        assert!(removable, "{level}");
        let mut fewer = both.clone();
        engine.apply(&mut fewer, &edit);
        assert_eq!(fewer.picks.get(&selection_slot(level)), None, "{level}");
        assert!(possible(&fewer), "{level}");
        assert!(
            !at(&fewer, 11).has_option_for(&option, &weapon_value(weapon)),
            "{level}"
        );
        assert_eq!(
            status_of(&fewer, &selection_slot(other), weapon),
            Ok(()),
            "{level}"
        );
    }
}

/// A rogue 3 / fighter 2 / rogue 9 has its Finesse Training slots at character levels 3 and 13 (rogue 3 and 11), and
/// takes a weapon once across both.
#[test]
fn a_multiclass_rogue_chooses_at_its_rogue_levels_and_once_per_weapon() {
    let build = multiclass(&[("rogue", None, 3), ("fighter", None, 2), ("rogue", None, 9)]);
    assert!(possible(&build));
    assert_eq!(levels_with_slot_of(&build, SELECTION), [3, 13]);
    let with_dagger = with_picks(build, &[(3, trained("dagger"))]);
    assert!(possible(&with_dagger));
    assert_eq!(
        status_of(&with_dagger, &selection_slot(13), "dagger"),
        Err(rules::Issue::AlreadyHeld)
    );
    let twice = with_picks(with_dagger, &[(13, trained("dagger"))]);
    assert!(!possible(&twice));
    assert_eq!(
        why_not(&twice, &selection_slot(13)),
        Some(rules::Issue::AlreadyHeld)
    );
}
