//! End to end on the real game data: the user's example prompt, driven through the MCP interface the way an agent
//! would drive it (plan, ticket 104, section 6; ticket 111).
//!
//! > a BFT main character, buffs and summoning, Lich or Aeon (compare), summons lasting 24h through caster level 25
//! > and Extend, buffed with the mass animal buffs
//!
//! The agent resolves "BFT", builds a 20-level Brown-Fur Transmuter arcanist with the summoning feats, reaches MR 10,
//! branches at the Mythic path into a Lich and an Aeon, and compares the two. It finds what to take with `options`
//! wherever it can, and every pick goes through `choose`. What the prompt implies but the planner doesn't model
//! (a "mass" animal buff the guide lacks, durations, caster level bonuses) is asserted absent, with the reason.

use rmcp::serde_json::{Value, json};

use crate::support::*;

/// The planner's links.
const LINK_BASE: &str = "https://viniciusataide.github.io/wotr-planner/#b=";

/// An agent holding one Build's code, which every `choose` it makes replaces.
struct Agent<'a> {
    client: &'a Client,
    build: Value,
}

impl<'a> Agent<'a> {
    fn on(client: &'a Client, build: &Value) -> Self {
        Self {
            client,
            build: build.clone(),
        }
    }

    /// `choose` with these steps, every one of which must apply.
    async fn choose(&mut self, steps: Value) -> Value {
        let result = self.try_choose(steps.clone()).await;
        assert!(result.get("rejected").is_none(), "{steps} → {result}");
        self.build = result["build"].clone();
        result
    }

    /// `choose` with these steps, keeping the code as it was whatever happens.
    async fn try_choose(&self, steps: Value) -> Value {
        self.client
            .ok("choose", json!({ "build": self.build, "steps": steps }))
            .await
    }

    /// The options open now for a choice (a parameterised feat once per value), filtered by a query.
    async fn options(&self, choice: &str, query: Option<&str>) -> Vec<Value> {
        let mut arguments = json!({ "build": self.build, "choice": choice, "values": true });
        if let Some(query) = query {
            arguments["query"] = json!(query);
        }
        let options = self.client.ok("options", arguments).await;
        options["options"].as_array().unwrap().clone()
    }

    /// The open option a query names, as `choose` takes it: the only match, or the one whose name is the query
    /// ("enduring spells" matches Greater Enduring Spells too).
    async fn value(&self, choice: &str, query: &str) -> Value {
        let options = self.options(choice, Some(query)).await;
        let named = |option: &&Value| text(&option["name"]).eq_ignore_ascii_case(query);
        let option = match &options[..] {
            [only] => only,
            _ => options
                .iter()
                .find(named)
                .unwrap_or_else(|| panic!("{query} at {choice}: {options:?}")),
        };
        option["value"].clone()
    }

    /// Takes the first preferred option that's open at a choice (by key), or else the first open one.
    async fn take(&mut self, choice: &str, preferred: &[&str]) -> Value {
        let open = self.options(choice, None).await;
        let value = preferred
            .iter()
            .find_map(|key| open.iter().find(|option| key_of(&option["value"]) == *key))
            .or(open.first())
            .unwrap_or_else(|| panic!("nothing open at {choice}"))["value"]
            .clone();
        self.choose(json!([{ "choice": choice, "value": value }]))
            .await;
        value
    }

    /// Fills every open choice in `todo` whose address starts with `prefix`, as `take` does.
    async fn fill(&mut self, prefix: &str, preferred: &[&str]) {
        while let Some(choice) = self
            .todo()
            .await
            .into_iter()
            .find(|choice| choice.starts_with(prefix))
        {
            self.take(&choice, preferred).await;
        }
    }

    async fn todo(&self) -> Vec<String> {
        let sheet = self.sheet().await;
        choices(&sheet["todo"])
    }

    async fn sheet(&self) -> Value {
        self.client
            .ok("sheet", json!({ "build": self.build }))
            .await
    }
}

/// A value's key: a key value itself, or the `key` of a `{key, parameter}`.
fn key_of(value: &Value) -> &str {
    value
        .as_str()
        .or_else(|| value["key"].as_str())
        .unwrap_or_default()
}

fn names(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap_or_else(|| panic!("{list} is a list"))
        .iter()
        .map(|item| text(&item["name"]))
        .collect()
}

