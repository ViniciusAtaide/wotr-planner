//! Class talents and their kin: rage powers and totem lines, wild talents and infusions, mercies, arcanist exploits,
//! ki powers, and the talents other classes borrow.

use rules::Alignment::{self, *};
use rules::{Build, Engine, Parameter, Pick, SelectionOptionKey, SkillKey, SlotId};

use super::{
    character, empty_and_filled, feat, feat_selection, general, human, option, selection, with,
};
use crate::Named;
use crate::generate::with_selections_filled;

/// One class for its levels with its archetype, and these options picked at their levels' first selection slot.
fn one_class(
    class: &str,
    archetype: Option<&str>,
    alignment: Alignment,
    levels: usize,
    picks: &[(u8, &str)],
) -> Build {
    let picks = picks
        .iter()
        .map(|(level, key)| (selection(*level, 0), option(key)))
        .collect();
    with(
        character("human", alignment, &[(class, archetype, levels)]),
        picks,
    )
}

/// A barbarian 20 holding each totem line (the Beast chain at 2, 6 and 10; another line's Lesser at 2), a barbarian
/// holding two lines' Lesser powers, a Primalist bloodrager, an Eldritch Scoundrel 20 and shamans 14 (base, Possessed,
/// Spirit Warden) (`gap …`).
pub(super) fn totem_lines() -> Vec<Named> {
    let power = |key: &str| format!("barbarian/rage-power/{key}");
    let mut out = Vec::new();
    let beast = [
        (2, power("beast-totem-lesser")),
        (6, power("beast-totem")),
        (10, power("beast-totem-greater")),
    ];
    let beast: Vec<(u8, &str)> = beast.iter().map(|(l, k)| (*l, k.as_str())).collect();
    out.push((
        "gap barbarian beast".into(),
        one_class("barbarian", None, ChaoticNeutral, 20, &beast),
    ));
    for line in ["celestial", "daemon", "fiend"] {
        let lesser = power(&format!("{line}-totem-lesser"));
        out.push((
            format!("gap barbarian {line}"),
            one_class("barbarian", None, ChaoticNeutral, 20, &[(2, &lesser)]),
        ));
    }
    let (beast_lesser, fiend_lesser) = (power("beast-totem-lesser"), power("fiend-totem-lesser"));
    out.push((
        "gap barbarian beast+fiend".into(),
        one_class(
            "barbarian",
            None,
            ChaoticNeutral,
            8,
            &[(2, &beast_lesser), (4, &fiend_lesser)],
        ),
    ));
    out.push((
        "gap bloodrager primalist".into(),
        one_class(
            "bloodrager",
            Some("bloodrager/primalist"),
            ChaoticNeutral,
            12,
            &[],
        ),
    ));
    out.push((
        "gap eldritch-scoundrel".into(),
        one_class(
            "rogue",
            Some("rogue/eldritch-scoundrel"),
            TrueNeutral,
            20,
            &[],
        ),
    ));
    for archetype in [
        None,
        Some("shaman/possessed-shaman"),
        Some("shaman/spirit-warden"),
    ] {
        out.push((
            format!("gap shaman {archetype:?}"),
            one_class("shaman", archetype, TrueNeutral, 14, &[]),
        ));
    }
    out
}

