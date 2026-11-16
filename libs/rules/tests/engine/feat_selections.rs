//! Feat slots as selections (ticket 301): a row names a selection of feats as it names any selection, and the slot
//! it makes is a Feat slot with the ids, the order and the labels the feat slots had.

use rules::{
    BASIC_FEAT_SELECTION, DataAt, DataIssue, FeatSource, FeatureSelectionKey, Parameter, SlotId,
    SlotKind, WeaponKey,
};

use crate::support::*;

/// A row's selections of feats and its other selections are numbered apart, each in row order: the ranger's combat
/// style at 2 is the level's selection 0 and its style feat the level's class feat slot 0, as before.
#[test]
fn a_row_numbers_feat_and_option_selections_apart() {
    let engine = engine();
    let sheet = engine.evaluate(&build("human", &["ranger", "ranger"]));
    let slots: Vec<(SlotId, SlotKind)> = sheet.levels[1]
        .slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect();
    assert_eq!(
        slots,
        [
            (
                SlotId::Selection { level: 2, index: 0 },
                SlotKind::Selection(FeatureSelectionKey::new("ranger/combat-style"))
            ),
            (
                SlotId::Feat {
                    level: 2,
                    source: FeatSource::Class,
                    index: 0
                },
                SlotKind::Feat(feat_list("ranger/combat-style-feats"))
            ),
        ]
    );
}

/// A selection of one feat that takes a value is still a pick: no feat is granted until a weapon is chosen, unlike a
/// selection of one option, which is granted with no pick.
#[test]
fn a_one_feat_selection_still_takes_a_pick() {
    let engine = engine();
    let empty = build("human", &["warpriest"]);
    let slot = class_slot(1, 0);
    let sheet = walked(&engine, &empty);
    let found = sheet.levels[0]
        .slots
        .iter()
        .find(|each| each.id == slot)
        .expect("the warpriest's slot");
    assert_eq!(
        found.kind,
        SlotKind::Feat(feat_list("warpriest/weapon-focus"))
    );
    assert!(
        !sheet.levels[0]
            .state
            .feats
            .contains_key(&feat("weapon-focus"))
    );
    let offered: Vec<_> = engine
        .options(&empty, &slot)
        .unwrap()
        .into_iter()
        .map(|option| option.pick)
        .collect();
    assert!(offered.contains(&pick_feat_for(
        "weapon-focus",
        Parameter::Weapon(WeaponKey::new("longsword"))
    )));
    assert!(offered.iter().all(|pick| pick.parameter().is_some()));
}

/// A selection offers feats or options, never both: the data check refuses one that lists both.
#[test]
fn a_selection_of_feats_and_options_is_refused() {
    let mut data = world();
    selection_mut(&mut data, "monk/bonus-feats")
        .shared_options
        .insert(option("cleric/domain/fire"));
    let problems = data.check();
    assert!(
        problems.iter().any(|problem| {
            problem.at == DataAt::FeatureSelection(feat_list("monk/bonus-feats"))
                && problem.issue == DataIssue::FeatsAndOptions
        }),
        "{problems:?}"
    );
}

/// The general slot and Extra Feat's read "Feat"; the same selection on a race's or a row's slot reads "Bonus feat"; any
/// other selection of feats reads its name.
#[test]
fn the_basic_selection_reads_feat_or_bonus_feat() {
    let engine = engine();
    let sheet = engine.evaluate(&build("human", &["fighter"]));
    let basic = FeatureSelectionKey::new(BASIC_FEAT_SELECTION);
    let label = |id: SlotId| {
        let slot = sheet.levels[0]
            .slots
            .iter()
            .find(|slot| slot.id == id)
            .expect("the slot");
        assert_eq!(slot.kind, SlotKind::Feat(basic.clone()));
        engine.slot_label(slot)
    };
    assert_eq!(label(general_slot(1)), "Feat");
    assert_eq!(label(race_slot(0)), "Bonus feat");
    assert_eq!(engine.feat_slot_label(&basic), "Bonus feat");
    assert_eq!(
        engine.feat_slot_label(&feat_list("combat-feats")),
        "Combat feat"
    );
}
