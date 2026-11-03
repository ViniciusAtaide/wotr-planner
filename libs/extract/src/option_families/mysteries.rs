//! The oracle's mysteries and their revelations: the revelations that grant later, the Battle mystery's Maneuver
//! Mastery, and the Dual-Cursed Oracle's revelations.

use std::collections::BTreeSet;

use rules::{
    ClassKey, FeatureKey, FeatureSelection, FeatureSelectionKey, LevelFrom, OptionLevelRow,
    Prerequisite, SelectionOption, SelectionOptionKey,
};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::slug;
use crate::option_levels::{
    FeatNames, empty_row, feature, name_list, option, page, section, selection,
};

use super::{Family, Skills, contains};

/// The Battle mystery's Maneuver Mastery: "Select one type of combat manoeuvre (Bull Rush, Dirty Trick, Disarm,
/// Sunder Armor, Trip) … At 7th level, you gain the corresponding feat. At 11th level, you gain the corresponding
/// Greater... feat." A selection its first row grants, one option per manoeuvre.
const MANEUVER: &str = "oracle/maneuver-mastery";

/// Revelations that grant later, by `mystery/revelation` anchor: each row with the phrase that says so.
enum Later {
    /// Feats granted at a level (at 0: the option's own feats, held when it's picked).
    Feats(u8, &'static [&'static str], &'static str),
    /// Feats granted at a level for the weapon of the pick's chosen feat.
    Chosen(u8, &'static str, &'static str),
    /// A feature at a level.
    Feature(u8, &'static str, &'static str),
}

const REVELATION_ROWS: [(&str, &[Later]); 5] = [
    (
        "battle/war-sight",
        &[Later::Feature(
            7,
            "Uncanny Dodge",
            "At 7th level, you gain Uncanny Dodge",
        )],
    ),
    (
        "battle/weapon-mastery",
        &[
            Later::Chosen(
                8,
                "Improved Critical",
                "At 8th level, you gain Improved Critical",
            ),
            Later::Chosen(
                12,
                "Greater Weapon Focus",
                "At 12th level, you gain Greater Weapon Focus",
            ),
        ],
    ),
    (
        "stone/stone-stability",
        &[
            Later::Feats(5, &["Trip"], "At 5th level, you gain Trip as a bonus feat"),
            Later::Feats(
                10,
                &["Greater Trip"],
                "At 10th level, you gain Greater Trip",
            ),
        ],
    ),
    (
        "waves/fluid-nature",
        &[Later::Feats(
            5,
            &["Dodge"],
            "At 5th level, you gain Dodge as a bonus feat",
        )],
    ),
    (
        "battle/skill-at-arms",
        &[Later::Feats(
            0,
            // By key: the guide's "Heavy Armor Proficiency" is Armor Proficiency (Heavy Armor).
            &[
                "martial-weapons-proficiency",
                "armor-proficiency-heavy-armor",
            ],
            "You gain the Martial Weapons Proficiency and Heavy Armor Proficiency feats",
        )],
    ),
];

/// "Prerequisites: Level 7" (once "Prequisites:"): the class level whose revelation slot may take it.
pub fn revelation_level(text: &str) -> Option<u8> {
    let rest = text
        .split("Prerequisites: Level ")
        .nth(1)
        .or_else(|| text.split("Prequisites: Level ").nth(1))?;
    rest.split_whitespace().next()?.parse().ok()
}

