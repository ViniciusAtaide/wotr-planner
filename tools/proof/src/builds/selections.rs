//! Selections: bloodlines, mysteries, elements, orders, domains, the curses, and the prerequisites they negate.

use rules::Alignment::{self, *};
use rules::{
    Build, BuildStart, ClassChoice, ClassKey, Engine, GameData, Pick, RaceKey, SlotId, SubraceKey,
};

use super::{atheist, character, human, levels, option, selection, with};
use crate::Named;
use crate::generate::with_selections_filled;

/// A human (or another race) of an alignment with nothing else chosen.
fn start(race: &str, subrace: Option<&str>, alignment: Alignment) -> Build {
    atheist(Build {
        start: BuildStart {
            race: Some(RaceKey::new(race)),
            subrace: subrace.map(SubraceKey::new),
            alignment: Some(alignment),
            ..BuildStart::default()
        },
        ..Build::default()
    })
}

/// A true neutral human taking one class for its levels, with its archetype and these options picked.
fn one_class(
    class: &str,
    archetype: Option<&str>,
    alignment: Alignment,
    count: usize,
    picks: &[(SlotId, &str)],
) -> Build {
    let mut build = start("human", None, alignment);
    build.levels = levels(&[(class, count)]);
    build.class_choices.insert(
        ClassKey::new(class),
        ClassChoice {
            archetype: archetype.map(rules::ArchetypeKey::new),
            ..ClassChoice::default()
        },
    );
    for (slot, key) in picks {
        build.picks.insert(slot.clone(), option(key));
    }
    build
}

/// The options of a class's selection, by key, from the game data.
fn options_of(data: &GameData, class: &str, selection: &str) -> Vec<String> {
    data.classes
        .iter()
        .find(|c| c.key.as_str() == class)
        .and_then(|c| c.selections.iter().find(|s| s.key.as_str() == selection))
        .map(|s| s.options.iter().map(|o| o.key.to_string()).collect())
        .unwrap_or_default()
}

fn mythic_ability(key: &str) -> Pick {
    crate::generate::mythic_ability_pick(key)
}

/// Half-elf subraces, a ranger/slayer's combat styles, a Living Deity and a Hellknight with Impossible Domain, a
/// lawful good Separatist, a bloodrager's Second Bloodline, a Hellknight who becomes a Signifer (`targeted …`).
/// Quirk kept: most name no class choice for their classes.
pub(super) fn targeted() -> Vec<Named> {
    let mut out = Vec::new();
    for subrace in [
        None,
        Some("half-elf/dual-heritage"),
        Some("half-elf/kindred-raised"),
    ] {
        let mut build = start("half-elf", subrace, TrueNeutral);
        build.levels = levels(&[("fighter", 1)]);
        out.push((format!("targeted half-elf {subrace:?}"), build));
    }
    let mut build = start("human", None, TrueNeutral);
    build.levels = levels(&[("ranger", 10), ("slayer", 2)]);
    build.picks.insert(
        selection(2, 0),
        option("ranger/combat-style-feat/archery-feats"),
    );
    build.picks.insert(
        selection(12, 0),
        option("slayer/slayer-talents/combat-style-feat-2"),
    );
    out.push(("targeted ranger10-slayer2".into(), build));
    let mut build = start("human", None, TrueNeutral);
    build.levels = levels(&[("arcanist", 8)]);
    build.class_choices.insert(
        ClassKey::new("arcanist"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("arcanist/magic-deceiver")),
            ..ClassChoice::default()
        },
    );
    build.picks.insert(
        selection(1, 0),
        option("arcanist/magic-deceiver/ways-of-the-deceiver/living-deity"),
    );
    build.picks.insert(
        SlotId::OptionSelection { level: 1, index: 0 },
        option("oracle/mystery/battle"),
    );
    build.picks.insert(
        SlotId::OptionSelection { level: 1, index: 2 },
        option("oracle/oracles-curse/plagued"),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        mythic_ability("impossible-domain"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        option("cleric/domain/nobility"),
    );
    out.push(("targeted living-deity-8".into(), build));
    let mut build = start("human", None, LawfulNeutral);
    build.levels = levels(&[("fighter", 6), ("hellknight", 3)]);
    for (level, ranks) in build.levels.iter_mut().zip([
        [("knowledge-arcana", 1), ("persuasion", 1)],
        [("knowledge-arcana", 1), ("persuasion", 1)],
        [("persuasion", 1), ("perception", 1)],
        [("persuasion", 1), ("perception", 1)],
        [("persuasion", 1), ("perception", 1)],
    ]) {
        level.skill_ranks = ranks
            .into_iter()
            .map(|(skill, n)| (rules::SkillKey::new(skill), n))
            .collect();
    }
    build.picks.insert(
        selection(7, 0),
        option("fighter/armiger/order/order-of-the-godclaw"),
    );
    build.picks.insert(
        selection(9, 0),
        option("hellknight/disciplines/pentamic-faith"),
    );
    build.picks.insert(
        SlotId::OptionSelection { level: 9, index: 0 },
        option("cleric/domain/law"),
    );
    out.push(("targeted hellknight-pentamic".into(), build.clone()));
    build.picks.insert(
        SlotId::Mythic { rank: 3 },
        mythic_ability("impossible-domain"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 3, index: 0 },
        option("cleric/domain/war"),
    );
    out.push(("targeted hellknight-pentamic-impossible".into(), build));
    let mut build = start("human", None, LawfulGood);
    build.levels = levels(&[("cleric", 3)]);
    build.class_choices.insert(
        ClassKey::new("cleric"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("cleric/separatist")),
            ..ClassChoice::default()
        },
    );
    build
        .picks
        .insert(selection(1, 0), option("cleric/domain/fire"));
    build
        .picks
        .insert(selection(1, 1), option("cleric/domain/chaos"));
    out.push(("targeted separatist-lawful-good".into(), build));
    let mut build = start("human", None, TrueNeutral);
    build.levels = levels(&[("bloodrager", 6)]);
    build
        .picks
        .insert(selection(1, 0), option("bloodrager/bloodline/draconic-red"));
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        mythic_ability("second-bloodrager-bloodline"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        option("bloodrager/bloodline/draconic-blue"),
    );
    out.push(("targeted bloodrager-second-dragon".into(), build));
    let mut build = start("human", None, LawfulNeutral);
    build.levels = levels(&[
        ("fighter", 6),
        ("hellknight", 1),
        ("hellknight-signifer", 1),
    ]);
    build.picks.insert(
        selection(7, 0),
        option("fighter/armiger/order/order-of-the-chain"),
    );
    out.push(("targeted hellknight-signifer".into(), build));
    out
}

