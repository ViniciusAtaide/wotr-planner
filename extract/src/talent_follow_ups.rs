//! Ticket 47: ticket 45's decisions 1 and 3 and the user's rule for the rogue talents Weapon Focus and Combat Trick
//! (user, 2026-11-02), applied to finished data.
//!
//! - **The Skill Focus talents repeat per skill** (a user fact: "Skill Focus can be taken once per wild talent as
//!   long as you select another skill"; "rogue talent follows the same rule as wild talent"). The wild talent
//!   (`kineticist#skill-focus`: "Identical to the feat") and the rogue talent (`rogue#skill-focus`: "This is the
//!   feat of the same name"), which the slayer shares, are taken once for each value of their chosen feat, Skill
//!   Focus (`repeat: PerValue`): a second pick for the same skill is refused, one for another skill is open.
//! - **The rogue talent Weapon Focus stays taken once** (user: "for the rogue talent weapon focus can be only taken
//!   once"): asserted, not changed.
//! - **Combat Trick** (`rogue#combat-trick`: "Any combat feat that the character qualifies for"; user: "the combat
//!   trick -> weapon focus can be taken repeatable times as long as you select a different weapon"): taken any
//!   number of times (`Unlimited`), each time a bonus Combat feat slot whose feat's prerequisites are checked ("that
//!   the character qualifies for"), its row 1. Weapon Focus in another weapon through a second slot is the feat's
//!   own once per weapon. An option's rows need a selection with option levels: the selections offering it (the
//!   rogue talents, the Sylvan Trickster's Fey Tricks and the Underground Chemist's Discovery, each levelled by the
//!   rogue's levels; the slayer's talents, levelled by the slayer's already) give its slot whichever takes it.
//! - **The bonus-feat wild talent's slot reads "Wild talent feat"** (ticket 45's decision 3), not "Elemental Focus
//!   feat": the slot's own label; the Elemental Focus selection keeps its name.
//!
//! Each rule asserts the guide phrase it rests on and what it finds in the data, so it fails loudly on data it
//! doesn't expect. [`apply`] runs on the finished data of an extraction and, through ticket 47's one-off rewrite,
//! on `game-data/`.

use std::collections::BTreeSet;

use rules::{
    ClassKey, FeatKey, FeatSlot, FeatureSelection, FeatureSelectionKey, GameData, ParameterKind,
    Repeat, SelectionOption, SelectionOptionKey,
};

use crate::build::section_text;
use crate::follow_ups::class_mut;
use crate::guide::Guide;
use crate::option_levels::{Log, empty_row, page, section};

const WILD_SKILL_FOCUS: &str = "kineticist/wild-talents/skill-focus";
const ROGUE_SKILL_FOCUS: &str = "rogue/rogue-talents/skill-focus";
const ROGUE_WEAPON_FOCUS: &str = "rogue/rogue-talents/weapon-focus";
const COMBAT_TRICK: &str = "rogue/rogue-talents/combat-trick";
const BONUS_FEAT_TALENT: &str = "kineticist/wild-talents/kineticist-bonus-feat";
const ELEMENT: &str = "kineticist/element";
/// The bonus-feat wild talent's slot, as the user names it.
const WILD_TALENT_FEAT: &str = "Wild talent feat";

/// The selections offering Combat Trick, with the class whose levels make each one's option level.
const OFFERING_COMBAT_TRICK: [(&str, &str); 4] = [
    ("rogue/rogue-talents", "rogue"),
    ("rogue/sylvan-trickster/fey-tricks", "rogue"),
    ("rogue/underground-chemist/discovery", "rogue"),
    ("slayer/slayer-talents", "slayer"),
];

/// Applies ticket 47's rules.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    skill_focus_per_skill(guide, data, &mut log);
    weapon_focus_once(guide, data);
    combat_trick(guide, data, &mut log);
    wild_talent_feat(data, &mut log);
    log
}

/// The Skill Focus talents, wild and rogue: once per skill of their chosen feat.
fn skill_focus_per_skill(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(guide, "kineticist", "skill-focus", "Identical to the feat");
    says(
        guide,
        "rogue",
        "skill-focus",
        "This is the feat of the same name.",
    );
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
    for key in [WILD_SKILL_FOCUS, ROGUE_SKILL_FOCUS] {
        let option = option_mut(data, key);
        assert_eq!(
            option.chosen_feat,
            Some(FeatKey::new("skill-focus")),
            "{key}"
        );
        assert_eq!(option.repeat, Repeat::Once, "{key}");
        option.repeat = Repeat::PerValue;
        log.push(format!("{key}: once per skill"));
    }
}

