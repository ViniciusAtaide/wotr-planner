//! Class rules from the hand-fix follow-ups (ticket 28): prestige classes advancing one class, arcane or divine;
//! levels counting as another class's; the druid's Nature Bond; the Crusader's note; how the files write them.

use std::collections::BTreeSet;

use extract::class_rules;
use rules::{
    Advancement, Archetype, ArchetypeKey, BabProgression, Class, ClassKey, ClassLevelRow, CountsAs,
    CountsToward, Feat, FeatCategory, FeatKey, FeatList, FeatListKey, FeatSlot, Feature,
    FeatureKey, FeatureSelectionKey, GameData, GuideRef, MythicAbility, MythicAbilityGroup,
    MythicAbilityKey, Prerequisite, SaveProgression, SelectionOptionKey,
};

fn row(level: u8) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    }
}

/// A prestige class whose ten rows advance the given spell types.
fn prestige(key: &str, arcane: bool, divine: bool) -> Class {
    let levels = (1..=10)
        .map(|level| ClassLevelRow {
            advances_arcane: arcane,
            advances_divine: divine,
            ..row(level)
        })
        .collect();
    class(key, levels, vec![])
}

fn class(key: &str, levels: Vec<ClassLevelRow>, archetypes: Vec<Archetype>) -> Class {
    Class {
        key: ClassKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        skill_points: 2,
        bab: BabProgression::Half,
        fort: SaveProgression::Poor,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Good,
        save_exceptions: vec![],
        advances: Advancement::Each,
        counts_as: BTreeSet::new(),
        class_skills: BTreeSet::new(),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes,
        spontaneous: false,
    }
}

fn feature(key: &str) -> Feature {
    Feature {
        key: FeatureKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        ability_bonuses: vec![],
        option_row: None,
        daily_choice: false,
        bab_floor: None,
        skill_ranks_at_least_mythic_rank: false,
    }
}

fn features(keys: &[&str]) -> Vec<FeatureKey> {
    keys.iter().map(|key| FeatureKey::new(*key)).collect()
}

fn archetype(key: &str, own: &[&str], first: Vec<FeatureKey>) -> Archetype {
    let mut level_1 = rules::ArchetypeLevelRow {
        level: 1,
        spell_levels: None,
        features: first,
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    };
    level_1.features.sort();
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        prerequisite: None,
        skill_points: None,
        class_skills: None,
        features: own.iter().map(|key| feature(key)).collect(),
        selections: vec![],
        feat_lists: vec![],
        levels: vec![level_1],
        spontaneous: None,
    }
}

fn data(classes: Vec<Class>, prestige_classes: Vec<Class>) -> GameData {
    GameData {
        progression: rules::Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        weapons: vec![],
        feats: vec![],
        classes,
        prestige_classes,
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![],
        backgrounds: vec![],
    }
}

#[test]
fn prestige_classes_advance_one_class_or_count_as_another_as_the_guide_says() {
    let mut data = data(
        vec![],
        vec![
            prestige("eldritch-knight", true, false),
            prestige("hellknight-signifer", true, true),
            prestige("loremaster", true, true),
            prestige("mystic-theurge", true, true),
            prestige("winter-witch", true, true),
        ],
    );
    class_rules::apply_prestige(&mut data.prestige_classes);

    let class = |key: &str| {
        data.prestige_classes
            .iter()
            .find(|c| c.key.as_str() == key)
            .unwrap()
    };
    for key in ["hellknight-signifer", "loremaster", "winter-witch"] {
        assert_eq!(class(key).advances, Advancement::Either, "{key}");
    }
    assert_eq!(class("mystic-theurge").advances, Advancement::Each);
    assert_eq!(class("eldritch-knight").advances, Advancement::Each);
    assert_eq!(
        class("eldritch-knight").counts_as,
        BTreeSet::from([CountsAs {
            class: ClassKey::new("fighter"),
            toward: CountsToward::FeatPrerequisites,
        }])
    );
    let toward_witch = |feature: &str| CountsAs {
        class: ClassKey::new("witch"),
        toward: CountsToward::Feature(FeatureKey::new(feature)),
    };
    assert_eq!(
        class("winter-witch").counts_as,
        BTreeSet::from([
            toward_witch("witch/major-hexes"),
            toward_witch("witch/grand-hexes")
        ])
    );
    assert_eq!(class("loremaster").counts_as, BTreeSet::new());
}

