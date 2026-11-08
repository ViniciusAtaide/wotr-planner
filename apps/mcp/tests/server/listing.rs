//! What a client sees on connecting: the tools and the server's instructions.

use crate::support::*;

#[tokio::test]
async fn the_server_offers_its_tools() {
    let client = Client::start().await;
    assert_eq!(
        client.tool_names().await,
        [
            "choose",
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
