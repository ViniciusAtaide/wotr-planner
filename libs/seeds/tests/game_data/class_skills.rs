//! Options' class skills (ticket 37) and the class-skill cleanup (ticket 38).

use std::collections::{BTreeSet, HashMap};

use rules::{
    Build, CharacterState, Engine, FeatKey, FeatSource, FeatureKey, GameData, Pick, Prerequisite,
    SkillKey, SlotId,
};

use crate::support::*;

// ---- options' class skills (ticket 37) ------------------------------------------------------------------------

/// The class skills at level 1 of a Build of `class` (with `archetype`) picking `option` in its first selection slot.
fn class_skills_at_1(
    class: &str,
    archetype: Option<&str>,
    option: Option<&str>,
) -> BTreeSet<String> {
    let mut build = match archetype {
        Some(archetype) => archetype_build(class, archetype, 1),
        None => build(class, 1),
    };
    if let Some(option) = option {
        build.picks.insert(selection_slot(1), pick(option));
    }
    assert!(
        engine().is_possible(&build),
        "{class} {archetype:?} {option:?}"
    );
    let sheet = engine().evaluate(&build);
    sheet
        .at(1)
        .unwrap()
        .class_skills
        .iter()
        .map(|skill| skill.to_string())
        .collect()
}

fn skill_set(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

#[test]
fn an_oracle_holds_its_mysterys_class_skills_and_one_without_it_doesnt() {
    let oracle = skill_set(&[
        "knowledge-arcana",
        "knowledge-world",
        "lore-nature",
        "lore-religion",
        "persuasion",
    ]);
    assert_eq!(class_skills_at_1("oracle", None, None), oracle);
    for (mystery, skills) in [
        ("battle", &["perception"][..]),
        ("bones", &["stealth"]),
        ("flame", &["athletics", "mobility"]),
        ("nature", &["athletics"]),
        ("stone", &["athletics"]),
        ("waves", &["athletics", "mobility"]),
        ("wind", &["mobility", "stealth"]),
        ("ancestor", &[]),
        ("life", &[]),
    ] {
        let held = class_skills_at_1("oracle", None, Some(&format!("oracle/mystery/{mystery}")));
        let expected: BTreeSet<String> = oracle.iter().cloned().chain(skill_set(skills)).collect();
        assert_eq!(held, expected, "{mystery}");
    }
    // A Divine Herbalist "does not gain additional class skills from his choice of mystery".
    assert_eq!(
        class_skills_at_1(
            "oracle",
            Some("oracle/divine-herbalist"),
            Some("oracle/mystery/flame")
        ),
        oracle
    );
}

#[test]
fn a_kineticist_holds_its_elements_class_skills_and_one_without_it_doesnt() {
    let kineticist = skill_set(&[
        "mobility",
        "perception",
        "persuasion",
        "stealth",
        "use-magic-device",
    ]);
    assert_eq!(class_skills_at_1("kineticist", None, None), kineticist);
    for (element, skills) in [
        ("air", &["lore-nature"][..]),
        ("earth", &["athletics", "knowledge-world"]),
        ("fire", &["lore-nature"]),
        ("water", &["athletics", "lore-nature"]),
    ] {
        let held = class_skills_at_1(
            "kineticist",
            None,
            Some(&format!("kineticist/element/{element}")),
        );
        let expected: BTreeSet<String> = kineticist
            .iter()
            .cloned()
            .chain(skill_set(skills))
            .collect();
        assert_eq!(held, expected, "{element}");
    }
}

#[test]
fn a_cavalier_holds_its_orders_class_skill() {
    let cavalier = skill_set(&["athletics", "mobility", "persuasion"]);
    assert_eq!(class_skills_at_1("cavalier", None, None), cavalier);
    for (order, skill) in [
        ("order-of-the-lion", "knowledge-world"),
        ("order-of-the-shroud", "lore-religion"),
        ("order-of-the-star", "lore-religion"),
        ("order-of-the-sword", "lore-religion"),
    ] {
        let held = class_skills_at_1("cavalier", None, Some(&format!("cavalier/order/{order}")));
        let expected: BTreeSet<String> =
            cavalier.iter().cloned().chain([skill.to_owned()]).collect();
        assert_eq!(held, expected, "{order}");
    }
}

#[test]
fn a_cleric_holds_both_domains_class_skills() {
    let mut cleric = build("cleric", 1);
    cleric
        .picks
        .insert(selection_slot(1), pick("cleric/domain/knowledge"));
    cleric.picks.insert(
        SlotId::Selection { level: 1, index: 1 },
        pick("cleric/domain/trickery"),
    );
    assert!(engine().is_possible(&cleric));
    let sheet = engine().evaluate(&cleric);
    let held: BTreeSet<String> = sheet
        .at(1)
        .unwrap()
        .class_skills
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        held,
        skill_set(&[
            "knowledge-arcana",
            "knowledge-world",
            "lore-nature",
            "lore-religion",
            "persuasion",
            "stealth",
            "trickery",
        ])
    );
}

