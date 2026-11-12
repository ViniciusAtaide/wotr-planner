//! Gear in the server (ticket 168): a Build holds gear only when it's chosen, at `gear.<place>` and
//! `gear.usable.<item key>`; an item's bonuses open a feat its scores alone don't, and `options` and `sheet` say so;
//! an item a pick needs can't come off until the pick does; `sheet` gives the numbers with gear and without; `compare`
//! and `improve` rank by the naked numbers unless asked, and never add an item. On the test world, with a feat that
//! needs Dexterity 19 as Greater Two-Weapon Fighting does, and on the real data for `search`.

use mcp::Planner;
use rmcp::serde_json::{Value, json};
use rules::test_world::{
    build, engine_on, feat, gear_item, general_slot, pick_feat, point_buy, world,
};
use rules::{Ability, Build, Engine, GameData, GearPlace, Prerequisite};

use crate::support::*;

const GTWF: &str = "greater-two-weapon-fighting";
const BELT: &str = "belt-of-dexterity-2";

/// The test world with Greater Two-Weapon Fighting as a feat that needs only Dexterity 19.
fn data() -> GameData {
    let mut data = world();
    let mut gtwf = (data.feats.iter())
        .find(|row| row.key == feat("dodge"))
        .expect("the test world has Dodge")
        .clone();
    gtwf.key = feat(GTWF);
    gtwf.name = "Greater Two-Weapon Fighting".into();
    gtwf.prerequisite = Some(Prerequisite::AbilityScore {
        ability: Ability::Dexterity,
        score: 19,
    });
    data.feats.push(gtwf);
    data
}

async fn client() -> Client {
    Client::start_on(Planner::from_data(data()).expect("the test world checks")).await
}

/// The Build a code holds, in this test world.
fn decode(code: &Value) -> Build {
    let engine: Engine = engine_on(data());
    engine.open_link(&text(code)).expect("the code opens")
}

/// A human fighter 1 with Dexterity 17 (the racial +2 left open).
fn fighter() -> Build {
    let mut build = build("human", &["fighter"]);
    build.start.ability_scores = point_buy([10, 17, 10, 10, 10, 10]);
    build
}

/// The fighter wearing the Dexterity +2 belt, with Greater Two-Weapon Fighting in the general slot.
fn belted_two_weapon_fighter() -> Build {
    let mut build = fighter();
    build.gear.worn.insert(GearPlace::Belt, gear_item(BELT));
    build.picks.insert(general_slot(1), pick_feat(GTWF));
    build
}

/// The option of a list with this value.
fn option<T>(options: &Value, value: T) -> &Value
where
    Value: PartialEq<T>,
    T: std::fmt::Display + Copy,
{
    options["options"]
        .as_array()
        .expect("options is a list")
        .iter()
        .find(|option| option["value"] == value)
        .unwrap_or_else(|| panic!("no {value} in {options}"))
}

#[tokio::test]
async fn choosing_a_belt_opens_a_locked_feat_with_the_gear_status() {
    let client = client().await;
    let slot = "slot.feat.1.general.0";
    let ask = |code: Value| json!({ "build": code, "choice": slot, "only_open": false, "query": "greater two-weapon" });

    let before = client.ok("options", ask(json!(code(&fighter())))).await;
    assert_eq!(
        option(&before, GTWF)["status"],
        json!({ "issue": "unmet", "reason": "Needs Dex 19 (have 17)" })
    );

    let belted = client
        .ok(
            "choose",
            json!({ "build": code(&fighter()), "steps": [{ "choice": "gear.belt", "value": BELT }] }),
        )
        .await;
    assert_eq!(belted["applied"], 1, "{belted}");
    // Gear is optional: no `todo` entry asks for it.
    assert!(
        choices(&belted["todo"])
            .iter()
            .all(|choice| !choice.starts_with("gear")),
        "{belted}"
    );

    let after = client.ok("options", ask(belted["build"].clone())).await;
    assert_eq!(
        option(&after, GTWF)["status"],
        json!({ "with_gear": "With Belt of Dexterity +2" })
    );

    let taken = client
        .ok(
            "choose",
            json!({ "build": belted["build"], "steps": [{ "choice": slot, "value": GTWF }] }),
        )
        .await;
    assert_eq!(taken["applied"], 1, "{taken}");
    assert_eq!(decode(&taken["build"]), belted_two_weapon_fighter());
}

