//! Extra feats' extra picks (ticket 49), and Extra Discovery and Extra Arcana for the Vivisectionist and the
//! Hexcrafter (ticket 64).

use std::collections::BTreeSet;

use rules::{Build, SlotId};

use crate::support::*;

// ---- Extra feats' extra picks (ticket 49) ---------------------------------------------------------------------

/// The "Extra …" feats whose guide entry gives one more pick of a class selection ("You gain one additional hex"),
/// with the selections that pick may be of: each one the guide's "Prerequisites" name. Extra Mercy's since ticket 54
/// made the paladin's mercies a selection; the Vivisectionist's Medical Discovery and the Hexcrafter's Hex Arcana since
/// ticket 64.
const EXTRA_PICKS: [(&str, &[&str]); 9] = [
    (
        "extra-arcana",
        &["magus/hexcrafter/hex-arcana", "magus/magus-arcana"],
    ),
    ("extra-arcanist-exploit", &["arcanist/arcanist-exploits"]),
    (
        "extra-discovery",
        &[
            "alchemist/discovery",
            "alchemist/vivisectionist/medical-discovery",
            "rogue/underground-chemist/discovery",
        ],
    ),
    ("extra-hex", &["shaman/hex", "witch/hex"]),
    ("extra-mercy", &["paladin/mercy"]),
    (
        "extra-rage-power",
        &[
            "barbarian/rage-power",
            "skald/battle-scion/rage-power-or-teamwork-feat",
        ],
    ),
    (
        "extra-rogue-talent",
        &[
            "rogue/rogue-talents",
            "rogue/sylvan-trickster/fey-tricks",
            "shifter/feyform-shifter/rogue-talent",
        ],
    ),
    ("extra-slayer-talent", &["slayer/slayer-talents"]),
    ("extra-wild-talent", &["kineticist/wild-talents"]),
];

