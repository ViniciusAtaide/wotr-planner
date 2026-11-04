//! Class rules (ticket 28).

use std::collections::BTreeSet;

use rules::{
    Advancement, Build, CharacterState, ClassChoice, ClassKey, CountsAs, CountsToward, FeatKey,
    FeatureKey, MythicPathKey, Parameter, Pick, SkillKey, SlotId, Story,
};

use crate::support::*;

#[test]
fn signifer_loremaster_and_winter_witch_advance_one_class_arcane_or_divine() {
    let either: BTreeSet<&str> = DATA
        .all_classes()
        .filter(|class| class.advances == Advancement::Either)
        .map(|class| class.key.as_str())
        .collect();
    assert_eq!(
        either,
        BTreeSet::from(["hellknight-signifer", "loremaster", "winter-witch"])
    );
    assert_eq!(class("mystic-theurge").advances, Advancement::Each);

    let mut both = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "cleric",
        "cleric",
        "cleric",
        "cleric",
        "cleric",
        "loremaster",
    ]);
    let choose = |build: &mut Build, advances: &[&str]| {
        build.class_choices.insert(
            ClassKey::new("loremaster"),
            ClassChoice {
                archetype: None,
                advances: advances.iter().map(|key| ClassKey::new(*key)).collect(),
            },
        );
    };
    // What the Loremaster's entry needs: Skill Focus (Knowledge (Arcana)), a metamagic feat and 3rd-level spells.
    both.picks.insert(
        general_slot(1),
        feat_pick(
            "skill-focus",
            Parameter::Skill(SkillKey::new("knowledge-arcana")),
        ),
    );
    both.picks.insert(
        general_slot(3),
        Pick::feat(FeatKey::new("metamagic-extend-spell")),
    );
    choose(&mut both, &["wizard", "cleric"]);
    assert!(!engine().is_possible(&both));
    choose(&mut both, &["cleric"]);
    assert!(engine().is_possible(&both));
}

#[test]
fn eldritch_knight_levels_count_as_fighter_levels_for_feats() {
    assert_eq!(
        class("eldritch-knight").counts_as,
        BTreeSet::from([CountsAs {
            class: ClassKey::new("fighter"),
            toward: CountsToward::FeatPrerequisites,
        }])
    );
    // Wizard 5, Fighter 1, Eldritch Knight 3: fighter level 4 for feats at character level 9.
    let mut build = classes(&[
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "wizard",
        "fighter",
        "eldritch-knight",
        "eldritch-knight",
        "eldritch-knight",
    ]);
    build.picks.insert(
        class_slot(6),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
    );
    let specialization = Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword"));
    let status = |level| {
        engine()
            .options(&build, &general_slot(level))
            .unwrap()
            .into_iter()
            .find(|option| option.pick == specialization)
            .unwrap()
            .status
    };
    assert!(status(7).is_err(), "fighter 1 / Eldritch Knight 1");
    assert_eq!(status(9), Ok(()));
}

#[test]
fn winter_witch_levels_stack_with_witch_levels_for_major_and_grand_hexes() {
    let toward = |feature: &str| CountsAs {
        class: ClassKey::new("witch"),
        toward: CountsToward::Feature(FeatureKey::new(feature)),
    };
    assert_eq!(
        class("winter-witch").counts_as,
        BTreeSet::from([toward("witch/major-hexes"), toward("witch/grand-hexes")])
    );
    let mut keys = vec!["witch"; 9];
    keys.push("winter-witch");
    let sheet = engine().evaluate(&classes(&keys));
    let major = FeatureKey::new("witch/major-hexes");
    assert_eq!(sheet.at(9).unwrap().feature_count(&major), 0);
    assert_eq!(sheet.at(10).unwrap().feature_count(&major), 1);
}

#[test]
fn the_gold_dragon_breaths_story_conditions_are_notes_that_never_block() {
    let engine = engine();
    let breath = |key: &str| pick(&format!("gold-dragon/improved-gold-dragon-breath/{key}"));
    for (key, story) in [
        ("annihilating-breath", Story::VowOfDahak),
        ("decaying-breath", Story::VowOfDahak),
        ("enlightening-breath", Story::Uncorrupted),
        ("rejuvenating-breath", Story::Uncorrupted),
    ] {
        assert_eq!(
            engine.story_conditions(&breath(key)),
            BTreeSet::from([story]),
            "{key}"
        );
    }
    let mut build = mythic("trickster", 17);
    build.path_switch = Some(MythicPathKey::new("gold-dragon"));
    let options = engine.options(&build, &path_selection(9)).unwrap();
    for key in ["annihilating-breath", "enlightening-breath"] {
        let option = options.iter().find(|o| o.pick == breath(key)).unwrap();
        assert_eq!(option.status, Ok(()), "{key}");
    }
}

