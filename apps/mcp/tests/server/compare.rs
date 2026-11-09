//! `compare`: Builds side by side, then only what differs between them, read from their sheets.

use rmcp::serde_json::{Value, json};
use rules::test_world::{build, general_slot, pick_feat, times};
use rules::{Build, MythicPathKey};

use crate::support::*;

/// A human fighter of `levels` levels.
fn fighter(levels: usize) -> Build {
    build("human", &times("fighter", levels))
}

async fn compare(client: &Client, builds: &[&Build]) -> Value {
    let codes: Vec<String> = builds.iter().map(|build| code(build)).collect();
    client.ok("compare", json!({ "builds": codes })).await
}

/// The two branches of a fighter 9 (MR 3, the path's rank), one per path with its MR 3 pick, through `choose` as an
/// agent branches.
async fn branches(client: &Client) -> (Value, Value) {
    let trunk = code(&fighter(9));
    let mut branch = Vec::new();
    for (path, pick) in [
        ("trickster", "trickster/tricks/perception-1"),
        ("aeon", "aeon/enforcing-gaze/attack"),
    ] {
        let chosen = client
            .ok(
                "choose",
                json!({ "build": trunk, "steps": [
                    { "choice": "mythic.path", "value": path },
                    { "choice": "slot.path-selection.3.0", "value": pick },
                ] }),
            )
            .await;
        assert_eq!(chosen["applied"], 2, "{chosen}");
        branch.push(chosen["build"].clone());
    }
    let aeon = branch.pop().unwrap();
    (branch.pop().unwrap(), aeon)
}

/// A `names` list of a `PickDiff`: one name per Build, none `null`.
fn names_of(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(text)
        .collect()
}

fn names(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["name"]))
        .collect()
}

#[tokio::test]
async fn two_path_branches_differ_in_their_path_and_its_features() {
    let client = Client::on_the_test_world().await;
    let (trickster, aeon) = branches(&client).await;
    let compared = client
        .ok("compare", json!({ "builds": [trickster, aeon] }))
        .await;

    // Each Build's summary, in the order given.
    let builds = compared["builds"].as_array().unwrap();
    assert_eq!(builds.len(), 2);
    assert_eq!(builds[0]["build"], trickster);
    assert_eq!(builds[0]["path"]["name"], "Trickster");
    assert_eq!(builds[1]["path"]["name"], "Aeon");
    assert_eq!(builds[0]["character_level"], 9);
    assert_eq!(builds[0]["mythic_rank"], 3);
    assert_eq!(
        builds[0]["classes"],
        json!([{ "class": { "key": "fighter", "name": "Fighter" }, "levels": 9 }])
    );
    assert_eq!(builds[0]["bab"], builds[1]["bab"]);
    assert_eq!(builds[0]["level_cap"], 20);
    assert!(
        text(&builds[1]["link"]).ends_with(&format!("#b={}", text(&aeon))),
        "{}",
        builds[1]
    );

    // The levels are the same Level-ups: nothing differs there.
    assert_eq!(compared["levels"], json!([]), "{compared}");
    assert_eq!(compared["start"], json!([]));

    // With no first ascension of their own, MR 1–3 are each path's, and MR 3 holds the path's own pick.
    let ranks = compared["ranks"].as_array().unwrap();
    let rank_3 = ranks.iter().find(|rank| rank["rank"] == 3).unwrap();
    assert_eq!(rank_3["path"], json!(["Trickster", "Aeon"]));
    let pick = at(&rank_3["picks"], "slot.path-selection.3.0");
    assert_eq!(
        pick["label"], "Trickster Tricks / Enforcing Gaze",
        "{rank_3}"
    );
    let picked = names_of(&pick["names"]);
    assert_eq!(picked.len(), 2);
    assert_ne!(picked[0], picked[1], "{pick}");
    assert!(rank_3.get("status").is_none(), "{rank_3}");

    // The path features each has and the other lacks, by rank.
    let features = compared["path_features"].as_array().unwrap();
    let ranks_listed: Vec<u64> = features
        .iter()
        .map(|rank| rank["rank"].as_u64().unwrap())
        .collect();
    assert_eq!(ranks_listed, [1, 2, 3], "{compared}");
    let rank_1 = &features[0]["only"];
    assert_eq!(
        names(&rank_1[0]),
        ["Bit of Fun", "Bypass Epic Damage Reduction (Trickster)"]
    );
    assert_eq!(
        names(&rank_1[1]),
        ["Bypass Epic Damage Reduction (Aeon)", "Force Reality"]
    );
    assert_eq!(names(&features[2]["only"][0]), ["Trickster Tricks"]);
    assert_eq!(names(&features[2]["only"][1]), Vec::<String>::new());
}

