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

/// 306e: a bard and a skald can't take Extra Rogue Talent: the game's feat needs a rogue's, a Sylvan Trickster's or a
/// Dark Lurker's talent selection (or the Feyform Shifter's, which is the rogue's blueprint), and their own are
/// another feature. The refusal is the feat's prerequisite (306aa: the slot is the feat's own for any class that holds
/// it).
// Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 has PrerequisiteFeature (Group Any) of
// RogueTalentSelection c074a5d615200494b8f2a9c845799d93, SylvanTricksterTalentSelection
// 290bbcc3c3bb92144b853fd8fb8ff452 and DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2; BardTalentSelection
// 94e2cd84bf3a8e04f8609fe502892f4f and SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57 are none of them.
#[test]
fn a_bard_and_a_skald_cant_take_extra_rogue_talent() {
    for class in ["bard", "skald"] {
        let build = with_selections_filled(classes_build(&[class, class, class]));
        assert!(
            feat_status(&build, &general_slot(3), "extra-rogue-talent").is_err(),
            "{class}"
        );
    }
}

/// 306aa: a rogue's pick in Extra Rogue Talent's slot stands, and the slot is the feat's own selection's, the rogue's
/// talents and Evasion; a bard is refused the feat, so has no such slot.
// Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 (RogueTalentSelection
// c074a5d615200494b8f2a9c845799d93's 38 and Evasion 576933720c440aa4d8d42b0c54b77e80; the prerequisites as in
// `a_bard_and_a_skald_cant_take_extra_rogue_talent`); `SelectFeature.Apply`, WrathDecompiled 6ce4a6b, adds the picked
// selection with its own list.
#[test]
fn a_pick_in_extra_rogue_talents_slot_stands_for_a_rogue_and_a_bard_is_refused_the_feat() {
    let mut rogue = with_selections_filled(classes_build(&["rogue", "rogue", "rogue"]));
    assert_eq!(
        feat_status(&rogue, &general_slot(3), "extra-rogue-talent"),
        Ok(())
    );
    rogue
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rogue-talent"));
    assert!(possible(&with_pick(
        rogue,
        extra_slot(3, 0),
        &rogue_talent("fast-stealth")
    )));

    let mut bard = with_selections_filled(classes_build(&["bard", "bard", "bard"]));
    assert!(feat_status(&bard, &general_slot(3), "extra-rogue-talent").is_err());
    bard.picks
        .insert(general_slot(3), feat_pick_plain("extra-rogue-talent"));
    assert!(!possible(&with_pick(
        bard,
        extra_slot(3, 0),
        &rogue_talent("fast-stealth")
    )));
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

/// 306j: the shaman's hexes are its own 34 (the eleven it shares with the witch are copies with the shaman's blueprints,
/// each naming the witch's key as an alias so an old link opens), the witch's and Hexcrafter's lists are the game's 27
/// with the Hex Channeler's channel, and the shaman's Extra Hex is its own feat.
// Game: ShamanHexSelection 4223fe18c75d4d14787af196a04e14e7 (34; the shaman's copies ShamanHexAmelioratingFeature
// 8e19da021854b9b4e805f0e919663894, AuraOfPurity 558e9265…, EvilEye 890cd930…, Fortune 64ed6d1a…, Healing 3fb96b69…,
// Iceplant 999e367b…, Misfortune e7fe4ed2…, Nails e87b3db8…, ProtectiveLuck f0db7409…, Slumber 894ef737…, Ward
// 41f4d078…); WitchHexSelection 9846043cf51251a4897728ed6e24e76f (27, with HexChannelerChannelFeature
// b40316f05d4772e4894688e6743602bd); HexcrafterMagusHexMagusSelection a18b8c3d6251d8641a8094e5c2a7bc78 (27, Hexcrafter
// at 4); ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35 (27); ExtraShamanHexSelection
// d0b4c8245d504b8c9c6d3fccc1f8c5b6 (34); SylvanTricksterTalentSelection 290bbcc3c3bb92144b853fd8fb8ff452 (64).
#[test]
fn the_shamans_hexes_are_their_own_and_open_old_links_through_aliases() {
    let shaman = selection("shaman/hex");
    assert_eq!(keys(shaman).len(), 34);
    assert!(shaman.shared_options.is_empty());
    assert!(
        keys(shaman)
            .iter()
            .all(|key| key.starts_with("shaman/hex/"))
    );
    for name in [
        "ameliorating",
        "aura-of-purity",
        "evil-eye",
        "fortune",
        "healing-hex",
        "iceplant",
        "misfortune",
        "nails",
        "protective-luck",
        "slumber",
        "ward",
    ] {
        let copy = DATA
            .options
            .iter()
            .find(|option| option.key.as_str() == format!("shaman/hex/{name}"))
            .unwrap_or_else(|| panic!("no shaman/hex/{name}"));
        let heir = rules::SelectionOptionKey::new(format!("witch/hex/{name}"));
        assert!(copy.aliases.contains(&heir), "{name}");
    }
    let witch = keys(selection("witch/hex"));
    assert_eq!(witch.len(), 27);
    assert!(witch.contains("witch/hex/channel"));
    assert_eq!(keys(selection("magus/hexcrafter/hex")), witch);
    assert_eq!(keys(selection("shaman/extra-hex")), keys(shaman));
    assert!(
        !keys(selection("witch/extra-hex"))
            .iter()
            .any(|key| key.starts_with("shaman/hex/"))
    );
    let feat = DATA
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "extra-hex-shaman")
        .expect("extra-hex-shaman");
    assert_eq!(
        feat.selection.as_ref().map(|key| key.as_str()),
        Some("shaman/extra-hex")
    );
    // A shaman is offered its own copy and not the witch's.
    let offers = offered(&build("shaman", 2), 2);
    assert_eq!(offers["shaman/hex/evil-eye"], Ok(()));
    assert!(!offers.contains_key("witch/hex/evil-eye"));
    // The Hexcrafter's row 4 names its own selection.
    let hexcrafter = DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key.as_str() == "magus/hexcrafter")
        .expect("hexcrafter");
    let row = hexcrafter
        .levels
        .iter()
        .find(|row| row.level == 4)
        .expect("row 4");
    assert!(
        row.selections
            .iter()
            .any(|key| key.as_str() == "magus/hexcrafter/hex")
    );
}

/// The blueprint GUIDs of an option list's options, sorted.
fn guids(key: &str) -> Vec<String> {
    let mut found: Vec<String> = keys(selection(key))
        .into_iter()
        .map(|option| {
            DATA.options
                .iter()
                .find(|candidate| candidate.key.as_str() == option)
                .unwrap_or_else(|| panic!("no option {option}"))
                .blueprint
                .as_ref()
                .unwrap_or_else(|| panic!("{option} has no blueprint"))
                .as_str()
                .to_owned()
        })
        .collect();
    found.sort();
    found
}

/// A sorted list of GUIDs, for comparing a list with the game's.
fn sorted(guids: &[&str]) -> Vec<String> {
    let mut list: Vec<String> = guids.iter().map(ToString::to_string).collect();
    list.sort();
    list
}

