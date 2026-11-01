//! Granted feats: a class level row's `feats`, held from that level on with no slot and no pick, whatever their
//! prerequisites. A grant of a feat already held adds nothing; picking a granted feat later is "already held".

use rules::{
    ArchetypeKey, Build, ClassChoice, FeatSlot, Issue, Pick, Prerequisite, SlotId, SlotKind,
    SlotOption, Unmet, effective_row,
};

use crate::support::*;

fn options(build: &Build, slot: &SlotId) -> Vec<SlotOption> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("the Build has no slot {slot:?}"))
}

fn status_of(options: &[SlotOption], pick: &Pick) -> Result<(), Issue> {
    options
        .iter()
        .find(|option| &option.pick == pick)
        .unwrap_or_else(|| panic!("{pick:?} isn't offered"))
        .status
        .clone()
}

fn needs_shield_proficiency() -> Issue {
    Issue::Unmet(Unmet::Condition {
        needs: Prerequisite::Feat(feat("shield-proficiency")),
        has: None,
    })
}

fn armiger(levels: usize) -> Build {
    let mut build = build("dwarf", &times("fighter", levels));
    build.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    build
}

#[test]
fn a_wizards_scribe_scrolls_is_held_with_no_pick() {
    let build = build("dwarf", &["wizard"]);
    let sheet = engine().evaluate(&build);

    assert!(sheet.at(1).unwrap().feats.contains(&feat("scribe-scrolls")));
    // No slot asks for it: the wizard's first level has only the general slot.
    let kinds: Vec<SlotKind> = sheet.levels[0]
        .slots
        .iter()
        .map(|slot| slot.kind.clone())
        .collect();
    assert_eq!(kinds, [SlotKind::Feat(FeatSlot::General)]);
    assert_eq!(engine().validate(&build), []);
}

#[test]
fn a_granted_feat_ignores_its_prerequisites() {
    // Improved Unarmed Strike needs Dex 13 here; a monk with Dex 12 still holds it.
    let mut build = build("dwarf", &["monk"]);
    build.start.ability_scores.dexterity = 12;
    let sheet = engine().evaluate(&build);

    assert!(
        sheet
            .at(1)
            .unwrap()
            .feats
            .contains(&feat("improved-unarmed-strike"))
    );
    assert_eq!(sheet.problems, []);
}

#[test]
fn a_fighter_can_take_shield_focus() {
    // Shield Focus needs Shield Proficiency, which the fighter's first level grants before its slots are filled.
    let mut fighter = build("dwarf", &["fighter"]);
    let shield_focus = pick_feat("shield-focus");
    assert_eq!(
        status_of(&options(&fighter, &class_slot(1, 0)), &shield_focus),
        Ok(())
    );
    fighter.picks.insert(class_slot(1, 0), shield_focus.clone());
    assert_eq!(engine().validate(&fighter), []);

    // A rogue has no Shield Proficiency.
    assert_eq!(
        status_of(
            &options(&build("dwarf", &["rogue"]), &general_slot(1)),
            &shield_focus
        ),
        Err(needs_shield_proficiency())
    );
}

#[test]
fn picking_a_feat_the_class_already_granted_is_already_held() {
    let build = build("dwarf", &["fighter"]);
    assert_eq!(
        status_of(
            &options(&build, &general_slot(1)),
            &pick_feat("shield-proficiency")
        ),
        Err(Issue::AlreadyHeld)
    );
}

#[test]
fn a_grant_of_a_feat_already_held_adds_nothing_and_breaks_nothing() {
    // Shield Proficiency picked at 1 as a rogue, then granted again by the fighter's first level at 2.
    let mut build = build("dwarf", &["rogue", "fighter"]);
    build
        .picks
        .insert(general_slot(1), pick_feat("shield-proficiency"));
    let sheet = engine().evaluate(&build);

    assert_eq!(sheet.problems, []);
    let held = |level| sheet.at(level).unwrap().feats.clone();
    assert!(held(1).contains(&feat("shield-proficiency")));
    assert_eq!(
        held(2).difference(&held(1)).cloned().collect::<Vec<_>>(),
        [feat("tower-shield-proficiency")]
    );
}

#[test]
fn an_archetype_row_replaces_the_base_rows_granted_feats() {
    // The Armiger's first-level row grants no feats, so it has no Shield Proficiency.
    let data = world();
    let fighter = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "fighter")
        .unwrap();
    let row = effective_row(fighter, Some(&fighter.archetypes[0]), 1).unwrap();
    assert!(row.feats.is_empty());
    let base = effective_row(fighter, None, 1).unwrap();
    assert_eq!(base.feats.len(), 2);
    // A level the Armiger leaves out is the fighter's, grants included.
    let row_3 = effective_row(fighter, Some(&fighter.archetypes[0]), 3).unwrap();
    assert_eq!(row_3, effective_row(fighter, None, 3).unwrap());

    let build = armiger(1);
    assert!(
        !engine()
            .evaluate(&build)
            .at(1)
            .unwrap()
            .feats
            .contains(&feat("shield-proficiency"))
    );
    assert_eq!(
        status_of(
            &options(&build, &general_slot(1)),
            &pick_feat("shield-focus")
        ),
        Err(needs_shield_proficiency())
    );
}
