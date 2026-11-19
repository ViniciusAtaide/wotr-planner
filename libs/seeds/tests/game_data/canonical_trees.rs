//! Every prerequisite in the checked data is in its normal form (ticket 332): no "and" in an "and", no "or" in an
//! "or", no "at least 1", no double negation, no group of one, every group sorted. `compile_prerequisites` puts every
//! entry's tree in it, compiled or hand-written, so every reader of the data reads one shape.

use rules::Prerequisite;

use crate::support::*;

#[test]
fn every_checked_prerequisite_is_canonical() {
    let trees = data_prerequisites(&DATA);
    assert!(trees.len() > 500, "{} trees", trees.len());
    let raw: Vec<&&Prerequisite> = trees.iter().filter(|tree| !tree.is_canonical()).collect();
    assert!(
        raw.is_empty(),
        "{} trees not canonical: {raw:#?}",
        raw.len()
    );
}

/// A negated group holds no negation of its own (design-P risk 5: a reading that pushes each `Not` onto its leaves
/// would read `Not(Any([Not(a), b]))` as needing `a`, where the engine's walkers skip it today).
#[test]
fn no_tree_nests_a_not_under_a_not() {
    fn nests(tree: &Prerequisite, under_not: bool) -> bool {
        match tree {
            Prerequisite::Not(child) => under_not || nests(child, true),
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                children.iter().any(|child| nests(child, under_not))
            }
            _ => false,
        }
    }
    let nesting: Vec<&Prerequisite> = data_prerequisites(&DATA)
        .into_iter()
        .filter(|tree| nests(tree, false))
        .collect();
    assert!(nesting.is_empty(), "{nesting:#?}");
}
