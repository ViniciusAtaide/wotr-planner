//! Ticket 125 on the real data: Favorite Metamagic's metamagic is a value `options` lists and `choose` takes, and a
//! code made before it took one opens with the metamagic open in `todo`.

use rmcp::serde_json::{Value, json};
use rules::{Build, ClassChoice, ClassKey, LevelUp, MythicAbilityKey, Pick, SlotId};

use crate::support::*;

/// A fighter 20 with Favorite Metamagic at MR 1 and no metamagic: what every code before ticket 125 held.
fn older_favorite_metamagic() -> Build {
    let mut build = Build::default();
    build
        .class_choices
        .insert(ClassKey::new("fighter"), ClassChoice::default());
    for _ in 0..20 {
        build.levels.push(LevelUp::new(ClassKey::new("fighter")));
    }
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(MythicAbilityKey::new("favorite-metamagic")),
    );
    build
}

fn persistent() -> Value {
    json!({ "key": "favorite-metamagic", "parameter": "metamagic-persistent-spell" })
}

#[tokio::test]
async fn real_data_an_older_favorite_metamagic_opens_with_its_metamagic_in_todo() {
    let client = Client::start().await;
    let older = code(&older_favorite_metamagic());
    let sheet = client.ok("sheet", json!({ "build": older })).await;
    let open = at(&sheet["todo"], "slot.mythic.1");
    assert_eq!(
        open["label"],
        "MR 1: Favorite Metamagic: choose a metamagic"
    );
}

#[tokio::test]
async fn real_data_favorite_metamagic_lists_each_metamagic_and_takes_one() {
    let client = Client::start().await;
    let older = code(&older_favorite_metamagic());
    let listed = client
        .ok(
            "options",
            json!({ "build": older, "choice": "slot.mythic.1", "values": true, "query": "favorite" }),
        )
        .await;
    let options = listed["options"].as_array().unwrap();
    assert_eq!(options.len(), 12, "{listed}");
    let offered = options
        .iter()
        .find(|option| option["value"] == persistent())
        .unwrap_or_else(|| panic!("no Persistent Spell in {listed}"));
    assert_eq!(offered["status"], "ok");
    assert_eq!(offered["name"], "Favorite Metamagic (Persistent Spell)");

    let chosen = client
        .ok(
            "choose",
            json!({ "build": older, "steps": [{ "choice": "slot.mythic.1", "value": persistent() }] }),
        )
        .await;
    assert_eq!(chosen["applied"], 1, "{chosen}");
    assert!(!choices(&chosen["todo"]).contains(&"slot.mythic.1".to_owned()));
}

#[tokio::test]
async fn real_data_favorite_metamagic_by_its_key_alone_needs_its_metamagic() {
    let client = Client::start().await;
    let mut empty = older_favorite_metamagic();
    empty.picks.clear();
    let chosen = client
        .ok(
            "choose",
            json!({ "build": code(&empty), "steps": [{ "choice": "slot.mythic.1", "value": "favorite-metamagic" }] }),
        )
        .await;
    assert_eq!(chosen["applied"], 0, "{chosen}");
    assert_eq!(chosen["rejected"]["issue"], "needs_parameter", "{chosen}");
}
