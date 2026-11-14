//! `validate` (ticket 133): a whole Build declared as a map of address → value, checked at once, with every mistake
//! back as a tree of causes; on the test world. That the entries' order doesn't matter is `validate/declare.rs`'s unit
//! test: an object's keys reach the server as a map, so no tool call can send them in two orders.

use rmcp::serde_json::{Map, Value, json};

use crate::support::*;

/// `validate` on the test world: the result for these declared choices.
async fn validate(choices: Value) -> Value {
    Client::on_the_test_world()
        .await
        .ok("validate", json!({ "choices": choices }))
        .await
}

/// A human fighter's Build start and levels, with `str` for Strength (the racial bonus left open).
fn fighter(levels: u8, strength: u8) -> Map<String, Value> {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
        ("start.score.str", json!(strength)),
        ("start.score.dex", json!(14)),
        ("start.score.con", json!(14)),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=levels {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "fighter" }),
        );
    }
    choices
}

fn with(mut choices: Map<String, Value>, more: &[(&str, Value)]) -> Value {
    for (address, value) in more {
        choices.insert((*address).to_owned(), value.clone());
    }
    Value::Object(choices)
}

/// The failures' `choice` addresses, roots only.
fn roots(result: &Value) -> Vec<String> {
    choices(&result["failures"])
}

/// The failure at an address among a list's (not its children's).
fn failure<'a>(list: &'a Value, choice: &str) -> &'a Value {
    at(list, choice)
}

