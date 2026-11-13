//! Parameterised and repeatable feats (ticket 27).

use std::collections::{BTreeSet, HashMap};

use rules::{
    ArmourType, FeatKey, Issue, Parameter, ParameterKind, Pick, Prerequisite, Repeat,
    SelectionOptionKey, SlotKind, SpellSchool, Unmet, WeaponProficiency,
};

use crate::support::*;

fn feat_for(feat: &str, value: Parameter) -> Prerequisite {
    Prerequisite::FeatFor {
        feat: FeatKey::new(feat),
        value,
    }
}

#[test]
fn the_weapons_are_the_guides_proficiency_lists_and_what_weapon_focus_names() {
    let count = |proficiency| {
        DATA.weapons
            .iter()
            .filter(|w| w.proficiency == proficiency)
            .count()
    };
    assert_eq!(count(Some(WeaponProficiency::Simple)), 15);
    assert_eq!(count(Some(WeaponProficiency::Martial)), 27);
    // The Exotic Weapon Proficiency list and the Sawtooth Saber (Weapon Finesse's exotic list; ticket 32).
    assert_eq!(count(Some(WeaponProficiency::Exotic)), 17);
    // Unarmed strike, touch attacks and rays, the natural attacks the guide names (ticket 32: one value each),
    // Kinetic Blast and Bomb.
    let others: BTreeSet<&str> = DATA
        .weapons
        .iter()
        .filter(|w| w.proficiency.is_none())
        .map(|w| w.key.as_str())
        .collect();
    assert_eq!(
        others,
        BTreeSet::from([
            "bite",
            "bomb",
            "claw",
            "gore",
            "hoof",
            "kinetic-blast",
            "rays",
            "slam",
            "spike",
            "tail",
            "talon",
            "touch-attacks",
            "unarmed-strike",
            "wing"
        ])
    );
}

#[test]
fn feats_repeat_and_take_values_as_the_guide_says() {
    use ParameterKind as K;
    let shape = |key| (feat(key).parameter, feat(key).repeat);
    assert_eq!(shape("weapon-focus"), (Some(K::Weapon), Repeat::PerValue));
    assert_eq!(shape("skill-focus"), (Some(K::Skill), Repeat::PerValue));
    assert_eq!(
        shape("exotic-weapon-proficiency"),
        (Some(K::ExoticWeapon), Repeat::PerValue)
    );
    assert_eq!(
        shape("spell-focus"),
        (Some(K::SpellSchool), Repeat::PerValue)
    );
    assert_eq!(
        shape("elemental-focus"),
        (Some(K::Energy), Repeat::PerValue)
    );
    assert_eq!(
        shape("armor-focus"),
        (Some(K::ArmourType), Repeat::PerValue)
    );
    assert_eq!(
        shape("improved-critical"),
        (Some(K::Weapon), Repeat::PerValue)
    );
    assert_eq!(shape("extra-hex"), (None, Repeat::Times(3)));
    assert_eq!(shape("extra-ki"), (None, Repeat::Unlimited));
    assert_eq!(shape("magical-tail"), (None, Repeat::Times(8)));
    let repeating: Vec<&str> = DATA
        .feats
        .iter()
        .filter(|f| f.repeat != Repeat::Once)
        .map(|f| f.key.as_str())
        .collect();
    // The 16 parameterised feats once per value (ticket 32; Stat Focus since ticket 125), Magical Tail and the 18
    // Extra feats the guide says repeat (not Extra Bane Weapon: it doesn't say).
    assert_eq!(repeating.len(), 35, "{repeating:?}");
}

#[test]
fn weapon_focus_is_taken_once_per_weapon_and_specialization_needs_the_same_weapon() {
    let mut build = build("fighter", 4);
    build.picks.insert(
        class_slot(1),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
    );
    build.picks.insert(
        general_slot(1),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("dagger")),
    );
    build.picks.insert(
        class_slot(2),
        Pick::feat_for(FeatKey::new("weapon-focus"), weapon("dagger")),
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("rapier")),
    );
    assert_eq!(
        why_not(&without(&build, &class_slot(4)), &class_slot(2)),
        Some(Issue::AlreadyHeld)
    );
    assert_eq!(
        why_not(&without(&build, &class_slot(2)), &class_slot(4)),
        Some(Issue::Unmet(Unmet::Condition {
            needs: Prerequisite::FeatForSameValue(FeatKey::new("weapon-focus")),
            has: None,
        }))
    );
    build.picks.insert(
        class_slot(4),
        Pick::feat_for(FeatKey::new("weapon-specialization"), weapon("longsword")),
    );
    build.picks.remove(&class_slot(2));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 4, "weapon-focus"),
        BTreeSet::from([weapon("dagger"), weapon("longsword")])
    );
}

