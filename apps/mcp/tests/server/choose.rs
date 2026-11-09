//! `choose`: a Build changes only by the options the engine offers, each applied as the engine's own edit, one step
//! at a time, and never into an impossible Build.

use rmcp::serde_json::{Value, json};
use rules::test_world::{build, class_slot, general_slot, pick_feat, times};
use rules::{Build, ClassKey};

use crate::support::*;

/// `choose` on the test world: these steps on this Build, as a successful call's result.
async fn choose(build: &Build, steps: Value) -> Value {
    Client::on_the_test_world()
        .await
        .ok("choose", json!({ "build": code(build), "steps": steps }))
        .await
}

/// The sheet of a code on the test world.
async fn sheet(code: &Value) -> Value {
    Client::on_the_test_world()
        .await
        .ok("sheet", json!({ "build": code }))
        .await
}

#[tokio::test]
async fn a_valid_choice_returns_a_new_code_whose_sheet_shows_it() {
    let fighter = build("human", &["fighter"]);
    let chosen = choose(
        &fighter,
        json!([{ "choice": "slot.feat.1.class.0", "value": "power-attack" }]),
    )
    .await;
    assert_eq!(chosen["applied"], 1);
    assert!(chosen.get("rejected").is_none(), "{chosen}");
    assert_ne!(chosen["build"], code(&fighter));
    assert!(text(&chosen["link"]).ends_with(&text(&chosen["build"])));
    assert!(!choices(&chosen["todo"]).contains(&"slot.feat.1.class.0".to_owned()));

    let after = sheet(&chosen["build"]).await;
    let pick = at(&after["levels"][0]["picks"], "slot.feat.1.class.0");
    assert_eq!(pick["value"], "power-attack");
    assert_eq!(pick["name"], "power-attack");
    // The engine's own edit: the code is the fighter with that pick and nothing else.
    let mut expected = fighter.clone();
    expected
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    assert_eq!(decode(&text(&chosen["build"])), expected);
}

#[tokio::test]
async fn an_unmet_feat_is_rejected_with_its_reason_and_the_code_unchanged() {
    let fighter = build("human", &["fighter"]);
    let refused = choose(
        &fighter,
        json!([{ "choice": "slot.feat.1.general.0", "value": "cleave" }]),
    )
    .await;
    assert_eq!(refused["applied"], 0);
    assert_eq!(
        refused["rejected"],
        json!({
            "step": 0,
            "choice": "slot.feat.1.general.0",
            "issue": "unmet",
            "reason": "Needs power-attack",
        })
    );
    assert_eq!(refused["build"], code(&fighter));
}

#[tokio::test]
async fn an_unoffered_value_is_not_an_option_with_the_closest_names() {
    let fighter = build("human", &["fighter"]);
    let refused = choose(
        &fighter,
        json!([{ "choice": "slot.feat.1.general.0", "value": "power-atack" }]),
    )
    .await;
    assert_eq!(refused["applied"], 0);
    assert_eq!(refused["rejected"]["issue"], "not_an_option");
    let reason = text(&refused["rejected"]["reason"]);
    assert!(reason.contains("Closest:"), "{reason}");
    assert!(reason.contains("\"power-attack\""), "{reason}");
    assert_eq!(refused["build"], code(&fighter));

    // A value of the wrong form: a class is `{class, …}`, not a key.
    let empty = build("human", &[]);
    let refused = choose(
        &empty,
        json!([{ "choice": "level.1.class", "value": "fighter" }]),
    )
    .await;
    let reason = text(&refused["rejected"]["reason"]);
    assert!(reason.contains("{\"class\":\"fighter\"}"), "{reason}");
}

#[tokio::test]
async fn a_name_that_shares_only_a_word_gets_no_closest_names() {
    let fighter = build("human", &["fighter"]);
    let refused = choose(
        &fighter,
        json!([{ "choice": "slot.feat.1.general.0", "value": "furious-focus" }]),
    )
    .await;
    assert_eq!(refused["rejected"]["issue"], "not_an_option");
    let reason = text(&refused["rejected"]["reason"]);
    assert!(!reason.contains("Closest"), "{reason}");
    assert!(!reason.contains("weapon-focus"), "{reason}");
    assert!(
        reason.contains("no offered value is close to it"),
        "{reason}"
    );
}

