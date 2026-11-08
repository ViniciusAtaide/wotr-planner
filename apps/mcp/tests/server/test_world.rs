//! The server on `rules::test_world`, the engine tests' small invented world, rather than the real data.

use mcp::Planner;
use rmcp::serde_json::json;

use crate::support::*;

async fn on_the_test_world() -> Client {
    let planner = Planner::from_data(rules::test_world::world()).expect("the test world checks");
    Client::start_on(planner).await
}

#[tokio::test]
async fn the_server_searches_and_describes_the_test_world() {
    let client = on_the_test_world().await;
    // The Seeker exists only in the test world.
    assert_eq!(
        client.search_for("seeker").await.first(),
        Some(&("class".to_owned(), "seeker".to_owned()))
    );

    let seeker = client
        .ok("describe", json!({ "kind": "class", "key": "seeker" }))
        .await;
    assert_eq!(seeker["name"], "Seeker");
    assert_eq!(seeker["levels"].as_array().unwrap().len(), 20);
    assert_eq!(seeker["levels"][0]["selections"][0]["key"], "seeker/creed");
}