/// 306k: the Aeon's three gaze lists are the game's, one per tier of its progression: ranks 3 and 4 take the first,
/// 5 to 7 the second, 8 to 10 the third.
// Game: the `m_Features` of AeonGazeFirstSelection 7e468a5266235454e9dcefee315ee6d5 (8), AeonGazeSecondSelection
// 7d498e034d18ca94baea19d7edee7403 (16) and AeonGazeThirdSelection 0bec49f67ecb49a5826fcfefb9408a35 (17); AeonProgression
// 34b9484b0d5ce9340ae51d2bf9518bbe grants the first at its levels 1 and 2, the second at 3, 4 and 5, the third at 6, 7
// and 8 (Aeon level 1 is mythic rank 3). The second adds HP 2eae…, concentration, summoner's gaze 22b8e950…,
// relativity of space, primordial magic, normalize size, area effects and power of law to the first's eight; the third
// adds the temporal jump 72514d0dcd054d1f8bb9abd4ddb9711c.
#[test]
fn the_aeons_three_gaze_lists_are_the_games() {
    let first = [
        "f102b11d85a11d14c811569c0ffcaa6e",
        "7d3707bec182f5d428c444d7b7ead812",
        "911be6cfed6bb71449cbf6d1173f4dc2",
        "741cde744a64425e9feb6b1eaef53eba",
        "dbabd88e7cd24b619dbf413ae7ce0033",
        "bb283854ec4242babbf0463bc3050264",
        "653831e19515438b948764291689dc3a",
        "e0cc4134510b074468b3d1185970c697",
    ];
    let second_only = [
        "bb6ac47cdff935d4e8ebf3ba8c5fdcda",
        "2eae9b2c25a969641a2cf230acdca478",
        "22b8e9508bce5474588e34168089ad69",
        "24856f367261454b8d5070da0e774053",
        "da5cbbd3b9a14a86b90555e7d3ed4451",
        "c90b37a7f4894ac5a27f506aba5760b5",
        "39d8ae51c0034adbb8394210c51901e2",
        "8c84573701184f6db5b7b7543ee64227",
    ];
    let second: Vec<&str> = first.iter().chain(&second_only).copied().collect();
    let mut third = second.clone();
    third.push("72514d0dcd054d1f8bb9abd4ddb9711c");
    assert_eq!(guids("aeon/enforcing-gaze"), sorted(&first));
    assert_eq!(guids("aeon/enforcing-gaze-2"), sorted(&second));
    assert_eq!(guids("aeon/enforcing-gaze-3"), sorted(&third));
    let path = DATA
        .mythic_paths
        .iter()
        .find(|path| path.key.as_str() == "aeon")
        .expect("aeon");
    for (rank, selection) in [
        (3, "aeon/enforcing-gaze"),
        (4, "aeon/enforcing-gaze"),
        (5, "aeon/enforcing-gaze-2"),
        (6, "aeon/enforcing-gaze-2"),
        (7, "aeon/enforcing-gaze-2"),
        (8, "aeon/enforcing-gaze-3"),
        (9, "aeon/enforcing-gaze-3"),
        (10, "aeon/enforcing-gaze-3"),
    ] {
        let row = path
            .ranks
            .iter()
            .find(|row| row.rank == rank)
            .expect("rank");
        assert_eq!(
            row.selections
                .iter()
                .map(|key| key.as_str())
                .collect::<Vec<_>>(),
            [selection],
            "rank {rank}"
        );
    }
    // What each rank's picker offers: the tier's list, the temporal jump from rank 8.
    let build = mythic("aeon", 20);
    let area_effects = "aeon/enforcing-gaze/enforcing-gaze-area-effects";
    let temporal_jump = "aeon/enforcing-gaze/enforcing-gaze-temporal-jump";
    assert_eq!(offered_at(&build, &path_selection(4)).len(), 8);
    assert!(!offered_at(&build, &path_selection(4)).contains(area_effects));
    assert_eq!(offered_at(&build, &path_selection(5)).len(), 16);
    assert!(offered_at(&build, &path_selection(5)).contains(area_effects));
    assert!(!offered_at(&build, &path_selection(7)).contains(temporal_jump));
    assert_eq!(offered_at(&build, &path_selection(8)).len(), 17);
    assert!(offered_at(&build, &path_selection(10)).contains(temporal_jump));
    // A Devil that was an Aeon loses the gaze picks of every tier.
    let devil = DATA
        .mythic_paths
        .iter()
        .find(|path| path.key.as_str() == "devil")
        .expect("devil");
    let dropped: Vec<&str> = devil
        .switch
        .as_ref()
        .expect("a switch rule")
        .dropped_selections
        .iter()
        .map(|key| key.as_str())
        .collect();
    assert_eq!(
        dropped,
        [
            "aeon/enforcing-gaze",
            "aeon/enforcing-gaze-2",
            "aeon/enforcing-gaze-3"
        ]
    );
}

/// 306k: an Elemental Witch picks one of four rays at 1, and the sheet keeps the Elemental Ray feature.
// Game: ElementalWitchArchetype d7c7cdbc760041fabaedc1623a53a431 `AddFeatures` ElementaWitchEnergySelection
// 9ec1bf2087874390a9d9be62884a0027 at 1 (its `m_Features`: ElementalWitchAcidRayFeature
// 8006c9d043724f34b2745b3e5a5c2ef1, ColdRay 8cb71f2cc612478ea00b03d180b9e22c, ElectricityRay
// 1c630250f9dd41a784ec0da1f1361615, FireRay 97e4fe7723b749849f182186764c23a4) and ElementaWitchEnergySelection11
// edbb5084c34a486fb39d38cc75f2f994, a feature, at 11.
#[test]
fn the_elemental_witch_picks_one_of_four_rays() {
    let energy = selection("witch/elemental-witch/energy");
    assert_eq!(
        energy.blueprint.as_ref().map(|guid| guid.as_str()),
        Some("9ec1bf2087874390a9d9be62884a0027")
    );
    assert_eq!(
        guids("witch/elemental-witch/energy"),
        sorted(&[
            "8006c9d043724f34b2745b3e5a5c2ef1",
            "8cb71f2cc612478ea00b03d180b9e22c",
            "1c630250f9dd41a784ec0da1f1361615",
            "97e4fe7723b749849f182186764c23a4",
        ])
    );
    let build = archetype_build("witch", "witch/elemental-witch", 11);
    let offers = offered(&build, 1);
    assert_eq!(offers.len(), 4);
    assert!(offers.values().all(|status| status.is_ok()));
    let picked = with_pick(
        build,
        selection_slot(1),
        "witch/elemental-witch/energy/fire",
    );
    assert!(possible(&picked));
    assert!(holds(
        &at(&picked, 1),
        "witch/elemental-witch/elemental-ray"
    ));
    // The row names the selection in place of the feature, which the 11th level's row still grants.
    let archetype = DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key.as_str() == "witch/elemental-witch")
        .expect("elemental witch");
    let row = |level: u8| {
        archetype
            .levels
            .iter()
            .find(|row| row.level == level)
            .unwrap()
    };
    assert_eq!(
        row(1)
            .selections
            .iter()
            .map(|key| key.as_str())
            .collect::<Vec<_>>(),
        ["witch/elemental-witch/energy"]
    );
    assert!(
        !row(1)
            .features
            .iter()
            .any(|key| key.as_str() == "witch/elemental-witch/elemental-ray")
    );
    assert!(
        row(11)
            .features
            .iter()
            .any(|key| key.as_str() == "witch/elemental-witch/elemental-ray")
    );
}

/// 306k: the Nine-Tailed Heir's bloodline is its own selection with the sorcerer's 21 bloodlines, the Extra Discovery
/// of a Vivisectionist its own feat, and the Greater Sword of Heaven lists the game's 12.
// Game: NineTailedHeirBloodlineSelection 7c813fb495d74246918a690ba86f9c86 (21 `m_AllFeatures`, the sorcerer's
// SorcererBloodlineSelection 24bef8d1bee12274686f6da6ccbc8914's; NineTailedHeirArchetype
// 65a630aa291f65047b90a2af5df75d83 `AddFeatures` it at 1 and `RemoveFeatures` the sorcerer's);
// ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae (25, needs VivsectionistDiscoverySelection
// 67f499218a0e22944abab6fe1c9eaeee); ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66 (28, needs
// DiscoverySelection cd86c437488386f438dcc9ae727ea2a6); AngelSwordAntiDemonFeature 249ff73e41a85b84eabbf254870a7499
// is in AngelGreaterSwordOfHeavenSelection e0ce40968bf0007408b11089a10f36cf (12 `m_AllFeatures`).
#[test]
fn the_heir_the_vivisectionists_extra_discovery_and_the_angel_have_the_games_lists() {
    assert_eq!(
        keys(selection("sorcerer/nine-tailed-heir/bloodline")),
        keys(selection("sorcerer/bloodline"))
    );
    assert_eq!(keys(selection("sorcerer/bloodline")).len(), 21);
    let heir = DATA
        .archetypes
        .iter()
        .find(|archetype| archetype.key.as_str() == "sorcerer/nine-tailed-heir")
        .expect("heir");
    let row = heir.levels.iter().find(|row| row.level == 1).unwrap();
    assert!(
        row.selections
            .iter()
            .any(|key| key.as_str() == "sorcerer/nine-tailed-heir/bloodline")
    );
    assert!(
        !row.selections
            .iter()
            .any(|key| key.as_str() == "sorcerer/bloodline")
    );

    let extra = feat("extra-discovery-vivisectionist");
    assert_eq!(
        extra.blueprint.as_ref().map(|guid| guid.as_str()),
        Some("10287e7b8cee479e82ea88bd6d2d4dae")
    );
    assert_eq!(
        extra.selection.as_ref().map(|key| key.as_str()),
        Some("alchemist/vivisectionist/extra-discovery")
    );
    assert_eq!(
        keys(selection("alchemist/vivisectionist/extra-discovery")).len(),
        25
    );
    assert_eq!(keys(selection("alchemist/extra-discovery")).len(), 28);
    assert!(
        !keys(selection("alchemist/extra-discovery"))
            .iter()
            .any(|key| key.starts_with("alchemist/vivisectionist/medical-discovery/"))
    );

    let greater = guids("angel/greater-sword-of-heaven");
    assert_eq!(
        greater,
        sorted(&[
            "ebcda8f579e7d494a8befda94279968f",
            "249ff73e41a85b84eabbf254870a7499",
            "f02537712cd2af34c93ce979599f7b12",
            "b35e4f149f17d924c97fbd27e30e948b",
            "317c795278362f84581f7cf35023a19f",
            "432c96835aee50341a4ff0b364cdb85a",
            "0b082785884d35c4c8f252d93b8f3616",
            "131fc914b66b62c49b9f9b5f750998f5",
            "3b27bebe971f9484f93f72e3d05182ee",
            "84f001f9aaaa73c479659edfa87f4782",
            "477fcaa2b00a1964da43472ddbc3c7de",
            "826e402bae1aca242bfe9919da8d94d7",
        ])
    );
    assert!(
        keys(selection("angel/greater-sword-of-heaven"))
            .contains("angel/greater-sword-of-heaven/anti-demon")
    );
}

