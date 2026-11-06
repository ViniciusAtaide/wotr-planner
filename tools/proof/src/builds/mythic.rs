//! Mythic picks: the slots Extra Feat and Extra Mythic Ability give, and the selections Witch Wandering Hex and
//! Beneficial Curse grant.

use rules::{Build, Engine, MythicAbilityKey, MythicFeatKey, MythicPathKey, Pick, SlotId};

use super::{feat, human, option, selection, with};
use crate::Named;
use crate::generate::with_selections_filled;

fn mythic(rank: u8) -> SlotId {
    SlotId::Mythic { rank }
}

fn ability(key: &str) -> Pick {
    Pick::MythicAbility(MythicAbilityKey::new(key))
}

fn mythic_feat(key: &str) -> Pick {
    Pick::mythic_feat(MythicFeatKey::new(key))
}

/// The slots Extra Feat and Extra Mythic Ability give, with fixed picks: each at MR 2 of a fighter 7, empty and
/// filled; both (MR 2 and 4) on a Trickster fighter 20; a sorcerer's Second Bloodline through Extra Mythic Ability; a
/// barbarian's Extra Rage Power through Extra Feat; and Builds no sheet holds (a pick in the slot with another mythic
/// feat, a feat whose prerequisites aren't met there). Chaotic neutral humans.
pub(super) fn extra_slots(engine: &Engine) -> Vec<Named> {
    let extra = |rank| SlotId::MythicExtra { rank };
    let fighter = |levels| human(&[("fighter", None, levels)]);
    let toughness = feat("toughness");
    let unstoppable = ability("unstoppable");
    let mut out = Vec::new();
    for (key, pick) in [
        ("extra-feat", &toughness),
        ("extra-mythic-ability", &unstoppable),
    ] {
        let build = with(fighter(7), vec![(mythic(2), mythic_feat(key))]);
        out.push((format!("t55 fighter {key} / empty"), build.clone()));
        out.push((
            format!("t55 fighter {key} / filled"),
            with(build, vec![(extra(2), pick.clone())]),
        ));
    }
    let mut both = with(
        fighter(20),
        vec![
            (mythic(2), mythic_feat("extra-feat")),
            (mythic(4), mythic_feat("extra-mythic-ability")),
        ],
    );
    both.mythic_path = Some(MythicPathKey::new("trickster"));
    out.push(("t55 trickster both / empty".into(), both.clone()));
    out.push((
        "t55 trickster both / filled".into(),
        with(
            both,
            vec![(extra(2), toughness.clone()), (extra(4), unstoppable)],
        ),
    ));

    let sorcerer = with(
        with_selections_filled(engine, human(&[("sorcerer", None, 7)])),
        vec![
            (mythic(2), mythic_feat("extra-mythic-ability")),
            (extra(2), ability("second-bloodline")),
        ],
    );
    out.push((
        "t55 sorcerer second bloodline / ability".into(),
        sorcerer.clone(),
    ));
    // Another bloodline than the one at 1.
    let first = sorcerer.picks[&selection(1, 0)].clone();
    let second = ["sorcerer/bloodline/abyssal", "sorcerer/bloodline/arcane"]
        .into_iter()
        .map(option)
        .find(|pick| *pick != first)
        .expect("one of two bloodlines differs");
    let sorcerer = with(
        sorcerer,
        vec![(SlotId::MythicSelection { rank: 2, index: 0 }, second)],
    );
    out.push((
        "t55 sorcerer second bloodline / filled".into(),
        with_selections_filled(engine, sorcerer),
    ));

    let barbarian = with(
        with_selections_filled(engine, human(&[("barbarian", None, 7)])),
        vec![
            (mythic(2), mythic_feat("extra-feat")),
            (extra(2), feat("extra-rage-power")),
        ],
    );
    out.push((
        "t55 barbarian extra rage power / feat".into(),
        barbarian.clone(),
    ));
    out.push((
        "t55 barbarian extra rage power / filled".into(),
        with_selections_filled(engine, barbarian),
    ));

    // Builds no sheet holds.
    out.push((
        "t55 extra slot pick with another mythic feat".into(),
        with(
            fighter(7),
            vec![
                (mythic(2), mythic_feat("emergency-potion")),
                (extra(2), toughness),
            ],
        ),
    ));
    out.push((
        "t55 extra feat unmet prerequisite".into(),
        with(
            fighter(7),
            vec![
                (mythic(2), mythic_feat("extra-feat")),
                (extra(2), feat("great-cleave")),
            ],
        ),
    ));
    out
}