pub(super) fn mysteries(guide: &Guide, skills: &Skills, feats: &FeatNames) -> Family {
    let oracle = page(guide, "oracle");
    let within = section(oracle, "oracle-mysteries");
    assert_eq!(within.children.len(), 9, "the oracle's mysteries");
    let mut family = Family {
        selection: selection(
            "oracle/mystery",
            "Mystery",
            oracle.guide("mystery"),
            &["oracle"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        nested: Vec::new(),
    };
    let mut rows_done = BTreeSet::new();
    for mystery in &within.children {
        let key = format!("oracle/mystery/{}", mystery.anchor);
        let text = mystery.text();
        let mut levels = Vec::new();
        // "The Battle Mystery adds Perception to the list of class skills"
        if let Some(added) = text.split(" adds ").nth(1).and_then(|rest| {
            rest.split(" to the list of class skills")
                .next()
                .filter(|_| text.contains("to the list of class skills"))
        }) {
            // The mystery's class skills are its row 1's while it's held (ticket 37).
            let mut row = empty_row(1);
            for name in name_list(added) {
                row.class_skills.insert(skills.get(&name).0);
            }
            levels.push(row);
        }
        let pool = FeatureSelectionKey::new(format!("oracle/revelations/{}", mystery.anchor));
        let mut revelations = selection(
            pool.as_str(),
            &format!("{} Revelations", mystery.name),
            oracle.guide(&mystery.anchor),
            &[],
        );
        let mut final_revelation = None;
        for revelation in &mystery.children {
            if revelation.name.ends_with("Final Revelation") {
                let feature_key = FeatureKey::new(format!("{key}/{}", revelation.anchor));
                family.features.push(feature(
                    &feature_key,
                    &revelation.name,
                    oracle.guide(&revelation.anchor),
                ));
                final_revelation = Some(feature_key);
                continue;
            }
            let option_key = format!("{pool}/{}", revelation.anchor);
            let mut taken = option(
                &option_key,
                &revelation.name,
                oracle.guide(&revelation.anchor),
                Vec::new(),
            );
            let text = revelation.text();
            if let Some(level) = revelation_level(&text) {
                taken.prerequisite = Some(Prerequisite::Any(vec![
                    Prerequisite::ClassLevel {
                        class: ClassKey::new("oracle"),
                        level,
                    },
                    // The Living Deity's revelations come by arcanist level.
                    Prerequisite::ClassLevel {
                        class: ClassKey::new("arcanist"),
                        level,
                    },
                ]));
            }
            let id = format!("{}/{}", mystery.anchor, revelation.anchor);
            if id == "flame/cinder-dance" {
                // "You cannot select this revelation if you have the Lame curse." (ticket 33)
                contains(
                    &text,
                    "You cannot select this revelation if you have the Lame curse.",
                    &id,
                );
                crate::option_follow_ups::and(
                    &mut taken.prerequisite,
                    Prerequisite::Not(Box::new(Prerequisite::SelectionOption(
                        SelectionOptionKey::new("oracle/oracles-curse/lame"),
                    ))),
                );
            }
            if let Some((_, later)) = REVELATION_ROWS.iter().find(|(of, _)| *of == id) {
                rows_done.insert(id.clone());
                for row in *later {
                    let (Later::Feats(_, _, phrase)
                    | Later::Chosen(_, _, phrase)
                    | Later::Feature(_, _, phrase)) = row;
                    contains(&text, phrase, &id);
                    match row {
                        Later::Feats(0, names, _) => {
                            let names: Vec<String> =
                                names.iter().map(|n| (*n).to_owned()).collect();
                            taken.feats = feats.all(&names, &id);
                        }
                        Later::Feats(level, names, _) => {
                            let names: Vec<String> =
                                names.iter().map(|n| (*n).to_owned()).collect();
                            taken.levels.push(OptionLevelRow {
                                feats: feats.all(&names, &id),
                                ..empty_row(*level)
                            });
                        }
                        Later::Chosen(level, name, _) => {
                            taken.levels.push(OptionLevelRow {
                                chosen_feats: feats.all(&[(*name).to_owned()], &id),
                                ..empty_row(*level)
                            });
                        }
                        Later::Feature(level, name, _) => {
                            let feature_key =
                                FeatureKey::new(format!("{option_key}/{}", slug(name)));
                            family.features.push(feature(
                                &feature_key,
                                name,
                                oracle.guide(&revelation.anchor),
                            ));
                            taken.levels.push(OptionLevelRow {
                                features: vec![feature_key],
                                ..empty_row(*level)
                            });
                        }
                    }
                }
            }
            if id == "battle/weapon-mastery" {
                contains(&text, "You gain Weapon Focus as a bonus feat", &id);
                taken.chosen_feat = feats.get("Weapon Focus");
            }
            if id == "battle/maneuver-mastery" {
                rows_done.insert(id.clone());
                family
                    .nested
                    .push(maneuver_mastery(oracle, revelation, feats));
                taken.levels.push(OptionLevelRow {
                    selections: vec![FeatureSelectionKey::new(MANEUVER)],
                    ..empty_row(1)
                });
            }
            revelations.options.push(taken);
        }
        assert!(
            revelations.options.len() >= 5,
            "{}: revelations",
            mystery.anchor
        );
        let final_revelation =
            final_revelation.unwrap_or_else(|| panic!("{}: no Final Revelation", mystery.anchor));
        levels.push(OptionLevelRow {
            features: vec![final_revelation],
            ..empty_row(20)
        });
        family.selection.options.push(SelectionOption {
            pool: Some(pool),
            ..option(&key, &mystery.name, oracle.guide(&mystery.anchor), levels)
        });
        family.nested.push(revelations);
    }
    assert_eq!(rows_done.len(), REVELATION_ROWS.len() + 1, "{rows_done:?}");
    // The pool selection the class levels grant: the held mysteries' revelations, rows by oracle level.
    family.nested.push(FeatureSelection {
        pool_of: Some(family.selection.key.clone()),
        ..selection(
            "oracle/revelation",
            "Revelation",
            oracle.guide("revelation"),
            &["oracle"],
        )
    });
    family
}

fn maneuver_mastery(oracle: &Page, revelation: &Section, feats: &FeatNames) -> FeatureSelection {
    let text = revelation.text();
    contains(
        &text,
        "At 7th level, you gain the corresponding feat",
        MANEUVER,
    );
    contains(
        &text,
        "At 11th level, you gain the corresponding Greater... feat",
        MANEUVER,
    );
    let names = text
        .split("Select one type of combat manoeuvre (")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("the manoeuvres");
    // Its maneuver follows its revelation's level, an oracle's or a Living Deity's (ticket 33).
    let mut nested = FeatureSelection {
        level_from: Some(LevelFrom::GrantingOption),
        ..selection(
            MANEUVER,
            "Maneuver Mastery",
            oracle.guide(&revelation.anchor),
            &[],
        )
    };
    for name in name_list(names) {
        let feat = feats.all(std::slice::from_ref(&name), MANEUVER);
        let greater = feats.all(&[format!("Greater {name}")], MANEUVER);
        nested.options.push(option(
            &format!("{MANEUVER}/{}", slug(&name)),
            &name,
            oracle.guide(&revelation.anchor),
            vec![
                OptionLevelRow {
                    feats: feat,
                    ..empty_row(7)
                },
                OptionLevelRow {
                    feats: greater,
                    ..empty_row(11)
                },
            ],
        ));
    }
    assert_eq!(nested.options.len(), 5);
    nested
}

/// "You can choose from the Ancestor, Life, Nature, Stone or Waves mysteries." (Lone Strider)
pub(super) const LONE_STRIDER_MYSTERIES: [&str; 5] =
    ["ancestor", "life", "nature", "stone", "waves"];

pub(super) const MYSTERY: &str = "oracle/mystery";
pub(super) const REVELATION: &str = "oracle/revelation";
pub(super) const SECOND_MYSTERY: &str = "oracle/second-mystery";
pub(super) const DUAL_CURSED: &str = "oracle/dual-cursed-oracle";
pub(super) const DUAL_CURSED_REVELATION: &str = "oracle/dual-cursed-oracle/revelation";

/// The Dual-Cursed Oracle's revelation pick: the held mystery's pool, plus Misfortune and Fortune.
pub(super) fn dual_cursed_revelations(guide: &Guide) -> FeatureSelection {
    let oracle = page(guide, "oracle");
    contains(
        &section(oracle, "additional-revelations").text(),
        "he can choose two Revelations unique to the archetype",
        "oracle#additional-revelations",
    );
    let own = ["Misfortune", "Fortune"].map(|name| {
        let anchor = name.to_lowercase();
        // The sections exist.
        contains(&section(oracle, &anchor).text(), "this revelation", &anchor);
        option(
            &format!("{DUAL_CURSED_REVELATION}/{anchor}"),
            name,
            format!("oracle#{anchor}"),
            Vec::new(),
        )
    });
    FeatureSelection {
        options: own.into(),
        pool_of: Some(FeatureSelectionKey::new(MYSTERY)),
        ..selection(
            DUAL_CURSED_REVELATION,
            "Revelation (Dual-Cursed Oracle)",
            "oracle#additional-revelations".into(),
            &["oracle"],
        )
    }
}
