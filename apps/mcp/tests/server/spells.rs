//! The spell catalogue (ticket 121): `search` and `describe` with `kind: spell` or `spell_list`, a class's spell
//! lists, and what the instructions say of it. On the real data.

use rmcp::serde_json::{Value, json};

use crate::support::*;

/// The `key`s of a list of `{key, …}`.
fn keys(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["key"]))
        .collect()
}

fn pair(kind: &str, key: &str) -> (String, String) {
    (kind.to_owned(), key.to_owned())
}

#[tokio::test]
async fn hold_monster_is_found_by_name_with_its_school_descriptors_and_summary() {
    let client = Client::start().await;
    let found = client
        .ok(
            "search",
            json!({ "query": "Hold Monster", "kinds": ["spell"] }),
        )
        .await;
    let hit = &found["matches"][0];
    assert_eq!(hit["kind"], "spell");
    assert_eq!(hit["key"], "hold-monster");
    assert_eq!(hit["name"], "Hold Monster");
    assert_eq!(hit["category"], "Enchantment");
    assert_eq!(hit["descriptors"], json!(["Compulsion", "Mind-Affecting"]));
    assert!(text(&hit["summary"]).contains("paralysed"), "{hit}");
    assert_eq!(
        hit["guide_url"],
        "https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/wizard-spells#hold-monster-enchantment"
    );
    // A spell has no prerequisite.
    assert!(hit.get("prerequisite").is_none(), "{hit}");

    // By key, without a kind.
    let matches = client.search_for("hold-monster").await;
    assert_eq!(matches.first(), Some(&pair("spell", "hold-monster")));

    // `compact` keeps the school, not the descriptors.
    let found = client
        .ok(
            "search",
            json!({ "query": "Hold Monster", "kinds": ["spell"], "compact": true }),
        )
        .await;
    let hit = &found["matches"][0];
    assert_eq!(hit["category"], "Enchantment");
    assert!(hit.get("descriptors").is_none(), "{hit}");
}

#[tokio::test]
async fn a_spells_school_or_descriptor_is_a_category() {
    let client = Client::start().await;
    let compulsions = client
        .ok(
            "search",
            json!({ "kinds": ["spell"], "category": "Compulsion", "limit": 100 }),
        )
        .await;
    let matches = compulsions["matches"].as_array().unwrap();
    assert!(keys(&compulsions["matches"]).contains(&"hold-monster".to_owned()));
    assert!(
        matches.iter().all(|hit| hit["descriptors"]
            .as_array()
            .is_some_and(|descriptors| descriptors.contains(&json!("Compulsion")))),
        "{compulsions}"
    );
    // No other item has the category: the filter alone finds only spells.
    let alone = client
        .ok("search", json!({ "category": "compulsion", "limit": 100 }))
        .await;
    assert_eq!(alone["total"], compulsions["total"]);

    // The school, with a query, and a list of categories.
    let enchantments = client
        .search(json!({ "query": "hold", "category": "Enchantment" }))
        .await;
    assert!(enchantments.contains(&pair("spell", "hold-monster")));
    assert!(enchantments.contains(&pair("spell", "hold-person")));
    assert!(enchantments.iter().all(|(kind, _)| kind == "spell"));
    let either = client
        .search(json!({ "query": "hold monster", "category": ["Necromancy", "mind-affecting"] }))
        .await;
    assert_eq!(either.first(), Some(&pair("spell", "hold-monster")));

    // Dismiss Spell's school is Universalist.
    let found = client
        .ok("search", json!({ "category": "Universalist" }))
        .await;
    assert_eq!(keys(&found["matches"]), ["dismiss-spell"]);
}

