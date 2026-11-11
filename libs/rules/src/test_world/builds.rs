//! Helpers for writing Builds against the test world: keys, picks, slots, Builds and why a pick can't be taken.

use crate::*;

use crate::test_world::world::world;

// ---- keys -----------------------------------------------------------------------------------------------------

pub fn feat(key: &str) -> FeatKey {
    FeatKey::new(key)
}
pub fn class(key: &str) -> ClassKey {
    ClassKey::new(key)
}
pub fn skill(key: &str) -> SkillKey {
    SkillKey::new(key)
}
pub fn path(key: &str) -> MythicPathKey {
    MythicPathKey::new(key)
}
pub fn feature(key: &str) -> FeatureKey {
    FeatureKey::new(key)
}
pub fn option(key: &str) -> SelectionOptionKey {
    SelectionOptionKey::new(key)
}
pub fn feat_list(key: &str) -> FeatListKey {
    FeatListKey::new(key)
}

pub fn weapon(key: &str) -> Parameter {
    Parameter::Weapon(WeaponKey::new(key))
}
pub fn skill_value(key: &str) -> Parameter {
    Parameter::Skill(skill(key))
}

pub fn pick_feat(key: &str) -> Pick {
    Pick::feat(feat(key))
}
/// A parameterised feat taken for `value`.
pub fn pick_feat_for(key: &str, value: Parameter) -> Pick {
    Pick::feat_for(feat(key), value)
}
pub fn pick_option(key: &str) -> Pick {
    Pick::option(option(key))
}
/// An option whose chosen feat is taken for `value`.
pub fn pick_option_for(key: &str, value: Parameter) -> Pick {
    Pick::option_for(option(key), value)
}
pub fn pick_mythic_ability(key: &str) -> Pick {
    Pick::mythic_ability(MythicAbilityKey::new(key))
}
/// A parameterised mythic ability taken for `value`.
pub fn pick_mythic_ability_for(key: &str, value: Parameter) -> Pick {
    Pick::mythic_ability_for(MythicAbilityKey::new(key), value)
}
pub fn pick_mythic_feat(key: &str) -> Pick {
    Pick::mythic_feat(MythicFeatKey::new(key))
}

// ---- slots ----------------------------------------------------------------------------------------------------

pub fn general_slot(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::General,
        index: 0,
    }
}
pub fn race_slot(index: u8) -> SlotId {
    SlotId::Feat {
        level: 1,
        source: FeatSource::Race,
        index,
    }
}
pub fn class_slot(level: u8, index: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::Class,
        index,
    }
}
pub fn selection_slot(level: u8, index: u8) -> SlotId {
    SlotId::Selection { level, index }
}

// ---- builds ---------------------------------------------------------------------------------------------------

/// Ability scores in the order Str, Dex, Con, Int, Wis, Cha.
pub fn scores(values: [u8; 6]) -> AbilityScores {
    let [
        strength,
        dexterity,
        constitution,
        intelligence,
        wisdom,
        charisma,
    ] = values;
    AbilityScores {
        strength,
        dexterity,
        constitution,
        intelligence,
        wisdom,
        charisma,
    }
}

/// A point buy in the order Str, Dex, Con, Int, Wis, Cha; it must fit (scores 7 to 18, at most 25 points).
pub fn point_buy(values: [u8; 6]) -> PointBuy {
    PointBuy::try_from(scores(values)).expect("a point buy that fits")
}

/// A Build of the given race with Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8 (21 of 25 points), an atheist (the
/// alignment needs a faith first) of true neutral, and one Level-up per class key.
pub fn build(race: &str, classes: &[&str]) -> Build {
    let mut build = Build {
        start: BuildStart {
            race: Some(RaceKey::new(race)),
            alignment: Some(Alignment::TrueNeutral),
            deity: Some(Faith::Atheism),
            ability_scores: point_buy([16, 14, 12, 13, 10, 8]),
            ..BuildStart::default()
        },
        ..Build::default()
    };
    for key in classes {
        build.levels.push(LevelUp::new(class(key)));
    }
    build
}

/// `n` copies of one class key.
pub fn times(key: &'static str, n: usize) -> Vec<&'static str> {
    vec![key; n]
}

pub fn engine() -> Engine {
    engine_on(world())
}

/// The engine on `data`, a test world that must pass [`GameData::check`]: every engine a test builds is one an app
/// could build. Panics with the problems otherwise.
pub fn engine_on(data: GameData) -> Engine {
    Engine::new(
        data.checked()
            .unwrap_or_else(|problems| panic!("the test world has problems: {problems:#?}")),
    )
}

// ---- impossible Builds ----------------------------------------------------------------------------------------

/// The Build without the pick in `slot`.
pub fn without(build: &Build, slot: &SlotId) -> Build {
    let mut build = build.clone();
    build.picks.remove(slot);
    build
}

/// Why the pick in `slot` can't be taken there, for a Build that only that pick makes impossible (it's possible
/// without it): what the slot's picker says of it, or `None` when the picker doesn't offer it at all (a slot the
/// Build doesn't have, or a pick the slot doesn't take).
pub fn why_not(build: &Build, slot: &SlotId) -> Option<Issue> {
    let engine = engine();
    assert!(!engine.is_possible(build), "the Build is possible");
    let rest = without(build, slot);
    assert!(engine.is_possible(&rest), "impossible without {slot:?} too");
    let pick = &build.picks[slot];
    let status = engine
        .options(&rest, slot)?
        .into_iter()
        .find(|option| &option.pick == pick)?
        .status;
    Some(status.expect_err("the picker offers it"))
}
