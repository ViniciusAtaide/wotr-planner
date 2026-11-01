//! Feat lists: while building, every fixed list is a pending one; a list of one feat becomes a granted feat, and
//! the rest are named after their owner (the archetype that alone uses one, else the class) and the first level
//! that uses them.

use std::collections::BTreeSet;

use extract::feat_lists::{PendingLists, grant_lone_feats, name_lists};
use rules::{
    Archetype, ArchetypeKey, ArchetypeLevelRow, BabProgression, Class, ClassKey, ClassLevelRow,
    FeatKey, FeatList, FeatListKey, FeatSlot, GameData, GuideRef, MythicPath, MythicPathKey,
    MythicRankRow, Progression, SaveProgression,
};

fn feats(keys: &[&str]) -> BTreeSet<FeatKey> {
    keys.iter().map(|key| FeatKey::new(*key)).collect()
}

fn row(level: u8, slots: Vec<FeatSlot>) -> ClassLevelRow {
    ClassLevelRow {
        level,
        arcane_spell_level: 0,
        divine_spell_level: 0,
        advances_arcane: false,
        advances_divine: false,
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: slots,
        selections: vec![],
    }
}

fn archetype_row(level: u8, slots: Vec<FeatSlot>) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level,
        spell_levels: None,
        features: vec![],
        feats: BTreeSet::new(),
        feat_slots: slots,
        selections: vec![],
    }
}

fn archetype(key: &str, levels: Vec<ArchetypeLevelRow>) -> Archetype {
    Archetype {
        key: ArchetypeKey::new(key),
        name: key.into(),
        summary: String::new(),
        guide: GuideRef::new(format!("ranger#{}", key.rsplit('/').next().unwrap())),
        prerequisite: None,
        skill_points: None,
        class_skills: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
    }
}

fn class(levels: Vec<ClassLevelRow>, archetypes: Vec<Archetype>) -> Class {
    Class {
        key: ClassKey::new("ranger"),
        name: "Ranger".into(),
        summary: String::new(),
        guide: GuideRef::new("ranger"),
        skill_points: 6,
        bab: BabProgression::Full,
        fort: SaveProgression::Good,
        reflex: SaveProgression::Good,
        will: SaveProgression::Poor,
        save_exceptions: vec![],
        advances: rules::Advancement::Each,
        counts_as: std::collections::BTreeSet::new(),
        class_skills: Default::default(),
        prerequisite: None,
        features: vec![],
        selections: vec![],
        feat_lists: vec![],
        levels,
        archetypes,
    }
}

fn data(classes: Vec<Class>, paths: Vec<MythicPath>) -> GameData {
    GameData {
        progression: Progression {
            mythic_ascensions: vec![],
        },
        races: vec![],
        skills: vec![],
        weapons: vec![],
        feats: vec![],
        classes,
        prestige_classes: vec![],
        mythic_abilities: vec![],
        mythic_feats: vec![],
        mythic_paths: paths,
    }
}

fn one_of(key: &str) -> FeatSlot {
    FeatSlot::OneOf {
        list: FeatListKey::new(key),
    }
}

#[test]
fn a_list_of_one_feat_becomes_a_granted_feat() {
    let mut lists = PendingLists::default();
    let scribe = lists.slot(feats(&["scribe-scrolls"]), true);
    let checked = lists.slot(feats(&["endurance"]), false);
    let bonus = lists.slot(feats(&["dodge", "toughness"]), true);
    let mut data = data(
        vec![class(
            vec![row(
                1,
                vec![FeatSlot::General, scribe.clone(), bonus.clone()],
            )],
            vec![archetype(
                "ranger/x",
                vec![archetype_row(1, vec![checked, scribe])],
            )],
        )],
        vec![],
    );

    grant_lone_feats(&mut data, &lists);

    let ranger = &data.classes[0];
    assert_eq!(ranger.levels[0].feats, feats(&["scribe-scrolls"]));
    assert_eq!(ranger.levels[0].feat_slots, [FeatSlot::General, bonus]);
    let x = &ranger.archetypes[0].levels[0];
    assert_eq!(x.feats, feats(&["endurance", "scribe-scrolls"]));
    assert_eq!(x.feat_slots, []);
}

