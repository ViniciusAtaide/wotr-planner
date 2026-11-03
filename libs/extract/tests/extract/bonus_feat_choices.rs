//! Ticket 61: the ranger style feats `style_feats` reads from a style's section, which the Titan Fighter's Massive
//! Attacks and the Stormwalker's Archery pick from: the names by the level they're open from, each level holding the
//! earlier ones'.

use extract::bonus_feat_choices::style_feats;

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn a_styles_feats_grow_by_level_and_a_note_after_them_is_passed_over() {
    let text = "At 2nd level, the Ranger can select from Alpha Strike, Beta Guard and Gamma Step. At 6th level, \
                Delta Feint is added. At 10th level, Epsilon Cut and Zeta Hold are added. It is possible to take \
                Zeta Hold at 10th level without Delta Feint.";
    assert_eq!(
        style_feats(text),
        vec![
            (2, names(&["Alpha Strike", "Beta Guard", "Gamma Step"])),
            (
                6,
                names(&["Alpha Strike", "Beta Guard", "Gamma Step", "Delta Feint"])
            ),
            (
                10,
                names(&[
                    "Alpha Strike",
                    "Beta Guard",
                    "Gamma Step",
                    "Delta Feint",
                    "Epsilon Cut",
                    "Zeta Hold"
                ])
            ),
        ]
    );
    assert_eq!(
        style_feats("At 2nd level, the Ranger may choose from Alpha Strike and Beta Guard."),
        vec![(2, names(&["Alpha Strike", "Beta Guard"]))]
    );
}
