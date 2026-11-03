//! Classes, archetypes, prestige classes, backgrounds and alignment rules.

use rules::Alignment::{self, *};
use rules::{Build, BuildStart, ClassChoice, ClassKey, Engine, GameData, LevelUp, RaceKey};

use super::{character, empty_and_filled};
use crate::Named;

/// A true neutral human with nothing else chosen, as ticket 47's Builds start.
pub(super) fn neutral_human() -> Build {
    Build {
        start: BuildStart {
            race: Some(RaceKey::new("human")),
            alignment: Some(TrueNeutral),
            ..BuildStart::default()
        },
        ..Build::default()
    }
}

/// Every class at 20, plain and with each archetype (`<class> <archetype>`, the plain one with nothing after the
/// space).
pub(super) fn every_class(data: &GameData) -> Vec<Named> {
    let mut out = Vec::new();
    for class in &data.classes {
        let archetypes =
            std::iter::once(None).chain(class.archetypes.iter().map(|a| Some(a.key.clone())));
        for archetype in archetypes {
            let mut build = neutral_human();
            build.class_choices.insert(
                class.key.clone(),
                ClassChoice {
                    archetype: archetype.clone(),
                    ..ClassChoice::default()
                },
            );
            for _ in 0..20 {
                build.levels.push(LevelUp::new(class.key.clone()));
            }
            let archetype = archetype.map(|a| a.to_string()).unwrap_or_default();
            out.push((format!("{} {archetype}", class.key), build));
        }
    }
    out
}

/// Every prestige class for 10 levels after 10 wizard, 10 cleric or 10 witch levels, advancing that class.
pub(super) fn prestige(data: &GameData) -> Vec<Named> {
    let mut out = Vec::new();
    for prestige in &data.prestige_classes {
        for base in ["wizard", "cleric", "witch"] {
            let mut build = neutral_human();
            build.class_choices.insert(
                prestige.key.clone(),
                ClassChoice {
                    archetype: None,
                    advances: [ClassKey::new(base)].into(),
                },
            );
            for _ in 0..10 {
                build.levels.push(LevelUp::new(ClassKey::new(base)));
            }
            for _ in 0..10 {
                build.levels.push(LevelUp::new(prestige.key.clone()));
            }
            out.push((format!("{base} {}", prestige.key), build));
        }
    }
    out
}

/// A wizard 3 with each background (`bg <key>`), and with none (`bg base`). Quirk kept: these name no class
/// choice for the wizard.
pub(super) fn backgrounds(data: &GameData) -> Vec<Named> {
    let mut base = neutral_human();
    base.levels = super::levels(&[("wizard", 3)]);
    let mut out: Vec<Named> = data
        .backgrounds
        .iter()
        .map(|background| {
            let mut build = base.clone();
            build.start.background = Some(background.key.clone());
            (format!("bg {}", background.key), build)
        })
        .collect();
    out.push(("bg base".into(), base));
    out
}

/// The archetypes with an alignment rule, each at 1 and 20 at an alignment it allows (empty and filled at 20) and at
/// 1 at one it doesn't; a plain monk 1 at Chaotic Neutral and at Lawful Neutral; a slayer 1 at Lawful Good; a
/// Drunken Master 4 / rogue 4 at Chaotic Neutral. Humans, but a tiefling Reformed Fiend.
pub(super) fn alignment_rules(engine: &Engine) -> Vec<Named> {
    let make = |class: &str, archetype: Option<&str>, alignment: Alignment, levels: usize| {
        let race = if archetype == Some("bloodrager/reformed-fiend") {
            "tiefling"
        } else {
            "human"
        };
        character(race, alignment, &[(class, archetype, levels)])
    };
    let mut out = Vec::new();
    for (archetype, allowed, not) in [
        ("bloodrager/reformed-fiend", NeutralGood, NeutralEvil),
        ("cleric/angelfire-apostle", NeutralGood, TrueNeutral),
        ("cleric/priest-of-balance", TrueNeutral, LawfulGood),
        ("monk/drunken-master", ChaoticNeutral, LawfulNeutral),
        ("shaman/prophet-of-pestilence", NeutralEvil, LawfulGood),
        ("slayer/bloodseeker", NeutralEvil, TrueNeutral),
        ("warpriest/mantis-zealot", LawfulNeutral, ChaoticEvil),
    ] {
        let class = archetype
            .split('/')
            .next()
            .expect("an archetype key names its class");
        out.push((
            format!("t63 {archetype} {allowed:?} 1"),
            make(class, Some(archetype), allowed, 1),
        ));
        out.push((
            format!("t63 {archetype} {not:?} 1"),
            make(class, Some(archetype), not, 1),
        ));
        out.extend(empty_and_filled(
            engine,
            "t63",
            vec![(
                format!("{archetype} {allowed:?} 20"),
                make(class, Some(archetype), allowed, 20),
            )],
        ));
    }
    out.push((
        "t63 monk ChaoticNeutral 1".into(),
        make("monk", None, ChaoticNeutral, 1),
    ));
    out.push((
        "t63 monk LawfulNeutral 1".into(),
        make("monk", None, LawfulNeutral, 1),
    ));
    out.push((
        "t63 slayer LawfulGood 1".into(),
        make("slayer", None, LawfulGood, 1),
    ));
    let multi = character(
        "human",
        ChaoticNeutral,
        &[("monk", Some("monk/drunken-master"), 4), ("rogue", None, 4)],
    );
    out.push((
        "t63 drunken master 4 rogue 4 / filled".into(),
        crate::generate::with_selections_filled(engine, multi),
    ));
    out
}
