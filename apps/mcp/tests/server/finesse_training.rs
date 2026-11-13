//! Ticket 172 on the real data: Finesse Training takes a weapon. A code from before it (a rogue 3 with no weapon)
//! opens with the weapon in `todo`, `options` lists the guide's 21 weapons, `choose` takes one, and the next Finesse
//! Training slot has that weapon taken.

use rmcp::serde_json::{Map, Value, json};
use rules::{Build, ClassChoice, ClassKey, LevelUp};

use crate::support::*;

/// A rogue of `levels` levels with no picks: at 3, what every code before ticket 172 held for Finesse Training.
fn rogue(levels: usize) -> Build {
    let mut build = Build::default();
    build
        .class_choices
        .insert(ClassKey::new("rogue"), ClassChoice::default());
    for _ in 0..levels {
        build.levels.push(LevelUp::new(ClassKey::new("rogue")));
    }
    build
}

const AT_3: &str = "slot.selection.3.0";
const AT_11: &str = "slot.selection.11.0";

fn shield() -> Value {
    json!({ "key": "rogue/finesse-training/weapon", "parameter": "light-shield-bash" })
}

#[tokio::test]
async fn real_data_an_older_rogue_opens_with_its_finesse_training_weapon_in_todo() {
    let client = Client::start().await;
    let sheet = client
        .ok("sheet", json!({ "build": code(&rogue(3)) }))
        .await;
    let open = at(&sheet["todo"], AT_3);
    assert_eq!(open["label"], "Level 3: Finesse Training", "{open}");
}

#[tokio::test]
async fn real_data_finesse_training_lists_the_guides_weapons_and_takes_one() {
    let client = Client::start().await;
    let older = code(&rogue(11));
    let listed = client
        .ok(
            "options",
            json!({ "build": older, "choice": AT_3, "values": true }),
        )
        .await;
    let options = listed["options"].as_array().unwrap();
    assert_eq!(options.len(), 21, "{listed}");
    let offered = options
        .iter()
        .find(|option| option["value"] == shield())
        .unwrap_or_else(|| panic!("no Light Shield Bash in {listed}"));
    assert_eq!(offered["status"], "ok");
    assert_eq!(offered["name"], "Finesse Training (Light Shield Bash)");

    let chosen = client
        .ok(
            "choose",
            json!({ "build": older, "steps": [{ "choice": AT_3, "value": shield() }] }),
        )
        .await;
    assert_eq!(chosen["applied"], 1, "{chosen}");
    assert!(!choices(&chosen["todo"]).contains(&AT_3.to_owned()));
    assert!(choices(&chosen["todo"]).contains(&AT_11.to_owned()));

    // At 11, the shield is taken.
    let next = client
        .ok(
            "options",
            json!({ "build": chosen["build"], "choice": AT_11, "values": true, "only_open": false }),
        )
        .await;
    let taken = next["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["value"] == shield())
        .unwrap_or_else(|| panic!("no Light Shield Bash in {next}"));
    assert_eq!(
        taken["status"],
        json!({
            "issue": "already_held",
            // Ticket 192: it says where the weapon is held.
            "reason": "Finesse Training (Light Shield Bash) is taken in level 3's Finesse Training slot",
        }),
        "{taken}"
    );
}

#[tokio::test]
async fn real_data_finesse_training_by_its_key_alone_needs_its_weapon() {
    let client = Client::start().await;
    let chosen = client
        .ok(
            "choose",
            json!({
                "build": code(&rogue(3)),
                "steps": [{ "choice": AT_3, "value": "rogue/finesse-training/weapon" }],
            }),
        )
        .await;
    assert_eq!(chosen["applied"], 0, "{chosen}");
    assert_eq!(chosen["rejected"]["issue"], "needs_parameter", "{chosen}");
}

#[tokio::test]
async fn real_data_a_weapon_taken_twice_says_where_in_choose_and_validate() {
    // Ticket 192.
    let client = Client::start().await;
    let held = "Finesse Training (Light Shield Bash) is taken in level 3's Finesse Training slot";
    let chosen = client
        .ok(
            "choose",
            json!({ "build": code(&rogue(11)), "steps": [
                { "choice": AT_3, "value": shield() },
                { "choice": AT_11, "value": shield() },
            ] }),
        )
        .await;
    assert_eq!(chosen["applied"], 1, "{chosen}");
    assert_eq!(chosen["rejected"]["reason"], held, "{chosen}");
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
        (AT_3, shield()),
        (AT_11, shield()),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=11 {
        choices.insert(format!("level.{level}.class"), json!({ "class": "rogue" }));
    }
    let validated = client.ok("validate", json!({ "choices": choices })).await;
    assert_eq!(validated["status"], "invalid", "{validated}");
    let failure = validated["failures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|failure| failure["choice"] == AT_11)
        .unwrap_or_else(|| panic!("no failure at {AT_11} in {validated}"));
    assert_eq!(failure["issue"], "already_held");
    assert_eq!(failure["reason"], held);
}
