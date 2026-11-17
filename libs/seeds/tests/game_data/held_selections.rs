//! A selection held as a fact of its own (ticket 247): the real classes' rows hold the selections they grant from
//! their level, whatever the slot holds, and an archetype that removes the row's selection holds none. The game adds
//! each selection a progression grants as a fact (`UnitLogic/Class/LevelUp/Actions/LevelUpHelper.cs`,
//! `AddFeaturesFromProgression`, WrathDecompiled 6ce4a6b), which a `PrerequisiteFeature` naming it reads.

use rules::{Build, FeatureSelectionKey, Prerequisite};

use crate::support::*;

/// Whether the Build holds `selection` at the end of `level`.
fn held(build: &Build, level: u8, selection: &str) -> bool {
    at(build, level)
        .check(&Prerequisite::Selection(FeatureSelectionKey::new(
            selection,
        )))
        .is_ok()
}

#[test]
fn a_skald_holds_its_rage_power_selection_from_3_with_no_rage_power_picked() {
    // Game: SkaldProgression 26418fed2bc153245972a5b54204ed75 grants SkaldRagePowerSelection
    // 2476514e31791394fa140f1a07941c96 at 3 (`barbarian/skald-rage-power`).
    let skald = build("skald", 3);
    assert!(
        !skald
            .picks
            .values()
            .any(|pick| pick.key().starts_with("barbarian/"))
    );
    assert!(!held(&skald, 2, "barbarian/skald-rage-power"));
    assert!(held(&skald, 3, "barbarian/skald-rage-power"));
    // Game: BattleScionArchetype 71d41dd6309ea994886908c34c87ccd9 removes SkaldRagePowerSelection at 3 and adds
    // BattleProwessSelection 29b480a26a88f9e47a10d8c9fab84ee6.
    let scion = archetype_build("skald", "skald/battle-scion", 3);
    assert!(!held(&scion, 3, "barbarian/skald-rage-power"));
    assert!(held(
        &scion,
        3,
        "skald/battle-scion/rage-power-or-teamwork-feat"
    ));
}

#[test]
fn an_exploiter_wizard_holds_the_exploit_selection_from_1() {
    // Game: ExploiterWizardArchetype 6d2027b4eec590c41adb5caeb7bb026d adds ExploiterExploitSelection
    // 2ba8a0040e0149e9ae9bfcb01a8ff01d at 1 (equivalent model: the arcanist's `arcanist/arcanist-exploits`).
    assert!(held(&exploiter(1), 1, "arcanist/exploiter-exploits"));
    assert!(!held(&build("wizard", 1), 1, "arcanist/arcanist-exploits"));
}

#[test]
fn a_rogue_talent_taken_in_a_slayer_talent_slot_holds_the_slayers_selection_only() {
    // Game: SlayerProgression 1f35d11937a06624b8a60db68c7869a1 grants SlayerTalentSelection2
    // 04430ad24988baa4daa0bcd4f1c7d118 at 2, not the rogue's talent selection, whatever its slot holds.
    let slayer = with_pick(
        build("slayer", 2),
        slot_of(&build("slayer", 2), 2, SLAYER_TALENTS),
        &rogue_talent("fast-stealth"),
    );
    assert!(possible(&slayer));
    assert!(held(&slayer, 2, SLAYER_TALENTS));
    assert!(!held(&slayer, 2, ROGUE_TALENTS));
}