#[test]
fn a_sorcerer_holds_its_bloodlines_class_skill() {
    let sorcerer = skill_set(&["knowledge-arcana", "persuasion", "use-magic-device"]);
    assert_eq!(class_skills_at_1("sorcerer", None, None), sorcerer);
    for (bloodline, skill) in [
        ("abyssal", "athletics"),
        ("celestial", "lore-religion"),
        ("draconic-red", "perception"),
        ("elemental-fire", "mobility"),
        ("fey", "lore-nature"),
        ("infernal", "knowledge-world"),
        ("serpentine", "stealth"),
        ("undead", "lore-religion"),
    ] {
        let held = class_skills_at_1(
            "sorcerer",
            None,
            Some(&format!("sorcerer/bloodline/{bloodline}")),
        );
        let expected: BTreeSet<String> =
            sorcerer.iter().cloned().chain([skill.to_owned()]).collect();
        assert_eq!(held, expected, "{bloodline}");
    }
}

/// Every option row's class skills, as `option level: skills`.
fn option_row_class_skills() -> Vec<(String, u8, Vec<String>)> {
    let mut rows = Vec::new();
    for class in DATA.classes.iter().chain(&DATA.prestige_classes) {
        let selections = class
            .selections
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.selections));
        for selection in selections {
            for option in &selection.options {
                for row in &option.levels {
                    if !row.class_skills.is_empty() {
                        rows.push((
                            option.key.to_string(),
                            row.level,
                            row.class_skills.iter().map(|s| s.to_string()).collect(),
                        ));
                    }
                }
            }
        }
    }
    rows.sort();
    rows
}

#[test]
fn options_class_skills_are_the_guides_on_their_first_row_and_the_animal_domains_on_its_4th() {
    let rows = option_row_class_skills();
    let mut expected: Vec<(String, u8, Vec<String>)> = Vec::new();
    let mut add = |option: String, skills: &[&str]| {
        // The Animal domain's comes with its companion: "At 4th level, you gain an Animal Companion. … Lore (Nature)
        // becomes a Class Skill."
        let level = if option == "cleric/domain/animal" {
            4
        } else {
            1
        };
        expected.push((
            option,
            level,
            skills.iter().map(|s| (*s).to_owned()).collect(),
        ));
    };
    for (bloodline, skill) in [
        ("abyssal", "athletics"),
        ("celestial", "lore-religion"),
        ("fey", "lore-nature"),
        ("infernal", "knowledge-world"),
        ("serpentine", "stealth"),
        ("undead", "lore-religion"),
    ] {
        add(format!("sorcerer/bloodline/{bloodline}"), &[skill]);
    }
    for colour in [
        "black", "blue", "brass", "bronze", "copper", "gold", "green", "red", "silver", "white",
    ] {
        add(
            format!("sorcerer/bloodline/draconic-{colour}"),
            &["perception"],
        );
    }
    for element in ["air", "earth", "fire", "water"] {
        add(
            format!("sorcerer/bloodline/elemental-{element}"),
            &["mobility"],
        );
    }
    for (mystery, skills) in [
        ("battle", &["perception"][..]),
        ("bones", &["stealth"]),
        ("flame", &["athletics", "mobility"]),
        ("nature", &["athletics"]),
        ("stone", &["athletics"]),
        ("waves", &["athletics", "mobility"]),
        ("wind", &["mobility", "stealth"]),
    ] {
        add(format!("oracle/mystery/{mystery}"), skills);
    }
    for (domain, skills) in [
        ("animal", &["lore-nature"][..]),
        (
            "knowledge",
            &[
                "knowledge-arcana",
                "knowledge-world",
                "lore-nature",
                "lore-religion",
            ],
        ),
        ("trickery", &["stealth", "trickery"]),
    ] {
        add(format!("cleric/domain/{domain}"), skills);
    }
    for (element, skills) in [
        ("air", &["lore-nature"][..]),
        ("earth", &["athletics", "knowledge-world"]),
        ("fire", &["lore-nature"]),
        ("water", &["athletics", "lore-nature"]),
    ] {
        add(format!("kineticist/element/{element}"), skills);
    }
    add(
        "cavalier/order/order-of-the-lion".into(),
        &["knowledge-world"],
    );
    for order in ["shroud", "star", "sword"] {
        add(
            format!("cavalier/order/order-of-the-{order}"),
            &["lore-religion"],
        );
    }
    expected.sort();
    assert_eq!(rows, expected);
}

