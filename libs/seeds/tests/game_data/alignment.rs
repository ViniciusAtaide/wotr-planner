//! An archetype's alignment requirement replaces its class's (ticket 63).

use std::collections::BTreeSet;

use rules::{ArchetypeKey, Build, CharacterState, ClassKey, Issue, Prerequisite, Unmet};

use crate::support::*;

/// Archetypes with an alignment rule, with what the rule allows, and a race it needs besides (the Reformed Fiend's
/// tiefling). The guide's; the Mantis Zealot's, the user's fact, became Achaekek in ticket 86. The game's rules for
/// these five agree; the game's set of archetypes with a rule differs (`the_archetypes_with_an_alignment_rule_are_the_games_eight`).
const ALIGNMENT_RULES: [(&str, &[rules::Alignment], Option<&str>); 5] = {
    use rules::Alignment::*;
    [
        // Game: ReformedFiendArchetype d55163eed9214367820654f0ebe0ff69 (Good; TieflingRace 5c4e42124dc2b4647af6e36cf2590500)
        (
            "bloodrager/reformed-fiend",
            &[LawfulGood, NeutralGood, ChaoticGood],
            Some("tiefling"),
        ),
        // Game: PriestOfBalanceArchetype a4560e3fb5d247d68fb1a2738fcc0855
        (
            "cleric/priest-of-balance",
            &[LawfulNeutral, TrueNeutral, ChaoticNeutral],
            None,
        ),
        // Game: DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1 (ArchetypeAlignment)
        (
            "monk/drunken-master",
            &[
                NeutralGood,
                ChaoticGood,
                TrueNeutral,
                ChaoticNeutral,
                NeutralEvil,
                ChaoticEvil,
            ],
            None,
        ),
        // Game: ProphetOfPestilenceArchetype bcd758a75fb54651a7f668fe2661a307 (ArchetypeAlignment)
        (
            "shaman/prophet-of-pestilence",
            &[TrueNeutral, NeutralEvil, ChaoticEvil],
            None,
        ),
        // Game: BloodseekerArchetype 11e473bfb29c42f580b9a801b6771788 (Evil, ArchetypeAlignment)
        (
            "slayer/bloodseeker",
            &[LawfulEvil, NeutralEvil, ChaoticEvil],
            None,
        ),
    ]
};

/// The archetypes the game gives an alignment rule (a `PrerequisiteAlignment`) that the data's don't have, with what
/// the rule allows.
const GAME_ONLY_ALIGNMENT_RULES: [(&str, &[rules::Alignment]); 3] = {
    use rules::Alignment::*;
    [
        (
            "kineticist/dark-elementalist",
            &[LawfulEvil, NeutralEvil, ChaoticEvil],
        ), // Game: DarkElementalistArchetype f12f18ae8842425489d29f302e69134c
        (
            "shifter/rageshaper",
            &[
                NeutralGood,
                TrueNeutral,
                NeutralEvil,
                ChaoticGood,
                ChaoticNeutral,
                ChaoticEvil,
            ],
        ), // Game: RageshaperShifterArchetype 41d5b44e421048d09d32623689173e11
        (
            "witch/hag-of-gyronna",
            &[ChaoticNeutral, NeutralEvil, ChaoticEvil],
        ), // Game: DarkSisterArchetype 411fa458481e44f0855d47a19358874b
    ]
};

/// A Build of `levels` levels of `archetype`'s class taken with it, at `alignment`, with no picks.
fn aligned(archetype: &str, alignment: rules::Alignment, levels: usize) -> Build {
    let class = archetype.split('/').next().unwrap();
    let mut build = archetype_build(class, archetype, levels);
    build.start.alignment = Some(alignment);
    // A deity that fits, for a class that serves one (ticket 86); else an atheist, since the alignment needs a
    // faith (ticket 94), for a Build no faith makes possible yet (the Reformed Fiend before its race is set).
    build.start.deity = None;
    let mut build = faithful(build);
    build.start.deity.get_or_insert(rules::Faith::Atheism);
    build
}

