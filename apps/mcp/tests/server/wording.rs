//! Three reasons the 308 queue's QA found reading wrongly (ticket 329), on the real game data, through `validate`,
//! `options`, `choose` and `describe`: selections named alike told apart by their owners, an archetype's level clause
//! read once, and Pentamic Faith's refusal of itself read as the game's rule. With the normal form (ticket 332), an
//! "or" inside an "or" reads as one, and "n of" with none held says no "more".

use rmcp::serde_json::{Map, Value, json};

use crate::support::*;

/// A human's Build start and `levels` levels of `class` with `archetype`, with `more` declared beside them.
fn declared(class: &str, archetype: &str, levels: u8, more: &[(&str, Value)]) -> Value {
    let mut choices = Map::new();
    for (address, value) in [
        ("start.race", json!("human")),
        ("start.faith", json!("atheism")),
        ("start.alignment", json!("true-neutral")),
    ] {
        choices.insert(address.to_owned(), value);
    }
    for level in 1..=levels {
        choices.insert(
            format!("level.{level}.class"),
            json!({ "class": class, "archetype": archetype }),
        );
    }
    for (address, value) in more {
        choices.insert((*address).to_owned(), value.clone());
    }
    json!({ "choices": choices })
}

/// The failure at `choice` of an invalid `validate` result.
fn failure<'a>(validated: &'a Value, choice: &str) -> &'a Value {
    assert_eq!(validated["status"], "invalid", "{validated}");
    at(&validated["failures"], choice)
}

/// The reason `options` gives for the option `value`, listed with every option's status.
fn reason(listed: &Value, value: &str) -> String {
    let row = rows(listed)
        .into_iter()
        .find(|row| row["value"] == value)
        .unwrap_or_else(|| panic!("no {value} in {listed}"));
    text(&row["status"]["reason"])
}

/// The game's Extra Rage Power (ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a) has three
/// `PrerequisiteFeature`s of group Any, each a selection the game names "Rage Power": RagePowerSelection
/// 28710502f46848d48b3f0d6132817c4e (BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12),
/// InstinctualWarriorRagePowerSelection 609f0e5336084442a0dafa3abd4d31c5 (InstinctualWarriorArchetype
/// adffdd8a99094a89823a79292a503ee9) and SkaldRagePowerSelection 2476514e31791394fa140f1a07941c96 (SkaldProgression
/// 26418fed2bc153245972a5b54204ed75). Each reads with what grants it, in every tool.
#[tokio::test]
async fn real_data_selections_named_alike_read_with_their_owners() {
    let client = Client::start().await;
    let rage_powers = "the Instinctual Warrior's Rage Power selection or the Barbarian's Rage Power selection \
                       or the Skald's Rage Power selection";
    let validated = client
        .ok(
            "validate",
            declared(
                "skald",
                "skald/battle-scion",
                5,
                &[("slot.feat.5.general.0", json!("extra-rage-power"))],
            ),
        )
        .await;
    let extra = failure(&validated, "slot.feat.5.general.0");
    assert_eq!(extra["needs"], json!([rage_powers]), "{validated}");

    let scion = client
        .ok("validate", declared("skald", "skald/battle-scion", 5, &[]))
        .await;
    assert_eq!(scion["status"], "valid", "{scion}");
    let listed = client
        .ok(
            "options",
            json!({ "build": scion["build"], "choice": "slot.feat.5.general.0", "query": "extra", "only_open": false }),
        )
        .await;
    assert_eq!(
        reason(&listed, "extra-rage-power"),
        format!("Needs {rage_powers}")
    );
    // ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a: ArcanistExploitSelection
    // b8bf3d5023f2d8c428fdf6438cecaea7 (ArcanistProgression 974911193b8a8b947a8275f96bb265d6) or ExploiterExploitSelection
    // 2ba8a0040e0149e9ae9bfcb01a8ff01d (ExploiterWizardArchetype 6d2027b4eec590c41adb5caeb7bb026d), both "Arcanist
    // Exploits".
    assert_eq!(
        reason(&listed, "extra-arcanist-exploit"),
        "Needs the Arcanist's Arcanist Exploits selection or the Exploiter Wizard's Arcanist Exploits selection"
    );
    // ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12: SylvanTricksterTalentSelection
    // 290bbcc3c3bb92144b853fd8fb8ff452, DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2
    // (DarkLurkerArchetype 33eb002078e5437b8c25db4b80b4a4fe) or RogueTalentSelection c074a5d615200494b8f2a9c845799d93
    // (RogueProgression b57b2a75a5abcaf47a01cf84672b50e9); the two "Rogue Talents" read with their owners. One "or" of
    // four, in the tree's sorted order (ticket 332).
    assert_eq!(
        reason(&listed, "extra-rogue-talent"),
        "Needs the Dark Lurker's Rogue Talents selection or the Rogue's Rogue Talents selection or the Fey Tricks \
         selection or the Rogue Talent selection"
    );

    let refused = client
        .ok(
            "choose",
            json!({ "build": scion["build"], "steps": [{ "choice": "slot.feat.5.general.0", "value": "extra-rage-power" }] }),
        )
        .await;
    assert_eq!(
        refused["rejected"]["reason"],
        format!("Needs {rage_powers}"),
        "{refused}"
    );
}

