//! Assumed gear (ticket 122): `sheet` and `compare` take an optional `assume` and give, beside the unassisted numbers,
//! the last level's lines and totals with it, labelled assumed; the Build, its code and its prerequisites never see
//! it.

use rmcp::serde_json::{Value, json};

use crate::support::*;

/// The Max DC run's Build (2026-11-09): a Kitsune Sorcerer 20 with Charisma 25, enchantment compulsion DC 34.
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

fn compulsion() -> Value {
    json!({ "school": "enchantment", "subschool": "compulsion", "save": "will", "mind_affecting": true })
}

fn end_game() -> Value {
    json!({
        "enhancement": { "cha": 8, "dex": 6 },
        "inherent": { "cha": 4 },
        "resistance": 7,
        "items": ["heartstone-of-the-night-hag-queen", "goggles-of-mind-control", "rod-of-power-source"],
    })
}

fn names(counts: &Value) -> Vec<String> {
    counts
        .as_array()
        .unwrap()
        .iter()
        .map(|part| text(&part["name"]))
        .collect()
}

#[tokio::test]
async fn the_sheet_gives_the_caster_line_and_totals_with_the_assumed_gear_beside_the_unassisted_ones()
 {
    let client = Client::start().await;
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": MAX_DC_RUN, "caster": compulsion(), "assume": end_game() }),
        )
        .await;
    let assumed = &sheet["assumed"];

    // The unassisted line is unchanged.
    assert_eq!(sheet["levels"][19]["caster"]["dcs"][8]["dc"], 34);
    // Charisma 25 raised to 37 (+13, +6); the heartstone +3 and the goggles +2 (mind-affecting); the rod is for
    // conjuration only.
    assert_eq!(assumed["scores"]["cha"], 37);
    assert_eq!(assumed["caster"]["dcs"][8]["dc"], 34 + 6 + 3 + 2);
    let dc_counts = names(&assumed["caster"]["dc_counts"]);
    assert!(
        dc_counts.contains(&"Heartstone of the Night Hag Queen (assumed)".to_owned()),
        "{dc_counts:?}"
    );
    assert!(
        !dc_counts.iter().any(|name| name.contains("Rod")),
        "{dc_counts:?}"
    );
    // The goggles' caster level +2 (mind-affecting).
    assert_eq!(assumed["caster"]["caster_level"], 22);
    assert_eq!(
        assumed["caster"]["caster_level_counts"],
        json!([{ "name": "Goggles of Mind Control (assumed)", "value": 2 }])
    );
    assert_eq!(
        sheet["levels"][19]["caster"].get("caster_level_counts"),
        None
    );
    // Totals: Will 15 + the resistance bonus 7; initiative 19 + Dexterity's 3 more.
    assert_eq!(sheet["totals"]["will"]["total"], 15);
    assert_eq!(assumed["totals"]["will"]["total"], 22);
    assert!(
        names(&assumed["totals"]["will"]["counts"]).contains(&"Resistance +7 (assumed)".to_owned())
    );
    assert_eq!(assumed["totals"]["initiative"]["total"], 22);
    // What was assumed, the items by name.
    assert_eq!(
        assumed["assumes"]["enhancement"],
        json!({ "cha": 8, "dex": 6 })
    );
    assert_eq!(
        assumed["assumes"]["items"][0]["name"],
        "Heartstone of the Night Hag Queen"
    );
    assert_eq!(assumed.get("attack"), None);
}

#[tokio::test]
async fn the_build_its_code_and_its_prerequisites_never_see_the_assumed_gear() {
    let client = Client::start().await;
    let plain = client
        .ok(
            "sheet",
            json!({ "build": MAX_DC_RUN, "caster": compulsion() }),
        )
        .await;
    let assuming = client
        .ok(
            "sheet",
            json!({ "build": MAX_DC_RUN, "caster": compulsion(), "assume": end_game() }),
        )
        .await;

    for field in [
        "build", "link", "todo", "levels", "ranks", "start", "totals",
    ] {
        assert_eq!(plain[field], assuming[field], "{field}");
    }
    assert_eq!(plain.get("assumed"), None);
    // The sheet's own Charisma, which every prerequisite checks, stays the Build's.
    assert_eq!(assuming["levels"][19]["scores"]["cha"], 25);
}

