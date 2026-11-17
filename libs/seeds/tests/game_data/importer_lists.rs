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