/// A druid in the guide's shape: the companion and the domain on the base row and the two archetypes that keep
/// both columns, a companion only for the Primal Druid.
fn druid_world() -> GameData {
    let mut first = row(1);
    first.features = features(&[
        "druid/animal-companion",
        "druid/domain-abilities",
        "druid/nature-sense",
    ]);
    let mut fourth = row(4);
    fourth.features = features(&["druid/animal-companion"]);
    let mut druid = class("druid", vec![first, row(2), row(3), fourth], vec![]);
    druid.features = ["animal-companion", "domain-abilities", "nature-sense"]
        .map(|key| feature(&format!("druid/{key}")))
        .to_vec();
    for key in ["defender-of-the-true-world", "feyspeaker"] {
        let owner = format!("druid/{key}");
        let companion = format!("{owner}/animal-companion");
        let domain = format!("{owner}/domain-abilities");
        druid.archetypes.push(archetype(
            &owner,
            &[&companion, &domain],
            features(&[&companion, &domain, "druid/nature-sense"]),
        ));
    }
    druid.archetypes.push(archetype(
        "druid/primal-druid",
        &["druid/primal-druid/animal-companion-primal-druid"],
        features(&["druid/primal-druid/animal-companion-primal-druid"]),
    ));
    let companions = Prerequisite::Any(vec![
        Prerequisite::Feature {
            feature: FeatureKey::new("druid/animal-companion"),
            count: 1,
        },
        Prerequisite::Feature {
            feature: FeatureKey::new("hunter/animal-companion"),
            count: 1,
        },
    ]);
    let mut data = data(vec![druid], vec![]);
    data.feats.push(Feat {
        key: FeatKey::new("boon-companion"),
        name: "Boon Companion".into(),
        category: FeatCategory::Class,
        summary: String::new(),
        guide: GuideRef::new("feats#boon-companion"),
        parameter: None,
        repeat: rules::Repeat::Once,
        extra_pick_of: Default::default(),
        prerequisite: Some(companions.clone()),
    });
    data.mythic_abilities.push(MythicAbility {
        key: MythicAbilityKey::new("mythical-beast"),
        name: "Mythical Beast".into(),
        group: MythicAbilityGroup::Class,
        summary: String::new(),
        guide: GuideRef::new("mythic-paths#mythical-beast"),
        prerequisite: Some(companions),
        features: vec![],
        selections: vec![],
    });
    data
}