#[test]
fn no_feature_stands_for_a_fixed_class_skill() {
    let features: Vec<(&str, &str)> = DATA
        .classes
        .iter()
        .chain(&DATA.prestige_classes)
        .flat_map(|class| {
            class
                .features
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.features))
        })
        .map(|feature| (feature.key.as_str(), feature.name.as_str()))
        .filter(|(_, name)| name.starts_with("Class Skill") || *name == "Skills")
        .collect();
    // What's left is a choice of one of three skills (ticket 37's first open question, left as features).
    assert_eq!(
        features,
        [
            ("sorcerer/bloodline/arcane/class-skill", "Class Skill"),
            ("sorcerer/sage-sorcerer/class-skill", "Class Skill"),
        ]
    );
    // The archetypes whose feature went hold the skill through their own list.
    for (class, archetype, skill) in [
        ("monk", "monk/sohei", "lore-nature"),
        ("sorcerer", "sorcerer/empyreal-sorcerer", "lore-religion"),
        ("sorcerer", "sorcerer/sylvan-sorcerer", "lore-nature"),
    ] {
        let sheet = engine().evaluate(&archetype_build(class, archetype, 1));
        assert!(
            sheet
                .at(1)
                .unwrap()
                .class_skills
                .contains(&SkillKey::new(skill)),
            "{archetype}"
        );
    }
}

// ---- class-skill cleanup (ticket 38) --------------------------------------------------------------------------

#[test]
fn an_overwhelming_mage_and_a_seeker_hold_their_bloodlines_class_skill_and_no_class_skill_feature()
{
    let sorcerer = skill_set(&["knowledge-arcana", "persuasion", "use-magic-device"]);
    for archetype in ["sorcerer/overwhelming-mage", "sorcerer/seeker-sorcerer"] {
        for (bloodline, skill) in [
            ("abyssal", "athletics"),
            ("celestial", "lore-religion"),
            ("draconic-gold", "perception"),
            ("elemental-water", "mobility"),
            ("fey", "lore-nature"),
            ("infernal", "knowledge-world"),
            ("serpentine", "stealth"),
            ("undead", "lore-religion"),
        ] {
            let build = with_option(
                archetype_build("sorcerer", archetype, 1),
                1,
                0,
                &format!("sorcerer/bloodline/{bloodline}"),
            );
            assert!(engine().is_possible(&build), "{archetype} {bloodline}");
            let sheet = engine().evaluate(&build);
            let state = sheet.at(1).unwrap();
            let held: BTreeSet<String> = state.class_skills.iter().map(|s| s.to_string()).collect();
            let expected: BTreeSet<String> =
                sorcerer.iter().cloned().chain([skill.to_owned()]).collect();
            assert_eq!(held, expected, "{archetype} {bloodline}");
            let class_skill_features: Vec<&FeatureKey> = state
                .features
                .keys()
                .filter(|feature| feature.as_str().ends_with("/class-skill"))
                .collect();
            assert_eq!(
                class_skill_features,
                [] as [&FeatureKey; 0],
                "{archetype} {bloodline}"
            );
        }
        let defined = DATA
            .classes
            .iter()
            .flat_map(|class| class.archetypes.iter())
            .flat_map(|a| &a.features)
            .any(|feature| feature.key.as_str() == format!("{archetype}/class-skill"));
        assert!(!defined, "{archetype}/class-skill is still defined");
    }
}