/// The level-1 class picker's status for `archetype` (a class key alone: the class without one) in a Build like
/// `build`, before its first level.
fn entry_status(build: &Build, archetype: &str) -> Result<(), Issue> {
    let (class, archetype) = match archetype.split_once('/') {
        Some((class, _)) => (class, Some(ArchetypeKey::new(archetype))),
        None => (archetype, None),
    };
    let mut before = build.clone();
    before.levels.clear();
    before.class_choices.clear();
    engine()
        .class_options(&before, 1)
        .unwrap()
        .into_iter()
        .find(|option| option.class == ClassKey::new(class) && option.archetype == archetype)
        .unwrap_or_else(|| panic!("no class option {class} {archetype:?}"))
        .status
}

/// Whether an `Alignment` appears anywhere in a prerequisite.
fn names_alignment(prerequisite: &Prerequisite) -> bool {
    prerequisite
        .leaves()
        .any(|leaf| matches!(leaf, Prerequisite::Alignment(_)))
}

/// What an `Any` of alignments reads as when none holds.
fn none_of(alignments: &[rules::Alignment]) -> Issue {
    Issue::Unmet(Unmet::Any(
        alignments
            .iter()
            .map(|alignment| Unmet::Condition {
                needs: Prerequisite::Alignment(*alignment),
                has: None,
            })
            .collect(),
    ))
}

/// The ticket's: a Chaotic Neutral monk 1 can take Drunken Master, and so can a monk 20 of any non-lawful alignment.
#[test]
fn a_chaotic_neutral_monk_1_can_take_drunken_master() {
    // Game: DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1 (NeutralGood, TrueNeutral, NeutralEvil, Chaotic)
    let build = aligned("monk/drunken-master", rules::Alignment::ChaoticNeutral, 1);
    assert!(possible(&build));
    assert_eq!(entry_status(&build, "monk/drunken-master"), Ok(()));
    let filled = with_selections_filled(aligned(
        "monk/drunken-master",
        rules::Alignment::NeutralEvil,
        20,
    ));
    assert!(possible(&filled));
}

/// The ticket's: a Lawful Neutral one can't, and the reason is the archetype's alignment, not the monk's.
#[test]
fn a_lawful_neutral_drunken_master_is_locked_by_the_archetypes_alignment() {
    use rules::Alignment::*;
    let build = aligned("monk/drunken-master", LawfulNeutral, 1);
    assert!(!possible(&build));
    // Game: PrerequisiteAlignment.RestrictInternal, WrathDecompiled 6ce4a6b (ArchetypeAlignment replaces the class's
    // restriction); DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1
    let status = entry_status(&build, "monk/drunken-master");
    let not_lawful = [
        NeutralGood,
        ChaoticGood,
        TrueNeutral,
        ChaoticNeutral,
        NeutralEvil,
        ChaoticEvil,
    ];
    assert_eq!(status, Err(none_of(&not_lawful)));
    assert_eq!(
        engine().issue_text(&status.unwrap_err()).unwrap(),
        "Needs Neutral Good alignment or Chaotic Good alignment or True Neutral alignment or Chaotic Neutral \
         alignment or Neutral Evil alignment or Chaotic Evil alignment"
    );
}

/// The ticket's: a monk without an archetype, or with one stating no alignment, still needs a lawful one.
#[test]
fn a_plain_monk_still_needs_a_lawful_alignment() {
    use rules::Alignment::*;
    // Game: MonkClass e8f21e5b58e0569468e420ebea456124 (Lawful); ZenArcherArchetype 2b1a58a7917084f49b097e86271df21c
    // has no alignment rule
    let lawful = [LawfulGood, LawfulNeutral, LawfulEvil];
    for alignment in rules::Alignment::ALL {
        let mut plain = monk(1);
        align(&mut plain, alignment);
        let zen = aligned("monk/zen-archer", alignment, 1);
        if lawful.contains(&alignment) {
            assert!(possible(&plain) && possible(&zen), "{alignment:?}");
            assert_eq!(entry_status(&plain, "monk"), Ok(()));
        } else {
            assert!(!possible(&plain) && !possible(&zen), "{alignment:?}");
            assert_eq!(entry_status(&plain, "monk"), Err(none_of(&lawful)));
            assert_eq!(entry_status(&zen, "monk/zen-archer"), Err(none_of(&lawful)));
        }
    }
}

