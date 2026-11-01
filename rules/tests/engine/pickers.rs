//! The sheet's other pickers: class and archetype at a level, the path at MR 3 and the Path switch at MR 8.

use rules::{
    ArchetypeKey, ClassChoice, ClassOption, Issue, MythicPathKey, PathOption, Prerequisite, Unmet,
};

use crate::support::*;

fn option_for<'a>(
    options: &'a [ClassOption],
    class_key: &str,
    archetype: Option<&str>,
) -> &'a ClassOption {
    options
        .iter()
        .find(|o| {
            o.class == class(class_key)
                && o.archetype.as_ref().map(ArchetypeKey::as_str) == archetype
        })
        .unwrap_or_else(|| panic!("no option {class_key} {archetype:?}"))
}

#[test]
fn class_options_list_every_class_with_its_archetypes_then_prestige_classes() {
    let options = engine().class_options(&build("elf", &[]), 1).unwrap();
    let listed: Vec<(&str, Option<&str>)> = options
        .iter()
        .map(|o| {
            (
                o.class.as_str(),
                o.archetype.as_ref().map(ArchetypeKey::as_str),
            )
        })
        .collect();
    assert_eq!(
        listed,
        [
            ("cleric", None),
            ("fighter", None),
            ("fighter", Some("fighter/armiger")),
            ("monk", None),
            ("rogue", None),
            ("rogue", Some("rogue/eldritch-scoundrel")),
            ("slayer", None),
            ("witch", None),
            ("wizard", None),
            ("arcane-trickster", None),
            ("eldritch-knight", None),
            ("mystic-theurge", None),
            ("winter-witch", None),
        ]
    );
}

#[test]
fn a_class_option_is_locked_by_its_entry_prerequisites_before_that_level() {
    let mut dwarf = build("dwarf", &[]);
    dwarf.start.race = Some(rules::RaceKey::new("dwarf"));
    let options = engine().class_options(&dwarf, 1).unwrap();

    assert_eq!(option_for(&options, "rogue", None).status, Ok(()));
    // Eldritch Scoundrel is for humans and elves.
    let Err(Issue::Unmet(unmet)) =
        &option_for(&options, "rogue", Some("rogue/eldritch-scoundrel")).status
    else {
        panic!("the scoundrel should be locked");
    };
    assert_eq!(unmet.conditions().len(), 2);
    // Arcane Trickster: no Mobility, no spells, no Sneak Attack.
    let Err(Issue::Unmet(unmet)) = &option_for(&options, "arcane-trickster", None).status else {
        panic!("Arcane Trickster should be locked");
    };
    assert_eq!(unmet.conditions().len(), 3);
}

#[test]
fn class_options_at_a_later_level_see_the_build_up_to_it() {
    // Rogue 3 / Wizard 3 with Mobility 4: Arcane Trickster opens at level 7, not at level 6.
    let mut build = build(
        "elf",
        &["rogue", "rogue", "rogue", "wizard", "wizard", "wizard"],
    );
    for level in &mut build.levels[..4] {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    let at = |level| {
        option_for(
            &engine().class_options(&build, level).unwrap(),
            "arcane-trickster",
            None,
        )
        .status
        .clone()
    };
    assert_eq!(at(7), Ok(()));
    let Err(Issue::Unmet(Unmet::Condition { needs, has })) = at(6) else {
        panic!("one condition should fail at level 6");
    };
    assert_eq!(
        needs,
        Prerequisite::SpellLevel {
            spell_type: rules::SpellType::Arcane,
            level: 2
        }
    );
    assert_eq!(has, Some(1));
    // Past the next level there is nothing to offer.
    assert_eq!(engine().class_options(&build, 8), None);
    assert_eq!(engine().class_options(&build, 0), None);
}

#[test]
fn a_class_already_taken_is_offered_only_with_its_archetype() {
    let mut build = build("elf", &times("fighter", 2));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    let options = engine().class_options(&build, 2).unwrap();
    let fighters: Vec<_> = options
        .iter()
        .filter(|o| o.class == class("fighter"))
        .map(|o| o.archetype.clone())
        .collect();
    assert_eq!(fighters, [Some(ArchetypeKey::new("fighter/armiger"))]);
}

#[test]
fn a_class_at_the_end_of_its_table_is_locked() {
    let mut capped = build("elf", &times("fighter", 20));
    capped.path_switch = Some(path("legend"));
    let options = engine().class_options(&capped, 21).unwrap();
    assert_eq!(
        option_for(&options, "fighter", None).status,
        Err(Issue::ClassLevelCap { cap: 20 })
    );
    assert_eq!(option_for(&options, "rogue", None).status, Ok(()));
}

#[test]
fn the_path_at_mr_3_is_any_path_without_a_switch_rule() {
    let options = engine().path_options(&build("elf", &[]));
    assert_eq!(
        options,
        [
            PathOption {
                path: path("aeon"),
                status: Ok(())
            },
            PathOption {
                path: path("trickster"),
                status: Ok(())
            }
        ]
    );
}

#[test]
fn the_switch_at_mr_8_is_any_path_with_a_switch_rule() {
    let options: Vec<MythicPathKey> = engine()
        .switch_options(&build("elf", &[]))
        .into_iter()
        .map(|o| o.path)
        .collect();
    assert_eq!(
        options,
        [
            path("devil"),
            path("gold-dragon"),
            path("legend"),
            path("swarm-that-walks")
        ]
    );
}
