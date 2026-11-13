//! `options`: every picker and stepper, with the engine's statuses and reasons, and the server's filters and errors.

use rmcp::serde_json::{Value, json};
use rules::test_world::{
    build, class_slot, general_slot, pick_feat, pick_feat_for, point_buy, times, weapon,
};
use rules::{Alignment, Build, DeityKey, Faith};

use crate::support::*;

/// `options` on the test world, as a successful call's result.
async fn options(build: &Build, arguments: Value) -> Value {
    let mut arguments = arguments;
    arguments["build"] = json!(code(build));
    Client::on_the_test_world()
        .await
        .ok("options", arguments)
        .await
}

/// The option with this value.
fn option(result: &Value, value: Value) -> &Value {
    result["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["value"] == value)
        .unwrap_or_else(|| panic!("no option {value} in {result}"))
}

/// The options' values.
fn values(result: &Value) -> Vec<Value> {
    result["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|option| option["value"].clone())
        .collect()
}

#[tokio::test]
async fn a_feat_slot_lists_its_open_feats_and_counts_the_rest() {
    let fighter = build("human", &["fighter"]);
    let open = options(&fighter, json!({ "choice": "slot.feat.1.general.0" })).await;
    assert_eq!(open["choice"], "slot.feat.1.general.0");
    assert_eq!(option(&open, json!("power-attack"))["status"], "ok");
    assert_eq!(option(&open, json!("toughness"))["status"], "ok");
    assert_eq!(option(&open, json!("power-attack"))["category"], "Weapon");
    // Cleave needs Power Attack: hidden, and counted.
    assert!(!values(&open).contains(&json!("cleave")));
    assert!(open["closed"].as_u64().unwrap() > 0);
    // No envelope unless asked (ticket 128): `options` changes nothing.
    assert!(
        open.get("build").is_none() && open.get("todo").is_none(),
        "{open}"
    );
}

#[tokio::test]
async fn a_locked_feat_shows_the_engines_issue_and_reason() {
    let fighter = build("human", &["fighter"]);
    let all = options(
        &fighter,
        json!({ "choice": "slot.feat.1.general.0", "only_open": false }),
    )
    .await;
    assert_eq!(all["closed"], 0);
    let cleave = option(&all, json!("cleave"));
    assert_eq!(cleave["status"]["issue"], "unmet");
    assert_eq!(cleave["status"]["reason"], "Needs power-attack");
    assert_eq!(option(&all, json!("power-attack"))["status"], "ok");
}

#[tokio::test]
async fn a_parameterised_feat_lists_its_values_on_request() {
    let mut fighter = build("human", &["fighter"]);
    fighter.picks.insert(
        class_slot(1, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    // Once, saying what a value is.
    let entries = options(&fighter, json!({ "choice": "slot.feat.1.general.0" })).await;
    let focus = option(&entries, json!("weapon-focus"));
    assert_eq!(focus["takes"], "a weapon");
    assert_eq!(focus["status"], "ok");

    // Once per value, the longsword already held.
    let values = options(
        &fighter,
        json!({ "choice": "slot.feat.1.general.0", "values": true, "only_open": false, "query": "weapon-focus" }),
    )
    .await;
    let dagger = option(
        &values,
        json!({ "key": "weapon-focus", "parameter": "dagger" }),
    );
    assert_eq!(dagger["status"], "ok");
    assert_eq!(dagger["name"], "weapon-focus (dagger)");
    let longsword = option(
        &values,
        json!({ "key": "weapon-focus", "parameter": "longsword" }),
    );
    assert_eq!(longsword["status"]["issue"], "already_held");
    // Ticket 192: it says where the pick is held.
    assert_eq!(
        longsword["status"]["reason"],
        "weapon-focus (longsword) is taken in level 1's Combat feat slot"
    );
}

#[tokio::test]
async fn the_query_filters_by_name_or_key() {
    let fighter = build("human", &["fighter"]);
    let cleaves = options(
        &fighter,
        json!({ "choice": "slot.feat.1.general.0", "only_open": false, "query": "CLEAV" }),
    )
    .await;
    assert_eq!(
        values(&cleaves),
        [
            json!("cleave"),
            json!("cleaving-finish"),
            json!("great-cleave")
        ]
    );
}

#[tokio::test]
async fn a_filled_slot_marks_its_pick() {
    let mut fighter = build("human", &["fighter"]);
    fighter
        .picks
        .insert(general_slot(1), pick_feat("toughness"));
    let open = options(&fighter, json!({ "choice": "slot.feat.1.general.0" })).await;
    assert_eq!(option(&open, json!("toughness"))["current"], true);
    assert!(
        option(&open, json!("power-attack"))
            .get("current")
            .is_none()
    );
}

#[tokio::test]
async fn the_class_picker_offers_classes_archetypes_and_prestige_entries() {
    let human = build("human", &[]);
    let first = options(
        &human,
        json!({ "choice": "level.1.class", "only_open": false }),
    )
    .await;
    let fighter = option(&first, json!({ "class": "fighter" }));
    assert_eq!(fighter["status"], "ok");
    assert_eq!(fighter["name"], "Fighter");
    assert_eq!(fighter["category"], "Base class");
    let armiger = option(
        &first,
        json!({ "class": "fighter", "archetype": "fighter/armiger" }),
    );
    assert_eq!(armiger["name"], "Armiger (Fighter)");
    // A prestige class needs arcane spells of the 2nd level.
    let knight = option(&first, json!({ "class": "eldritch-knight" }));
    assert_eq!(knight["status"]["issue"], "unmet");
    assert_eq!(knight["category"], "Prestige class");
    assert!(
        knight["status"]["reason"]
            .as_str()
            .unwrap()
            .starts_with("Needs"),
        "{knight}"
    );
}

#[tokio::test]
async fn a_prestige_class_is_offered_once_per_advancement() {
    // A wizard 3 can cast 2nd-level arcane spells: the Eldritch Knight at 4 asks which class it advances.
    let wizard = build("human", &times("wizard", 3));
    let fourth = options(&wizard, json!({ "choice": "level.4.class" })).await;
    let advancing = option(
        &fourth,
        json!({ "class": "eldritch-knight", "advances": ["wizard"] }),
    );
    assert_eq!(advancing["name"], "Eldritch Knight, advancing Wizard");
    assert_eq!(advancing["status"], "ok");
    option(
        &fourth,
        json!({ "class": "eldritch-knight", "advances": [] }),
    );
    // A class already taken keeps its archetype: no archetype options for the wizard at 4.
    let wizards: Vec<Value> = values(&fourth)
        .into_iter()
        .filter(|value| value["class"] == "wizard")
        .collect();
    assert_eq!(wizards, [json!({ "class": "wizard" })]);
}

#[tokio::test]
async fn the_build_start_pickers_carry_the_faith_rules() {
    // An empty Build: every alignment waits for a faith.
    let empty = Build::default();
    let alignments = options(
        &empty,
        json!({ "choice": "start.alignment", "only_open": false }),
    )
    .await;
    assert_eq!(values(&alignments).len(), 9);
    let good = option(&alignments, json!("lawful-good"));
    assert_eq!(good["status"]["issue"], "needs_faith");
    assert_eq!(good["status"]["reason"], "Choose a deity or atheism first");

    // A lawful good follower of the Lawgiver: the Balance doesn't allow lawful good.
    let mut follower = build("human", &[]);
    follower.start.deity = Some(Faith::Deity(DeityKey::new("lawgiver")));
    follower.start.alignment = Some(Alignment::LawfulGood);
    let faiths = options(
        &follower,
        json!({ "choice": "start.faith", "only_open": false }),
    )
    .await;
    assert_eq!(option(&faiths, json!("atheism"))["status"], "ok");
    let lawgiver = option(&faiths, json!("lawgiver"));
    assert_eq!(
        (lawgiver["status"].clone(), lawgiver["current"].clone()),
        (json!("ok"), json!(true))
    );
    let balance = option(&faiths, json!("balance"));
    assert_eq!(balance["status"]["issue"], "deity_alignment");
    assert!(
        balance["status"]["reason"]
            .as_str()
            .unwrap()
            .contains("allows"),
        "{balance}"
    );
}

#[tokio::test]
async fn the_race_background_and_racial_ability_pickers() {
    let empty = Build::default();
    let races = options(&empty, json!({ "choice": "start.race" })).await;
    let race_values = values(&races);
    for race in [
        "half-elf",
        "half-elf/dual-heritage",
        "human",
        "elf/blightborn",
    ] {
        assert!(race_values.contains(&json!(race)), "{race_values:?}");
    }
    assert_eq!(
        option(&races, json!("half-elf/dual-heritage"))["name"],
        "Dual Heritage"
    );

    let backgrounds = options(&empty, json!({ "choice": "start.background" })).await;
    assert!(!values(&backgrounds).is_empty());

    // A human chooses where its +2 goes; a dwarf doesn't.
    let human = build("human", &[]);
    let abilities = options(&human, json!({ "choice": "start.racial-ability" })).await;
    assert_eq!(
        values(&abilities),
        ["str", "dex", "con", "int", "wis", "cha"].map(|key| json!(key))
    );
    assert_eq!(option(&abilities, json!("int"))["name"], "Intelligence");
    let dwarf = build("dwarf", &[]);
    let refused = Client::on_the_test_world()
        .await
        .call(
            "options",
            json!({ "build": code(&dwarf), "choice": "start.racial-ability" }),
        )
        .await;
    assert_eq!(error_code(&refused), "no_such_choice");
}

#[tokio::test]
async fn an_ability_increase_offers_each_ability() {
    let fighter = build("human", &times("fighter", 4));
    let increase = options(&fighter, json!({ "choice": "level.4.increase" })).await;
    assert_eq!(values(&increase).len(), 6);
    assert_eq!(option(&increase, json!("str"))["name"], "+1 Strength");
}

#[tokio::test]
async fn a_point_buy_stepper_offers_a_step_each_way() {
    // Int 13: one down to 12, one up to 14.
    let fighter = build("human", &["fighter"]);
    let int = options(&fighter, json!({ "choice": "start.score.int" })).await;
    assert_eq!(values(&int), [json!(12), json!(14)]);
    assert_eq!(
        option(&int, json!(14))["name"],
        "Intelligence 14 (20 of 25 points)"
    );

    // Str 13 with Power Attack taken: the step down would break it.
    let mut strong = build("human", &["fighter"]);
    strong.start.ability_scores = point_buy([13, 14, 12, 13, 10, 8]);
    strong
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    let str_steps = options(
        &strong,
        json!({ "choice": "start.score.str", "only_open": false }),
    )
    .await;
    let down = option(&str_steps, json!(12));
    assert_eq!(down["status"]["issue"], "breaks_later");
    // Ticket 192: it names what it would break.
    assert_eq!(
        down["status"]["reason"], "power-attack needs Strength above 12",
        "{down}"
    );
    assert_eq!(option(&str_steps, json!(14))["status"], "ok");
}

#[tokio::test]
async fn the_skill_steppers_offer_ranks_one_at_a_time() {
    let fighter = build("human", &["fighter"]);
    // No ranks yet: only a step up, to 1.
    let athletics = options(&fighter, json!({ "choice": "level.1.skill.athletics" })).await;
    assert_eq!(values(&athletics), [json!(1)]);
    assert_eq!(
        option(&athletics, json!(1))["name"],
        "Athletics: 1 rank at level 1"
    );
    // The level's skill points: one rank more in any skill.
    let points = options(&fighter, json!({ "choice": "level.1.skill" })).await;
    assert_eq!(
        values(&points),
        ["athletics", "knowledge-arcana", "mobility", "persuasion"].map(|key| json!(key))
    );
}

#[tokio::test]
async fn the_mythic_pickers_open_with_their_ranks() {
    let fighter = build("human", &times("fighter", 9));
    let paths = options(&fighter, json!({ "choice": "mythic.path" })).await;
    let path_values = values(&paths);
    assert!(path_values.contains(&json!("trickster")), "{path_values:?}");
    assert!(path_values.contains(&json!("aeon")), "{path_values:?}");
    // The switch paths aren't offered at MR 3.
    assert!(!path_values.contains(&json!("legend")), "{path_values:?}");
    let first = options(&fighter, json!({ "choice": "mythic.first-ascension" })).await;
    assert!(!values(&first).is_empty());

    // MR 8 isn't reached.
    let refused = Client::on_the_test_world()
        .await
        .call(
            "options",
            json!({ "build": code(&fighter), "choice": "mythic.switch" }),
        )
        .await;
    assert_eq!(error_code(&refused), "no_such_choice");
}

#[tokio::test]
async fn an_address_the_build_lacks_is_no_such_choice() {
    let client = Client::on_the_test_world().await;
    let fighter = code(&build("human", &["fighter"]));
    for choice in [
        "slot.feat.2.general.0",
        "slot.feat.1.class.5",
        "level.3.class",
        "level.1.increase",
        "level.2.skill.athletics",
        "level.1.skill.lore",
        "mythic.path",
        "start.subrace",
        "class.fighter.archetype",
        "level.1",
        "not an address",
    ] {
        let result = client
            .call("options", json!({ "build": fighter, "choice": choice }))
            .await;
        assert_eq!(error_code(&result), "no_such_choice", "{choice}");
    }
    // No level past the cap.
    let twenty = code(&build("human", &times("fighter", 20)));
    let result = client
        .call(
            "options",
            json!({ "build": twenty, "choice": "level.21.class" }),
        )
        .await;
    assert_eq!(error_code(&result), "no_such_choice");
    // An unknown code is stale before the address is looked at.
    let result = client
        .call(
            "options",
            json!({ "build": "AAAA", "choice": "start.race" }),
        )
        .await;
    assert_eq!(error_code(&result), "stale_link");
}

#[tokio::test]
async fn real_data_names_the_class_options() {
    let client = Client::start().await;
    let empty = client.ok("new_build", json!({})).await;
    let classes = client
        .ok(
            "options",
            json!({ "build": empty["build"], "choice": "level.1.class", "query": "brown fur" }),
        )
        .await;
    assert_eq!(
        classes["options"][0]["value"],
        json!({ "class": "arcanist", "archetype": "arcanist/brown-fur-transmuter" })
    );
    assert_eq!(
        classes["options"][0]["name"],
        "Brown-Fur Transmuter (Arcanist)"
    );
}