/// Every option whose rows had class-skill features, through its class at 3: each bloodline, mystery, element and
/// order; domain pairs; a Divine Herbalist; Crossblooded, Sage, Empyreal, Sylvan, Overwhelming and Seeker sorcerers;
/// a sorcerer's Second Bloodline at MR 1 (`option …`).
pub(super) fn options(data: &GameData) -> Vec<Named> {
    let with = |class: &str, archetype: Option<&str>, count: usize, picks: &[(SlotId, &str)]| {
        one_class(class, archetype, TrueNeutral, count, picks)
    };
    let mut out = Vec::new();
    for (class, sel) in [
        ("sorcerer", "sorcerer/bloodline"),
        ("oracle", "oracle/mystery"),
        ("kineticist", "kineticist/element"),
        ("cavalier", "cavalier/order"),
    ] {
        for option in options_of(data, class, sel) {
            out.push((
                format!("option {class} {option}"),
                with(class, None, 3, &[(selection(1, 0), option.as_str())]),
            ));
        }
    }
    for (first, second) in [
        ("animal", "knowledge"),
        ("trickery", "air"),
        ("scalykind", "law"),
    ] {
        out.push((
            format!("option cleric {first}+{second}"),
            with(
                "cleric",
                None,
                3,
                &[
                    (selection(1, 0), &format!("cleric/domain/{first}")),
                    (selection(1, 1), &format!("cleric/domain/{second}")),
                ],
            ),
        ));
    }
    out.push((
        "option inquisitor trickery".into(),
        with(
            "inquisitor",
            None,
            3,
            &[(selection(1, 0), "cleric/domain/trickery")],
        ),
    ));
    for mystery in ["flame", "battle"] {
        out.push((
            format!("option divine-herbalist {mystery}"),
            with(
                "oracle",
                Some("oracle/divine-herbalist"),
                3,
                &[(selection(1, 0), &format!("oracle/mystery/{mystery}"))],
            ),
        ));
    }
    out.push((
        "option crossblooded abyssal+celestial".into(),
        with(
            "sorcerer",
            Some("sorcerer/crossblooded"),
            3,
            &[
                (selection(1, 0), "sorcerer/bloodline/abyssal"),
                (selection(1, 1), "sorcerer/bloodline/celestial"),
            ],
        ),
    ));
    out.push((
        "option sage".into(),
        with("sorcerer", Some("sorcerer/sage-sorcerer"), 3, &[]),
    ));
    out.push((
        "option empyreal".into(),
        with("sorcerer", Some("sorcerer/empyreal-sorcerer"), 3, &[]),
    ));
    out.push((
        "option sylvan".into(),
        with("sorcerer", Some("sorcerer/sylvan-sorcerer"), 3, &[]),
    ));
    out.push((
        "option overwhelming fey".into(),
        with(
            "sorcerer",
            Some("sorcerer/overwhelming-mage"),
            3,
            &[(selection(1, 0), "sorcerer/bloodline/fey")],
        ),
    ));
    out.push((
        "option seeker serpentine".into(),
        with(
            "sorcerer",
            Some("sorcerer/seeker-sorcerer"),
            3,
            &[(selection(1, 0), "sorcerer/bloodline/serpentine")],
        ),
    ));
    let mut second = with(
        "sorcerer",
        None,
        7,
        &[(selection(1, 0), "sorcerer/bloodline/arcane")],
    );
    second.picks.insert(
        SlotId::Mythic { rank: 1 },
        mythic_ability("second-bloodline"),
    );
    second.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        option("sorcerer/bloodline/undead"),
    );
    out.push(("option second-bloodline undead".into(), second));
    out
}

