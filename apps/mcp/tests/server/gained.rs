//! A feature gained more than once at a level says how many times (ticket 192, review): the Trickster's third Mythic
//! rank gives Sneak Attack twice. On the real data.

use rmcp::serde_json::{Map, Value, json};

use crate::support::*;

/// A human fighter of nine levels, with the Trickster's path when `path`: MR 3 comes at level 9.
async fn fighter_9(client: &Client, path: bool) -> Value {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    if path {
        choices.insert("mythic.path".to_owned(), json!("trickster"));
    }
    for level in 1..=9 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "fighter" }),
        );
    }
    client.ok("validate", json!({ "choices": choices })).await["build"].clone()
}

fn feature<'a>(list: &'a Value, key: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|feature| feature["key"] == key)
        .unwrap_or_else(|| panic!("no {key} in {list}"))
}

#[tokio::test]
async fn real_data_the_sheet_says_a_feature_gained_twice_at_a_level_twice() {
    let client = Client::start().await;
    let build = fighter_9(&client, true).await;
    let sheet = client.ok("sheet", json!({ "build": build })).await;
    // Level 9 gains Sneak Attack twice, and each of the others once (no `times`).
    let level = &sheet["levels"][8]["features"];
    assert_eq!(
        feature(level, "trickster/sneak-attack"),
        &json!({ "key": "trickster/sneak-attack", "name": "Sneak Attack (Trickster)", "times": 2 })
    );
    assert_eq!(
        feature(level, "trickster/sneaky-quack"),
        &json!({ "key": "trickster/sneaky-quack", "name": "Sneaky Quack" })
    );
    // So does MR 3's own row: one entry, not the same entry twice.
    let rank = &sheet["ranks"][2]["features"];
    assert_eq!(rank.as_array().unwrap().len(), 3, "{rank}");
    assert_eq!(feature(rank, "trickster/sneak-attack")["times"], 2);
}

#[tokio::test]
async fn real_data_compare_sees_a_feature_gained_twice_against_none() {
    let client = Client::start().await;
    let with = fighter_9(&client, true).await;
    let without = fighter_9(&client, false).await;
    let compared = client
        .ok("compare", json!({ "builds": [with, without] }))
        .await;
    let rank = compared["path_features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diff| diff["rank"] == 3)
        .unwrap_or_else(|| panic!("no rank 3 in {compared}"));
    // The Trickster's list carries the count; the other Build has none of them.
    assert_eq!(
        feature(&rank["only"][0], "trickster/sneak-attack")["times"],
        2
    );
    assert_eq!(rank["only"][1], json!([]));
}
