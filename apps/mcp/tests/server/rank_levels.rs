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

    // Without the move the mythic feat fails; before the table's level, or past the last, the level itself does.
    let unmoved = ok("validate", declared(&[mythic])).await;
    assert_eq!(choices(&unmoved["failures"]), ["slot.mythic.4"]);
    let early = ok("validate", declared(&[("mythic.4.level", json!(10))])).await;
    let failure = at(&early["failures"], "mythic.4.level");
    assert_eq!(failure["issue"], "not_allowed", "{early}");
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

/// A Legend of 22 levels with its switch taken at level 20.
fn late_legend() -> Build {
    let mut build = build("human", &[times("fighter", 20), times("rogue", 2)].concat());
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    build.rank_levels.insert(8, 20);
    build
}

#[tokio::test]
async fn a_rank_that_is_never_reached_shows_the_tables_level_and_no_move() {
    let sheet = ok("sheet", json!({ "build": code(&late_legend()) })).await;
    let ranks = sheet["ranks"].as_array().unwrap();
    // MR 9 and 10 never come after a Legend switch: their table levels, as before, with nothing moved.
    for (index, level) in [(8, 17), (9, 19)] {
        assert_eq!(ranks[index]["status"], "not_reached", "{sheet}");
        assert_eq!(ranks[index]["character_level"], level);
        assert_eq!(ranks[index].get("table_level"), None);
    }
    // The switch itself is taken at level 20, and says so.
    assert_eq!(ranks[7]["character_level"], 20);
    assert_eq!(ranks[7]["table_level"], 17);
    // A Build too short for a rank shows its table level too.
    let short = ok("sheet", json!({ "build": code(&moved()) })).await;
    assert_eq!(short["ranks"][4]["character_level"], 14);
    assert_eq!(short["ranks"][4].get("table_level"), None);
    // And compare has no level to report for a rank neither Build takes.
    let mut at_table = late_legend();
    at_table.rank_levels.clear();
    let compared = ok(
        "compare",
        json!({ "builds": [code(&at_table), code(&late_legend())] }),
    )
    .await;
    let levels: Vec<(u64, Value)> = (compared["ranks"].as_array().unwrap().iter())
        .filter(|rank| rank.get("character_level").is_some())
        .map(|rank| {
            (
                rank["rank"].as_u64().unwrap(),
                rank["character_level"].clone(),
            )
        })
        .collect();
    assert_eq!(levels, [(8, json!([17, 20]))]);
}

#[tokio::test]
async fn the_level_that_opens_a_pick_names_the_ranks_the_move_takes_along() {
    // Toughness (Mythic) at MR 2: MR 2 at level 13 opens it, and takes MR 3 and 4 with it.
    let refused = ok(
        "choose",
        json!({ "build": code(&fighter()), "steps": [
            { "choice": "slot.mythic.2", "value": "toughness-mythic" },
        ] }),
    )
    .await;
    assert_eq!(
        refused["rejected"]["reason"],
        "Needs toughness. MR 2 taken at level 13 opens it, which moves MR 3 to level 13, MR 4 to level 13: \
         `choose` `mythic.2.level` 13 first"
    );
}

/// A human fighter of `levels` levels (a rogue past the 20th) as `validate` takes it, with more choices.
fn plain(levels: u8, more: &[(&str, Value)]) -> Value {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=levels {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": if level <= 20 { "fighter" } else { "rogue" } }),
        );
    }
    for (address, value) in more {
        choices.insert((*address).to_owned(), value.clone());
    }
    json!({ "choices": choices })
}

/// The rank levels a valid `validate` result's Build holds.
fn rank_levels_of(valid: &Value) -> Vec<(u8, u8)> {
    assert_eq!(valid["status"], "valid", "{valid}");
    decode(&text(&valid["build"]))
        .rank_levels
        .into_iter()
        .collect()
}

#[tokio::test]
async fn validate_takes_the_level_a_rank_comes_at_as_no_change() {
    // Each rank's table level, on a 14-level Build, is the Build with neither declared.
    let none = ok("validate", plain(14, &[])).await;
    let both = ok(
        "validate",
        plain(
            14,
            &[("mythic.4.level", json!(11)), ("mythic.5.level", json!(14))],
        ),
    )
    .await;
    assert_eq!(both["status"], "valid", "{both}");
    assert_eq!(both["build"], none["build"]);
    assert_eq!(both["link"], none["link"]);
    // One alone, on the 13-level fighter, too.
    let one = ok("validate", declared(&[("mythic.4.level", json!(11))])).await;
    let without = ok("validate", declared(&[])).await;
    assert_eq!(one["status"], "valid", "{one}");
    assert_eq!(one["build"], without["build"]);
}