#[tokio::test]
async fn sheet_shows_both_numbers_and_the_dependent_pick() {
    let client = client().await;
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": code(&belted_two_weapon_fighter()) }),
        )
        .await;

    let belt = at(&sheet["gear"], "gear.belt");
    assert_eq!(belt["value"], BELT);
    assert_eq!(belt["name"], "Belt of Dexterity +2");
    assert_eq!(belt["summary"], "Dex +2 enhancement");

    // The scores count the belt; `naked` gives them without it.
    assert_eq!(sheet["levels"][0]["scores"]["dex"], 19);
    assert_eq!(sheet["naked"]["levels"][0]["level"], 1);
    assert_eq!(sheet["naked"]["levels"][0]["scores"]["dex"], 17);
    assert_eq!(sheet["naked"]["totals"]["reflex"]["total"], 3, "{sheet}");
    assert_eq!(sheet["totals"]["reflex"]["total"], 4, "{sheet}");

    assert_eq!(
        sheet["gear_dependent"],
        json!([{
            "choice": "slot.feat.1.general.0",
            "label": "Level 1: Feat",
            "name": "Greater Two-Weapon Fighting",
            "needs": "With Belt of Dexterity +2",
            "items": [BELT],
        }])
    );
    let pick = at(&sheet["levels"][0]["picks"], "slot.feat.1.general.0");
    assert_eq!(pick["with_gear"], "With Belt of Dexterity +2");
}

#[tokio::test]
async fn withdrawing_the_belt_is_refused_naming_the_pick_until_the_pick_is_withdrawn() {
    let client = client().await;
    let build = json!(code(&belted_two_weapon_fighter()));

    let refused = client
        .call("withdraw", json!({ "build": build, "choice": "gear.belt" }))
        .await;
    assert_eq!(error_code(&refused), "not_removable");
    let reason = error_text(&refused);
    assert!(
        reason.contains("Greater Two-Weapon Fighting needs Belt of Dexterity +2"),
        "{reason}"
    );

    // Replacing it is locked too, with the same reason; `choose` refuses it.
    let options = client
        .ok(
            "options",
            json!({ "build": build, "choice": "gear.belt", "only_open": false }),
        )
        .await;
    for value in ["none", "belt-of-constitution-2"] {
        assert_eq!(
            option(&options, value)["status"],
            json!({
                "issue": "breaks_later",
                "reason": "Greater Two-Weapon Fighting needs Belt of Dexterity +2",
            }),
            "{value}"
        );
    }
    assert_eq!(option(&options, BELT)["current"], true);
    let chosen = client
        .ok(
            "choose",
            json!({ "build": build, "steps": [{ "choice": "gear.belt", "value": "none" }] }),
        )
        .await;
    assert_eq!(chosen["rejected"]["issue"], "breaks_later", "{chosen}");

    // With the pick withdrawn, the belt comes off.
    let cleared = client
        .ok(
            "withdraw",
            json!({ "build": build, "choice": "slot.feat.1.general.0" }),
        )
        .await;
    let bare = client
        .ok(
            "withdraw",
            json!({ "build": cleared["build"], "choice": "gear.belt" }),
        )
        .await;
    assert_eq!(decode(&bare["build"]), fighter());
    let sheet = client.ok("sheet", json!({ "build": bare["build"] })).await;
    for field in ["gear", "gear_dependent", "naked"] {
        assert_eq!(sheet.get(field), None, "{field}: {sheet}");
    }
}

#[tokio::test]
async fn a_usable_item_is_used_and_stopped_at_its_own_address() {
    let client = client().await;
    let address = "gear.usable.manual-of-dexterity-2";
    let options = client
        .ok(
            "options",
            json!({ "build": code(&fighter()), "choice": address }),
        )
        .await;
    assert_eq!(option(&options, "none")["current"], true, "{options}");
    assert_eq!(option(&options, "manual-of-dexterity-2")["status"], "ok");

    let used = client
        .ok(
            "choose",
            json!({ "build": code(&fighter()), "steps": [{ "choice": address, "value": "manual-of-dexterity-2" }] }),
        )
        .await;
    let sheet = client.ok("sheet", json!({ "build": used["build"] })).await;
    assert_eq!(
        at(&sheet["gear"], address)["name"],
        "Manual of Dexterity +2"
    );
    assert_eq!(sheet["levels"][0]["scores"]["dex"], 19);

    let stopped = client
        .ok(
            "withdraw",
            json!({ "build": used["build"], "choice": address }),
        )
        .await;
    assert_eq!(decode(&stopped["build"]), fighter());

    // A worn item has no usable address.
    let worn = client
        .call(
            "options",
            json!({ "build": code(&fighter()), "choice": "gear.usable.belt-of-dexterity-2" }),
        )
        .await;
    assert_eq!(error_code(&worn), "no_such_choice");
    assert!(error_text(&worn).contains("gear.belt"), "{worn:?}");
}

