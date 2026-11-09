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
async fn an_empty_query_without_a_filter_is_an_error() {
    let client = Client::start().await;
    for arguments in [json!({ "query": "  " }), json!({})] {
        let message = error_text(&client.call("search", arguments).await);
        assert!(
            message.contains("query") && message.contains("category"),
            "{message}"
        );
    }
}

#[tokio::test]
async fn a_category_filters_the_matches_and_needs_no_query() {
    let client = Client::on_the_test_world().await;
    // Every option of a Feature selection, by its name in any case, with no query.
    let talents = client.search(json!({ "category": "rogue talents" })).await;
    assert_eq!(
        talents,
        [
            pair("option", "rogue/rogue-talents/combat-trick"),
            pair("option", "rogue/rogue-talents/crippling-strike"),
            pair("option", "rogue/rogue-talents/fast-stealth"),
            pair("option", "rogue/rogue-talents/resiliency"),
            pair("option", "rogue/rogue-talents/weapon-focus"),
        ]
    );

    // With a query, only the category's matches: the feat, not the rogue talent of the same name.
    let found = client
        .search(json!({ "query": "weapon focus", "category": "Weapon" }))
        .await;
    assert_eq!(found, [pair("feat", "weapon-focus")]);

    // One or more categories.
    let found = client
        .search(json!({
            "category": ["Miscellaneous", "Combat Manoeuvres"],
            "kinds": ["feat"],
            "limit": 50,
        }))
        .await;
    for key in ["dodge", "toughness", "combat-expertise"] {
        assert!(found.contains(&pair("feat", key)), "{key}: {found:?}");
    }
    assert!(!found.contains(&pair("feat", "power-attack")), "{found:?}");
}

#[tokio::test]
async fn an_empty_query_lists_a_kind_and_counts_what_the_limit_hid() {
    let client = Client::on_the_test_world().await;
    let races = client.ok("search", json!({ "kinds": ["race"] })).await;
    let names: Vec<String> = races["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| text(&hit["name"]))
        .collect();
    assert_eq!(names, ["Dwarf", "Elf", "Half-Elf", "Human"]);
    assert_eq!(races["total"], 4);

    let two = client
        .ok("search", json!({ "kinds": ["race"], "limit": 2 }))
        .await;
    assert_eq!(two["matches"].as_array().unwrap().len(), 2);
    assert_eq!(two["total"], 4);
}

#[tokio::test]
async fn no_match_says_so_and_hints_only_at_close_names() {
    let client = Client::on_the_test_world().await;
    // A tabletop feat: "focus" alone doesn't make Weapon Focus a near name.
    let found = client
        .ok("search", json!({ "query": "Furious Focus" }))
        .await;
    assert_eq!(found["matches"], json!([]));
    assert_eq!(found["total"], 0);
    let note = text(&found["note"]);
    assert!(
        note.starts_with("No item in the planner's data matches `Furious Focus`."),
        "{note}"
    );
    assert!(!note.contains("weapon-focus"), "{note}");

    // A misspelt name gets the near one.
    let found = client.ok("search", json!({ "query": "power atack" })).await;
    assert_eq!(found["matches"], json!([]));
    let note = text(&found["note"]);
    assert!(note.contains("`power-attack`"), "{note}");

    // A category no item has names the close ones.
    let found = client
        .ok("search", json!({ "category": "Rogue Talent" }))
        .await;
    assert_eq!(found["matches"], json!([]));
    let note = text(&found["note"]);
    assert!(note.contains("\"Rogue Talents\""), "{note}");

    // A match leaves the note out.
    let found = client.ok("search", json!({ "query": "dodge" })).await;
    assert!(found.get("note").is_none(), "{found}");
}

#[tokio::test]
async fn real_data_every_melee_mythic_feat_in_one_call() {
    let client = Client::start().await;
    let found = client
        .ok(
            "search",
            json!({ "category": "melee", "kinds": ["mythic_feat"] }),
        )
        .await;
    let matches = found["matches"].as_array().unwrap();
    assert!(matches.len() > 10, "more than the default limit: {found}");
    assert_eq!(found["total"], matches.len());
    assert!(
        matches
            .iter()
            .all(|hit| hit["kind"] == "mythic_feat" && hit["category"] == "Melee"),
        "{found}"
    );
}
