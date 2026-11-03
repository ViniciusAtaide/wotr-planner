//! The families given in prose: the shaman's spirits and the ranger's combat styles.

use std::collections::{BTreeMap, BTreeSet};

use rules::{FeatListKey, FeatureKey, OptionLevelRow};

use crate::guide::Guide;
use crate::names::slug;

use super::reading::{FeatNames, name_list, page, section};
use super::rows::{empty_row, feature, list, option, selection};
use super::{Family, Owner};

/// "A Shaman's choice of Spirit grants him abilities": under each spirit's heading, its ability (1st level), then
/// the ones from 8th and 16th level, and its Manifestation (20th), each stating its level.
pub(super) fn shaman_spirits(guide: &Guide) -> Family {
    let shaman = page(guide, "shaman");
    let spirits = section(shaman, "spirits");
    let mut family = Family {
        owner: Owner::Class("shaman"),
        selection: selection(
            "shaman/spirit",
            "Spirit",
            shaman.guide("spirit"),
            &["shaman"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for spirit in &spirits.children {
        let key = format!("shaman/spirit/{}", spirit.anchor);
        let mut rows: Vec<OptionLevelRow> = Vec::new();
        for (index, ability) in spirit
            .children
            .iter()
            .filter(|child| !child.name.ends_with("Bonus Spells"))
            .enumerate()
        {
            let text = ability.text();
            // "At 20th level, a Shaman fully unlocks the power of his guardian spirit" (Manifestation).
            let level = if index == 0 {
                1
            } else if ability.name.starts_with("Manifestation") {
                20
            } else {
                [8u8, 16, 20]
                    .into_iter()
                    .find(|level| text.starts_with(&format!("At {level}th level")))
                    .unwrap_or_else(|| panic!("{}: no level in {text:?}", ability.anchor))
            };
            let name = ability.name.clone();
            let feature_key = FeatureKey::new(format!("{key}/{}", slug(&name)));
            family
                .features
                .push(feature(&feature_key, &name, shaman.guide(&ability.anchor)));
            rows.push(OptionLevelRow {
                features: vec![feature_key],
                ..empty_row(level)
            });
        }
        let levels: Vec<u8> = rows.iter().map(|row| row.level).collect();
        assert_eq!(levels, [1, 8, 16, 20], "{}", spirit.anchor);
        family.selection.options.push(option(
            &key,
            &spirit.name,
            shaman.guide(&spirit.anchor),
            rows,
        ));
    }
    assert_eq!(family.selection.options.len(), 9);
    family
}

/// "At 2nd level, the Ranger may choose from A, B and C. At 6th level, D and E are added. At 10th level, F is
/// added." Each style's list grows at 2, 6 and 10 with the style's feats and, as "Besides the style feats, you can
/// also select from Dodge, Great Fortitude, Iron Will, Lightning Reflexes and Toughness. From 10th level, you can
/// select a Skill Focus", those too. A style feat ignores prerequisites ("does not need to meet the
/// prerequisites").
pub(super) fn ranger_styles(guide: &Guide, feats: &FeatNames) -> Family {
    let ranger = page(guide, "ranger");
    let styles = section(ranger, "combat-style-feat");
    let intro = styles.text();
    assert!(intro.contains("does not need to meet the prerequisites"));
    let besides = intro
        .split("you can also select from ")
        .nth(1)
        .and_then(|rest| rest.split(". ").next())
        .expect("the ranger's extra feats");
    let extras = feats.all(&name_list(besides), "the ranger's extra feats");
    assert!(intro.contains("From 10th level, you can select a Skill Focus"));
    let skill_focus = feats.get("Skill Focus").expect("Skill Focus");
    let mut family = Family {
        owner: Owner::Class("ranger"),
        selection: selection(
            "ranger/combat-style-feat",
            "Combat Style Feat",
            ranger.guide("combat-style-feat"),
            &["ranger"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        ties: BTreeMap::new(),
        nested: Vec::new(),
    };
    for style in &styles.children {
        let key = format!("ranger/combat-style-feat/{}", style.anchor);
        let mut held = BTreeSet::new();
        let mut rows = Vec::new();
        for sentence in style.text().split(". ") {
            let Some(rest) = sentence.trim().strip_prefix("At ") else {
                continue;
            };
            let (level, rest) = rest
                .split_once("th level, ")
                .or_else(|| rest.split_once("nd level, "))
                .expect(sentence);
            let level: u8 = level.parse().expect(sentence);
            let names = if let Some(names) = rest.split(" from ").nth(1) {
                names
            } else {
                rest.trim_end_matches('.')
                    .trim_end_matches(" are added")
                    .trim_end_matches(" is added")
            };
            held.extend(feats.all(&name_list(names), &key));
            let mut feats_now = held.clone();
            feats_now.extend(extras.iter().cloned());
            if level >= 10 {
                feats_now.insert(skill_focus.clone());
            }
            let list_key = format!("{key}-{level}");
            family.lists.push(list(
                &list_key,
                &format!("{} (from level {level})", style.name),
                ranger.guide(&style.anchor),
                true,
                feats_now,
            ));
            rows.push(OptionLevelRow {
                feat_list: Some(FeatListKey::new(&list_key)),
                ..empty_row(level)
            });
        }
        let levels: Vec<u8> = rows.iter().map(|row| row.level).collect();
        assert_eq!(levels, [2, 6, 10], "{}", style.anchor);
        family
            .selection
            .options
            .push(option(&key, &style.name, ranger.guide(&style.anchor), rows));
    }
    family
}
