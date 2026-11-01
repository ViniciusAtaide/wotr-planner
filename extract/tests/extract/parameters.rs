//! Parameterised and repeatable feats and options (ticket 27): the weapons from the guide's proficiency lists, what
//! a feat is taken for and how often, the prerequisites that name a value or the pick's own, the options that
//! repeat or take a value, the bonus feats of the player's choosing, and how the files write them.

use std::collections::BTreeSet;

use extract::guide::Guide;
use extract::html::parse_page;
use extract::parameters;
use rules::{
    Archetype, ArchetypeKey, ArchetypeLevelRow, BabProgression, Class, ClassKey, ClassLevelRow,
    DataAt, Feat, FeatCategory, FeatKey, FeatListKey, FeatSlot, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, GuideRef, Parameter, ParameterKind,
    Prerequisite, Repeat, SaveProgression, SelectionOption, SelectionOptionKey, SkillKey,
    SpellSchool, Weapon, WeaponKey, WeaponProficiency,
};

/// A feats page in the guide's style, with short weapon lists.
const FEATS_PAGE: &str = r#"<html><head><link rel="canonical" href="https://gamefaqs.gamespot.com/x/faqs/80843/feats"></head>
<body><div id="faqwrap">
<h3><a id="weapon-feats"></a>Weapon Feats</h3>
<h4><a id="simple-weapon-proficiency"></a>Simple Weapon Proficiency</h4>
<p>Become proficient in all of the following weapons:<br>Club, Dagger, Light Crossbow.<br>All classes apart from Druids receive this feat automatically.</p>
<h4><a id="martial-weapons-proficiency"></a>Martial Weapons Proficiency</h4>
<p>Become proficient in all of the following weapons:<br>Hand Axe, Long Bow.</p>
<h4><a id="exotic-weapon-proficiency"></a>Exotic Weapon Proficiency</h4>
<p>Prerequisites:<br>Base Attack Bonus: 1</p>
<p>Become proficient in one of the following weapons:<br>Bastard Sword, Two-bladed Sword.</p>
</div></body></html>"#;

fn guide() -> Guide {
    let page = parse_page("feats", FEATS_PAGE).unwrap();
    Guide {
        pages: [("feats".to_owned(), page)].into(),
        saved_at: "2026-10-31".into(),
    }
}

fn feat_key(key: &str) -> FeatKey {
    FeatKey::new(key)
}

fn weapon(key: &str) -> Parameter {
    Parameter::Weapon(WeaponKey::new(key))
}

fn skill(key: &str) -> Parameter {
    Parameter::Skill(SkillKey::new(key))
}

fn feat_for(feat: &str, value: Parameter) -> Prerequisite {
    Prerequisite::FeatFor {
        feat: feat_key(feat),
        value,
    }
}

#[test]
fn the_weapons_are_the_proficiency_lists_then_what_weapon_focus_names() {
    let weapons: Vec<(String, String, Option<WeaponProficiency>)> = parameters::weapons(&guide())
        .into_iter()
        .map(|w| (w.key.to_string(), w.name, w.proficiency))
        .collect();
    use WeaponProficiency::*;
    let expected: Vec<(&str, &str, Option<WeaponProficiency>)> = vec![
        ("club", "Club", Some(Simple)),
        ("dagger", "Dagger", Some(Simple)),
        ("light-crossbow", "Light Crossbow", Some(Simple)),
        ("hand-axe", "Hand Axe", Some(Martial)),
        ("long-bow", "Long Bow", Some(Martial)),
        ("bastard-sword", "Bastard Sword", Some(Exotic)),
        ("two-bladed-sword", "Two-bladed Sword", Some(Exotic)),
        ("sawtooth-saber", "Sawtooth Saber", None),
        ("unarmed-strike", "Unarmed Strike", None),
        ("natural-weapons", "Natural Weapons", None),
        ("touch-attacks", "Touch Attacks", None),
        ("rays", "Rays", None),
    ];
    assert_eq!(
        weapons,
        expected
            .into_iter()
            .map(|(key, name, proficiency)| (key.to_owned(), name.to_owned(), proficiency))
            .collect::<Vec<_>>()
    );
}

