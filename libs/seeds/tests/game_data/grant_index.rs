//! The grant index on the real data (ticket 334, design-G §4): a few grants the game makes, and its agreement with the
//! walk on the bench Builds.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use rules::{
    BASIC_FEAT_SELECTION, Build, DataAt, DeityKey, Engine, Fact, FeatKey, FeatureKey,
    FeatureSelectionKey, Grant, GrantIndex, GrantValue, MythicPathKey, Pick, SelectionOptionKey,
    When,
};

use crate::support::*;

/// The grants an entry gives while it's held ([`When::Always`]).
fn always(by: DataAt) -> Vec<&'static Grant> {
    let index: &'static GrantIndex = &GRANTS;
    (index.gives(&by).iter())
        .filter(|site| site.when == When::Always)
        .flat_map(|site| &site.grants)
        .collect()
}

/// Irori's follower holds Improved Unarmed Strike for the unarmed strike, the deity's favoured weapon: the game's
/// `IroriFeature` 23a77a5985de08349820429ce1b5a234 grants `ImprovedUnarmedStrike` 7812ad3672a4b9a4fb894ea402095167
/// through `AddFeatureOnClassLevel` (Cleric 1), as 308h found.
#[test]
fn irori_gives_improved_unarmed_strike() {
    assert_eq!(
        always(DataAt::Deity(DeityKey::new("irori"))),
        [&Grant {
            fact: Fact::Feat(FeatKey::new(rules::IMPROVED_UNARMED_STRIKE)),
            value: GrantValue::Every,
            opens_slot: false,
        }]
    );
}

/// Achaekek's follower is proficient with the sawtooth saber, an exotic weapon, which the planner holds as Exotic
/// Weapon Proficiency for that weapon: the game's `AchaekekFeature` a3189d5b7c4d4d91beaa8bfffac3e38e grants
/// `SawtoothSabreProficiency` 5829f3b193704b8096820875f2eb2368 (`AddProficiencies`: `SawtoothSabre`) through
/// `AddFeatureOnClassLevel` (Cleric 1).
#[test]
fn achaekek_gives_exotic_weapon_proficiency_for_the_sawtooth_saber() {
    assert_eq!(
        always(DataAt::Deity(DeityKey::new("achaekek"))),
        [&Grant {
            fact: Fact::Feat(FeatKey::new(rules::EXOTIC_WEAPON_PROFICIENCY)),
            value: GrantValue::Fixed(weapon("sawtooth-saber")),
            opens_slot: false,
        }]
    );
}

/// Extra Rage Power opens a slot of rage powers: the game's `ExtraRagePowerSelection`
/// 0c7f01fbbe687bb4baff8195cb02fe6a is the feat and the selection at once (a `BlueprintFeatureSelection`), which the
/// planner holds as the feat granting `barbarian/extra-rage-power`, a selection sharing the barbarian's rage powers.
#[test]
fn extra_rage_power_opens_a_rage_power_slot() {
    let selection = FeatureSelectionKey::new("barbarian/extra-rage-power");
    assert_eq!(
        always(DataAt::Feat(FeatKey::new("extra-rage-power"))),
        [&Grant {
            fact: Fact::Selection(selection.clone()),
            value: GrantValue::Every,
            opens_slot: true,
        }]
    );
    let index: &GrantIndex = &GRANTS;
    assert!(
        index
            .offered_in(&Fact::Option(SelectionOptionKey::new(
                "barbarian/rage-power/animal-fury"
            )))
            .contains(&selection)
    );
}

/// A switch to Legend grants its features: the game's `LegendProgression` 905383229aaf79e4b8d7e2d316b68715 gives, at
/// its one level, `LegendFeature` 7f99961610390044391f300c7ef5e0c8 and `Legend_UniqueRestFeature`
/// 27327f8954cd4f91ab3deaebf1f8cfa7 (the planner's `legend/legend` and `legend/legends-corruption-resistance`).
#[test]
fn legends_switch_grants_its_features() {
    let given: BTreeSet<&Fact> = always(DataAt::MythicPath(MythicPathKey::new("legend")))
        .into_iter()
        .map(|grant| &grant.fact)
        .collect();
    assert_eq!(
        given,
        BTreeSet::from([
            &Fact::Feature(FeatureKey::new("legend/legend")),
            &Fact::Feature(FeatureKey::new("legend/legends-corruption-resistance")),
        ])
    );
}