/// The selections Witch Wandering Hex and Beneficial Curse grant: a witch, a shaman, an oracle and a Hexcrafter magus,
/// each 20, empty and filled; Witch Wandering Hex and Beneficial Curse at MR 1 (level 6) with their slots empty and
/// filled, Witch Wandering Hex through Extra Mythic Ability at MR 2, both on a Trickster at 20; and Builds no sheet
/// holds (Witch Wandering Hex without a hex, a hex or a curse held twice, a major hex at 6). The fixed picks go on the
/// filled Builds, choosing a hex or curse the Build doesn't hold (or does). Chaotic neutral humans.
pub(super) fn granted_selections(engine: &Engine) -> Vec<Named> {
    let granted = |rank| SlotId::MythicSelection { rank, index: 0 };
    let make =
        |class: &str, archetype: Option<&str>, levels: usize| human(&[(class, archetype, levels)]);
    let mut out = Vec::new();
    for (class, archetype) in [
        ("witch", None),
        ("shaman", None),
        ("oracle", None),
        ("magus", Some("magus/hexcrafter")),
    ] {
        let name = archetype.unwrap_or(class);
        out.extend(super::empty_and_filled(
            engine,
            "t62",
            vec![(format!("{name} 20"), make(class, archetype, 20))],
        ));
    }
    let filled = |class: &str, archetype: Option<&str>, levels: usize| {
        with_selections_filled(engine, make(class, archetype, levels))
    };
    // The options a filled Build's own slots hold, so a fixed pick names one it doesn't (or does).
    let held = |build: &Build, prefix: &str| -> Vec<String> {
        let mut keys: Vec<String> = build
            .picks
            .values()
            .map(|pick| pick.key().to_owned())
            .filter(|key| key.starts_with(prefix))
            .collect();
        keys.sort();
        keys
    };
    let free = |build: &Build, candidates: &[&str]| -> Pick {
        let held = held(build, "");
        let key = candidates
            .iter()
            .find(|key| !held.contains(&key.to_string()))
            .expect("a candidate the Build doesn't hold");
        option(key)
    };
    let hexes = [
        "witch/hex/ward",
        "witch/hex/slumber",
        "witch/hex/misfortune",
        "witch/hex/fortune",
    ];
    let curses = [
        "oracle/oracles-curse/wolf-scarred-face",
        "oracle/oracles-curse/wasting",
        "oracle/oracles-curse/lame",
    ];
    let witch6 = filled("witch", None, 6);
    let shaman6 = filled("shaman", None, 6);
    let oracle6 = filled("oracle", None, 6);
    let hexcrafter6 = filled("magus", Some("magus/hexcrafter"), 6);
    let witch7 = filled("witch", None, 7);
    let mut trickster_witch = filled("witch", None, 20);
    trickster_witch.mythic_path = Some(MythicPathKey::new("trickster"));
    let mut trickster_oracle = filled("oracle", None, 20);
    trickster_oracle.mythic_path = Some(MythicPathKey::new("trickster"));
    let wandering = || (mythic(1), ability("witch-wandering-hex"));
    let beneficial = || (mythic(1), ability("beneficial-curse"));
    let fixed = vec![
        (
            "witch 6 wandering hex empty",
            with(witch6.clone(), vec![wandering()]),
        ),
        (
            "witch 6 wandering hex filled",
            with(
                witch6.clone(),
                vec![wandering(), (granted(1), free(&witch6, &hexes))],
            ),
        ),
        (
            "shaman 6 wandering hex filled",
            with(
                shaman6.clone(),
                vec![wandering(), (granted(1), free(&shaman6, &hexes))],
            ),
        ),
        (
            "hexcrafter 6 wandering hex filled",
            with(
                hexcrafter6.clone(),
                vec![wandering(), (granted(1), free(&hexcrafter6, &hexes))],
            ),
        ),
        (
            "oracle 6 beneficial curse empty",
            with(oracle6.clone(), vec![beneficial()]),
        ),
        (
            "oracle 6 beneficial curse filled",
            with(
                oracle6.clone(),
                vec![beneficial(), (granted(1), free(&oracle6, &curses))],
            ),
        ),
        (
            "witch 7 wandering hex through extra mythic ability",
            with(
                witch7.clone(),
                vec![
                    (mythic(2), mythic_feat("extra-mythic-ability")),
                    (
                        SlotId::MythicExtra { rank: 2 },
                        ability("witch-wandering-hex"),
                    ),
                    (granted(2), free(&witch7, &hexes)),
                ],
            ),
        ),
        (
            "trickster witch 20 wandering hex",
            with(
                trickster_witch.clone(),
                vec![wandering(), (granted(1), free(&trickster_witch, &hexes))],
            ),
        ),
        (
            "trickster oracle 20 beneficial curse",
            with(
                trickster_oracle.clone(),
                vec![beneficial(), (granted(1), free(&trickster_oracle, &curses))],
            ),
        ),
        // Builds no sheet holds.
        (
            "fighter 6 wandering hex",
            with(make("fighter", None, 6), vec![wandering()]),
        ),
        (
            "witch 6 wandering hex held hex",
            with(
                witch6.clone(),
                vec![
                    wandering(),
                    (granted(1), option(&held(&witch6, "witch/hex/")[0])),
                ],
            ),
        ),
        (
            "witch 6 wandering hex agony",
            with(
                witch6,
                vec![wandering(), (granted(1), option("witch/hex/agony"))],
            ),
        ),
        (
            "oracle 6 beneficial curse held curse",
            with(
                oracle6.clone(),
                vec![
                    beneficial(),
                    (
                        granted(1),
                        option(&held(&oracle6, "oracle/oracles-curse/")[0]),
                    ),
                ],
            ),
        ),
    ];
    for (name, build) in fixed {
        out.push((format!("t62 {name}"), build));
    }
    out
}
