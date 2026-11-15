//! Each comparison finds the difference planted for it in the fixture, and lets the entries that agree through.

use crate::support::{assert_agrees, assert_finding, report};

#[test]
fn mapping_finds_a_missing_blueprint_a_wrong_type_a_prestige_flag_and_a_stray_archetype() {
    let report = report();
    assert_finding(
        &report,
        "mapping / not in the dump",
        "feat lost-feat",
        "-",
        "feat",
    );
    assert_finding(
        &report,
        "mapping / a type its kind can't be",
        "feat misnamed",
        "BlueprintCharacterClass",
        "feat",
    );
    assert_finding(
        &report,
        "mapping / prestige or not",
        "class sentinel",
        "base class",
        "prestige class",
    );
    assert_finding(
        &report,
        "mapping / not among its class's archetypes",
        "archetype warden/stray",
        "not in WardenClass's m_Archetypes",
        "an archetype of warden",
    );
    assert_agrees(&report, "mapping", "feat steady-hand");
    assert_agrees(&report, "mapping", "archetype warden/stoic");
}

#[test]
fn prerequisites_find_a_value_a_missing_leaf_an_extra_one_a_grouping_and_an_unmapped_blueprint() {
    let report = report();
    assert_finding(
        &report,
        "prerequisites / value differs",
        "feat steady-hand",
        "Strength 13",
        "Strength 15",
    );
    assert_finding(
        &report,
        "prerequisites / missing in planner",
        "feat steady-hand",
        "BAB 1",
        "-",
    );
    assert_finding(
        &report,
        "prerequisites / not in game",
        "feat old-habit",
        "-",
        "KeenEyeFeat",
    );
    assert_finding(
        &report,
        "prerequisites / grouping differs",
        "feat wide-stance",
        "all of (SteadyHandFeat; KeenEyeFeat)",
        "one of (SteadyHandFeat; KeenEyeFeat)",
    );
    assert_finding(
        &report,
        "prerequisites / missing in planner, a blueprint it doesn't map",
        "feat stranger",
        "UnmappedThing",
        "-",
    );
    // A feat, an alignment mask against an `Any` of alignments, a class level, a class's own alignment.
    for entry in [
        "feat keen-eye",
        "feat faithful",
        "selection option warden/trick/a",
        "class warden",
    ] {
        assert_agrees(&report, "prerequisites", entry);
    }
}

#[test]
fn class_tables_find_skill_points_a_save_and_class_skills_for_a_class_and_an_archetype() {
    let report = report();
    let user = "(the user's facts)";
    assert_finding(
        &report,
        &format!("class tables / skill points differ {user}"),
        "class warden",
        "3",
        "4",
    );
    assert_finding(
        &report,
        "class tables / Reflex differs",
        "class warden",
        "+2 at level 1",
        "+0 at level 1; 3 of 3 levels differ",
    );
    assert_finding(
        &report,
        &format!("class tables / class skills differ {user}"),
        "class warden",
        "SkillAthletics, SkillStealth",
        "SkillAthletics, SkillPerception",
    );
    assert_finding(
        &report,
        &format!("class tables / skill points differ {user}"),
        "archetype warden/stoic",
        "4",
        "5",
    );
    assert_finding(
        &report,
        "class tables / Will differs",
        "archetype warden/stoic",
        "+0 at level 1",
        "+2 at level 1; 3 of 3 levels differ",
    );
    // The prestige formulas against the game's prestige tables.
    assert_agrees(&report, "class tables", "class sentinel");
}

#[test]
fn progression_rows_find_a_grant_at_another_level_in_one_side_only_and_an_archetype_s_removal() {
    let report = report();
    assert_finding(
        &report,
        "progression rows / at another level",
        "class warden",
        "WardenSenseFeature at level 3",
        "warden/sense at level 2",
    );
    assert_finding(
        &report,
        "progression rows / in game only",
        "class warden",
        "WardenExtraFeature at level 3",
        "-",
    );
    assert_finding(
        &report,
        "progression rows / in planner only",
        "class warden",
        "-",
        "warden/ghost at level 3",
    );
    assert_finding(
        &report,
        "progression rows / in planner only",
        "archetype warden/stoic",
        "-",
        "warden/trick at level 2",
    );
    // The bundle of Steady Hand, the hidden feature and the combat feat selection against the slot all pass.
    for name in [
        "SteadyHand",
        "WardenDrillsBundle",
        "WardenHiddenFeature",
        "WardenCombatSelection",
    ] {
        assert!(
            !report.text().contains(name) || name == "SteadyHand",
            "{name} reported"
        );
    }
    assert!(!report.text().contains("steady-hand at level"));
    assert_agrees(&report, "progression rows", "class sentinel");
}

