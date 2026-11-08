//! `sheet`: a Build level by level and rank by rank, each choice by its address.

use rmcp::serde_json::{Value, json};
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
    assert_eq!(first["skill_points"], json!({ "available": 3, "spent": 0 }));

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
    let sheet = sheet_of(&fighter(), false).await;
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
    let sheet = sheet_of(&build("human", &times("fighter", 9)), false).await;
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

    // Without the flag, no marks.
    let plain = sheet_of(&fighter(), false).await;
    assert!(plain.get("removable").is_none());
    assert!(
        at(&plain["levels"][0]["picks"], "slot.feat.1.general.0")
            .get("removable")
            .is_none()
    );
}