#[test]
fn prerequisites_name_the_values_the_guide_names() {
    let has = |key: &str, needs: &Prerequisite| {
        feat(key)
            .prerequisite
            .iter()
            .flat_map(Prerequisite::leaves)
            .any(|leaf| leaf == needs)
    };
    let same = |key: &str| Prerequisite::FeatForSameValue(FeatKey::new(key));
    assert!(has(
        "aldori-dueling-mastery",
        &feat_for("weapon-focus", weapon("dueling-sword"))
    ));
    assert!(has(
        "acrobatic-spellcaster",
        &feat_for("skill-focus", skill("mobility"))
    ));
    assert!(has(
        "slayers-feint",
        &feat_for("skill-focus", skill("mobility"))
    ));
    assert!(has(
        "augment-summoning",
        &feat_for(
            "spell-focus",
            Parameter::SpellSchool(SpellSchool::Conjuration)
        )
    ));
    for weapon_ in [
        "dagger",
        "estoc",
        "punching-dagger",
        "rapier",
        "sai",
        "starknife",
    ] {
        assert!(has(
            "fencing-grace",
            &feat_for("weapon-focus", weapon(weapon_))
        ));
    }
    assert!(has("greater-weapon-focus", &same("weapon-focus")));
    assert!(has("weapon-specialization", &same("weapon-focus")));
    assert!(has(
        "greater-weapon-specialization",
        &same("weapon-specialization")
    ));
    assert!(has("greater-spell-focus", &same("spell-focus")));
    assert!(has("greater-elemental-focus", &same("elemental-focus")));

    // The mythic Armor Focus feats name their armour type.
    let light = DATA
        .mythic_feats
        .iter()
        .find(|f| f.key.as_str() == "mythic-armor-focus-light-armor-assault")
        .unwrap();
    assert_eq!(
        light.prerequisite,
        Some(feat_for(
            "armor-focus",
            Parameter::ArmourType(ArmourType::Light)
        ))
    );
}

#[test]
fn an_aldori_swordlord_needs_weapon_focus_in_the_dueling_sword() {
    let swordlord = DATA
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "aldori-swordlord")
        .unwrap();
    let Some(Prerequisite::All(lines)) = &swordlord.prerequisite else {
        panic!("{:?}", swordlord.prerequisite)
    };
    assert!(lines.contains(&feat_for("weapon-focus", weapon("dueling-sword"))));
    assert!(!lines.contains(&Prerequisite::Feat(FeatKey::new("weapon-focus"))));
}

#[test]
fn a_warpriest_takes_weapon_focus_in_a_weapon_of_his_choosing() {
    // Base, Champion of the Faith and Feral Champion: a bonus slot of Weapon Focus alone, prerequisites ignored
    // (a warpriest 1 has BAB 0).
    for archetype in [
        None,
        Some("warpriest/champion-of-the-faith"),
        Some("warpriest/feral-champion"),
    ] {
        let mut build = match archetype {
            None => build("warpriest", 1),
            Some(archetype) => archetype_build("warpriest", archetype, 1),
        };
        let slot = class_slot(1);
        let options = engine().options(&build, &slot).unwrap();
        assert!(
            options
                .iter()
                .all(|option| option.pick.key() == "weapon-focus" && option.status.is_ok()),
            "{archetype:?}"
        );
        assert_eq!(options.len(), DATA.weapons.len());
        build.picks.insert(
            slot,
            Pick::feat_for(FeatKey::new("weapon-focus"), weapon("longsword")),
        );
        assert!(engine().is_possible(&build), "{archetype:?}");
        assert_eq!(
            values_at(&build, 1, "weapon-focus"),
            BTreeSet::from([weapon("longsword")])
        );
    }
    // A Cult Leader doesn't get it.
    let build = archetype_build("warpriest", "warpriest/cult-leader", 1);
    assert_eq!(engine().options(&build, &class_slot(1)), None);
}

#[test]
fn a_master_of_all_takes_three_skill_focus_in_three_skills() {
    let mut build = archetype_build("rogue", "rogue/master-of-all", 15);
    // "Prerequisites: Half-Elf".
    build.start.race = Some(rules::RaceKey::new("half-elf"));
    for (level, skill_) in [(3, "mobility"), (9, "perception"), (15, "persuasion")] {
        build.picks.insert(
            class_slot(level),
            Pick::feat_for(FeatKey::new("skill-focus"), skill(skill_)),
        );
    }
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 15, "skill-focus"),
        BTreeSet::from([skill("mobility"), skill("perception"), skill("persuasion")])
    );
    // Not twice in one skill.
    build.picks.insert(
        class_slot(15),
        Pick::feat_for(FeatKey::new("skill-focus"), skill("mobility")),
    );
    assert_eq!(why_not(&build, &class_slot(15)), Some(Issue::AlreadyHeld));
}