fn levels_naming<'a>(
    rows: impl IntoIterator<Item = (u8, &'a [rules::FeatureSelectionKey])>,
    key: &str,
) -> Vec<u8> {
    rows.into_iter()
        .filter(|(_, names)| names.iter().any(|name| name.as_str() == key))
        .map(|(level, _)| level)
        .collect()
}

/// The feats a feat list holds.
fn feat_keys(list: &rules::FeatureSelection) -> BTreeSet<String> {
    list.feats.iter().map(|feat| feat.to_string()).collect()
}

fn class_rows(key: &str) -> Vec<(u8, &'static [rules::FeatureSelectionKey])> {
    class(key)
        .levels
        .iter()
        .map(|row| (row.level, row.selections.as_slice()))
        .collect()
}

fn archetype_rows(key: &str) -> Vec<(u8, &'static [rules::FeatureSelectionKey])> {
    DATA.archetypes
        .iter()
        .find(|archetype| archetype.key.as_str() == key)
        .unwrap_or_else(|| panic!("no archetype {key}"))
        .levels
        .iter()
        .map(|row| (row.level, row.selections.as_slice()))
        .collect()
}

/// 306l: the magus's, the Arcane Rider's and the warpriest's bonus feats are three lists of their own, each the game's
/// full member set (306q and 306r added the four feats the planner had no entry for).
// Game: the `m_AllFeatures` of MagusFeatSelection 66befe7b24c42dd458952e3c47c93563 (181), ArcaneRiderFeatSelection
// 8e627812dc034b9db12fa396fdc9ec75 (182 entries, one listed twice: the magus's 181) and
// WarpriestFeatSelection 303fd456ddb14437946e344bad9a893b (155; the magus's less its spell and metamagic feats), each
// with `IgnorePrerequisites` false. MagusProgression 5, 11 and 17 grant the first; ArcaneRiderArchetype
// 2dff80a045794cb9b5452fe7b66b79d7 `AddFeatures` the second at 5, 11 and 17 and `RemoveFeatures` the first at those
// levels; WarpriestProgression 062b215705430ad47b7174c1e75d285e grants the third at 3, 6, 9, 12, 15 and 18.
#[test]
fn the_magus_arcane_rider_and_warpriest_bonus_feats_are_their_own() {
    let list = |key: &str| {
        let list = selection(key);
        assert!(!list.ignores_prerequisites, "{key} checks prerequisites");
        feat_keys(list)
    };
    let magus = list("magus/bonus-feats");
    let rider = list("magus/arcane-rider/bonus-feats");
    let warpriest = list("warpriest/bonus-feats");
    assert_eq!(magus.len(), 181);
    assert_eq!(rider.len(), 182 - 1);
    assert_eq!(warpriest.len(), 155);
    for (key, guid) in [
        ("magus/bonus-feats", "66befe7b24c42dd458952e3c47c93563"),
        (
            "magus/arcane-rider/bonus-feats",
            "8e627812dc034b9db12fa396fdc9ec75",
        ),
        ("warpriest/bonus-feats", "303fd456ddb14437946e344bad9a893b"),
    ] {
        assert_eq!(
            selection(key).blueprint.as_ref().map(|guid| guid.as_str()),
            Some(guid)
        );
    }
    // The rider's list is the magus's: the game lists TricksterImprovedImprovedImprovedCriticalImproved
    // 319c882ab3cc51544ad2f3f43633d5b1 twice.
    assert_eq!(magus, rider);
    // The warpriest's has none of the magus's spell and metamagic feats; the magus's has Spell Penetration and Combat
    // Casting (MagusFeatSelection `m_AllFeatures`: SpellPenetration ee7dc126939e4d9438357fbd5980d459, CombatCasting
    // 06964d468fde1dc4aa71a92ea04d930d).
    for key in [
        "spell-penetration",
        "combat-casting",
        "metamagic-quicken-spell",
    ] {
        assert!(magus.contains(key), "{key} is in the magus's list");
        assert!(!warpriest.contains(key), "{key} is not in the warpriest's");
    }
    assert!(warpriest.is_subset(&magus));
    // Neither is the general list: Alertness (not in any of the three) is a feat, not a bonus feat.
    assert!(!magus.contains("alertness") && !warpriest.contains("alertness"));

    let magus_levels = [5, 11, 17];
    assert_eq!(
        levels_naming(class_rows("magus"), "magus/bonus-feats"),
        magus_levels
    );
    assert_eq!(
        levels_naming(class_rows("warpriest"), "warpriest/bonus-feats"),
        [3, 6, 9, 12, 15, 18]
    );
    assert_eq!(
        levels_naming(class_rows("magus"), "feats"),
        Vec::<u8>::new()
    );
    assert_eq!(
        levels_naming(class_rows("warpriest"), "combat-feats"),
        Vec::<u8>::new()
    );
    // ArcaneRiderArchetype's own list at 5, 11 and 17; the other archetypes that replace the magus's row keep the magus's
    // list (BladeboundArchetype 9b3915d3… adds MagusFeatSelection at 17; the rest keep what MagusProgression gives).
    assert_eq!(
        levels_naming(
            archetype_rows("magus/arcane-rider"),
            "magus/arcane-rider/bonus-feats"
        ),
        magus_levels
    );
    for (key, levels) in [
        ("magus/armored-battlemage", vec![5, 17]),
        ("magus/bladebound", vec![17]),
        ("magus/eldritch-scion", vec![11]),
        ("magus/hexcrafter", vec![11]),
        ("magus/spell-dancer", vec![17]),
        ("magus/sword-saint", vec![11]),
    ] {
        assert_eq!(
            levels_naming(archetype_rows(key), "magus/bonus-feats"),
            levels,
            "{key}"
        );
    }
    for (key, levels) in [
        ("warpriest/champion-of-the-faith", vec![12]),
        ("warpriest/cult-leader", vec![6, 18]),
        ("warpriest/feral-champion", vec![9, 15]),
        ("warpriest/mantis-zealot", vec![18]),
        ("warpriest/shieldbearer", vec![12]),
    ] {
        assert_eq!(
            levels_naming(archetype_rows(key), "warpriest/bonus-feats"),
            levels,
            "{key}"
        );
    }
}

