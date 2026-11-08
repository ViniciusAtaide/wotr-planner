//! What a client sees on connecting: the tools and the server's instructions.

use crate::support::*;

#[tokio::test]
async fn the_server_offers_search_and_describe() {
    let client = Client::start().await;
    assert_eq!(client.tool_names().await, ["describe", "search"]);
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
