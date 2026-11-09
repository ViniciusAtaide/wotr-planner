//! Derived state at a character level: class levels, BAB, saves, ability scores (Destiny Beyond Birth removes a
//! race's penalty), skills, spell-level access.

use rules::{
    Ability, ClassChoice, ClassKey, Engine, LevelUp, Save, SaveException, SaveProgression,
    SkillPoints, SlotId, SubraceKey,
};

use crate::support::*;

#[test]
fn multiclass_bab_and_saves_add_up_the_class_tables() {
    let build = build(
        "human",
        &["fighter", "fighter", "rogue", "rogue", "rogue", "wizard"],
    );
    let sheet = engine().evaluate(&build);

    // Fighter 2 (+2, F3 R0 W0) + Rogue 1 (+0, F0 R2 W0).
    let at_3 = sheet.at(3).expect("level 3");
    assert_eq!((at_3.bab, at_3.fort, at_3.reflex, at_3.will), (2, 3, 2, 0));

    // Fighter 2 + Rogue 3 (+2, F1 R3 W1) + Wizard 1 (+0, F0 R0 W2).
    let at_6 = sheet.at(6).expect("level 6");
    assert_eq!((at_6.bab, at_6.fort, at_6.reflex, at_6.will), (4, 4, 3, 3));
    assert_eq!(at_6.class_level(&class("fighter")), 2);
    assert_eq!(at_6.class_level(&class("rogue")), 3);
    assert_eq!(at_6.class_level(&class("wizard")), 1);
    assert_eq!(at_6.character_level, 6);
}

#[test]
fn prestige_classes_use_the_prestige_save_formula() {
    let sheet = engine().evaluate(&arcane_trickster_build(vec![class("wizard")]));

    // Rogue 3 (+2, F1 R3 W1) + Wizard 3 (+1, F1 R1 W3) + Arcane Trickster 1 (½ BAB +0; prestige poor
    // ⌊(1+1)/3⌋ = 0, good ⌊(1+1)/2⌋ = 1).
    let at_7 = sheet.at(7).unwrap();
    assert_eq!((at_7.bab, at_7.fort, at_7.reflex, at_7.will), (3, 2, 5, 5));
    // Arcane Trickster 2: +1, F1 R1 W1.
    let at_8 = sheet.at(8).unwrap();
    assert_eq!((at_8.bab, at_8.fort, at_8.reflex, at_8.will), (4, 3, 5, 5));
}

#[test]
fn a_save_exception_replaces_the_formula_at_its_level_only() {
    let mut data = world();
    let wizard = data
        .classes
        .iter_mut()
        .find(|c| c.key == class("wizard"))
        .unwrap();
    wizard.save_exceptions.push(SaveException {
        level: 2,
        save: Save::Will,
        value: 9,
    });
    let sheet = Engine::new(data).evaluate(&build("elf", &times("wizard", 3)));

    let will: Vec<_> = (1..=3).map(|level| sheet.at(level).unwrap().will).collect();
    assert_eq!(will, [2, 9, 3]);
    assert_eq!(sheet.at(2).unwrap().fort, 0, "only the Will cell");
}

/// A Build of fighters taken as the armiger, whose save progressions the test sets.
fn armiger_build(levels: usize) -> rules::Build {
    let mut build = build("human", &times("fighter", levels));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("fighter/armiger")),
            advances: Default::default(),
        },
    );
    build
}

fn armiger_saves(
    data: &mut rules::GameData,
    fort: Option<SaveProgression>,
    will: Option<SaveProgression>,
) {
    let fighter = data
        .classes
        .iter_mut()
        .find(|c| c.key == class("fighter"))
        .unwrap();
    let armiger = &mut fighter.archetypes[0];
    armiger.fort = fort;
    armiger.will = will;
}

#[test]
fn an_archetype_save_progression_replaces_its_classs() {
    // Fighter: good Fortitude, poor Reflex and Will. This armiger: poor Fortitude and good Will, Reflex the
    // fighter's.
    let mut data = world();
    armiger_saves(
        &mut data,
        Some(SaveProgression::Poor),
        Some(SaveProgression::Good),
    );
    let engine = Engine::new(data);

    let saves = |build: &rules::Build, level| {
        let sheet = engine.evaluate(build);
        let at = sheet.at(level).unwrap();
        (at.fort, at.reflex, at.will)
    };
    // Armiger 1: F ⌊1/3⌋ = 0, R 0, W 2 + ⌊1/2⌋ = 2. Armiger 3: F 1, R 1, W 3.
    assert_eq!(saves(&armiger_build(3), 1), (0, 0, 2));
    assert_eq!(saves(&armiger_build(3), 3), (1, 1, 3));
    // A plain fighter keeps the fighter's: F 2 + ⌊3/2⌋ = 3, R 1, W 1.
    assert_eq!(saves(&build("human", &times("fighter", 3)), 3), (3, 1, 1));
}