#[test]
fn a_druid_chooses_an_animal_companion_or_a_domain_at_level_1() {
    let mut data = druid_world();
    class_rules::nature_bond(&mut data);
    let druid = &data.classes[0];

    let bond = &druid.selections[0];
    assert_eq!(bond.key, FeatureSelectionKey::new("druid/nature-bond"));
    assert_eq!(bond.name, "Nature Bond");
    assert_eq!(
        bond.options
            .iter()
            .map(|o| (o.key.as_str(), o.name.as_str(), o.guide.0.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "druid/nature-bond/animal-companion",
                "Animal Companion",
                "druid"
            ),
            ("druid/nature-bond/domain", "Domain", "druid"),
        ]
    );
    // The first row picks instead of granting both; the companion at 4 stays (the Animal domain's, ticket 31).
    let selection = vec![FeatureSelectionKey::new("druid/nature-bond")];
    assert_eq!(druid.levels[0].features, features(&["druid/nature-sense"]));
    assert_eq!(druid.levels[0].selections, selection);
    assert_eq!(
        druid.levels[3].features,
        features(&["druid/animal-companion"])
    );
    assert!(
        !druid
            .features
            .iter()
            .any(|f| f.key.as_str() == "druid/domain-abilities")
    );
    for archetype in &druid.archetypes[..2] {
        let first = &archetype.levels[0];
        assert_eq!(
            first.features,
            features(&["druid/nature-sense"]),
            "{}",
            archetype.key
        );
        assert_eq!(first.selections, selection, "{}", archetype.key);
        assert_eq!(
            archetype.features.len(),
            1,
            "{}: the companion only",
            archetype.key
        );
    }
    // The Primal Druid has its companion and no choice.
    assert_eq!(druid.archetypes[2].levels[0].selections, []);

    // A companion option counts as a companion.
    let companion = Prerequisite::SelectionOption(SelectionOptionKey::new(
        "druid/nature-bond/animal-companion",
    ));
    for prerequisite in [
        data.feats[0].prerequisite.as_ref(),
        data.mythic_abilities[0].prerequisite.as_ref(),
    ] {
        let Some(Prerequisite::Any(children)) = prerequisite else {
            panic!("{prerequisite:?}");
        };
        assert_eq!(children.last(), Some(&companion));
    }
}

#[test]
fn the_files_write_advancement_counts_as_and_the_crusaders_note() {
    let mut loremaster = prestige("loremaster", true, true);
    loremaster.advances = Advancement::Either;
    let mut knight = prestige("eldritch-knight", true, false);
    knight.counts_as = BTreeSet::from([
        CountsAs {
            class: ClassKey::new("fighter"),
            toward: CountsToward::FeatPrerequisites,
        },
        CountsAs {
            class: ClassKey::new("fighter"),
            toward: CountsToward::Feature(FeatureKey::new("fighter/bravery")),
        },
    ]);
    let mut crusader = archetype("cleric/crusader", &[], vec![]);
    crusader.feat_lists.push(FeatList {
        key: FeatListKey::new("cleric/crusader/bonus-feats-1"),
        name: "Crusader Bonus Feats".into(),
        guide: GuideRef::new("cleric#crusader-bonus-feats"),
        ignores_prerequisites: true,
        every_feat: false,
        feats: BTreeSet::from([FeatKey::new("shield-bash"), FeatKey::new("weapon-focus")]),
    });
    crusader.levels[0].feat_slots = vec![FeatSlot::OneOf {
        list: FeatListKey::new("cleric/crusader/bonus-feats-1"),
    }];
    let cleric = class("cleric", vec![row(1)], vec![crusader]);
    let data = data(vec![cleric], vec![knight, loremaster]);

    let files = extract::ron_out::files(&data);
    let file = |name: &str| {
        files
            .iter()
            .find(|(path, _)| path.ends_with(name))
            .map(|(_, text)| text.clone())
            .unwrap_or_else(|| panic!("no {name}"))
    };
    let loremaster = file("prestige/loremaster.ron");
    assert!(
        loremaster.contains("\n    advances: Either,\n"),
        "{loremaster}"
    );
    let knight = file("prestige/eldritch-knight.ron");
    assert!(
        knight.contains(
            "\n    counts_as: [(class: \"fighter\", toward: FeatPrerequisites), (class: \"fighter\", toward: \
             Feature(\"fighter/bravery\"))],\n"
        ),
        "{knight}"
    );
    assert!(!knight.contains("advances:"), "{knight}");
    let cleric = file("classes/cleric.ron");
    assert!(
        cleric.contains(&format!(
            "            {}\n            feat_lists: [\n",
            class_rules::CRUSADER_NOTE
        )),
        "{cleric}"
    );

    // As `data` reads the files: the note is a comment.
    fn read<T: serde::de::DeserializeOwned>(text: &str) -> T {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
            .unwrap()
    }
    assert_eq!(read::<Class>(&loremaster), data.prestige_classes[1]);
    assert_eq!(read::<Class>(&knight), data.prestige_classes[0]);
    assert_eq!(read::<Class>(&cleric), data.classes[0]);
}