// Game: the `m_AllFeatures` of CavalierBonusFeatSelection dd17090d14958ef48ba601688b611970 (146 entries, among them
// AdvancedWeaponTraining2 70a139f0a4c6c534eaa34feea0d08622, 3 ee9ab0117ca06b84f9c66469f4428c61, 4
// 0b55d725ded1ae549bb858fba1d84114 and BleedingCriticalFeature 42be7d465db94213baf0578c721803ff), GendarmeFeatSelection
// ef1cd58e0b7fc7f45baedb09407a1cd1 (155) and CavalierTacticianFeatSelection 7bc55b5e381358c45b42153b8b2603a6 (12). Each
// GUID is a member the planner's feat of that `blueprint` stands for.
const CAVALIER_BONUS_FEATS: [&str; 146] = [
    "006a966007802a0478c9e21007207aac",
    "055fb8dcaf6be334ca13172bafa9e782",
    "05a3b543b0a0a0346a5061e90f293f0b",
    "0641368c74334671aaaf32a2730587be",
    "09c9e82965fb4334b984a1e9df3bd088",
    "0ab2f21a922feee4dab116238e3150b4",
    "0b442a7b4aa598d4e912a4ecee0500ff",
    "0b55d725ded1ae549bb858fba1d84114",
    "0c17102f650d9044290922b0fad9132f",
    "0da0c194d6e1d43419eb8d990b28e0ab",
    "0f15c6f70d8fb2b49aa6cc24239cc5fa",
    "0f8939ae6f220984e8fb568abbdfba95",
    "0fc1ed8532168f74a9441bd17ad59e66",
    "1191ef3065e6f8e4f9fbe1b7e3c0f760",
    "121811173a614534e8720d7550aae253",
    "12c1b556df3b667458ae200bfb38ccb8",
    "14a1fc1356df9f146900e1e42142fc9d",
    "197306972c98bb843af738dc7529a7ac",
    "1a0298abacb6e0f45b7e28553e99e76c",
    "1b0f68188dcc435429fb87a022239681",
    "1e1f627d26ad36f43bbd26cc2bf8ac7e",
    "1fc8c07a82d749a1a35b6992ad359788",
    "203992ef5b35c864390b4e4a1e200629",
    "204a749f60f443e29fbb9e8f6da0a14d",
    "25bc9c439ac44fd44ac3b1e58890916f",
    "2a6091b97ad940943b46262600eaeaeb",
    "2a681cb9fcaab664286cb36fff761245",
    "2c61fdbf242866f4e93c3e1477fb96b5",
    "308cd7dc4f10efd428f531bbf4f2823d",
    "31470b17e8446ae4ea0dacd6c5817d86",
    "319c882ab3cc51544ad2f3f43633d5b1",
    "3aa4cbdd4af5ba54888b0dc7f07f80c4",
    "3af3476f6034a924e96f3ffa198424b7",
    "4153755355a0b9b4e956c9ca232c22cf",
    "422dab7309e1ad343935f33a4d6e9f11",
    "42be7d465db94213baf0578c721803ff",
    "453f5181a5ed3a445abfa3bcd3f4ac0c",
    "46f4fb320f35704488ba3d513397789d",
    "46f970a6b9b5d2346b10892673fe6e74",
    "479c7f3b0dba69a4bbcb43e101f3f7f9",
    "47b352ea0f73c354aba777945760b441",
    "4c44724ffa8844f4d9bedb5bb27d144a",
    "4c7205d859a1e114895e798af383d76a",
    "4cc71ae82bdd85b40b3cfe6697bb7949",
    "4e219f5894ad0ea4daa0699e28c37b1d",
    "4f1a78b02ac71bd4fa7d6e011d6f8ce0",
    "52913092cd018da47845f36e6fbe240f",
    "52c6b07a68940af41b270b3710682dc7",
    "54d824028117e884a8f9356c7c66149b",
    "54ee847996c25cd4ba8773d7b8555174",
    "5662d1b793db90c4b9ba68037fd2a768",
    "56f94badbba018b4b8277ce6e2e79e72",
    "57c3ea6b05da4226aaeeb21da04e82ea",
    "59bd93899149fa44687ff4121389b3a9",
    "59eb2a5507975244c893402d582bf77b",
    "5d7c3a3eed0546a598e3d2a1c7e0026a",
    "5ffcd225924514348ac71730179b5b24",
    "60d57680c3f64539b72423b3dbcd38cd",
    "6105f450bb2acbd458d277e71e19d835",
    "61a17ccbbb3d79445b0926347ec07577",
    "63d8e3a9ab4d72e4081a7862d7246a79",
    "67b09c86234cecc4c8309f22f7d33973",
    "68e814f1f3ce55942a52c1dd536eaa5b",
    "697d64669eb2c0543abb9c9b07998a38",
    "6a556375036ac8b4ebd80e74d308d108",
    "6d3728d4e9c9898458fe5e9532951132",
    "70a139f0a4c6c534eaa34feea0d08622",
    "7115a6c08bd101247b70d72a4ff99453",
    "7213b7bd026d4414da2308df23715d8f",
    "72ba6ad46d94ecd41bad8e64739ea392",
    "7477e2e0b72f4ce4fb674f4b21d5e81d",
    "76d4885a395976547a13c5d6bf95b482",
    "7812ad3672a4b9a4fb894ea402095167",
    "787e56055e3ef864d9c78a3ec21e56be",
    "797f25d709f559546b29e7bcb181cc74",
    "7b64641c76ff4a744a2bce7f91a20f9a",
    "7c492212d25d8f04fbd43eb99d780b1e",
    "7cf5edc65e785a24f9cf93af987d66b3",
    "7fb3023bda2f42728e76cf269f5a2fa0",
    "805fd6181a104bf0aca9ae79ef220c16",
    "80a50d9744a40ac4c96e2cc6451a6703",
    "87ec6541cddfa394ab540dd13399d319",
    "8976de442862f82488a4b138a0a89907",
    "8a6a1920019c45d40b4561f05dcb3240",
    "8ac59959b1b23c347a0361dc97cc786d",
    "8f3d1e6b4be006f4d896081f2f889665",
    "90e54424d682d104ab36436bd527af09",
    "92db263bfb7445d4aa592a5d15128fc5",
    "95ef0ff14771f2549897f300ce62c95c",
    "9719015edcbf142409592e2cbaab7fe1",
    "97e216dbb46ae3c4faef90cf6bbe6fd5",
    "992fd59da1783de49b135ad89142c6d7",
    "9972f33f977fc724c838e59641b2fca5",
    "9a01b6815d6c3684cb25f30b8bf20932",
    "9af88f3ed8a017b45a6837eab7437629",
    "9c928dc570bb9e54a9649b3ebfe47a41",
    "a0e95a17fed549bcbf85a977811f6094",
    "a29a582c3daa4c24bb0e991c596ccb28",
    "ac57069b6bf8c904086171683992a92a",
    "ac8aaf29054f5b74eb18f2af950e752d",
    "adf54af2a681792489826f7fd1b62889",
    "aed0b33e17a3b3d44a718852e87305bd",
    "af0aae1b973114f47a19ea532237b5fc",
    "afd05ca5363036c44817c071189b67e1",
    "b186cea78dce3a04aacff0a81786008c",
    "b3614622866fe7046b787a548bbd7f59",
    "bcbd674ec70ff6f4894bb5f07b6f4095",
    "bdf58317985383540920c723db07aa3b",
    "bfd867bce95e420e82b3c96abb133876",
    "c126adbdf6ddd8245bda33694cd774e8",
    "c3453e7e215c1f149b938be27ac754c6",
    "c36562b8e7ae12d408487ba8b532d966",
    "c3a66c1bbd2fb65498b130802d5f183a",
    "c450b2902b1f4ebdbdd5063260823b57",
    "c4b555225f565bb40a855c1bfeeff07e",
    "c5a39c8f1a2d6824ca565e6c1e4075a5",
    "c5e2aa13b1b6470d8ec70a962ddabdbd",
    "c62ba548b1a34b94b9802925b35737c2",
    "c806103e27cce6f429e5bf47067966cf",
    "c920f2cd2244d284aa69a146aeefcb2c",
    "cb8686e7357a68c42bdd9d4e65334633",
    "cc198d726ac34b4599bd1b4d22a7f2b7",
    "cc9c862ef2e03af4f89be5088851ea35",
    "ceea53555d83f2547ae5fc47e0399e14",
    "d26eb8ab2aabd0e45a4d7eec0340bbce",
    "d41d5bd9a775d7245929256d58a3e03e",
    "d76497bfc48516e45a0831628f767a0f",
    "d809b6c4ff2aaff4fa70d712a70f7d7b",
    "dbec636d84482944f87435bd31522fcc",
    "e2d1fa11f6b095e4fb2fd1dcf5e36eb3",
    "e5bee904e6724fc44aae8887dcefae51",
    "e70ecf1ed95ca2f40b754f1adb22bbdd",
    "e92cb7c4c900dcf4282def347af42b9c",
    "eb6eb946c68ef094f89c7633f5bfdc9b",
    "ed699d64870044b43bb5a7fbe3f29494",
    "ee9ab0117ca06b84f9c66469f4428c61",
    "f09b89812cc94b89a09069671002b899",
    "f308a03bea0d69843a8ed0af003d47a9",
    "f3ef866a9da642c4b9f7dcd76c3567fe",
    "f4201c85a991369408740c6888362e20",
    "f47df34d53f8c904f9981a3ee8e84892",
    "f7de245bb20f12f47864c7cb8b1d1abb",
    "f979ed68d1e74d21962edc66f0a1d169",
    "fc37b70e3d064a147a3a99db4a86ee12",
    "fc85925cd241dd0408ab2f4cb171b7e3",
    "ffa1b373190af4f4db7a5501904a1983",
];
const GENDARME_FEATS: [&str; 155] = [
    "006a966007802a0478c9e21007207aac",
    "055fb8dcaf6be334ca13172bafa9e782",
    "05a3b543b0a0a0346a5061e90f293f0b",
    "0641368c74334671aaaf32a2730587be",
    "09c9e82965fb4334b984a1e9df3bd088",
    "0ab2f21a922feee4dab116238e3150b4",
    "0b442a7b4aa598d4e912a4ecee0500ff",
    "0b55d725ded1ae549bb858fba1d84114",
    "0c17102f650d9044290922b0fad9132f",
    "0da0c194d6e1d43419eb8d990b28e0ab",
    "0f15c6f70d8fb2b49aa6cc24239cc5fa",
    "0f8939ae6f220984e8fb568abbdfba95",
    "0fc1ed8532168f74a9441bd17ad59e66",
    "1191ef3065e6f8e4f9fbe1b7e3c0f760",
    "121811173a614534e8720d7550aae253",
    "12c1b556df3b667458ae200bfb38ccb8",
    "14a1fc1356df9f146900e1e42142fc9d",
    "197306972c98bb843af738dc7529a7ac",
    "1a0298abacb6e0f45b7e28553e99e76c",
    "1b0f68188dcc435429fb87a022239681",
    "1e1f627d26ad36f43bbd26cc2bf8ac7e",
    "1fc8c07a82d749a1a35b6992ad359788",
    "203992ef5b35c864390b4e4a1e200629",
    "204a749f60f443e29fbb9e8f6da0a14d",
    "25bc9c439ac44fd44ac3b1e58890916f",
    "2a6091b97ad940943b46262600eaeaeb",
    "2a681cb9fcaab664286cb36fff761245",
    "2c61fdbf242866f4e93c3e1477fb96b5",
    "308cd7dc4f10efd428f531bbf4f2823d",
    "31470b17e8446ae4ea0dacd6c5817d86",
    "319c882ab3cc51544ad2f3f43633d5b1",
    "32429740d6a5470aaaa02f20d61e43d3",
    "3306f4c6913d4d429358b907cdb4ffd1",
    "39425a13df904e3cb0e3f6debfe65cab",
    "3aa4cbdd4af5ba54888b0dc7f07f80c4",
    "3af3476f6034a924e96f3ffa198424b7",
    "4153755355a0b9b4e956c9ca232c22cf",
    "422dab7309e1ad343935f33a4d6e9f11",
    "42be7d465db94213baf0578c721803ff",
    "453f5181a5ed3a445abfa3bcd3f4ac0c",
    "46f4fb320f35704488ba3d513397789d",
    "46f970a6b9b5d2346b10892673fe6e74",
    "479c7f3b0dba69a4bbcb43e101f3f7f9",
    "47b352ea0f73c354aba777945760b441",
    "4c44724ffa8844f4d9bedb5bb27d144a",
    "4c7205d859a1e114895e798af383d76a",
    "4cc71ae82bdd85b40b3cfe6697bb7949",
    "4e219f5894ad0ea4daa0699e28c37b1d",
    "4f1a78b02ac71bd4fa7d6e011d6f8ce0",
    "52913092cd018da47845f36e6fbe240f",
    "52c6b07a68940af41b270b3710682dc7",
    "54d824028117e884a8f9356c7c66149b",
    "54ee847996c25cd4ba8773d7b8555174",
    "5662d1b793db90c4b9ba68037fd2a768",
    "56f94badbba018b4b8277ce6e2e79e72",
    "57c3ea6b05da4226aaeeb21da04e82ea",
    "59bd93899149fa44687ff4121389b3a9",
    "59eb2a5507975244c893402d582bf77b",
    "5d7c3a3eed0546a598e3d2a1c7e0026a",
    "5ffcd225924514348ac71730179b5b24",
    "60d57680c3f64539b72423b3dbcd38cd",
    "6105f450bb2acbd458d277e71e19d835",
    "61a17ccbbb3d79445b0926347ec07577",
    "63d8e3a9ab4d72e4081a7862d7246a79",
    "67b09c86234cecc4c8309f22f7d33973",
    "68e814f1f3ce55942a52c1dd536eaa5b",
    "697d64669eb2c0543abb9c9b07998a38",
    "6a556375036ac8b4ebd80e74d308d108",
    "6d3728d4e9c9898458fe5e9532951132",
    "70a139f0a4c6c534eaa34feea0d08622",
    "7115a6c08bd101247b70d72a4ff99453",
    "7213b7bd026d4414da2308df23715d8f",
    "72ba6ad46d94ecd41bad8e64739ea392",
    "7477e2e0b72f4ce4fb674f4b21d5e81d",
    "76d4885a395976547a13c5d6bf95b482",
    "7812ad3672a4b9a4fb894ea402095167",
    "787e56055e3ef864d9c78a3ec21e56be",
    "797f25d709f559546b29e7bcb181cc74",
    "7b64641c76ff4a744a2bce7f91a20f9a",
    "7ba4b4023a7748669098c479dd9e8de6",
    "7c492212d25d8f04fbd43eb99d780b1e",
    "7cf5edc65e785a24f9cf93af987d66b3",
    "7fb3023bda2f42728e76cf269f5a2fa0",
    "805fd6181a104bf0aca9ae79ef220c16",
    "80a50d9744a40ac4c96e2cc6451a6703",
    "87ec6541cddfa394ab540dd13399d319",
    "8976de442862f82488a4b138a0a89907",
    "8a6a1920019c45d40b4561f05dcb3240",
    "8ac59959b1b23c347a0361dc97cc786d",
    "8f3d1e6b4be006f4d896081f2f889665",
    "90e54424d682d104ab36436bd527af09",
    "92c7e14b4da24549a9219e990b30d723",
    "92db263bfb7445d4aa592a5d15128fc5",
    "95ef0ff14771f2549897f300ce62c95c",
    "9719015edcbf142409592e2cbaab7fe1",
    "97e216dbb46ae3c4faef90cf6bbe6fd5",
    "992fd59da1783de49b135ad89142c6d7",
    "9972f33f977fc724c838e59641b2fca5",
    "9a01b6815d6c3684cb25f30b8bf20932",
    "9af88f3ed8a017b45a6837eab7437629",
    "9c928dc570bb9e54a9649b3ebfe47a41",
    "a0e95a17fed549bcbf85a977811f6094",
    "a29a582c3daa4c24bb0e991c596ccb28",
    "a2e947d6be234abba7c3ac0bd5dc9b1d",
    "ac57069b6bf8c904086171683992a92a",
    "ac8aaf29054f5b74eb18f2af950e752d",
    "adf54af2a681792489826f7fd1b62889",
    "aed0b33e17a3b3d44a718852e87305bd",
    "af0aae1b973114f47a19ea532237b5fc",
    "afd05ca5363036c44817c071189b67e1",
    "b186cea78dce3a04aacff0a81786008c",
    "b3614622866fe7046b787a548bbd7f59",
    "b46bf228d6b2427abd028e2fe7b84600",
    "bcbd674ec70ff6f4894bb5f07b6f4095",
    "bdf58317985383540920c723db07aa3b",
    "bfd867bce95e420e82b3c96abb133876",
    "c126adbdf6ddd8245bda33694cd774e8",
    "c3453e7e215c1f149b938be27ac754c6",
    "c36562b8e7ae12d408487ba8b532d966",
    "c3a66c1bbd2fb65498b130802d5f183a",
    "c450b2902b1f4ebdbdd5063260823b57",
    "c4b555225f565bb40a855c1bfeeff07e",
    "c5a39c8f1a2d6824ca565e6c1e4075a5",
    "c5e2aa13b1b6470d8ec70a962ddabdbd",
    "c610310d31414edabcedf0c8a6fe32c4",
    "c62ba548b1a34b94b9802925b35737c2",
    "c806103e27cce6f429e5bf47067966cf",
    "c920f2cd2244d284aa69a146aeefcb2c",
    "cb8686e7357a68c42bdd9d4e65334633",
    "cc198d726ac34b4599bd1b4d22a7f2b7",
    "cc9c862ef2e03af4f89be5088851ea35",
    "ceea53555d83f2547ae5fc47e0399e14",
    "d05cbf9cca8c4385a76bb6d65f65a717",
    "d26eb8ab2aabd0e45a4d7eec0340bbce",
    "d41d5bd9a775d7245929256d58a3e03e",
    "d76497bfc48516e45a0831628f767a0f",
    "d809b6c4ff2aaff4fa70d712a70f7d7b",
    "dbec636d84482944f87435bd31522fcc",
    "e2d1fa11f6b095e4fb2fd1dcf5e36eb3",
    "e5bee904e6724fc44aae8887dcefae51",
    "e70ecf1ed95ca2f40b754f1adb22bbdd",
    "e92cb7c4c900dcf4282def347af42b9c",
    "eb6eb946c68ef094f89c7633f5bfdc9b",
    "ed699d64870044b43bb5a7fbe3f29494",
    "ee9ab0117ca06b84f9c66469f4428c61",
    "f09b89812cc94b89a09069671002b899",
    "f308a03bea0d69843a8ed0af003d47a9",
    "f3ef866a9da642c4b9f7dcd76c3567fe",
    "f4201c85a991369408740c6888362e20",
    "f47df34d53f8c904f9981a3ee8e84892",
    "f7de245bb20f12f47864c7cb8b1d1abb",
    "f979ed68d1e74d21962edc66f0a1d169",
    "fc37b70e3d064a147a3a99db4a86ee12",
    "fc85925cd241dd0408ab2f4cb171b7e3",
    "ffa1b373190af4f4db7a5501904a1983",
];
const TACTICIAN_FEATS: [&str; 12] = [
    "9093ceeefe9b84746a5993d619d7c86f",
    "c920f2cd2244d284aa69a146aeefcb2c",
    "992fd59da1783de49b135ad89142c6d7",
    "b186cea78dce3a04aacff0a81786008c",
    "422dab7309e1ad343935f33a4d6e9f11",
    "5662d1b793db90c4b9ba68037fd2a768",
    "6337b37f2a7c11b4ab0831d6780bce2a",
    "0b707584fc2ea724aa72c396c2230dc7",
    "8976de442862f82488a4b138a0a89907",
    "1191ef3065e6f8e4f9fbe1b7e3c0f760",
    "d26eb8ab2aabd0e45a4d7eec0340bbce",
    "c4b555225f565bb40a855c1bfeeff07e",
];

