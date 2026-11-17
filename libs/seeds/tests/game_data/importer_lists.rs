//! The importer's option lists, one section per slice (ticket 306): each list a family's selection offers is the
//! game's own, with the keys the old shared list offered beyond it named.

use std::collections::BTreeSet;

use crate::support::*;

/// The selection `key` in the data.
fn selection(key: &str) -> &'static rules::FeatureSelection {
    DATA.selections
        .iter()
        .find(|selection| selection.key.as_str() == key)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

/// What `key` offers, in the rogue talents' own words: the keys with `rogue/rogue-talents/` taken off.
fn talents(key: &str) -> BTreeSet<String> {
    keys(selection(key))
        .into_iter()
        .map(|option| {
            option
                .strip_prefix("rogue/rogue-talents/")
                .unwrap_or_else(|| panic!("{option} is no rogue talent"))
                .to_owned()
        })
        .collect()
}

/// The rogue's own list: what `rogue/rogue-talents` offers.
fn rogues() -> BTreeSet<String> {
    talents("rogue/rogue-talents")
}

/// 306e: the bard's, skald's and Inciter's talents are their own lists, the rogue's less what each lacks.
// Game: the `m_AllFeatures` of BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f (24; the one BardProgression
// 8127f5ff40f5b484b8be98609358b9d2 grants at 2, 6, 10, 14 and 18), SkaldTalentSelection
// d2a8fde8985691045b90e1ec57e3cc57 (21; SkaldProgression at 2, 7, 12 and 17) and ProvocateurTalentSelection
// aa88cb5263a54791b8d1e471bc192ad4 (24; ProvocateurArchetype a1f5ab7eb78942f6889304acbd92e945 `AddFeatures` at 4,
// 9, 14 and 19). Each lacks the rogue's talents listed here: the lists are the rogue's less these.
#[test]
fn the_bard_skald_and_inciter_talents_are_their_own_lists() {
    let rogue = rogues();
    assert_eq!(rogue.len(), 38);
    let lacking = |lists: &[&str]| {
        lists
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>()
    };
    let bard = lacking(&[
        "hunters-surprise",
        "jump-up",
        "mystery-seeker",
        "skill-mastery-knowledge",
        "skill-mastery-lore",
        "skill-mastery-stealth-and-trickery",
        "slippery-mind",
        "survivalist",
        "trailblazer",
        "unbalancing-trick",
        "underhanded-trick",
        "weakening-wound",
        "weapon-snatcher",
        "wearying-strike",
    ]);
    let skald = lacking(&[
        "emboldening-strike",
        "hunters-surprise",
        "improved-uncanny-dodge",
        "jump-up",
        "mystery-seeker",
        "petrifying-strike",
        "skill-mastery-knowledge",
        "skill-mastery-lore",
        "skill-mastery-stealth-and-trickery",
        "slippery-mind",
        "survivalist",
        "trailblazer",
        "unbalancing-trick",
        "uncanny-dodge",
        "underhanded-trick",
        "weapon-snatcher",
        "wearying-strike",
    ]);
    let inciter = lacking(&[
        "hunters-surprise",
        "improved-uncanny-dodge",
        "jump-up",
        "mystery-seeker",
        "skill-mastery-knowledge",
        "skill-mastery-lore",
        "skill-mastery-stealth-and-trickery",
        "slippery-mind",
        "survivalist",
        "trailblazer",
        "unbalancing-trick",
        "uncanny-dodge",
        "underhanded-trick",
        "weapon-snatcher",
    ]);
    for (key, lacks, offers) in [
        ("bard/rogue-talents", bard, 24),
        ("rogue/skald-talents", skald, 21),
        ("skald/inciter/rogue-talents", inciter, 24),
    ] {
        let own = talents(key);
        assert_eq!(own.len(), offers, "{key}");
        assert_eq!(
            own,
            rogue.difference(&lacks).cloned().collect::<BTreeSet<_>>(),
            "{key}"
        );
        // They own none: the rogue's list holds each one.
        assert!(selection(key).options.is_empty(), "{key}");
    }
}

/// 306e: a bard and a skald get no pick from Extra Rogue Talent: the game's feat needs a rogue's, a Sylvan Trickster's or a
/// Dark Lurker's talent selection, and their own are another feature. (306aa reworks the feat's `needs_slot_of`.)
// Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 has PrerequisiteFeature (Group Any) of
// RogueTalentSelection c074a5d615200494b8f2a9c845799d93, SylvanTricksterTalentSelection
// 290bbcc3c3bb92144b853fd8fb8ff452 and DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2; BardTalentSelection
// 94e2cd84bf3a8e04f8609fe502892f4f and SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57 are none of them.
#[test]
fn a_bard_and_a_skald_cant_take_extra_rogue_talent() {
    let with_extra = |class: &str| {
        let mut build = with_selections_filled(classes_build(&[class, class, class]));
        build
            .picks
            .insert(general_slot(3), feat_pick_plain("extra-rogue-talent"));
        with_pick(build, extra_slot(3, 0), &rogue_talent("fast-stealth"))
    };
    // A rogue's pick in the feat's slot stands; a bard's or a skald's is refused (the planner still lets them take
    // the feat itself: its prerequisite reads the talents held, which 306aa/S4 write as the game's).
    assert!(possible(&with_extra("rogue")));
    for class in ["bard", "skald"] {
        assert!(!possible(&with_extra(class)), "{class}");
    }
}

