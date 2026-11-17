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
/// Dark Lurker's talent selection, and their own are another feature. Its slot is the feat's own for any class that
/// holds the feat (306aa), so the refusal is the feat's prerequisite, which S4 (308) writes as the game's.
// Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 has PrerequisiteFeature (Group Any) of
// RogueTalentSelection c074a5d615200494b8f2a9c845799d93, SylvanTricksterTalentSelection
// 290bbcc3c3bb92144b853fd8fb8ff452 and DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2; BardTalentSelection
// 94e2cd84bf3a8e04f8609fe502892f4f and SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57 are none of them.
#[test]
#[ignore = "waits on ticket 308: the importer writes Extra Rogue Talent's prerequisite as the game's selections, \
            which a bard's and a skald's talent selections aren't"]
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
/// talents and Evasion, whatever class takes the feat.
// Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 (RogueTalentSelection
// c074a5d615200494b8f2a9c845799d93's 38 and Evasion 576933720c440aa4d8d42b0c54b77e80); `SelectFeature.Apply`,
// WrathDecompiled 6ce4a6b, adds the picked selection with its own list.
#[test]
fn a_pick_in_extra_rogue_talents_slot_stands_for_a_rogue_and_a_bard() {
    for class in ["rogue", "bard"] {
        let mut build = with_selections_filled(classes_build(&[class, class, class]));
        build
            .picks
            .insert(general_slot(3), feat_pick_plain("extra-rogue-talent"));
        assert!(
            possible(&with_pick(
                build,
                extra_slot(3, 0),
                &rogue_talent("fast-stealth")
            )),
            "{class}"
        );
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
/// less the four members the planner has no feat for yet (Advanced Weapon Training 2 to 4 and Bleeding Critical).
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
    assert_eq!(magus.len(), 181 - 4);
    assert_eq!(rider.len(), 182 - 1 - 4);
    assert_eq!(warpriest.len(), 155 - 4);
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
