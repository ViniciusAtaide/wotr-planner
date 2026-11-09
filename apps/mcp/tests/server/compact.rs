//! Compact responses (ticket 128): `compact` on `search` and `options`, long prerequisites shortened in `search`,
//! and `options` without the envelope unless asked.

use rmcp::serde_json::{self, Value, json};

use crate::support::*;

/// A result's size as compact JSON, in bytes.
fn size(value: &Value) -> usize {
    serde_json::to_string(value).unwrap().len()
}

/// A JSON object's keys, sorted.
fn keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

#[tokio::test]
async fn real_data_compact_search_is_under_half_the_mythic_abilities_size() {
    // The Max DC run's `search {kinds: ["mythic_ability"]}`: all 63 abilities with full prerequisites.
    let client = Client::start().await;
    let arguments = json!({ "kinds": ["mythic_ability"], "limit": 100 });
    let full = client.ok("search", arguments.clone()).await;
    let mut compact_arguments = arguments;
    compact_arguments["compact"] = json!(true);
    let compact = client.ok("search", compact_arguments).await;

    assert_eq!(compact["total"], full["total"]);
    let hits = compact["matches"].as_array().unwrap();
    assert_eq!(hits.len(), full["matches"].as_array().unwrap().len());
    for hit in hits {
        assert_eq!(keys(hit), ["category", "key", "kind", "name"], "{hit}");
    }
    assert!(
        size(&compact) * 2 < size(&full),
        "compact {} bytes, full {} bytes",
        size(&compact),
        size(&full)
    );
}

#[tokio::test]
async fn real_data_a_long_prerequisite_is_shortened_in_search_and_whole_in_describe() {
    let client = Client::start().await;
    let found = client
        .ok(
            "search",
            json!({ "query": "Witch Wandering Hex", "kinds": ["mythic_ability"], "limit": 1 }),
        )
        .await;
    let short = text(&found["matches"][0]["prerequisite"]);
    assert!(
        short.starts_with("one of ")
            && short.contains(" hexes (Air Barrier, ")
            && short.ends_with(", …)"),
        "{short}"
    );
    let described = client
        .ok(
            "describe",
            json!({ "kind": "mythic_ability", "key": "witch-wandering-hex" }),
        )
        .await;
    let whole = text(&described["prerequisite"]);
    assert!(
        whole.starts_with("Air Barrier or Battle Master or "),
        "{whole}"
    );
    assert!(whole.len() > 4 * short.len(), "{short} | {whole}");

    // A short prerequisite stays as it is.
    let found = client
        .ok(
            "search",
            json!({ "query": "Superior Summoning", "limit": 1 }),
        )
        .await;
    assert_eq!(found["matches"][0]["prerequisite"], "Augment Summoning");
}

#[tokio::test]
async fn real_data_options_of_one_name_are_told_apart() {
    let client = Client::start().await;
    let described = client
        .ok(
            "describe",
            json!({ "kind": "mythic_ability", "key": "mythical-beast" }),
        )
        .await;
    let whole = text(&described["prerequisite"]);
    assert!(
        !whole.contains("Animal Companion or Animal Companion"),
        "{whole}"
    );
    assert!(
        whole.ends_with("Animal Companion (Nature Bond) or Animal Companion (Nature Revelations)"),
        "{whole}"
    );
    // Two options of one name in selections of one name read with the selections' owners too.
    let disruptive = client
        .ok("describe", json!({ "kind": "feat", "key": "disruptive" }))
        .await;
    assert_eq!(
        disruptive["prerequisite"],
        "Fighter level 6 or Arcane (Bloodrager Bloodline) or Arcane (Sorcerer Bloodline)"
    );
    // Shortened in `search`, by the name they share.
    let found = client
        .ok(
            "search",
            json!({ "query": "Mythical Beast", "kinds": ["mythic_ability"], "limit": 1 }),
        )
        .await;
    assert_eq!(
        found["matches"][0]["prerequisite"],
        "one of 17 animal companions (Animal Companion (Mad Dog), Animal Companion (Cavalier), Animal Companion \
         (Beast Rider), …)"
    );
}

#[tokio::test]
async fn real_data_compact_options_leave_out_the_summaries() {
    let client = Client::start().await;
    let empty = client.ok("new_build", json!({})).await;
    let arguments = json!({ "build": empty["build"], "choice": "start.race" });
    let full = client.ok("options", arguments.clone()).await;
    assert!(
        full["options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|line| line.get("summary").is_some()),
        "{full}"
    );
    let mut compact_arguments = arguments;
    compact_arguments["compact"] = json!(true);
    let compact = client.ok("options", compact_arguments).await;
    let lines = compact["options"].as_array().unwrap();
    assert_eq!(lines.len(), full["options"].as_array().unwrap().len());
    assert!(
        lines.iter().all(|line| line.get("summary").is_none()),
        "{compact}"
    );
    assert!(lines.iter().all(|line| line["status"] == "ok"), "{compact}");
}

#[tokio::test]
async fn options_return_the_envelope_only_when_asked() {
    let client = Client::on_the_test_world().await;
    let empty = client.ok("new_build", json!({})).await;
    let arguments = json!({ "build": empty["build"], "choice": "start.race" });
    let bare = client.ok("options", arguments.clone()).await;
    assert_eq!(keys(&bare), ["choice", "closed", "options"], "{bare}");

    let mut with = arguments;
    with["envelope"] = json!(true);
    let with = client.ok("options", with).await;
    assert_eq!(with["build"], empty["build"]);
    assert_eq!(with["link"], empty["link"]);
    assert_eq!(with["todo"], empty["todo"]);
}