#[test]
fn a_sohei_holds_lore_nature_as_a_class_skill_and_a_plain_monk_doesnt() {
    // A lawful neutral monk, plain or Sohei, at 1.
    let monk = |archetype: Option<&str>| {
        let mut monk = match archetype {
            Some(archetype) => archetype_build("monk", archetype, 1),
            None => build("monk", 1),
        };
        monk.start.alignment = Some(rules::Alignment::LawfulNeutral);
        assert!(engine().is_possible(&monk), "{archetype:?}");
        let sheet = engine().evaluate(&monk);
        sheet.at(1).cloned().unwrap()
    };
    let skills = |state: &CharacterState| -> BTreeSet<String> {
        state.class_skills.iter().map(|s| s.to_string()).collect()
    };
    let plain = skill_set(&[
        "athletics",
        "knowledge-world",
        "lore-religion",
        "mobility",
        "perception",
        "persuasion",
        "stealth",
    ]);
    assert_eq!(skills(&monk(None)), plain);
    let sohei = monk(Some("monk/sohei"));
    let expected: BTreeSet<String> = plain
        .iter()
        .cloned()
        .chain(["lore-nature".to_owned()])
        .collect();
    assert_eq!(skills(&sohei), expected);
    assert!(!holds(&sohei, "monk/sohei/skills"));
}

/// The real data with `skill` taken off `class`'s own list, so that the skill an option row gives shows.
fn engine_without_class_skill(class: &str, skill: &str) -> Engine {
    let mut data = DATA.clone();
    let class = data
        .classes
        .iter_mut()
        .find(|c| c.key.as_str() == class)
        .unwrap();
    assert!(class.class_skills.remove(&SkillKey::new(skill)));
    Engine::new(data)
}

/// `engine` gives `skill` as a class skill from `level` on, not before.
fn class_skill_from_level_by(engine: &Engine, build: &Build, level: u8, skill: &str) {
    assert!(engine.is_possible(build));
    let sheet = engine.evaluate(build);
    let has = |level: u8| {
        sheet
            .at(level)
            .unwrap()
            .class_skills
            .contains(&SkillKey::new(skill))
    };
    assert!(!has(level - 1), "{skill} before {level}");
    assert!(has(level), "{skill} at {level}");
}

#[test]
fn the_animal_domains_lore_nature_comes_with_its_companion_at_the_holders_4th_domain_level() {
    let animal = cleric(["animal", "fire"], 4);
    assert!(engine().is_possible(&animal));
    class_skill_from_level(&animal, 4, "lore-nature");
    from_level(&animal, 4, "cleric/domain/animal/animal-companion");
    // A Separatist's second domain runs two levels behind: cleric 6.
    let separatist = with_option(
        with_option(
            archetype_build("cleric", "cleric/separatist", 6),
            1,
            0,
            "cleric/domain/fire",
        ),
        1,
        1,
        "cleric/domain/animal",
    );
    class_skill_from_level(&separatist, 6, "lore-nature");
    // A druid's and an inquisitor's own lists hold Lore (Nature) from level 1; without it there, the domain's shows
    // by their class levels.
    let druid = with_pick(
        with_option(druid(None, 4), 1, 0, "druid/nature-bond/domain"),
        option_slot(1, 0),
        "cleric/domain/animal",
    );
    assert!(engine().is_possible(&druid));
    assert!(has_class_skill(&at(&druid, 1), "lore-nature"));
    class_skill_from_level_by(
        &engine_without_class_skill("druid", "lore-nature"),
        &druid,
        4,
        "lore-nature",
    );
    let inquisitor = with_option(build("inquisitor", 4), 1, 0, "cleric/domain/animal");
    assert!(engine().is_possible(&inquisitor));
    class_skill_from_level_by(
        &engine_without_class_skill("inquisitor", "lore-nature"),
        &inquisitor,
        4,
        "lore-nature",
    );
}

