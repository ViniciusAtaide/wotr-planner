//! Ticket 47: ticket 45's decisions 1 and 3 and the user's rule for the rogue talents Weapon Focus and Combat Trick
//! (user, 2026-11-02), built where the rogue talents are.
//!
//! - **The Skill Focus rogue talent repeats per skill** (a user fact: "rogue talent follows the same rule as wild
//!   talent"). The rogue talent (`rogue#skill-focus`: "This is the feat of the same name"), which the slayer shares,
//!   is taken once for each value of its chosen feat, Skill Focus (`repeat: PerValue`): a second pick for the same
//!   skill is refused, one for another skill is open. The wild talent is built so (ticket 45's
//!   [`wild_talents`](crate::wild_talents)).
//! - **The rogue talent Weapon Focus stays taken once** (user: "for the rogue talent weapon focus can be only taken
//!   once"): asserted, not changed.
//! - **Combat Trick** (`rogue#combat-trick`: "Any combat feat that the character qualifies for"; user: "the combat
//!   trick -> weapon focus can be taken repeatable times as long as you select a different weapon"): taken any
//!   number of times (`Unlimited`), each time a bonus Combat feat slot whose feat's prerequisites are checked ("that
//!   the character qualifies for"), its row 1. Weapon Focus in another weapon through a second slot is the feat's
//!   own once per weapon. An option's rows need a selection with option levels: the selections offering it (the
//!   rogue talents, the Sylvan Trickster's Fey Tricks and the Underground Chemist's Discovery, each levelled by the
//!   rogue's levels; the slayer's talents, levelled by the slayer's already; the Feyform Shifter's Rogue Talent and
//!   the Vivisectionist's Medical Discovery, by their own class's, ticket 60's) give its slot whichever takes it.
//! - **The bonus-feat wild talent's slot reads "Wild talent feat"** (ticket 45's decision 3): built so by ticket 45's
//!   [`wild_talents`](crate::wild_talents).
//!
//! Each rule asserts the guide phrase it rests on when the providers are read ([`provide`]); [`check`] asserts what
//! only the finished data shows. The two selections [`crate::shared_options`] builds (Fey Tricks, the Underground
//! Chemist's Discovery) are levelled there.

use std::collections::BTreeSet;

use rules::{
    ClassKey, FeatKey, FeatSlot, FeatureSelection, GameData, ParameterKind, Repeat,
    SelectionOption, SelectionOptionKey,
};

use crate::build::section_text;
use crate::guide::Guide;
use crate::option_levels::{empty_row, page, section};
use crate::picks::Picks;

const ROGUE_SKILL_FOCUS: &str = "rogue/rogue-talents/skill-focus";
const ROGUE_WEAPON_FOCUS: &str = "rogue/rogue-talents/weapon-focus";
const COMBAT_TRICK: &str = "rogue/rogue-talents/combat-trick";

/// The selections offering Combat Trick, with the class whose levels make each one's option level.
const OFFERING_COMBAT_TRICK: [(&str, &str); 6] = [
    ("alchemist/vivisectionist/medical-discovery", "alchemist"),
    ("rogue/rogue-talents", "rogue"),
    ("rogue/sylvan-trickster/fey-tricks", "rogue"),
    ("rogue/underground-chemist/discovery", "rogue"),
    ("shifter/feyform-shifter/rogue-talent", "shifter"),
    ("slayer/slayer-talents", "slayer"),
];

/// Ticket 47's rules, built where the rogue talents are: the Skill Focus talent once per skill, Combat Trick any
/// number of times with its slot as row 1, and the selections offering it levelled by their class (the two that
/// [`crate::shared_options`] builds are levelled there).
pub(crate) fn provide(guide: &Guide, picks: &mut Picks) {
    says(
        guide,
        "rogue",
        "skill-focus",
        "This is the feat of the same name.",
    );
    picks.option_repeat(ROGUE_SKILL_FOCUS, Repeat::PerValue);
    says(
        guide,
        "rogue",
        "weapon-focus",
        "This is the feat of the same name.",
    );
    says(
        guide,
        "rogue",
        "combat-trick",
        "Any combat feat that the character qualifies for.",
    );
    picks.option_repeat(COMBAT_TRICK, Repeat::Unlimited);
    let mut row = empty_row(1);
    row.feat_slots = vec![FeatSlot::CombatFeat];
    picks.option_rows(COMBAT_TRICK, vec![row]);
    for (key, class) in OFFERING_COMBAT_TRICK {
        if !SHARED_HERE.contains(&key) && key != "slayer/slayer-talents" {
            picks.option_levels(key, &[class]);
        }
    }
}

