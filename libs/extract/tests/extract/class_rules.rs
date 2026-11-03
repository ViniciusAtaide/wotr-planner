//! Class rules from the hand-fix follow-ups (ticket 28): prestige classes advancing one class, arcane or divine;
//! levels counting as another class's; the druid's Nature Bond; the Crusader's note; how the files write them.

use std::collections::BTreeSet;

use extract::class_rules;
use extract::picks::{Pick, Picks};
use rules::{
    Advancement, Archetype, ArchetypeKey, BabProgression, Class, ClassKey, ClassLevelRow, CountsAs,
    CountsToward, DataAt, FeatKey, FeatList, FeatListKey, FeatSlot, Feature, FeatureKey,
    FeatureSelectionKey, GameData, GuideRef, MythicAbilityKey, Prerequisite, SaveProgression,
    SelectionOptionKey,
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

#[test]
fn a_druid_chooses_an_animal_companion_or_a_domain_at_level_1() {
    let mut picks = Picks::default();
    class_rules::provide(&mut picks);
    let bond = FeatureSelectionKey::new("druid/nature-bond");

    let selections: Vec<_> = picks.selections_of("druid").collect();
    let [selection] = selections[..] else {
        panic!("{selections:?}");
    };
    assert_eq!(selection.key, bond);
    assert_eq!(selection.name, "Nature Bond");
    assert_eq!(
        selection
            .options
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
    // The domain option grants the druid's domain pick, levelled by druid levels.
    assert_eq!(
        selection.option_levels,
        BTreeSet::from([ClassKey::new("druid")])
    );
    assert_eq!(
        selection.options[1].levels[0].selections,
        [FeatureSelectionKey::new("druid/domain")]
    );

    // The first rows of the druid and of the two archetypes that keep both columns pick instead of granting both;
    // the companion at 4 isn't the bond's, and the Primal Druid has its companion and no choice.
    let pick = |table: &str, feature: &str, level: u8| {
        picks
            .pick(table, "", &FeatureKey::new(feature), level)
            .map(|pick| format!("{pick:?}"))
    };
    for table in [
        "druid",
        "druid/defender-of-the-true-world",
        "druid/feyspeaker",
    ] {
        assert_eq!(
            pick(table, &format!("{table}/domain-abilities"), 1),
            Some(format!("{:?}", Pick::Selection(bond.clone()))),
            "{table}"
        );
        assert_eq!(
            pick(table, &format!("{table}/animal-companion"), 1),
            Some(format!("{:?}", Pick::OptionRow)),
            "{table}"
        );
    }
    assert_eq!(pick("druid", "druid/animal-companion", 4), None);
    assert_eq!(
        pick(
            "druid/primal-druid",
            "druid/primal-druid/animal-companion-primal-druid",
            1
        ),
        None
    );

    // A companion option counts as a companion.
    let companion = Prerequisite::SelectionOption(SelectionOptionKey::new(
        "druid/nature-bond/animal-companion",
    ));
    let line = ["Animal Companion".to_owned()];
    for owner in [
        DataAt::Feat(FeatKey::new("boon-companion")),
        DataAt::MythicAbility(MythicAbilityKey::new("mythical-beast")),
    ] {
        assert_eq!(
            picks.alternatives_of(&owner, &line),
            std::slice::from_ref(&companion)
        );
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

    // As `seeds` reads the files: the note is a comment.
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