fn blueprints_of(key: &str) -> BTreeSet<String> {
    let list = selection(key);
    assert!(!list.ignores_prerequisites, "{key} checks prerequisites");
    list.feats
        .iter()
        .map(|feat| {
            let blueprint = feat_blueprint(feat.as_str());
            blueprint.unwrap_or_else(|| panic!("{feat} has no blueprint"))
        })
        .collect()
}

fn feat_blueprint(key: &str) -> Option<String> {
    feat(key).blueprint.as_ref().map(|guid| guid.to_string())
}

/// 306m: the cavalier's bonus feats (6, 12, 18), its Tactician's teamwork feats (1, 9, 17) and the Gendarme's bonus
/// feats (1, 3, 6, 9, 12, 15, 18, 20) are three lists of their own, each the game's full member set.
// Game: CavalierProgression aa70326bdaa7015438df585cf2ab93b9 grants CavalierBonusFeatSelection at 6, 12 and 18 and
// CavalierTacticianFeatSelection at 1, 9 and 17; GendarmeArchetype 48fd69254e9e5c540bd65101d3252184 `AddFeatures`
// GendarmeFeatSelection at 1, 3, 6, 9, 12, 15, 18 and 20 and `RemoveFeatures` CavalierBonusFeatSelection at 6, 12 and 18
// and CavalierTacticianFeatSelection at 1, 9 and 17; FearsomeLeaderArchetype 13cc5ccd5cef4c98a5806f033e19a8bb,
// GhostRiderArchetype c0503d9864b94134ab1237a2bf2d38a0 and KnightOfTheWallArchetype 112dd0e61f95c3a459ac0a565472e685
// remove the Tactician's (the others keep both). All three have `IgnorePrerequisites` false.
#[test]
fn the_cavaliers_three_feat_lists_are_the_games() {
    for (key, guid, members) in [
        (
            "cavalier/bonus-feats",
            "dd17090d14958ef48ba601688b611970",
            CAVALIER_BONUS_FEATS.as_slice(),
        ),
        (
            "cavalier/tactician-feats",
            "7bc55b5e381358c45b42153b8b2603a6",
            TACTICIAN_FEATS.as_slice(),
        ),
        (
            "cavalier/gendarme/bonus-feats",
            "ef1cd58e0b7fc7f45baedb09407a1cd1",
            GENDARME_FEATS.as_slice(),
        ),
    ] {
        assert_eq!(
            selection(key).blueprint.as_ref().map(|guid| guid.as_str()),
            Some(guid),
            "{key}"
        );
        let expected: BTreeSet<String> = members.iter().map(|guid| guid.to_string()).collect();
        assert_eq!(
            expected.len(),
            members.len(),
            "{key}: a member listed twice"
        );
        assert_eq!(blueprints_of(key), expected, "{key}");
    }
    // The old pool's five feats are in the cavalier's list; Mounted Combat is the game's first (a member by its
    // GUID in the list above), and the list is no longer the Mounted Combat category.
    assert!(feat_keys(selection("cavalier/bonus-feats")).contains("mounted-combat"));
    assert!(feat_keys(selection("cavalier/bonus-feats")).contains("improved-initiative"));
    assert!(!feat_keys(selection("cavalier/tactician-feats")).contains("mounted-combat"));

    let class_levels = |selection: &str| levels_naming(class_rows("cavalier"), selection);
    assert_eq!(class_levels("cavalier/bonus-feats"), [6, 12, 18]);
    assert_eq!(class_levels("cavalier/tactician-feats"), [1, 9, 17]);
    assert_eq!(class_levels("combat-feats"), Vec::<u8>::new());
    assert_eq!(class_levels("teamwork-feats"), Vec::<u8>::new());
    for (archetype, bonus, tactician) in [
        ("cavalier/beast-rider", vec![], vec![1]),
        ("cavalier/cavalier-of-the-paw", vec![8], vec![1]),
        ("cavalier/disciple-of-the-pike", vec![6, 12], vec![1, 9, 17]),
        ("cavalier/fearsome-leader", vec![6, 12, 18], vec![]),
        ("cavalier/standard-bearer", vec![], vec![1]),
    ] {
        assert_eq!(
            levels_naming(archetype_rows(archetype), "cavalier/bonus-feats"),
            bonus,
            "{archetype}"
        );
        assert_eq!(
            levels_naming(archetype_rows(archetype), "cavalier/tactician-feats"),
            tactician,
            "{archetype}"
        );
    }
    let gendarme = archetype_rows("cavalier/gendarme");
    assert_eq!(
        levels_naming(gendarme.clone(), "cavalier/gendarme/bonus-feats"),
        [1, 3, 6, 9, 12, 15, 18, 20]
    );
    for gone in [
        "cavalier/bonus-feats",
        "cavalier/tactician-feats",
        "combat-feats",
    ] {
        assert_eq!(
            levels_naming(gendarme.clone(), gone),
            Vec::<u8>::new(),
            "{gone}"
        );
    }
}

