//! `sheet`: a Build level by level and rank by rank, each choice by its address.

use rmcp::serde_json::{Map, Value, json};
use rules::test_world::{
    build, class_slot, general_slot, pick_feat, pick_feat_for, race_slot, times, weapon,
};
use rules::{Ability, Build};

use crate::support::*;

/// A human fighter 4: Power Attack in level 1's Combat feat slot, Cleave in its general slot, Toughness in the
/// human's bonus slot, Weapon Focus (Longsword) at 2, Dodge at 3, +1 Strength at 4, and level 4's Combat feat slot
/// left open.
fn fighter() -> Build {
    let mut fighter = build("human", &times("fighter", 4));
    let picks = [
        (class_slot(1, 0), pick_feat("power-attack")),
        (general_slot(1), pick_feat("cleave")),
        (race_slot(0), pick_feat("toughness")),
        (
            class_slot(2, 0),
            pick_feat_for("weapon-focus", weapon("longsword")),
        ),
        (general_slot(3), pick_feat("dodge")),
    ];
    fighter.picks.extend(picks);
    fighter.levels[3].ability_increase = Some(Ability::Strength);
    fighter
}

async fn sheet_of(build: &Build, removable: bool) -> Value {
    Client::on_the_test_world()
        .await
        .ok(
            "sheet",
            json!({ "build": code(build), "removable": removable }),
        )
        .await
}

async fn sheet_with_todo(build: &Build) -> Value {
    Client::on_the_test_world()
        .await
        .ok("sheet", json!({ "build": code(build), "todo": true }))
        .await
}

fn names(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["name"]))
        .collect()
}