#[test]
fn the_extra_feats_that_give_a_pick_name_their_selections() {
    let named: BTreeSet<(String, BTreeSet<String>)> = DATA
        .feats
        .iter()
        .filter(|feat| !feat.extra_pick_of.is_empty())
        .map(|feat| {
            (
                feat.key.to_string(),
                feat.extra_pick_of.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect();
    let expected: BTreeSet<(String, BTreeSet<String>)> = EXTRA_PICKS
        .iter()
        .map(|(feat, selections)| {
            (
                feat.to_string(),
                selections.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(named, expected);
    // The resource ones give no pick.
    for key in [
        "extra-arcane-pool",
        "extra-bane-weapon",
        "extra-bloodrage",
        "extra-bombs",
        "extra-channel",
        "extra-ki",
        "extra-lay-on-hands",
        "extra-performance",
        "extra-rage",
        "extra-reservoir",
    ] {
        assert!(feat(key).extra_pick_of.is_empty(), "{key}");
    }
}

#[test]
fn a_barbarian_with_extra_rage_power_at_3_gets_a_rage_power_slot_there() {
    let engine = engine();
    let mut build = with_selections_filled(barbarian(3));
    build
        .picks
        .insert(general_slot(3), feat_pick_plain("extra-rage-power"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(3, 0)),
        Some(selection_kind("barbarian/rage-power"))
    );
    // Its options are the barbarian's rage powers, as the rage power slot at 2 offers them.
    let keys = |slot: &SlotId| -> BTreeSet<String> {
        engine
            .options(&build, slot)
            .unwrap()
            .into_iter()
            .map(|option| option.pick.key().to_owned())
            .collect()
    };
    assert_eq!(keys(&extra_slot(3, 0)), keys(&selection_slot(2)));
    // 33, and the Instinctual Warrior's five monk powers (ticket 66), locked here.
    assert_eq!(keys(&extra_slot(3, 0)).len(), 38);
    let sheet = engine.evaluate(&build);
    let slot = sheet.slot(&extra_slot(3, 0)).unwrap();
    assert_eq!(engine.slot_label(slot), "Rage Power");

    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(3, 0)));
    assert!(possible(&filled));
}

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
            Some(selection_kind("barbarian/rage-power")),
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

#[test]
fn a_witchs_extra_hex_gives_a_hex_slot() {
    let mut build = with_selections_filled(classes_build(&["witch"]));
    build
        .picks
        .insert(general_slot(1), feat_pick_plain("extra-hex"));
    assert!(possible(&build));
    assert_eq!(
        kind_at(&build, &extra_slot(1, 0)),
        Some(selection_kind("witch/hex"))
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&extra_slot(1, 0)));
    assert!(possible(&filled));
}

/// Each feat gives its slot to a character of the class whose selection it names: the feat in the general slot at
/// `level`, after the class's earlier picks.
#[test]
fn each_extra_feat_gives_a_slot_of_its_classs_selection() {
    let cases: [(Build, &str, u8, &str); 22] = [
        (barbarian(3), "extra-rage-power", 3, "barbarian/rage-power"),
        (classes_build(&["witch"]), "extra-hex", 1, "witch/hex"),
        (
            classes_build(&times("shaman", 3)),
            "extra-hex",
            3,
            "shaman/hex",
        ),
        (
            classes_build(&times("magus", 3)),
            "extra-arcana",
            3,
            "magus/magus-arcana",
        ),
        (
            classes_build(&["arcanist"]),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        (
            classes_build(&times("alchemist", 3)),
            "extra-discovery",
            3,
            "alchemist/discovery",
        ),
        (
            chaotic(archetype_build("rogue", "rogue/underground-chemist", 11)),
            "extra-discovery",
            11,
            "rogue/underground-chemist/discovery",
        ),
        (
            classes_build(&times("rogue", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        (
            chaotic(archetype_build("rogue", "rogue/sylvan-trickster", 3)),
            "extra-rogue-talent",
            3,
            "rogue/sylvan-trickster/fey-tricks",
        ),
        (
            classes_build(&times("slayer", 3)),
            "extra-slayer-talent",
            3,
            "slayer/slayer-talents",
        ),
        (
            classes_build(&times("kineticist", 3)),
            "extra-wild-talent",
            3,
            "kineticist/wild-talents",
        ),
        // A witch who becomes a shaman: her Extra Hex is a witch's hex, the selection she had a slot of first.
        (
            classes_build(&["witch", "shaman", "shaman"]),
            "extra-hex",
            3,
            "witch/hex",
        ),
        // Ticket 54: the paladin's mercies and the Arcane Enforcer's exploits are picks.
        (paladin(3), "extra-mercy", 3, "paladin/mercy"),
        (
            enforcer(1),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        // Ticket 57: the Exploiter Wizard's exploits are picks too.
        (
            exploiter(1),
            "extra-arcanist-exploit",
            1,
            "arcanist/arcanist-exploits",
        ),
        // Ticket 60: talents and hexes taken through another class.
        (
            classes_build(&times("bard", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        (
            classes_build(&times("skald", 3)),
            "extra-rogue-talent",
            3,
            "rogue/rogue-talents",
        ),
        (
            chaotic(archetype_build("shifter", "shifter/feyform-shifter", 9)),
            "extra-rogue-talent",
            9,
            "shifter/feyform-shifter/rogue-talent",
        ),
        (
            chaotic(archetype_build(
                "inquisitor",
                "inquisitor/sanctified-slayer",
                9,
            )),
            "extra-slayer-talent",
            9,
            "slayer/slayer-talents",
        ),
        (
            chaotic(archetype_build("magus", "magus/hexcrafter", 5)),
            "extra-hex",
            5,
            "witch/hex",
        ),
        // Ticket 64: the Vivisectionist's Medical Discovery and the Hexcrafter's Hex Arcana.
        (
            chaotic(archetype_build("alchemist", "alchemist/vivisectionist", 3)),
            "extra-discovery",
            3,
            "alchemist/vivisectionist/medical-discovery",
        ),
        (
            chaotic(archetype_build("magus", "magus/hexcrafter", 7)),
            "extra-arcana",
            7,
            "magus/hexcrafter/hex-arcana",
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
            Some(selection_kind(selection)),
            "{feat} at {level}"
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
    let mut build = with_selections_filled(borrower("alchemist", Some(VIVISECTIONIST), 5));
    for level in [3, 5] {
        build
            .picks
            .insert(general_slot(level), feat_pick_plain("extra-discovery"));
    }
    assert!(possible(&build));
    for level in [3, 5] {
        let slot = extra_slot(level, 0);
        assert_eq!(
            kind_at(&build, &slot),
            Some(selection_kind(MEDICAL_DISCOVERY)),
            "at {level}"
        );
        assert_eq!(label_at(&build, &slot), "Medical Discovery");
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

/// Ticket 60's decision 3: Extra Arcana gives a Hexcrafter one more Hex Arcana, the arcana and the hexes, a major hex
/// from magus 12 as in its own slots.
#[test]
fn a_hexcrafters_extra_arcana_gives_a_hex_arcana_slot() {
    let mut build = with_selections_filled(borrower("magus", Some(HEXCRAFTER), 7));
    build
        .picks
        .insert(general_slot(7), feat_pick_plain("extra-arcana"));
    assert!(possible(&build));
    let slot = extra_slot(7, 0);
    assert_eq!(kind_at(&build, &slot), Some(selection_kind(HEX_ARCANA)));
    assert_eq!(label_at(&build, &slot), "Hex Arcana");
    assert_eq!(
        offered_keys(&build, &slot),
        offered_keys(&build, &selection_slot(6))
    );
    assert_eq!(offered_keys(&build, &slot).len(), 41);
    assert_eq!(
        reason_in(&build, &slot, "witch/hex/agony"),
        "Needs Magus level 12 (have 7)"
    );
    let filled = with_selections_filled(build);
    assert!(filled.picks.contains_key(&slot));
    assert!(possible(&filled));
}

/// "Prerequisites: Dicovery" and "Prerequisites: Magus Arcana" name the class feature, and the archetypes' are their
/// version of it (Medical Discovery where the alchemist has Discovery; Hex Arcana, "Besides the usual Magus Arcana
/// abilities"): a Vivisectionist from alchemist 2 and a Hexcrafter from magus 6 meet them whatever they picked there,
/// as a Sanctified Slayer 8 meets Extra Slayer Talent's (ticket 60). Before, neither does.
#[test]
fn extra_discovery_and_extra_arcana_reach_a_vivisectionist_and_a_hexcrafter_whatever_they_picked() {
    let fast_stealth = rogue_talent("fast-stealth");
    let vivisectionist = with_picks(
        borrower("alchemist", Some(VIVISECTIONIST), 3),
        &[(2, pick(&fast_stealth))],
    );
    assert!(possible(&vivisectionist));
    assert_eq!(
        feat_status(&vivisectionist, &general_slot(3), "extra-discovery"),
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
    // Before their first slot, neither.
    assert!(
        feat_status(
            &borrower("alchemist", Some(VIVISECTIONIST), 1),
            &general_slot(1),
            "extra-discovery"
        )
        .is_err()
    );
    let hexcrafter_5 = with_picks(
        borrower("magus", Some(HEXCRAFTER), 5),
        &[(4, pick("witch/hex/slumber"))],
    );
    assert!(feat_status(&hexcrafter_5, &general_slot(5), "extra-arcana").is_err());
    // Another class's archetype doesn't: a rogue 2 holding Fast Stealth, then alchemist 1.
    let rogue_then_vivisectionist = with_selections_filled(multiclass(&[
        ("rogue", None, 2),
        ("alchemist", Some(VIVISECTIONIST), 1),
    ]));
    assert!(
        feat_status(
            &rogue_then_vivisectionist,
            &general_slot(3),
            "extra-discovery"
        )
        .is_err()
    );
}

/// A plain alchemist and magus are as before: their Extra feat needs a discovery or an arcana held, and gives a slot of
/// the alchemist's Discovery or the magus's Magus Arcana, offering what their own slots offer.
#[test]
fn a_plain_alchemists_and_magus_extra_picks_are_unchanged() {
    for (class, feat, own, selection, label) in [
        (
            "alchemist",
            "extra-discovery",
            2,
            "alchemist/discovery",
            "Discovery",
        ),
        (
            "magus",
            "extra-arcana",
            3,
            "magus/magus-arcana",
            "Magus Arcana",
        ),
    ] {
        let empty = borrower(class, None, 3);
        assert!(
            feat_status(&empty, &general_slot(3), feat).is_err(),
            "{feat}: nothing held"
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
        assert_eq!(kind_at(&build, &slot), Some(selection_kind(selection)));
        assert_eq!(label_at(&build, &slot), label);
        assert_eq!(
            offered_keys(&build, &slot),
            offered_keys(&build, &selection_slot(own)),
            "{feat}"
        );
    }
}
