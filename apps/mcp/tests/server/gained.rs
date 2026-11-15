//! A feature gained more than once at a level says how many times (ticket 192, review): the Trickster's third Mythic
//! rank gives Sneak Attack twice. A feature held more than once says its rank (ticket 243), and one a level raises
//! without a grant says its step (ticket 254). On the real data.

use rmcp::serde_json::{Map, Value, json};

use crate::support::*;

/// A human fighter of nine levels, with the Trickster's path when `path`: MR 3 comes at level 9.
async fn fighter_9(client: &Client, path: bool) -> Value {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    if path {
        choices.insert("mythic.path".to_owned(), json!("trickster"));
    }
    for level in 1..=9 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "fighter" }),
        );
    }
    client.ok("validate", json!({ "choices": choices })).await["build"].clone()
}

fn feature<'a>(list: &'a Value, key: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|feature| feature["key"] == key)
        .unwrap_or_else(|| panic!("no {key} in {list}"))
}

#[tokio::test]
async fn real_data_the_sheet_says_a_feature_gained_twice_at_a_level_twice() {
    let client = Client::start().await;
    let build = fighter_9(&client, true).await;
    let sheet = client.ok("sheet", json!({ "build": build })).await;
    // Level 9 gains Sneak Attack twice, and each of the others once (no `times`). Its rank, 2, is the times it is
    // gained, so the line leaves it out.
    let level = &sheet["levels"][8]["features"];
    assert_eq!(
        feature(level, "trickster/sneak-attack"),
        &json!({ "key": "trickster/sneak-attack", "name": "Sneak Attack (Trickster)", "times": 2 })
    );
    assert_eq!(
        feature(level, "trickster/sneaky-quack"),
        &json!({ "key": "trickster/sneaky-quack", "name": "Sneaky Quack" })
    );
    // So does MR 3's own row: one entry, not the same entry twice.
    let rank = &sheet["ranks"][2]["features"];
    assert_eq!(rank.as_array().unwrap().len(), 3, "{rank}");
    assert_eq!(feature(rank, "trickster/sneak-attack")["times"], 2);
}

#[tokio::test]
async fn real_data_compare_sees_a_feature_gained_twice_against_none() {
    let client = Client::start().await;
    let with = fighter_9(&client, true).await;
    let without = fighter_9(&client, false).await;
    let compared = client
        .ok("compare", json!({ "builds": [with, without] }))
        .await;
    let rank = compared["path_features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diff| diff["rank"] == 3)
        .unwrap_or_else(|| panic!("no rank 3 in {compared}"));
    // The Trickster's list carries the count; the other Build has none of them.
    assert_eq!(
        feature(&rank["only"][0], "trickster/sneak-attack")["times"],
        2
    );
    assert_eq!(rank["only"][1], json!([]));
}

/// Fey Magic is one feature the Feyspeaker gains again every two levels from 4: the level that grants it a second time
/// says rank 2, the first says none. Game: FeyspeakerArchetype da69747aa3dd0044ebff5f3d701cdde3 adds
/// FeyspeakerMagicSelection 237efc6a131c8174bbd466ed239f21fc at 4, 6, 8, …, 18. The rank is the planner's count of
/// grants: the game's stops at the blueprint's `Ranks` (`Feature.AddRank`), which is 1 here, so its own stays 1.
#[tokio::test]
async fn real_data_the_sheet_says_the_rank_of_a_feature_gained_again() {
    let client = Client::start().await;
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=6 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "druid", "archetype": "druid/feyspeaker" }),
        );
    }
    let build = client.ok("validate", json!({ "choices": choices })).await["build"].clone();
    let sheet = client.ok("sheet", json!({ "build": build })).await;
    let fey_magic = |level: usize| {
        feature(
            &sheet["levels"][level - 1]["features"],
            "druid/feyspeaker/fey-magic",
        )
        .clone()
    };
    assert_eq!(
        fey_magic(4),
        json!({ "key": "druid/feyspeaker/fey-magic", "name": "Fey Magic (Feyspeaker)" })
    );
    assert_eq!(
        fey_magic(6),
        json!({ "key": "druid/feyspeaker/fey-magic", "name": "Fey Magic (Feyspeaker)", "rank": 2 })
    );
}