#[tokio::test]
async fn compare_gives_each_summary_its_lines_with_the_assumed_gear() {
    let client = Client::start().await;
    let compared = client
        .ok(
            "compare",
            json!({
                "builds": [MAX_DC_RUN],
                "caster": compulsion(),
                "weapon": "dagger",
                "assume": { "enhancement": { "str": 6 }, "weapon_enhancement": 5, "items": ["mindmasters-eyes"] },
            }),
        )
        .await;
    let summary = &compared["builds"][0];

    assert_eq!(summary["caster"]["dcs"][8]["dc"], 34);
    assert_eq!(summary["assumed"]["caster"]["dcs"][8]["dc"], 36);
    let attack = &summary["assumed"]["attack"];
    assert!(
        names(&attack["counts"]).contains(&"Weapon enhancement +5 (assumed)".to_owned()),
        "{attack}"
    );
    assert_eq!(
        attack["attacks"][0].as_i64().unwrap(),
        summary["attack"]["attacks"][0].as_i64().unwrap() + 3 + 5
    );
    assert_eq!(summary["assumed"].get("totals"), None);

    let result = client
        .call(
            "compare",
            json!({ "builds": [MAX_DC_RUN], "assume": { "resistance": 2 } }),
        )
        .await;
    assert_eq!(error_code(&result), "invalid_arguments");
}

#[tokio::test]
async fn values_above_the_guides_highest_an_unknown_ability_and_an_unknown_item_are_refused() {
    let client = Client::start().await;
    for (assume, named) in [
        (json!({ "enhancement": { "cha": 9 } }), "1 to 8"),
        (json!({ "inherent": { "cha": 10 } }), "1 to 9"),
        (json!({ "enhancement": { "luck": 2 } }), "luck"),
        (json!({ "weapon_enhancement": 6 }), "above 5"),
        (json!({ "resistance": 8 }), "above 7"),
        (
            json!({ "items": ["cloak-of-the-ancient-herald"] }),
            "rod-of-power-source",
        ),
    ] {
        let result = client
            .call("sheet", json!({ "build": MAX_DC_RUN, "assume": assume }))
            .await;

        assert_eq!(error_code(&result), "invalid_arguments", "{assume}");
        assert!(error_text(&result).contains(named), "{result:?}");
    }
}

#[tokio::test]
async fn the_leaves_out_lists_say_gear_is_left_out_unless_assumed() {
    let client = Client::start().await;
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": MAX_DC_RUN, "caster": compulsion(), "weapon": "dagger" }),
        )
        .await;

    for list in ["attack_leaves_out", "caster_leaves_out", "totals_leave_out"] {
        let gear: Vec<String> = sheet[list]
            .as_array()
            .unwrap()
            .iter()
            .map(text)
            .filter(|item| item.starts_with("gear"))
            .collect();
        assert_eq!(gear.len(), 1, "{list}: {gear:?}");
        assert!(gear[0].contains("unless assumed"), "{list}: {gear:?}");
    }
}

#[tokio::test]
async fn the_sheet_tool_names_every_item_and_the_instructions_label_assumed_numbers() {
    let client = Client::start().await;
    let description = client.tool_description("sheet").await;
    let gear = seeds::read_game_data_files(&seeds::game_data_dir())
        .expect("game-data reads")
        .gear;

    assert!(!gear.is_empty());
    for item in &gear {
        assert!(
            description.contains(item.key.as_str()),
            "{} is missing",
            item.key
        );
    }
    for highest in ["+8", "+4 to all", "+5", "+7"] {
        assert!(description.contains(highest), "{highest}");
    }
    let instructions = client.instructions();
    assert!(instructions.contains("Assumed numbers"), "{instructions}");
    assert!(
        instructions.contains("gear unless assumed"),
        "{instructions}"
    );
}
