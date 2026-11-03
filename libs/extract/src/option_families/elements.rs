//! The kineticist's elements: class skills, the wild-talent bonus feat list, the defensive talent, Expanded Element,
//! the Blood Kineticist's and the Kinetic Knight's, and their check.

use std::collections::BTreeSet;

use rules::{
    FeatListKey, Feature, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData,
    OptionLevelRow, OptionRow, Repeat, SelectionOption,
};

use crate::guide::Guide;
use crate::names::slug;
use crate::option_levels::{
    FeatNames, empty_row, feature, list, name_list, option, page, section, selection,
};
use crate::picks::{Pick, Picks, Rule};

use super::{Family, Skills, all_text, contains, sharing};

/// The guide's spellings that aren't a feat's name: "Lighting Reflexes".
pub fn element_feat(name: &str) -> String {
    name.replace("Lighting Reflexes", "Lightning Reflexes")
}

/// The kineticist's tables: the class's and each archetype's, each with an "Elemental Focus" choice at 1.
const KINETICIST_TABLES: [&str; 8] = [
    "kineticist",
    "kineticist/blood-kineticist",
    "kineticist/dark-elementalist",
    "kineticist/elemental-engine",
    "kineticist/kinetic-knight",
    "kineticist/kinetic-sharpshooter",
    "kineticist/overwhelming-soul",
    "kineticist/psychokineticist",
];
const ELEMENT: &str = "kineticist/element";
const EXPANDED_ELEMENT: &str = "kineticist/expanded-element";
const BLOOD_ELEMENT: &str = "kineticist/blood-kineticist/element";
const KNIGHT_ELEMENT: &str = "kineticist/kinetic-knight/element";
const KNIGHT_TALENT: &str = "kineticist/kinetic-knight/defensive-wild-talent";

