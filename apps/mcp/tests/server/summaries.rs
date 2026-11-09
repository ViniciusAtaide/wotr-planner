//! Ticket 112's MCP part: an item's short summary, where the data has one, in `search`'s matches, `options`' lines
//! and everything `describe` lists. The test world has no summaries, so these tests give some of its items one.

use rmcp::serde_json::{Value, json};
use rules::GameData;
use rules::test_world::build;

use crate::support::*;

const POWER_ATTACK: &str = "Trade attack for damage.";
const FIGHTER: &str = "A martial class with many bonus feats.";
const ARMIGER: &str = "A fighter who serves an order.";
const HUMAN: &str = "A bonus feat and skill points.";
const AEON: &str = "The path of balance.";
const SNEAK_ATTACK: &str = "Extra damage against a flanked foe.";
const FAST_STEALTH: &str = "Move at full speed while sneaking.";
const ROGUE_TALENTS: &str = "A talent at every even rogue level.";

/// The test world with summaries for a feat, a class, an archetype, a race, a path, a class feature, a Feature
/// selection and one of its options; every other item's stays empty.
fn summarised_world() -> GameData {
    let mut data = rules::test_world::world();
    let set = |summary: &mut String, text: &str| *summary = text.to_owned();
    let feat = data
        .feats
        .iter_mut()
        .find(|feat| feat.key.as_str() == "power-attack")
        .expect("power-attack");
    set(&mut feat.summary, POWER_ATTACK);
    let fighter = data
        .classes
        .iter_mut()
        .find(|class| class.key.as_str() == "fighter")
        .expect("fighter");
    set(&mut fighter.summary, FIGHTER);
    set(&mut fighter.archetypes[0].summary, ARMIGER);
    let human = data
        .races
        .iter_mut()
        .find(|race| race.key.as_str() == "human")
        .expect("human");
    set(&mut human.summary, HUMAN);
    let aeon = data
        .mythic_paths
        .iter_mut()
        .find(|path| path.key.as_str() == "aeon")
        .expect("aeon");
    set(&mut aeon.summary, AEON);
    let rogue = data
        .classes
        .iter_mut()
        .find(|class| class.key.as_str() == "rogue")
        .expect("rogue");
    let sneak_attack = rogue
        .features
        .iter_mut()
        .find(|feature| feature.key.as_str() == "rogue/sneak-attack")
        .expect("sneak attack");
    set(&mut sneak_attack.summary, SNEAK_ATTACK);
    let fast_stealth = rogue.selections[0]
        .options
        .iter_mut()
        .find(|option| option.key.as_str() == "rogue/rogue-talents/fast-stealth")
        .expect("fast stealth");
    set(&mut fast_stealth.summary, FAST_STEALTH);
    set(&mut rogue.selections[0].summary, ROGUE_TALENTS);
    data
}

async fn client() -> Client {
    Client::start_on(mcp::Planner::from_data(summarised_world()).expect("the world checks")).await
}

/// The item of a list of `{key, …}` with this key.
fn keyed<'a>(list: &'a Value, key: &str) -> &'a Value {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .find(|item| item["key"] == key)
        .unwrap_or_else(|| panic!("no {key} in {list}"))
}

#[tokio::test]
async fn search_matches_carry_their_summary_when_it_isnt_empty() {
    let client = client().await;
    for (query, summary) in [
        ("power-attack", POWER_ATTACK),
        ("fighter", FIGHTER),
        ("armiger", ARMIGER),
        ("human", HUMAN),
        ("aeon", AEON),
        ("fast-stealth", FAST_STEALTH),
    ] {
        let found = client
            .ok("search", json!({ "query": query, "limit": 1 }))
            .await;
        assert_eq!(found["matches"][0]["summary"], summary, "{query}: {found}");
    }
    let found = client
        .ok("search", json!({ "query": "cleave", "limit": 1 }))
        .await;
    assert_eq!(found["matches"][0]["key"], "cleave");
    assert!(found["matches"][0].get("summary").is_none(), "{found}");
}

#[tokio::test]
async fn describe_gives_the_summaries_of_class_features_and_selection_options() {
    let client = client().await;
    let rogue = client
        .ok("describe", json!({ "kind": "class", "key": "rogue" }))
        .await;
    let sneak_attack = rogue["levels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row["features"].as_array().unwrap())
        .find(|feature| feature["key"] == "rogue/sneak-attack")
        .unwrap_or_else(|| panic!("no sneak attack in {rogue}"));
    assert_eq!(sneak_attack["summary"], SNEAK_ATTACK);
    assert_eq!(rogue["selections"][0]["summary"], ROGUE_TALENTS);
    let talents = &rogue["selections"][0]["options"];
    assert_eq!(
        keyed(talents, "rogue/rogue-talents/fast-stealth")["summary"],
        FAST_STEALTH
    );
    assert!(
        keyed(talents, "rogue/rogue-talents/combat-trick")
            .get("summary")
            .is_none(),
        "{talents}"
    );

    // A selection option and a class described on their own.
    let option = client
        .ok(
            "describe",
            json!({ "kind": "option", "key": "rogue/rogue-talents/fast-stealth" }),
        )
        .await;
    assert_eq!(option["summary"], FAST_STEALTH);
    let fighter = client
        .ok("describe", json!({ "kind": "class", "key": "fighter" }))
        .await;
    assert_eq!(fighter["summary"], FIGHTER);
}

#[tokio::test]
async fn option_lines_carry_their_summary_when_it_isnt_empty() {
    let client = client().await;
    let fighter = code(&build("human", &["fighter"]));
    let options = |choice: &str, values: bool| json!({ "build": fighter, "choice": choice, "only_open": false, "values": values });
    let line = |list: &Value, value: Value| -> Value {
        list["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|line| line["value"] == value)
            .unwrap_or_else(|| panic!("no {value} in {list}"))
            .clone()
    };

    let feats = client
        .ok("options", options("slot.feat.1.general.0", false))
        .await;
    assert_eq!(line(&feats, json!("power-attack"))["summary"], POWER_ATTACK);
    assert!(line(&feats, json!("cleave")).get("summary").is_none());

    let classes = client.ok("options", options("level.2.class", false)).await;
    assert_eq!(
        line(&classes, json!({ "class": "rogue" })).get("summary"),
        None
    );
    let empty = code(&build("human", &[]));
    let first = client
        .ok(
            "options",
            json!({ "build": empty, "choice": "level.1.class", "only_open": false }),
        )
        .await;
    assert_eq!(
        line(&first, json!({ "class": "fighter" }))["summary"],
        FIGHTER
    );
    assert_eq!(
        line(
            &first,
            json!({ "class": "fighter", "archetype": "fighter/armiger" })
        )["summary"],
        ARMIGER
    );

    let races = client
        .ok(
            "options",
            json!({ "build": empty, "choice": "start.race", "only_open": false }),
        )
        .await;
    assert_eq!(line(&races, json!("human"))["summary"], HUMAN);
}