#[test]
fn feats_take_values_and_repeat_as_the_guide_says() {
    use ParameterKind as K;
    let shape = |key: &str| parameters::feat_shape(&feat_key(key));
    assert_eq!(
        shape("weapon-focus"),
        Some((Some(K::Weapon), Repeat::PerValue))
    );
    assert_eq!(
        shape("skill-focus"),
        Some((Some(K::Skill), Repeat::PerValue))
    );
    assert_eq!(
        shape("exotic-weapon-proficiency"),
        Some((Some(K::ExoticWeapon), Repeat::Once))
    );
    assert_eq!(
        shape("improved-critical"),
        Some((Some(K::Weapon), Repeat::Once))
    );
    assert_eq!(
        shape("spell-focus"),
        Some((Some(K::SpellSchool), Repeat::Once))
    );
    assert_eq!(
        shape("elemental-focus"),
        Some((Some(K::Energy), Repeat::Once))
    );
    assert_eq!(
        shape("armor-focus"),
        Some((Some(K::ArmourType), Repeat::Once))
    );
    assert_eq!(shape("extra-hex"), Some((None, Repeat::Times(3))));
    assert_eq!(shape("extra-ki"), Some((None, Repeat::Unlimited)));
    assert_eq!(shape("magical-tail"), Some((None, Repeat::Times(8))));
    // The guide doesn't say Extra Bane Weapon repeats, nor gives Toughness a value.
    assert_eq!(shape("extra-bane-weapon"), None);
    assert_eq!(shape("toughness"), None);
}

#[test]
fn prerequisites_name_the_guides_values() {
    let feat = |key: &str| DataAt::Feat(feat_key(key));
    let rewrite = |at: DataAt, mut tree: Prerequisite| {
        let n = parameters::rewrite_prerequisite(&at, &mut tree);
        tree.canonicalize();
        (n, tree)
    };
    let fighter = |level| Prerequisite::ClassLevel {
        class: ClassKey::new("fighter"),
        level,
    };

    // "Choose a single weapon for which you have already selected Weapon Focus".
    assert_eq!(
        rewrite(
            feat("weapon-specialization"),
            Prerequisite::All(vec![
                Prerequisite::Feat(feat_key("weapon-focus")),
                fighter(4)
            ])
        ),
        (
            1,
            Prerequisite::All(vec![
                Prerequisite::FeatForSameValue(feat_key("weapon-focus")),
                fighter(4)
            ])
        )
    );
    // "Weapon Focus (One-Handed Piercing)" ... "(i.e. Dagger, Estoc, Punching Dagger, Rapier, Sai, Starknife)".
    let (n, fencing) = rewrite(
        feat("fencing-grace"),
        Prerequisite::All(vec![
            Prerequisite::Feat(feat_key("weapon-finesse")),
            Prerequisite::Feat(feat_key("weapon-focus")),
        ]),
    );
    assert_eq!(n, 1);
    let mut expected = Prerequisite::All(vec![
        Prerequisite::Feat(feat_key("weapon-finesse")),
        Prerequisite::Any(
            [
                "dagger",
                "estoc",
                "punching-dagger",
                "rapier",
                "sai",
                "starknife",
            ]
            .map(|w| feat_for("weapon-focus", weapon(w)))
            .to_vec(),
        ),
    ]);
    expected.canonicalize();
    assert_eq!(fencing, expected);
    // "Skill Focus - Mobility or Slayer Proficiencies".
    let slayer = Prerequisite::Feature {
        feature: FeatureKey::new("slayer/slayer-proficiencies"),
        count: 1,
    };
    assert_eq!(
        rewrite(
            feat("slayers-feint"),
            Prerequisite::Any(vec![
                Prerequisite::Feat(feat_key("skill-focus")),
                slayer.clone()
            ])
        ),
        (
            1,
            Prerequisite::Any(vec![feat_for("skill-focus", skill("mobility")), slayer])
        )
    );
    // "Spell Focus (Conjuration)".
    assert_eq!(
        rewrite(
            feat("augment-summoning"),
            Prerequisite::Feat(feat_key("spell-focus"))
        ),
        (
            1,
            feat_for(
                "spell-focus",
                Parameter::SpellSchool(SpellSchool::Conjuration)
            )
        )
    );
    // A prestige class's: "Skill Focus (Knowledge: Arcana or Knowledge: World)".
    assert_eq!(
        rewrite(
            DataAt::Class(ClassKey::new("loremaster")),
            Prerequisite::Feat(feat_key("skill-focus"))
        ),
        (
            1,
            Prerequisite::Any(vec![
                feat_for("skill-focus", skill("knowledge-arcana")),
                feat_for("skill-focus", skill("knowledge-world")),
            ])
        )
    );
    // Others are left alone: "Weapon Focus" on its own is any weapon.
    assert_eq!(
        rewrite(
            feat("dazzling-display"),
            Prerequisite::Feat(feat_key("weapon-focus"))
        ),
        (0, Prerequisite::Feat(feat_key("weapon-focus")))
    );
}

