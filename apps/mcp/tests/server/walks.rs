//! The guarantee as a property: random walks of `choose` and `withdraw` over the test world, and after every step
//! the returned code opens through the engine's checked open (`Engine::open_link`: decoded and possible).
//!
//! Each walk starts from an empty Build with a random run of 0 to 12 levels and, at each step, either adds a level,
//! takes an offered `ok` value at an open choice from `todo`, changes a choice the Build already holds (from
//! `sheet`'s picks, the point buy included), tries an option that isn't open (it must be refused, with the code
//! unchanged), moves a score to a random target (several single steps, which may be refused), or withdraws a choice
//! `sheet` marks removable. Every open option tried must be applied, and every removable choice must come off.

use rmcp::serde_json::{Value, json};
use rules::Engine;

use crate::support::*;

/// How many walks, how many steps each, and the generator's fixed seed.
const WALKS: usize = 500;
const STEPS: usize = 20;
const SEED: u64 = 0x5EED_0109;

/// SplitMix64: a small, fixed-seed generator, so a failing walk can be replayed.
struct Dice(u64);

impl Dice {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `n` (`n > 0`).
    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("below a usize")
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        (!items.is_empty()).then(|| &items[self.below(items.len())])
    }
}

struct Walker<'a> {
    client: &'a Client,
    engine: &'a Engine,
    dice: Dice,
    /// The walk so far, for the failure message.
    moves: Vec<String>,
    tally: Tally,
}

/// What the walks did, so the test shows they covered each kind of step.
#[derive(Debug, Default)]
struct Tally {
    /// Open options applied.
    applied: usize,
    /// Closed options refused.
    refused: usize,
    /// Score moves to a random target, applied and refused.
    moved: usize,
    refused_moves: usize,
    /// Choices withdrawn.
    withdrawn: usize,
    /// Codes after a step whose Build holds a Mythic path.
    with_path: usize,
}

