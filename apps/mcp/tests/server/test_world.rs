//! The server on `rules::test_world`, the engine tests' small invented world, rather than the real data.

use mcp::Planner;
use rmcp::serde_json::json;

use crate::support::*;

#[tokio::test]
async fn the_server_searches_and_describes_the_test_world() {
    let client = Client::on_the_test_world().await;
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

/// Ticket 239: an entry with no guide link shows no `guide_url`, in `describe` and `search`.
#[tokio::test]
async fn an_entry_without_a_guide_link_shows_none() {
    let linked = Client::on_the_test_world().await;
    let seeker = linked
        .ok("describe", json!({ "kind": "class", "key": "seeker" }))
        .await;
    assert!(seeker["guide_url"].is_string(), "{seeker}");

    let mut data = rules::test_world::world();
    for class in &mut data.classes {
        class.guide = None;
    }
    let planner = Planner::from_data(data).expect("the test world checks");
    let bare = Client::start_on(planner).await;
    let seeker = bare
        .ok("describe", json!({ "kind": "class", "key": "seeker" }))
        .await;
    assert_eq!(seeker["name"], "Seeker");
    assert_eq!(seeker.get("guide_url"), None, "{seeker}");
}