/// 306f: the Dark Lurker's talent list is the rogue's 38 and Evasion, at the seven levels the archetype adds it; a
/// plain rogue's list has no Evasion.
// Game: the `m_AllFeatures` of DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2 (39: RogueTalentSelection
// c074a5d615200494b8f2a9c845799d93's 38 and Evasion 576933720c440aa4d8d42b0c54b77e80); DarkLurkerArchetype
// 33eb002078e5437b8c25db4b80b4a4fe `AddFeatures` it at 4, 6, 10, 12, 16, 18 and 20 and `RemoveFeatures` Evasion and the
// rogue's selection at 2
#[test]
fn the_dark_lurker_offers_its_own_list_with_evasion() {
    let rogue = rogues();
    let mut expected = rogue.clone();
    expected.insert("evasion".to_owned());
    assert_eq!(expected.len(), 39);
    assert_eq!(talents("rogue/dark-lurker/rogue-talents"), expected);
    assert!(!rogue.contains("evasion"));

    let lurker = archetype_build("rogue", "rogue/dark-lurker", 20);
    for level in [4, 6, 10, 12, 16, 18, 20] {
        assert_eq!(
            offered(&lurker, level)["rogue/rogue-talents/evasion"],
            Ok(()),
            "level {level}"
        );
    }
    // The levels the archetype takes the rogue's slot away (2, 8, 14) hold no talent slot at all.
    let rows = |level: u8| {
        DATA.archetypes
            .iter()
            .find(|archetype| archetype.key.as_str() == "rogue/dark-lurker")
            .and_then(|archetype| archetype.levels.iter().find(|row| row.level == level))
            .map(|row| row.selections.len())
    };
    for level in [2, 8, 14] {
        assert_eq!(rows(level), Some(0), "level {level}");
    }
    for level in [4, 6, 10, 12, 16, 18, 20] {
        assert_eq!(rows(level), Some(1), "level {level}");
    }
    let plain = build("rogue", 20);
    assert!(!offered(&plain, 4).contains_key("rogue/rogue-talents/evasion"));
}

/// 306f: the Master of All's Skill Focus no longer ignores prerequisites: the game's selection doesn't.
// Game: MasterOfAllSkillFocus f2d2c1702d8a4cc6adfcbd4ebff8eee4 has `IgnorePrerequisites` false
#[test]
fn the_master_of_alls_skill_focus_checks_prerequisites() {
    assert!(!selection("rogue/master-of-all/skill-focus").ignores_prerequisites);
}

/// 306g: the slayer's talent lists at 2, 6 and 10 are the game's three, each with Uncanny Dodge and its improved form.
// Game: the `m_AllFeatures` of SlayerTalentSelection2 04430ad24988baa4daa0bcd4f1c7d118 (37), SlayerTalentSelection6
// 43d1b15873e926848be2abf0ea3ad9a8 (38) and SlayerTalentSelection10 913b9cf25c9536949b43a2651b7ffb66 (39); each lists
// UncannyDodgeTalent ca5274d057152fa45b7527cad0927840 and ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8
// (the user's fact, ticket 276: apply the game's), ExecutionerPainfulStrike 052c82521039f7b4e978207eb533ead5 and
// VanguardTacticianExtraUseTalent c67db0b6ede802b43a6cf00136b1a5e6
#[test]
fn the_slayers_three_lists_are_the_games() {
    let lists = [
        ("slayer/slayer-talents", 37),
        ("slayer/slayer-talents-6", 38),
        ("slayer/slayer-talents-10", 39),
    ];
    for (key, count) in lists {
        let offers = keys(selection(key));
        assert_eq!(offers.len(), count, "{key}");
        for talent in [
            "rogue/rogue-talents/uncanny-dodge",
            "rogue/rogue-talents/improved-uncanny-dodge",
            "slayer/slayer-talents/painful-strike",
            "slayer/slayer-talents/vanguard-tactician",
        ] {
            assert!(offers.contains(talent), "{key}: {talent}");
        }
    }
}

/// 306g: Extra Slayer Talent's own list is the game's 41, and holds Uncanny Dodge and its improved form.
// Game: the `m_AllFeatures` of ExtraSlayerTalentSelection 53f7237f5b1447bb851ba68045a00e41 (41, UncannyDodgeTalent
// ca5274d057152fa45b7527cad0927840 and ImprovedUncannyDodgeTalent e821c61b2711cea4cb993725b910e7e8 among them; ticket
// 276, applied)
#[test]
fn the_slayers_extra_talent_list_is_the_games_41() {
    let offers = keys(selection("slayer/extra-slayer-talent"));
    assert_eq!(offers.len(), 41);
    for talent in ["uncanny-dodge", "improved-uncanny-dodge"] {
        assert!(offers.contains(&format!("rogue/rogue-talents/{talent}")));
    }
}

