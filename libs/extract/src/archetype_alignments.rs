//! Ticket 63: the archetypes' alignment rules. An archetype's alignment requirement replaces its class's (the user,
//! 2026-11-02; `rules` does that), so each archetype the guide gives one has it, and no other archetype has any:
//!
//! - five were in the data already (the Drunken Master, Bloodseeker, Angelfire Apostle, Reformed Fiend and Prophet of
//!   Pestilence), each from its section's line ([`RULES`]);
//! - the **Priest of Balance**'s "Alignement restricted to Lawful Neutral, Neutral and Chaotic Neutral" (so spelt in
//!   the guide, which the requirement reader passes over) is its prerequisite, stated where the archetype is built
//!   ([`provide`]);
//! - the **Mantis Zealot** is the user's fact (2026-11-02, ticket 63's decision 1): its section says only that it
//!   worships Achaekek, and the user gave the alignments one step from Achaekek's Lawful Evil, Lawful Neutral or
//!   Neutral Evil; stated where the archetype is built too.
//!
//! Each rule asserts the guide phrase it rests on where it's read, and [`check`] what it finds in the finished data, so
//! it fails loudly on data it doesn't expect.

use std::collections::BTreeSet;

use rules::{Alignment, ArchetypeKey, CharacterState, DataAt, GameData, Prerequisite};

use crate::follow_ups::says;
use crate::picks::Picks;

use Alignment::*;

/// Who says an archetype's alignment rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// In the data already, from the guide's line.
    Data,
    /// The guide's line, added here.
    Guide,
    /// The user's fact; the guide's line is what the section says instead.
    User,
}

/// An archetype's alignment rule: its key, its page and anchor, the line its section says, who says the rule, and the
/// alignments it allows.
pub type Rule = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Source,
    &'static [Alignment],
);

/// Each archetype with an alignment rule.
pub const RULES: [Rule; 7] = [
    (
        "bloodrager/reformed-fiend",
        "bloodrager",
        "reformed-fiend",
        "Alignment must be good",
        Source::Data,
        &[LawfulGood, NeutralGood, ChaoticGood],
    ),
    (
        "cleric/angelfire-apostle",
        "cleric",
        "angelfire-apostle",
        "Alignment must be good",
        Source::Data,
        &[LawfulGood, NeutralGood, ChaoticGood],
    ),
    (
        "cleric/priest-of-balance",
        "cleric",
        "priest-of-balance",
        "Alignement restricted to Lawful Neutral, Neutral and Chaotic Neutral",
        Source::Guide,
        &[LawfulNeutral, TrueNeutral, ChaoticNeutral],
    ),
    (
        "monk/drunken-master",
        "monk",
        "drunken-master",
        "Cannot be Lawful",
        Source::Data,
        &[
            NeutralGood,
            ChaoticGood,
            TrueNeutral,
            ChaoticNeutral,
            NeutralEvil,
            ChaoticEvil,
        ],
    ),
    (
        "shaman/prophet-of-pestilence",
        "shaman",
        "prophet-of-pestilence",
        "Alignment must be Neutral, Neutral Evil or Chaotic Evil",
        Source::Data,
        &[TrueNeutral, NeutralEvil, ChaoticEvil],
    ),
    (
        "slayer/bloodseeker",
        "slayer",
        "bloodseeker",
        "Alignment must be evil",
        Source::Data,
        &[LawfulEvil, NeutralEvil, ChaoticEvil],
    ),
    (
        "warpriest/mantis-zealot",
        "warpriest",
        "mantis-zealot",
        "May only select Achaekek as a deity",
        Source::User,
        &[LawfulEvil, LawfulNeutral, NeutralEvil],
    ),
];

/// Every alignment, in the game's order.
const EVERY: [Alignment; 9] = [
    LawfulGood,
    NeutralGood,
    ChaoticGood,
    LawfulNeutral,
    TrueNeutral,
    ChaoticNeutral,
    LawfulEvil,
    NeutralEvil,
    ChaoticEvil,
];