#[test]
fn lists_are_named_after_their_owner_and_first_level() {
    let mut lists = PendingLists::default();
    let style = lists.slot(feats(&["cleave", "power-attack"]), true);
    let archery = lists.slot(feats(&["point-blank-shot", "rapid-shot"]), true);
    let own = lists.slot(feats(&["dodge", "mobility"]), false);
    let mut data = data(
        vec![class(
            vec![
                row(1, vec![]),
                row(2, vec![style.clone()]),
                row(3, vec![]),
                row(6, vec![style]),
            ],
            vec![
                // Two archetypes share a list the base class doesn't use: the class owns it.
                archetype(
                    "ranger/nomad",
                    vec![archetype_row(2, vec![archery.clone()])],
                ),
                archetype(
                    "ranger/stormwalker",
                    vec![
                        archetype_row(2, vec![archery]),
                        // Only this archetype uses this one: it owns it.
                        archetype_row(3, vec![own]),
                    ],
                ),
            ],
        )],
        vec![],
    );

    name_lists(&mut data, &lists);

    let ranger = &data.classes[0];
    let summary = |lists: &[FeatList]| -> Vec<(String, String, String, bool, usize)> {
        lists
            .iter()
            .map(|list| {
                (
                    list.key.to_string(),
                    list.name.clone(),
                    list.guide.0.clone(),
                    list.ignores_prerequisites,
                    list.feats.len(),
                )
            })
            .collect()
    };
    // Both class lists start at level 2: the base class's keeps `-2`, the archetypes' gets `-2b`.
    assert_eq!(
        summary(&ranger.feat_lists),
        [
            (
                "ranger/bonus-feats-2".into(),
                "Bonus Feats (from level 2)".into(),
                "ranger".into(),
                true,
                2
            ),
            (
                "ranger/bonus-feats-2b".into(),
                "Bonus Feats (from level 2)".into(),
                "ranger".into(),
                true,
                2
            ),
        ]
    );
    assert_eq!(
        ranger.levels[1].feat_slots,
        [one_of("ranger/bonus-feats-2")]
    );
    assert_eq!(
        ranger.archetypes[0].levels[0].feat_slots,
        [one_of("ranger/bonus-feats-2b")]
    );
    // An owner with one list names it without a level.
    let stormwalker = &ranger.archetypes[1];
    assert_eq!(
        summary(&stormwalker.feat_lists),
        [(
            "ranger/stormwalker/bonus-feats".into(),
            "Bonus Feats".into(),
            "ranger#stormwalker".into(),
            false,
            2
        )]
    );
    assert_eq!(
        stormwalker.levels[1].feat_slots,
        [one_of("ranger/stormwalker/bonus-feats")]
    );
}

#[test]
fn a_mythic_rank_list_is_the_paths() {
    let mut lists = PendingLists::default();
    let slot = lists.slot(feats(&["dodge", "toughness"]), false);
    let mut data = data(
        vec![],
        vec![MythicPath {
            key: MythicPathKey::new("trickster"),
            name: "Trickster".into(),
            summary: String::new(),
            guide: GuideRef::new("trickster"),
            features: vec![],
            selections: vec![],
            feat_lists: vec![],
            ranks: vec![MythicRankRow {
                rank: 4,
                features: vec![],
                feat_slots: vec![slot],
                selections: vec![],
            }],
            switch: None,
        }],
    );

    name_lists(&mut data, &lists);

    let path = &data.mythic_paths[0];
    assert_eq!(path.feat_lists[0].key.as_str(), "trickster/bonus-feats");
    assert_eq!(path.feat_lists[0].guide.0, "trickster");
    assert_eq!(path.ranks[0].feat_slots, [one_of("trickster/bonus-feats")]);
}