/// An Overwhelming Mage and a Seeker with every bloodline they offer, a lawful neutral monk and Sohei, the Animal
/// domain through a cleric (both slots), a Separatist's second domain, a druid's Nature Bond, a Blight Druid and an
/// inquisitor (`cleanup …`).
pub(super) fn cleanup(data: &GameData) -> Vec<Named> {
    let bloodlines = options_of(data, "sorcerer", "sorcerer/bloodline");
    let mut out = Vec::new();
    for archetype in ["sorcerer/overwhelming-mage", "sorcerer/seeker-sorcerer"] {
        for bloodline in &bloodlines {
            out.push((
                format!("cleanup {archetype} {bloodline}"),
                one_class(
                    "sorcerer",
                    Some(archetype),
                    TrueNeutral,
                    3,
                    &[(selection(1, 0), bloodline.as_str())],
                ),
            ));
        }
    }
    out.push((
        "cleanup monk".into(),
        one_class("monk", None, LawfulNeutral, 20, &[]),
    ));
    out.push((
        "cleanup monk/sohei".into(),
        one_class("monk", Some("monk/sohei"), LawfulNeutral, 20, &[]),
    ));
    let animal = "cleric/domain/animal";
    let fire = "cleric/domain/fire";
    out.push((
        "cleanup cleric animal+fire".into(),
        one_class(
            "cleric",
            None,
            TrueNeutral,
            6,
            &[(selection(1, 0), animal), (selection(1, 1), fire)],
        ),
    ));
    out.push((
        "cleanup cleric fire+animal".into(),
        one_class(
            "cleric",
            None,
            TrueNeutral,
            6,
            &[(selection(1, 0), fire), (selection(1, 1), animal)],
        ),
    ));
    out.push((
        "cleanup separatist fire+animal".into(),
        one_class(
            "cleric",
            Some("cleric/separatist"),
            TrueNeutral,
            8,
            &[(selection(1, 0), fire), (selection(1, 1), animal)],
        ),
    ));
    out.push((
        "cleanup druid nature-bond animal".into(),
        one_class(
            "druid",
            None,
            TrueNeutral,
            6,
            &[
                (selection(1, 0), "druid/nature-bond/domain"),
                (SlotId::OptionSelection { level: 1, index: 0 }, animal),
            ],
        ),
    ));
    out.push((
        "cleanup blight-druid animal".into(),
        one_class(
            "druid",
            Some("druid/blight-druid"),
            TrueNeutral,
            6,
            &[(selection(1, 0), animal)],
        ),
    ));
    out.push((
        "cleanup inquisitor animal".into(),
        one_class(
            "inquisitor",
            None,
            TrueNeutral,
            6,
            &[(selection(1, 0), animal)],
        ),
    ));
    out
}