/// Infernal Resistance is granted once, at bloodrager 4, and rises at bloodrager 5 and 8 with no grant (ticket 254):
/// levels 5 and 8 list it with their step and no `times`. Game: BloodragerInfernalBloodline 9aef64b53406f114cb43f898a3aec01e grants
/// BloodragerInfernalResistance ac77b22f74aed034ca2b16ba6a4bfde9 at 4 only; BloodragerInfernalResistanceBuff
/// 66c973ccad1293345ae6c5bfeced4a7b has two ContextRankConfigs on BloodragerClass level: the resistance's table, 5 to
/// 7 and 10 from 8, and the poison-save bonus's (DamageDice, custom {4 -> 2, 8 -> 4}), 2 to 4 and 4 from 5
/// (`ContextRankConfig.ApplyProgression` Custom: the first entry with value <= base, WrathDecompiled 6ce4a6b).
#[tokio::test]
async fn real_data_the_sheet_says_the_step_a_level_raises_a_feature_to() {
    let client = Client::start().await;
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("chaotic-neutral")),
        ("slot.selection.1.0", json!("bloodrager/bloodline/infernal")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=8 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "bloodrager" }),
        );
    }
    let build = client.ok("validate", json!({ "choices": choices })).await["build"].clone();
    let sheet = client.ok("sheet", json!({ "build": build })).await;
    const KEY: &str = "bloodrager/bloodline/infernal/infernal-resistance";
    let resistance = |level: usize| feature(&sheet["levels"][level - 1]["features"], KEY).clone();
    let name = "Infernal Resistance";
    assert_eq!(resistance(4), json!({ "key": KEY, "name": name }));
    assert_eq!(
        resistance(5),
        json!({ "key": KEY, "name": name, "step": 2 })
    );
    assert_eq!(
        resistance(8),
        json!({ "key": KEY, "name": name, "step": 3 })
    );
}

/// A feature granted after its step level is gained, with no `step`: the Second Bloodrager Bloodline (MR 1, after
/// bloodrager 6) grants Infernal Resistance when bloodrager 5 has passed, and the sheet lists it as gained there, its
/// step 2 reached at 8 only when it is already held before. Game: as above, on BloodragerClass level.
#[tokio::test]
async fn real_data_a_feature_granted_after_its_step_level_is_gained_with_no_step() {
    let client = Client::start().await;
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("chaotic-neutral")),
        ("slot.selection.1.0", json!("bloodrager/bloodline/abyssal")),
        ("slot.mythic.1", json!("second-bloodrager-bloodline")),
        (
            "slot.mythic-selection.1.0",
            json!("bloodrager/bloodline/infernal"),
        ),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=8 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "bloodrager" }),
        );
    }
    let validated = client.ok("validate", json!({ "choices": choices })).await;
    let sheet = client
        .ok("sheet", json!({ "build": validated["build"].clone() }))
        .await;
    const KEY: &str = "bloodrager/bloodline/infernal/infernal-resistance";
    let levels = sheet["levels"].as_array().unwrap();
    let steps: Vec<(usize, Value)> = levels
        .iter()
        .enumerate()
        .filter_map(|(index, level)| {
            level["features"]
                .as_array()?
                .iter()
                .find(|feature| feature["key"] == KEY)
                .map(|feature| (index + 1, feature.clone()))
        })
        .collect();
    let name = "Infernal Resistance";
    assert_eq!(
        steps,
        [
            (6, json!({ "key": KEY, "name": name })),
            (8, json!({ "key": KEY, "name": name, "step": 3 })),
        ],
        "{validated}"
    );
}
