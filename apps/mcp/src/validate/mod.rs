//! `validate`: a whole Build, declared as a map of address → value, checked at once (ticket 133). The declaration is
//! read straight into the Build's fields (never through the pickers' offers or [`Engine::apply`](rules::Engine::apply),
//! whose job is to keep a Build possible: here the point is to find what they'd refuse), then the engine walks it once
//! ([`Engine::failures`](rules::Engine::failures)). Every failure comes back, linked into a tree of causes.
//!
//! - `declare.rs`: reading the declaration into a Build. What the walk can't see is refused there, each refusal a root
//!   (and the rest still walked): an address that isn't one, a value of the wrong shape or one naming nothing, one
//!   thing declared twice with different values, a level missing below a declared one, the rules no game data
//!   changes (a point-buy score or the budget, an ability increase off every 4th level, a skill's ranks above the
//!   character level), and a racial ability for a race that doesn't choose one.
//! - `wording.rs`: the walk's failures worded from the declared Build and its sheet.
//! - `tree.rs`: a failure, what it would have given, and the tree: each failure under the earliest earlier one its
//!   unmet prerequisites name or that it can't be reached without. It doesn't walk again to prove a link.
//!
//! This is diagnostics only (ticket 39 stands): a Build with any failure never becomes a code, and a valid one passes
//! the same final gate as `choose`'s, [`reopen`].

mod declare;
mod tree;
mod wording;

use std::collections::BTreeSet;

use rules::Issue;

use crate::code::{envelope, reopen};
use crate::error::{NEEDS_PARAMETER, NO_SUCH_CHOICE, NOT_AN_OPTION, ToolError};
use crate::options::issue_code;
use crate::planner::Planner;
use crate::shapes::{ValidateArgs, ValidateResult};
use crate::similar::{closeness, closest_note, nearest};

use declare::Declared;
use tree::Node;
use wording::Wording;

/// `validate`: the declared Build's envelope when every choice holds, otherwise every failure as a tree.
pub(crate) fn validate(planner: &Planner, args: ValidateArgs) -> Result<ValidateResult, ToolError> {
    let engine = &planner.engine;
    let Declared { build, refusals } = Declared::read(planner, &args.choices);
    let failures = engine.failures(&build);
    if refusals.is_empty() && failures.is_empty() {
        // The same final gate as `choose`'s: what comes back is exactly what the code holds.
        let build = reopen(engine, &build).ok_or_else(|| {
            ToolError::internal(
                "The declared Build passed the planner's walk but its code doesn't open. This is a bug in the \
                 planner.",
            )
        })?;
        let sheet = engine.evaluate(&build);
        return Ok(ValidateResult::Valid {
            envelope: envelope(engine, &build, &sheet),
        });
    }
    let sheet = engine.evaluate(&build);
    let wording = Wording {
        planner,
        build: &build,
        sheet: &sheet,
    };
    let nodes: Vec<Node> = refusals
        .into_iter()
        .chain(failures.iter().map(|failure| wording.node(failure)))
        .collect();
    let count = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
    Ok(ValidateResult::Invalid {
        count,
        failures: tree::tree(planner, nodes),
    })
}

/// Why the declaration refuses a choice before the walk sees it. Each has its one code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Refused {
    /// Text that isn't an address, or an address `validate` doesn't take (`level.N`, `class.<class>.advances`).
    NoSuchChoice,
    /// A value of the wrong shape, or one that names nothing of its kind (an alignment, an ability, a value).
    NotAnOption,
    /// A parameterised item given by its key alone.
    NeedsParameter,
    /// One thing declared twice with different values.
    DeclaredTwice,
    /// A rule no game data changes: a point-buy score or the budget, an ability increase off every 4th level, a
    /// skill's ranks above the character level.
    FixedRule,
    /// A level with no class below a declared one.
    MissingLevel,
    /// A level's choice past the levels the Build has.
    NotReached,
}

impl Refused {
    /// Its code, as a failure's `issue` gives it.
    pub(super) fn code(self) -> &'static str {
        match self {
            Refused::NoSuchChoice => NO_SUCH_CHOICE,
            Refused::NotAnOption => NOT_AN_OPTION,
            Refused::NeedsParameter => NEEDS_PARAMETER,
            Refused::DeclaredTwice => "declared_twice",
            Refused::FixedRule => "fixed_rule",
            Refused::MissingLevel => "missing_level",
            Refused::NotReached => issue_code(&Issue::NotReached),
        }
    }
}

/// ". Closest: Human (`human`), …" for the (name, key) pairs close to the given text, each key once, or "" when
/// none is: a suggestion after a miss, never what decides the issue.
pub(super) fn closest<'a>(
    given: &str,
    candidates: impl Iterator<Item = (&'a str, &'a str)>,
) -> String {
    let mut seen = BTreeSet::new();
    let near = nearest(
        candidates.filter(|(_, key)| seen.insert(*key)),
        |(name, key)| closeness(given, key).max(closeness(given, name)),
        |_, _| std::cmp::Ordering::Equal,
    );
    let names: Vec<String> = near
        .iter()
        .map(|(name, key)| format!("{name} (`{key}`)"))
        .collect();
    closest_note(&names).unwrap_or_default()
}