/// The selections offering Combat Trick that [`crate::shared_options`] builds, and levels.
const SHARED_HERE: [&str; 2] = [
    "rogue/sylvan-trickster/fey-tricks",
    "rogue/underground-chemist/discovery",
];

/// What the rules hold on the finished data: Skill Focus once per skill, the talents' Skill Focus once per skill
/// and Weapon Focus once, Combat Trick's row, and each selection offering it levelled by its class (others may
/// widen it: ticket 60). Changes nothing.
pub(crate) fn check(data: &GameData) {
    let skill_focus = data
        .feats
        .iter()
        .find(|feat| feat.key.as_str() == "skill-focus")
        .expect("Skill Focus");
    assert_eq!(
        (skill_focus.parameter, skill_focus.repeat),
        (Some(ParameterKind::Skill), Repeat::PerValue),
        "skill-focus: once per skill"
    );
    let option = find_option(data, ROGUE_SKILL_FOCUS);
    assert_eq!(
        (option.chosen_feat.clone(), option.repeat),
        (Some(FeatKey::new("skill-focus")), Repeat::PerValue),
        "{ROGUE_SKILL_FOCUS}"
    );
    let option = find_option(data, ROGUE_WEAPON_FOCUS);
    assert_eq!(
        (option.chosen_feat.clone(), option.repeat),
        (Some(FeatKey::new("weapon-focus")), Repeat::Once),
        "{ROGUE_WEAPON_FOCUS}"
    );
    let trick = find_option(data, COMBAT_TRICK);
    assert_eq!(
        (
            trick.feats.is_empty(),
            trick.chosen_feat.is_none(),
            trick.parameter.is_none(),
            trick.repeat,
            trick.prerequisite.is_none(),
        ),
        (true, true, true, Repeat::Unlimited, true),
        "{COMBAT_TRICK}"
    );
    let key = SelectionOptionKey::new(COMBAT_TRICK);
    let offering: BTreeSet<String> = selections(data)
        .filter(|selection| selection.offers(&key))
        .map(|selection| selection.key.to_string())
        .collect();
    assert_eq!(
        offering,
        OFFERING_COMBAT_TRICK
            .iter()
            .map(|(selection, _)| selection.to_string())
            .collect(),
        "the selections offering {COMBAT_TRICK}"
    );
    for (key, class) in OFFERING_COMBAT_TRICK {
        let selection = selections(data)
            .find(|selection| selection.key.as_str() == key)
            .unwrap_or_else(|| panic!("no selection {key}"));
        assert!(
            selection.level_from.is_none()
                && selection.skipped_levels.is_empty()
                && selection.option_levels.contains(&ClassKey::new(class)),
            "{key}: levelled by {class}"
        );
    }
}

/// An option, in the selection that defines it (its key extends the selection's).
fn find_option<'d>(data: &'d GameData, key: &str) -> &'d SelectionOption {
    selections(data)
        .flat_map(|selection| &selection.options)
        .find(|option| option.key.as_str() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
}

/// Asserts that a talent's entry says `phrase`, in its prose or its table (a wild talent's description is a table
/// row).
fn says(guide: &Guide, page_slug: &str, anchor: &str, phrase: &str) {
    let text = section_text(section(page(guide, page_slug), anchor));
    assert!(
        text.contains(phrase),
        "{page_slug}#{anchor}: no {phrase:?} in {text:?}"
    );
}

/// Every selection of every class, archetype and path.
fn selections(data: &GameData) -> impl Iterator<Item = &FeatureSelection> {
    data.classes
        .iter()
        .chain(&data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter()
                .chain(class.archetypes.iter().flat_map(|a| &a.selections))
        })
        .chain(data.mythic_paths.iter().flat_map(|path| &path.selections))
}
