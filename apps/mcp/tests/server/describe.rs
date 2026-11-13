//! `describe`: one item, with a class's or archetype's rows by level and a Mythic path's features by rank.

use rmcp::serde_json::{Value, json};

use crate::support::*;

/// The `key`s of a list of `{key, name}`.
fn keys(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["key"]))
        .collect()
}

/// The `name`s of a list of `{key, name}`.
fn names(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["name"]))
        .collect()
}

#[tokio::test]
async fn a_class_has_its_rows_by_level() {
    let client = Client::start().await;
    let arcanist = client
        .ok("describe", json!({ "kind": "class", "key": "arcanist" }))
        .await;
    assert_eq!(arcanist["name"], "Arcanist");
    assert_eq!(arcanist["category"], "Base class");
    let levels = arcanist["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 20);

    let first = &levels[0];
    assert_eq!(first["level"], 1);
    assert_eq!(
        names(&first["features"]),
        ["Arcane Reservoir", "Cantrips", "Consume Spells"]
    );
    assert_eq!(keys(&first["selections"]), ["arcanist/arcanist-exploits"]);
    assert_eq!(first["spell_levels"], json!({ "arcane": 1, "divine": 0 }));
    // Half BAB, a good Will save.
    assert_eq!(first["bab"], 0);
    assert_eq!(first["will"], 2);
    assert_eq!(levels[19]["bab"], 10);

    assert!(keys(&arcanist["archetypes"]).contains(&"arcanist/brown-fur-transmuter".to_owned()));
    assert!(names(&arcanist["selections"]).contains(&"Arcanist Exploits".to_owned()));
}

#[tokio::test]
async fn an_archetype_has_its_rows_merged_over_its_classs() {
    let client = Client::start().await;
    let bft = client
        .ok(
            "describe",
            json!({ "kind": "archetype", "key": "arcanist/brown-fur-transmuter" }),
        )
        .await;
    assert_eq!(bft["parent"], "arcanist");
    let levels = bft["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 20);
    // Its own rows replace the arcanist's at their levels; the rest are the arcanist's.
    assert!(names(&levels[2]["features"]).contains(&"Powerful Change".to_owned()));
    assert!(
        keys(&levels[0]["features"]).contains(&"arcanist/brown-fur-transmuter/cantrips".to_owned())
    );
    assert_eq!(
        keys(&levels[0]["selections"]),
        ["arcanist/arcanist-exploits"]
    );
    assert!(names(&levels[8]["features"]).contains(&"Shared Transmutation".to_owned()));
}

#[tokio::test]
async fn an_archetypes_replacement_for_a_skipped_option_row_is_in_its_table() {
    // The Empyreal Sorcerer's Apotheosis is its selection's `replaced_by` at 20, not on a class row.
    let client = Client::start().await;
    let empyreal = client
        .ok(
            "describe",
            json!({ "kind": "archetype", "key": "sorcerer/empyreal-sorcerer" }),
        )
        .await;
    let levels = empyreal["levels"].as_array().unwrap();
    assert_eq!(
        keys(&levels[19]["features"]),
        ["sorcerer/empyreal-sorcerer/apotheosis"]
    );
    assert!(
        !keys(&levels[18]["features"])
            .contains(&"sorcerer/empyreal-sorcerer/apotheosis".to_owned())
    );
}

#[tokio::test]
async fn a_mythic_path_has_its_features_by_rank_and_its_selections() {
    let client = Client::start().await;
    let lich = client
        .ok("describe", json!({ "kind": "path", "key": "lich" }))
        .await;
    assert_eq!(lich["name"], "Lich");
    let ranks = lich["ranks"].as_array().unwrap();
    assert_eq!(ranks.len(), 10);
    assert_eq!(ranks[1]["rank"], 2);
    assert_eq!(
        names(&ranks[1]["features"]),
        // In the data's canonical order, by key.
        ["Hard to Kill", "Negative Energy Insight", "Summon Spectre"]
    );
    assert_eq!(
        keys(&ranks[2]["selections"]),
        ["lich/lich-powers", "lich/skeletal-champion"]
    );

    let selections = lich["selections"].as_array().unwrap();
    let powers = selections
        .iter()
        .find(|selection| selection["key"] == "lich/lich-powers")
        .expect("the Lich powers are listed");
    assert_eq!(powers["name"], "Lich Powers");
    assert!(names(&powers["options"]).contains(&"Deadly Magic".to_owned()));
}

#[tokio::test]
async fn a_feat_has_its_prerequisite_in_words() {
    let client = Client::start().await;
    let feat = client
        .ok(
            "describe",
            json!({ "kind": "feat", "key": "augment-summoning" }),
        )
        .await;
    assert_eq!(feat["name"], "Augment Summoning");
    assert_eq!(feat["prerequisite"], "Spell Focus (Conjuration)");
}

#[tokio::test]
async fn an_unknown_key_is_an_error_result() {
    let client = Client::start().await;
    let result = client
        .call(
            "describe",
            json!({ "kind": "archetype", "key": "brown-fur-transmuter" }),
        )
        .await;
    let message = error_text(&result);
    assert!(message.contains("brown-fur-transmuter"), "{message}");
    // It points at the right key.
    assert!(
        message.contains("arcanist/brown-fur-transmuter"),
        "{message}"
    );

    // A key of another kind isn't found either, and the server carries on.
    let result = client
        .call("describe", json!({ "kind": "class", "key": "lich" }))
        .await;
    assert!(error_text(&result).contains("lich"));
    client
        .ok("describe", json!({ "kind": "path", "key": "lich" }))
        .await;
}