/// The entries a Build holds at the end of its walk, which can give it facts: its race and subrace, background and
/// deity, classes and archetypes, paths, gear, and every feat, feature, option, selection, mythic ability and mythic
/// feat it holds.
fn holders(engine: &Engine, build: &Build) -> BTreeSet<DataAt> {
    let sheet = engine.evaluate(build);
    let state = sheet.levels.last().map(|level| &level.state);
    let mut held = BTreeSet::new();
    let start = &build.start;
    held.extend(start.race.clone().map(DataAt::Race));
    held.extend(start.subrace.clone().map(DataAt::Subrace));
    held.extend(start.background.clone().map(DataAt::Background));
    let paths = [
        &build.first_ascension,
        &build.mythic_path,
        &build.path_switch,
    ];
    held.extend(paths.into_iter().flatten().cloned().map(DataAt::MythicPath));
    held.extend(
        build
            .gear
            .worn
            .values()
            .chain(&build.gear.used)
            .cloned()
            .map(DataAt::Gear),
    );
    let Some(state) = state else {
        return held;
    };
    held.extend(state.deity().cloned().map(DataAt::Deity));
    for class in &state.classes {
        held.insert(DataAt::Class(class.class.clone()));
        held.extend(class.archetype.clone().map(DataAt::Archetype));
    }
    held.extend(state.feats().map(|(key, _)| DataAt::Feat(key.clone())));
    held.extend(
        state
            .features()
            .map(|(key, _)| DataAt::Feature(key.clone())),
    );
    held.extend(
        state
            .options()
            .map(|(key, _)| DataAt::SelectionOption(key.clone())),
    );
    held.extend(
        state
            .selections()
            .map(|(key, _)| DataAt::FeatureSelection(key.clone())),
    );
    held.extend(state.mythic_abilities().cloned().map(DataAt::MythicAbility));
    held.extend(state.mythic_feats().cloned().map(DataAt::MythicFeat));
    held
}

/// The facts a Build's walk holds at its end, of the kinds a grant gives: features, feats, options and selections.
fn held_facts(engine: &Engine, build: &Build) -> BTreeSet<Fact> {
    let sheet = engine.evaluate(build);
    let Some(level) = sheet.levels.last() else {
        return BTreeSet::new();
    };
    let state = &level.state;
    (state.feats().map(|(key, _)| Fact::Feat(key.clone())))
        .chain(state.features().map(|(key, _)| Fact::Feature(key.clone())))
        .chain(state.options().map(|(key, _)| Fact::Option(key.clone())))
        .chain(
            state
                .selections()
                .map(|(key, _)| Fact::Selection(key.clone())),
        )
        .collect()
}

/// A self-check of design-G §4: every feature, feat, option and selection a bench Build's walk holds has a grantor the
/// Build holds in the index, or was picked, or a selection the Build holds offers it. A miss is a grant kind the index
/// lacks. The one exception is the general feat slot's selection ([`BASIC_FEAT_SELECTION`]), which no entry of the
/// data grants: the walk holds it at odd levels by its own rule, as the game's `BasicFeatsProgression`
/// 5b72dd2ca2cb73b49903806ee8986325 grants it (`hold_feat_selections`, ticket 305).
#[test]
fn the_index_agrees_with_the_walk() {
    let engine = engine();
    let index: &GrantIndex = &GRANTS;
    let general = Fact::Selection(FeatureSelectionKey::new(BASIC_FEAT_SELECTION));
    let mut misses = Vec::new();
    let mut checked = 0;
    for (name, build) in bench() {
        let held = holders(&engine, &build);
        let picked: BTreeSet<Fact> = (build.picks.values())
            .map(|pick| match pick {
                Pick::Feat { feat, .. } => Fact::Feat(feat.clone()),
                Pick::SelectionOption { option, .. } => Fact::Option(option.clone()),
                Pick::MythicAbility { ability, .. } => Fact::MythicAbility(ability.clone()),
                Pick::MythicFeat { feat, .. } => Fact::MythicFeat(feat.clone()),
            })
            .collect();
        for fact in held_facts(&engine, &build) {
            checked += 1;
            let granted = (index.given_by(&fact)).any(|(site, _)| held.contains(&site.by));
            let offered = (index.offered_in(&fact).iter())
                .any(|selection| held.contains(&DataAt::FeatureSelection(selection.clone())));
            if !(granted || offered || picked.contains(&fact) || fact == general) {
                misses.push(format!("{name}: {fact:?}"));
            }
        }
    }
    assert!(checked > 0);
    assert!(misses.is_empty(), "held with no grantor: {misses:#?}");
}

/// Design-G §1's cost: the median of 20 builds of the index on the real data, beside the median of 20 `Engine::new`s
/// (which builds one), natively. Prints them; asserts nothing. Run in release:
///
/// ```text
/// cargo test --release -p seeds --test game_data grant_index_build_time -- --ignored --nocapture
/// ```
#[test]
#[ignore = "a timing, for ticket 334's Answer: run in release"]
fn grant_index_build_time() {
    fn median(mut times: Vec<Duration>) -> Duration {
        times.sort();
        times[times.len() / 2]
    }
    let data: &rules::GameData = &DATA;
    let index = (0..20)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(GrantIndex::new(data));
            start.elapsed()
        })
        .collect();
    let checked = seeds::check_game_data_files(&seeds::game_data_dir()).expect("game-data checks");
    let engines = (0..20)
        .map(|_| {
            let checked = checked.clone();
            let start = Instant::now();
            std::hint::black_box(Engine::new(checked));
            start.elapsed()
        })
        .collect();
    let (index, engine) = (median(index), median(engines));
    let sites = GRANTS.sites().len();
    let grants: usize = GRANTS.sites().iter().map(|site| site.grants.len()).sum();
    println!(
        "grant index: {index:?} ({sites} sites, {grants} grants); Engine::new: {engine:?}; share {:.1}%",
        100.0 * index.as_secs_f64() / engine.as_secs_f64()
    );
}