/// The skills a summoner puts ranks in, in order of preference.
const SKILLS: [&str; 11] = [
    "knowledge-arcana",
    "lore-nature",
    "perception",
    "use-magic-device",
    "persuasion",
    "knowledge-world",
    "lore-religion",
    "mobility",
    "athletics",
    "stealth",
    "trickery",
];

/// The general feats left after the summoning ones, in order of preference.
const FEATS: [&str; 7] = [
    "greater-spell-focus",
    "spell-penetration",
    "greater-spell-penetration",
    "metamagic-quicken-spell",
    "extra-arcanist-exploit",
    "toughness",
    "metamagic-persistent-spell",
];

/// The exploits, in order of preference.
const EXPLOITS: [&str; 5] = [
    "arcanist/arcanist-exploits/potent-magic",
    "arcanist/arcanist-exploits/swift-consume",
    "arcanist/arcanist-exploits/spell-resistance",
    "arcanist/arcanist-exploits/arcane-barrier",
    "arcanist/greater-exploits/greater-spell-resistance",
];

#[tokio::test]
async fn the_users_prompt_bft_summoner_lich_or_aeon() {
    let client = Client::start().await;

    // "BFT": initials. The archetype is the first match; an agent asks the user when a name could mean more.
    let hits = client.search_for("BFT").await;
    assert_eq!(
        hits[0],
        (
            "archetype".to_owned(),
            "arcanist/brown-fur-transmuter".to_owned()
        )
    );

    // The Build start: a human with +2 Intelligence, an atheist of true neutral with the Arcane background, and a
    // point buy with Intelligence high (Str 8, Dex 14, Con 14, Int 18: all 25 points).
    let empty = client
        .ok("new_build", json!({ "name": "BFT summoner" }))
        .await;
    let mut agent = Agent::on(&client, &empty["build"]);
    assert!(
        agent
            .options("start.race", Some("human"))
            .await
            .iter()
            .any(|option| option["value"] == "human")
    );
    let background = agent.value("start.background", "arcane").await;
    agent
        .choose(json!([
            { "choice": "start.race", "value": "human" },
            { "choice": "start.racial-ability", "value": "int" },
            { "choice": "start.faith", "value": "atheism" },
            { "choice": "start.alignment", "value": "true-neutral" },
            { "choice": "start.background", "value": background },
            { "choice": "start.score.str", "value": 8 },
            { "choice": "start.score.int", "value": 18 },
            { "choice": "start.score.dex", "value": 14 },
            { "choice": "start.score.con", "value": 14 },
        ]))
        .await;
    let start = agent.sheet().await;
    assert_eq!(start["point_buy_spent"], 25);

    // Level 1: the Brown-Fur Transmuter, as the class picker offers it.
    let bft = agent.value("level.1.class", "brown-fur").await;
    assert_eq!(
        bft,
        json!({ "class": "arcanist", "archetype": "arcanist/brown-fur-transmuter" })
    );
    agent
        .choose(json!([{ "choice": "level.1.class", "value": bft }]))
        .await;

    // Augment Summoning first, in the human's bonus feat: it needs Spell Focus (Conjuration), so it's `unmet` and
    // the code stays as it was.
    let early = agent
        .try_choose(json!([{ "choice": "slot.feat.1.race.0", "value": "augment-summoning" }]))
        .await;
    assert_eq!(early["applied"], 0);
    assert_eq!(early["rejected"]["issue"], "unmet", "{early}");
    assert!(
        text(&early["rejected"]["reason"]).contains("Spell Focus"),
        "{early}"
    );
    assert_eq!(early["build"], agent.build);

    // So Spell Focus (Conjuration) in the bonus feat, which is checked before the general one, then Augment
    // Summoning in the general feat. The parameterised feat's value comes from `options {values: true}`.
    let spell_focus = agent
        .value("slot.feat.1.race.0", "spell focus conjuration")
        .await;
    assert_eq!(
        spell_focus,
        json!({ "key": "spell-focus", "parameter": "conjuration" })
    );
    agent
        .choose(json!([
            { "choice": "slot.feat.1.race.0", "value": spell_focus },
            { "choice": "slot.feat.1.general.0", "value": "augment-summoning" },
        ]))
        .await;

    // Levels 2 to 20, one batch: the class taken with an archetype is offered with it, and `{class}` names it.
    let levels: Vec<Value> = (2..=20)
        .map(|n| json!({ "choice": format!("level.{n}.class"), "value": { "class": "arcanist" } }))
        .collect();
    let leveled = agent.choose(Value::Array(levels)).await;
    assert_eq!(leveled["applied"], 19);

    // Superior Summoning, which the data has (it needs Augment Summoning), and Metamagic (Extend Spell).
    let superior = client.search_for("superior summoning").await;
    assert_eq!(
        superior[0],
        ("feat".to_owned(), "superior-summoning".to_owned())
    );
    let extend = agent.value("slot.feat.5.general.0", "extend spell").await;
    assert_eq!(extend, "metamagic-extend-spell");
    agent
        .choose(json!([
            { "choice": "slot.feat.3.general.0", "value": "superior-summoning" },
            { "choice": "slot.feat.5.general.0", "value": extend },
        ]))
        .await;

    // Every ability increase to Intelligence, in one batch.
    let increases: Vec<Value> = [4, 8, 12, 16, 20]
        .map(|n| json!({ "choice": format!("level.{n}.increase"), "value": "int" }))
        .into();
    agent.choose(Value::Array(increases)).await;

    // The rest of the feats and the exploits, from what `options` offers, then the skill points: one batch per
    // level, a rank in each skill in turn, which stops when the level's points run out.
    agent.fill("slot.feat.", &FEATS).await;
    agent.fill("slot.selection.", &EXPLOITS).await;
    for n in 1..=20 {
        let steps: Vec<Value> = SKILLS
            .iter()
            .map(|skill| json!({ "choice": format!("level.{n}.skill"), "value": skill }))
            .collect();
        let spent = agent.try_choose(Value::Array(steps)).await;
        assert!(spent["applied"].as_u64().unwrap() >= 7, "{spent}");
        agent.build = spent["build"].clone();
    }
    let sheet = agent.sheet().await;
    let last = &sheet["levels"][19];
    assert_eq!(last["class_level"], 20);
    assert_eq!(last["scores"]["int"], 25);

    // The Mythic ranks are reached (MR 10 at level 19), and only the Mythic choices are left open.
    let todo = agent.todo().await;
    assert!(
        todo.iter()
            .all(|choice| choice.starts_with("slot.mythic.") || choice.starts_with("mythic.")),
        "{todo:?}"
    );
    assert!(todo.contains(&"slot.mythic.10".to_owned()), "{todo:?}");

    // The first ascension, offered for every path; this Build takes the Lich's (Danse Macabre), which both
    // branches keep at MR 1–2. Then Enduring Spells at MR 1 and Spell Focus (Mythic) in Conjuration at MR 2.
    let ascensions = agent.options("mythic.first-ascension", None).await;
    for path in ["lich", "aeon"] {
        assert!(
            ascensions.iter().any(|option| option["value"] == path),
            "{ascensions:?}"
        );
    }
    let enduring = agent.value("slot.mythic.1", "enduring spells").await;
    assert_eq!(enduring, "enduring-spells");
    let mythic_focus = agent
        .value("slot.mythic.2", "spell focus conjuration")
        .await;
    agent
        .choose(json!([
            { "choice": "mythic.first-ascension", "value": "lich" },
            { "choice": "slot.mythic.1", "value": enduring },
            { "choice": "slot.mythic.2", "value": mythic_focus },
        ]))
        .await;
    let trunk = agent.build.clone();

    // Branch at the path: the same code, once per path. Each branch takes Greater Enduring Spells at MR 3 and
    // Ascendant Summons at MR 5, its path's picks (the Aeon's Summoner's Gaze where it opens, at MR 5), and fills
    // the rest from what `options` offers.
    let mut branches = Vec::new();
    for path in ["lich", "aeon"] {
        let mut branch = Agent::on(&client, &trunk);
        branch
            .choose(json!([{ "choice": "mythic.path", "value": path }]))
            .await;
        let greater = branch
            .value("slot.mythic.3", "greater enduring spells")
            .await;
        let ascendant = branch.value("slot.mythic.5", "ascendant summons").await;
        branch
            .choose(json!([
                { "choice": "slot.mythic.3", "value": greater },
                { "choice": "slot.mythic.5", "value": ascendant },
            ]))
            .await;
        branch
            .fill(
                "slot.path-selection.",
                &[
                    "lich/lich-powers/deadly-magic",
                    "aeon/enforcing-gaze/enforcing-gaze-summoners-gaze",
                ],
            )
            .await;
        branch.fill("slot.", &[]).await;
        let todo = branch.todo().await;
        assert_eq!(todo, Vec::<String>::new(), "{path}: every choice made");
        branches.push(branch.build.clone());
    }
    let [lich, aeon] = [branches[0].clone(), branches[1].clone()];

    // Compare the branches: facts from the two sheets.
    let compared = client
        .ok("compare", json!({ "builds": [lich, aeon] }))
        .await;
    let builds = compared["builds"].as_array().unwrap();
    for (summary, path) in builds.iter().zip(["Lich", "Aeon"]) {
        assert_eq!(summary["path"]["name"], path);
        assert_eq!(summary["first_ascension"]["name"], "Lich");
        assert_eq!(summary["mythic_rank"], 10);
        assert_eq!(summary["character_level"], 20);
        assert_eq!(summary["open_choices"], 0);
        assert_eq!(
            summary["classes"],
            json!([{
                "class": { "key": "arcanist", "name": "Arcanist" },
                "archetype": { "key": "arcanist/brown-fur-transmuter", "name": "Brown-Fur Transmuter" },
                "levels": 20,
            }])
        );
        // "Caster level 25" isn't something the planner can check: its caster level is class levels only, with no
        // mythic, gear or buff bonus, so a 20th-level arcanist's is 20 on either path.
        assert_eq!(summary["caster_level"], 20);
        assert_eq!(summary["spell_levels"]["arcane"], 9);
    }
    // The levels' Level-ups are the same, so the differences are the path's: its pick at each rank and its features.
    assert!(
        compared["levels"]
            .as_array()
            .unwrap()
            .iter()
            .all(|level| level.get("numbers").is_none() && level.get("picks").is_none()),
        "{compared}"
    );
    let ranks = compared["ranks"].as_array().unwrap();
    let rank_3 = ranks.iter().find(|rank| rank["rank"] == 3).unwrap();
    assert_eq!(rank_3["path"], json!(["Lich", "Aeon"]));
    assert_eq!(
        at(&rank_3["picks"], "slot.path-selection.3.0")["label"],
        "Lich Powers / Enforcing Gaze"
    );
    // MR 1–2 are the shared first ascension's, so the path features differ from MR 3 on.
    let features = compared["path_features"].as_array().unwrap();
    let first = features[0]["rank"].as_u64().unwrap();
    assert_eq!(first, 3, "{compared}");
    let at_rank = |rank: u64| {
        features
            .iter()
            .find(|row| row["rank"].as_u64() == Some(rank))
            .unwrap_or_else(|| panic!("no rank {rank} in {features:?}"))
    };
    assert!(names(&at_rank(3)["only"][0]).contains(&"Summon Fallen".to_owned()));
    assert!(names(&at_rank(3)["only"][1]).contains(&"Summon Axiomite".to_owned()));
    assert!(names(&at_rank(9)["only"][0]).contains(&"Summon Ravener Dragon".to_owned()));

    // Both final codes open checked, and their links are the planner's.
    for code in [&lich, &aeon] {
        let opened = client.ok("new_build", json!({ "link": code })).await;
        assert_eq!(&opened["build"], code);
        assert_eq!(text(&opened["link"]), format!("{LINK_BASE}{}", text(code)));
    }

    // What the prompt implies and the data lacks. The spell catalogue (ticket 121) has Animal Growth, by name, with
    // its lists; no spell is "Mass Animal Growth", so that query's only hits are summaries holding its words (ticket
    // 123), which its note says. Durations ("summons lasting 24h") and what Extend Spell or Enduring Spells do to
    // them aren't modelled either; the agent labels that as its own game knowledge.
    let found = client
        .ok("search", json!({ "query": "mass animal growth" }))
        .await;
    assert!(
        found["matches"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| hit["matched"] == "summary"),
        "{found}"
    );
    assert!(
        text(&found["note"]).starts_with("No item's name"),
        "{found}"
    );
    let found = client
        .search(json!({ "query": "animal growth", "kinds": ["spell"] }))
        .await;
    assert_eq!(
        found.first(),
        Some(&("spell".to_owned(), "animal-growth".to_owned()))
    );
}