#[tokio::test]
async fn a_shared_first_ascension_leaves_only_the_later_ranks_differing() {
    let client = Client::on_the_test_world().await;
    let mut trickster = fighter(9);
    trickster.first_ascension = Some(MythicPathKey::new("aeon"));
    trickster.mythic_path = Some(MythicPathKey::new("trickster"));
    let mut aeon = trickster.clone();
    aeon.mythic_path = Some(MythicPathKey::new("aeon"));
    let compared = compare(&client, &[&trickster, &aeon]).await;
    assert_eq!(
        compared["builds"][0]["first_ascension"]["name"], "Aeon",
        "{compared}"
    );
    let features = compared["path_features"].as_array().unwrap();
    assert_eq!(features.len(), 1, "{compared}");
    assert_eq!(features[0]["rank"], 3);
}

#[tokio::test]
async fn level_differences_list_the_picks_numbers_and_feats_that_differ() {
    let client = Client::on_the_test_world().await;
    let mut power = fighter(4);
    power
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    let mut dodge = fighter(5);
    dodge.picks.insert(general_slot(1), pick_feat("dodge"));
    let compared = compare(&client, &[&power, &dodge]).await;

    let levels = compared["levels"].as_array().unwrap();
    let first = &levels[0];
    assert_eq!(first["level"], 1);
    let pick = at(&first["picks"], "slot.feat.1.general.0");
    assert_eq!(pick["names"], json!(["power-attack", "dodge"]));
    assert_eq!(names(&first["feats"][0]), ["power-attack"]);
    assert_eq!(names(&first["feats"][1]), ["dodge"]);
    assert!(first.get("numbers").is_none(), "{first}");

    // Levels 2 to 4 are the same; level 5 only the second Build has.
    let listed: Vec<u64> = levels
        .iter()
        .map(|level| level["level"].as_u64().unwrap())
        .collect();
    assert_eq!(listed, [1, 5], "{compared}");
    let fifth = &levels[1];
    let bab = fifth["numbers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|number| number["what"] == "bab")
        .unwrap();
    assert_eq!(bab["values"], json!([null, 5]));
    assert_eq!(
        at(&fifth["picks"], "level.5.class")["names"],
        json!([null, "Fighter"])
    );
    assert_eq!(compared["builds"][0]["character_level"], 4);
    assert_eq!(compared["builds"][1]["character_level"], 5);
}

#[tokio::test]
async fn the_same_build_twice_differs_in_nothing() {
    let client = Client::on_the_test_world().await;
    let compared = compare(&client, &[&fighter(9), &fighter(9)]).await;
    for list in ["start", "levels", "ranks", "path_features"] {
        assert_eq!(compared[list], json!([]), "{list}: {compared}");
    }
}

#[tokio::test]
async fn one_build_gets_its_summary_and_no_differences() {
    let client = Client::on_the_test_world().await;
    let compared = compare(&client, &[&fighter(9)]).await;
    let summaries = compared["builds"].as_array().unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0]["build"], code(&fighter(9)));
    assert_eq!(summaries[0]["character_level"], 9);
    assert_eq!(summaries[0]["classes"][0]["levels"], 9);
    for list in ["start", "levels", "ranks", "path_features"] {
        assert_eq!(compared[list], json!([]), "{list}: {compared}");
    }
}

#[tokio::test]
async fn compare_takes_one_to_four_builds() {
    let client = Client::on_the_test_world().await;
    let none = client.call("compare", json!({ "builds": [] })).await;
    assert_eq!(error_code(&none), "invalid_arguments");
    assert!(error_text(&none).contains("1 to 4"), "{none:?}");
    let five = vec![code(&fighter(1)); 5];
    let five = client.call("compare", json!({ "builds": five })).await;
    assert_eq!(error_code(&five), "invalid_arguments");
    let four = vec![code(&fighter(1)); 4];
    let four = client.ok("compare", json!({ "builds": four })).await;
    assert_eq!(four["builds"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn a_code_that_doesnt_open_is_named() {
    let client = Client::on_the_test_world().await;
    let result = client
        .call(
            "compare",
            json!({ "builds": [code(&fighter(1)), "not-a-code"] }),
        )
        .await;
    assert_eq!(error_code(&result), "stale_link");
    assert!(error_text(&result).contains("builds[1]"), "{result:?}");
}