/// The feats and options a prerequisite names as `Feat` leaves (a `Not` included), by key.
fn feat_leaves(prerequisite: &Option<rules::Prerequisite>) -> BTreeSet<String> {
    (prerequisite.iter().flat_map(|tree| tree.leaves()))
        .filter_map(|leaf| match leaf {
            rules::Prerequisite::Feat(key) => Some(key.as_str().to_string()),
            _ => None,
        })
        .collect()
}

fn class_levels(prerequisite: &Option<rules::Prerequisite>) -> Vec<(String, u8)> {
    (prerequisite.iter().flat_map(|tree| tree.leaves()))
        .filter_map(|leaf| match leaf {
            rules::Prerequisite::ClassLevel { class, level } => {
                Some((class.as_str().to_string(), *level))
            }
            _ => None,
        })
        .collect()
}

fn feature_leaves(prerequisite: &Option<rules::Prerequisite>) -> BTreeSet<String> {
    (prerequisite.iter().flat_map(|tree| tree.leaves()))
        .filter_map(|leaf| match leaf {
            rules::Prerequisite::Feature { feature, .. } => Some(feature.as_str().to_string()),
            _ => None,
        })
        .collect()
}

fn aliases_of(key: &str) -> BTreeSet<String> {
    (feat(key).aliases.iter())
        .map(|alias| alias.as_str().to_string())
        .collect()
}

/// 306q: Advanced Weapon Training is the game's four feats, each after the one before; a second pick of the old key
/// opens as the second feat.
// Game: AdvancedWeaponTraining1 3aa4cbdd4af5ba54888b0dc7f07f80c4 (Fighter 5, PrerequisiteNoFeature of itself),
// AdvancedWeaponTraining2 70a139f0a4c6c534eaa34feea0d08622 (PrerequisiteFeature AdvancedWeaponTraining1, Fighter 10),
// AdvancedWeaponTraining3 ee9ab0117ca06b84f9c66469f4428c61 (the second, Fighter 15), AdvancedWeaponTraining4
// 0b55d725ded1ae549bb858fba1d84114 (the third, Fighter 20); each `Ranks` 1, a combat feat of `IgnorePrerequisites`
// true, listed by MagusFeatSelection, ArcaneRiderFeatSelection, WarpriestFeatSelection, CavalierBonusFeatSelection and
// GendarmeFeatSelection (see the lists' tests above) and by the general `FeatureSelection` 247a4068296e8be42890143f451b4b45.
#[test]
fn advanced_weapon_training_is_four_feats_each_after_the_last() {
    for (key, guid, before, level) in [
        (
            "advanced-weapon-training",
            "3aa4cbdd4af5ba54888b0dc7f07f80c4",
            None,
            5,
        ),
        (
            "advanced-weapon-training-2",
            "70a139f0a4c6c534eaa34feea0d08622",
            Some("advanced-weapon-training"),
            10,
        ),
        (
            "advanced-weapon-training-3",
            "ee9ab0117ca06b84f9c66469f4428c61",
            Some("advanced-weapon-training-2"),
            15,
        ),
        (
            "advanced-weapon-training-4",
            "0b55d725ded1ae549bb858fba1d84114",
            Some("advanced-weapon-training-3"),
            20,
        ),
    ] {
        let one = feat(key);
        assert_eq!(feat_blueprint(key).as_deref(), Some(guid), "{key}");
        assert_eq!(one.ranks, 1, "{key}");
        let mut expected = BTreeSet::from([key.to_string()]);
        expected.extend(before.map(str::to_string));
        assert_eq!(feat_leaves(&one.prerequisite), expected, "{key}");
        assert_eq!(
            class_levels(&one.prerequisite),
            vec![("fighter".to_string(), level)],
            "{key}"
        );
        for list in [
            "feats",
            "magus/bonus-feats",
            "magus/arcane-rider/bonus-feats",
            "warpriest/bonus-feats",
            "cavalier/bonus-feats",
            "cavalier/gendarme/bonus-feats",
        ] {
            assert!(feat_keys(selection(list)).contains(key), "{list} {key}");
        }
    }
    // A second pick of the old key opens as the second feat, the only heir that holds; the third and fourth are no one's heir.
    assert_eq!(
        aliases_of("advanced-weapon-training-2"),
        BTreeSet::from(["advanced-weapon-training".to_string()])
    );
    assert!(aliases_of("advanced-weapon-training-3").is_empty());
    assert!(aliases_of("advanced-weapon-training-4").is_empty());
}

