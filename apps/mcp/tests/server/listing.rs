//! What a client sees on connecting: the tools, the server's instructions and its one resource, the glossary.

use rmcp::model::ResourceContents;

use crate::support::*;

#[tokio::test]
async fn the_server_offers_its_tools() {
    let client = Client::start().await;
    assert_eq!(
        client.tool_names().await,
        [
            "choose",
            "compare",
            "describe",
            "new_build",
            "options",
            "search",
            "sheet",
            "withdraw"
        ]
    );
}

#[tokio::test]
async fn the_instructions_draw_the_honesty_boundary() {
    let client = Client::start().await;
    let instructions = client.instructions();
    // What the agent may claim, and what it must label as its own knowledge.
    assert!(instructions.contains("legal"), "{instructions}");
    assert!(
        instructions.contains("own game knowledge"),
        "{instructions}"
    );
    // Guide links are given, never fetched.
    assert!(instructions.contains("never fetch"), "{instructions}");
}

#[tokio::test]
async fn the_instructions_give_the_workflow() {
    let instructions = Client::start().await.instructions();
    // In order: resolve names, ask, start, the Build start, levels, Mythic, the link.
    let steps = [
        "`search`",
        "ask the user",
        "`new_build`",
        "Build start",
        "`level.N.class`",
        "Mythic ranks",
        "final `link`",
    ];
    let at: Vec<usize> = steps
        .iter()
        .map(|step| {
            instructions
                .find(step)
                .unwrap_or_else(|| panic!("no {step} in {instructions}"))
        })
        .collect();
    assert!(at.is_sorted(), "{steps:?} at {at:?}");
    // Batching, values, branching and the glossary.
    for phrase in [
        "`rejected`",
        "`options {values: true}`",
        "`advances`",
        "number to reach",
        "`compare`",
        "wotr://glossary",
    ] {
        assert!(
            instructions.contains(phrase),
            "no {phrase} in {instructions}"
        );
    }
}

#[tokio::test]
async fn the_instructions_say_what_to_know_before_planning() {
    let instructions = Client::start().await.instructions();
    let before = instructions
        .find("Before you plan:")
        .unwrap_or_else(|| panic!("no paragraph in {instructions}"));
    let workflow = instructions.find("Workflow:").expect("the workflow");
    assert!(before < workflow, "{instructions}");
    let paragraph = &instructions[before..workflow];
    for phrase in [
        // Scores and the point buy.
        "start at 10",
        "25-point budget",
        "`point_buy_spent`",
        // The faith before the alignment.
        "faith",
        "before the alignment",
        // Each Mythic rank's character level.
        "`character_level`",
    ] {
        assert!(paragraph.contains(phrase), "no {phrase} in {paragraph}");
    }
    // The skill map, with the values.
    assert!(instructions.contains("`{skill: ranks}`"), "{instructions}");
}

#[tokio::test]
async fn the_instructions_give_the_pick_order_within_a_level() {
    // Ticket 126: a pick that needs another taken at the same level goes in the later slot.
    let instructions = Client::start().await.instructions();
    let before = instructions
        .find("Before you plan:")
        .expect("the paragraph");
    let workflow = instructions.find("Workflow:").expect("the workflow");
    let paragraph = &instructions[before..workflow];
    let order = [
        "`slot.feat.N.class.I`",
        "race slots",
        "`slot.feat.N.general.0`",
        "`slot.path-selection.R.I`",
    ];
    let at: Vec<usize> = order
        .iter()
        .map(|phrase| {
            paragraph
                .find(phrase)
                .unwrap_or_else(|| panic!("no {phrase} in {paragraph}"))
        })
        .collect();
    assert!(at.is_sorted(), "{order:?} at {at:?}");
    for phrase in ["later slot", "comes after this slot", "swap the two picks"] {
        assert!(paragraph.contains(phrase), "no {phrase} in {paragraph}");
    }
}

#[tokio::test]
async fn the_instructions_say_to_compare_class_splits_before_committing() {
    // Ticket 115, option A: no `reach` tool; branch the splits and `compare` them first.
    let instructions = Client::start().await.instructions();
    let before = instructions
        .find("Before you plan:")
        .expect("the paragraph");
    let workflow = instructions.find("Workflow:").expect("the workflow");
    let paragraph = &instructions[before..workflow];
    for phrase in [
        "class split",
        "Fighter 5 / Barbarian 15",
        "branch",
        "`compare` the branches",
        "`only_open: false`",
    ] {
        assert!(paragraph.contains(phrase), "no {phrase} in {paragraph}");
    }
}

#[tokio::test]
async fn the_glossary_is_a_resource() {
    let client = Client::start().await;
    let capabilities = client.capabilities();
    assert!(capabilities.resources.is_some(), "{capabilities:?}");

    let resources = client.resources().await;
    let uris: Vec<&str> = resources
        .iter()
        .map(|resource| resource.uri.as_str())
        .collect();
    assert_eq!(uris, ["wotr://glossary"]);
    assert_eq!(resources[0].mime_type.as_deref(), Some("text/markdown"));

    let read = client
        .read_resource("wotr://glossary")
        .await
        .expect("the glossary reads");
    let [ResourceContents::TextResourceContents { text, uri, .. }] = &read.contents[..] else {
        panic!("one text: {read:?}");
    };
    assert_eq!(uri, "wotr://glossary");
    // The text embedded at build time is the repo's GLOSSARY.md.
    let file = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../GLOSSARY.md"))
        .expect("GLOSSARY.md reads");
    assert_eq!(text, &file);
    assert!(text.contains("**Withdrawal**"), "the glossary's terms");
}

#[tokio::test]
async fn an_unknown_resource_is_not_found() {
    let client = Client::start().await;
    let read = client.read_resource("wotr://nothing").await;
    assert!(read.is_err(), "{read:?}");
}
