//! Ranking by a named number (ticket 127): `compare`'s `by` orders Builds by a number of the caster or attack line,
//! and `improve` orders a choice's open options by the change each makes to it, on the Max DC run's Build.

use rmcp::serde_json::{Value, json};
use rules::test_world::{build, class_slot, pick_feat, pick_feat_for, times, weapon};
use rules::{Ability, Build};

use crate::support::*;

/// The Max DC run's Build (2026-11-09): a Kitsune Sorcerer 20, Arcane bloodline, Second Bloodline Fey at MR 1,
/// Spell Focus, Greater and Mythic (MR 2) in Enchantment, Expanded Arsenal (Conjuration) at MR 4, School Power
/// Enchantment.
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

fn compulsion_dc() -> Value {
    json!({ "caster_line": { "school": "enchantment", "subschool": "compulsion" } })
}

/// The run's Build with another second bloodline at MR 1.
async fn second_bloodline(client: &Client, bloodline: &str) -> String {
    let result = client
        .ok(
            "choose",
            json!({
                "build": MAX_DC_RUN,
                "steps": [{ "choice": "slot.mythic-selection.1.0", "value": bloodline }],
            }),
        )
        .await;
    assert_eq!(result["applied"], 1, "{result}");
    text(&result["build"])
}

#[tokio::test]
async fn compare_by_the_compulsion_dc_orders_three_bloodline_branches() {
    let client = Client::start().await;
    let infernal = second_bloodline(&client, "sorcerer/bloodline/infernal").await;
    let celestial = second_bloodline(&client, "sorcerer/bloodline/celestial").await;
    let compared = client
        .ok(
            "compare",
            json!({ "builds": [infernal, celestial, MAX_DC_RUN], "by": compulsion_dc() }),
        )
        .await;

    // Fey's arcana adds +2 to compulsions; Infernal's to charms, so it ties with Celestial, in the order given.
    let ranked: Vec<(Value, Value)> = compared["builds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|summary| (summary["given"].clone(), summary["number"].clone()))
        .collect();
    assert_eq!(
        ranked,
        [
            (json!(2), json!(34)),
            (json!(0), json!(32)),
            (json!(1), json!(32))
        ]
    );
    assert_eq!(compared["builds"][0]["build"], MAX_DC_RUN);
    // The differences follow the same order.
    let rank_1 = compared["ranks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|diff| diff["rank"] == 1)
        .unwrap();
    assert_eq!(
        rank_1["picks"][0]["names"],
        json!(["Fey", "Infernal", "Celestial"])
    );
    // The result says what the number is and what it leaves out.
    let by = &compared["by"];
    assert!(
        text(&by["number"]).contains("Enchantment (compulsion)"),
        "{by}"
    );
    assert!(text(&by["order"]).contains("highest"), "{by}");
    assert!(
        by["leaves_out"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| text(item).contains("gaze")),
        "{by}"
    );
}

#[tokio::test]
async fn compare_without_by_keeps_the_order_given_and_no_number() {
    let client = Client::start().await;
    let infernal = second_bloodline(&client, "sorcerer/bloodline/infernal").await;
    let compared = client
        .ok("compare", json!({ "builds": [infernal, MAX_DC_RUN] }))
        .await;

    assert_eq!(compared["builds"][1]["build"], MAX_DC_RUN);
    assert_eq!(compared["builds"][0].get("number"), None);
    assert_eq!(compared["builds"][0].get("given"), None);
    assert_eq!(compared.get("by"), None);
}

#[tokio::test]
async fn improve_on_the_mr_4_mythic_feat_slot_lists_spell_focus_mythic_first_for_the_enchantment_dc()
 {
    let client = Client::start().await;
    // Without the MR 2 Spell Focus (Mythic), MR 4 (Expanded Arsenal (Conjuration) now) can take it.
    let without = client
        .ok(
            "withdraw",
            json!({ "build": MAX_DC_RUN, "choice": "slot.mythic.2" }),
        )
        .await;
    let improved = client
        .ok(
            "improve",
            json!({ "build": text(&without["build"]), "choice": "slot.mythic.4", "by": compulsion_dc() }),
        )
        .await;

    assert_eq!(improved["current"], 33);
    assert_eq!(
        improved["options"][0],
        json!({
            "value": { "key": "spell-focus-mythic", "parameter": "enchantment" },
            "name": "Spell Focus (Mythic) (Enchantment)",
            "number": 34,
            "change": 1,
        })
    );
    let no_change = improved["no_change"].as_array().unwrap();
    for name in [
        "Expanded Arsenal (Conjuration)",
        "School Mastery (Enchantment)",
    ] {
        assert!(no_change.contains(&json!(name)), "{improved}");
    }
    assert!(improved["closed"].as_u64().unwrap() > 0);
    assert!(improved["leaves_out"].is_array());
}

#[tokio::test]
async fn improve_lists_what_replacing_a_held_pick_loses() {
    let client = Client::start().await;
    let improved = client
        .ok(
            "improve",
            json!({ "build": MAX_DC_RUN, "choice": "slot.mythic.2", "by": compulsion_dc() }),
        )
        .await;

    assert_eq!(improved["current"], 34);
    assert_eq!(
        improved["no_change"],
        json!(["Spell Focus (Mythic) (Enchantment)"])
    );
    let options = improved["options"].as_array().unwrap();
    assert!(!options.is_empty());
    assert!(
        options
            .iter()
            .all(|line| line["change"] == -1 && line["number"] == 33),
        "{improved}"
    );
}

#[tokio::test]
async fn improve_gives_conditional_effects_no_change() {
    let client = Client::start().await;
    // MR 4's superpower: Favorable Magic changes the odds, not the DC.
    let improved = client
        .ok(
            "improve",
            json!({ "build": MAX_DC_RUN, "choice": "slot.path-selection.4.0", "by": compulsion_dc() }),
        )
        .await;

    assert_eq!(improved["options"], json!([]));
    assert!(
        improved["no_change"]
            .as_array()
            .unwrap()
            .contains(&json!("Favorable Magic")),
        "{improved}"
    );
}

#[tokio::test]
async fn improve_ranks_by_spell_penetration_and_filters_by_query() {
    let client = Client::start().await;
    let improved = client
        .ok(
            "improve",
            json!({
                "build": MAX_DC_RUN,
                "choice": "slot.mythic.6",
                "by": { "caster_line": { "school": "enchantment", "number": "spell_penetration" } },
                "query": "spell",
            }),
        )
        .await;

    // Spell Penetration (Mythic), held, adds the mythic rank (10) after Greater Spell Penetration.
    assert_eq!(improved["current"], 36);
    assert!(
        improved["no_change"]
            .as_array()
            .unwrap()
            .contains(&json!("Spell Penetration (Mythic)")),
        "{improved}"
    );
    let options = improved["options"].as_array().unwrap();
    assert!(
        options
            .iter()
            .all(|line| text(&line["name"]).to_lowercase().contains("spell")
                && line["change"] == -10),
        "{improved}"
    );
}

/// A human fighter 4 with Str 16: Power Attack at 1, Weapon Focus (Longsword) at 2, +1 Strength at 4.
fn fighter() -> Build {
    let mut fighter = build("human", &times("fighter", 4));
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    fighter.picks.insert(
        class_slot(2, 0),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    fighter.levels[3].ability_increase = Some(Ability::Strength);
    fighter
}

#[tokio::test]
async fn improve_and_compare_rank_by_the_attack_line() {
    let client = Client::on_the_test_world().await;
    let fighter = code(&fighter());
    let improved = client
        .ok(
            "improve",
            json!({
                "build": fighter,
                "choice": "slot.feat.2.class.0",
                "by": { "attack_line": { "weapon": "longsword" } },
            }),
        )
        .await;

    // BAB +4, Str +3, Weapon Focus +1: 8. Any other feat there loses Weapon Focus's +1.
    assert_eq!(improved["current"], 8);
    assert!(
        improved["no_change"]
            .as_array()
            .unwrap()
            .contains(&json!("weapon-focus (longsword)")),
        "{improved}"
    );
    assert!(
        improved["options"]
            .as_array()
            .unwrap()
            .iter()
            .all(|line| line["change"] == -1),
        "{improved}"
    );

    let mut without_focus = self::fighter();
    without_focus.picks.remove(&class_slot(2, 0));
    let compared = client
        .ok(
            "compare",
            json!({
                "builds": [code(&without_focus), fighter],
                "by": { "attack_line": { "weapon": "longsword", "number": "attack" } },
            }),
        )
        .await;
    assert_eq!(compared["builds"][0]["given"], 1);
    assert_eq!(compared["builds"][0]["number"], 8);
    assert_eq!(compared["builds"][1]["number"], 7);
    assert!(text(&compared["by"]["number"]).contains("first attack"));
}

#[tokio::test]
async fn compare_by_the_attack_line_without_a_weapon_reads_each_builds_weapon_focus_weapon() {
    let client = Client::on_the_test_world().await;
    let mut dagger = fighter();
    dagger.picks.insert(
        class_slot(2, 0),
        pick_feat_for("weapon-focus", weapon("dagger")),
    );
    let mut unfocused = fighter();
    unfocused.picks.remove(&class_slot(2, 0));
    let compared = client
        .ok(
            "compare",
            json!({
                "builds": [code(&unfocused), code(&dagger), code(&fighter())],
                "by": { "attack_line": { "number": "attack" } },
            }),
        )
        .await;

    // Each Build's own Weapon Focus weapon: BAB +4, Str +3, Weapon Focus +1, the dagger's as the longsword's (no
    // Weapon Finesse), so in the order given. No Weapon Focus: no line, last.
    let ranked: Vec<(Value, Value, Value)> = compared["builds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|summary| {
            (
                summary["given"].clone(),
                summary["number"].clone(),
                summary["attack"]["weapon"]["key"].clone(),
            )
        })
        .collect();
    assert_eq!(
        ranked,
        [
            (json!(1), json!(8), json!("dagger")),
            (json!(2), json!(8), json!("longsword")),
            (json!(0), Value::Null, Value::Null),
        ]
    );
    assert!(
        text(&compared["by"]["number"]).contains("Weapon Focus"),
        "{compared}"
    );
}

#[tokio::test]
async fn improve_by_the_attack_line_without_a_weapon_reads_the_builds_weapon_focus_weapon() {
    let client = Client::on_the_test_world().await;
    let improved = client
        .ok(
            "improve",
            json!({
                "build": code(&fighter()),
                "choice": "slot.feat.1.class.0",
                "by": { "attack_line": {} },
            }),
        )
        .await;

    // The longsword's line, the Build's Weapon Focus weapon, read for every option.
    assert_eq!(improved["current"], 8, "{improved}");
    assert!(
        text(&improved["number"]).contains("longsword"),
        "{improved}"
    );
}

#[tokio::test]
async fn a_number_that_isnt_one_and_a_choice_the_build_lacks_are_refused() {
    let client = Client::start().await;
    for (choice, by, code) in [
        (
            "slot.mythic.4",
            json!({ "caster_line": { "school": "chronomancy" } }),
            "invalid_arguments",
        ),
        (
            "slot.mythic.4",
            json!({ "attack_line": { "weapon": "no-such-weapon" } }),
            "invalid_arguments",
        ),
        ("level.21.class", compulsion_dc(), "no_such_choice"),
    ] {
        let result = client
            .call(
                "improve",
                json!({ "build": MAX_DC_RUN, "choice": choice, "by": by }),
            )
            .await;
        assert_eq!(error_code(&result), code, "{result:?}");
    }
    let result = client
        .call(
            "compare",
            json!({ "builds": [MAX_DC_RUN], "by": { "caster_line": { "school": "chronomancy" } } }),
        )
        .await;
    assert_eq!(error_code(&result), "invalid_arguments");
}

#[tokio::test]
async fn the_instructions_say_a_ranking_by_a_number_is_a_fact() {
    let client = Client::start().await;
    let instructions = client.instructions();

    assert!(instructions.contains("`improve`"), "{instructions}");
    assert!(instructions.contains("`by`"), "{instructions}");
    assert!(
        instructions.contains("that ranking is a fact"),
        "{instructions}"
    );
    assert!(
        instructions.contains("without ranking them by your own judgement"),
        "{instructions}"
    );
}