/// The rogue talent Weapon Focus: once, as it is.
fn weapon_focus_once(guide: &Guide, data: &mut GameData) {
    says(
        guide,
        "rogue",
        "weapon-focus",
        "This is the feat of the same name.",
    );
    let option = option_mut(data, ROGUE_WEAPON_FOCUS);
    assert_eq!(
        (option.chosen_feat.clone(), option.repeat),
        (Some(FeatKey::new("weapon-focus")), Repeat::Once),
        "{ROGUE_WEAPON_FOCUS}"
    );
}

/// Combat Trick: any number of times, a bonus Combat feat slot each time, through every selection offering it.
fn combat_trick(guide: &Guide, data: &mut GameData, log: &mut Log) {
    says(
        guide,
        "rogue",
        "combat-trick",
        "Any combat feat that the character qualifies for.",
    );
    let trick = option_mut(data, COMBAT_TRICK);
    assert_eq!(
        (
            trick.feats.is_empty(),
            trick.chosen_feat.is_none(),
            trick.parameter.is_none(),
            trick.repeat,
            trick.prerequisite.is_none(),
            trick.levels.is_empty(),
        ),
        (true, true, true, Repeat::Once, true, true),
        "{COMBAT_TRICK}: a bare option"
    );
    trick.repeat = Repeat::Unlimited;
    let mut row = empty_row(1);
    row.feat_slots = vec![FeatSlot::CombatFeat];
    trick.levels = vec![row];
    log.push(format!(
        "{COMBAT_TRICK}: any number of times, a Combat feat slot each time"
    ));

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
        let selection = selection_mut(data, key);
        assert!(
            selection.level_from.is_none() && selection.skipped_levels.is_empty(),
            "{key}: levelled by its classes"
        );
        let levels = BTreeSet::from([ClassKey::new(class)]);
        if selection.option_levels.is_empty() {
            selection.option_levels = levels;
            log.push(format!("{key}: levelled by {class}"));
        } else {
            assert_eq!(selection.option_levels, levels, "{key}");
        }
    }
}

/// The bonus-feat wild talent's one slot, labelled "Wild talent feat".
fn wild_talent_feat(data: &mut GameData, log: &mut Log) {
    let talent = option_mut(data, BONUS_FEAT_TALENT);
    let [row] = talent.levels.as_mut_slice() else {
        panic!("{BONUS_FEAT_TALENT}: one row")
    };
    let [
        FeatSlot::OptionList {
            selection,
            at_level: None,
            label,
        },
    ] = row.feat_slots.as_mut_slice()
    else {
        panic!("{BONUS_FEAT_TALENT}: one option list slot")
    };
    assert_eq!(
        (row.level, selection.as_str(), label.as_deref()),
        (1, ELEMENT, None),
        "{BONUS_FEAT_TALENT}'s slot"
    );
    *label = Some(WILD_TALENT_FEAT.to_owned());
    log.push(format!(
        "{BONUS_FEAT_TALENT}: its slot reads {WILD_TALENT_FEAT:?}"
    ));
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

fn selection_mut<'d>(data: &'d mut GameData, key: &str) -> &'d mut FeatureSelection {
    let wanted = FeatureSelectionKey::new(key);
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .flat_map(|class| {
            class
                .selections
                .iter_mut()
                .chain(class.archetypes.iter_mut().flat_map(|a| &mut a.selections))
        })
        .chain(
            data.mythic_paths
                .iter_mut()
                .flat_map(|path| &mut path.selections),
        )
        .find(|selection| selection.key == wanted)
        .unwrap_or_else(|| panic!("no selection {key}"))
}

/// An option, in the selection that defines it (its key extends the selection's).
fn option_mut<'d>(data: &'d mut GameData, key: &str) -> &'d mut SelectionOption {
    let class = key.split('/').next().expect("a key");
    class_mut(data, class)
        .selections
        .iter_mut()
        .flat_map(|selection| &mut selection.options)
        .find(|option| option.key.as_str() == key)
        .unwrap_or_else(|| panic!("no option {key}"))
}