#[test]
fn the_rogue_talents_weapon_focus_and_skill_focus_name_their_value() {
    let mut build = build("rogue", 2);
    let talent = |key: &str, value| {
        Pick::option_for(
            SelectionOptionKey::new(format!("rogue/rogue-talents/{key}")),
            value,
        )
    };
    build
        .picks
        .insert(selection_slot(2), talent("weapon-focus", weapon("rapier")));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "weapon-focus"),
        BTreeSet::from([weapon("rapier")])
    );
    build
        .picks
        .insert(selection_slot(2), talent("skill-focus", skill("stealth")));
    assert!(engine().is_possible(&build));
    assert_eq!(
        values_at(&build, 2, "skill-focus"),
        BTreeSet::from([skill("stealth")])
    );
}

#[test]
fn rage_powers_discoveries_incense_and_mutations_repeat_as_the_guide_says() {
    let options: HashMap<&str, Repeat> = DATA
        .class_selections()
        .flat_map(|selection| &selection.options)
        .filter(|option| option.repeat != Repeat::Once)
        .map(|option| (option.key.as_str(), option.repeat))
        .collect();
    let incense =
        |key: &str| format!("alchemist/incense-synthesizer/incense-fog/incense-fog-{key}");
    let mut expected: HashMap<String, Repeat> = [
        (
            "barbarian/rage-power/increased-damage-reduction",
            Repeat::Times(3),
        ),
        ("barbarian/rage-power/swift-foot", Repeat::Times(3)),
        (
            "stalwart-defender/defensive-powers/increased-damage-reduction",
            Repeat::Times(2),
        ),
        ("alchemist/discovery/preserve-organs", Repeat::Times(3)),
        (
            "shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-new-strain",
            Repeat::Unlimited,
        ),
        // "At 4th level and every four levels thereafter, a Primalist can choose ... two Rage Powers."
        (
            "bloodrager/primalist/primal-choices/rage-powers",
            Repeat::Unlimited,
        ),
    ]
    .into_iter()
    .map(|(key, repeat)| (key.to_owned(), repeat))
    .collect();
    for key in [
        "clarifying-incense",
        "neutralizing-incense",
        "rallying-incense",
        "toxic-incense",
        "unsettling-fog",
    ] {
        expected.insert(incense(key), Repeat::Times(3));
    }
    expected.insert(incense("improved-incense"), Repeat::Times(2));
    // "the Kineticist may choose a second Elemental Focus or increase her primary focus", "there isn't a third
    // level of specialization" (ticket 31).
    for element in ["air", "earth", "fire", "water"] {
        expected.insert(format!("kineticist/element/{element}"), Repeat::Times(2));
    }
    // The user's (ticket 47): the Skill Focus talents once per skill, Combat Trick any number of times.
    expected.insert(
        "kineticist/wild-talents/skill-focus".into(),
        Repeat::PerValue,
    );
    expected.insert("rogue/rogue-talents/skill-focus".into(), Repeat::PerValue);
    expected.insert("rogue/rogue-talents/combat-trick".into(), Repeat::Unlimited);
    // The Battle Scion's teamwork feat in place of a rage power, as often as it likes (ticket 189).
    expected.insert(
        "skald/battle-scion/rage-power-or-teamwork-feat/teamwork-feat".into(),
        Repeat::Unlimited,
    );
    // "She can select a second weapon at 11th level and a third at 19th level" (ticket 172): once per weapon.
    expected.insert("rogue/finesse-training/weapon".into(), Repeat::PerValue);
    let expected: HashMap<&str, Repeat> = expected
        .iter()
        .map(|(key, repeat)| (key.as_str(), *repeat))
        .collect();
    assert_eq!(options, expected);

    // A barbarian takes Increased Damage Reduction three times, not four.
    let mut build = build("barbarian", 16);
    align(&mut build, rules::Alignment::ChaoticNeutral);
    let rage_power_levels: Vec<u8> = (1..=16)
        .filter(|level| {
            engine()
                .evaluate(&build)
                .levels
                .get(usize::from(*level) - 1)
                .is_some_and(|l| {
                    l.slots.iter().any(|slot| {
                        slot.kind
                            == SlotKind::Selection(rules::FeatureSelectionKey::new(
                                "barbarian/rage-power",
                            ))
                    })
                })
        })
        .collect();
    let reduction = || pick("barbarian/rage-power/increased-damage-reduction");
    let later: Vec<u8> = rage_power_levels
        .iter()
        .copied()
        .filter(|level| *level >= 8)
        .take(4)
        .collect();
    assert_eq!(later.len(), 4, "{rage_power_levels:?}");
    for level in &later {
        build.picks.insert(selection_slot(*level), reduction());
    }
    assert_eq!(
        why_not(&build, &selection_slot(later[3])),
        Some(Issue::MaxTimes { times: 3 })
    );
}