/// The kineticist's elements, built with the class: the selection and its options' rows, the features and lists the
/// rows name, Expanded Element and the archetypes' own selections. Each table's "Class Skills" cell at 1 is the
/// table's element pick, its "Defensive Wild Talent" at 2 the element's row, and "Expanded Element" at 7 and 15 a
/// pick of another element.
pub(super) fn elements(guide: &Guide, skills: &Skills, feats: &FeatNames, picks: &mut Picks) {
    let kineticist_page = page(guide, "kineticist");
    let within = section(kineticist_page, "elemental-focus");
    let bonus = all_text(section(kineticist_page, "kineticist-bonus-feat"));
    contains(
        &bonus,
        "You do not need to meet the normal prerequisites for the feat you select",
        "kineticist#kineticist-bonus-feat",
    );
    contains(
        &section(kineticist_page, "elemental-defense").text(),
        "At 2nd level, the Kineticist unlocks the Defensive Wild Talent corresponding to her Elemental Focus",
        "kineticist#elemental-defense",
    );
    let expanded = section(kineticist_page, "expanded-element").text();
    contains(
        &expanded,
        "At 7th level, the Kineticist may choose a second Elemental Focus or increase her primary focus",
        "kineticist#expanded-element",
    );
    // The 15th-level element isn't the 7th-level one (ticket 33).
    contains(
        &expanded,
        "the one she chose at 1st level, but only if she didn't choose it at 7th",
        "kineticist#expanded-element",
    );
    let mut family = Family {
        selection: selection(
            ELEMENT,
            "Elemental Focus",
            kineticist_page.guide("elemental-focus"),
            &["kineticist"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        nested: Vec::new(),
    };
    assert_eq!(within.children.len(), 4, "the four elements");
    for element in &within.children {
        let short = element.anchor.trim_start_matches("elemental-focus-");
        let key = format!("{ELEMENT}/{short}");
        let line = |label: &str| -> String {
            element
                .list_items()
                .find_map(|line| line.strip_prefix(label))
                .unwrap_or_else(|| panic!("{}: no {label:?}", element.anchor))
                .trim()
                .trim_end_matches('.')
                .to_owned()
        };
        // The element's class skills are its row 1's while it's held (ticket 37).
        let mut first = empty_row(1);
        for name in name_list(&line("Additional class skills:")) {
            first.class_skills.insert(skills.get(&name).0);
        }
        let names: Vec<String> = line("Wild Talent bonus feats:")
            .split(", ")
            .map(element_feat)
            .collect();
        let list_key = format!("{key}/wild-talent-feats");
        family.lists.push(list(
            &list_key,
            &format!(
                "Wild Talent Bonus Feats ({})",
                element.name.trim_start_matches("Elemental Focus - ")
            ),
            kineticist_page.guide(&element.anchor),
            true,
            feats.all(&names, &list_key),
        ));
        first.feat_list = Some(FeatListKey::new(&list_key));
        let talent = line("Defensive Wild Talent:");
        let talent_section = section(kineticist_page, &slug(&talent));
        contains(&all_text(talent_section), "Defensive", &talent);
        let talent_key = FeatureKey::new(format!("{key}/{}", slug(&talent)));
        family.features.push(feature(
            &talent_key,
            &talent,
            kineticist_page.guide(&talent_section.anchor),
        ));
        let second = OptionLevelRow {
            features: vec![talent_key],
            ..empty_row(2)
        };
        family.selection.options.push(SelectionOption {
            // "choose a second Elemental Focus or increase her primary focus": held twice at most.
            repeat: Repeat::Times(2),
            ..option(
                &key,
                &element.name,
                kineticist_page.guide(&element.anchor),
                vec![first, second],
            )
        });
    }

    for feature in &family.features {
        picks.feature("kineticist", feature.clone());
    }
    for list in &family.lists {
        picks.feat_list("kineticist", list.clone());
    }
    // The guide says only that the new element brings its blasts: like Crossblooded's second bloodline, the pick
    // holds the element and gives its feat list, not its class skills or defensive talent.
    picks.selection(
        "kineticist",
        FeatureSelection {
            feat_lists_only: true,
            once_per_option: true,
            ..sharing(
                EXPANDED_ELEMENT,
                "Expanded Element",
                kineticist_page.guide("expanded-element"),
                &["kineticist"],
                family.all_keys(),
            )
        },
    );
    // "Must select Water at 1st and 7th levels" (Blood Kineticist: Water granted with no pick; its 7th level is
    // Blood Blast). The Kinetic Knight's Defensive Wild Talent comes at 4th, not 2nd: its own selection skips row 2
    // and its 4th-level feature grants that row.
    picks.selection(
        "kineticist/blood-kineticist",
        sharing(
            BLOOD_ELEMENT,
            "Elemental Focus (Blood Kineticist)",
            "kineticist#blood-kineticist".to_owned(),
            &["kineticist"],
            family.keys(&["water"]),
        ),
    );
    picks.selection(
        "kineticist/kinetic-knight",
        FeatureSelection {
            skipped_levels: BTreeSet::from([2]),
            ..sharing(
                KNIGHT_ELEMENT,
                "Elemental Focus (Kinetic Knight)",
                "kineticist#kinetic-knight".to_owned(),
                &["kineticist"],
                family.all_keys(),
            )
        },
    );
    picks.rule(Rule {
        feature: FeatureKey::new(KNIGHT_TALENT),
        table: Some("kineticist/kinetic-knight".to_owned()),
        cell: None,
        levels: Some(vec![4]),
        pick: Pick::Feature(Feature {
            option_row: Some(OptionRow {
                selection: FeatureSelectionKey::new(KNIGHT_ELEMENT),
                level: 2,
            }),
            ..feature(
                &FeatureKey::new(KNIGHT_TALENT),
                "Defensive Wild Talent",
                "kineticist#kinetic-knight".to_owned(),
            )
        }),
    });
    for table in KINETICIST_TABLES {
        let pick = match table {
            "kineticist/blood-kineticist" => BLOOD_ELEMENT,
            "kineticist/kinetic-knight" => KNIGHT_ELEMENT,
            _ => ELEMENT,
        };
        picks.rule(Rule {
            feature: FeatureKey::new(format!("{table}/class-skills")),
            table: Some(table.to_owned()),
            cell: None,
            levels: Some(vec![1]),
            pick: Pick::Selection(FeatureSelectionKey::new(pick)),
        });
        picks.rule(Rule {
            feature: FeatureKey::new(format!("{table}/defensive-wild-talent")),
            table: Some(table.to_owned()),
            cell: None,
            levels: Some(vec![2]),
            pick: Pick::OptionRow,
        });
    }
    picks.rule(Rule {
        feature: FeatureKey::new(EXPANDED_ELEMENT),
        table: None,
        cell: None,
        levels: Some(vec![7, 15]),
        pick: Pick::Selection(FeatureSelectionKey::new(EXPANDED_ELEMENT)),
    });
    picks.selection("kineticist", family.selection);
}

/// The kineticist's rows in force hold the element picks where the class's and the archetypes' tables have the
/// cells: every table's choice at 1, Expanded Element at 7 and 15 (the Blood Kineticist's 7th is Blood Blast), and
/// the Kinetic Knight's defensive talent at 4 only.
pub(super) fn check_elements(data: &GameData) {
    let kineticist = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "kineticist")
        .expect("the kineticist");
    let archetypes: BTreeSet<&str> = kineticist
        .archetypes
        .iter()
        .map(|a| a.key.as_str())
        .collect();
    assert_eq!(
        archetypes,
        KINETICIST_TABLES[1..].iter().copied().collect(),
        "the kineticist's archetypes"
    );
    let in_force = |archetype: Option<&str>, level: u8| -> (Vec<String>, Vec<String>) {
        let own = archetype.and_then(|key| {
            kineticist
                .archetypes
                .iter()
                .find(|a| a.key.as_str() == key)
                .and_then(|a| a.levels.iter().find(|row| row.level == level))
        });
        let (features, selections) = match own {
            Some(row) => (&row.features, &row.selections),
            None => {
                let row = kineticist
                    .levels
                    .iter()
                    .find(|row| row.level == level)
                    .expect("a kineticist row");
                (&row.features, &row.selections)
            }
        };
        (
            features.iter().map(|f| f.to_string()).collect(),
            selections.iter().map(|s| s.to_string()).collect(),
        )
    };
    for table in KINETICIST_TABLES {
        let archetype = (table != "kineticist").then_some(table);
        let pick = match table {
            "kineticist/blood-kineticist" => BLOOD_ELEMENT,
            "kineticist/kinetic-knight" => KNIGHT_ELEMENT,
            _ => ELEMENT,
        };
        assert!(
            in_force(archetype, 1).1.iter().any(|s| s == pick),
            "{table}: the element pick at 1"
        );
        let expanded_at: &[u8] = if table == "kineticist/blood-kineticist" {
            &[15]
        } else {
            &[7, 15]
        };
        for &level in expanded_at {
            assert!(
                in_force(archetype, level)
                    .1
                    .iter()
                    .any(|s| s == EXPANDED_ELEMENT),
                "{table}: Expanded Element at {level}"
            );
        }
        for level in 1..=20 {
            assert!(
                level == 4 && table == "kineticist/kinetic-knight"
                    || !in_force(archetype, level)
                        .0
                        .iter()
                        .any(|f| f == KNIGHT_TALENT),
                "the Kinetic Knight's talent at 4 only"
            );
        }
    }
}