#[tokio::test]
async fn three_independent_mistakes_come_back_as_three_roots() {
    let result = validate(with(
        fighter(3, 12),
        &[
            // Strength 12 for Power Attack (13).
            ("slot.feat.1.class.0", json!("power-attack")),
            // A background the data lacks.
            ("start.background", json!("nobel")),
            // Great Cleave needs Cleave, which the Build doesn't take, and BAB 4.
            ("slot.feat.3.general.0", json!("great-cleave")),
        ],
    ))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    assert_eq!(result["count"], 3, "{result}");
    assert!(result.get("build").is_none(), "{result}");
    assert_eq!(
        roots(&result),
        [
            "start.background",
            "slot.feat.1.class.0",
            "slot.feat.3.general.0"
        ]
    );
    let failures = &result["failures"];
    let background = failure(failures, "start.background");
    assert_eq!(background["issue"], "unknown_key");
    assert_eq!(background["label"], "Background");
    let power_attack = failure(failures, "slot.feat.1.class.0");
    assert_eq!(power_attack["issue"], "unmet");
    assert_eq!(power_attack["needs"], json!(["Str 13 (have 12)"]));
    assert_eq!(power_attack["children"], json!([]));
    let great_cleave = failure(failures, "slot.feat.3.general.0");
    assert_eq!(great_cleave["issue"], "unmet");
    assert_eq!(great_cleave["needs"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn a_refused_pick_holds_the_later_picks_that_need_it() {
    // Power Attack refused for Strength 12 at level 1; Cleave at level 3 needs it, and Great Cleave at level 5 Cleave.
    let result = validate(with(
        fighter(5, 12),
        &[
            ("slot.feat.1.class.0", json!("power-attack")),
            ("slot.feat.3.general.0", json!("cleave")),
            ("slot.feat.5.general.0", json!("great-cleave")),
        ],
    ))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    assert_eq!(result["count"], 3);
    assert_eq!(roots(&result), ["slot.feat.1.class.0"]);
    let power_attack = failure(&result["failures"], "slot.feat.1.class.0");
    assert_eq!(power_attack["reason"], "Needs Str 13 (have 12)");
    assert_eq!(
        choices(&power_attack["children"]),
        ["slot.feat.3.general.0"]
    );
    let cleave = failure(&power_attack["children"], "slot.feat.3.general.0");
    assert_eq!(cleave["issue"], "unmet");
    assert_eq!(cleave["label"], "Level 3: Feat");
    assert_eq!(cleave["needs"], json!(["power-attack"]));
    assert_eq!(cleave["also_needs"], json!([]));
    assert_eq!(choices(&cleave["children"]), ["slot.feat.5.general.0"]);
}

#[tokio::test]
async fn a_pick_refused_while_reading_holds_the_picks_that_need_it() {
    // Weapon Focus without its weapon: Dazzling Display and Aldori Dueling Mastery need it.
    let result = validate(with(
        fighter(3, 16),
        &[
            ("slot.feat.1.class.0", json!("weapon-focus")),
            ("slot.feat.1.general.0", json!("dazzling-display")),
            ("slot.feat.3.general.0", json!("aldori-dueling-mastery")),
        ],
    ))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    assert_eq!(roots(&result), ["slot.feat.1.class.0"], "{result}");
    let weapon_focus = failure(&result["failures"], "slot.feat.1.class.0");
    assert_eq!(weapon_focus["issue"], "needs_parameter");
    // A refusal on a slot is labelled by the sheet's slot, as the walk's failures are.
    assert_eq!(weapon_focus["label"], "Level 1: Combat feat");
    assert_eq!(
        choices(&weapon_focus["children"]),
        ["slot.feat.1.general.0", "slot.feat.3.general.0"]
    );
    let aldori = failure(&weapon_focus["children"], "slot.feat.3.general.0");
    assert_eq!(aldori["label"], "Level 3: Feat");

    // The same with a weapon that isn't one, and with the slot declared twice.
    let result = validate(with(
        fighter(1, 16),
        &[
            (
                "slot.feat.1.class.0",
                json!({ "key": "weapon-focus", "parameter": "longswrod" }),
            ),
            ("slot.feat.1.general.0", json!("dazzling-display")),
        ],
    ))
    .await;
    let weapon_focus = failure(&result["failures"], "slot.feat.1.class.0");
    assert_eq!(weapon_focus["issue"], "not_an_option");
    assert!(
        text(&weapon_focus["reason"]).contains("`longsword`"),
        "{weapon_focus}"
    );
    assert_eq!(
        choices(&weapon_focus["children"]),
        ["slot.feat.1.general.0"]
    );
    let result = validate(with(
        fighter(1, 16),
        &[
            ("slot.feat.1.class.0", json!("weapon-focus")),
            (" slot.feat.1.class.0", json!("power-attack")),
            ("slot.feat.1.general.0", json!("dazzling-display")),
        ],
    ))
    .await;
    let twice = failure(&result["failures"], "slot.feat.1.class.0");
    assert_eq!(twice["issue"], "declared_twice", "{result}");
    assert_eq!(choices(&twice["children"]), ["slot.feat.1.general.0"]);
}

#[tokio::test]
async fn a_key_the_data_lacks_is_unknown_and_a_known_one_out_of_place_is_not_allowed() {
    // The engine says which: Scribe Scrolls is a feat the fighter's class slot doesn't take, `power-atack` none.
    let result = validate(with(
        fighter(1, 16),
        &[
            ("slot.feat.1.class.0", json!("scribe-scrolls")),
            ("slot.feat.1.general.0", json!("power-atack")),
        ],
    ))
    .await;
    let failures = &result["failures"];
    let out_of_place = failure(failures, "slot.feat.1.class.0");
    assert_eq!(out_of_place["issue"], "not_allowed", "{result}");
    assert!(
        text(&out_of_place["reason"]).contains("isn't a pick this slot takes"),
        "{out_of_place}"
    );
    let unknown = failure(failures, "slot.feat.1.general.0");
    assert_eq!(unknown["issue"], "unknown_key");
    assert_eq!(
        unknown["reason"],
        "`power-atack` isn't a feat in the planner's data. Closest: power-attack (`power-attack`)."
    );
}

#[tokio::test]
async fn an_empty_faith_is_no_faith() {
    let result = validate(with(fighter(1, 16), &[("start.faith", json!(" "))])).await;
    assert!(
        roots(&result).contains(&"start.faith".to_owned()),
        "{result}"
    );
    let faith = failure(&result["failures"], "start.faith");
    assert_eq!(faith["issue"], "not_an_option");
    assert_eq!(faith["label"], "Faith");
}

#[tokio::test]
async fn a_pick_in_a_slot_the_level_doesnt_give_is_not_reached() {
    // Under the refused class at its level: a class the data lacks gives no class slot.
    let result = validate(with(
        fighter(2, 16),
        &[
            ("level.2.class", json!({ "class": "figter" })),
            ("slot.feat.2.class.0", json!("cleave")),
        ],
    ))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    assert_eq!(roots(&result), ["level.2.class"]);
    let class = failure(&result["failures"], "level.2.class");
    assert_eq!(class["issue"], "unknown_key");
    assert!(
        text(&class["reason"]).contains("Fighter (`fighter`)"),
        "{class}"
    );
    let slot = failure(&class["children"], "slot.feat.2.class.0");
    assert_eq!(slot["issue"], "not_reached");

    // On its own, with the slots that level does have: a fighter's level 2 has no general feat slot.
    let result = validate(with(
        fighter(2, 16),
        &[("slot.feat.2.general.0", json!("power-attack"))],
    ))
    .await;
    assert_eq!(roots(&result), ["slot.feat.2.general.0"]);
    let slot = failure(&result["failures"], "slot.feat.2.general.0");
    assert_eq!(slot["issue"], "not_reached");
    let reason = text(&slot["reason"]);
    assert!(
        reason.starts_with("Level 2 (Fighter 2) has no such slot. Its slots: "),
        "{reason}"
    );
    assert!(reason.contains("slot.feat.2.class.0"), "{reason}");
}

#[tokio::test]
async fn a_fixed_rule_break_is_a_root_and_the_walk_still_runs() {
    // A skill above the character level, and an unknown background the walk refuses.
    let result = validate(with(
        fighter(1, 16),
        &[
            ("level.1.skill.athletics", json!(2)),
            ("start.background", json!("nobel")),
        ],
    ))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    assert_eq!(
        roots(&result),
        ["level.1.skill.athletics", "start.background"]
    );
    let skill = failure(&result["failures"], "level.1.skill.athletics");
    assert_eq!(skill["issue"], "fixed_rule");
    assert!(
        text(&skill["reason"]).contains("never go above the character level"),
        "{skill}"
    );

    // A point buy over 25: the walk checks the rest with every score 10, so Power Attack's Strength sits under it.
    let result = validate(with(
        fighter(1, 18),
        &[
            ("start.score.int", json!(18)),
            ("slot.feat.1.class.0", json!("power-attack")),
            ("start.background", json!("nobel")),
        ],
    ))
    .await;
    assert_eq!(
        roots(&result),
        ["start.score.str", "start.background"],
        "{result}"
    );
    let point_buy = failure(&result["failures"], "start.score.str");
    assert_eq!(point_buy["issue"], "fixed_rule");
    assert_eq!(point_buy["label"], "Point buy");
    assert_eq!(
        choices(&point_buy["children"]),
        ["slot.feat.1.class.0"],
        "{point_buy}"
    );
}

#[tokio::test]
async fn a_valid_declared_build_returns_the_code_choose_makes() {
    let declared = with(
        fighter(3, 16),
        &[
            ("start.racial-ability", json!("str")),
            ("start.score.int", json!(13)),
            ("start.background", json!("none")),
            ("level.1.skill", json!({ "athletics": 1 })),
            ("level.2.skill.athletics", json!(1)),
            ("slot.feat.1.class.0", json!("power-attack")),
            ("slot.feat.1.race.0", json!("cleave")),
            ("slot.feat.3.general.0", json!("combat-expertise")),
        ],
    );
    let result = validate(declared).await;
    assert_eq!(result["status"], "valid", "{result}");
    assert!(result.get("failures").is_none(), "{result}");
    assert!(text(&result["link"]).ends_with(&text(&result["build"])));
    assert!(result["todo"].is_array());

    let client = Client::on_the_test_world().await;
    let start = client.ok("new_build", json!({})).await;
    let steps = json!([
        { "choice": "start.race", "value": "human" },
        { "choice": "start.racial-ability", "value": "str" },
        { "choice": "start.faith", "value": "atheism" },
        { "choice": "start.alignment", "value": "true-neutral" },
        { "choice": "start.background", "value": "none" },
        { "choice": "start.score.str", "value": 16 },
        { "choice": "start.score.dex", "value": 14 },
        { "choice": "start.score.con", "value": 14 },
        { "choice": "start.score.int", "value": 13 },
        { "choice": "level.1.class", "value": { "class": "fighter" } },
        { "choice": "level.1.skill", "value": { "athletics": 1 } },
        { "choice": "slot.feat.1.class.0", "value": "power-attack" },
        { "choice": "slot.feat.1.race.0", "value": "cleave" },
        { "choice": "level.2.class", "value": { "class": "fighter" } },
        { "choice": "level.2.skill.athletics", "value": 1 },
        { "choice": "level.3.class", "value": { "class": "fighter" } },
        { "choice": "slot.feat.3.general.0", "value": "combat-expertise" },
    ]);
    let chosen = client
        .ok("choose", json!({ "build": start["build"], "steps": steps }))
        .await;
    assert!(chosen.get("rejected").is_none(), "{chosen}");
    assert_eq!(result["build"], chosen["build"]);
    assert_eq!(result["todo"], chosen["todo"]);
}

#[tokio::test]
async fn a_forged_declared_build_never_returns_a_code() {
    // What no picker offers, and the declaration refuses: a racial bonus for a race without one, an ability increase
    // at level 3, an archetype the class doesn't have, a level missing below a declared one, `level.N` (which only
    // `withdraw` takes) and an address that isn't one.
    let result = validate(json!({
        "start.race": "dwarf",
        "start.racial-ability": "str",
        "start.faith": "atheism",
        "start.alignment": "true-neutral",
        "level.1.class": { "class": "fighter" },
        "level.2.class": { "class": "fighter" },
        "level.3.class": { "class": "fighter" },
        "level.3.increase": "str",
        "class.fighter.archetype": "fighter/one",
        "level.1.class.archetype": "fighter/one",
        "level.5.class": { "class": "fighter" },
        "slot.feat.5.general.0": "power-attack",
        "level.4": { "class": "fighter" },
    }))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    assert!(result.get("build").is_none(), "{result}");
    assert!(result.get("link").is_none(), "{result}");
    let failures = &result["failures"];
    for (choice, issue) in [
        ("start.racial-ability", "not_an_option"),
        ("level.3.increase", "fixed_rule"),
        ("level.1.class.archetype", "no_such_choice"),
        ("level.4", "no_such_choice"),
        ("level.4.class", "missing_level"),
        ("class.fighter.archetype", "unknown_key"),
    ] {
        assert_eq!(
            failure(failures, choice)["issue"],
            issue,
            "{choice}: {result}"
        );
    }
    // Level 5 is past the missing level 4, and its slot under it.
    let missing = failure(failures, "level.4.class");
    let level_5 = failure(&missing["children"], "level.5.class");
    assert_eq!(level_5["issue"], "not_reached");
    assert_eq!(
        choices(&level_5["children"]),
        ["slot.feat.5.general.0"],
        "{missing}"
    );
}

#[tokio::test]
async fn two_archetypes_for_one_class_are_declared_twice() {
    let result = validate(with(
        fighter(2, 16),
        &[
            (
                "level.1.class",
                json!({ "class": "fighter", "archetype": "a" }),
            ),
            (
                "level.2.class",
                json!({ "class": "fighter", "archetype": "b" }),
            ),
        ],
    ))
    .await;
    assert_eq!(roots(&result), ["class.fighter.archetype"], "{result}");
    let twice = failure(&result["failures"], "class.fighter.archetype");
    assert_eq!(twice["issue"], "declared_twice");
}

/// Gear the user asked for is declared at its place (ticket 168): a belt is worn, and a usable item at its own
/// address is used, so the Build's code holds them.
#[tokio::test]
async fn declared_gear_is_held() {
    let result = validate(with(
        fighter(1, 12),
        &[
            ("gear.belt", json!("belt-of-dexterity-2")),
            ("gear.ring.0", json!("none")),
            (
                "gear.usable.manual-of-dexterity-2",
                json!("manual-of-dexterity-2"),
            ),
        ],
    ))
    .await;
    assert_eq!(result["status"], "valid", "{result}");
    let build = decode(&text(&result["build"]));
    assert_eq!(
        build
            .gear
            .worn
            .values()
            .map(|key| key.as_str())
            .collect::<Vec<_>>(),
        ["belt-of-dexterity-2"]
    );
    assert!(
        build
            .gear
            .used
            .iter()
            .any(|key| key.as_str() == "manual-of-dexterity-2")
    );
}

/// An item the data lacks, one at a place that doesn't take it, and a usable address given another key each fail
/// with what fits.
#[tokio::test]
async fn declared_gear_that_doesnt_fit_fails() {
    let result = validate(with(
        fighter(1, 12),
        &[
            ("gear.belt", json!("belt-of-dexterity-3")),
            ("gear.cloak", json!("ring-of-luck")),
            ("gear.usable.manual-of-dexterity-2", json!("ring-of-luck")),
        ],
    ))
    .await;
    assert_eq!(result["status"], "invalid", "{result}");
    let unknown = failure(&result["failures"], "gear.belt");
    assert_eq!(unknown["issue"], "unknown_key", "{result}");
    assert!(
        text(&unknown["reason"]).contains("belt-of-dexterity-2"),
        "{unknown}"
    );
    let misplaced = failure(&result["failures"], "gear.cloak");
    assert_eq!(misplaced["issue"], "not_allowed", "{result}");
    assert!(
        text(&misplaced["reason"]).contains("gear.ring.0"),
        "{misplaced}"
    );
    let usable = failure(&result["failures"], "gear.usable.manual-of-dexterity-2");
    assert_eq!(usable["issue"], "not_an_option", "{result}");
}

#[tokio::test]
async fn real_data_a_locked_rage_power_names_only_the_class_the_character_has() {
    // Ticket 192 (after 189): Clear Mind needs "Barbarian level 8 or Skald level 8"; a barbarian names its own class,
    // in `validate` as in the picker.
    let client = Client::start().await;
    let mut choices = rmcp::serde_json::Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
        (
            "slot.selection.4.0",
            json!("barbarian/rage-power/clear-mind"),
        ),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=4 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "barbarian" }),
        );
    }
    let validated = client.ok("validate", json!({ "choices": choices })).await;
    assert_eq!(validated["status"], "invalid", "{validated}");
    let failure = validated["failures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|failure| failure["choice"] == "slot.selection.4.0")
        .unwrap_or_else(|| panic!("no failure in {validated}"));
    assert_eq!(
        failure["reason"], "Needs Barbarian level 8 (have 4)",
        "{validated}"
    );
    // `needs` says the same, not the "or Skald level 8 (have 0)" of the data.
    assert_eq!(
        failure["needs"],
        json!(["Barbarian level 8 (have 4)"]),
        "{validated}"
    );
}
