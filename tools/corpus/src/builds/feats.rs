//! Feats: the Extra feats that give a pick, class bonus feat lists, the sorcerer's and slayer's feat slots, and the
//! talents that repeat a feat.

use rules::Alignment::*;
use rules::{
    Build, Engine, FeatKey, FeatSource, Parameter, Pick, SelectionOptionKey, SkillKey, SlotId,
    WeaponKey,
};

use super::{character, empty_and_filled, feat, general, human, option, selection, with};
use crate::Named;
use crate::generate::with_selections_filled;

/// `count` levels of one class, as a part of [`character`].
fn only<'a>(
    class: &'a str,
    archetype: Option<&'a str>,
    count: usize,
) -> [(&'a str, Option<&'a str>, usize); 1] {
    [(class, archetype, count)]
}

/// Each Extra feat in the general slot of a character of its class, taken twice, by a witch who becomes a shaman, and
/// Extra Mercy; each class's selections filled first (`/ before`), then the feat (`/ feat`), then the selections
/// again (`/ filled`, the extra slot's where the rules give one). Chaotic neutral humans, the paladin lawful good.
pub(super) fn extra_feats(engine: &Engine) -> Vec<Named> {
    let cases: Vec<(&str, Build, &str, &[u8])> = vec![
        (
            "barbarian rage power",
            human(&only("barbarian", None, 5)),
            "extra-rage-power",
            &[3],
        ),
        (
            "barbarian rage power twice",
            human(&only("barbarian", None, 5)),
            "extra-rage-power",
            &[3, 5],
        ),
        (
            "witch hex",
            human(&only("witch", None, 3)),
            "extra-hex",
            &[1],
        ),
        (
            "shaman hex",
            human(&only("shaman", None, 3)),
            "extra-hex",
            &[3],
        ),
        (
            "magus arcana",
            human(&only("magus", None, 3)),
            "extra-arcana",
            &[3],
        ),
        (
            "arcanist exploit",
            human(&only("arcanist", None, 3)),
            "extra-arcanist-exploit",
            &[1, 3],
        ),
        (
            "alchemist discovery",
            human(&only("alchemist", None, 3)),
            "extra-discovery",
            &[3],
        ),
        (
            "underground chemist discovery",
            human(&only("rogue", Some("rogue/underground-chemist"), 11)),
            "extra-discovery",
            &[11],
        ),
        (
            "rogue talent",
            human(&only("rogue", None, 5)),
            "extra-rogue-talent",
            &[3, 5],
        ),
        (
            "sylvan trickster fey trick",
            human(&only("rogue", Some("rogue/sylvan-trickster"), 3)),
            "extra-rogue-talent",
            &[3],
        ),
        (
            "slayer talent",
            human(&only("slayer", None, 3)),
            "extra-slayer-talent",
            &[3],
        ),
        (
            "kineticist wild talent",
            human(&only("kineticist", None, 3)),
            "extra-wild-talent",
            &[3],
        ),
        (
            "witch then shaman hex",
            human(&[("witch", None, 1), ("shaman", None, 2)]),
            "extra-hex",
            &[3],
        ),
        (
            "paladin mercy",
            character("human", LawfulGood, &only("paladin", None, 3)),
            "extra-mercy",
            &[3],
        ),
        // A barbarian 3 taking Extra Rage Power at 1, before any rage power slot: no slot, prerequisite unmet.
        (
            "barbarian rage power at 1",
            human(&only("barbarian", None, 3)),
            "extra-rage-power",
            &[1],
        ),
    ];
    let mut out = Vec::new();
    for (name, build, key, levels) in cases {
        let mut build = with_selections_filled(engine, build);
        out.push((format!("t49 {name} / before"), build.clone()));
        for level in levels {
            build.picks.insert(general(*level), feat(key));
        }
        out.push((format!("t49 {name} / feat"), build.clone()));
        out.push((
            format!("t49 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    out
}

/// Extra Discovery and Extra Arcana on the archetypes that borrow discoveries and arcana: a Vivisectionist 5 with
/// Extra Discovery at 3 and 5, a Hexcrafter 7 with Extra Arcana at 7, a Hexcrafter 19 with it at 7, 13 and 19, and a
/// plain alchemist 3 and magus 3 with theirs, each before, with the feat and filled; a Vivisectionist and a Hexcrafter
/// 20, empty and filled; a Vivisectionist 3 holding only a rogue talent and a Hexcrafter 7 holding only hexes, each
/// with the feat; and Builds no sheet holds: Extra Discovery on a Vivisectionist 1, Extra Arcana on a Hexcrafter 5,
/// Extra Discovery on a rogue 2 / Vivisectionist 1. Each fixed one also filled. Chaotic neutral humans.
pub(super) fn borrowed_extra_feats(engine: &Engine) -> Vec<Named> {
    let viv = Some("alchemist/vivisectionist");
    let hex = Some("magus/hexcrafter");
    let mut out = Vec::new();
    let cases: Vec<(&str, Build, &str, &[u8])> = vec![
        (
            "vivisectionist discovery",
            human(&only("alchemist", viv, 5)),
            "extra-discovery",
            &[3, 5],
        ),
        (
            "hexcrafter arcana",
            human(&only("magus", hex, 7)),
            "extra-arcana",
            &[7],
        ),
        (
            "hexcrafter arcana thrice",
            human(&only("magus", hex, 19)),
            "extra-arcana",
            &[7, 13, 19],
        ),
        (
            "alchemist discovery",
            human(&only("alchemist", None, 3)),
            "extra-discovery",
            &[3],
        ),
        (
            "magus arcana",
            human(&only("magus", None, 3)),
            "extra-arcana",
            &[3],
        ),
    ];
    for (name, build, key, levels) in cases {
        let mut build = with_selections_filled(engine, build);
        out.push((format!("t64 {name} / before"), build.clone()));
        for level in levels {
            build.picks.insert(general(*level), feat(key));
        }
        out.push((format!("t64 {name} / feat"), build.clone()));
        out.push((
            format!("t64 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    out.extend(empty_and_filled(
        engine,
        "t64",
        vec![
            (
                "vivisectionist 20".to_owned(),
                human(&only("alchemist", viv, 20)),
            ),
            ("hexcrafter 20".to_owned(), human(&only("magus", hex, 20))),
        ],
    ));
    let fixed = [
        (
            "vivisectionist talent only",
            with(
                human(&only("alchemist", viv, 3)),
                vec![
                    (selection(2, 0), option("rogue/rogue-talents/fast-stealth")),
                    (general(3), feat("extra-discovery")),
                ],
            ),
        ),
        (
            "hexcrafter hexes only",
            with(
                human(&only("magus", hex, 7)),
                vec![
                    (selection(4, 0), option("witch/hex/slumber")),
                    (selection(6, 0), option("witch/hex/cackle")),
                    (general(7), feat("extra-arcana")),
                ],
            ),
        ),
        (
            "vivisectionist 1 discovery",
            with(
                human(&only("alchemist", viv, 1)),
                vec![(general(1), feat("extra-discovery"))],
            ),
        ),
        (
            "hexcrafter 5 arcana",
            with(
                human(&only("magus", hex, 5)),
                vec![
                    (selection(4, 0), option("witch/hex/slumber")),
                    (general(5), feat("extra-arcana")),
                ],
            ),
        ),
        (
            "rogue 2 vivisectionist 1 discovery",
            with(
                human(&[("rogue", None, 2), ("alchemist", viv, 1)]),
                vec![
                    (selection(2, 0), option("rogue/rogue-talents/fast-stealth")),
                    (general(3), feat("extra-discovery")),
                ],
            ),
        ),
    ];
    for (name, build) in fixed {
        out.push((format!("t64 {name}"), build.clone()));
        out.push((
            format!("t64 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    out
}

/// The class bonus feat lists: a Titan Fighter, a hunter and each of its archetypes, a Stormwalker, and a fighter, a
/// ranger and a Nomad, each 20, empty and filled; a ranger 2 / Titan Fighter 6, a hunter 3 / Titan Fighter 2 and a
/// Stormwalker 6 / Urban Hunter 6, empty and filled; fixed picks in the lists' slots; and Builds no sheet holds.
/// Chaotic neutral humans.
pub(super) fn class_feat_lists(engine: &Engine) -> Vec<Named> {
    let class_slot = |level| SlotId::Feat {
        level,
        source: FeatSource::Class,
        index: 0,
    };
    let feat_for = |key: &str, weapon: &str| {
        Pick::feat_for(FeatKey::new(key), Parameter::Weapon(WeaponKey::new(weapon)))
    };
    let mut plain: Vec<(String, Build)> = Vec::new();
    for (class, archetype) in [
        ("fighter", Some("fighter/titan-fighter")),
        ("fighter", None),
        ("hunter", None),
        ("hunter", Some("hunter/colluding-scoundrel")),
        ("hunter", Some("hunter/divine-hound")),
        ("hunter", Some("hunter/divine-hunter")),
        ("hunter", Some("hunter/forester")),
        ("hunter", Some("hunter/tandem-executioner")),
        ("hunter", Some("hunter/urban-hunter")),
        ("hunter", Some("hunter/wandering-marksman")),
        ("ranger", Some("ranger/stormwalker")),
        ("ranger", Some("ranger/nomad")),
        ("ranger", None),
    ] {
        plain.push((
            format!("{} 20", archetype.unwrap_or(class)),
            human(&only(class, archetype, 20)),
        ));
    }
    let titan_fighter = Some("fighter/titan-fighter");
    for (name, parts) in [
        (
            "ranger 2 titan 6",
            vec![("ranger", None, 2), ("fighter", titan_fighter, 6)],
        ),
        (
            "hunter 3 titan 2",
            vec![("hunter", None, 3), ("fighter", titan_fighter, 2)],
        ),
        (
            "stormwalker 6 urban hunter 6",
            vec![
                ("ranger", Some("ranger/stormwalker"), 6),
                ("hunter", Some("hunter/urban-hunter"), 6),
            ],
        ),
    ] {
        plain.push((name.to_owned(), human(&parts)));
    }
    let mut out = empty_and_filled(engine, "t61", plain);
    let titan = |levels| human(&only("fighter", titan_fighter, levels));
    let fixed = vec![
        (
            "titan 18 massive attacks",
            with(
                titan(18),
                vec![
                    (class_slot(2), feat("two-weapon-fighting")),
                    (class_slot(6), feat("improved-two-weapon-fighting")),
                    (class_slot(10), feat("greater-two-weapon-fighting")),
                    (class_slot(14), feat("double-slice")),
                    (class_slot(18), feat("shield-bash")),
                ],
            ),
        ),
        (
            "titan 10 greater twf without improved",
            with(
                titan(10),
                vec![(class_slot(10), feat("greater-two-weapon-fighting"))],
            ),
        ),
        (
            "hunter 2 outflank",
            with(
                human(&only("hunter", None, 2)),
                vec![(class_slot(2), feat("outflank"))],
            ),
        ),
        (
            "urban hunter 12 captor",
            with(
                human(&only("hunter", Some("hunter/urban-hunter"), 12)),
                vec![
                    (class_slot(6), feat("trip")),
                    (class_slot(9), feat("disarm")),
                    (class_slot(12), feat("greater-trip")),
                ],
            ),
        ),
        (
            "stormwalker 14 archery",
            with(
                human(&only("ranger", Some("ranger/stormwalker"), 14)),
                vec![
                    (class_slot(2), feat("precise-shot")),
                    (class_slot(6), feat("manyshot")),
                    (class_slot(10), feat_for("improved-critical", "long-bow")),
                    (class_slot(14), feat_for("improved-critical", "longsword")),
                ],
            ),
        ),
        (
            "ranger 2 titan 6 style and massive attacks",
            with(
                human(&[
                    ("ranger", None, 2),
                    ("fighter", Some("fighter/titan-fighter"), 6),
                ]),
                vec![
                    (
                        selection(2, 0),
                        option("ranger/combat-style-feat/two-weapon-combat-feats"),
                    ),
                    (class_slot(2), feat("double-slice")),
                    (class_slot(4), feat("two-weapon-fighting")),
                ],
            ),
        ),
        // Builds no sheet holds.
        (
            "titan 6 greater twf at 6",
            with(
                titan(6),
                vec![(class_slot(6), feat("greater-two-weapon-fighting"))],
            ),
        ),
        (
            "titan 6 twf twice",
            with(
                titan(6),
                vec![
                    (class_slot(2), feat("two-weapon-fighting")),
                    (class_slot(6), feat("two-weapon-fighting")),
                ],
            ),
        ),
        (
            "titan 2 power attack",
            with(titan(2), vec![(class_slot(2), feat("power-attack"))]),
        ),
        (
            "hunter 3 precise shot twice",
            with(
                human(&only("hunter", None, 3)),
                vec![
                    (class_slot(2), feat("precise-shot")),
                    (class_slot(3), feat("precise-shot")),
                ],
            ),
        ),
        (
            "stormwalker 14 improved critical twice",
            with(
                human(&only("ranger", Some("ranger/stormwalker"), 14)),
                vec![
                    (class_slot(10), feat_for("improved-critical", "long-bow")),
                    (class_slot(14), feat_for("improved-critical", "long-bow")),
                ],
            ),
        ),
    ];
    for (name, build) in fixed {
        out.push((format!("t61 {name}"), build));
    }
    out
}

/// The sorcerer's and slayer's feat slots: a kitsune Nine Tailed Heir, a Geomancer, a fighter 1 / slayer 2 and a
/// rogue 1 / slayer 2, and picks the slots' lists touch (an Empyreal Sorcerer's and a Seeker's level-1 feat off the
/// bloodline's list, a base sorcerer's level-1 feat, Extra Slayer Talent at slayer 1 and at slayer 2) (`decision
/// …`). True neutral.
pub(super) fn sorcerer_and_slayer_slots() -> Vec<Named> {
    let make =
        |race: &str, parts: &[(&str, Option<&str>, usize)]| character(race, TrueNeutral, parts);
    let feat_at = |mut build: Build, level: u8, source: FeatSource, key: &str| {
        build.picks.insert(
            SlotId::Feat {
                level,
                source,
                index: 0,
            },
            feat(key),
        );
        build
    };
    let bloodline = |mut build: Build, key: &str| {
        build.picks.insert(selection(1, 0), option(key));
        build
    };
    vec![
        (
            "decision nine-tailed-heir fey".into(),
            bloodline(
                make(
                    "kitsune",
                    &only("sorcerer", Some("sorcerer/nine-tailed-heir"), 7),
                ),
                "sorcerer/bloodline/fey",
            ),
        ),
        (
            "decision geomancer abyssal".into(),
            bloodline(
                make("human", &only("sorcerer", Some("sorcerer/geomancer"), 3)),
                "sorcerer/bloodline/abyssal",
            ),
        ),
        (
            "decision fighter1-slayer2".into(),
            make("human", &[("fighter", None, 1), ("slayer", None, 2)]),
        ),
        (
            "decision rogue1-slayer2".into(),
            make("human", &[("rogue", None, 1), ("slayer", None, 2)]),
        ),
        (
            "decision stale empyreal toughness".into(),
            feat_at(
                make(
                    "human",
                    &only("sorcerer", Some("sorcerer/empyreal-sorcerer"), 3),
                ),
                1,
                FeatSource::Class,
                "toughness",
            ),
        ),
        (
            "decision stale seeker extend".into(),
            feat_at(
                bloodline(
                    make(
                        "human",
                        &only("sorcerer", Some("sorcerer/seeker-sorcerer"), 3),
                    ),
                    "sorcerer/bloodline/abyssal",
                ),
                1,
                FeatSource::Class,
                "metamagic-extend-spell",
            ),
        ),
        (
            "decision new sorcerer toughness".into(),
            feat_at(
                bloodline(
                    make("human", &only("sorcerer", None, 3)),
                    "sorcerer/bloodline/draconic-red",
                ),
                1,
                FeatSource::Class,
                "toughness",
            ),
        ),
        (
            "decision stale slayer1 extra-slayer-talent".into(),
            feat_at(
                make("human", &only("slayer", None, 3)),
                1,
                FeatSource::General,
                "extra-slayer-talent",
            ),
        ),
        (
            "decision fighter1-slayer2 extra-slayer-talent".into(),
            feat_at(
                make("human", &[("fighter", None, 1), ("slayer", None, 2)]),
                3,
                FeatSource::General,
                "extra-slayer-talent",
            ),
        ),
    ]
}

/// The talents that repeat a feat, picked through each selection that offers them: kineticists, rogues and slayers
/// taking Skill Focus for two skills or one twice, rogues taking Combat Trick twice with Weapon Focus in two weapons or
/// one twice, the rogue talent Weapon Focus twice, Combat Trick through the slayer's talents, Fey Tricks and
/// Discovery (`follow …`). True neutral humans.
pub(super) fn repeats() -> Vec<Named> {
    let start = |class: &str, archetype: Option<&str>, n: usize| {
        character("human", TrueNeutral, &only(class, archetype, n))
    };
    let sel = |level| selection(level, 0);
    let trick_slot = |level| SlotId::OptionFeat { level, index: 0 };
    let skill = |key: &str| Parameter::Skill(SkillKey::new(key));
    let weapon = |key: &str| Parameter::Weapon(WeaponKey::new(key));
    let option_for = |key: &str, value| Pick::option_for(SelectionOptionKey::new(key), value);
    let focus = |value| Pick::feat_for(FeatKey::new("weapon-focus"), value);
    const KIN_SF: &str = "kineticist/wild-talents/skill-focus";
    const ROGUE_SF: &str = "rogue/rogue-talents/skill-focus";
    const ROGUE_WF: &str = "rogue/rogue-talents/weapon-focus";
    const TRICK: &str = "rogue/rogue-talents/combat-trick";
    let kineticist = with(
        start("kineticist", None, 8),
        vec![(sel(1), option("kineticist/element/earth"))],
    );
    let mut out: Vec<Named> = vec![
        (
            "follow kin sf two skills".into(),
            with(
                kineticist.clone(),
                vec![
                    (sel(2), option_for(KIN_SF, skill("perception"))),
                    (sel(4), option_for(KIN_SF, skill("stealth"))),
                ],
            ),
        ),
        (
            "follow stale kin sf one skill twice".into(),
            with(
                kineticist.clone(),
                vec![
                    (sel(2), option_for(KIN_SF, skill("perception"))),
                    (sel(4), option_for(KIN_SF, skill("perception"))),
                ],
            ),
        ),
        (
            "follow kin sf once and bonus feat".into(),
            with(
                kineticist,
                vec![
                    (sel(2), option_for(KIN_SF, skill("perception"))),
                    (
                        sel(4),
                        option("kineticist/wild-talents/kineticist-bonus-feat"),
                    ),
                ],
            ),
        ),
    ];
    for class in ["rogue", "slayer"] {
        out.push((
            format!("follow {class} sf two skills"),
            with(
                start(class, None, 6),
                vec![
                    (sel(2), option_for(ROGUE_SF, skill("perception"))),
                    (sel(4), option_for(ROGUE_SF, skill("stealth"))),
                ],
            ),
        ));
        out.push((
            format!("follow stale {class} sf one skill twice"),
            with(
                start(class, None, 6),
                vec![
                    (sel(2), option_for(ROGUE_SF, skill("stealth"))),
                    (sel(4), option_for(ROGUE_SF, skill("stealth"))),
                ],
            ),
        ));
    }
    out.push((
        "follow rogue trick twice".into(),
        with(
            start("rogue", None, 8),
            vec![
                (sel(2), option(TRICK)),
                (trick_slot(2), focus(weapon("longsword"))),
                (sel(4), option(TRICK)),
                (trick_slot(4), focus(weapon("dagger"))),
                (sel(6), option_for(ROGUE_WF, weapon("rapier"))),
            ],
        ),
    ));
    out.push((
        "follow stale rogue trick one weapon twice".into(),
        with(
            start("rogue", None, 4),
            vec![
                (sel(2), option(TRICK)),
                (trick_slot(2), focus(weapon("longsword"))),
                (sel(4), option(TRICK)),
                (trick_slot(4), focus(weapon("longsword"))),
            ],
        ),
    ));
    out.push((
        "follow stale rogue wf talent twice".into(),
        with(
            start("rogue", None, 4),
            vec![
                (sel(2), option_for(ROGUE_WF, weapon("longsword"))),
                (sel(4), option_for(ROGUE_WF, weapon("dagger"))),
            ],
        ),
    ));
    out.push((
        "follow rogue trick once".into(),
        with(start("rogue", None, 4), vec![(sel(2), option(TRICK))]),
    ));
    out.push((
        "follow slayer trick once".into(),
        with(start("slayer", None, 4), vec![(sel(2), option(TRICK))]),
    ));
    out.push((
        "follow sylvan trick once".into(),
        with(
            start("rogue", Some("rogue/sylvan-trickster"), 4),
            vec![(sel(2), option(TRICK))],
        ),
    ));
    out.push((
        "follow chemist trick once".into(),
        with(
            start("rogue", Some("rogue/underground-chemist"), 12),
            vec![(sel(10), option(TRICK))],
        ),
    ));
    out
}