/// Builds with a negated prerequisite in a picker's reasons: a fire Kinetic Sharpshooter 5; a barbarian 6 of the
/// Celestial totem line; a slayer 6 and 12; a Lawful Good Champion of the Faith 1 with the evil blessing as his
/// chosen alignment's; a Lawful Good Separatist 3 with Chaos as his own domain; a flame oracle 1 with the Lame curse.
/// Each also with its selections filled.
pub(super) fn negated_prerequisites(engine: &Engine) -> Vec<Named> {
    let make = |class: &str, archetype: Option<&str>, alignment: Alignment, levels: usize| {
        character("human", alignment, &[(class, archetype, levels)])
    };
    let plain = vec![
        (
            "sharpshooter 5",
            with(
                make(
                    "kineticist",
                    Some("kineticist/kinetic-sharpshooter"),
                    TrueNeutral,
                    5,
                ),
                vec![(selection(1, 0), option("kineticist/element/fire"))],
            ),
        ),
        (
            "celestial barbarian 6",
            with(
                make("barbarian", None, ChaoticNeutral, 6),
                vec![
                    (
                        selection(2, 0),
                        option("barbarian/rage-power/celestial-totem-lesser"),
                    ),
                    (
                        selection(4, 0),
                        option("barbarian/rage-power/celestial-totem"),
                    ),
                ],
            ),
        ),
        ("slayer 6", make("slayer", None, TrueNeutral, 6)),
        ("slayer 12", make("slayer", None, TrueNeutral, 12)),
        (
            "champion 1",
            make(
                "warpriest",
                Some("warpriest/champion-of-the-faith"),
                LawfulGood,
                1,
            ),
        ),
        (
            "separatist 3",
            with(
                make("cleric", Some("cleric/separatist"), LawfulGood, 3),
                vec![(selection(1, 0), option("cleric/domain/law"))],
            ),
        ),
        (
            "lame oracle 1",
            with(
                make("oracle", None, TrueNeutral, 1),
                vec![
                    (selection(1, 0), option("oracle/mystery/flame")),
                    (selection(1, 1), option("oracle/oracles-curse/lame")),
                ],
            ),
        ),
    ];
    let mut out: Vec<Named> = plain
        .iter()
        .map(|(name, build)| (format!("t65 {name}"), build.clone()))
        .collect();
    for (name, build) in plain {
        out.push((
            format!("t65 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    out
}

/// A Dual-Cursed Oracle 1, 5, 6 and 20 and a fighter 1 / Dual-Cursed Oracle 5, empty and filled; fixed curse picks
/// at the Dual-Cursed Oracle's level 1 (two curses, the second only, one curse twice) and Beneficial Curse at MR 1 on
/// a Dual-Cursed Oracle 6 holding two curses (a third curse, each held one), each also filled; and a plain oracle 1
/// with a pick in a fourth level-1 slot, which it has none of. Chaotic neutral humans.
pub(super) fn dual_curses(engine: &Engine) -> Vec<Named> {
    let curse = |name: &str| option(&format!("oracle/oracles-curse/{name}"));
    let dual = Some("oracle/dual-cursed-oracle");
    let mut plain: Vec<(String, Build)> = [1, 5, 6, 20]
        .into_iter()
        .map(|levels| {
            (
                format!("dual-cursed oracle {levels}"),
                human(&[("oracle", dual, levels)]),
            )
        })
        .collect();
    plain.push((
        "fighter 1 dual-cursed oracle 5".to_owned(),
        human(&[("fighter", None, 1), ("oracle", dual, 5)]),
    ));
    let mut out = super::empty_and_filled(engine, "t67", plain);
    let two = vec![
        (selection(1, 2), curse("plagued")),
        (selection(1, 3), curse("lame")),
    ];
    let mythic = SlotId::Mythic { rank: 1 };
    let granted = SlotId::MythicSelection { rank: 1, index: 0 };
    let beneficial = mythic_ability("beneficial-curse");
    let dual1 = || human(&[("oracle", dual, 1)]);
    let dual6 = with(human(&[("oracle", dual, 6)]), two.clone());
    let fixed = [
        ("two curses", with(dual1(), two)),
        (
            "second curse only",
            with(dual1(), vec![(selection(1, 3), curse("lame"))]),
        ),
        (
            "one curse twice",
            with(
                dual1(),
                vec![
                    (selection(1, 2), curse("plagued")),
                    (selection(1, 3), curse("plagued")),
                ],
            ),
        ),
        (
            "beneficial curse third",
            with(
                dual6.clone(),
                vec![
                    (mythic.clone(), beneficial.clone()),
                    (granted.clone(), curse("wasting")),
                ],
            ),
        ),
        (
            "beneficial curse first",
            with(
                dual6.clone(),
                vec![
                    (mythic.clone(), beneficial.clone()),
                    (granted.clone(), curse("plagued")),
                ],
            ),
        ),
        (
            "beneficial curse second",
            with(
                dual6.clone(),
                vec![
                    (mythic.clone(), beneficial.clone()),
                    (granted, curse("lame")),
                ],
            ),
        ),
        (
            "beneficial curse empty",
            with(dual6, vec![(mythic, beneficial)]),
        ),
        (
            "plain oracle fourth slot",
            with(
                human(&[("oracle", None, 1)]),
                vec![(selection(1, 3), curse("lame"))],
            ),
        ),
    ];
    for (name, build) in fixed {
        out.push((format!("t67 {name}"), build.clone()));
        out.push((
            format!("t67 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    out
}