/// 306h: the skald's and the Instinctual Warrior's rage powers are their own lists; the barbarian's has 33, and Extra
/// Rage Power's is the game's 38.
// Game: the `m_AllFeatures` of RagePowerSelection 28710502f46848d48b3f0d6132817c4e (33), SkaldRagePowerSelection
// 2476514e31791394fa140f1a07941c96 (33; SkaldProgression 26418fed2bc153245972a5b54204ed75 at 3, 6, 9, 12, 15, 18),
// InstinctualWarriorRagePowerSelection 609f0e5336084442a0dafa3abd4d31c5 (38; InstinctualWarriorArchetype
// adffdd8a99094a89823a79292a503ee9 `AddFeatures` at 4, 8, 10, 12, 16, 18, 20) and ExtraRagePowerSelection
// 0c7f01fbbe687bb4baff8195cb02fe6a (38). The five monk powers are RageDeflectArrowsFeature
// d971bbb15f354621ac37fd51d0ab9044 and its four siblings, which only the Instinctual Warrior's list and Extra Rage
// Power's hold.
#[test]
fn the_skald_and_the_instinctual_warrior_have_their_own_rage_powers() {
    let monk_powers = [
        "barbarian/rage-power/deflect-arrows-instinctual-warrior",
        "barbarian/rage-power/diamond-body-instinctual-warrior",
        "barbarian/rage-power/diamond-soul-instinctual-warrior",
        "barbarian/rage-power/perfect-self-instinctual-warrior",
        "barbarian/rage-power/purity-of-body-instinctual-warrior",
    ];
    let lists = [
        ("barbarian/rage-power", 33, false),
        ("barbarian/skald-rage-power", 33, false),
        ("barbarian/instinctual-warrior/rage-power", 38, true),
        ("barbarian/extra-rage-power", 38, true),
    ];
    for (key, count, has_monk_powers) in lists {
        let offers = keys(selection(key));
        assert_eq!(offers.len(), count, "{key}");
        for power in monk_powers {
            assert_eq!(offers.contains(power), has_monk_powers, "{key}: {power}");
        }
        for power in [
            "barbarian/rage-power/clear-mind",
            "barbarian/rage-power/come-and-get-me",
        ] {
            assert!(offers.contains(power), "{key}: {power}");
        }
    }
}

/// 306i: the Exploiter Wizard's exploits are their own selection, and the arcanist's, the Exploiter's and Extra
/// Arcanist Exploit's lists are the game's 22: the sixteen exploits and the six greater ones, which stay the Greater
/// Exploits' own options and join the others as shared ones. Extra Arcana's is the 15 magus arcana, Extra Mercy's the
/// paladin's 15 mercies.
// Game: the `m_AllFeatures` of ArcanistExploitSelection b8bf3d5023f2d8c428fdf6438cecaea7 (22),
// ExploiterExploitSelection 2ba8a0040e0149e9ae9bfcb01a8ff01d (22; ExploiterWizardArchetype
// 6d2027b4eec590c41adb5caeb7bb026d `AddFeatures` at 1, 5, 9, 13, 17), ExtraArcanistExploitSelection
// 0a7065e13f4449fa81bfb33199bf7f6a (22), ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 (15) and
// ExtraMercySelection 8a49abed5be9473da1e1fd1e2457562e (15). The six greater exploits: ArcanistExploitBurningFlameFeature
// 9677876c5dbb6074780c392e68f5929c, DancingElectricity 72eb9635…, GreaterSpellResistance 1eba9b43…, HellfireRay 7d563761…,
// IcyTomb 0e7d6f93…, LingeringAcid 479ab600….
#[test]
fn the_exploiters_and_extra_arcanist_exploits_lists_are_the_games() {
    let greater = keys(selection("arcanist/greater-exploits"));
    assert_eq!(greater.len(), 6);
    for list in [
        "arcanist/arcanist-exploits",
        "arcanist/exploiter-exploits",
        "arcanist/extra-arcanist-exploit",
    ] {
        let offers = keys(selection(list));
        assert_eq!(offers.len(), 22, "{list}");
        assert!(greater.is_subset(&offers), "{list} lacks a greater exploit");
    }
    assert_eq!(keys(selection("magus/extra-arcana")).len(), 15);
    assert_eq!(keys(selection("paladin/extra-mercy")).len(), 15);
    // The Exploiter Wizard's rows name its own selection, in the slot order the arcanist's key had.
    let exploiter = DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key.as_str() == "wizard/exploiter-wizard")
        .expect("exploiter wizard");
    for level in [1, 5, 9, 13, 17] {
        let row = exploiter
            .levels
            .iter()
            .find(|row| row.level == level)
            .unwrap_or_else(|| panic!("no row {level}"));
        let held = row
            .selections
            .iter()
            .any(|key| key.as_str() == "arcanist/exploiter-exploits");
        assert!(held, "{level}");
    }
}
