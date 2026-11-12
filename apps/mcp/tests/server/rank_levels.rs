//! A Mythic rank taken later than the rank table's level (ticket 171), through the server: `mythic.R.level` is
//! offered by `options`, taken by `choose` and `validate`, taken back by `withdraw`; `sheet` gives the level each
//! rank is taken at; a move that would refuse a pick is refused, naming the pick; and a rank's pick whose
//! prerequisite comes at a later level says which level of the rank opens it. On the test world, with Toughness
//! (Mythic) at MR 4 (the table's level: 11) and Toughness at level 13, the gold Enforcer's shape.

use rmcp::serde_json::{Map, Value, json};
use rules::test_world::{build, general_slot, path, pick_feat, pick_mythic_feat, times};
use rules::{Build, SlotId};

use crate::support::*;

/// A human fighter 13 on the Trickster's path with Toughness in level 13's general slot.
fn fighter() -> Build {
    let mut build = build("human", &times("fighter", 13));
    build.mythic_path = Some(path("trickster"));
    build.picks.insert(general_slot(13), pick_feat("toughness"));
    build
}

/// The fighter with MR 4 taken at level 13 and Toughness (Mythic) there.
fn moved() -> Build {
    let mut build = fighter();
    build.rank_levels.insert(4, 13);
    build.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat("toughness-mythic"),
    );
    build
}

async fn ok(tool: &str, arguments: Value) -> Value {
    Client::on_the_test_world().await.ok(tool, arguments).await
}

#[tokio::test]
async fn options_list_the_levels_a_rank_can_be_taken_at() {
    let options = ok(
        "options",
        json!({ "build": code(&fighter()), "choice": "mythic.4.level" }),
    )
    .await;
    assert_eq!(options["choice"], "mythic.4.level");
    assert_eq!(
        options["options"],
        json!([
            { "value": 11, "name": "Level 11 (the earliest)", "status": "ok", "current": true },
            { "value": 12, "name": "Level 12", "status": "ok" },
            { "value": 13, "name": "Level 13", "status": "ok" },
        ])
    );
    // A move that takes the ranks above along says so.
    let options = ok(
        "options",
        json!({ "build": code(&fighter()), "choice": "mythic.3.level" }),
    )
    .await;
    assert_eq!(
        options["options"][3]["name"],
        "Level 12, which moves MR 4 to level 12"
    );
}

#[tokio::test]
async fn a_rank_the_build_does_not_reach_has_no_level_to_choose() {
    let client = Client::on_the_test_world().await;
    for (choice, phrase) in [
        ("mythic.5.level", "MR 5 comes at level 14 at the earliest"),
        ("mythic.11.level", "There is no MR 11"),
    ] {
        let refused = client
            .call(
                "options",
                json!({ "build": code(&fighter()), "choice": choice }),
            )
            .await;
        assert_eq!(error_code(&refused), "no_such_choice");
        assert!(error_text(&refused).contains(phrase), "{refused:?}");
    }
}

#[tokio::test]
async fn a_ranks_pick_that_needs_a_later_level_names_the_level_that_opens_it_and_taking_the_rank_there_does()
 {
    let client = Client::on_the_test_world().await;
    let step = json!({ "choice": "slot.mythic.4", "value": "toughness-mythic" });
    let refused = client
        .ok(
            "choose",
            json!({ "build": code(&fighter()), "steps": [step] }),
        )
        .await;
    assert_eq!(
        refused["rejected"],
        json!({
            "step": 0,
            "choice": "slot.mythic.4",
            "issue": "unmet",
            "reason": "Needs toughness. MR 4 taken at level 13 opens it: `choose` `mythic.4.level` 13 first",
        })
    );
    // The rank first, then its pick.
    let taken = client
        .ok(
            "choose",
            json!({ "build": code(&fighter()), "steps": [
                { "choice": "mythic.4.level", "value": 13 },
                step,
            ] }),
        )
        .await;
    assert_eq!(taken["applied"], 2, "{taken}");
    assert_eq!(decode(&text(&taken["build"])), moved());

    // The sheet gives the level the rank is taken at, and the table's beside it only for a rank taken later.
    let sheet = client.ok("sheet", json!({ "build": taken["build"] })).await;
    let ranks = sheet["ranks"].as_array().unwrap();
    assert_eq!(ranks[3]["character_level"], 13);
    assert_eq!(ranks[3]["table_level"], 11);
    assert_eq!(ranks[3]["status"], "reached");
    assert_eq!(ranks[2]["character_level"], 9);
    assert_eq!(ranks[2].get("table_level"), None);
    assert_eq!(sheet["levels"][10].get("mythic_ranks"), None);
    assert_eq!(sheet["levels"][12]["mythic_ranks"], json!([4]));
}

