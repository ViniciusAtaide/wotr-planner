//! `search`: names, keys and initials across the game data's kinds.

use rmcp::serde_json::json;

use crate::support::*;

fn pair(kind: &str, key: &str) -> (String, String) {
    (kind.to_owned(), key.to_owned())
}

#[tokio::test]
async fn initials_find_the_brown_fur_transmuter() {
    let client = Client::start().await;
    let matches = client.search_for("BFT").await;
    assert_eq!(
        matches.first(),
        Some(&pair("archetype", "arcanist/brown-fur-transmuter"))
    );
}

#[tokio::test]
async fn a_match_carries_its_parent_prerequisite_and_guide_link() {
    let client = Client::start().await;
    let found = client
        .ok(
            "search",
            json!({ "query": "Brown-Fur Transmuter", "limit": 1 }),
        )
        .await;
    let hit = &found["matches"][0];
    assert_eq!(hit["name"], "Brown-Fur Transmuter");
    assert_eq!(hit["parent"], "arcanist");
    assert_eq!(
        hit["guide_url"],
        "https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/arcanist#brown-fur-transmuter"
    );

    let found = client
        .ok(
            "search",
            json!({ "query": "Superior Summoning", "limit": 1 }),
        )
        .await;
    let hit = &found["matches"][0];
    assert_eq!(hit["kind"], "feat");
    assert_eq!(hit["category"], "Magic");
    assert_eq!(hit["prerequisite"], "Augment Summoning");
}

#[tokio::test]
async fn an_exact_name_comes_first() {
    let client = Client::start().await;
    let matches = client.search_for("Augment Summoning").await;
    assert_eq!(matches.first(), Some(&pair("feat", "augment-summoning")));
    // Before the longer names that start with it and the ones that hold its words, kind before kind.
    let matches = client.search_for("spell focus").await;
    assert_eq!(
        matches,
        [
            pair("feat", "spell-focus"),
            pair("mythic_feat", "spell-focus-mythic"),
            pair("feat", "greater-spell-focus"),
        ]
    );
}

#[tokio::test]
async fn a_key_finds_its_item() {
    let client = Client::start().await;
    let matches = client.search_for("lich").await;
    assert_eq!(matches.first(), Some(&pair("path", "lich")));
}

#[tokio::test]
async fn kinds_filter_the_matches() {
    let client = Client::start().await;
    let matches = client
        .search(json!({ "query": "summon", "kinds": ["mythic_feat", "mythic_ability"] }))
        .await;
    assert_eq!(
        matches.first(),
        Some(&pair("mythic_ability", "ascendant-summons"))
    );
    assert!(
        client
            .search(json!({ "query": "summon" }))
            .await
            .iter()
            .any(|(kind, _)| kind == "feat"),
        "unfiltered, the feats match too"
    );
    assert!(
        matches
            .iter()
            .all(|(kind, _)| kind == "mythic_feat" || kind == "mythic_ability"),
        "{matches:?}"
    );
}

#[tokio::test]
async fn limit_caps_the_matches() {
    let client = Client::start().await;
    let matches = client
        .search(json!({ "query": "summon", "limit": 3 }))
        .await;
    assert_eq!(matches.len(), 3);
}

#[tokio::test]
async fn races_deities_and_backgrounds_are_found_too() {
    let client = Client::start().await;
    for (query, kind) in [
        ("Aasimar", "race"),
        ("Erastil", "deity"),
        ("Bounty Hunter", "background"),
    ] {
        let matches = client.search_for(query).await;
        assert_eq!(
            matches.first().map(|(found, _)| found.as_str()),
            Some(kind),
            "{query}: {matches:?}"
        );
    }
}

#[tokio::test]
async fn an_empty_query_is_an_error() {
    let client = Client::start().await;
    let result = client.call("search", json!({ "query": "  " })).await;
    assert!(error_text(&result).contains("query"));
}
