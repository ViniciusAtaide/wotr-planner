//! The caster line (ticket 119): `sheet` and `compare` take an optional `caster` (a school, and optionally a
//! subschool, an energy type and metamagic) and show the engine's line for it, on the Max DC run's Build.

use rmcp::serde_json::{Value, json};

use crate::support::*;

/// The Max DC run's Build (2026-11-09): a Kitsune Sorcerer 20, Arcane bloodline, Second Bloodline Fey, Spell Focus,
/// Greater and Mythic in Enchantment, School Power Enchantment.
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

fn compulsion() -> Value {
    json!({ "school": "enchantment", "subschool": "compulsion" })
}

#[tokio::test]
async fn the_sheet_gives_each_level_the_enchantment_compulsion_dcs_with_their_breakdown() {
    let client = Client::start().await;
    let sheet = client
        .ok(
            "sheet",
            json!({ "build": MAX_DC_RUN, "caster": compulsion() }),
        )
        .await;
    let caster = |level: usize| sheet["levels"][level]["caster"].clone();

    // Level 1: 10 + 1 + Cha +5, Kitsune +1, Spell Focus +1.
    assert_eq!(caster(0)["dcs"], json!([{ "spell_level": 1, "dc": 18 }]));
    assert_eq!(
        caster(0)["class"],
        json!({ "key": "sorcerer", "name": "Sorcerer" })
    );
    assert_eq!(caster(0)["ability"], "Charisma");
    // Level 20: 9th-level spells at DC 34, Heighten Spell held.
    let last = caster(19);
    assert_eq!(last["caster_level"], 20);
    assert_eq!(last["dcs"][8], json!({ "spell_level": 9, "dc": 34 }));
    assert_eq!(last["heightened"], json!({ "spell_level": 9, "dc": 34 }));
    assert_eq!(
        last["dc_counts"],
        json!([
            { "name": "Charisma", "value": 7 },
            { "name": "Kitsune", "value": 1 },
            { "name": "Greater Spell Focus (Enchantment)", "value": 1 },
            { "name": "Spell Focus (Enchantment)", "value": 1 },
            { "name": "Spell Focus (Mythic) (Enchantment)", "value": 1 },
            { "name": "Fey Arcana", "value": 2 },
            { "name": "School Power (Enchantment)", "value": 2 },
        ])
    );
    assert_eq!(last["spell_penetration"], 36);
    assert_eq!(last["concentration"], 31);

    let leaves_out = sheet["caster_leaves_out"].as_array().unwrap();
    assert!(
        leaves_out.iter().any(|item| text(item).contains("gaze")),
        "{leaves_out:?}"
    );
}

#[tokio::test]
async fn with_metamagic_the_arcane_arcana_adds_one() {
    let client = Client::start().await;
    let sheet = client
        .ok(
            "sheet",
            json!({
                "build": MAX_DC_RUN,
                "caster": { "school": "enchantment", "subschool": "compulsion", "metamagic": true },
            }),
        )
        .await;

    assert_eq!(sheet["levels"][19]["caster"]["dcs"][8]["dc"], 35);
}

#[tokio::test]
async fn without_a_caster_the_sheet_has_no_caster_line() {
    let client = Client::start().await;
    let sheet = client.ok("sheet", json!({ "build": MAX_DC_RUN })).await;

    assert_eq!(sheet["levels"][19].get("caster"), None);
    assert_eq!(sheet.get("caster_leaves_out"), None);
}

#[tokio::test]
async fn compare_gives_each_builds_summary_its_caster_line() {
    let client = Client::start().await;
    let compared = client
        .ok(
            "compare",
            json!({ "builds": [MAX_DC_RUN], "caster": compulsion() }),
        )
        .await;

    let caster = &compared["builds"][0]["caster"];
    assert_eq!(caster["dcs"][8]["dc"], 34);
    assert!(
        caster["dc_counts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|part| part["name"] == "Fey Arcana"),
        "{caster}"
    );
    assert!(compared["caster_leaves_out"].is_array());
}

#[tokio::test]
async fn an_unknown_school_and_a_subschool_of_another_school_are_refused() {
    let client = Client::start().await;
    for (caster, named) in [
        (json!({ "school": "chronomancy" }), "chronomancy"),
        (
            json!({ "school": "evocation", "subschool": "charm" }),
            "Enchantment",
        ),
        (json!({ "school": "evocation", "energy": "sonic" }), "sonic"),
    ] {
        let result = client
            .call("sheet", json!({ "build": MAX_DC_RUN, "caster": caster }))
            .await;

        assert_eq!(error_code(&result), "invalid_arguments");
        assert!(error_text(&result).contains(named), "{result:?}");
    }
}

#[tokio::test]
async fn the_instructions_let_an_agent_state_the_caster_line_as_fact() {
    let client = Client::start().await;
    let instructions = client.instructions();

    assert!(instructions.contains("caster line"), "{instructions}");
    assert!(instructions.contains("caster_leaves_out"), "{instructions}");
}
