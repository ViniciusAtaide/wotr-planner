//! Build codes: `new_build`, the envelope every Build tool returns, and codes that don't open.

use rmcp::serde_json::json;
use rules::Build;
use rules::test_world::{build, class_slot, general_slot, pick_feat};

use crate::support::*;

const LINK: &str = "https://viniciusataide.github.io/wotr-planner/#b=";

#[tokio::test]
async fn a_new_build_is_empty_with_the_build_start_to_do() {
    let client = Client::on_the_test_world().await;
    let empty = client.ok("new_build", json!({})).await;

    let empty_code = text(&empty["build"]);
    assert_eq!(empty_code, code(&Build::default()));
    assert_eq!(empty["link"], format!("{LINK}{empty_code}"));
    assert_eq!(
        choices(&empty["todo"]),
        [
            "start.race",
            "start.faith",
            "start.alignment",
            "start.background"
        ]
    );
    assert_eq!(empty["todo"][0]["label"], "Race");
    // The glossary's Faith (a deity, or atheism), not "Deity".
    assert_eq!(empty["todo"][1]["label"], "Faith");
}

#[tokio::test]
async fn a_new_build_carries_its_name() {
    let client = Client::on_the_test_world().await;
    let named = client.ok("new_build", json!({ "name": "Summoner" })).await;
    assert_eq!(
        decode(&text(&named["build"])).name.as_deref(),
        Some("Summoner")
    );
}

#[tokio::test]
async fn a_new_build_opens_a_link_or_a_bare_code() {
    let client = Client::on_the_test_world().await;
    let mut fighter = build("human", &["fighter"]);
    fighter
        .picks
        .insert(class_slot(1, 0), pick_feat("power-attack"));
    let code = code(&fighter);

    for link in [
        format!("{LINK}{code}"),
        code.clone(),
        format!(" #b={code} "),
    ] {
        let opened = client.ok("new_build", json!({ "link": link })).await;
        assert_eq!(text(&opened["build"]), code, "{link}");
    }
    // The fighter still has its racial bonus, background and level 1's general slot to choose.
    let opened = client.ok("new_build", json!({ "link": code })).await;
    let todo = choices(&opened["todo"]);
    assert!(
        todo.contains(&"start.racial-ability".to_owned()),
        "{todo:?}"
    );
    assert!(
        todo.contains(&"slot.feat.1.general.0".to_owned()),
        "{todo:?}"
    );
    assert!(
        !todo.contains(&"slot.feat.1.class.0".to_owned()),
        "{todo:?}"
    );
}

#[tokio::test]
async fn a_forged_impossible_code_is_a_stale_link() {
    let client = Client::on_the_test_world().await;
    // Cleave at level 1 without Power Attack: the code decodes, but the Build isn't possible.
    let mut forged = build("human", &["fighter"]);
    forged.picks.insert(general_slot(1), pick_feat("cleave"));
    let forged = code(&forged);

    let result = client.call("new_build", json!({ "link": forged })).await;
    assert_eq!(error_code(&result), "stale_link");
    let result = client.call("sheet", json!({ "build": forged })).await;
    assert_eq!(error_code(&result), "stale_link");
    // A code that doesn't decode at all gets the same error.
    let result = client.call("sheet", json!({ "build": "not-a-code" })).await;
    assert_eq!(error_code(&result), "stale_link");
    assert!(error_text(&result).contains("new_build"));
}