#[tokio::test]
async fn a_step_that_would_lean_on_the_gear_says_so() {
    let client = client().await;
    // Dexterity 17 with the racial +2 is 19 on its own, 21 with the belt: Greater Two-Weapon Fighting needs no gear.
    let mut build = belted_two_weapon_fighter();
    build.start.racial_ability = Some(Ability::Dexterity);
    let options = client
        .ok(
            "options",
            json!({ "build": code(&build), "choice": "start.score.dex" }),
        )
        .await;
    // A point less is 18 on its own: the feat would then hold only with the belt.
    assert_eq!(
        option(&options, 16)["status"],
        json!({
            "leans_on_gear": "Greater Two-Weapon Fighting would then be allowed only because of the gear",
        }),
        "{options}"
    );
    assert_eq!(option(&options, 18)["status"], "ok");
}

/// A human fighter 4 with this Strength who has a level-4 increase open, using the Tome of Perfection +4 or not.
fn strong(strength: u8, tome: bool) -> Build {
    let mut build = build("human", &["fighter", "fighter", "fighter", "fighter"]);
    build.start.ability_scores = point_buy([strength, 10, 10, 10, 10, 10]);
    if tome {
        build.gear.used.insert(gear_item("tome-of-perfection-4"));
    }
    build
}

#[tokio::test]
async fn compare_ranks_naked_by_default_and_with_gear_when_asked() {
    let client = client().await;
    // Strength 14 with a +4 tome is 18; 16 without one is 16.
    let builds = json!([code(&strong(14, true)), code(&strong(16, false))]);
    let by = json!({ "attack_line": { "weapon": "longsword" } });

    let naked = client
        .ok("compare", json!({ "builds": builds, "by": by }))
        .await;
    assert_eq!(naked["numbers"], "naked");
    assert_eq!(naked["builds"][0]["given"], 1, "{naked}");
    assert_eq!(naked["builds"][1]["given"], 0, "{naked}");
    let tome = &naked["builds"][1];
    assert_eq!(tome["scores"]["str"], 14);
    assert_eq!(tome["with_gear"]["scores"]["str"], 18, "{tome}");
    assert!(
        tome["with_gear"]["number"].as_i64() > tome["number"].as_i64(),
        "{tome}"
    );
    assert_eq!(tome["gear"][0]["key"], "tome-of-perfection-4");
    assert_eq!(
        naked["gear"][0]["choice"],
        "gear.usable.tome-of-perfection-4"
    );
    // The differences are naked too.
    let level_1 = &naked["levels"][0];
    let strength = (level_1["numbers"].as_array().unwrap().iter())
        .find(|number| number["what"] == "str")
        .expect("Strength differs");
    assert_eq!(strength["values"], json!([16, 14]));

    let geared = client
        .ok(
            "compare",
            json!({ "builds": builds, "by": by, "with_gear": true }),
        )
        .await;
    assert_eq!(geared["numbers"], "with_gear");
    assert_eq!(geared["builds"][0]["given"], 0, "{geared}");
    assert_eq!(geared["builds"][0]["scores"]["str"], 18);
    assert_eq!(geared["builds"][0]["naked"]["scores"]["str"], 14);
    assert_eq!(geared["builds"][1].get("naked"), None);
}

