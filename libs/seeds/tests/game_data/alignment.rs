//! An archetype's alignment requirement replaces its class's (ticket 63).

use std::collections::BTreeSet;

use rules::{ArchetypeKey, Build, CharacterState, ClassKey, Issue, Prerequisite, Unmet};

use crate::support::*;

/// Every archetype with an alignment rule, with what the rule allows, and a race it needs besides (the Reformed
/// Fiend's tiefling). The Mantis Zealot's is the user's fact; the others are the guide's.
const ALIGNMENT_RULES: [(&str, &[rules::Alignment], Option<&str>); 7] = {
    use rules::Alignment::*;
    [
        (
            "bloodrager/reformed-fiend",
            &[LawfulGood, NeutralGood, ChaoticGood],
            Some("tiefling"),
        ),
        (
            "cleric/angelfire-apostle",
            &[LawfulGood, NeutralGood, ChaoticGood],
            None,
        ),
        (
            "cleric/priest-of-balance",
            &[LawfulNeutral, TrueNeutral, ChaoticNeutral],
            None,
        ),
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
        (
            "shaman/prophet-of-pestilence",
            &[TrueNeutral, NeutralEvil, ChaoticEvil],
            None,
        ),
        (
            "slayer/bloodseeker",
            &[LawfulEvil, NeutralEvil, ChaoticEvil],
            None,
        ),
        (
            "warpriest/mantis-zealot",
            &[LawfulNeutral, LawfulEvil, NeutralEvil],
            None,
        ),
    ]
};

/// A Build of `levels` levels of `archetype`'s class taken with it, at `alignment`, with no picks.
fn aligned(archetype: &str, alignment: rules::Alignment, levels: usize) -> Build {
    let class = archetype.split('/').next().unwrap();
    let mut build = archetype_build(class, archetype, levels);
    build.start.alignment = Some(alignment);
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
    let lawful = [LawfulGood, LawfulNeutral, LawfulEvil];
    for alignment in rules::Alignment::ALL {
        let mut plain = monk(1);
        plain.start.alignment = Some(alignment);
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
    let evil = [LawfulEvil, NeutralEvil, ChaoticEvil];
    for alignment in rules::Alignment::ALL {
        let bloodseeker = aligned("slayer/bloodseeker", alignment, 1);
        let mut slayer = build("slayer", 1);
        slayer.start.alignment = Some(alignment);
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
/// Achaekek), so open at those and locked at the rest (Chaotic Evil and Lawful Good among them).
#[test]
fn a_mantis_zealot_is_open_at_le_ln_and_ne_and_locked_at_ce_and_lg() {
    use rules::Alignment::*;
    let allowed = [LawfulNeutral, LawfulEvil, NeutralEvil];
    let zealot = class("warpriest")
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == "warpriest/mantis-zealot")
        .unwrap();
    assert_eq!(
        zealot.prerequisite,
        Some(Prerequisite::Any(
            allowed.map(Prerequisite::Alignment).to_vec()
        ))
    );
    for alignment in rules::Alignment::ALL {
        let build = aligned("warpriest/mantis-zealot", alignment, 1);
        let status = entry_status(&build, "warpriest/mantis-zealot");
        if allowed.contains(&alignment) {
            assert!(possible(&build), "{alignment:?}");
            assert_eq!(status, Ok(()), "{alignment:?}");
        } else {
            assert!(!possible(&build), "{alignment:?}");
            assert_eq!(status, Err(none_of(&allowed)), "{alignment:?}");
        }
    }
    for alignment in [ChaoticEvil, LawfulGood] {
        assert!(!possible(&aligned("warpriest/mantis-zealot", alignment, 1)));
    }
}

/// The ticket's: the archetypes with an alignment rule are these seven; only the Drunken Master's clashes with its
/// class's (the monk's Lawful), and each, the Drunken Master too, is open exactly at the alignments its own rule
/// allows and buildable to 20 there.
#[test]
fn every_archetype_with_an_alignment_rule_is_buildable_at_an_alignment_it_allows() {
    let with_rule: BTreeSet<&str> = DATA
        .classes
        .iter()
        .flat_map(|class| &class.archetypes)
        .filter(|archetype| archetype.prerequisite.as_ref().is_some_and(names_alignment))
        .map(|archetype| archetype.key.as_str())
        .collect();
    let expected: BTreeSet<&str> = ALIGNMENT_RULES.iter().map(|(key, ..)| *key).collect();
    assert_eq!(with_rule, expected);

    let mut clashing = Vec::new();
    for (archetype, allowed, race) in ALIGNMENT_RULES {
        let class_key = archetype.split('/').next().unwrap();
        // The alignments the class alone allows.
        let class_allows: Vec<rules::Alignment> = rules::Alignment::ALL
            .into_iter()
            .filter(|alignment| {
                let state = CharacterState {
                    alignment: Some(*alignment),
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
    assert_eq!(clashing, ["monk/drunken-master"]);
}