/// Kineticists holding wild talents and their bonus feat, Expanded Defense, a Kinetic Knight and a Blood Kineticist
/// with talents, a talent off the element, Extra Wild Talent at kineticist 1 and 3 (`talent …`). True neutral humans.
pub(super) fn wild_talents() -> Vec<Named> {
    let kineticist =
        |archetype: Option<&str>, n: usize, element: Option<&str>, picks: &[(u8, &str)]| {
            let mut build = character("human", TrueNeutral, &[("kineticist", archetype, n)]);
            if let Some(element) = element {
                build.picks.insert(
                    selection(1, 0),
                    option(&format!("kineticist/element/{element}")),
                );
            }
            for (level, key) in picks {
                let key = if key.starts_with("kineticist/") {
                    key.to_string()
                } else {
                    format!("kineticist/wild-talents/{key}")
                };
                build.picks.insert(selection(*level, 0), option(&key));
            }
            build
        };
    let mut fire = kineticist(
        None,
        20,
        Some("fire"),
        &[
            (2, "kineticist-bonus-feat"),
            (4, "foxfire"),
            (6, "flame-shield"),
            (7, "kineticist/element/water"),
            (8, "expanded-defense"),
            (10, "kinetic-healer"),
            (12, "healing-burst"),
            (15, "kineticist/element/air"),
            (16, "aerial-evasion"),
            (18, "tidal-wave"),
            (20, "kinetic-revivification"),
        ],
    );
    fire.picks.insert(
        selection(14, 0),
        Pick::option_for(
            SelectionOptionKey::new("kineticist/wild-talents/skill-focus"),
            Parameter::Skill(SkillKey::new("perception")),
        ),
    );
    fire.picks.insert(
        SlotId::OptionFeat { level: 2, index: 0 },
        feat("improved-iron-will"),
    );
    let extra_at = |level| {
        with(
            kineticist(None, 3, Some("fire"), &[]),
            vec![(general(level), feat("extra-wild-talent"))],
        )
    };
    vec![
        ("talent fire 20".into(), fire),
        (
            "talent knight earth".into(),
            kineticist(
                Some("kineticist/kinetic-knight"),
                8,
                Some("earth"),
                &[
                    (2, "elemental-whispers"),
                    (4, "skilled-kineticist"),
                    (6, "jagged-flesh"),
                ],
            ),
        ),
        (
            "talent blood".into(),
            kineticist(
                Some("kineticist/blood-kineticist"),
                20,
                None,
                &[
                    (2, "kinetic-healer"),
                    (12, "kinetic-revivification"),
                    (20, "tidal-wave"),
                ],
            ),
        ),
        (
            "talent stale fire kinetic-healer".into(),
            kineticist(None, 4, Some("fire"), &[(2, "kinetic-healer")]),
        ),
        (
            "talent stale air celerity at 4".into(),
            kineticist(None, 6, Some("air"), &[(4, "celerity")]),
        ),
        (
            "talent stale bonus feat twice".into(),
            kineticist(
                None,
                4,
                Some("fire"),
                &[(2, "kineticist-bonus-feat"), (4, "kineticist-bonus-feat")],
            ),
        ),
        ("talent extra-wild-talent at 1".into(), extra_at(1)),
        ("talent extra-wild-talent at 3".into(), extra_at(3)),
    ]
}

