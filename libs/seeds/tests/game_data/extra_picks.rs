//! Extra feats' extra picks (ticket 49), and Extra Discovery and Extra Arcana for the Vivisectionist and the
//! Hexcrafter (ticket 64).

use std::collections::BTreeSet;

use rules::{Build, SlotId};

use crate::support::*;

// ---- Extra feats' extra picks (ticket 49) ---------------------------------------------------------------------

/// The "Extra …" feats that give one more pick of a class selection ("You gain one additional hex"), with the
/// selection each grants: the game's own Extra selection, which holds its own list and opens its slot whenever the
/// feat is taken (`SelectFeature.Apply`, WrathDecompiled 6ce4a6b; ticket 306aa). Extra Mercy's since ticket 54 made the
/// paladin's mercies a selection; the Vivisectionist's Medical Discovery since ticket 64. Which classes may take the
/// feat is its prerequisite's (S4, 308).
const EXTRA_PICKS: [(&str, &[&str]); 11] = [
    ("extra-arcana", &["magus/extra-arcana"]), // Game: ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 (its own list of the 15 magus arcana; met by MagusArcanaSelection, HexcrafterMagusHexArcanaSelection or EldritchMagusArcanaSelection)
    (
        "extra-arcanist-exploit",
        &["arcanist/extra-arcanist-exploit"],
    ), // Game: ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a (ArcanistExploitSelection or ExploiterExploitSelection)
    ("extra-discovery", &["alchemist/extra-discovery"]), // Game: ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66 (DiscoverySelection cd86c437488386f438dcc9ae727ea2a6); UndergroundChemistSelection c3d08fdb481aa2e4a9f71c4349141826 meets neither
    (
        "extra-discovery-vivisectionist",
        &["alchemist/vivisectionist/extra-discovery"],
    ), // Game: ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae (VivsectionistDiscoverySelection 67f499218a0e22944abab6fe1c9eaeee)
    ("extra-hex", &["witch/extra-hex"]), // Game: ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35; the shaman's is another feat, ExtraShamanHexSelection d0b4c8245d504b8c9c6d3fccc1f8c5b6
    ("extra-hex-shaman", &["shaman/extra-hex"]), // Game: ExtraShamanHexSelection d0b4c8245d504b8c9c6d3fccc1f8c5b6 (ShamanHexSelection 4223fe18c75d4d14787af196a04e14e7)
    ("extra-mercy", &["paladin/extra-mercy"]), // Game: ExtraMercySelection 8a49abed5be9473da1e1fd1e2457562e (SelectionMercy 02b187038a8dce545bb34bbfb346428d)
    ("extra-rage-power", &["barbarian/extra-rage-power"]), // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a (RagePowerSelection, InstinctualWarriorRagePowerSelection or SkaldRagePowerSelection; BattleScionArchetype 71d41dd6309ea994886908c34c87ccd9's BattleProwessSelection meets none)
    ("extra-rogue-talent", &["rogue/extra-rogue-talent"]), // Game: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 (RogueTalentSelection, SylvanTricksterTalentSelection 290bbcc3c3bb92144b853fd8fb8ff452 or DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2; FeyformShifterShifterArchetype 1cdfc7d306d1430eac19427539b62091 adds RogueTalentSelection)
    ("extra-slayer-talent", &["slayer/extra-slayer-talent"]), // Game: ExtraSlayerTalentSelection 53f7237f5b1447bb851ba68045a00e41
    ("extra-wild-talent", &["kineticist/extra-wild-talent"]), // Game: ExtraWildTalentSelection 0f73730cf1b44ee882671e55d5f6e471 (WildTalentSelection 5c883ae0cd6d7d5448b7a420f51f8459)
];

/// An Extra feat that gives a pick, with the selections the pick may be of.
type ExtraPick = (String, BTreeSet<String>);