#[tokio::test]
async fn a_batch_stops_at_its_first_failure() {
    let fighter = build("human", &["fighter"]);
    let batch = choose(
        &fighter,
        json!([
            { "choice": "slot.feat.1.class.0", "value": "power-attack" },
            { "choice": "level.2.class", "value": { "class": "fighter" } },
            // Great Cleave needs Cleave and BAB +4.
            { "choice": "slot.feat.1.general.0", "value": "great-cleave" },
            { "choice": "slot.feat.1.general.0", "value": "toughness" },
        ]),
    )
    .await;
    assert_eq!(batch["applied"], 2);
    assert_eq!(batch["rejected"]["step"], 2);
    assert_eq!(batch["rejected"]["choice"], "slot.feat.1.general.0");
    assert_eq!(batch["rejected"]["issue"], "unmet");
    // The code as it stood before the failing step: the first two applied, the last not.
    let mut expected = fighter.clone();
    expected
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    expected
        .levels
        .push(rules::LevelUp::new(ClassKey::new("fighter")));
    assert_eq!(decode(&text(&batch["build"])), expected);

    // An address the Build lacks stops a batch too, as a rejection.
    let lacking = choose(
        &fighter,
        json!([
            { "choice": "slot.feat.1.class.0", "value": "power-attack" },
            { "choice": "slot.feat.9.general.0", "value": "toughness" },
        ]),
    )
    .await;
    assert_eq!(lacking["applied"], 1);
    assert_eq!(lacking["rejected"]["issue"], "no_such_choice");
}

#[tokio::test]
async fn no_level_is_added_past_the_level_cap() {
    // The engine's class picker offers level 21 too; a 21-level Build isn't possible without a Legend switch.
    let twenty = build("human", &times("fighter", 20));
    let refused = choose(
        &twenty,
        json!([{ "choice": "level.21.class", "value": { "class": "fighter" } }]),
    )
    .await;
    assert_eq!(refused["rejected"]["issue"], "no_such_choice");
    assert!(
        text(&refused["rejected"]["reason"]).contains("Level 20 is the last level"),
        "{refused}"
    );
    assert_eq!(refused["build"], code(&twenty));
}

#[tokio::test]
async fn a_forged_impossible_code_is_a_stale_link() {
    // Cleave without Power Attack: decodes, but isn't possible.
    let mut forged = build("human", &["fighter"]);
    forged.picks.insert(general_slot(1), pick_feat("cleave"));
    let result = Client::on_the_test_world()
        .await
        .call(
            "choose",
            json!({ "build": code(&forged), "steps": [
                { "choice": "slot.feat.1.class.0", "value": "power-attack" },
            ] }),
        )
        .await;
    assert_eq!(error_code(&result), "stale_link");
}

#[tokio::test]
async fn a_score_steps_several_points_one_at_a_time() {
    // Int 13 at 18 of 25 points: 14 costs 2 more, 15 another 2, 16 another 3, so 16 spends exactly 25.
    let fighter = build("human", &["fighter"]);
    let raised = choose(
        &fighter,
        json!([{ "choice": "start.score.int", "value": 16 }]),
    )
    .await;
    assert_eq!(raised["applied"], 1, "{raised}");
    assert_eq!(
        decode(&text(&raised["build"]))
            .start
            .ability_scores
            .get(rules::Ability::Intelligence),
        16
    );
    let after = sheet(&raised["build"]).await;
    assert_eq!(after["point_buy_spent"], 25);

    // Down several points works the same way.
    let lowered = choose(
        &fighter,
        json!([{ "choice": "start.score.int", "value": 10 }]),
    )
    .await;
    assert_eq!(
        decode(&text(&lowered["build"]))
            .start
            .ability_scores
            .get(rules::Ability::Intelligence),
        10
    );

    // The same value is a no-op, counted as applied.
    let same = choose(
        &fighter,
        json!([{ "choice": "start.score.int", "value": 13 }]),
    )
    .await;
    assert_eq!(same["applied"], 1);
    assert_eq!(same["build"], code(&fighter));
}

#[tokio::test]
async fn stepping_past_the_point_buy_budget_is_rejected_at_the_step_that_breaks_it() {
    let fighter = build("human", &["fighter"]);
    let refused = choose(
        &fighter,
        json!([{ "choice": "start.score.int", "value": 18 }]),
    )
    .await;
    assert_eq!(refused["applied"], 0);
    assert_eq!(refused["rejected"]["issue"], "not_an_option");
    let reason = text(&refused["rejected"]["reason"]);
    assert!(reason.contains("Intelligence 16 → 17"), "{reason}");
    assert!(reason.contains("25"), "{reason}");
    // The step's code as it stood: Int 13, not the 16 reached on the way.
    assert_eq!(refused["build"], code(&fighter));
}

#[tokio::test]
async fn a_step_that_breaks_a_later_choice_is_rejected_with_the_engines_issue() {
    // Str 16 with Power Attack (Str 13): down to 13 is fine, 12 breaks it.
    let mut strong = build("human", &["fighter"]);
    strong
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    let refused = choose(
        &strong,
        json!([{ "choice": "start.score.str", "value": 10 }]),
    )
    .await;
    assert_eq!(refused["rejected"]["issue"], "breaks_later");
    let reason = text(&refused["rejected"]["reason"]);
    assert!(reason.contains("Strength 13 → 12"), "{reason}");
    assert_eq!(refused["build"], code(&strong));
}