#[test]
fn an_archetype_save_progression_replaces_the_class_exceptions_for_that_save_only() {
    let mut data = world();
    armiger_saves(&mut data, None, Some(SaveProgression::Good));
    let fighter = data
        .classes
        .iter_mut()
        .find(|c| c.key == class("fighter"))
        .unwrap();
    fighter.save_exceptions = vec![
        SaveException {
            level: 2,
            save: Save::Will,
            value: 9,
        },
        SaveException {
            level: 2,
            save: Save::Fortitude,
            value: 7,
        },
    ];
    let sheet = Engine::new(data).evaluate(&armiger_build(2));

    let at_2 = sheet.at(2).unwrap();
    // Will is the armiger's good progression (2 + ⌊2/2⌋ = 3), not the fighter's cell; Fortitude is still the
    // fighter's, cell included.
    assert_eq!((at_2.fort, at_2.will), (7, 3));
}

#[test]
fn ability_scores_add_racial_modifiers_and_the_increase_every_4_levels() {
    // Base Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8; elf: Dex +2, Int +2, Con -2.
    let mut build = build("elf", &times("fighter", 8));
    build.levels[3].ability_increase = Some(Ability::Strength);
    build.levels[7].ability_increase = Some(Ability::Strength);
    let sheet = engine().evaluate(&build);

    assert_eq!(
        sheet.at(1).unwrap().ability_scores,
        scores([16, 16, 10, 15, 10, 8])
    );
    assert_eq!(sheet.at(4).unwrap().ability_scores.strength, 17);
    assert_eq!(sheet.at(8).unwrap().ability_scores.strength, 18);
}

#[test]
fn a_human_puts_the_chosen_racial_bonus_where_the_build_says() {
    let mut build = build("human", &["fighter"]);
    build.start.racial_ability = Some(Ability::Strength);
    let sheet = engine().evaluate(&build);
    assert_eq!(sheet.at(1).unwrap().ability_scores.strength, 18);
}

#[test]
fn a_subrace_replaces_the_race_ability_modifiers() {
    // Blightborn: Dex +2, Wis +2, Con -2 instead of the elf's Int +2.
    let mut build = build("elf", &["fighter"]);
    build.start.subrace = Some(SubraceKey::new("elf/blightborn"));
    let sheet = engine().evaluate(&build);
    assert_eq!(
        sheet.at(1).unwrap().ability_scores,
        scores([16, 16, 10, 13, 12, 8])
    );
}

/// A Build start's racial modifier to each ability, in the order Str, Dex, Con, Int, Wis, Cha.
fn racial_modifiers(start: &rules::BuildStart) -> [i8; 6] {
    let engine = engine();
    rules::AbilityScores::ABILITIES.map(|ability| engine.racial_modifier(start, ability))
}

#[test]
fn the_racial_modifiers_are_the_races_or_its_subraces_with_the_chosen_bonus_where_the_start_puts_it()
 {
    let mut start = build("elf", &[]).start;
    assert_eq!(racial_modifiers(&start), [0, 2, -2, 2, 0, 0]);
    start.subrace = Some(SubraceKey::new("elf/blightborn"));
    assert_eq!(racial_modifiers(&start), [0, 2, -2, 0, 2, 0]);

    let mut start = build("human", &[]).start;
    assert_eq!(racial_modifiers(&start), [0; 6], "not placed yet");
    start.racial_ability = Some(Ability::Wisdom);
    assert_eq!(racial_modifiers(&start), [0, 0, 0, 0, 2, 0]);

    start.race = None;
    assert_eq!(racial_modifiers(&start), [0; 6]);
}

#[test]
fn a_race_or_subrace_with_a_chosen_bonus_lets_the_start_choose_its_ability() {
    let engine = engine();
    let mut start = build("human", &[]).start;
    assert!(engine.chooses_racial_ability(&start));
    start.race = Some(rules::RaceKey::new("half-elf"));
    start.subrace = Some(SubraceKey::new("half-elf/dual-heritage"));
    assert!(engine.chooses_racial_ability(&start));

    let mut start = build("elf", &[]).start;
    assert!(!engine.chooses_racial_ability(&start));
    start.subrace = Some(SubraceKey::new("elf/blightborn"));
    assert!(!engine.chooses_racial_ability(&start));
    start.race = None;
    start.subrace = None;
    assert!(!engine.chooses_racial_ability(&start));
}

