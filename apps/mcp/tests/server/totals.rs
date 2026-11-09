//! The totals (ticket 120): `sheet` gives initiative, each save and each skill's total at the last level, each with
//! what it counts, on the Max DC run's Build.

use rmcp::serde_json::json;

use crate::support::*;

/// The Max DC run's Build (2026-11-09): a Kitsune Sorcerer 20 with the Pickpocket background, Improved Initiative and
/// its mythic feat, Iron Will and 20 ranks of Persuasion.
const MAX_DC_RUN: &str = "AdWOPUgDQRCFd3bPE2OyAQnYSNzGSi0sbKzU8xqDKIg_oEiWsKAGznCeKCkkWIigEBALU1gaTCdYqCjpVNAmYCFBEAwpIldYWCro7uFPKot0KebxzfDmzfCnyGpf4Gzwsxh96C98uNmel_qCEpue8AsHyqPtY_utHU3ddLtwNBfP5dxKkt4HZi-n7nqzW-mD2_UNQ0vtpheLummEu8pXbuI8_BaajCaO-Yzkw7bN5vj1c3DHzJ8M31RO85n3zCu-eJT50BlZcJZXLMFMKzbPLUfYrJTaYyN8jQ0ZbCDJHc7Gl-yYsIUNCBHAjZT6_C0hhAF-Sq_iWnsZDkhHVAFCWJ4i-Fel6XuK8J_8O1OJMtwHGAghnoJS5KHuoaZpylNVDRBUb3i78AU";

#[tokio::test]
async fn the_sheet_gives_initiative_saves_and_skill_totals_with_what_they_count() {
    let client = Client::start().await;
    let sheet = client.ok("sheet", json!({ "build": MAX_DC_RUN })).await;
    let totals = &sheet["totals"];

    assert_eq!(
        totals["initiative"],
        json!({
            "total": 19,
            "counts": [
                { "name": "Dexterity", "value": 3 },
                { "name": "Pickpocket", "value": 2 },
                { "name": "Improved Initiative", "value": 4 },
                { "name": "Improved Initiative (Mythic)", "value": 10 },
            ],
        })
    );
    assert_eq!(totals["will"]["total"], 15);
    assert_eq!(
        totals["will"]["counts"][0],
        json!({ "name": "Base save", "value": 12 })
    );
    assert_eq!(
        totals["skills"]["persuasion"],
        json!({
            "ranks": 20,
            "total": 30,
            "counts": [
                { "name": "Ranks", "value": 20 },
                { "name": "Charisma", "value": 7 },
                { "name": "Class skill", "value": 3 },
            ],
        })
    );
    // Every skill, trained or not.
    assert_eq!(totals["skills"].as_object().unwrap().len(), 11);
    assert_eq!(totals["skills"]["mobility"]["total"], 5);

    let leaves_out = sheet["totals_leave_out"].as_array().unwrap();
    assert!(
        leaves_out.iter().any(|item| text(item).contains("gear")),
        "{leaves_out:?}"
    );
}

#[tokio::test]
async fn the_instructions_let_an_agent_state_the_totals_and_cite_interactions() {
    let client = Client::start().await;
    let instructions = client.instructions();

    assert!(instructions.contains("totals_leave_out"), "{instructions}");
    assert!(instructions.contains("`interactions`"), "{instructions}");
}