#[tokio::test]
async fn a_spell_is_found_by_its_summarys_words() {
    let client = Client::start().await;
    let found = client
        .ok(
            "search",
            json!({ "query": "paralysed", "kinds": ["spell"], "limit": 100 }),
        )
        .await;
    let hold_monster = found["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["key"] == "hold-monster")
        .unwrap_or_else(|| panic!("no Hold Monster in {found}"));
    assert_eq!(hold_monster["matched"], "summary");
}

#[tokio::test]
async fn hold_monster_is_described_with_its_facts_and_every_list_and_level() {
    let client = Client::start().await;
    let spell = client
        .ok(
            "describe",
            json!({ "kind": "spell", "key": "hold-monster" }),
        )
        .await;
    assert_eq!(spell["name"], "Hold Monster");
    assert!(text(&spell["summary"]).contains("paralysed"), "{spell}");
    assert_eq!(
        spell["facts"],
        json!({
            "school": "Enchantment",
            "descriptors": ["Compulsion", "Mind-Affecting"],
            "target": "One enemy creature within medium range",
            "duration": "1 round / level",
            "saving_throw": "Will negates",
            "spell_resistance": true,
            "casting_time": "Standard Action",
        })
    );

    let lists = spell["lists"].as_array().unwrap();
    let level_on = |list: &str| {
        lists
            .iter()
            .find(|on| on["key"] == list)
            .map(|on| on["level"].as_u64().unwrap())
    };
    assert_eq!(level_on("wizard"), Some(5));
    assert_eq!(level_on("bard"), Some(4));
    assert_eq!(level_on("inquisitor"), Some(4));
    assert_eq!(level_on("witch"), Some(5));
    assert_eq!(level_on("cleric"), None);

    let wizard = lists.iter().find(|on| on["key"] == "wizard").unwrap();
    assert_eq!(wizard["name"], "Wizard Spells");
    assert_eq!(
        wizard["owner"],
        json!({ "kind": "class", "key": "wizard", "name": "Wizard" })
    );
    assert!(
        wizard["used_by"]
            .as_array()
            .unwrap()
            .contains(&json!({ "kind": "class", "key": "sorcerer", "name": "Sorcerer" })),
        "{wizard}"
    );
    assert!(wizard.get("only").is_none(), "{wizard}");
}

#[tokio::test]
async fn a_list_entry_for_one_archetype_says_so() {
    let client = Client::start().await;
    let fireball = client
        .ok("describe", json!({ "kind": "spell", "key": "fireball" }))
        .await;
    let bard = fireball["lists"]
        .as_array()
        .unwrap()
        .iter()
        .find(|on| on["key"] == "bard")
        .unwrap_or_else(|| panic!("no bard list in {fireball}"));
    assert_eq!(bard["level"], 3);
    assert_eq!(bard["only"], "Flame Dancer only");
}

#[tokio::test]
async fn a_spell_list_is_described_with_its_spells_by_level() {
    let client = Client::start().await;
    let wizard = client
        .ok("describe", json!({ "kind": "spell_list", "key": "wizard" }))
        .await;
    assert_eq!(wizard["kind"], "spell_list");
    assert_eq!(wizard["name"], "Wizard Spells");
    assert_eq!(
        wizard["owner"],
        json!({ "kind": "class", "key": "wizard", "name": "Wizard" })
    );
    assert_eq!(
        keys(&wizard["used_by"]),
        ["arcanist", "sorcerer", "rogue/eldritch-scoundrel"]
    );
    let levels: Vec<u64> = wizard["spells"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["level"].as_u64().unwrap())
        .collect();
    assert_eq!(levels, (0..=9).collect::<Vec<u64>>());
    let fifth = &wizard["spells"][5]["spells"];
    let hold_monster = fifth
        .as_array()
        .unwrap()
        .iter()
        .find(|spell| spell["key"] == "hold-monster")
        .unwrap_or_else(|| panic!("no Hold Monster in {fifth}"));
    assert_eq!(
        hold_monster,
        &json!({
            "key": "hold-monster",
            "name": "Hold Monster",
            "school": "Enchantment",
            "descriptors": ["Compulsion", "Mind-Affecting"],
        })
    );

    // A Mythic spellbook runs to level 10, and its key is its path's.
    let lich = client
        .ok("describe", json!({ "kind": "spell_list", "key": "lich" }))
        .await;
    assert_eq!(lich["owner"]["kind"], "path");
    assert_eq!(
        lich["spells"].as_array().unwrap().last().unwrap()["level"],
        10
    );

    // `search` finds a list too.
    let found = client
        .ok(
            "search",
            json!({ "query": "lich", "kinds": ["spell_list"] }),
        )
        .await;
    assert_eq!(keys(&found["matches"]), ["lich", "lich/merged"], "{found}");
    assert_eq!(found["matches"][0]["category"], "Mythic spellbook");
    assert_eq!(found["matches"][1]["parent"], "lich");
}

#[tokio::test]
async fn a_class_archetype_or_path_names_the_lists_it_casts_from() {
    let client = Client::start().await;
    let lists = |item: &Value| keys(&item["spell_lists"]);
    let sorcerer = client
        .ok("describe", json!({ "kind": "class", "key": "sorcerer" }))
        .await;
    assert_eq!(lists(&sorcerer), ["wizard"]);
    let bard = client
        .ok("describe", json!({ "kind": "class", "key": "bard" }))
        .await;
    assert_eq!(lists(&bard), ["bard"]);
    let deceiver = client
        .ok(
            "describe",
            json!({ "kind": "archetype", "key": "arcanist/magic-deceiver" }),
        )
        .await;
    assert_eq!(lists(&deceiver), ["arcanist/magic-deceiver"]);
    let angel = client
        .ok("describe", json!({ "kind": "path", "key": "angel" }))
        .await;
    assert_eq!(lists(&angel), ["angel", "angel/merged"]);
    // A class that casts nothing has none.
    let fighter = client
        .ok("describe", json!({ "kind": "class", "key": "fighter" }))
        .await;
    assert!(fighter.get("spell_lists").is_none(), "{fighter}");
}

#[tokio::test]
async fn the_instructions_make_the_spell_catalogue_planner_data() {
    let instructions = Client::start().await.instructions();
    let boundary = instructions
        .find("The honesty boundary:")
        .expect("the honesty boundary");
    let boundary = &instructions[boundary..];
    for phrase in [
        "spell catalogue",
        "`kind: spell`",
        "saving throw",
        "spell resistance",
        "every spell list",
        "planner's summary of the guide",
        // What it doesn't hold stays the agent's own knowledge.
        "spells known or prepared",
        "spells per day",
    ] {
        assert!(boundary.contains(phrase), "no {phrase} in {boundary}");
    }
    // Spells as a whole are no longer only the agent's knowledge.
    assert!(
        !instructions.contains("spells and spell availability"),
        "{instructions}"
    );
    assert!(
        !instructions.contains("Which spells exist"),
        "{instructions}"
    );
}