impl Walker<'_> {
    /// The options at an address (`only_open` as given), or none when the Build has no such choice.
    async fn options(&self, code: &str, choice: &str, only_open: bool) -> Vec<Value> {
        let result = self
            .client
            .call(
                "options",
                json!({ "build": code, "choice": choice, "values": true, "only_open": only_open }),
            )
            .await;
        if result.is_error == Some(true) {
            assert_eq!(error_code(&result), "no_such_choice", "{choice}");
            return Vec::new();
        }
        let content = result.structured_content.expect("structured content");
        content["options"].as_array().cloned().unwrap_or_default()
    }

    /// `choose` one step; the new code.
    async fn choose(&mut self, code: &str, choice: &str, value: Value) -> Value {
        self.moves.push(format!("choose {choice} {value}"));
        self.client
            .ok(
                "choose",
                json!({ "build": code, "steps": [{ "choice": choice, "value": value }] }),
            )
            .await
    }

    /// Takes a random open option at an address, which must be applied; the new code, or `None` when nothing is
    /// open there.
    async fn take_open(&mut self, code: &str, choice: &str) -> Option<String> {
        let open = self.options(code, choice, true).await;
        let option = self.dice.pick(&open)?.clone();
        assert_eq!(option["status"], "ok");
        let chosen = self.choose(code, choice, option["value"].clone()).await;
        assert_eq!(
            (chosen["applied"].clone(), chosen.get("rejected")),
            (json!(1), None),
            "an open option was refused: {:?}",
            self.moves
        );
        self.tally.applied += 1;
        Some(text(&chosen["build"]))
    }

    /// Tries a random option that isn't open at an address, which must be refused with the code unchanged.
    async fn try_closed(&mut self, code: &str, choice: &str) {
        let all = self.options(code, choice, false).await;
        let closed: Vec<Value> = all
            .into_iter()
            .filter(|option| option["status"] != "ok")
            .collect();
        let Some(option) = self.dice.pick(&closed).cloned() else {
            return;
        };
        let chosen = self.choose(code, choice, option["value"].clone()).await;
        assert_eq!(
            chosen["applied"], 0,
            "a closed option was applied: {:?}",
            self.moves
        );
        assert_eq!(chosen["rejected"]["issue"], option["status"]["issue"]);
        assert_eq!(chosen["build"], code);
        self.tally.refused += 1;
    }

    /// One random step from a code; the code after it.
    async fn step(&mut self, code: String) -> String {
        let roll = self.dice.below(100);
        let build = self.engine.open_link(&code).expect("the walk's code opens");
        let levels = build.levels.len();
        if roll < 15 {
            let choice = format!("level.{}.class", levels + 1);
            return self.take_open(&code, &choice).await.unwrap_or(code);
        }
        if roll < 25 {
            // Withdraw a removable choice: it must come off.
            let sheet = self
                .client
                .ok("sheet", json!({ "build": code, "removable": true }))
                .await;
            let removable: Vec<String> = sheet["removable"]
                .as_array()
                .expect("the removable choices")
                .iter()
                .map(text)
                .collect();
            let Some(choice) = self.dice.pick(&removable).cloned() else {
                return code;
            };
            self.moves.push(format!("withdraw {choice}"));
            let result = self
                .client
                .call("withdraw", json!({ "build": code, "choice": choice }))
                .await;
            assert_ne!(
                result.is_error,
                Some(true),
                "a removable choice was refused: {result:?} after {:?}",
                self.moves
            );
            let content = result.structured_content.expect("structured content");
            self.tally.withdrawn += 1;
            return text(&content["build"]);
        }
        if roll < 30 {
            // A score to a random target: several single steps, refused at the first that can't be taken.
            let ability = ["str", "dex", "con", "int", "wis", "cha"][self.dice.below(6)];
            let target = 6 + self.dice.below(14);
            let choice = format!("start.score.{ability}");
            let chosen = self.choose(&code, &choice, json!(target)).await;
            if chosen.get("rejected").is_some() {
                assert_eq!(chosen["build"], code.as_str());
                self.tally.refused_moves += 1;
            } else {
                self.tally.moved += 1;
            }
            return text(&chosen["build"]);
        }
        let sheet = self.client.ok("sheet", json!({ "build": code })).await;
        let todo = choices(&sheet["todo"]);
        if roll < 40 {
            // Change a held choice: any of the sheet's pick lines, the point buy included.
            let mut held: Vec<String> = choices(&sheet["start"]);
            for level in sheet["levels"].as_array().expect("levels") {
                held.extend(choices(&level["picks"]));
            }
            // A rank with no slots (not reached yet) leaves `picks` out.
            for rank in sheet["ranks"].as_array().expect("ranks") {
                if !rank["picks"].is_null() {
                    held.extend(choices(&rank["picks"]));
                }
            }
            held.extend(
                ["str", "dex", "con", "int", "wis", "cha"].map(|a| format!("start.score.{a}")),
            );
            let Some(choice) = self.dice.pick(&held).cloned() else {
                return code;
            };
            return self.take_open(&code, &choice).await.unwrap_or(code);
        }
        let Some(choice) = self.dice.pick(&todo).cloned() else {
            let choice = format!("level.{}.class", levels + 1);
            return self.take_open(&code, &choice).await.unwrap_or(code);
        };
        if roll < 48 {
            self.try_closed(&code, &choice).await;
            return code;
        }
        self.take_open(&code, &choice).await.unwrap_or(code)
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn random_walks_never_return_an_impossible_build() {
    let client = Client::on_the_test_world().await;
    let engine = rules::test_world::engine();
    let mut walker = Walker {
        client: &client,
        engine: &engine,
        dice: Dice(SEED),
        moves: Vec::new(),
        tally: Tally::default(),
    };
    let mut deepest = 0;
    for walk in 0..WALKS {
        walker.moves.clear();
        let mut code = text(&client.ok("new_build", json!({})).await["build"]);
        // A random head start of 0 to 12 levels, each a random open class, so many walks reach the Mythic track.
        for level in 1..=walker.dice.below(13) {
            let choice = format!("level.{level}.class");
            code = walker.take_open(&code, &choice).await.unwrap_or(code);
        }
        for _ in 0..STEPS {
            code = walker.step(code).await;
            let opened = engine.open_link(&code);
            assert!(
                opened.is_ok(),
                "walk {walk} returned a code that doesn't open: {code} after {:?}",
                walker.moves
            );
            let build = opened.expect("checked above");
            deepest = deepest.max(build.levels.len());
            walker.tally.with_path += usize::from(build.mythic_path.is_some());
        }
    }
    let tally = &walker.tally;
    eprintln!("{WALKS} walks of {STEPS} steps, seed {SEED:#x}: deepest level {deepest}, {tally:?}");
    // Each kind of step happened, and the walks reached the Mythic track (MR 3, the path, at level 9).
    assert!(deepest >= 9, "the deepest walk reached level {deepest}");
    assert!(tally.with_path > 0, "{tally:?}");
    for count in [
        tally.applied,
        tally.refused,
        tally.moved,
        tally.refused_moves,
        tally.withdrawn,
    ] {
        assert!(count > 0, "{tally:?}");
    }
}
