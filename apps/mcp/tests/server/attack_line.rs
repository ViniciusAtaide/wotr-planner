//! The attack line (ticket 114): `sheet` and `compare` take an optional `weapon` and show the engine's line for it.

use rmcp::serde_json::{Value, json};
use rules::test_world::{build, class_slot, pick_feat, pick_feat_for, times, weapon};
use rules::{Ability, Build};

use crate::support::*;

/// A human fighter 4 with Str 16: Power Attack at 1, Weapon Focus (Longsword) at 2, +1 Strength at 4.
fn fighter() -> Build {
    let mut fighter = build("human", &times("fighter", 4));
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(
        class_slot(2, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    fighter.levels[3].ability_increase = Some(Ability::Strength);
    fighter
}

#[tokio::test]
async fn the_sheet_gives_each_level_the_weapons_attack_line() {
    let client = Client::on_the_test_world().await;
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": code(&fighter()), "weapon": "longsword" }),
        )
        .await;
    let attack = |level: usize| sheet["levels"][level]["attack"].clone();

    // Level 1: BAB +1, Str +3, Power Attack −1/+2.
    assert_eq!(
        attack(0)["weapon"],
        json!({ "key": "longsword", "name": "longsword" })
    );
    assert_eq!(attack(0)["attacks"], json!([4]));
    assert_eq!(attack(0)["damage"], "1d8+3");
    assert_eq!(attack(0)["critical"], "19–20/×2");
    assert_eq!(
        attack(0)["power_attack"],
        json!({ "attack_penalty": 1, "damage_bonus": 2 })
    );
    // Level 2: Weapon Focus +1.
    assert_eq!(attack(1)["attacks"], json!([6]));
    assert_eq!(
        attack(1)["counts"],
        json!([
            { "name": "Strength", "attack": 3, "damage": 3 },
            { "name": "weapon-focus", "attack": 1, "damage": 0 },
        ])
    );
    // Level 4: BAB +4, Power Attack −2/+4.
    assert_eq!(attack(3)["attacks"], json!([8]));
    assert_eq!(attack(3)["power_attack"]["damage_bonus"], 4);

    let leaves_out = sheet["attack_leaves_out"].as_array().unwrap();
    assert!(
        leaves_out
            .iter()
            .any(|item| text(item).contains("mythic feats")),
        "{leaves_out:?}"
    );
}

#[tokio::test]
async fn without_a_weapon_the_sheet_has_no_attack_line() {
    let client = Client::on_the_test_world().await;
    let sheet = client
        .ok("sheet", json!({ "build": code(&fighter()) }))
        .await;

    assert_eq!(sheet["levels"][0].get("attack"), None);
    assert_eq!(sheet.get("attack_leaves_out"), None);
}

#[tokio::test]
async fn a_weapon_without_a_line_is_refused() {
    let client = Client::on_the_test_world().await;
    let result = client
        .call(
            "sheet",
            json!({ "build": code(&fighter()), "weapon": "no-such-weapon" }),
        )
        .await;

    assert_eq!(error_code(&result), "invalid_arguments");
    assert!(error_text(&result).contains("no-such-weapon"), "{result:?}");
}

#[tokio::test]
async fn compare_gives_each_builds_summary_its_attack_line() {
    let client = Client::on_the_test_world().await;
    let short = build("human", &times("fighter", 1));
    let compared: Value = client
        .ok(
            "compare",
            json!({ "builds": [code(&fighter()), code(&short)], "weapon": "longsword" }),
        )
        .await;

    assert_eq!(compared["builds"][0]["attack"]["attacks"], json!([8]));
    assert_eq!(compared["builds"][1]["attack"]["attacks"], json!([4]));
    assert_eq!(compared["builds"][1]["attack"]["power_attack"], Value::Null);
    assert!(compared["attack_leaves_out"].is_array());
}

#[tokio::test]
async fn the_instructions_let_an_agent_state_the_attack_line_as_fact() {
    let client = Client::on_the_test_world().await;
    let instructions = client.instructions();

    assert!(instructions.contains("attack line"), "{instructions}");
    for left_out in [
        "rage",
        "stances",
        "mythic feats",
        "class features",
        "gear",
        "buffs",
    ] {
        assert!(
            instructions.contains(left_out),
            "{left_out}: {instructions}"
        );
    }
}