#[test]
fn skill_ranks_accumulate_and_skill_points_come_from_class_and_intelligence() {
    // Int 13 (+1). Rogue: 8 + 1 = 9 points a level; fighter: 2 + 1 = 3.
    let mut build = build("dwarf", &["rogue", "fighter"]);
    build.levels[0].skill_ranks.insert(skill("mobility"), 1);
    build.levels[0].skill_ranks.insert(skill("athletics"), 1);
    build.levels[1].skill_ranks.insert(skill("mobility"), 1);
    let sheet = engine().evaluate(&build);

    let points = |available, spent| SkillPoints { available, spent };
    assert_eq!(sheet.levels[0].skill_points, points(9, 2));
    assert_eq!(sheet.levels[1].skill_points, points(3, 1));
    let at_2 = sheet.at(2).unwrap();
    assert_eq!(at_2.skill_ranks(&skill("mobility")), 2);
    assert_eq!(at_2.skill_ranks(&skill("athletics")), 1);
    assert_eq!(at_2.skill_ranks(&skill("persuasion")), 0);
}

#[test]
fn spell_level_access_comes_from_the_class_table() {
    let sheet = engine().evaluate(&build("elf", &times("wizard", 5)));
    assert_eq!(sheet.at(1).unwrap().arcane_spell_level, 1);
    assert_eq!(sheet.at(3).unwrap().arcane_spell_level, 2);
    assert_eq!(sheet.at(5).unwrap().arcane_spell_level, 3);
    assert_eq!(sheet.at(5).unwrap().divine_spell_level, 0);
    assert_eq!(sheet.at(5).unwrap().caster_level, 5);
}

/// Rogue 3 (Sneak Attack x2), Wizard 3 (2nd-level arcane), Mobility 4: ready for Arcane Trickster at level 7.
fn arcane_trickster_build(advances: Vec<ClassKey>) -> rules::Build {
    let mut build = build(
        "elf",
        &["rogue", "rogue", "rogue", "wizard", "wizard", "wizard"],
    );
    for level in &mut build.levels[..4] {
        level.skill_ranks.insert(skill("mobility"), 1);
    }
    for _ in 0..2 {
        build.levels.push(LevelUp::new(class("arcane-trickster")));
    }
    build.class_choices.insert(
        class("arcane-trickster"),
        ClassChoice {
            archetype: None,
            advances: advances.into_iter().collect(),
        },
    );
    build
}

#[test]
fn prestige_levels_advance_the_spell_level_access_of_the_class_chosen_on_entry() {
    let sheet = engine().evaluate(&arcane_trickster_build(vec![class("wizard")]));

    // Wizard 3 + Arcane Trickster 1 casts as a wizard 4, then as a wizard 5.
    assert_eq!(sheet.at(6).unwrap().arcane_spell_level, 2);
    assert_eq!(sheet.at(7).unwrap().arcane_spell_level, 2);
    assert_eq!(sheet.at(8).unwrap().arcane_spell_level, 3);
    assert_eq!(sheet.at(8).unwrap().caster_level, 5);
    assert_eq!(
        sheet
            .at(8)
            .unwrap()
            .feature_count(&feature("rogue/sneak-attack")),
        3
    );
}

#[test]
fn a_race_or_its_subrace_adds_skill_points_every_level() {
    // Int 13 (+1). A human fighter: 2 + 1 + the human's 1 = 4 points a level.
    let sheet = engine().evaluate(&build("human", &["fighter", "fighter"]));
    assert_eq!(sheet.levels[0].skill_points.available, 4);
    assert_eq!(sheet.levels[1].skill_points.available, 4);

    // A subrace's points replace the race's: the half-elf has none, its invented Adaptable subrace 1.
    let mut build = build("half-elf", &["fighter"]);
    assert_eq!(
        engine().evaluate(&build).levels[0].skill_points.available,
        3
    );
    build.start.subrace = Some(SubraceKey::new("half-elf/adaptable"));
    assert_eq!(
        engine().evaluate(&build).levels[0].skill_points.available,
        4
    );
}

#[test]
fn prestige_levels_advance_nothing_without_a_choice_on_entry() {
    let sheet = engine().evaluate(&arcane_trickster_build(vec![]));
    assert_eq!(sheet.at(8).unwrap().arcane_spell_level, 2);
}

// ---- Destiny Beyond Birth -------------------------------------------------------------------------------------

#[test]
fn destiny_beyond_birth_removes_the_races_ability_penalty() {
    let mut build = build("elf", &times("fighter", 7));
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        pick_mythic_feat("destiny-beyond-birth"),
    );
    assert!(engine().is_possible(&build));
    let sheet = engine().evaluate(&build);
    // Elf: Con 12 - 2. The bonuses stay (Dex and Int +2).
    assert_eq!(sheet.at(6).unwrap().ability_scores.constitution, 10);
    let at_7 = sheet.at(7).unwrap();
    assert_eq!(at_7.ability_scores.constitution, 12);
    assert_eq!(at_7.ability_scores.dexterity, 16);
    assert_eq!(at_7.base_ability_scores.constitution, 10);
}
