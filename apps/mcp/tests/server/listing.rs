//! What a client sees on connecting: the tools, the server's instructions and its two resources, the glossary and the
//! build guide.

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
            "improve",
            "new_build",
            "options",
            "search",
            "sheet",
            "validate",
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
async fn the_instructions_point_to_the_build_guide_and_stay_short() {
    let instructions = Client::start().await.instructions();
    // A client cut the old 9,656-character instructions at about 4,500, honesty boundary and all.
    assert!(
        instructions.len() < 4_000,
        "{} characters",
        instructions.len()
    );
    for phrase in [
        "wotr://build-guide",
        "`wotr-build` skill",
        "`search`",
        "ask the user",
        "`new_build`",
        "`validate`",
        "`rejected`",
        "final `link`",
        "wotr://glossary",
    ] {
        assert!(
            instructions.contains(phrase),
            "no {phrase} in {instructions}"
        );
    }
}

#[tokio::test]
async fn the_guide_gives_the_workflow() {
    let client = Client::start().await;
    let workflow = client.guide_section("Workflow").await;
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
            workflow
                .find(step)
                .unwrap_or_else(|| panic!("no {step} in {workflow}"))
        })
        .collect();
    assert!(at.is_sorted(), "{steps:?} at {at:?}");
    // Batching, values, branching and the glossary.
    let guide = client.guide().await;
    for phrase in [
        "`rejected`",
        "`options {values: true}`",
        "`advances`",
        "number to reach",
        "`compare`",
        "`validate`",
        "wotr://glossary",
    ] {
        assert!(guide.contains(phrase), "no {phrase} in {guide}");
    }
}

#[tokio::test]
async fn the_guide_says_what_to_know_before_planning() {
    let client = Client::start().await;
    let guide = client.guide().await;
    let before = guide.find("## Before you plan").expect("the section");
    let workflow = guide.find("## Workflow").expect("the workflow");
    assert!(before < workflow, "{guide}");
    let paragraph = client.guide_section("Before you plan").await;
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
    assert!(guide.contains("`{skill: ranks}`"), "{guide}");
}

#[tokio::test]
async fn the_guide_gives_the_pick_order_within_a_level() {
    // Ticket 126: a pick that needs another taken at the same level goes in the later slot.
    let paragraph = Client::start().await.guide_section("Before you plan").await;
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
async fn the_guide_says_to_compare_class_splits_before_committing() {
    // Ticket 115, option A: no `reach` tool; branch the splits and `compare` them first.
    let paragraph = Client::start().await.guide_section("Before you plan").await;
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
    assert_eq!(uris, ["wotr://glossary", "wotr://build-guide"]);
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
async fn the_build_guide_is_the_skill_without_its_frontmatter() {
    let client = Client::start().await;
    let guide = client.guide().await;
    // One text for Claude Code's `wotr-build` skill and for any client reading the resource.
    let file = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../.claude/skills/wotr-build/SKILL.md"
    ))
    .expect("the skill reads");
    assert!(file.starts_with("---\nname: wotr-build\n"), "{file}");
    assert!(file.ends_with(&guide), "{guide}");
    assert!(guide.starts_with("# Building with wotr-planner"), "{guide}");
    assert!(!guide.contains("description:"), "{guide}");
    for section in [
        "Steps",
        "Before you plan",
        "Workflow",
        "The honesty boundary",
    ] {
        client.guide_section(section).await;
    }
}

#[tokio::test]
async fn an_unknown_resource_is_not_found() {
    let client = Client::start().await;
    let read = client.read_resource("wotr://nothing").await;
    assert!(read.is_err(), "{read:?}");
}

#[tokio::test]
async fn the_guides_point_buy_costs_are_the_engines() {
    let guide = Client::start().await.guide().await;
    let costs: Vec<String> = (7..=18u8)
        .map(|score| {
            let spent = rules::PointBuy::default()
                .with(rules::Ability::Strength, score)
                .expect("a score from 7 to 18 fits")
                .cost();
            format!("{score} = {spent}")
        })
        .collect();
    let line = costs.join(", ");
    assert!(guide.contains(&line), "no {line} in the guide");
}