#[tokio::test]
async fn skill_ranks_step_too() {
    let fighter = build("human", &["fighter"]);
    let ranked = choose(
        &fighter,
        json!([{ "choice": "level.1.skill.athletics", "value": 1 }]),
    )
    .await;
    assert_eq!(ranked["applied"], 1, "{ranked}");
    let after = decode(&text(&ranked["build"]));
    assert_eq!(
        after.levels[0]
            .skill_ranks
            .get(&rules::SkillKey::new("athletics")),
        Some(&1)
    );
    // No skill's total goes above the character level.
    let refused = choose(
        &fighter,
        json!([{ "choice": "level.1.skill.athletics", "value": 2 }]),
    )
    .await;
    assert_eq!(refused["rejected"]["issue"], "not_an_option");
    assert_eq!(refused["build"], code(&fighter));
}

#[tokio::test]
async fn a_prestige_class_needs_its_advances() {
    let wizard = build("human", &times("wizard", 3));
    let refused = choose(
        &wizard,
        json!([{ "choice": "level.4.class", "value": { "class": "eldritch-knight" } }]),
    )
    .await;
    assert_eq!(refused["rejected"]["issue"], "not_an_option");
    let reason = text(&refused["rejected"]["reason"]);
    assert!(reason.contains("advances"), "{reason}");
    assert!(reason.contains("[\"wizard\"]"), "{reason}");
    assert!(reason.contains("[]"), "{reason}");
    assert_eq!(refused["build"], code(&wizard));

    let taken = choose(
        &wizard,
        json!([{ "choice": "level.4.class", "value": { "class": "eldritch-knight", "advances": ["wizard"] } }]),
    )
    .await;
    assert_eq!(taken["applied"], 1, "{taken}");
    let after = sheet(&taken["build"]).await;
    assert_eq!(after["levels"][3]["class"]["key"], "eldritch-knight");
    let class = at(&after["levels"][3]["picks"], "level.4.class");
    assert_eq!(
        class["value"],
        json!({ "class": "eldritch-knight", "advances": ["wizard"] })
    );

    // `[]` advances nothing.
    let none = choose(
        &wizard,
        json!([{ "choice": "level.4.class", "value": { "class": "eldritch-knight", "advances": [] } }]),
    )
    .await;
    assert_eq!(none["applied"], 1, "{none}");
}

#[tokio::test]
async fn real_data_augment_summoning_waits_for_spell_focus_conjuration() {
    let client = Client::start().await;
    let hits = client.search_for("BFT").await;
    assert_eq!(
        hits[0],
        (
            "archetype".to_owned(),
            "arcanist/brown-fur-transmuter".to_owned()
        )
    );
    let empty = client.ok("new_build", json!({})).await;
    let bft = json!({ "class": "arcanist", "archetype": "arcanist/brown-fur-transmuter" });
    let arcanist = client
        .ok(
            "choose",
            json!({ "build": empty["build"], "steps": [{ "choice": "level.1.class", "value": bft }] }),
        )
        .await;
    assert_eq!(arcanist["applied"], 1, "{arcanist}");

    // Augment Summoning first: it needs Spell Focus (Conjuration).
    let early = client
        .ok(
            "choose",
            json!({ "build": arcanist["build"], "steps": [
                { "choice": "slot.feat.1.general.0", "value": "augment-summoning" },
            ] }),
        )
        .await;
    assert_eq!(early["applied"], 0);
    assert_eq!(early["rejected"]["issue"], "unmet");
    let reason = text(&early["rejected"]["reason"]);
    assert!(reason.contains("Spell Focus"), "{reason}");
    assert_eq!(early["build"], arcanist["build"]);

    // Spell Focus (Conjuration) at level 1, then Augment Summoning at level 3's feat.
    let summoner = client
        .ok(
            "choose",
            json!({ "build": arcanist["build"], "steps": [
                { "choice": "slot.feat.1.general.0", "value": { "key": "spell-focus", "parameter": "conjuration" } },
                { "choice": "level.2.class", "value": { "class": "arcanist" } },
                { "choice": "level.3.class", "value": { "class": "arcanist" } },
                { "choice": "slot.feat.3.general.0", "value": "augment-summoning" },
            ] }),
        )
        .await;
    assert_eq!(summoner["applied"], 4, "{summoner}");
    assert!(summoner.get("rejected").is_none(), "{summoner}");
    let after = client
        .ok("sheet", json!({ "build": summoner["build"] }))
        .await;
    assert_eq!(
        at(&after["levels"][2]["picks"], "slot.feat.3.general.0")["name"],
        "Augment Summoning"
    );
}
