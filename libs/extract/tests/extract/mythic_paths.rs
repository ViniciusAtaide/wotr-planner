//! The mythic path rules the guide states in prose (ticket 26): each switch's keeps, previous paths, drops and
//! granted features; the features' ability bonuses; All-skilled's feats; Gold Dragon's Dragon Feats; and the
//! Mythic rank condition. Plus how the files write them.

use std::collections::BTreeSet;

use extract::grammar::Cond;
use extract::prose;
use extract::resolve::{Catalog, Context};
use rules::{
    Ability, AbilityBonus, FeatKey, FeatList, FeatListKey, FeatSlot, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GuideRef, MythicPath, MythicPathKey, MythicRankRow,
    Prerequisite, SelectionOption, SelectionOptionKey,
};

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

fn path(key: &str, features: &[&str]) -> MythicPath {
    MythicPath {
        key: MythicPathKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(key),
        features: features.iter().map(|key| feature(key)).collect(),
        selections: vec![],
        feat_lists: vec![],
        ranks: vec![],
        switch: prose::path_switch(key),
    }
}

fn features(items: &[&str]) -> BTreeSet<FeatureKey> {
    items.iter().map(|key| FeatureKey::new(*key)).collect()
}

fn feats(items: &[&str]) -> BTreeSet<FeatKey> {
    items.iter().map(|key| FeatKey::new(*key)).collect()
}

#[test]
fn each_switch_keeps_and_drops_what_the_guide_says() {
    let switch = |key: &str| prose::path_switch(key).unwrap();

    // "All rank 3 - 8 mythic abilities and feats" are lost, and "your rank is stuck at 3".
    let legend = switch("legend");
    assert_eq!(
        (legend.kept_through_rank, legend.path_kept_through_rank),
        (2, 2)
    );
    assert_eq!(legend.rank_after_switch(), 3);
    assert_eq!(
        legend.features,
        features(&["legend/legend", "legend/legends-corruption-resistance"])
    );
    assert_eq!(
        (legend.further_ascensions, legend.level_cap),
        (false, Some(40))
    );

    // "You lose any powers you gained at ranks 3 - 7, retaining only your mythic abilities and feats."
    for key in ["gold-dragon", "swarm-that-walks"] {
        let switch = switch(key);
        assert_eq!(
            (switch.kept_through_rank, switch.path_kept_through_rank),
            (7, 2),
            "{key}"
        );
        assert!(
            switch.from.is_empty() && switch.features.is_empty(),
            "{key}"
        );
        assert_eq!((switch.further_ascensions, switch.level_cap), (true, None));
    }

    // Only from Aeon or Azata; the Aeon's gaze abilities and the Azata's dragon companion are lost.
    let devil = switch("devil");
    assert_eq!(
        devil.from,
        [MythicPathKey::new("aeon"), MythicPathKey::new("azata")].into()
    );
    assert_eq!(
        (devil.kept_through_rank, devil.path_kept_through_rank),
        (7, 7)
    );
    assert_eq!(
        devil.dropped_features,
        features(&["aeon/enforcing-gaze-extra-gaze", "azata/dragon-companion"])
    );
    assert_eq!(
        devil.dropped_selections,
        [FeatureSelectionKey::new("aeon/enforcing-gaze")].into()
    );

    assert_eq!(prose::path_switch("trickster"), None);
}

#[test]
fn a_mythic_rank_line_is_a_mythic_rank_condition() {
    // "Mythic Rank: 9", "Rank 5": no longer a Character level stand-in.
    assert_eq!(
        Catalog::default().resolve(&Cond::MythicRank(9), &Context::default()),
        Ok(Prerequisite::MythicRank(9))
    );
}