/// 306q: Magical Tail is the game's one feat of eight ranks, not its first rank's blueprint.
// Game: MagicalTail febb8fe9a2d142fb80c1be6b0b539d9d (a `BlueprintFeature` with `Ranks` 8, the Kitsune race its
// prerequisite); MagicalTail1 and the rest are what each rank grants.
#[test]
fn magical_tail_is_the_games_one_feat_of_eight_ranks() {
    assert_eq!(
        feat_blueprint("magical-tail").as_deref(),
        Some("febb8fe9a2d142fb80c1be6b0b539d9d")
    );
    assert_eq!(feat("magical-tail").ranks, 8);
}

/// 306q: Abundant Smite keeps its old key (a mythic ability has no alias) and Abundant Smite Chaos is its own ability.
// Game: AbundantSmite 7e5b63faeca24474db0bfd019167dda4 (PrerequisiteFeature SmiteEvilFeature
// 3a6db57fce75b0244a6a5819528ddf26) and AbundantSmiteChaos 4cdc155e26204491ba4d193646cb4443 (SmiteChaosFeature
// 67d08ed4892de3441a43aae91a7dd7c9), each `Ranks` 1.
#[test]
fn abundant_smite_is_two_mythic_abilities() {
    let ability = |key: &str| {
        DATA.mythic_abilities
            .iter()
            .find(|ability| ability.key.as_str() == key)
            .unwrap_or_else(|| panic!("no mythic ability {key}"))
    };
    for (key, guid, feature) in [
        (
            "abundant-smite-smite-chaos",
            "7e5b63faeca24474db0bfd019167dda4",
            "smite-evil",
        ),
        (
            "abundant-smite-chaos",
            "4cdc155e26204491ba4d193646cb4443",
            "hellknight/smite-chaos",
        ),
    ] {
        let one = ability(key);
        assert_eq!(one.blueprint.as_ref().map(|g| g.as_str()), Some(guid));
        assert_eq!(
            feature_leaves(&one.prerequisite),
            BTreeSet::from([feature.to_string()]),
            "{key}"
        );
    }
    let list = selection("mythic/extra-mythic-ability");
    for key in ["abundant-smite-smite-chaos", "abundant-smite-chaos"] {
        assert!(
            list.mythic_abilities.iter().any(|a| a.as_str() == key),
            "{key}"
        );
    }
}

/// 306r: Extra Channel is six feats, one per channel the game's Extra Channel blueprints name, each with its own
/// feature as the prerequisite; the old key keeps the cleric's two.
// Game: ExtraChannel cd9f19775bd9d3343a31a065e93f0c47 (ChannelEnergyFeature a79013ff… or ChannelNegativeFeature
// 3adb2c90…), ExtraChannelOracle 53da6bf487997e947960bd6c02be9adf (OracleRevelationChannel ade57ae9…),
// ExtraChannelShaman 4cc73e43ab814bc48ade2e291729b359 (ShamanLifeSpiritChannelEnergyFeature or
// WitchDoctorChannelPositiveFeature eb388d17…), ExtraChannelHospitaler 8d4f82fdb4d09b247ae8cd1ae7ce02de
// (ChannelEnergyHospitalerFeature a9ab1bbc…), ExtraChannelHexChanneler 88f426f13de236642b9585efcb82f1d8
// (HexChannelerChannelFeature b40316f0…), ExtraChannelEmpyrealSorcerer 347e8d794c9598e4abed70adda868ccd
// (ChannelEnergyEmpyrealFeature 7d49d7f5…); each `Ranks` 10 and 2 more uses a day (`IncreaseResourceAmount` `Value` 2).
#[test]
fn extra_channel_is_six_feats_one_per_channel() {
    for (key, guid) in [
        ("extra-channel", "cd9f19775bd9d3343a31a065e93f0c47"),
        ("extra-channel-oracle", "53da6bf487997e947960bd6c02be9adf"),
        ("extra-channel-shaman", "4cc73e43ab814bc48ade2e291729b359"),
        (
            "extra-channel-hospitaler",
            "8d4f82fdb4d09b247ae8cd1ae7ce02de",
        ),
        (
            "extra-channel-hex-channeler",
            "88f426f13de236642b9585efcb82f1d8",
        ),
        (
            "extra-channel-empyreal-sorcerer",
            "347e8d794c9598e4abed70adda868ccd",
        ),
    ] {
        assert_eq!(feat_blueprint(key).as_deref(), Some(guid), "{key}");
        assert_eq!(feat(key).ranks, 10, "{key}");
        assert!(feat_keys(selection("feats")).contains(key), "{key}");
        let heirs = aliases_of(key);
        if key == "extra-channel" {
            assert!(heirs.is_empty());
        } else {
            assert_eq!(
                heirs,
                BTreeSet::from(["extra-channel".to_string()]),
                "{key}"
            );
        }
    }
    let channel = |key: &str| feature_leaves(&feat(key).prerequisite);
    assert_eq!(
        channel("extra-channel"),
        BTreeSet::from([
            "channel-positive-energy".to_string(),
            "cleric/channel-energy".to_string(),
            "cleric/priest-of-balance/channel-negative-energy".to_string(),
        ])
    );
    // The Oracle's and the Hex Channeler's channels are options, not features.
    let option_leaves = |key: &str| {
        (feat(key).prerequisite.iter().flat_map(|tree| tree.leaves()))
            .filter_map(|leaf| match leaf {
                rules::Prerequisite::SelectionOption(option) => Some(option.as_str().to_string()),
                _ => None,
            })
            .collect::<BTreeSet<_>>()
    };
    assert_eq!(
        option_leaves("extra-channel-oracle"),
        BTreeSet::from(["oracle/revelations/life/channel".to_string()])
    );
    assert_eq!(
        option_leaves("extra-channel-hex-channeler"),
        BTreeSet::from(["witch/hex/channel".to_string()])
    );
    assert_eq!(
        channel("extra-channel-hospitaler"),
        BTreeSet::from(["paladin/hospitaler/channel-positive-energy-hospitaler".to_string()])
    );
    assert_eq!(
        channel("extra-channel-empyreal-sorcerer"),
        BTreeSet::from(["sorcerer/empyreal-sorcerer/channel-positive-energy".to_string()])
    );
}

/// 306q: Extra Rage and Extra Performance each have the class variants the game gives them, naming the old key as
/// an alias.
// Game: ExtraRage 1a54bbbafab728348a015cf9ffcf50a7 (RageFeature), ExtraRageInstinctualWarrior
// dcc338110bab48ddbdc22e82cef7a6db (FocusedRageFeature 17b5ab90…), ExtraPerformance 0d3651b2cb0d89448b112e23214e744e,
// ExtraPerformanceSkald 15c78493a0ce471cb279e6363d702cc8 (RagingSong 334cb75a…), ExtraPerformancePaladinMartyr
// 639ce2b85a13404caa618f36748d7fa5 (MartyrInspireCourageFeature c0a1b34b…); each `Ranks` 10, 6 more rounds
// (`IncreaseResourceAmount` `Value` 6).
#[test]
fn extra_rage_and_extra_performance_have_the_games_class_variants() {
    for (key, guid, old, feature) in [
        (
            "extra-rage-instinctual-warrior",
            "dcc338110bab48ddbdc22e82cef7a6db",
            "extra-rage",
            "barbarian/instinctual-warrior/focused-rage",
        ),
        (
            "extra-performance-skald",
            "15c78493a0ce471cb279e6363d702cc8",
            "extra-performance",
            "skald/raging-song",
        ),
        (
            "extra-performance-martyr",
            "639ce2b85a13404caa618f36748d7fa5",
            "extra-performance",
            "paladin/martyr/inspire-courage",
        ),
    ] {
        assert_eq!(feat_blueprint(key).as_deref(), Some(guid), "{key}");
        assert_eq!(feat(key).ranks, 10, "{key}");
        assert_eq!(aliases_of(key), BTreeSet::from([old.to_string()]), "{key}");
        assert_eq!(
            feature_leaves(&feat(key).prerequisite),
            BTreeSet::from([feature.to_string()]),
            "{key}"
        );
        assert!(feat_keys(selection("feats")).contains(key), "{key}");
    }
}