/// The ticket's: a Bloodseeker ("Alignment must be evil") needs an evil alignment; the slayer has no rule of its own.
#[test]
fn a_bloodseeker_needs_an_evil_alignment() {
    use rules::Alignment::*;
    // Game: BloodseekerArchetype 11e473bfb29c42f580b9a801b6771788 (Evil); SlayerClass c75e0971973957d4dbad24bc7957e4fb
    // has no alignment rule
    let evil = [LawfulEvil, NeutralEvil, ChaoticEvil];
    for alignment in rules::Alignment::ALL {
        let bloodseeker = aligned("slayer/bloodseeker", alignment, 1);
        let mut slayer = build("slayer", 1);
        align(&mut slayer, alignment);
        assert!(possible(&slayer), "{alignment:?}");
        if evil.contains(&alignment) {
            assert!(possible(&bloodseeker), "{alignment:?}");
            assert_eq!(entry_status(&bloodseeker, "slayer/bloodseeker"), Ok(()));
        } else {
            assert!(!possible(&bloodseeker), "{alignment:?}");
            assert_eq!(
                entry_status(&bloodseeker, "slayer/bloodseeker"),
                Err(none_of(&evil))
            );
        }
    }
}

/// The ticket's, the user's fact: a Mantis Zealot is Lawful Evil, Lawful Neutral or Neutral Evil (one step from
/// Achaekek), so open at those and locked at the rest (Chaotic Evil and Lawful Good among them). Since ticket 86 the
/// rule is Achaekek, whose alignments are those three.
#[test]
fn a_mantis_zealot_is_open_at_le_ln_and_ne_and_locked_at_ce_and_lg() {
    use rules::Alignment::*;
    // Game: MantisZealotArchetype fc403240fdca4a52a413578169ea7117 (needs AchaekekFeature); AchaekekFeature
    // a3189d5b7c4d4d91beaa8bfffac3e38e (LawfulNeutral, LawfulEvil, NeutralEvil)
    let allowed = [LawfulNeutral, LawfulEvil, NeutralEvil];
    let zealot = class("warpriest")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "warpriest/mantis-zealot")
        .unwrap();
    assert_eq!(
        zealot.prerequisite,
        Some(Prerequisite::Deity(rules::DeityKey::new("achaekek")))
    );
    for alignment in rules::Alignment::ALL {
        let build = aligned("warpriest/mantis-zealot", alignment, 1);
        let status = entry_status(&build, "warpriest/mantis-zealot");
        if allowed.contains(&alignment) {
            assert!(possible(&build), "{alignment:?}");
            assert_eq!(status, Ok(()), "{alignment:?}");
        } else {
            assert!(!possible(&build), "{alignment:?}");
            assert!(status.is_err(), "{alignment:?}");
        }
    }
    for alignment in [ChaoticEvil, LawfulGood] {
        assert!(!possible(&aligned("warpriest/mantis-zealot", alignment, 1)));
    }
}

/// The archetypes whose own prerequisite names an alignment.
fn archetypes_with_an_alignment_rule() -> BTreeSet<&'static str> {
    DATA.archetypes()
        .filter(|archetype| archetype.prerequisite.as_ref().is_some_and(names_alignment))
        .map(|archetype| archetype.key.as_str())
        .collect()
}