#[test]
fn option_lists_find_options_on_one_side_a_list_s_flag_a_choice_the_game_lacks_and_the_pools() {
    let report = report();
    let game_only = "option lists / in game only";
    let planner_only = "option lists / in planner only";
    assert_finding(
        &report,
        game_only,
        "selection warden/trick",
        "WardenTrickCFeature",
        "-",
    );
    assert_finding(
        &report,
        planner_only,
        "selection warden/trick",
        "-",
        "warden/trick/b (WardenTrickBFeature)",
    );
    assert_finding(
        &report,
        game_only,
        "feat list warden/drills",
        "WideStanceFeat",
        "-",
    );
    assert_finding(
        &report,
        planner_only,
        "feat list warden/drills",
        "-",
        "keen-eye (KeenEyeFeat)",
    );
    assert_finding(
        &report,
        "option lists / ignores prerequisites or not",
        "feat list warden/drills",
        "false",
        "true",
    );
    assert_finding(
        &report,
        "option lists / a choice the game doesn't offer",
        "selection warden/knack",
        "BlueprintFeature",
        "options with a blueprint: 1",
    );
    assert_finding(
        &report,
        game_only,
        "pool general feats",
        "UnpickedFeat",
        "-",
    );
    assert_finding(
        &report,
        planner_only,
        "pool general feats",
        "-",
        "old-habit (OldHabitFeat)",
    );
    assert_finding(
        &report,
        planner_only,
        "pool mythic abilities",
        "-",
        "mythic-spare (MythicSpareFeature)",
    );
    assert_finding(
        &report,
        game_only,
        "pool mythic feats",
        "UnclaimedMythicFeat",
        "-",
    );
}

#[test]
fn mythic_ranks_find_a_grant_at_another_rank_after_the_starting_ranks_and_a_nested_progression() {
    let report = report();
    assert_finding(
        &report,
        "mythic ranks / at another level",
        "mythic path seraph",
        "SeraphWingsFeature at rank 4",
        "seraph/wings at rank 5",
    );
    // The first step (the path's first levels), the halo (path level 1 at rank 3) and the summon (a nested
    // progression's level 5) agree: the wings are the path's only finding.
    let findings = report
        .text()
        .lines()
        .filter(|line| line.starts_with("  mythic path seraph |"))
        .count();
    assert_eq!(findings, 1, "{}", report.text());
}

#[test]
fn items_find_an_amount_a_type_a_slot_and_a_bonus_on_each_side() {
    let report = report();
    assert_finding(
        &report,
        "items / amount differs",
        "gear item cloak-of-test",
        "AllSaves +3 Resistance",
        "AllSaves +2 Resistance",
    );
    assert_finding(
        &report,
        "items / slot differs",
        "gear item headband-of-test",
        "BlueprintItemEquipmentHead",
        "Belt",
    );
    assert_finding(
        &report,
        "items / type differs",
        "gear item headband-of-test",
        "SkillPerception +1 Insight",
        "SkillPerception +1 Competence",
    );
    assert_finding(
        &report,
        "items / bonus missing in planner",
        "gear item headband-of-test",
        "Initiative +1 Morale",
        "-",
    );
    assert_finding(
        &report,
        "items / bonus not in game",
        "gear item ring-of-nothing",
        "-",
        "Concentration +1 Untyped",
    );
    assert_agrees(&report, "items", "gear item belt-of-test-strength");
    assert!(report.text().contains("1 a manual or tome (deferred)"));
    assert!(
        report
            .text()
            .contains("1 a bonus to AC, which the planner doesn't count")
    );
}