#[tokio::test]
async fn validate_takes_the_level_the_rank_below_holds_a_rank_at_as_no_change() {
    // MR 3 at 13 holds MR 4 at 13: that is where MR 4 comes, so declaring it changes nothing (`choose` stores
    // the same Build).
    let both = ok(
        "validate",
        declared(&[("mythic.3.level", json!(13)), ("mythic.4.level", json!(13))]),
    )
    .await;
    let alone = ok("validate", declared(&[("mythic.3.level", json!(13))])).await;
    assert_eq!(both["status"], "valid", "{both}");
    assert_eq!(both["build"], alone["build"]);
    assert_eq!(rank_levels_of(&both), [(3, 13)]);
    let chosen = ok(
        "choose",
        json!({ "build": code(&fighter()), "steps": [
            { "choice": "mythic.3.level", "value": 13 },
            { "choice": "mythic.4.level", "value": 13 },
        ] }),
    )
    .await;
    assert_eq!(
        decode(&text(&chosen["build"])).rank_levels,
        decode(&text(&both["build"])).rank_levels
    );
}

#[tokio::test]
async fn validate_still_refuses_a_level_before_where_a_rank_comes() {
    let early = ok("validate", declared(&[("mythic.4.level", json!(10))])).await;
    let failure = at(&early["failures"], "mythic.4.level");
    assert_eq!(failure["issue"], "not_allowed", "{early}");
    // MR 3 at 13 holds MR 4 there: MR 4's own table level, 11, is before where it comes, and the reason says so.
    let below = ok(
        "validate",
        declared(&[("mythic.3.level", json!(13)), ("mythic.4.level", json!(11))]),
    )
    .await;
    let failure = at(&below["failures"], "mythic.4.level");
    assert_eq!(failure["issue"], "not_allowed", "{below}");
    let reason = text(&failure["reason"]);
    assert!(
        reason.contains("MR 4 comes at level 13 at the earliest")
            && reason.contains("MR 3 is taken at level 13"),
        "{reason}"
    );
}

#[tokio::test]
async fn validate_keeps_a_level_a_rank_has_of_its_own_under_a_later_rank_below() {
    // 171's dormant level: MR 4 at 12 under MR 3 at 13 is accepted and kept, as it was.
    let dormant = ok(
        "validate",
        declared(&[("mythic.3.level", json!(13)), ("mythic.4.level", json!(12))]),
    )
    .await;
    assert_eq!(rank_levels_of(&dormant), [(3, 13), (4, 12)]);
    let alone = ok("validate", declared(&[("mythic.3.level", json!(13))])).await;
    assert_ne!(dormant["build"], alone["build"]);
}

#[tokio::test]
async fn validate_refuses_the_earliest_level_of_a_rank_the_build_does_not_reach() {
    // Too few levels: MR 5 comes at level 14, the Build has 13. `choose` and `options` refuse it too.
    let short = ok("validate", declared(&[("mythic.5.level", json!(14))])).await;
    assert_eq!(short["status"], "invalid", "{short}");
    assert_eq!(
        at(&short["failures"], "mythic.5.level")["issue"],
        "not_allowed"
    );
    let refused = Client::on_the_test_world()
        .await
        .call(
            "options",
            json!({ "build": code(&fighter()), "choice": "mythic.5.level" }),
        )
        .await;
    assert_eq!(error_code(&refused), "no_such_choice");

    // A Legend switch ended the ascensions at MR 8: MR 9 comes at level 17 and is never reached.
    let legend = [
        ("mythic.path", json!("trickster")),
        ("mythic.switch", json!("legend")),
    ];
    let legend_build = ok("validate", plain(22, &legend)).await;
    assert_eq!(legend_build["status"], "valid", "{legend_build}");
    let mut with_nine = legend.to_vec();
    with_nine.push(("mythic.9.level", json!(17)));
    let ended = ok("validate", plain(22, &with_nine)).await;
    assert_eq!(ended["status"], "invalid", "{ended}");
    assert_eq!(
        at(&ended["failures"], "mythic.9.level")["issue"],
        "not_allowed"
    );
}

#[tokio::test]
async fn validate_takes_every_level_options_offers_for_a_rank() {
    let client = Client::on_the_test_world().await;
    for rank in [3, 4] {
        let address = format!("mythic.{rank}.level");
        let options = client
            .ok(
                "options",
                json!({ "build": code(&fighter()), "choice": address }),
            )
            .await;
        for option in options["options"].as_array().unwrap() {
            let valid = client
                .ok(
                    "validate",
                    declared(&[(address.as_str(), option["value"].clone())]),
                )
                .await;
            assert_eq!(valid["status"], "valid", "{address} {option}: {valid}");
        }
    }
}