#[tokio::test]
async fn improve_ranks_naked_by_default_and_never_proposes_gear() {
    let client = client().await;
    let build = code(&strong(14, true));
    let ask = |with_gear: bool| {
        json!({
            "build": build,
            "choice": "level.4.increase",
            "by": { "attack_line": { "weapon": "longsword" } },
            "with_gear": with_gear,
        })
    };
    let naked = client.ok("improve", ask(false)).await;
    let geared = client.ok("improve", ask(true)).await;
    assert_eq!(naked["numbers"], "naked");
    assert_eq!(geared["numbers"], "with_gear");
    // Strength 14 naked, 18 with the tome: two points of attack apart.
    assert_eq!(
        geared["current"].as_i64().unwrap() - naked["current"].as_i64().unwrap(),
        2,
        "{naked} {geared}"
    );

    for choice in ["gear.belt", "gear.usable.tome-of-perfection-4"] {
        let refused = client
            .call(
                "improve",
                json!({ "build": build, "choice": choice, "by": { "attack_line": { "weapon": "longsword" } } }),
            )
            .await;
        assert_eq!(error_code(&refused), "invalid_arguments", "{choice}");
        assert!(error_text(&refused).contains("never proposes gear"));
    }
}

#[tokio::test]
async fn search_finds_gear_by_name_place_and_bonus() {
    let client = Client::start().await;
    let found = client
        .ok(
            "search",
            json!({ "query": "Belt of Incredible Dexterity +2", "kinds": ["gear"] }),
        )
        .await;
    let belt = &found["matches"][0];
    assert_eq!(belt["key"], "belt-of-incredible-dexterity-2");
    assert_eq!(belt["category"], "Belt");
    assert_eq!(belt["summary"], "Dex +2 enhancement");
    assert_eq!(belt["places"], json!(["gear.belt"]));
    assert_eq!(
        belt.get("guide_url"),
        None,
        "the generic belts have no guide entry"
    );

    // By its bonus's words, its place, and its other effects in a field of their own (not under `compact`).
    let by_bonus = client
        .search(json!({ "query": "Dexterity enhancement", "kinds": ["gear"], "limit": 100 }))
        .await;
    assert!(
        by_bonus.contains(&("gear".into(), "assailants-belt".into())),
        "{by_bonus:?}"
    );
    // "Heart of Ice" is a belt by its slot alone.
    let belts = client
        .search(json!({ "query": "belt", "kinds": ["gear"], "limit": 100 }))
        .await;
    assert!(
        belts.contains(&("gear".into(), "heart-of-ice".into())),
        "{belts:?}"
    );
    let assailant = client
        .ok(
            "describe",
            json!({ "kind": "gear", "key": "assailants-belt" }),
        )
        .await;
    assert_eq!(
        assailant["other_effects"],
        json!(["Landing a critical hit: +2 circumstance on attacks and damage for 3 rounds."])
    );
    let ring = client
        .ok(
            "describe",
            json!({ "kind": "gear", "key": "colossus-ring" }),
        )
        .await;
    assert_eq!(ring["places"], json!(["gear.ring.0", "gear.ring.1"]));
    let compact = client
        .ok(
            "search",
            json!({ "query": "assailant's belt", "kinds": ["gear"], "compact": true }),
        )
        .await;
    assert_eq!(compact["matches"][0].get("other_effects"), None);
    assert_eq!(compact["matches"][0].get("places"), None);
}

#[tokio::test]
async fn the_instructions_and_the_guide_say_gear_comes_only_when_asked() {
    let client = client().await;
    let instructions = client.instructions();
    let guide = client.guide().await;
    for text in [&instructions, &guide] {
        for phrase in [
            "a Build has no gear unless the user asks for it",
            "`gear.belt`",
            "`gear.usable.<",
            "`gear_dependent`",
            "`with_gear: true`",
            "`naked`",
        ] {
            assert!(
                text.to_lowercase().contains(&phrase.to_lowercase()),
                "no {phrase} in {text}"
            );
        }
        assert!(!text.contains("The numbers are naked"), "{text}");
    }
}

#[tokio::test]
async fn an_unfiltered_search_finds_no_gear() {
    let client = Client::start().await;
    for query in ["belt", "dex"] {
        let found = client.search(json!({ "query": query, "limit": 100 })).await;
        assert!(
            found.iter().all(|(kind, _)| kind != "gear"),
            "{query}: {found:?}"
        );
    }
    // Asked for by kind, it is there.
    let found = client
        .search(json!({ "query": "belt", "kinds": ["gear", "feat"], "limit": 100 }))
        .await;
    assert!(found.iter().any(|(kind, _)| kind == "gear"), "{found:?}");
}