/// Every feature a prerequisite names in the real data, wherever the prerequisite sits.
fn prerequisite_features(data: &GameData) -> BTreeSet<FeatureKey> {
    data_prerequisites(data)
        .into_iter()
        .flat_map(Prerequisite::leaves)
        .filter_map(|leaf| match leaf {
            Prerequisite::Feature { feature, .. } => Some(feature.clone()),
            _ => None,
        })
        .collect()
}

/// A picker's reasons name features by their names: every feature a prerequisite needs has one the sheet knows,
/// so no reason shows a bare key.
#[test]
fn every_feature_a_prerequisite_needs_is_named() {
    let engine = engine();
    let features = prerequisite_features(&DATA);
    assert!(!features.is_empty());
    let unnamed: Vec<&FeatureKey> = features
        .iter()
        .filter(|feature| engine.feature_name(feature) == feature.as_str())
        .collect();
    assert!(unnamed.is_empty(), "unnamed: {unnamed:?}");
}

/// Features that share a name read apart in a reason: each one a prerequisite needs reads its own way, and one whose
/// name another feature shares says whose it is ("Weapon Training (Fighter) or Weapon Training (Sohei)"). Of the
/// alternatives, a reason names those the character's classes give.
#[test]
fn features_sharing_a_name_read_apart_in_a_reason() {
    let engine = engine();
    let mut seen: HashMap<String, &FeatureKey> = HashMap::new();
    let features = prerequisite_features(&DATA);
    for feature in &features {
        let text = engine.prerequisite_text(
            &Prerequisite::Feature {
                feature: feature.clone(),
                count: 1,
            },
            None,
        );
        if let Some(other) = seen.insert(text.clone(), feature) {
            panic!("{other} and {feature} both read {text:?}");
        }
    }
    // The reason names only the alternatives the character's classes give, all of them when none does.
    let reason = |build: &Build| {
        let advanced = engine
            .options(
                build,
                &SlotId::Feat {
                    level: 1,
                    source: FeatSource::General,
                    index: 0,
                },
            )
            .unwrap()
            .into_iter()
            .find(|option| option.pick == Pick::feat(FeatKey::new("advanced-weapon-training")))
            .unwrap();
        engine.issue_text(&advanced.status.unwrap_err()).unwrap()
    };
    assert_eq!(
        reason(&build("fighter", 1)),
        "Needs Weapon Training (Fighter), Fighter level 5 (have 1)"
    );
    assert_eq!(
        reason(&archetype_build("monk", "monk/sohei", 1)),
        "Needs Weapon Training (Sohei), Fighter level 5 (have 0)"
    );
    assert_eq!(
        reason(&build("wizard", 1)),
        "Needs (Weapon Training (Spears / Polearms) or Weapon Training (Fighter) or Weapon Training (Sohei)), \
         Fighter level 5 (have 0)"
    );
}

/// Every slot of a long real Build has a label, and a reason reads as the game says it.
#[test]
fn the_sheet_labels_a_real_build() {
    let engine = engine();
    let build = build("fighter", 20);
    let sheet = engine.evaluate(&build);
    for slot in sheet.levels.iter().flat_map(|level| &level.slots) {
        let label = engine.slot_label(slot);
        assert!(!label.is_empty() && !label.contains('/'), "{label}");
    }
    let cleave = engine
        .options(
            &build,
            &SlotId::Feat {
                level: 1,
                source: FeatSource::General,
                index: 0,
            },
        )
        .unwrap()
        .into_iter()
        .find(|option| option.pick == Pick::feat(FeatKey::new("cleave")))
        .unwrap();
    assert_eq!(
        engine.issue_text(&cleave.status.unwrap_err()).as_deref(),
        Some("Needs Power Attack")
    );
}