#[tokio::test]
async fn the_sheet_shows_each_level_with_its_numbers_and_picks() {
    let sheet = sheet_of(&fighter(), false).await;
    assert_eq!(sheet["build"], code(&fighter()));
    let levels = sheet["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 4);

    let first = &levels[0];
    assert_eq!(first["level"], 1);
    assert_eq!(
        first["class"],
        json!({ "key": "fighter", "name": "Fighter" })
    );
    assert_eq!(first["class_level"], 1);
    assert_eq!(
        (first["bab"].clone(), first["fort"].clone()),
        (json!(1), json!(2))
    );
    // The class first, then the slots in the order they're checked: the class's, the race's, the general one.
    assert_eq!(
        choices(&first["picks"]),
        [
            "level.1.class",
            "slot.feat.1.class.0",
            "slot.feat.1.race.0",
            "slot.feat.1.general.0"
        ]
    );
    assert_eq!(
        at(&first["picks"], "level.1.class")["value"],
        json!({ "class": "fighter" })
    );
    let combat = at(&first["picks"], "slot.feat.1.class.0");
    assert_eq!(combat["label"], "Combat feat");
    assert_eq!(combat["value"], "power-attack");
    assert_eq!(
        at(&first["picks"], "slot.feat.1.general.0")["label"],
        "Feat"
    );
    // Granted and picked feats alike are the level's new feats.
    let feats = names(&first["feats"]);
    for feat in ["shield-proficiency", "power-attack", "cleave", "toughness"] {
        assert!(feats.contains(&feat.to_owned()), "{feats:?}");
    }
    // A human fighter with Int 13: 2 + 1 + the human's 1.
    assert_eq!(first["skill_points"], json!({ "available": 4, "spent": 0 }));

    // A parameterised pick's value names its parameter.
    let second = &levels[1];
    let focus = at(&second["picks"], "slot.feat.2.class.0");
    assert_eq!(
        focus["value"],
        json!({ "key": "weapon-focus", "parameter": "longsword" })
    );
    assert_eq!(focus["name"], "weapon-focus (longsword)");
    assert_eq!(names(&second["feats"]), ["weapon-focus (longsword)"]);
    assert_eq!(names(&second["features"]), ["Bravery"]);

    // Level 4: the increase, then the open Combat feat slot.
    let fourth = &levels[3];
    assert_eq!(fourth["bab"], 4);
    assert_eq!(fourth["scores"]["str"], 17);
    let increase = at(&fourth["picks"], "level.4.increase");
    assert_eq!(
        (increase["value"].clone(), increase["name"].clone()),
        (json!("str"), json!("+1 Strength"))
    );
    let open = at(&fourth["picks"], "slot.feat.4.class.0");
    assert_eq!(
        (open["value"].clone(), open["name"].clone()),
        (Value::Null, Value::Null)
    );
}

#[tokio::test]
async fn the_sheet_shows_the_build_start_and_what_is_left_to_do() {
    let sheet = sheet_with_todo(&fighter()).await;
    let start = &sheet["start"];
    assert_eq!(at(start, "start.race")["value"], "human");
    assert_eq!(at(start, "start.faith")["value"], "atheism");
    assert_eq!(at(start, "start.alignment")["value"], "true-neutral");
    assert_eq!(at(start, "start.score.int")["value"], 13);
    assert_eq!(at(start, "start.racial-ability")["value"], Value::Null);
    assert_eq!(sheet["point_buy_spent"], 18);

    assert_eq!(
        choices(&sheet["todo"]),
        [
            "start.racial-ability",
            "start.background",
            "level.1.skill",
            "level.2.skill",
            "level.3.skill",
            "level.4.skill",
            "slot.feat.4.class.0",
        ]
    );
    assert_eq!(
        at(&sheet["todo"], "slot.feat.4.class.0")["label"],
        "Level 4: Combat feat"
    );
    // No Mythic rank before level 6.
    let ranks = sheet["ranks"].as_array().unwrap();
    assert_eq!(ranks.len(), 10);
    assert!(ranks.iter().all(|rank| rank["status"] == "not_reached"));
}

#[tokio::test]
async fn the_mythic_track_opens_its_choices_as_the_ranks_are_reached() {
    // Fighter 9 reaches MR 3: the first ascension and the path are open, and each rank's mythic pick.
    let sheet = sheet_with_todo(&build("human", &times("fighter", 9))).await;
    let ranks = sheet["ranks"].as_array().unwrap();
    assert_eq!(ranks[0]["status"], "reached");
    assert_eq!(ranks[0]["character_level"], 6);
    assert_eq!(choices(&ranks[0]["picks"]), ["slot.mythic.1"]);
    assert_eq!(ranks[0]["picks"][0]["label"], "Mythic ability");
    assert_eq!(ranks[3]["status"], "not_reached");
    assert_eq!(sheet["levels"][5]["mythic_ranks"], json!([1]));

    let todo = choices(&sheet["todo"]);
    let mythic: Vec<&str> = todo
        .iter()
        .map(String::as_str)
        .skip_while(|choice| !choice.starts_with("mythic.") && !choice.starts_with("slot.mythic"))
        .collect();
    assert_eq!(
        mythic,
        [
            "mythic.first-ascension",
            "slot.mythic.1",
            "slot.mythic.2",
            "mythic.path",
            "slot.mythic.3"
        ]
    );
}

#[tokio::test]
async fn removable_marks_the_choices_that_can_come_off_now() {
    let sheet = sheet_of(&fighter(), true).await;
    let levels = &sheet["levels"];
    // Cleave needs Power Attack, so Power Attack stays until Cleave goes.
    assert_eq!(
        at(&levels[0]["picks"], "slot.feat.1.class.0")["removable"],
        false
    );
    assert_eq!(
        at(&levels[0]["picks"], "slot.feat.1.general.0")["removable"],
        true
    );
    assert_eq!(
        at(&levels[2]["picks"], "slot.feat.3.general.0")["removable"],
        true
    );
    // An open choice has nothing to withdraw.
    assert!(
        at(&levels[3]["picks"], "slot.feat.4.class.0")
            .get("removable")
            .is_none()
    );

    let removable: Vec<String> = sheet["removable"]
        .as_array()
        .unwrap()
        .iter()
        .map(text)
        .collect();
    for choice in ["level.4", "slot.feat.1.general.0", "level.4.increase"] {
        assert!(removable.contains(&choice.to_owned()), "{removable:?}");
    }
    assert!(!removable.contains(&"slot.feat.1.class.0".to_owned()));
    // A level's class isn't withdrawn (`choose` changes it), so it's neither listed nor marked.
    assert!(!removable.contains(&"level.4.class".to_owned()));
    assert!(
        at(&levels[3]["picks"], "level.4.class")
            .get("removable")
            .is_none()
    );

    // Without the flag, no marks.
    let plain = sheet_of(&fighter(), false).await;
    assert!(plain.get("removable").is_none());
    assert!(
        at(&plain["levels"][0]["picks"], "slot.feat.1.general.0")
            .get("removable")
            .is_none()
    );
}

#[tokio::test]
async fn a_full_sheet_leaves_out_what_did_not_change_since_the_level_before() {
    let sheet = sheet_of(&fighter(), false).await;
    let levels = sheet["levels"].as_array().unwrap();

    // The first row says everything.
    for field in ["class", "scores", "skill_points", "attack", "picks"] {
        assert!(levels[0].get(field).is_some(), "{field}: {}", levels[0]);
    }
    assert_eq!(levels[0]["attack"]["weapon"]["key"], "longsword");
    assert!(levels[0]["attack"].get("counts").is_some());
    // Level 2 changed no score and no class: they go, and so does the attack line's weapon.
    assert!(levels[1].get("class").is_none(), "{}", levels[1]);
    assert!(levels[1].get("archetype").is_none());
    assert!(levels[1].get("scores").is_none(), "{}", levels[1]);
    assert!(levels[1]["attack"].get("weapon").is_none());
    assert!(levels[1]["attack"].get("critical").is_none());
    // The numbers that move every level are always given.
    for row in levels {
        for field in [
            "level",
            "class_level",
            "bab",
            "fort",
            "reflex",
            "will",
            "picks",
        ] {
            assert!(row.get(field).is_some(), "{field}: {row}");
        }
        for field in ["attacks", "damage"] {
            assert!(row["attack"].get(field).is_some(), "{field}: {row}");
        }
    }
    // The +1 Strength level gives all six scores.
    let scores = levels[3]["scores"].as_object().unwrap();
    assert_eq!(scores.len(), 6, "{}", levels[3]);
    assert_eq!(scores["str"], 17);
    assert_eq!(scores["dex"], full_row(&sheet, 3)["scores"]["dex"]);
    assert_eq!(levels[2].get("scores"), None);
    // `class` still stands among the picks of every level.
    assert!(levels[2]["picks"][0]["choice"] == "level.3.class");
    // The sheet leaves out `todo` unless asked.
    assert!(sheet.get("todo").is_none(), "{sheet}");
}

#[tokio::test]
async fn todo_comes_with_the_sheet_only_when_asked() {
    let client = Client::on_the_test_world().await;
    let sheet = client
        .ok("sheet", json!({ "build": code(&fighter()), "todo": true }))
        .await;
    assert_eq!(choices(&sheet["todo"]).len(), 7, "{sheet}");
    assert_eq!(sheet["build"], code(&fighter()));
    assert!(sheet["link"].is_string());
    let plain = sheet_of(&fighter(), false).await;
    assert!(plain.get("todo").is_none());
    assert!(plain["link"].is_string());
}

/// A human fighter 20 on the real data: a greatsword (Weapon Focus), two Strength increases and the Trickster's path.
async fn real_fighter_20(client: &Client) -> String {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
        ("start.score.str", json!(17)),
        ("start.score.dex", json!(13)),
        ("start.score.con", json!(14)),
        (
            "slot.feat.1.race.0",
            json!({ "key": "weapon-focus", "parameter": "greatsword" }),
        ),
        ("level.4.increase", json!("str")),
        ("level.8.increase", json!("str")),
        ("mythic.path", json!("trickster")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=20 {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": "fighter" }),
        );
    }
    let declared = client.ok("validate", json!({ "choices": choices })).await;
    assert_eq!(declared["status"], "valid", "{declared}");
    text(&declared["build"])
}

/// Every level's full row rebuilt from the trimmed answer is the row `levels: [n]` gives.
async fn assert_every_row_rebuilds(client: &Client, arguments: Value) {
    let sheet = client.ok("sheet", arguments.clone()).await;
    let count = sheet["levels"].as_array().unwrap().len() as u64;
    for n in 1..=count {
        let mut one = arguments.clone();
        one["levels"] = json!([n]);
        let alone = client.ok("sheet", one).await;
        assert_eq!(
            alone["levels"].as_array().unwrap().len(),
            1,
            "level {n}: {alone}"
        );
        assert_eq!(
            full_row(&sheet, n),
            alone["levels"][0],
            "level {n} of the trimmed sheet, rebuilt, against `levels: [{n}]`"
        );
    }
}

#[tokio::test]
async fn real_data_every_level_of_a_fighter_20_rebuilds_from_the_trimmed_sheet() {
    let client = Client::start().await;
    let build = real_fighter_20(&client).await;
    assert_every_row_rebuilds(&client, json!({ "build": build })).await;

    let sheet = client.ok("sheet", json!({ "build": build })).await;
    // The greatsword's line is told once and the rows after it say only what moved.
    let last = &sheet["levels"][19];
    assert_eq!(
        full_row(&sheet, 20)["attack"]["weapon"]["key"],
        "greatsword"
    );
    assert!(last["attack"].get("weapon").is_none(), "{last}");
    // The path's features come with the level that gains them, none repeated.
    assert!(sheet["levels"][8]["features"].as_array().unwrap().len() > 1);
}

#[tokio::test]
async fn real_data_every_level_of_a_caster_20_rebuilds_from_the_trimmed_sheet() {
    let client = Client::start().await;
    let caster = json!({ "school": "enchantment", "subschool": "compulsion" });
    assert_every_row_rebuilds(
        &client,
        json!({ "build": crate::caster_line::MAX_DC_RUN, "caster": caster }),
    )
    .await;
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": crate::caster_line::MAX_DC_RUN, "caster": caster }),
        )
        .await;
    let last = &sheet["levels"][19]["caster"];
    // The ability and the class are said by the first row alone; the numbers that move are on every row.
    assert!(last.get("ability").is_none(), "{last}");
    assert!(last["dcs"].is_array() && last["concentration"].is_number());
    assert_eq!(full_row(&sheet, 20)["caster"]["ability"], "Charisma");
}