#[test]
fn options_repeat_or_take_a_value_as_the_guide_says() {
    let option = |key: &str| SelectionOptionKey::new(key);
    assert_eq!(
        parameters::option_repeat(&option("barbarian/rage-power/increased-damage-reduction")),
        Some(Repeat::Times(3))
    );
    assert_eq!(
        parameters::option_repeat(&option(
            "stalwart-defender/defensive-powers/increased-damage-reduction"
        )),
        Some(Repeat::Times(2))
    );
    assert_eq!(
        parameters::option_repeat(&option(
            "shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-new-strain"
        )),
        Some(Repeat::Unlimited)
    );
    assert_eq!(
        parameters::option_repeat(&option("rogue/rogue-talents/fast-stealth")),
        None
    );
    assert_eq!(
        parameters::chosen_feat(&option("rogue/rogue-talents/weapon-focus")),
        Some(feat_key("weapon-focus"))
    );
    assert_eq!(
        parameters::chosen_feat(&option("rogue/rogue-talents/fast-stealth")),
        None
    );
}

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

fn archetype_row(level: u8) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level,
        spell_levels: None,
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: vec![],
        selections: vec![],
        feats_for: Default::default(),
    }
}

fn class(key: &str, levels: Vec<ClassLevelRow>, archetypes: Vec<Archetype>) -> Class {
    Class {
        key: ClassKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        skill_points: 2,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Poor,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: rules::Advancement::Each,
        counts_as: std::collections::BTreeSet::new(),
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

fn archetype(key: &str, features: Vec<Feature>, levels: Vec<ArchetypeLevelRow>) -> Archetype {
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        prerequisite: None,
        skill_points: None,
        class_skills: None,
        features,
        selections: vec![],
        feat_lists: vec![],
        levels,
        spontaneous: None,
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

#[test]
fn a_bonus_feat_of_the_players_choosing_is_a_one_feat_bonus_list() {
    // Warpriest: "gains Weapon Focus as a bonus feat in a weapon of his choosing". The table granted it outright.
    let mut first = row(1);
    first.feats = [
        feat_key("simple-weapon-proficiency"),
        feat_key("weapon-focus"),
    ]
    .into();
    let mut cult_leader_first = archetype_row(1);
    cult_leader_first.feats = [feat_key("simple-weapon-proficiency")].into();
    let mut warpriest = class(
        "warpriest",
        vec![first, row(2)],
        vec![archetype(
            "warpriest/cult-leader",
            vec![],
            vec![cult_leader_first.clone()],
        )],
    );
    parameters::apply_choices(&mut warpriest);
    parameters::add_class_lists(&mut warpriest);
    let slot = FeatSlot::OneOf {
        list: FeatListKey::new("warpriest/weapon-focus"),
    };
    assert_eq!(
        warpriest.levels[0].feats,
        [feat_key("simple-weapon-proficiency")].into()
    );
    assert_eq!(warpriest.levels[0].feat_slots, [slot]);
    assert_eq!(warpriest.archetypes[0].levels[0], cult_leader_first);
    let list = &warpriest.feat_lists[0];
    assert_eq!(
        (
            list.key.as_str(),
            list.guide.0.as_str(),
            list.ignores_prerequisites,
            list.feats.clone()
        ),
        (
            "warpriest/weapon-focus",
            "warpriest#sacred-weapon",
            true,
            [feat_key("weapon-focus")].into()
        )
    );

    // Master of All: "At 3rd, 9th and 15th levels, a Master of All receives a Skill Focus as a bonus feat." The
    // feature standing in for it goes.
    let stand_in = "rogue/master-of-all/skill-focus-master-of-all";
    let rows = [3, 9]
        .map(|level| {
            let mut r = archetype_row(level);
            r.features = vec![FeatureKey::new(stand_in)];
            r
        })
        .to_vec();
    let mut rogue = class(
        "rogue",
        vec![row(1)],
        vec![archetype(
            "rogue/master-of-all",
            vec![
                feature(stand_in),
                feature("rogue/master-of-all/bardic-knowledge"),
            ],
            rows,
        )],
    );
    parameters::apply_choices(&mut rogue);
    parameters::add_class_lists(&mut rogue);
    let master = &rogue.archetypes[0];
    assert_eq!(
        master
            .features
            .iter()
            .map(|f| f.key.as_str())
            .collect::<Vec<_>>(),
        ["rogue/master-of-all/bardic-knowledge"]
    );
    for row in &master.levels {
        assert!(row.features.is_empty());
        assert_eq!(
            row.feat_slots,
            [FeatSlot::OneOf {
                list: FeatListKey::new("rogue/master-of-all/skill-focus")
            }]
        );
    }
    assert_eq!(master.feat_lists[0].feats, [feat_key("skill-focus")].into());
    assert!(rogue.feat_lists.is_empty());
}

#[test]
fn the_files_write_weapons_parameters_repeats_and_chosen_feats() {
    let feat = |key: &str, parameter, repeat, prerequisite| Feat {
        key: feat_key(key),
        name: key.into(),
        category: FeatCategory::Weapon,
        summary: String::new(),
        guide: GuideRef::new(format!("feats#{key}")),
        parameter,
        repeat,
        prerequisite,
    };
    let mut talents = class("rogue", vec![row(1)], vec![]);
    let option = |key: &str, chosen_feat: Option<&str>, repeat| SelectionOption {
        key: SelectionOptionKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        feats: BTreeSet::new(),
        chosen_feat: chosen_feat.map(feat_key),
        repeat,
        prerequisite: None,
        levels: vec![],
        pool: None,
        replaces_row_of: None,
        parameter: None,
        values: Default::default(),
    };
    talents.selections.push(FeatureSelection {
        key: FeatureSelectionKey::new("rogue/rogue-talents"),
        name: "Rogue Talents".into(),
        summary: String::new(),
        guide: GuideRef::new("rogue#rogue-talents"),
        options: vec![
            option("rogue/rogue-talents/resiliency", None, Repeat::Times(2)),
            option(
                "rogue/rogue-talents/weapon-focus",
                Some("weapon-focus"),
                Repeat::Once,
            ),
        ],
        shared_options: BTreeSet::new(),
        option_levels: BTreeSet::new(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
    });
    let data = GameData {
        progression: rules::Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        weapons: vec![
            Weapon {
                key: WeaponKey::new("dagger"),
                name: "Dagger".into(),
                proficiency: Some(WeaponProficiency::Simple),
            },
            Weapon {
                key: WeaponKey::new("rays"),
                name: "Rays".into(),
                proficiency: None,
            },
        ],
        feats: vec![
            feat(
                "weapon-focus",
                Some(ParameterKind::Weapon),
                Repeat::PerValue,
                Some(Prerequisite::BaseAttackBonus(1)),
            ),
            feat(
                "weapon-specialization",
                Some(ParameterKind::Weapon),
                Repeat::Once,
                Some(Prerequisite::FeatForSameValue(feat_key("weapon-focus"))),
            ),
            feat(
                "aldori-dueling-mastery",
                None,
                Repeat::Once,
                Some(feat_for("weapon-focus", weapon("dagger"))),
            ),
            feat("extra-hex", None, Repeat::Times(3), None),
        ],
        classes: vec![talents],
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![],
    };
    let files = extract::ron_out::files(&data);
    let file = |name: &str| {
        files
            .iter()
            .find(|(path, _)| path.ends_with(name))
            .map(|(_, text)| text.clone())
            .unwrap_or_else(|| panic!("no {name}"))
    };
    let weapons = file("weapons.ron");
    assert_eq!(
        weapons,
        "[\n    (key: \"dagger\", name: \"Dagger\", proficiency: Simple),\n    (key: \"rays\", name: \"Rays\"),\n]\n"
    );
    let feats = file("feats.ron");
    for line in [
        "(key: \"weapon-focus\", name: \"weapon-focus\", category: Weapon, guide: \"feats#weapon-focus\", \
         parameter: Weapon, repeat: PerValue, prerequisite: BaseAttackBonus(1)),",
        "(key: \"weapon-specialization\", name: \"weapon-specialization\", category: Weapon, guide: \
         \"feats#weapon-specialization\", parameter: Weapon, prerequisite: FeatForSameValue(\"weapon-focus\")),",
        "prerequisite: FeatFor(feat: \"weapon-focus\", value: Weapon(\"dagger\"))),",
        "guide: \"feats#extra-hex\", repeat: Times(3)),",
    ] {
        assert!(feats.contains(line), "{line}\n{feats}");
    }
    let rogue = file("classes/rogue.ron");
    for line in [
        "(key: \"rogue/rogue-talents/resiliency\", name: \"rogue/rogue-talents/resiliency\", guide: \
         \"rogue/rogue-talents/resiliency\", repeat: Times(2)),",
        "guide: \"rogue/rogue-talents/weapon-focus\", chosen_feat: \"weapon-focus\"),",
    ] {
        assert!(rogue.contains(line), "{line}\n{rogue}");
    }

    // As `data` reads the files.
    fn read<T: serde::de::DeserializeOwned>(text: &str) -> T {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
            .unwrap()
    }
    assert_eq!(read::<Vec<Weapon>>(&weapons), data.weapons);
    assert_eq!(read::<Vec<Feat>>(&feats), data.feats);
    assert_eq!(read::<Class>(&rogue), data.classes[0]);
}