#[test]
fn a_crusaders_bonus_feats_still_skip_every_prerequisite_and_the_data_says_so() {
    // Weapon Focus needs BAB 1; a crusader cleric 1 has 0.
    let crusader = archetype_build("cleric", "cleric/crusader", 1);
    let focus = Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword"));
    let status = engine()
        .options(&crusader, &class_slot(1))
        .unwrap()
        .into_iter()
        .find(|option| option.pick == focus)
        .unwrap()
        .status;
    assert_eq!(status, Ok(()));

    let cleric =
        std::fs::read_to_string(seeds::game_data_dir().join("classes/cleric.ron")).unwrap();
    assert!(
        cleric.contains("// Crusader Bonus Feats:"),
        "the note on the Crusader's lists"
    );
}

#[test]
fn an_assassin_is_true_neutral_or_evil() {
    use rules::Alignment::*;
    let assassin = class("assassin").prerequisite.as_ref().unwrap();
    let allowed = [
        LawfulGood,
        NeutralGood,
        ChaoticGood,
        LawfulNeutral,
        TrueNeutral,
        ChaoticNeutral,
        LawfulEvil,
        NeutralEvil,
        ChaoticEvil,
    ]
    .into_iter()
    .filter(|alignment| {
        let state = CharacterState {
            alignment: Some(*alignment),
            character_level: 6,
            skill_ranks: [(SkillKey::new("stealth"), 5)].into(),
            ..CharacterState::default()
        };
        state.check(assassin).is_ok()
    })
    .collect::<Vec<_>>();
    assert_eq!(allowed, [TrueNeutral, LawfulEvil, NeutralEvil, ChaoticEvil]);
}

fn nature_bond(archetype: Option<&str>, option: Option<&str>) -> CharacterState {
    let mut druid = match archetype {
        Some(archetype) => archetype_build("druid", archetype, 1),
        None => build("druid", 1),
    };
    if let Some(option) = option {
        druid.picks.insert(
            selection_slot(1),
            pick(&format!("druid/nature-bond/{option}")),
        );
    }
    engine().evaluate(&druid).at(1).cloned().unwrap()
}

#[test]
fn a_druid_chooses_an_animal_companion_or_a_domain_at_level_1() {
    let offered: BTreeSet<String> = offered(&build("druid", 1), 1).into_keys().collect();
    assert_eq!(
        offered,
        BTreeSet::from([
            "druid/nature-bond/animal-companion".to_owned(),
            "druid/nature-bond/domain".to_owned()
        ])
    );
    // The archetypes whose table keeps both columns choose too; the others have their companion, their domain
    // or neither.
    let chooses = |archetype: &str| {
        let row = class("druid")
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == archetype)
            .unwrap()
            .levels
            .iter()
            .find(|row| row.level == 1)
            .unwrap();
        row.selections
            .contains(&rules::FeatureSelectionKey::new("druid/nature-bond"))
    };
    for archetype in ["druid/defender-of-the-true-world", "druid/feyspeaker"] {
        assert!(chooses(archetype), "{archetype}");
    }
    for archetype in [
        "druid/blight-druid",
        "druid/drovier",
        "druid/elemental-rampager",
        "druid/primal-druid",
        "druid/winter-child",
    ] {
        assert!(!chooses(archetype), "{archetype}");
    }
}

#[test]
fn domain_zealot_needs_a_druids_domain_and_boon_companion_its_companion() {
    let mythic_ability = |key: &str| {
        DATA.mythic_abilities
            .iter()
            .find(|a| a.key.as_str() == key)
            .unwrap()
            .prerequisite
            .clone()
            .unwrap()
    };
    let domain_zealot = mythic_ability("domain-zealot");
    let boon_companion = feat("boon-companion").prerequisite.clone().unwrap();

    // Ticket 31: Domain Zealot needs a domain held, so the domain option needs its domain picked.
    assert!(
        nature_bond(None, Some("domain"))
            .check(&domain_zealot)
            .is_err()
    );
    let mut weather = build("druid", 1);
    weather
        .picks
        .insert(selection_slot(1), pick("druid/nature-bond/domain"));
    weather.picks.insert(
        SlotId::OptionSelection { level: 1, index: 0 },
        pick("cleric/domain/weather"),
    );
    let domain = engine().evaluate(&weather).at(1).cloned().unwrap();
    assert_eq!(domain.check(&domain_zealot), Ok(()));
    assert!(domain.check(&boon_companion).is_err());

    let companion = nature_bond(None, Some("animal-companion"));
    assert!(companion.check(&domain_zealot).is_err());
    assert_eq!(companion.check(&boon_companion), Ok(()));

    assert!(nature_bond(None, None).check(&domain_zealot).is_err());
    // A Blight Druid picks a domain and a Winter Child always has one; a Primal Druid never does.
    let mut blight = archetype_build("druid", "druid/blight-druid", 1);
    blight
        .picks
        .insert(selection_slot(1), pick("cleric/domain/death"));
    assert_eq!(
        engine()
            .evaluate(&blight)
            .at(1)
            .unwrap()
            .check(&domain_zealot),
        Ok(())
    );
    assert_eq!(
        nature_bond(Some("druid/winter-child"), None).check(&domain_zealot),
        Ok(())
    );
    assert!(
        nature_bond(Some("druid/primal-druid"), None)
            .check(&domain_zealot)
            .is_err()
    );
}