/// Mercies and exploits: a paladin 20 and each archetype's (a dwarf: the Stonelord's race), an Arcane Enforcer 20,
/// each empty and with its selections filled; Extra Mercy (at 3 and 9; a Stonelord's at 7) and Extra Arcanist Exploit
/// (at 1 and 5) with the extra slots filled; an Enforcer who becomes an arcanist; and Builds no sheet holds (a level-6
/// mercy at 3, Exhausted without Fatigued, Extra Mercy with no mercy, one exploit twice). Dwarves, the paladins
/// lawful good, the others chaotic neutral.
pub(super) fn mercies_and_exploits(engine: &Engine) -> Vec<Named> {
    let make = |class: &str, archetype: Option<&str>, alignment: Alignment, levels: usize| {
        character("dwarf", alignment, &[(class, archetype, levels)])
    };
    let mut paladins: Vec<(String, Build)> =
        vec![("paladin".into(), make("paladin", None, LawfulGood, 20))];
    for archetype in [
        "divine-guardian",
        "divine-hunter-paladin",
        "divine-scion",
        "hospitaler",
        "martyr",
        "stonelord",
        "tortured-crusader",
        "warrior-of-the-holy-light",
    ] {
        paladins.push((
            archetype.into(),
            make(
                "paladin",
                Some(&format!("paladin/{archetype}")),
                LawfulGood,
                20,
            ),
        ));
    }
    paladins.push((
        "arcane enforcer".into(),
        make("slayer", Some("slayer/arcane-enforcer"), ChaoticNeutral, 20),
    ));
    let mut out = empty_and_filled(engine, "t54", paladins);
    for (name, build, key, levels) in [
        (
            "paladin extra mercy",
            make("paladin", None, LawfulGood, 9),
            "extra-mercy",
            vec![3, 9],
        ),
        (
            "stonelord extra mercy",
            make("paladin", Some("paladin/stonelord"), LawfulGood, 7),
            "extra-mercy",
            vec![7],
        ),
        (
            "enforcer extra exploit",
            make("slayer", Some("slayer/arcane-enforcer"), ChaoticNeutral, 7),
            "extra-arcanist-exploit",
            vec![1, 5],
        ),
    ] {
        let mut build = with_selections_filled(engine, build);
        for level in levels {
            build.picks.insert(general(level), feat(key));
        }
        out.push((format!("t54 {name} / feat"), build.clone()));
        out.push((
            format!("t54 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    // An Enforcer who becomes an arcanist: one selection, so an exploit is taken once.
    let both = character(
        "dwarf",
        ChaoticNeutral,
        &[
            ("slayer", Some("slayer/arcane-enforcer"), 1),
            ("arcanist", None, 2),
        ],
    );
    out.push((
        "t54 enforcer then arcanist / filled".into(),
        with_selections_filled(engine, both.clone()),
    ));
    let wooden_flesh = || option("arcanist/arcanist-exploits/wooden-flesh");
    out.push((
        "t54 enforcer then arcanist / twice".into(),
        with(
            both,
            vec![
                (selection(1, 0), wooden_flesh()),
                (selection(3, 0), wooden_flesh()),
            ],
        ),
    ));
    // Builds no sheet holds.
    out.push((
        "t54 dazed at 3".into(),
        with(
            make("paladin", None, LawfulGood, 6),
            vec![(selection(3, 0), option("paladin/mercy/dazed"))],
        ),
    ));
    out.push((
        "t54 exhausted without fatigued".into(),
        with(
            make("paladin", None, LawfulGood, 9),
            vec![(selection(9, 0), option("paladin/mercy/exhausted"))],
        ),
    ));
    out.push((
        "t54 stonelord extra mercy at 3".into(),
        with(
            make("paladin", Some("paladin/stonelord"), LawfulGood, 5),
            vec![(general(3), feat("extra-mercy"))],
        ),
    ));
    let armored_mask = || option("arcanist/arcanist-exploits/armored-mask");
    out.push((
        "t54 enforcer exploit twice".into(),
        with(
            make("slayer", Some("slayer/arcane-enforcer"), ChaoticNeutral, 4),
            vec![
                (selection(1, 0), armored_mask()),
                (selection(4, 0), armored_mask()),
            ],
        ),
    ));
    out
}

/// Greater exploits: an Arcane Enforcer 12 and 13 and an Exploiter Wizard 13 and 20, each empty and with its
/// selections filled; an Exploiter's Extra Arcanist Exploit at 1 and 5, its extra slots filled; fixed greater
/// exploits (an Enforcer's Burning Flame, an Exploiter's Hellfire Ray); an Enforcer who becomes an Exploiter Wizard;
/// and Builds no sheet holds (Icy Tomb without Ice Missile, one exploit twice, the feat with no exploit, a greater
/// exploit at 9). Chaotic neutral humans.
pub(super) fn greater_exploits(engine: &Engine) -> Vec<Named> {
    let enforcer_key = Some("slayer/arcane-enforcer");
    let exploiter_key = Some("wizard/exploiter-wizard");
    let enforcer = |levels: usize| human(&[("slayer", enforcer_key, levels)]);
    let exploiter = |levels: usize| human(&[("wizard", exploiter_key, levels)]);
    let exploit = |name: &str| option(&format!("arcanist/arcanist-exploits/{name}"));
    let greater = |name: &str| option(&format!("arcanist/greater-exploits/{name}"));
    let mut out = empty_and_filled(
        engine,
        "t57",
        vec![
            ("enforcer 12".to_owned(), enforcer(12)),
            ("enforcer 13".to_owned(), enforcer(13)),
            ("exploiter 13".to_owned(), exploiter(13)),
            ("exploiter 20".to_owned(), exploiter(20)),
        ],
    );
    let mut extra = with_selections_filled(engine, exploiter(9));
    for level in [1, 5] {
        extra
            .picks
            .insert(general(level), feat("extra-arcanist-exploit"));
    }
    out.push(("t57 exploiter extra exploit / feat".into(), extra.clone()));
    out.push((
        "t57 exploiter extra exploit / filled".into(),
        with_selections_filled(engine, extra),
    ));
    out.push((
        "t57 enforcer burning flame".into(),
        with(
            enforcer(13),
            vec![
                (selection(1, 0), exploit("flame-arc")),
                (selection(13, 1), greater("burning-flame")),
            ],
        ),
    ));
    out.push((
        "t57 exploiter hellfire ray".into(),
        with(
            exploiter(13),
            vec![
                (selection(1, 0), exploit("wooden-flesh")),
                (selection(5, 0), exploit("flame-arc")),
                (selection(13, 1), greater("hellfire-ray")),
            ],
        ),
    ));
    // An Enforcer who becomes an Exploiter Wizard: one selection, so an exploit is taken once.
    let both = human(&[("slayer", enforcer_key, 1), ("wizard", exploiter_key, 2)]);
    out.push((
        "t57 enforcer then exploiter / filled".into(),
        with_selections_filled(engine, both.clone()),
    ));
    out.push((
        "t57 enforcer then exploiter / twice".into(),
        with(
            both,
            vec![
                (selection(1, 0), exploit("wooden-flesh")),
                (selection(2, 0), exploit("wooden-flesh")),
            ],
        ),
    ));
    // Builds no sheet holds.
    out.push((
        "t57 enforcer icy tomb without ice missile".into(),
        with(enforcer(13), vec![(selection(13, 1), greater("icy-tomb"))]),
    ));
    out.push((
        "t57 exploiter exploit twice".into(),
        with(
            exploiter(5),
            vec![
                (selection(1, 0), exploit("armored-mask")),
                (selection(5, 0), exploit("armored-mask")),
            ],
        ),
    ));
    out.push((
        "t57 exploiter extra exploit with no exploit".into(),
        with(
            exploiter(3),
            vec![(general(1), feat("extra-arcanist-exploit"))],
        ),
    ));
    out.push((
        "t57 exploiter greater exploit at 9".into(),
        with(
            exploiter(12),
            vec![
                (selection(1, 0), exploit("flame-arc")),
                (selection(9, 1), greater("burning-flame")),
            ],
        ),
    ));
    out
}

/// Ki powers: a monk 3, 4, 8 and 20 and each archetype's 20, each empty and with its selections filled; a monk 4 /
/// rogue 4 likewise; a monk 16 holding Barkskin at 4, Abundant Step at 8 and Quivering Palm at 16; and Builds no sheet
/// holds (Abundant Step at 4, Barkskin twice, a Traditional Monk's or a Sohei's ki power where it picks none). Lawful
/// neutral humans, the Drunken Master chaotic neutral and the Student of Stone an oread.
pub(super) fn ki_powers(engine: &Engine) -> Vec<Named> {
    let monk = |levels: usize, archetype: Option<&str>| {
        let race = if archetype == Some("monk/student-of-stone") {
            "oread"
        } else {
            "human"
        };
        let alignment = if archetype == Some("monk/drunken-master") {
            ChaoticNeutral
        } else {
            LawfulNeutral
        };
        character(race, alignment, &[("monk", archetype, levels)])
    };
    let power = |name: &str| option(&format!("monk/ki-power/{name}"));
    let mut plain = vec![
        ("monk 3".to_owned(), monk(3, None)),
        ("monk 4".to_owned(), monk(4, None)),
        ("monk 8".to_owned(), monk(8, None)),
        ("monk 20".to_owned(), monk(20, None)),
    ];
    for archetype in [
        "drunken-master",
        "quarterstaff-master",
        "scaled-fist",
        "sensei",
        "sohei",
        "student-of-stone",
        "traditional-monk",
        "zen-archer",
    ] {
        plain.push((
            format!("{archetype} 20"),
            monk(20, Some(&format!("monk/{archetype}"))),
        ));
    }
    plain.push((
        "monk 4 rogue 4".to_owned(),
        character(
            "human",
            LawfulNeutral,
            &[("monk", None, 4), ("rogue", None, 4)],
        ),
    ));
    let mut out = empty_and_filled(engine, "t58", plain);
    out.push((
        "t58 monk 16 barkskin abundant step quivering palm".into(),
        with(
            monk(16, None),
            vec![
                (selection(4, 0), power("barkskin")),
                (selection(8, 0), power("abundant-step")),
                (selection(16, 0), power("quivering-palm")),
            ],
        ),
    ));
    // Builds no sheet holds.
    out.push((
        "t58 abundant step at 4".into(),
        with(
            monk(8, None),
            vec![(selection(4, 0), power("abundant-step"))],
        ),
    ));
    out.push((
        "t58 barkskin twice".into(),
        with(
            monk(6, None),
            vec![
                (selection(4, 0), power("barkskin")),
                (selection(6, 0), power("barkskin")),
            ],
        ),
    ));
    out.push((
        "t58 traditional monk ki power at 6".into(),
        with(
            monk(8, Some("monk/traditional-monk")),
            vec![(selection(6, 0), power("barkskin"))],
        ),
    ));
    out.push((
        "t58 sohei ki power at 4".into(),
        with(
            monk(4, Some("monk/sohei")),
            vec![(selection(4, 0), power("barkskin"))],
        ),
    ));
    out
}

/// Infusions: a Fire kineticist 2, 3, 5, 9 and 20 (and one 20 with no element) and each archetype's 20 (Fire, the
/// Blood Kineticist its own Water), each empty and with its selections filled; an Earth kineticist 3 / rogue 3
/// likewise; a Fire kineticist 19 holding Extended Range, Burning Infusion, Water by Expanded Element, Spray, Wall,
/// Kinetic Blade, Grappling Infusion and Foxfire Infusion (after the Foxfire wild talent); a Kinetic Sharpshooter 13
/// holding Exploding Arrows, Burning Infusion, Spindle, Rending Arrows and Wall; and Builds no sheet holds (Detonation
/// at 3, Spray on Fire alone, Extended Range twice, Exploding Arrows on the base class, Extended Range on the
/// Sharpshooter, an infusion at a Blood Kineticist's 5 or a Kinetic Knight's 3). True neutral humans.
pub(super) fn infusions(engine: &Engine) -> Vec<Named> {
    let infusion = |name: &str| option(&format!("kineticist/infusions/{name}"));
    let make = |parts: &[(&str, Option<&str>, usize)], element: Option<&str>| {
        let mut build = character("human", TrueNeutral, parts);
        if let Some(element) = element {
            build.picks.insert(
                selection(1, 0),
                option(&format!("kineticist/element/{element}")),
            );
        }
        build
    };
    let kineticist = |levels: usize, archetype: Option<&str>, element: Option<&str>| {
        make(&[("kineticist", archetype, levels)], element)
    };
    let fire = Some("fire");
    let mut plain = vec![
        ("kineticist 2".to_owned(), kineticist(2, None, fire)),
        ("kineticist 3".to_owned(), kineticist(3, None, fire)),
        ("kineticist 5".to_owned(), kineticist(5, None, fire)),
        ("kineticist 9".to_owned(), kineticist(9, None, fire)),
        ("kineticist 20".to_owned(), kineticist(20, None, fire)),
        (
            "kineticist 20 no element".to_owned(),
            kineticist(20, None, None),
        ),
    ];
    for archetype in [
        "blood-kineticist",
        "dark-elementalist",
        "elemental-engine",
        "kinetic-knight",
        "kinetic-sharpshooter",
        "overwhelming-soul",
        "psychokineticist",
    ] {
        // The Blood Kineticist's Water comes with it (its element selection has the one option, so no slot).
        let element = if archetype == "blood-kineticist" {
            None
        } else {
            fire
        };
        plain.push((
            format!("{archetype} 20"),
            kineticist(20, Some(&format!("kineticist/{archetype}")), element),
        ));
    }
    plain.push((
        "kineticist 3 rogue 3".to_owned(),
        make(
            &[("kineticist", None, 3), ("rogue", None, 3)],
            Some("earth"),
        ),
    ));
    let mut out = empty_and_filled(engine, "t59", plain);
    out.push((
        "t59 kineticist 19 eight infusions".into(),
        with(
            kineticist(19, None, fire),
            vec![
                (selection(3, 0), infusion("extended-range")),
                (selection(4, 0), option("kineticist/wild-talents/foxfire")),
                (selection(5, 0), infusion("burning-infusion")),
                (selection(7, 0), option("kineticist/element/water")),
                (selection(9, 0), infusion("spray")),
                (selection(11, 0), infusion("wall")),
                (selection(13, 0), infusion("kinetic-blade")),
                (selection(17, 0), infusion("grappling-infusion")),
                (selection(19, 0), infusion("foxfire-infusion")),
            ],
        ),
    ));
    let sharpshooter = Some("kineticist/kinetic-sharpshooter");
    out.push((
        "t59 sharpshooter 13 five infusions".into(),
        with(
            kineticist(13, sharpshooter, fire),
            vec![
                (selection(3, 0), infusion("exploding-arrows")),
                (selection(5, 0), infusion("burning-infusion")),
                (selection(9, 0), infusion("spindle")),
                (selection(11, 0), infusion("rending-arrows")),
                (selection(13, 0), infusion("wall")),
            ],
        ),
    ));
    // Builds no sheet holds.
    let one = |name: &str, build: Build, picks: &[(u8, &str)]| {
        let picks = picks
            .iter()
            .map(|(level, key)| (selection(*level, 0), infusion(key)))
            .collect();
        (format!("t59 {name}"), with(build, picks))
    };
    out.push(one(
        "detonation at 3",
        kineticist(3, None, fire),
        &[(3, "detonation")],
    ));
    out.push(one(
        "spray on fire",
        kineticist(9, None, fire),
        &[(9, "spray")],
    ));
    out.push(one(
        "extended range twice",
        kineticist(5, None, fire),
        &[(3, "extended-range"), (5, "extended-range")],
    ));
    out.push(one(
        "exploding arrows on the base class",
        kineticist(3, None, fire),
        &[(3, "exploding-arrows")],
    ));
    out.push(one(
        "extended range on the sharpshooter",
        kineticist(3, sharpshooter, fire),
        &[(3, "extended-range")],
    ));
    out.push(one(
        "blood kineticist infusion at 5",
        kineticist(5, Some("kineticist/blood-kineticist"), None),
        &[(5, "extended-range")],
    ));
    out.push(one(
        "kinetic knight infusion at 3",
        kineticist(3, Some("kineticist/kinetic-knight"), fire),
        &[(3, "extended-range")],
    ));
    out
}

/// The talents other classes borrow: a bard, a skald and each of their archetypes, a Feyform Shifter, a
/// Vivisectionist, a Sanctified Slayer and a Hexcrafter, each 20, empty and filled; a rogue 2 / bard 2, a bard 2 /
/// skald 2, a rogue 2 / Feyform 8, a rogue 2 / Vivisectionist 2, a slayer 2 / Sanctified Slayer 8 and a witch 1 /
/// Hexcrafter 6, empty and filled; fixed picks (a bard 3's Combat Trick and Extra Rogue Talent's slot, an
/// Archaeologist 10's Crippling Strike, a Feyform 9's Extra Rogue Talent, a Sanctified Slayer 9's Extra Slayer Talent, a
/// Hexcrafter 18's hexes and arcana with Extra Hex, a Vivisectionist 10's discoveries and talents); and Builds no sheet
/// holds. Chaotic neutral humans.
pub(super) fn borrowed_talents(engine: &Engine) -> Vec<Named> {
    let talent = |name: &str| option(&format!("rogue/rogue-talents/{name}"));
    let feyform = Some("shifter/feyform-shifter");
    let vivisectionist = Some("alchemist/vivisectionist");
    let sanctified = Some("inquisitor/sanctified-slayer");
    let hexcrafter = Some("magus/hexcrafter");
    let mut plain: Vec<(String, Build)> = Vec::new();
    for (class, archetypes) in [
        (
            "bard",
            [
                "archaeologist",
                "beast-tamer",
                "chelaxian-diva",
                "dirge-bard",
                "flame-dancer",
                "thundercaller",
                "tranquil-whisperer",
            ],
        ),
        (
            "skald",
            [
                "battle-scion",
                "battle-singer",
                "court-poet",
                "demon-dancer",
                "herald-of-the-horn",
                "hunt-caller",
                "inciter",
            ],
        ),
    ] {
        plain.push((format!("{class} 20"), human(&[(class, None, 20)])));
        for archetype in archetypes {
            let key = format!("{class}/{archetype}");
            let build = human(&[(class, Some(&key), 20)]);
            plain.push((format!("{key} 20"), build));
        }
    }
    for (class, archetype) in [
        ("shifter", feyform),
        ("alchemist", vivisectionist),
        ("inquisitor", sanctified),
        ("magus", hexcrafter),
    ] {
        plain.push((
            format!("{} 20", archetype.expect("an archetype")),
            human(&[(class, archetype, 20)]),
        ));
    }
    for (name, parts) in [
        ("rogue 2 bard 2", [("rogue", None, 2), ("bard", None, 2)]),
        ("bard 2 skald 2", [("bard", None, 2), ("skald", None, 2)]),
        (
            "rogue 2 feyform 8",
            [("rogue", None, 2), ("shifter", feyform, 8)],
        ),
        (
            "rogue 2 vivisectionist 2",
            [("rogue", None, 2), ("alchemist", vivisectionist, 2)],
        ),
        (
            "slayer 2 sanctified 8",
            [("slayer", None, 2), ("inquisitor", sanctified, 8)],
        ),
        (
            "witch 1 hexcrafter 6",
            [("witch", None, 1), ("magus", hexcrafter, 6)],
        ),
    ] {
        plain.push((name.to_owned(), human(&parts)));
    }
    let mut out = empty_and_filled(engine, "t60", plain);
    let hex = |name: &str| option(&format!("witch/hex/{name}"));
    let fixed = vec![
        (
            "bard 3 extra rogue talent",
            with(
                human(&[("bard", None, 3)]),
                vec![
                    (selection(2, 0), talent("combat-trick")),
                    (general(3), feat("extra-rogue-talent")),
                    (feat_selection(3), talent("fast-stealth")),
                ],
            ),
        ),
        (
            "archaeologist 10 crippling strike",
            with(
                human(&[("bard", Some("bard/archaeologist"), 10)]),
                vec![
                    (selection(2, 0), talent("slow-reactions")),
                    (selection(6, 0), talent("fast-stealth")),
                    (selection(10, 0), talent("crippling-strike")),
                ],
            ),
        ),
        (
            "feyform 9 extra rogue talent",
            with(
                human(&[("shifter", feyform, 9)]),
                vec![
                    (selection(8, 0), talent("combat-trick")),
                    (general(9), feat("extra-rogue-talent")),
                    (feat_selection(9), talent("iron-guts")),
                ],
            ),
        ),
        (
            "sanctified 9 extra slayer talent",
            with(
                human(&[("inquisitor", sanctified, 9)]),
                vec![
                    (selection(8, 0), option("slayer/slayer-talents/trapfinding")),
                    (general(9), feat("extra-slayer-talent")),
                    (feat_selection(9), talent("combat-trick")),
                ],
            ),
        ),
        (
            "hexcrafter 18 hexes",
            with(
                human(&[("magus", hexcrafter, 18)]),
                vec![
                    (selection(4, 0), hex("slumber")),
                    (general(5), feat("extra-hex")),
                    (feat_selection(5), hex("cackle")),
                    (
                        selection(6, 0),
                        option("magus/magus-arcana/arcane-accuracy"),
                    ),
                    (selection(12, 0), hex("restless-slumber")),
                    (selection(18, 0), hex("death-curse")),
                ],
            ),
        ),
        (
            "vivisectionist 10",
            with(
                human(&[("alchemist", vivisectionist, 10)]),
                vec![
                    (selection(2, 0), option("alchemist/discovery/feral-mutagen")),
                    (selection(4, 0), talent("slow-reactions")),
                    (selection(6, 0), option("alchemist/discovery/feral-wings")),
                    (selection(10, 0), talent("confounding-blades")),
                ],
            ),
        ),
        // Builds no sheet holds.
        (
            "bard 2 crippling strike",
            with(
                human(&[("bard", None, 2)]),
                vec![(selection(2, 0), talent("crippling-strike"))],
            ),
        ),
        (
            "feyform 8 crippling strike",
            with(
                human(&[("shifter", feyform, 8)]),
                vec![(selection(8, 0), talent("crippling-strike"))],
            ),
        ),
        (
            "vivisectionist 2 crippling strike",
            with(
                human(&[("alchemist", vivisectionist, 2)]),
                vec![(selection(2, 0), talent("crippling-strike"))],
            ),
        ),
        (
            "sanctified 8 combat style",
            with(
                human(&[("inquisitor", sanctified, 8)]),
                vec![(
                    selection(8, 0),
                    option("slayer/slayer-talents/combat-style-feat-2"),
                )],
            ),
        ),
        (
            "hexcrafter 9 agony",
            with(
                human(&[("magus", hexcrafter, 9)]),
                vec![(selection(9, 0), hex("agony"))],
            ),
        ),
        (
            "rogue 2 bard 2 fast stealth twice",
            with(
                human(&[("rogue", None, 2), ("bard", None, 2)]),
                vec![
                    (selection(2, 0), talent("fast-stealth")),
                    (selection(4, 0), talent("fast-stealth")),
                ],
            ),
        ),
        (
            "witch 1 hexcrafter 4 cackle twice",
            with(
                human(&[("witch", None, 1), ("magus", hexcrafter, 4)]),
                vec![
                    (selection(1, 0), hex("cackle")),
                    (selection(5, 0), hex("cackle")),
                ],
            ),
        ),
    ];
    for (name, build) in fixed {
        out.push((format!("t60 {name}"), build));
    }
    out
}

/// The Instinctual Warrior's monk powers and Extra Rage Power: an Instinctual Warrior 3, 4, 8, 12, 16 and 20, empty
/// and filled; one 20 with Improved Unarmed Strike and each monk power at its level; one 9 holding only Deflect Arrows
/// with Extra Rage Power at 5 and 9 and Purity of Body in the slot at 9; one 9 holding a plain rage power with Extra
/// Rage Power at 5; a fighter 5 / Instinctual Warrior 8 with Purity of Body at barbarian 8; a plain barbarian 20 with
/// Improved Unarmed Strike; and Builds no sheet holds: Diamond Body at 8, Deflect Arrows without Improved Unarmed
/// Strike, Perfect Self on a plain barbarian 20 and on an Armored Hulk 20, Purity of Body at barbarian 4 (character
/// level 9) on the fighter 5 / Instinctual Warrior 8. Each fixed one also filled. Chaotic neutral humans.
pub(super) fn instinctual_warrior(engine: &Engine) -> Vec<Named> {
    let power = |name: &str| option(&format!("barbarian/rage-power/{name}-instinctual-warrior"));
    let iw = Some("barbarian/instinctual-warrior");
    let warrior = |levels| human(&[("barbarian", iw, levels)]);
    let plain = [3, 4, 8, 12, 16, 20]
        .into_iter()
        .map(|levels| (format!("instinctual warrior {levels}"), warrior(levels)))
        .collect();
    let mut out = empty_and_filled(engine, "t66", plain);
    let unarmed = (general(1), feat("improved-unarmed-strike"));
    let fixed = [
        (
            "each power",
            with(
                warrior(20),
                vec![
                    unarmed.clone(),
                    (selection(4, 0), power("deflect-arrows")),
                    (selection(8, 0), power("purity-of-body")),
                    (selection(12, 0), power("diamond-body")),
                    (selection(16, 0), power("diamond-soul")),
                    (selection(20, 0), power("perfect-self")),
                ],
            ),
        ),
        (
            "deflect arrows only, extra rage power",
            with(
                warrior(9),
                vec![
                    unarmed.clone(),
                    (selection(4, 0), power("deflect-arrows")),
                    (general(5), feat("extra-rage-power")),
                    (general(9), feat("extra-rage-power")),
                    (feat_selection(9), power("purity-of-body")),
                ],
            ),
        ),
        (
            "plain power, extra rage power",
            with(
                warrior(9),
                vec![
                    (selection(4, 0), option("barbarian/rage-power/animal-fury")),
                    (general(5), feat("extra-rage-power")),
                ],
            ),
        ),
        (
            "barbarian 20 unarmed",
            with(human(&[("barbarian", None, 20)]), vec![unarmed.clone()]),
        ),
        (
            "diamond body at 8",
            with(warrior(8), vec![(selection(8, 0), power("diamond-body"))]),
        ),
        (
            "deflect arrows bare",
            with(warrior(4), vec![(selection(4, 0), power("deflect-arrows"))]),
        ),
        (
            "barbarian perfect self",
            with(
                human(&[("barbarian", None, 20)]),
                vec![(selection(20, 0), power("perfect-self"))],
            ),
        ),
        (
            "armored hulk perfect self",
            with(
                human(&[("barbarian", Some("barbarian/armored-hulk"), 20)]),
                vec![(selection(20, 0), power("perfect-self"))],
            ),
        ),
    ];
    for (name, build) in fixed {
        out.push((format!("t66 {name}"), build.clone()));
        out.push((
            format!("t66 {name} / filled"),
            with_selections_filled(engine, build),
        ));
    }
    // A fighter 5 / Instinctual Warrior 8: Purity of Body in the barbarian's slot at character level 13 (barbarian
    // 8), and not in the one at 9 (barbarian 4, character level 9).
    let monk = human(&[("fighter", None, 5), ("barbarian", iw, 8)]);
    out.push((
        "t66 fighter 5 instinctual warrior 8".to_owned(),
        monk.clone(),
    ));
    let purity = with(
        monk.clone(),
        vec![(selection(13, 0), power("purity-of-body"))],
    );
    out.push((
        "t66 fighter 5 instinctual warrior 8 purity".to_owned(),
        purity.clone(),
    ));
    out.push((
        "t66 fighter 5 instinctual warrior 8 purity / filled".to_owned(),
        with_selections_filled(engine, purity),
    ));
    out.push((
        "t66 fighter 5 instinctual warrior 8 purity at 9".to_owned(),
        with(monk, vec![(selection(9, 0), power("purity-of-body"))]),
    ));
    out
}