#[tokio::test]
async fn levels_returns_those_levels_alone_each_in_full_sorted() {
    let client = Client::on_the_test_world().await;
    let ask = |levels: Value| {
        client.ok(
            "sheet",
            json!({ "build": code(&fighter()), "levels": levels }),
        )
    };
    let alone = ask(json!([3])).await;
    let rows = alone["levels"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{alone}");
    assert_eq!(rows[0]["level"], 3);
    // In full: the class, the scores, the skill points and the attack line's weapon.
    for field in ["class", "scores", "skill_points"] {
        assert!(rows[0].get(field).is_some(), "{field}: {}", rows[0]);
    }
    assert!(rows[0]["attack"].get("weapon").is_some());
    // What else the sheet holds stays: the start, the totals, no rank rows.
    assert!(alone["start"].is_array() && alone["totals"].is_object());
    assert!(alone["ranks"].as_array().unwrap().is_empty());

    let both = ask(json!([4, 2, 4])).await;
    let order: Vec<u64> = both["levels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["level"].as_u64().unwrap())
        .collect();
    assert_eq!(order, [2, 4], "sorted, once each");
    assert!(both["levels"][0].get("scores").is_some());

    let none = ask(json!([])).await;
    assert!(none["levels"].as_array().unwrap().is_empty(), "{none}");
    assert_eq!(none["totals"], alone["totals"]);
}

#[tokio::test]
async fn ranks_returns_those_rank_rows_alone() {
    let client = Client::start().await;
    let build = real_fighter_20(&client).await;
    let all = client.ok("sheet", json!({ "build": build })).await;
    let sheet = client
        .ok("sheet", json!({ "build": build, "ranks": [3] }))
        .await;
    let ranks = sheet["ranks"].as_array().unwrap();
    assert_eq!(ranks.len(), 1, "{sheet}");
    assert_eq!(ranks[0], all["ranks"][2]);
    assert!(sheet["levels"].as_array().unwrap().is_empty());

    let both = client
        .ok(
            "sheet",
            json!({ "build": build, "ranks": [4, 2], "levels": [6] }),
        )
        .await;
    assert_eq!(both["ranks"][0]["rank"], 2);
    assert_eq!(both["ranks"][1]["rank"], 4);
    assert_eq!(both["levels"][0]["level"], 6);
}

#[tokio::test]
async fn a_level_the_build_lacks_and_a_rank_that_does_not_exist_are_refused_with_the_reason() {
    let client = Client::on_the_test_world().await;
    let refused = |arguments: Value| async {
        let result = client.call("sheet", arguments).await;
        (error_code(&result), error_text(&result))
    };
    let build = code(&fighter());

    let (code_, text_) = refused(json!({ "build": build, "levels": [2, 5, 9] })).await;
    assert_eq!(code_, "invalid_arguments");
    assert!(
        text_.contains("`levels`: The Build has 4 levels, so it doesn't reach level 5."),
        "{text_}"
    );
    let (_, text_) = refused(json!({ "build": build, "levels": [0] })).await;
    assert!(
        text_.contains("`levels`: There is no level 0: levels start at 1."),
        "{text_}"
    );
    let (code_, text_) = refused(json!({ "build": build, "ranks": [3, 11] })).await;
    assert_eq!(code_, "invalid_arguments");
    assert!(
        text_.contains("`ranks`: There is no MR 11: the Mythic ranks are 1 to 10."),
        "{text_}"
    );
    // A rank that exists but isn't reached comes back as its `not_reached` row, as in the whole sheet.
    let sheet = client
        .ok("sheet", json!({ "build": build, "ranks": [1] }))
        .await;
    assert_eq!(sheet["ranks"].as_array().unwrap().len(), 1, "{sheet}");
    assert_eq!(sheet["ranks"][0]["status"], "not_reached");
}

/// A human fighter 2, then an arcanist (the Brown-Fur Transmuter) to level 10, on the real data: the class and the
/// archetype change at level 3, and the caster line starts there.
async fn fighter_then_archetyped_arcanist(client: &Client) -> String {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
        ("start.score.int", json!(16)),
        ("start.score.str", json!(14)),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=10 {
        let class = match level {
            1 | 2 => json!({ "class": "fighter" }),
            3 => json!({ "class": "arcanist", "archetype": "arcanist/brown-fur-transmuter" }),
            _ => json!({ "class": "arcanist" }),
        };
        choices.insert(format!("level.{level}.class"), class);
    }
    let declared = client.ok("validate", json!({ "choices": choices })).await;
    assert_eq!(declared["status"], "valid", "{declared}");
    text(&declared["build"])
}

#[tokio::test]
async fn real_data_a_multiclass_with_an_archetype_and_a_late_caster_line_rebuilds_from_the_trimmed_sheet()
 {
    let client = Client::start().await;
    let build = fighter_then_archetyped_arcanist(&client).await;
    let arguments = json!({ "build": build, "caster": { "school": "transmutation" } });
    assert_every_row_rebuilds(&client, arguments.clone()).await;

    let sheet = client.ok("sheet", arguments).await;
    let rows = sheet["levels"].as_array().unwrap();
    // The class changes at 3 and the archetype comes with it; 4 says neither.
    assert!(rows[2].get("class").is_some() && rows[2].get("archetype").is_some());
    assert!(rows[3].get("class").is_none() && rows[3].get("archetype").is_none());
    assert_eq!(full_row(&sheet, 4)["archetype"], rows[2]["archetype"]);
    // No caster line before the arcanist; its first one has everything and the next leaves the class out.
    assert!(rows[1].get("caster").is_none());
    assert!(rows[2]["caster"].get("class").is_some(), "{}", rows[2]);
    assert!(rows[3]["caster"].get("class").is_none(), "{}", rows[3]);
}