/// Whether a prerequisite is about alignment alone: an `Alignment`, or a group (or `Not`) of nothing else.
fn only_alignment(prerequisite: &Prerequisite) -> bool {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => {
            !children.is_empty() && children.iter().all(only_alignment)
        }
        Prerequisite::Not(child) => only_alignment(child),
        Prerequisite::Alignment(_) => true,
        _ => false,
    }
}

/// Whether an `Alignment` appears anywhere in a prerequisite.
fn names_alignment(prerequisite: &Prerequisite) -> bool {
    match prerequisite {
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children.iter().any(names_alignment),
        Prerequisite::Not(child) => names_alignment(child),
        Prerequisite::Alignment(_) => true,
        _ => false,
    }
}

/// The alignments a prerequisite's alignment rule allows: its conditions about alignment alone (the whole of it, or
/// those of a top-level `All`), checked for each alignment. `None` when it states no such condition.
pub fn alignment_rule(prerequisite: &Prerequisite) -> Option<BTreeSet<Alignment>> {
    let conditions: Vec<&Prerequisite> = match prerequisite {
        _ if only_alignment(prerequisite) => vec![prerequisite],
        Prerequisite::All(children) => children.iter().filter(|c| only_alignment(c)).collect(),
        _ => vec![],
    };
    if conditions.is_empty() {
        return None;
    }
    Some(
        EVERY
            .into_iter()
            .filter(|alignment| {
                let state = CharacterState {
                    alignment: Some(*alignment),
                    ..CharacterState::default()
                };
                conditions
                    .iter()
                    .all(|condition| state.check(condition).is_ok())
            })
            .collect(),
    )
}

/// "Any of these alignments", in the order given.
fn any_of(alignments: &[Alignment]) -> Prerequisite {
    Prerequisite::Any(
        alignments
            .iter()
            .copied()
            .map(Prerequisite::Alignment)
            .collect(),
    )
}

/// The Priest of Balance's and the Mantis Zealot's rules are their prerequisites from the start.
pub fn provide(guide: &crate::guide::Guide, picks: &mut Picks) {
    for (key, page, anchor, line, source, alignments) in RULES {
        says(guide, page, anchor, line);
        if source != Source::Data {
            picks.stated(
                DataAt::Archetype(ArchetypeKey::new(key)),
                any_of(alignments),
            );
        }
    }
}

/// Each archetype with a rule has exactly it, and no other archetype names an alignment.
pub fn check(_guide: &crate::guide::Guide, data: &GameData) {
    let keys: BTreeSet<&str> = RULES.iter().map(|(key, ..)| *key).collect();
    for (key, _, _, line, source, alignments) in RULES {
        let archetype = data
            .classes
            .iter()
            .flat_map(|class| &class.archetypes)
            .find(|archetype| archetype.key.as_str() == key)
            .unwrap_or_else(|| panic!("no archetype {key}"));
        let allowed: BTreeSet<Alignment> = alignments.iter().copied().collect();
        let rule = archetype.prerequisite.as_ref().and_then(alignment_rule);
        assert_eq!(
            rule,
            Some(allowed),
            "{key}: its alignment rule isn't {line:?}"
        );
        if source != Source::Data {
            let canonical = |mut prerequisite: Prerequisite| {
                prerequisite.canonicalize();
                prerequisite
            };
            assert_eq!(
                archetype.prerequisite.clone().map(canonical),
                Some(canonical(any_of(alignments))),
                "{key}: more than its alignment rule"
            );
        }
    }
    for class in &data.classes {
        for archetype in &class.archetypes {
            let named = archetype.prerequisite.as_ref().is_some_and(names_alignment);
            assert_eq!(
                named,
                keys.contains(archetype.key.as_str()),
                "{}: an alignment rule the guide doesn't give, or none where it does",
                archetype.key
            );
        }
    }
}
