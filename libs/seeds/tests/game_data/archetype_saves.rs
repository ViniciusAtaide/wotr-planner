//! An archetype's save progressions (ticket 90): the Arcane Enforcer has good Reflex and Will and poor Fortitude
//! ("High saving throws for Reflex and Will, low saving throws for Fortitude"), where the slayer has good
//! Fortitude and Reflex and poor Will.

use rules::{CharacterState, ClassKey, SaveProgression};

use crate::support::*;

fn saves(state: &CharacterState) -> (u8, u8, u8) {
    (state.fort, state.reflex, state.will)
}

#[test]
fn the_arcane_enforcer_has_good_reflex_and_will_and_poor_fortitude() {
    let enforcer = enforcer(13);
    // Poor ⌊L/3⌋, good 2 + ⌊L/2⌋.
    // Game: SavesLow dc0c7c1aba755c54f96c089cdf7d14a3 and SavesHigh ff4662bde9e75f145853417313842751 (0/2 at 1, 4/8 at
    // 13); ArcaneEnforcerArchetype 23a5d0ca607f6094097d509b2b1f4a10 (Fortitude SavesLow, Will SavesHigh, Reflex the
    // slayer's); ClassData.FortitudeSave (the archetype's when set), WrathDecompiled 6ce4a6b
    assert_eq!(saves(&at(&enforcer, 1)), (0, 2, 2));
    assert_eq!(saves(&at(&enforcer, 13)), (4, 8, 8));
}

#[test]
fn a_plain_slayer_keeps_good_fortitude_and_reflex_and_poor_will() {
    let slayer = build("slayer", 13);
    // Game: SlayerClass c75e0971973957d4dbad24bc7957e4fb (Fortitude and Reflex SavesHigh, Will SavesLow)
    assert_eq!(saves(&at(&slayer, 1)), (2, 2, 0));
    assert_eq!(saves(&at(&slayer, 13)), (8, 8, 4));
}

/// The Enforcer Build from ticket 88's QA, at level 1 (the ticket's repro link), which read Fort +2 / Ref +2 /
/// Will +0.
#[test]
fn the_qa_enforcer_build_has_the_arcane_enforcers_saves() {
    let build = engine()
        .open_link(
            "AQFQAK__DFwlwpHwUQ-GdkLAnXq5_aYx6Q-Gdo0MN_9IBaYx6WEsyN0dGAABAAEDAQEQEA4KCAwBAAMBAQEBAQAAAAIAAgEAAAABAAIAAAABAQEAAQA",
        )
        .expect("the QA link opens");
    let at_1 = at(&build, 1);
    assert_eq!(at_1.class_level(&ClassKey::new("slayer")), 1);
    // Game: ArcaneEnforcerArchetype 23a5d0ca607f6094097d509b2b1f4a10 (as above; the link is not a game fact)
    assert_eq!(saves(&at_1), (0, 2, 2));
}

/// Every archetype's own save progressions, as the data has them, by key.
fn replacing() -> Vec<SavesOf> {
    let mut replacing: Vec<SavesOf> = DATA
        .classes
        .iter()
        .flat_map(|class| DATA.archetypes_of(&class.key))
        .filter(|archetype| {
            archetype.fort.is_some() || archetype.reflex.is_some() || archetype.will.is_some()
        })
        .map(|archetype| {
            (
                archetype.key.as_str(),
                archetype.fort,
                archetype.reflex,
                archetype.will,
            )
        })
        .collect();
    replacing.sort_by_key(|(key, ..)| *key);
    replacing
}

/// An archetype's key and its own Fortitude, Reflex and Will progressions (none: the class's).
type SavesOf = (
    &'static str,
    Option<SaveProgression>,
    Option<SaveProgression>,
    Option<SaveProgression>,
);

/// The Arcane Enforcer replaces its Fortitude and Will progressions and keeps the slayer's Reflex, and every
/// archetype the data gives saves of its own is one of the game's (the three it lacks wait on a ticket).
#[test]
fn the_arcane_enforcer_replaces_its_fortitude_and_will_progressions() {
    // Game: ArcaneEnforcerArchetype 23a5d0ca607f6094097d509b2b1f4a10 (m_FortitudeSave SavesLow, m_WillSave SavesHigh,
    // no m_ReflexSave)
    let replacing = replacing();
    assert!(replacing.contains(&(
        "slayer/arcane-enforcer",
        Some(SaveProgression::Poor),
        None,
        Some(SaveProgression::Good)
    )));
    let game = game_replacing();
    for saves in &replacing {
        assert!(game.contains(saves), "{saves:?}");
    }
}

/// Four archetypes replace a save progression of their class's: the Arcane Enforcer, the Dark Elementalist (poor
/// Fortitude and Reflex, good Will, against the kineticist's good, good, poor), the Priest of Balance (poor
/// Fortitude, against the cleric's good) and the Traditional Monk (good Will, against the monk's poor).
// Game: ArcaneEnforcerArchetype 23a5d0ca607f6094097d509b2b1f4a10; DarkElementalistArchetype
// f12f18ae8842425489d29f302e69134c (KineticistClass 42a455d9ec1ad924d889272429eb8391); PriestOfBalanceArchetype
// a4560e3fb5d247d68fb1a2738fcc0855 (ClericClass 67819271767a9dd4fbfd4ae700befea0); TraditionalMonk
// 8f3242d209994884bbb6c0999660f4df (MonkClass e8f21e5b58e0569468e420ebea456124)
#[test]
fn four_archetypes_replace_a_save_progression() {
    assert_eq!(replacing(), game_replacing());
}

/// The game's archetypes with save progressions of their own, by key (cited above).
fn game_replacing() -> [SavesOf; 4] {
    use SaveProgression::{Good, Poor};
    [
        ("cleric/priest-of-balance", Some(Poor), None, None),
        (
            "kineticist/dark-elementalist",
            Some(Poor),
            Some(Poor),
            Some(Good),
        ),
        ("monk/traditional-monk", None, None, Some(Good)),
        ("slayer/arcane-enforcer", Some(Poor), None, Some(Good)),
    ]
}