#[tokio::test]
async fn choosing_a_worn_ring_at_the_other_ring_place_says_it_moved() {
    let client = client().await;
    let result = client
        .ok(
            "choose",
            json!({ "build": code(&fighter()), "steps": [
                { "choice": "gear.ring.0", "value": "ring-of-luck" },
                { "choice": "gear.ring.1", "value": "ring-of-luck" },
            ] }),
        )
        .await;
    assert_eq!(result["applied"], 2, "{result}");
    assert_eq!(
        result["moved"],
        json!([{
            "step": 1,
            "choice": "gear.ring.1",
            "item": "ring-of-luck",
            "name": "Ring of Luck",
            "from": "gear.ring.0",
        }])
    );
    let build = decode(&result["build"]);
    assert_eq!(build.gear.worn.get(&GearPlace::FirstRing), None);
    assert_eq!(
        build.gear.worn.get(&GearPlace::SecondRing),
        Some(&gear_item("ring-of-luck"))
    );
    // A ring chosen at an empty place moves nothing.
    let single = client
        .ok(
            "choose",
            json!({ "build": code(&fighter()), "steps": [{ "choice": "gear.ring.1", "value": "ring-of-luck" }] }),
        )
        .await;
    assert_eq!(single.get("moved"), None, "{single}");
}

/// Taking off what isn't there is a no-op, as choosing what the Build already holds is anywhere: the step counts as
/// applied and the code doesn't change.
#[tokio::test]
async fn none_at_an_empty_place_is_a_no_op_like_any_held_value() {
    let client = client().await;
    let build = code(&fighter());
    for (choice, value) in [("gear.body", "none"), ("start.race", "human")] {
        let result = client
            .ok(
                "choose",
                json!({ "build": build, "steps": [{ "choice": choice, "value": value }] }),
            )
            .await;
        assert_eq!(result["applied"], 1, "{choice}: {result}");
        assert_eq!(result["build"], build, "{choice}");
        assert_eq!(result.get("rejected"), None, "{choice}");
    }
}

#[tokio::test]
async fn a_step_that_leaves_a_pick_leaning_on_the_gear_is_applied_and_says_so() {
    let client = client().await;
    let leaning = json!([{
        "choice": "slot.feat.1.general.0",
        "label": "Level 1: Feat",
        "name": "Greater Two-Weapon Fighting",
        "needs": "With Belt of Dexterity +2",
        "items": [BELT],
    }]);
    // Dexterity 17 with the racial +2 is 19 on its own; a point less leans on the belt.
    let mut build = belted_two_weapon_fighter();
    build.start.racial_ability = Some(Ability::Dexterity);
    let result = client
        .ok(
            "choose",
            json!({ "build": code(&build), "steps": [{ "choice": "start.score.dex", "value": 16 }] }),
        )
        .await;
    assert_eq!(result["applied"], 1, "{result}");
    assert_eq!(
        result["leans_on_gear"],
        json!([{ "step": 0, "choice": "start.score.dex", "leaning": leaning }])
    );
    // A step that leaves nothing leaning says nothing.
    let up = client
        .ok(
            "choose",
            json!({ "build": code(&build), "steps": [{ "choice": "start.score.dex", "value": 18 }] }),
        )
        .await;
    assert_eq!(up.get("leans_on_gear"), None, "{up}");

    // A withdrawal too: a fighter 5 whose Dexterity 19 at level 5 needs its level-4 increase.
    let mut fifth = rules::test_world::build("human", &["fighter"; 5]);
    fifth.start.ability_scores = point_buy([10, 16, 10, 10, 10, 10]);
    fifth.start.racial_ability = Some(Ability::Dexterity);
    fifth.levels[3].ability_increase = Some(Ability::Dexterity);
    fifth.gear.worn.insert(GearPlace::Belt, gear_item(BELT));
    fifth.picks.insert(general_slot(5), pick_feat(GTWF));
    let withdrawn = client
        .ok(
            "withdraw",
            json!({ "build": code(&fifth), "choice": "level.4.increase" }),
        )
        .await;
    assert_eq!(
        withdrawn["leans_on_gear"],
        json!([{
            "choice": "slot.feat.5.general.0",
            "label": "Level 5: Feat",
            "name": "Greater Two-Weapon Fighting",
            "needs": "With Belt of Dexterity +2",
            "items": [BELT],
        }]),
        "{withdrawn}"
    );
}