#[tokio::test]
async fn a_move_a_pick_needs_is_refused_by_choose_and_withdraw_naming_the_pick() {
    let client = Client::on_the_test_world().await;
    let back = client
        .ok(
            "choose",
            json!({ "build": code(&moved()), "steps": [{ "choice": "mythic.4.level", "value": 11 }] }),
        )
        .await;
    assert_eq!(
        back["rejected"],
        json!({
            "step": 0,
            "choice": "mythic.4.level",
            "issue": "breaks_later",
            "reason": "toughness-mythic needs MR 4 later than level 11",
        })
    );
    let refused = client
        .call(
            "withdraw",
            json!({ "build": code(&moved()), "choice": "mythic.4.level" }),
        )
        .await;
    assert_eq!(error_code(&refused), "not_removable");
    let reason = error_text(&refused);
    assert!(
        reason.contains("toughness-mythic needs MR 4 later than level 11"),
        "{reason}"
    );
    // It isn't among what can come off now; the pick is.
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": code(&moved()), "removable": true }),
        )
        .await;
    let removable: Vec<String> = (sheet["removable"].as_array().unwrap().iter())
        .map(text)
        .collect();
    assert!(!removable.contains(&"mythic.4.level".to_owned()));
    assert!(removable.contains(&"slot.mythic.4".to_owned()));

    // The pick first, then the rank comes back, to the code the Build had before the move.
    let without = client
        .ok(
            "withdraw",
            json!({ "build": code(&moved()), "choice": "slot.mythic.4" }),
        )
        .await;
    let earliest = client
        .ok(
            "withdraw",
            json!({ "build": without["build"], "choice": "mythic.4.level" }),
        )
        .await;
    assert_eq!(text(&earliest["build"]), code(&fighter()));
    // A rank at its table level holds nothing to withdraw.
    let nothing = client
        .call(
            "withdraw",
            json!({ "build": code(&fighter()), "choice": "mythic.4.level" }),
        )
        .await;
    assert_eq!(error_code(&nothing), "no_such_choice");
}

/// The fighter declared as `validate` takes it.
fn declared(more: &[(&str, Value)]) -> Value {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
        ("mythic.path", json!("trickster")),
        ("slot.feat.13.general.0", json!("toughness")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=13 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "fighter" }),
        );
    }
    for (address, value) in more {
        choices.insert((*address).to_owned(), value.clone());
    }
    json!({ "choices": choices })
}

#[tokio::test]
async fn validate_takes_a_ranks_level_and_says_why_one_is_none() {
    let mythic = ("slot.mythic.4", json!("toughness-mythic"));
    let valid = ok(
        "validate",
        declared(&[("mythic.4.level", json!(13)), mythic.clone()]),
    )
    .await;
    assert_eq!(valid["status"], "valid", "{valid}");
    let build = decode(&text(&valid["build"]));
    assert_eq!(build.rank_levels.get(&4), Some(&13));

    // Without the move the mythic feat fails; at the table's level, or past the last, the level itself does.
    let unmoved = ok("validate", declared(&[mythic])).await;
    assert_eq!(choices(&unmoved["failures"]), ["slot.mythic.4"]);
    let table = ok("validate", declared(&[("mythic.4.level", json!(11))])).await;
    let failure = at(&table["failures"], "mythic.4.level");
    assert_eq!(failure["issue"], "not_allowed", "{table}");
    assert!(
        text(&failure["reason"]).contains("MR 4 comes at level 11 at the earliest"),
        "{failure}"
    );
    let past = ok("validate", declared(&[("mythic.4.level", json!(14))])).await;
    let failure = at(&past["failures"], "mythic.4.level");
    assert_eq!(failure["issue"], "not_reached", "{past}");
    assert!(
        text(&failure["reason"]).contains("the Build has 13 level(s)"),
        "{failure}"
    );
    // A key isn't a level.
    let key = ok("validate", declared(&[("mythic.4.level", json!("later"))])).await;
    let failure = at(&key["failures"], "mythic.4.level");
    assert!(
        text(&failure["reason"]).contains("takes a number"),
        "{failure}"
    );
}

#[tokio::test]
async fn compare_gives_the_level_of_a_rank_two_builds_take_at_different_levels() {
    let compared = ok(
        "compare",
        json!({ "builds": [code(&fighter()), code(&moved())] }),
    )
    .await;
    let rank = (compared["ranks"].as_array().unwrap().iter())
        .find(|rank| rank["rank"] == 4)
        .unwrap_or_else(|| panic!("no MR 4 in {compared}"));
    assert_eq!(rank["character_level"], json!([11, 13]));
}

#[tokio::test]
async fn the_instructions_and_the_guide_say_a_rank_can_be_taken_later() {
    let client = Client::on_the_test_world().await;
    let instructions = client.instructions();
    let guide = client.guide().await;
    for text in [&instructions, &guide] {
        for phrase in ["`mythic.R.level`", "comes after this slot", "taken later"] {
            assert!(text.contains(phrase), "no {phrase} in {text}");
        }
    }
    let before = client.guide_section("Before you plan").await;
    for phrase in [
        "6, 7, 9, 11, 14, 15, 16, 17, 17, 19",
        "takes the ranks above it along",
        "`breaks_later`",
        "taking the rank later fixes it too",
    ] {
        assert!(before.contains(phrase), "no {phrase} in {before}");
    }
    for tool in ["options", "withdraw"] {
        let description = client.tool_description(tool).await;
        assert!(
            description.contains("`mythic.R.level`"),
            "{tool}: {description}"
        );
    }
}