#[test]
fn the_path_rules_add_legends_features_the_ability_bonuses_all_skilled_and_dragon_feats() {
    let mut azata = path("azata", &["azata/dragon-companion"]);
    azata.selections.push(FeatureSelection {
        key: FeatureSelectionKey::new("azata/superpowers"),
        name: "Superpowers".into(),
        summary: String::new(),
        guide: GuideRef::new("azata#superpowers"),
        options: vec![SelectionOption {
            key: SelectionOptionKey::new("azata/superpowers/all-skilled"),
            name: "All-skilled".into(),
            summary: String::new(),
            guide: GuideRef::new("azata#all-skilled"),
            feats: BTreeSet::new(),
            chosen_feat: None,
            repeat: rules::Repeat::Once,
            prerequisite: None,
            levels: vec![],
            pool: None,
            replaces_row_of: None,
            group: None,
            arcana: None,
            parameter: None,
            values: Default::default(),
        }],
        shared_options: BTreeSet::new(),
        option_levels: BTreeSet::new(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    });
    let mut dragon = path(
        "gold-dragon",
        &[
            "gold-dragon/perfect-body",
            "gold-dragon/perfect-mind",
            "gold-dragon/perfect-soul",
        ],
    );
    dragon.ranks.push(MythicRankRow {
        rank: 9,
        features: vec![],
        feat_slots: prose::dragon_feats_slots(),
        selections: vec![],
    });
    let mut paths = vec![
        path("legend", &[]),
        dragon,
        path(
            "swarm-that-walks",
            &[
                "swarm-that-walks/eye-of-the-swarm",
                "swarm-that-walks/the-swarm",
            ],
        ),
        azata,
    ];
    prose::path_rules(&mut paths);

    let bonuses = |path: &MythicPath, key: &str| {
        path.features
            .iter()
            .find(|f| f.key.as_str() == key)
            .unwrap_or_else(|| panic!("{key}"))
            .ability_bonuses
            .clone()
    };
    // Legend's two features, the first with +4 to every score.
    assert_eq!(
        paths[0]
            .features
            .iter()
            .map(|f| (f.key.as_str(), f.guide.0.as_str()))
            .collect::<Vec<_>>(),
        [
            ("legend/legend", "other-mythic-paths#legend"),
            (
                "legend/legends-corruption-resistance",
                "other-mythic-paths#legends-corruption-resistance"
            ),
        ]
    );
    assert_eq!(
        bonuses(&paths[0], "legend/legend"),
        [AbilityBonus::Every { amount: 4 }]
    );
    assert_eq!(
        bonuses(&paths[1], "gold-dragon/perfect-body"),
        [
            AbilityBonus::Highest { amount: 4 },
            AbilityBonus::AtLeast { score: 14 }
        ]
    );
    assert_eq!(
        bonuses(&paths[1], "gold-dragon/perfect-mind"),
        [
            AbilityBonus::Highest { amount: 2 },
            AbilityBonus::AtLeast { score: 18 }
        ]
    );
    assert_eq!(
        bonuses(&paths[1], "gold-dragon/perfect-soul"),
        [AbilityBonus::Highest { amount: 2 }]
    );
    assert_eq!(
        bonuses(&paths[2], "swarm-that-walks/the-swarm"),
        [Ability::Intelligence, Ability::Wisdom, Ability::Charisma]
            .map(|ability| AbilityBonus::Fixed { ability, amount: 4 })
    );
    // Dragon Feats: three slots on the Gold Dragon's own list of every feat, ignoring prerequisites.
    assert_eq!(
        paths[1].feat_lists,
        [FeatList {
            key: FeatListKey::new("gold-dragon/dragon-feats"),
            name: "Dragon Feats".into(),
            guide: GuideRef::new("gold-dragon#dragon-feats"),
            ignores_prerequisites: true,
            every_feat: true,
            feats: BTreeSet::new(),
        }]
    );
    assert_eq!(
        prose::dragon_feats_slots(),
        vec![
            FeatSlot::OneOf {
                list: FeatListKey::new("gold-dragon/dragon-feats")
            };
            3
        ]
    );
    // All-skilled: every weapon, armour and shield proficiency, not the companion's barding.
    assert_eq!(
        paths[3].selections[0].options[0].feats,
        feats(&[
            "armor-proficiency-heavy-armor",
            "armor-proficiency-light-armor",
            "armor-proficiency-medium-armor",
            "exotic-weapon-proficiency",
            "martial-weapons-proficiency",
            "shield-proficiency",
            "simple-weapon-proficiency",
            "tower-shield-proficiency",
        ])
    );
}

#[test]
fn the_new_shapes_are_written_on_one_line_each_and_read_back() {
    let mut legend = path("legend", &[]);
    prose::path_rules(std::slice::from_mut(&mut legend));
    let mut devil = path("devil", &["devil/wings"]);
    devil.selections.push(FeatureSelection {
        key: FeatureSelectionKey::new("devil/picks"),
        name: "Picks".into(),
        summary: String::new(),
        guide: GuideRef::new("devil#picks"),
        options: vec![SelectionOption {
            key: SelectionOptionKey::new("devil/picks/two"),
            name: "Two".into(),
            summary: String::new(),
            guide: GuideRef::new("devil#two"),
            feats: feats(&["dodge", "toughness"]),
            chosen_feat: None,
            repeat: rules::Repeat::Once,
            prerequisite: Some(Prerequisite::MythicRank(9)),
            levels: vec![],
            pool: None,
            replaces_row_of: None,
            group: None,
            arcana: None,
            parameter: None,
            values: Default::default(),
        }],
        shared_options: BTreeSet::new(),
        option_levels: BTreeSet::new(),
        skipped_levels: BTreeSet::new(),
        feat_lists_only: false,
        only_without_option: false,
        pool_of: None,
        levels_behind: 0,
        level_from: None,
        once_per_option: false,
        level_also_from: None,
    });
    let data = rules::GameData {
        progression: rules::Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        weapons: vec![],
        feats: vec![],
        classes: vec![],
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: vec![devil, legend],
        backgrounds: vec![],
    };
    let files = extract::ron_out::files(&data);
    let file = |name: &str| {
        files
            .iter()
            .find(|(path, _)| path.ends_with(format!("mythic/paths/{name}.ron")))
            .map(|(_, text)| text.clone())
            .unwrap()
    };
    let (devil_text, legend_text) = (file("devil"), file("legend"));
    assert!(
        devil_text.contains(
            "    switch: (from: [\"aeon\", \"azata\"], kept_through_rank: 7, path_kept_through_rank: 7, \
             dropped_features: [\"aeon/enforcing-gaze-extra-gaze\", \"azata/dragon-companion\"], \
             dropped_selections: [\"aeon/enforcing-gaze\"], further_ascensions: true),\n"
        ),
        "{devil_text}"
    );
    assert!(
        devil_text.contains("feats: [\"dodge\", \"toughness\"], prerequisite: MythicRank(9))"),
        "{devil_text}"
    );
    assert!(
        legend_text.contains(
            "    switch: (kept_through_rank: 2, path_kept_through_rank: 2, features: [\"legend/legend\", \
             \"legend/legends-corruption-resistance\"], further_ascensions: false, level_cap: Some(40)),\n"
        ),
        "{legend_text}"
    );
    assert!(
        legend_text.contains(
            "(key: \"legend/legend\", name: \"Legend\", guide: \"other-mythic-paths#legend\", \
             ability_bonuses: [Every(amount: 4)]),"
        ),
        "{legend_text}"
    );
    // As `seeds` reads the files.
    let read = |text: &str| -> MythicPath {
        ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
            .from_str(text)
            .unwrap()
    };
    assert_eq!(read(&devil_text), data.mythic_paths[0]);
    assert_eq!(read(&legend_text), data.mythic_paths[1]);

    // A list of every feat writes its flag and no feats.
    let mut dragon = path("gold-dragon", &[]);
    prose::path_rules(std::slice::from_mut(&mut dragon));
    let files = extract::ron_out::files(&rules::GameData {
        mythic_paths: vec![dragon.clone()],
        ..data
    });
    let text = &files.last().unwrap().1;
    assert!(
        text.contains(
            "        (key: \"gold-dragon/dragon-feats\", name: \"Dragon Feats\", guide: \"gold-dragon#dragon-feats\", \
             ignores_prerequisites: true, every_feat: true),\n"
        ),
        "{text}"
    );
    assert_eq!(read(text), dragon);
}