/// The game's Chain Arrows (ChainArrowsInfusion 0ff81c5691c64c4aa8b2ecc35c885901) has a `PrerequisiteClassLevel`
/// (KineticistClass 42a455d9ec1ad924d889272429eb8391, 16) and a `PrerequisiteArchetypeLevel` (KineticSharpshooterArchetype
/// e0684d0a6ef44cae9d095f5318618a85 of the Kineticist, 16), both of group All: the data's two groups are the game's,
/// and the class level they both need reads once. Rending Arrows (8896ac251d394dceaf650793aa85e177, 11) and Honed
/// Infusion (cf531860c9fb44c09bda9d8904657314, 12) are written the same.
#[tokio::test]
async fn real_data_an_archetype_level_reads_its_class_level_once() {
    let client = Client::start().await;
    let knight = client
        .ok(
            "validate",
            declared("kineticist", "kineticist/kinetic-knight", 5, &[]),
        )
        .await;
    assert_eq!(knight["status"], "valid", "{knight}");
    let listed = client
        .ok(
            "options",
            json!({ "build": knight["build"], "choice": "slot.selection.5.0", "only_open": false }),
        )
        .await;
    for (value, level) in [
        ("kineticist/infusions/chain-arrows", 16),
        ("kineticist/infusions/honed-infusion", 12),
        ("kineticist/infusions/rending-arrows", 11),
    ] {
        assert_eq!(
            reason(&listed, value),
            format!("Needs Kineticist level {level} (have 5), the Kinetic Sharpshooter archetype")
        );
    }

    let refused = client
        .ok(
            "choose",
            json!({ "build": knight["build"], "steps": [{ "choice": "slot.selection.5.0", "value": "kineticist/infusions/chain-arrows" }] }),
        )
        .await;
    assert_eq!(
        refused["rejected"]["reason"],
        "Needs Kineticist level 16 (have 5), the Kinetic Sharpshooter archetype",
        "{refused}"
    );

    let validated = client
        .ok(
            "validate",
            declared(
                "kineticist",
                "kineticist/kinetic-knight",
                5,
                &[(
                    "slot.selection.5.0",
                    json!("kineticist/infusions/chain-arrows"),
                )],
            ),
        )
        .await;
    assert_eq!(
        failure(&validated, "slot.selection.5.0")["needs"],
        json!([
            "Kineticist level 16 (have 5)",
            "the Kinetic Sharpshooter archetype"
        ]),
        "{validated}"
    );
}

/// The game's Pentamic Faith (HellknightDisciplinePentamicFaith b9750875e9d7454e85347d739a1bc894) needs the Order of
/// the Godclaw (`PrerequisiteFeature` HellKnightOrderOfTheGodclaw 5636564c278583342aec54eb2b409029) and refuses itself
/// (`PrerequisiteNoFeature` of its own blueprint). The planner keeps that blueprint as an option and a selection; the
/// refusal names it once, after what it needs.
#[tokio::test]
async fn real_data_pentamic_faith_refuses_itself_once() {
    let client = Client::start().await;
    let described = client
        .ok(
            "describe",
            json!({ "kind": "option", "key": "hellknight/disciplines/pentamic-faith" }),
        )
        .await;
    assert_eq!(
        described["prerequisite"], "Order of the Godclaw, not with Pentamic Faith",
        "{described}"
    );
}

/// Impossible Domain (ExtraDomain 213a8480d22206b45acbfa0619ca5aaf) needs DomainsSelection
/// 48525e5da45c9c243a343fc6545dbdb9, which the planner keeps as the Cleric's and the Inquisitor's "Domain" (with the
/// Sacred Huntsmaster's), DivineHunterDomainsSelection 72909a37a1ed5344f88ec9d1d31f5c5b or FakeDivineSparkSelection
/// e6d337e709b4478cb6e5389d3f85b601: the two "Domain"s read with their owners.
#[tokio::test]
async fn real_data_impossible_domain_tells_its_domain_selections_apart() {
    let client = Client::start().await;
    let described = client
        .ok(
            "describe",
            json!({ "kind": "mythic_ability", "key": "impossible-domain" }),
        )
        .await;
    // One "or" of five, in the tree's sorted order (ticket 332).
    assert_eq!(
        described["prerequisite"],
        "the Domain (Living Deity) selection or the Cleric's Domain selection or the Domain (Divine Hunter) selection \
         or the Inquisitor's Domain selection or the Domain (Sacred Huntsmaster) selection",
        "{described}"
    );
}

/// The game's Diva Strike (DivaStrike 3306f4c6913d4d429358b907cdb4ffd1) needs 3 of Diva Style
/// (b46bf228d6b2427abd028e2fe7b84600), Feint (c610310d31414edabcedf0c8a6fe32c4) and Combat Expertise
/// (CombatExpertiseFeature 4c44724ffa8844f4d9bedb5bb27d144a): a `PrerequisiteFeaturesFromList` with `Amount` 3. With
/// none of them held, the reason says how many it needs, not how many more (ticket 330 item 3). Its other two parts
/// are the same blueprint's: a `PrerequisiteStatValue` of Charisma 15 (Group All), and BAB 7 (`PrerequisiteStatValue`
/// BaseAttackBonus 7) or BardClass 772c83a25e2268e448e841dcd548235f level 7 (`PrerequisiteClassLevel`), both Group Any:
/// a skald's reason keeps the BAB alone, the bard being out of its classes' reach.
#[tokio::test]
async fn real_data_n_of_with_none_held_reads_n_of() {
    let client = Client::start().await;
    let scion = client
        .ok("validate", declared("skald", "skald/battle-scion", 5, &[]))
        .await;
    assert_eq!(scion["status"], "valid", "{scion}");
    let listed = client
        .ok(
            "options",
            json!({ "build": scion["build"], "choice": "slot.feat.5.general.0", "query": "diva", "only_open": false }),
        )
        .await;
    assert_eq!(
        reason(&listed, "diva-strike"),
        "Needs BAB +7 (have +3), (3 of: Combat Expertise, Diva Style, Feint), Cha 15 (have 10)"
    );
}
