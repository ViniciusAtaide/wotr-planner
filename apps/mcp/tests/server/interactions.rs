//! Interactions notes (ticket 124): `describe` gives an item's note on how it combines with others, for the five
//! items the Max DC run couldn't settle, and none for an item without one.

use rmcp::serde_json::json;

use crate::support::*;

#[tokio::test]
async fn describe_gives_the_dc_runs_items_their_interactions_notes() {
    let client = Client::start().await;
    for (kind, key, says) in [
        ("option", "azata/superpowers/favorable-magic", "doesn't say"),
        ("feat", "metamagic-persistent-spell", "doesn't say"),
        ("mythic_feat", "expanded-arsenal", "Greater Spell Focus"),
        ("mythic_ability", "second-bloodline", "arcana"),
        ("mythic_ability", "bloodline-ascendance", "doesn't say"),
    ] {
        let item = client
            .ok("describe", json!({ "kind": kind, "key": key }))
            .await;
        let note = text(&item["interactions"]);
        assert!(note.contains(says), "{key}: {note}");
    }

    let plain = client
        .ok("describe", json!({ "kind": "feat", "key": "power-attack" }))
        .await;
    assert!(plain.get("interactions").is_none(), "{plain}");
}