/// The data's Extra feats that give a pick.
fn data_extra_picks() -> BTreeSet<ExtraPick> {
    DATA.feats
        .iter()
        .filter(|feat| feat.selection.is_some())
        .map(|feat| {
            (
                feat.key.to_string(),
                picks_of(feat).iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect()
}

/// The game's: `EXTRA_PICKS`.
fn game_extra_picks() -> BTreeSet<ExtraPick> {
    EXTRA_PICKS
        .iter()
        .map(|(feat, selections)| {
            (
                feat.to_string(),
                selections.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect()
}

#[test]
fn the_extra_feats_that_give_a_pick_name_their_selections() {
    // The resource ones give no pick.
    for key in [
        "extra-arcane-pool",  // Game: ExtraArcanePool 42f96fc8d6c80784194262e51b0a1d25
        "extra-bane-weapon",  // Game: ExtraBane 756dc2f7340b0b34285a1dc367ff7359
        "extra-bloodrage",    // Game: ExtraBloodRage 7624c6e46551720459cf5292da67d2f9
        "extra-bombs",        // Game: ExtraBombs 54c57ce67fa1d9044b1b3edc459e05e2
        "extra-channel",      // Game: ExtraChannel cd9f19775bd9d3343a31a065e93f0c47
        "extra-ki",           // Game: ExtraKi 231a2a603d0b437e939553e6da3e7247
        "extra-lay-on-hands", // Game: ExtraLayOnHands a2b2f20dfb4d3ed40b9198e22be82030
        "extra-performance",  // Game: ExtraPerformance 0d3651b2cb0d89448b112e23214e744e
        "extra-rage",         // Game: ExtraRage 1a54bbbafab728348a015cf9ffcf50a7
        "extra-reservoir",    // Game: ExtraReservoir 29afc53efa4d469b9476760fa09576de
    ] {
        assert!(feat(key).selection.is_none(), "{key}");
    }
}

/// Each Extra feat grants the game's own Extra selection, whose slot is of that selection for any class (Extra Hex's
/// two feats, the witch's and the shaman's, each their own).
// Game: the rows of EXTRA_PICKS
#[test]
fn the_extra_feats_that_give_a_pick_name_the_games_selections() {
    assert_eq!(data_extra_picks(), game_extra_picks());
}

#[test]
fn a_barbarian_with_extra_rage_power_at_3_gets_a_rage_power_slot_there() {
    let engine = engine();
    let mut build = with_selections_filled(barbarian(3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a needs RagePowerSelection
    // 28710502f46848d48b3f0d6132817c4e, which BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12 gives at 2
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(3, 0)),
        Some(selection_kind("barbarian/extra-rage-power"))
    );
    let keys = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    // 33, and the Instinctual Warrior's five monk powers (ticket 66), locked here. The slot at 2:
    // `the_barbarians_rage_power_slot_offers_33_and_extra_rage_power_38`.
    // Game: ExtraRagePowerSelection 0c7f01fbbe687bb4baff8195cb02fe6a (38; the monk powers, e.g. RageDeflectArrowsFeature
    // d971bbb15f354621ac37fd51d0ab9044, need Instinctual Warrior 4)
    assert_eq!(keys(&extra_slot(3, 0)).len(), 38);
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&extra_slot(3, 0)).unwrap();
    assert_eq!(engine.slot_label(slot), "Rage Power");

    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(3, 0)));
    assert!(possible(&filled));
}

/// The barbarian's own rage power slot offers 33 powers, without the Instinctual Warrior's five monk powers; Extra Rage
/// Power's list is its own 38.
// Game: RagePowerSelection 28710502f46848d48b3f0d6132817c4e (33); ExtraRagePowerSelection
// 0c7f01fbbe687bb4baff8195cb02fe6a (38)
#[test]
fn the_barbarians_rage_power_slot_offers_33_and_extra_rage_power_38() {
    let mut build = with_selections_filled(barbarian(3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    assert_eq!(offered_at(&build, &selection_slot(2)).len(), 33);
    assert_eq!(offered_at(&build, &extra_slot(3, 0)).len(), 38);
}

/// A feature that is itself a selection skips the rank check: Extra Rage Power may be taken again.
// Game: BlueprintFeatureSelection.CanSelect, WrathDecompiled 6ce4a6b; ExtraRagePowerSelection
// 0c7f01fbbe687bb4baff8195cb02fe6a
#[test]
fn extra_rage_power_taken_again_gives_a_second_slot() {
    let mut build = with_selections_filled(barbarian(5));
    for level in [3, 5] {
        build
            .picks
            .insert(general_slot(level), feat_pick_plain("extra-rage-power"));
    }
    let build = with_selections_filled(build);
    assert!(possible(&build));
    for level in [3, 5] {
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind("barbarian/extra-rage-power")),
            "at {level}"
        );
        assert!(build.picks.contains_key(&extra_slot(level, 0)));
    }
}

#[test]
fn extra_rage_power_isnt_removable_while_its_slots_pick_depends_on_it_and_takes_the_slot_with_it() {
    let engine = engine();
    let mut build = with_selections_filled(barbarian(3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    let mut build = with_selections_filled(build);
    assert!(build.picks.contains_key(&extra_slot(3, 0)));
    let feat = rules::Choice::Slot(general_slot(3));
    assert!(!engine.is_removable(&build, &feat));
    assert!(!engine.removable(&build).contains(&feat));
    assert!(engine.is_removable(&build, &rules::Choice::Slot(extra_slot(3, 0))));

    build.apply(&rules::Edit::Clear(extra_slot(3, 0)));
    assert!(engine.is_removable(&build, &feat));
    build.apply(&rules::Edit::Clear(general_slot(3)));
    assert!(possible(&build));
    assert_eq!(kind_at(&build, &extra_slot(3, 0)), None);
}

/// Any Extra feat opens its own selection's slot whenever it is held, whatever slots the character had before: a
/// Dark Lurker 5 has none of the plain rogue's. Who may hold the feat is its prerequisite's: a bard or a skald, whose
/// talent selections the feat doesn't name, is refused it and gets no slot from it.
// Game: `SelectFeature.Apply` adds the picked selection with its own list to the level-up's selections (WrathDecompiled
// 6ce4a6b, UnitLogic/Class/LevelUp/Actions/SelectFeature.cs); ExtraRogueTalentSelection
// 14d9089df87a43b696fa9451ca2f0a12 needs RogueTalentSelection c074a5d615200494b8f2a9c845799d93,
// SylvanTricksterTalentSelection 290bbcc3c3bb92144b853fd8fb8ff452 or DarkLurkerRogueTalentSelection
// 1edbb68422c046f98a86f4726f6721b2 (not BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f or SkaldTalentSelection
// d2a8fde8985691045b90e1ec57e3cc57); DarkLurkerArchetype 33eb002078e5437b8c25db4b80b4a4fe removes the rogue's.
#[test]
fn an_extra_feat_opens_its_own_list_for_any_class() {
    let own = Some(selection_kind(slot_selection("extra-rogue-talent")));
    for (build, holds) in [
        (build("rogue", 3), true),
        (archetype_build("rogue", "rogue/dark-lurker", 5), true),
        (classes_build(&times("bard", 3)), false),
        (classes_build(&times("skald", 3)), false),
    ] {
        let level = build.levels.len() as u8;
        let mut build = with_selections_filled(build);
        assert_eq!(
            feat_status(&build, &general_slot(level), "extra-rogue-talent").is_ok(),
            holds,
            "{level}"
        );
        build
            .picks
            .insert(general_slot(level), feat_pick_plain("extra-rogue-talent"));
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            if holds { own.clone() } else { None },
            "{level}"
        );
    }
}

#[test]
fn a_witchs_extra_hex_gives_a_hex_slot() {
    // Game: WitchProgression c5e9e2b086a9814409fb6a29a5acdd0f (WitchHexSelection 9846043cf51251a4897728ed6e24e76f at
    // 1); ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35
    let mut build = with_selections_filled(classes_build(&["witch"]));
    build
        .picks
        .insert(general_slot(1), feat_pick_plain("extra-hex"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(1, 0)),
        Some(selection_kind(slot_selection("extra-hex")))
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(1, 0)));
    assert!(possible(&filled));
}

/// Each feat gives its slot to a character of the class whose selection it names: the feat in the general slot at
/// `level`, after the class's earlier picks.
#[test]
fn each_extra_feat_gives_a_slot_of_its_classs_selection() {
    let cases: [(Build, &str, u8, &str); 17] = [
        // Game: BarbarianProgression 74ad9fbd930a1d84aaf9a230f29f8f12 (RagePowerSelection at 2)
        (barbarian(3), "extra-rage-power", 3, "barbarian/rage-power"),
        // Game: WitchProgression c5e9e2b086a9814409fb6a29a5acdd0f (WitchHexSelection at 1)
        (classes_build(&["witch"]), "extra-hex", 1, "witch/hex"),
        // Game: ShamanProgression d04a543c82b45694c805301bce9dac24 (ShamanHexSelection 4223fe18c75d4d14787af196a04e14e7
        // at 2), ExtraShamanHexSelection d0b4c8245d504b8c9c6d3fccc1f8c5b6 (equivalent model: the shaman's Extra Hex)
        (
            classes_build(&times("shaman", 3)),
            "extra-hex-shaman",
            3,
            "shaman/hex",
        ),
        // Game: MagusProgression 1b912721a7e075d4f9cfe8dafa39414c (MagusArcanaSelection at 3)
        (
            classes_build(&times("magus", 3)),
            "extra-arcana",
            3,
            "magus/magus-arcana",
        ),
        // Game: ArcanistProgression 974911193b8a8b947a8275f96bb265d6 (ArcanistExploitSelection
        // b8bf3d5023f2d8c428fdf6438cecaea7 at 1)
        (
            classes_build(&["arcanist"]),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        // Game: AlchemistProgression efd55ff9be2fda34981f5b9c83afe4f1 (DiscoverySelection cd86c437488386f438dcc9ae727ea2a6
        // at 2)
        (
            classes_build(&times("alchemist", 3)),
            "extra-discovery",
            3,
            "alchemist/discovery",
        ),
        // Game: RogueProgression b57b2a75a5abcaf47a01cf84672b50e9 (RogueTalentSelection at 2)
        (
            classes_build(&times("rogue", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        // Game: SylvanTricksterArchetype 490394869f666c141bf8647b1a365220 (SylvanTricksterTalentSelection
        // 290bbcc3c3bb92144b853fd8fb8ff452 at 2)
        (
            chaotic(archetype_build("rogue", "rogue/sylvan-trickster", 3)),
            "extra-rogue-talent",
            3,
            "rogue/sylvan-trickster/fey-tricks",
        ),
        // Game: SlayerProgression 1f35d11937a06624b8a60db68c7869a1 (SlayerTalentSelection2 04430ad24988baa4daa0bcd4f1c7d118
        // at 2)
        (
            classes_build(&times("slayer", 3)),
            "extra-slayer-talent",
            3,
            "slayer/slayer-talents",
        ),
        // Game: KineticistProgression b79e92dd495edd64e90fb483c504b8df (WildTalentSelection 5c883ae0cd6d7d5448b7a420f51f8459
        // at 2)
        (
            classes_build(&times("kineticist", 3)),
            "extra-wild-talent",
            3,
            "kineticist/wild-talents",
        ),
        // A witch who becomes a shaman: her Extra Hex is a witch's hex, of the selection the feat grants (ticket
        // 306aa), whatever her classes.
        // Game: ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35 gives a witch hex (equivalent model: the game's
        // Extra Hex (Shaman) is another feat she may take instead)
        (
            classes_build(&["witch", "shaman", "shaman"]),
            "extra-hex",
            3,
            "witch/hex",
        ),
        // Ticket 54: the paladin's mercies and the Arcane Enforcer's exploits are picks.
        // Game: PaladinProgression fd325cbba872e5f40b618970678db002 (SelectionMercy 02b187038a8dce545bb34bbfb346428d at
        // 3); ArcaneEnforcerArchetype 23a5d0ca607f6094097d509b2b1f4a10 (ArcanistExploitSelection at 1)
        (paladin(3), "extra-mercy", 3, "paladin/mercy"),
        (
            enforcer(1),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        // Ticket 57: the Exploiter Wizard's exploits are picks too.
        // Game: ExploiterWizardArchetype 6d2027b4eec590c41adb5caeb7bb026d (ExploiterExploitSelection
        // 2ba8a0040e0149e9ae9bfcb01a8ff01d at 1)
        (
            exploiter(1),
            "extra-arcanist-exploit",
            1,
            "arcanist/exploiter-exploits",
        ),
        // Ticket 60: talents taken through another class.
        // Game: FeyformShifterShifterArchetype 1cdfc7d306d1430eac19427539b62091 (RogueTalentSelection at 8)
        (
            chaotic(archetype_build("shifter", "shifter/feyform-shifter", 9)),
            "extra-rogue-talent",
            9,
            "shifter/feyform-shifter/rogue-talent",
        ),
        // Game: SanctifiedSlayerArchetype 19b4a243d9a4bd34887d03c3dea03ced (SanctifiedSlayerSlayerTalentFeature
        // db3767ba22ce84942af56525cb9f9ce6 at 8)
        (
            chaotic(archetype_build(
                "inquisitor",
                "inquisitor/sanctified-slayer",
                9,
            )),
            "extra-slayer-talent",
            9,
            "slayer/slayer-talents-6",
        ),
        // Ticket 64: the Vivisectionist's Medical Discovery.
        // Game: VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 (VivsectionistDiscoverySelection
        // 67f499218a0e22944abab6fe1c9eaeee at 2); ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae
        (
            chaotic(archetype_build("alchemist", "alchemist/vivisectionist", 3)),
            "extra-discovery-vivisectionist",
            3,
            "alchemist/vivisectionist/medical-discovery",
        ),
    ];
    for (build, feat, level, selection) in cases {
        let mut build = with_selections_filled(build);
        build
            .picks
            .insert(general_slot(level), feat_pick_plain(feat));
        assert!(possible(&build), "{feat} at {level}");
        assert_eq!(
            kind_at(&build, &extra_slot(level, 0)),
            Some(selection_kind(slot_selection(feat))),
            "{feat} at {level} (the game's feat needs {selection})"
        );
        let filled = with_selections_filled(build);
        assert!(
            filled.picks.contains_key(&extra_slot(level, 0)),
            "{feat} at {level}: nothing open"
        );
        assert!(possible(&filled), "{feat} at {level}");
    }
}

fn times(class: &'static str, n: usize) -> Vec<&'static str> {
    vec![class; n]
}

/// The selection of the slot an Extra feat gives (ticket 302): the feat's own, which lists its own picks.
fn slot_selection(feat_key: &str) -> &'static str {
    let own = any_selection(
        feat(feat_key)
            .selection
            .as_ref()
            .expect("a selection")
            .as_str(),
    );
    assert!(own.has_list(), "{feat_key}: lists its own picks");
    own.key.as_str()
}

/// The Extra feats the game refuses: Extra Discovery to an Underground Chemist (its discovery selection meets no
/// prerequisite) or a Vivisectionist (which loses the alchemist's Discovery), Extra Rogue Talent to a bard or a skald
/// (their talent selections aren't named), Extra Hex to a Hexcrafter (its hex is its own selection).
// Game: ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66 needs DiscoverySelection
// cd86c437488386f438dcc9ae727ea2a6 only (not UndergroundChemistSelection c3d08fdb481aa2e4a9f71c4349141826;
// VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 `RemoveFeatures` it at 2 and every even level);
// ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 (not
// BardTalentSelection 94e2cd84bf3a8e04f8609fe502892f4f or SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57);
// ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35 needs WitchHexSelection 9846043cf51251a4897728ed6e24e76f (or
// WinterWitchWitchHexSelection b921af3627142bd4d9cf3aefb5e2610a; not HexcrafterMagusHexMagusSelection
// a18b8c3d6251d8641a8094e5c2a7bc78)
#[test]
fn the_extra_feats_the_game_refuses_to_an_underground_chemist_a_bard_a_skald_and_a_hexcrafter() {
    let cases: [(Build, &str, u8); 5] = [
        (
            chaotic(archetype_build("rogue", "rogue/underground-chemist", 11)),
            "extra-discovery",
            11,
        ),
        (
            chaotic(archetype_build("alchemist", VIVISECTIONIST, 5)),
            "extra-discovery",
            5,
        ),
        (classes_build(&times("bard", 3)), "extra-rogue-talent", 3),
        (classes_build(&times("skald", 3)), "extra-rogue-talent", 3),
        (
            chaotic(archetype_build("magus", "magus/hexcrafter", 5)),
            "extra-hex",
            5,
        ),
    ];
    for (build, feat, level) in cases {
        let build = with_selections_filled(build);
        assert!(
            feat_status(&build, &general_slot(level), feat).is_err(),
            "{feat} at {level}"
        );
    }
}

// ---- Extra Discovery and Extra Arcana for the Vivisectionist and the Hexcrafter (ticket 64) -------------------------

/// The keys a slot's picker offers.
fn offered_keys(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    offered_in(build, slot).into_keys().collect()
}

/// The sheet's label of `slot`.
fn label_at(build: &Build, slot: &SlotId) -> String {
    let engine = engine();
    let sheet = engine.evaluate(build);
    engine.slot_label(
        sheet
            .slot(slot)
            .unwrap_or_else(|| panic!("no slot {slot:?}")),
    )
}

/// Ticket 60's decision 2: Extra Discovery gives a Vivisectionist one more Medical Discovery, at the level it's taken
/// (ticket 49's rule: the Vivisectionist has no slot of the alchemist's Discovery, so its first is Medical Discovery's).
#[test]
fn a_vivisectionists_extra_discovery_gives_a_medical_discovery_slot_at_the_level_its_taken() {
    // Game: ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae (needs VivsectionistDiscoverySelection
    // 67f499218a0e22944abab6fe1c9eaeee; its list is the same 25); its own feat since ticket 306k
    let mut build = with_selections_filled(borrower("alchemist", Some(VIVISECTIONIST), 5));
    for level in [3, 5] {
        build.picks.insert(
            general_slot(level),
            feat_pick_plain("extra-discovery-vivisectionist"),
        );
    }
    assert!(possible(&build));
    for level in [3, 5] {
        let slot = extra_slot(level, 0);
        assert_eq!(
            kind_at(&build, &slot),
            Some(selection_kind("alchemist/vivisectionist/extra-discovery")),
            "at {level}"
        );
        assert_eq!(label_at(&build, &slot), "Extra Discovery (Vivisectionist)");
        // What the Vivisectionist's own slot at 2 offers: its discoveries and talents.
        assert_eq!(
            offered_keys(&build, &slot),
            offered_keys(&build, &selection_slot(2))
        );
        assert_eq!(offered_keys(&build, &slot).len(), MEDICAL_DISCOVERIES.len());
    }
    assert_eq!(kind_at(&build, &extra_slot(4, 0)), None);
    let filled = with_selections_filled(build);
    for level in [3, 5] {
        assert!(
            filled.picks.contains_key(&extra_slot(level, 0)),
            "at {level}"
        );
    }
    assert!(possible(&filled));
}

/// Extra Arcana's pick is one of its own 15 magus arcana for every magus: a Hexcrafter's is no hex (ticket 60's
/// decision 3, a Hex Arcana slot, was the guide's reading).
// Game: ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 (its own list of the 15 magus arcana, no hexes)
#[test]
fn a_hexcrafters_extra_arcana_gives_a_magus_arcana() {
    let mut build = with_selections_filled(borrower("magus", Some(HEXCRAFTER), 7));
    build
        .picks
        .insert(general_slot(7), feat_pick_plain("extra-arcana"));
    assert!(possible(&build));
    let slot = extra_slot(7, 0);
    assert_eq!(
        kind_at(&build, &slot),
        Some(selection_kind("magus/extra-arcana"))
    );
    assert_eq!(offered_keys(&build, &slot).len(), 15);
}

/// "Prerequisites: Dicovery" and "Prerequisites: Magus Arcana" name the class feature, and the archetypes' are their
/// version of it (Medical Discovery where the alchemist has Discovery; Hex Arcana, "Besides the usual Magus Arcana
/// abilities"): a Vivisectionist from alchemist 2 and a Hexcrafter from magus 6 meet them whatever they picked there,
/// as a Sanctified Slayer 8 meets Extra Slayer Talent's (ticket 60). Before, neither does.
#[test]
fn extra_discovery_and_extra_arcana_reach_a_vivisectionist_and_a_hexcrafter_whatever_they_picked() {
    // Game: LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b (a progression adds the selection itself
    // as a fact, whatever was picked); ExtraVivsectionistDiscoverySelection 10287e7b8cee479e82ea88bd6d2d4dae needs
    // VivsectionistDiscoverySelection 67f499218a0e22944abab6fe1c9eaeee (VivisectionistArchetype
    // 68cbcd9fbf1fb1d489562f829bb97e38 from 2); ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 is met by
    // HexcrafterMagusHexArcanaSelection ad6b9cecb5286d841a66e23cea3ef7bf (from 6)
    let fast_stealth = rogue_talent("fast-stealth");
    let vivisectionist = with_picks(
        borrower("alchemist", Some(VIVISECTIONIST), 3),
        &[(2, pick(&fast_stealth))],
    );
    assert!(possible(&vivisectionist));
    assert_eq!(
        feat_status(
            &vivisectionist,
            &general_slot(3),
            "extra-discovery-vivisectionist"
        ),
        Ok(())
    );
    let hexcrafter = with_picks(
        borrower("magus", Some(HEXCRAFTER), 7),
        &[
            (4, pick("witch/hex/slumber")),
            (6, pick("witch/hex/cackle")),
        ],
    );
    assert!(possible(&hexcrafter));
    assert_eq!(
        feat_status(&hexcrafter, &general_slot(7), "extra-arcana"),
        Ok(())
    );
    // Before its first slot, the Vivisectionist doesn't. The Hexcrafter 5's: `a_hexcrafter_5_meets_extra_arcana`.
    assert!(
        feat_status(
            &borrower("alchemist", Some(VIVISECTIONIST), 1),
            &general_slot(1),
            "extra-discovery-vivisectionist"
        )
        .is_err()
    );
    // Another class's archetype doesn't: a rogue 2 holding Fast Stealth, then alchemist 1.
    let rogue_then_vivisectionist = with_selections_filled(multiclass(&[
        ("rogue", None, 2),
        ("alchemist", Some(VIVISECTIONIST), 1),
    ]));
    assert!(
        feat_status(
            &rogue_then_vivisectionist,
            &general_slot(3),
            "extra-discovery-vivisectionist"
        )
        .is_err()
    );
}

/// The Hexcrafter keeps the magus arcana at 3, so a Hexcrafter 5 meets Extra Arcana.
// Game: HexcrafterArchetype 79ccf7a306a5d5547bebd97299f6fc89 (removes MagusArcanaSelection
// e9dc4dfc73eaaf94aae27e0ed6cc9ada at 6 to 18 only); ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8;
// LevelUpHelper.AddFeaturesFromProgression, WrathDecompiled 6ce4a6b
#[test]
#[ignore = "waits on ticket 283: the game gives the Hexcrafter its magus arcana at 3 (Extra Arcana's prerequisite is \
            the game's since 308c, so the row is what's left)"]
fn a_hexcrafter_5_meets_extra_arcana() {
    let hexcrafter_5 = with_selections_filled(with_picks(
        borrower("magus", Some(HEXCRAFTER), 5),
        &[(4, pick("witch/hex/slumber"))],
    ));
    assert_eq!(
        feat_status(&hexcrafter_5, &general_slot(5), "extra-arcana"),
        Ok(())
    );
}

/// A plain alchemist and magus: their Extra feat gives a slot of the alchemist's Discovery or the magus's Magus Arcana,
/// offering what their own slots offer. Each needs its class's selection, held from alchemist 2 or magus 3 whatever is
/// picked in it (Extra Discovery since 306aa, Extra Arcana since 308c).
// Game: ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66 (needs DiscoverySelection
// cd86c437488386f438dcc9ae727ea2a6, from alchemist 2; `LevelUpHelper.AddFeaturesFromProgression`, WrathDecompiled
// 6ce4a6b, adds the selection itself as a fact, whatever was picked) and ExtraArcanaSelection
// 00727883edf145e2a6bce9ad176ecfd8 (needs MagusArcanaSelection e9dc4dfc73eaaf94aae27e0ed6cc9ada, from magus 3, or
// the Hexcrafter's or the Eldritch Scion's arcana selection).
#[test]
fn a_plain_alchemists_and_magus_extra_picks_are_unchanged() {
    for (class, feat, own, label) in [
        ("alchemist", "extra-discovery", 2, "Extra Discovery"),
        ("magus", "extra-arcana", 3, "Magus Arcana"),
    ] {
        let empty = borrower(class, None, 3);
        assert_eq!(
            feat_status(&empty, &general_slot(3), feat),
            Ok(()),
            "{feat}: nothing picked"
        );
        // Before the Discovery selection is held (alchemist 1), the game refuses it.
        assert!(
            feat_status(&borrower(class, None, 1), &general_slot(1), feat).is_err(),
            "{feat}: before the selection"
        );
        let mut build = with_selections_filled(empty);
        assert_eq!(
            feat_status(&build, &general_slot(3), feat),
            Ok(()),
            "{feat}"
        );
        build.picks.insert(general_slot(3), feat_pick_plain(feat));
        assert!(possible(&build), "{feat}");
        let slot = extra_slot(3, 0);
        assert_eq!(
            kind_at(&build, &slot),
            Some(selection_kind(slot_selection(feat)))
        );
        assert_eq!(label_at(&build, &slot), label);
        assert_eq!(
            offered_keys(&build, &slot),
            offered_keys(&build, &selection_slot(own)),
            "{feat}"
        );
    }
}