/// The ticket's: the archetypes with an alignment rule are these; only the Drunken Master's rule clashes with its class's (the monk's Lawful), and each archetype of
/// `ALIGNMENT_RULES`, the Drunken Master too, is open exactly at the alignments its own rule allows and buildable to 20
/// there.
#[test]
fn every_archetype_with_an_alignment_rule_is_buildable_at_an_alignment_it_allows() {
    // Leaving out the four archetypes whose rule waits on a ticket (the Angelfire Apostle's, which the game doesn't
    // have, and the three of GAME_ONLY_ALIGNMENT_RULES: `the_archetypes_with_an_alignment_rule_are_the_games_eight`),
    // the archetypes with a rule are these five.
    let waiting: BTreeSet<&str> = GAME_ONLY_ALIGNMENT_RULES
        .iter()
        .map(|(key, _)| *key)
        .chain(["cleric/angelfire-apostle"])
        .collect();
    let expected: BTreeSet<&str> = ALIGNMENT_RULES.iter().map(|(key, ..)| *key).collect();
    assert_eq!(&archetypes_with_an_alignment_rule() - &waiting, expected);

    let mut clashing = Vec::new();
    for (archetype, allowed, race) in ALIGNMENT_RULES {
        let class_key = archetype.split('/').next().unwrap();
        // The alignments the class alone allows.
        let class_allows: Vec<rules::Alignment> = rules::Alignment::ALL
            .into_iter()
            .filter(|alignment| {
                // A deity, for a class that serves one.
                let state = CharacterState {
                    alignment: Some(*alignment),
                    faith: Some(rules::Faith::Deity(rules::DeityKey::new("pharasma"))),
                    ..CharacterState::default()
                };
                class(class_key)
                    .prerequisite
                    .as_ref()
                    .is_none_or(|p| state.check(p).is_ok())
            })
            .collect();
        if !allowed.iter().any(|a| class_allows.contains(a)) {
            clashing.push(archetype);
        }
        for alignment in rules::Alignment::ALL {
            let mut build = aligned(archetype, alignment, 1);
            build.start.race = race.map(rules::RaceKey::new);
            let open = entry_status(&build, archetype).is_ok();
            assert_eq!(
                open,
                allowed.contains(&alignment),
                "{archetype} {alignment:?}"
            );
            assert_eq!(possible(&build), open, "{archetype} {alignment:?}");
        }
        let mut full = aligned(archetype, allowed[0], 20);
        full.start.race = race.map(rules::RaceKey::new);
        assert!(possible(&full), "{archetype}");
        assert!(
            possible(&with_selections_filled(full)),
            "{archetype} filled"
        );
    }
    // Game: DrunkenMasterArchetype a8532f88b704413ba68f62e4e7ff84c1 against MonkClass e8f21e5b58e0569468e420ebea456124
    // (Lawful): the only one of these five whose rule shares no alignment with its class's
    assert_eq!(clashing, ["monk/drunken-master"]);
}

/// The game gives eight archetypes an alignment rule: the five of `ALIGNMENT_RULES`, the Dark Elementalist, the
/// Rageshaper and the Hag of Gyronna. The Angelfire Apostle has none (its deity decides). Each of the three is open
/// exactly at the alignments its rule allows.
// Game: GAME_ONLY_ALIGNMENT_RULES' rows; AngelfireApostleArchetype 857bc9fadf70f294795a9cba974a48b8 has no
// PrerequisiteAlignment
#[test]
#[ignore = "waits on ticket 267: the game gives Dark Elementalist, Rageshaper and Hag of Gyronna an alignment rule and Angelfire Apostle none"]
fn the_archetypes_with_an_alignment_rule_are_the_games_eight() {
    let expected: BTreeSet<&str> = ALIGNMENT_RULES
        .iter()
        .map(|(key, ..)| *key)
        .chain(GAME_ONLY_ALIGNMENT_RULES.iter().map(|(key, _)| *key))
        .collect();
    assert_eq!(archetypes_with_an_alignment_rule(), expected);
    for (archetype, allowed) in GAME_ONLY_ALIGNMENT_RULES {
        for alignment in rules::Alignment::ALL {
            let build = aligned(archetype, alignment, 1);
            assert_eq!(
                entry_status(&build, archetype).is_ok(),
                allowed.contains(&alignment),
                "{archetype} {alignment:?}"
            );
        }
    }
}

/// The Angelfire Apostle has no alignment rule of its own: Torag, who allows Lawful Neutral and doesn't exclude the
/// archetype, makes a Lawful Neutral one possible.
// Game: AngelfireApostleArchetype 857bc9fadf70f294795a9cba974a48b8 (no PrerequisiteAlignment); ToragFeature
// d2d5c5a58885a6b489727467e13c3337 (no PrerequisiteNoArchetype for it)
#[test]
#[ignore = "waits on ticket 267: the game gives the Angelfire Apostle no alignment rule, so a Lawful Neutral one of Torag is possible"]
fn a_lawful_neutral_angelfire_apostle_of_torag_is_possible() {
    let mut build = archetype_build("cleric", "cleric/angelfire-apostle", 1);
    build.start.alignment = Some(rules::Alignment::LawfulNeutral);
    build.start.deity = Some(rules::Faith::Deity(rules::DeityKey::new("torag")));
    assert!(possible(&build));
}