/// 306r: Bleeding Critical and Spell Bane are feats, and Bleeding Critical is in the game's lists that hold it.
// Game: BleedingCriticalFeature 42be7d465db94213baf0578c721803ff (PrerequisiteFeature CriticalFocus
// 8ac59959b1b23c347a0361dc97cc786d, base attack bonus 11) and SpellBane d2d1b1f27bdf4ddfa5bf8b7244786ff9
// (InquisitorBaneNormalFeatureAdd 7ddf7fbe… or LivingGrimoireSacredWordFeature 0e19c677…, Any), listed by
// FeatureSelection 247a4068296e8be42890143f451b4b45 and, for Bleeding Critical, by the magus's, warpriest's and
// cavalier's lists (see their tests above).
#[test]
fn bleeding_critical_and_spell_bane_are_feats() {
    assert_eq!(
        feat_blueprint("bleeding-critical").as_deref(),
        Some("42be7d465db94213baf0578c721803ff")
    );
    assert_eq!(
        feat_leaves(&feat("bleeding-critical").prerequisite),
        BTreeSet::from(["critical-focus".to_string()])
    );
    assert_eq!(
        feat_blueprint("spell-bane").as_deref(),
        Some("d2d1b1f27bdf4ddfa5bf8b7244786ff9")
    );
    assert_eq!(
        feature_leaves(&feat("spell-bane").prerequisite),
        BTreeSet::from([
            "inquisitor/bane".to_string(),
            "inquisitor/living-grimoire/sacred-word".to_string(),
        ])
    );
    for list in [
        "feats",
        "magus/bonus-feats",
        "magus/arcane-rider/bonus-feats",
        "warpriest/bonus-feats",
        "cavalier/bonus-feats",
        "cavalier/gendarme/bonus-feats",
    ] {
        assert!(
            feat_keys(selection(list)).contains("bleeding-critical"),
            "{list}"
        );
    }
    assert!(feat_keys(selection("feats")).contains("spell-bane"));
}

/// The game's `m_AllFeatures` of SorcererBonusFeat: 21 feats (user fact 281, 2026-11-15: the game's list replaces ticket
/// 92's Magic feats).
const SORCERER_BONUS_FEATS_GUIDS: [&str; 21] = [
    "06964d468fde1dc4aa71a92ea04d930d", // CombatCasting
    "1978c3f91cfbbc24b9c9b0d017f4beec", // GreaterSpellPenetration
    "ee7dc126939e4d9438357fbd5980d459", // SpellPenetration
    "38155ca9e4055bb48a89240a2055dcc3", // AugmentSummoning
    "0477936c0f74841498b5c8753a8062a3", // SuperiorSummoning
    "a1de1e4f92195b442adb946f0e2b9d4e", // EmpowerSpellFeat
    "f180e72e4a9cbaa4da8be9bc958132ef", // ExtendSpellFeat
    "2f5d1e705c7967546b72ad8218ccf99c", // HeightenSpellFeat
    "7f2b282626862e345935bbea5e66424b", // MaximizeSpellFeat
    "ef7ece7bb5bb66a41b256976b27f424e", // QuickenSpellFeat
    "46fad72f54a33dc4692d3b62eca7bb78", // ReachSpellFeat
    "cd26b9fa3f734461a0fcedc81cafaaac", // PersistentSpellFeat
    "85f3340093d144dd944fff9a9adfd2f2", // SelectiveSpellFeat
    "1c17446a3eb744f438488711b792ca4d", // GreaterElementalFocusSelection
    "bb24cc01319528849b09a3ae8eec0b31", // ElementalFocusSelection
    "5b04b45b228461c43bad768eb0f7c7bf", // SpellFocusGreater
    "16fa59cc9a72a6043b566b49184f53fe", // SpellFocus
    "f327a765a4353d04f872482ef3e48c35", // SpellSpecializationFirst
    "fbf5d9ce931f47f3a0c818b3f8ef8414", // BolsteredSpellFeat
    "c101ad6879a94204a780506f7a554865", // PiercingSpell
    "8ad7fd39abea4722b39eb5a67d606a41", // IntensifiedSpell
];

/// What SeekerFeatSelection and SkaldFeatSelection hold beyond the sorcerer's 21.
const SEEKER_AND_SKALD_EXTRA_GUIDS: [&str; 5] = [
    "2fb48bed3746431bb3080c34f11597da", // AmbuscadingSpell
    "e55e2c96ae974bc9ac9fda8b3a56e432", // ArcaneSpiritFeat
    "c39576f8842e4505b14aa918b3a36a0a", // DispelFocus
    "d0cf79c8e0a44325b00dc8fa6ad37d7c", // GreaterDispelFocus
    "d8d641edc9a74a64809f56feff9495b6", // SpellChain
];

/// 306n: the sorcerer's level-1 bonus feat, the Seekers' feats at 5, 10 and 15 and the skald's level-1 bonus feat are
/// lists of their own, each the game's full member set, and no row names the Magic feats any more.
// Game: SorcererProgression 997665565ca80a649aedd72455c4df1f grants SorcererBonusFeat
// d6dd06f454b34014ab0903cb1ed2ade3 at 1 (21 feats; OverwhelimgMageArchetype 734917ec8b8542378c1d35519439d67f removes
// it); both SeekerArchetypes (sorcerer 7229db6fc0b07af4180e783eed43c4d9, oracle c539d65b7682c8d469e6f122922f1dee)
// `AddFeatures` SeekerFeatSelection c6b609279cc3174478624182ac1ad812 at 5, 10 and 15 (26 feats: the 21 and five
// more); SkaldProgression 26418fed2bc153245972a5b54204ed75 grants SkaldFeatSelection
// 0a1999535b4f77b4d89f689a385e5ec9 at 1 (the same 26). All three have `IgnorePrerequisites` false.
#[test]
fn the_sorcerers_seekers_and_skalds_bonus_feats_are_the_games() {
    let sorcerer: BTreeSet<String> = (SORCERER_BONUS_FEATS_GUIDS.iter())
        .map(|guid| guid.to_string())
        .collect();
    assert_eq!(sorcerer.len(), 21);
    let seeker: BTreeSet<String> = (sorcerer.iter().cloned())
        .chain(
            SEEKER_AND_SKALD_EXTRA_GUIDS
                .iter()
                .map(|guid| guid.to_string()),
        )
        .collect();
    assert_eq!(seeker.len(), 26);
    for (key, guid, members) in [
        (
            "sorcerer/bonus-feat",
            "d6dd06f454b34014ab0903cb1ed2ade3",
            &sorcerer,
        ),
        (
            "sorcerer/seeker-sorcerer/bonus-feats",
            "c6b609279cc3174478624182ac1ad812",
            &seeker,
        ),
        (
            "oracle/seeker/bonus-feats",
            "c6b609279cc3174478624182ac1ad812",
            &seeker,
        ),
        (
            "skald/bonus-feat",
            "0a1999535b4f77b4d89f689a385e5ec9",
            &seeker,
        ),
    ] {
        assert_eq!(
            selection(key).blueprint.as_ref().map(|guid| guid.as_str()),
            Some(guid),
            "{key}"
        );
        assert_eq!(&blueprints_of(key), members, "{key}");
    }

    // No sorcerer, Seeker or skald row names the Magic feats; each names its own list at the game's levels.
    let named = |rows: Vec<(u8, &'static [rules::FeatureSelectionKey])>, key: &str| {
        (
            levels_naming(rows.clone(), key),
            levels_naming(rows, "magic-feats"),
        )
    };
    assert_eq!(
        named(class_rows("sorcerer"), "sorcerer/bonus-feat"),
        (vec![1], vec![])
    );
    for archetype in [
        "sorcerer/crossblooded",
        "sorcerer/empyreal-sorcerer",
        "sorcerer/geomancer",
        "sorcerer/nine-tailed-heir",
        "sorcerer/sage-sorcerer",
        "sorcerer/seeker-sorcerer",
        "sorcerer/sylvan-sorcerer",
    ] {
        assert_eq!(
            named(archetype_rows(archetype), "sorcerer/bonus-feat"),
            (vec![1], vec![]),
            "{archetype}"
        );
    }
    assert_eq!(
        named(
            archetype_rows("sorcerer/overwhelming-mage"),
            "sorcerer/bonus-feat"
        ),
        (vec![], vec![])
    );
    for (archetype, list) in [
        (
            "sorcerer/seeker-sorcerer",
            "sorcerer/seeker-sorcerer/bonus-feats",
        ),
        ("oracle/seeker", "oracle/seeker/bonus-feats"),
    ] {
        assert_eq!(
            named(archetype_rows(archetype), list),
            (vec![5, 10, 15], vec![]),
            "{archetype}"
        );
    }
    assert_eq!(
        named(class_rows("skald"), "skald/bonus-feat"),
        (vec![1], vec![])
    );
    for archetype in [
        "skald/battle-scion",
        "skald/court-poet",
        "skald/herald-of-the-horn",
        "skald/inciter",
    ] {
        assert_eq!(
            named(archetype_rows(archetype), "skald/bonus-feat"),
            (vec![1], vec![]),
            "{archetype}"
        );
    }
}
