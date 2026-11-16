//! The component vocabulary (ticket 295): hand-made components on the test world, each kind's expected values from the
//! game's code that gives it its meaning, cited by file and method at WrathDecompiled 6ce4a6b (`Kingmaker/…` under
//! `Assembly-CSharp`).

use std::collections::BTreeSet;

use rules::agreement::judge_all;
use rules::{
    Ability, BonusTarget, Build, ClassChoice, Component, Condition, DataAt, DataIssue, DeityKey,
    Descriptor, Engine, Fact, Feat, FeatCategory, FeatKey, Feature, FeatureKey,
    FeatureSelectionKey, GameData, Group, IfHeld, Issue, Pick, Pool, Prerequisite, RankBase,
    RankConfig, RankProgression, Repeat, Save, SelectionOptionKey, Share, SharedValue, SlotId,
    SpellType, StatRef, StatRule, StatTarget, Value, WeaponProficiency, compile_requires,
    game_stacked, game_stacking,
};

use crate::agreement::{SAMPLE, corpus};
use crate::support::*;

// ---- helpers --------------------------------------------------------------------------------------------------

/// A feat with these components and nothing else: no prerequisite, taken once.
fn component_feat(key: &str, components: Vec<Component>) -> Feat {
    Feat {
        key: feat(key),
        blueprint: None,
        name: key.into(),
        category: FeatCategory::Miscellaneous,
        summary: String::new(),
        guide: None,
        parameter: None,
        repeat: Repeat::Once,
        extra_pick_of: BTreeSet::new(),
        requires_proficiency: false,
        prerequisite: None,
        components,
    }
}

/// A feature with these components and nothing else.
fn component_feature(key: &str, components: Vec<Component>) -> Feature {
    Feature {
        key: feature(key),
        blueprint: None,
        name: key.into(),
        summary: String::new(),
        guide: None,
        option_row: None,
        daily_choice: false,
        favoured_weapon_proficiency: false,
        weapons: BTreeSet::new(),
        components,
    }
}

/// The test world with feats and features added.
fn engine_with(feats: Vec<Feat>, features: Vec<Feature>) -> Engine {
    let mut data = world();
    data.feats.extend(feats);
    data.features.extend(features);
    engine_on(data)
}

/// A human of these classes (Str 16, Dex 14, Con 12, Int 13, Wis 10, Cha 8) with `feat` in the human's bonus feat slot
/// at level 1: the feat that holds the components under test.
fn holding(feat: &str, classes: &[&str]) -> Build {
    let mut build = build("human", classes);
    build.picks.insert(race_slot(0), pick_feat(feat));
    build
}

fn requires(condition: Condition) -> Component {
    Component::Requires {
        condition,
        group: Group::All,
    }
}

fn requires_any(condition: Condition) -> Component {
    Component::Requires {
        condition,
        group: Group::Any,
    }
}

fn has(fact: Fact) -> Condition {
    Condition::Has(fact)
}

fn fact_feature(key: &str) -> Fact {
    Fact::Feature(feature(key))
}

/// What a picker says of a pick in a slot of a Build.
fn status(engine: &Engine, build: &Build, slot: &SlotId, pick: &Pick) -> Result<(), Issue> {
    engine
        .options(build, slot)
        .expect("the slot is the Build's")
        .into_iter()
        .find(|option| &option.pick == pick)
        .expect("the slot offers the pick")
        .status
}

/// Whether the walked Build holds a feature at a level.
fn holds_feature(engine: &Engine, build: &Build, level: u8, key: &str) -> bool {
    walked(engine, build)
        .at(level)
        .expect("a level of the Build")
        .feature_count(&feature(key))
        > 0
}

/// A stat bonus with no type, multiplier or condition.
fn stat_bonus(stat: StatTarget, value: Value) -> Component {
    typed_bonus(stat, value, Descriptor::Untyped)
}

fn typed_bonus(stat: StatTarget, value: Value, descriptor: Descriptor) -> Component {
    Component::StatBonus {
        stat,
        value,
        descriptor,
        multiplier: 1,
        at_least: None,
        only_if: None,
    }
}

/// The amount of the bonuses in force of one holder, by its name, to one number at a level.
fn bonus_of(sheet: &rules::Sheet, level: u8, name: &str, target: &BonusTarget) -> i16 {
    sheet.levels[usize::from(level) - 1]
        .state
        .bonuses_in_force
        .iter()
        .filter(|bonus| bonus.target.number() == Some(target))
        .filter(|bonus| bonus.name == name)
        .map(|bonus| bonus.amount)
        .sum()
}

fn rank(base: RankBase, progression: RankProgression) -> Value {
    Value::Rank(RankConfig {
        base,
        progression,
        min: None,
        max: None,
    })
}

/// The component bonuses in force at a level, by name and amount, to one total.
fn will_total(engine: &Engine, build: &Build, level: u8) -> i16 {
    let sheet = walked(engine, build);
    let level = &sheet.levels[usize::from(level) - 1];
    engine.totals(level).will.total
}

// ---- Requires ---------------------------------------------------------------------------------------------------

/// `Blueprints/Classes/BlueprintFeature.cs` `MeetsPrerequisites`: every prerequisite of the `All` group holds and, when
/// the `Any` group has any, one of them does. The `Requires` compile into that tree, so the picker refuses with the
/// reasons it gives any prerequisite.
#[test]
fn requires_all_and_any_compile_into_the_prerequisite_tree_with_its_reasons() {
    let components = vec![
        requires(has(Fact::Feat(feat("power-attack")))),
        requires(Condition::Stat {
            stat: StatRef::Ability(Ability::Strength),
            value: 13,
            full: false,
        }),
        requires_any(Condition::ClassLevel(class("fighter"), 4)),
        requires_any(Condition::ClassLevel(class("rogue"), 2)),
    ];
    let class_level = |key: &str, level| Prerequisite::ClassLevel {
        class: class(key),
        level,
    };
    assert_eq!(
        compile_requires(&components, |_| None),
        Some(Prerequisite::All(vec![
            Prerequisite::Feat(feat("power-attack")),
            Prerequisite::AbilityScore {
                ability: Ability::Strength,
                score: 13,
            },
            Prerequisite::Any(vec![class_level("fighter", 4), class_level("rogue", 2)]),
        ]))
    );
    // One condition of either group is that condition (`flag ?? true` with one prerequisite).
    assert_eq!(
        compile_requires(
            &[requires_any(Condition::ClassLevel(class("fighter"), 4))],
            |_| None
        ),
        Some(class_level("fighter", 4))
    );
    assert_eq!(compile_requires(&[], |_| None), None);
    // `PrerequisiteFeaturesFromList.CheckInternal` returns on the first fact held with `num >= Amount`, and with none
    // held fails: an `Amount` of 0 needs one, as 1 does.
    let either = BTreeSet::from([Fact::Feat(feat("dodge")), Fact::Feat(feat("power-attack"))]);
    assert_eq!(
        compile_requires(&[requires(Condition::AtLeast(0, either))], |_| None),
        Some(Prerequisite::AtLeast(
            1,
            vec![
                Prerequisite::Feat(feat("dodge")),
                Prerequisite::Feat(feat("power-attack"))
            ]
        ))
    );

    let engine = engine_with(vec![component_feat("trained-cleaver", components)], vec![]);
    let pick = pick_feat("trained-cleaver");
    // A fighter 1 without Power Attack: two of the three conditions fail, with the tree's reasons.
    let Err(Issue::Unmet(unmet)) = status(
        &engine,
        &build("human", &["fighter"]),
        &general_slot(1),
        &pick,
    ) else {
        panic!("refused for its prerequisites");
    };
    assert_eq!(
        engine.unmet_text(&unmet),
        "Fighter level 4 (have 1), power-attack"
    );
    // A fighter 5 with Power Attack at 1 meets them.
    let mut build = build("human", &times("fighter", 5));
    build
        .picks
        .insert(general_slot(1), pick_feat("power-attack"));
    assert_eq!(status(&engine, &build, &general_slot(5), &pick), Ok(()));
}

/// Ticket 296: the checked data holds an entry's `Requires` as the `prerequisite` they compile into, canonical as the
/// files' trees are read, and no longer among its components, so every reader of a prerequisite (the engine, the MCP
/// server's text, the game check, the browser's JSON) reads one tree. The other components stay.
#[test]
fn checked_data_holds_the_requires_compiled_into_the_prerequisite() {
    let flag = Component::Flag(rules::MechanicsFlag::WeaponFinesse);
    let mut data = world();
    data.feats.push(component_feat(
        "trained-cleaver",
        vec![
            requires_any(Condition::ClassLevel(class("rogue"), 2)),
            requires(has(Fact::Feat(feat("power-attack")))),
            flag.clone(),
            requires_any(Condition::ClassLevel(class("fighter"), 4)),
        ],
    ));
    let checked = data.checked().expect("the data checks");
    let cleaver = (checked.feats.iter())
        .find(|entry| entry.key == feat("trained-cleaver"))
        .unwrap();

    let class_level = |key: &str, level| Prerequisite::ClassLevel {
        class: class(key),
        level,
    };
    let mut expected = Prerequisite::All(vec![
        Prerequisite::Feat(feat("power-attack")),
        Prerequisite::Any(vec![class_level("fighter", 4), class_level("rogue", 2)]),
    ]);
    expected.canonicalize();
    assert_eq!(cleaver.prerequisite, Some(expected));
    assert_eq!(cleaver.components, [flag]);
    // Checked again, as the browser checks the JSON the server sends, it holds the field alone.
    assert_eq!(GameData::clone(&checked).check(), []);
}

/// `Blueprints/Classes/Prerequisites/PrerequisiteClassLevel.cs` `GetClassLevel`: the class level plus, for each held
/// `ClassLevelsForPrerequisites` whose fake class it is and whose `ForSelection` lists the pick,
/// `(int)(Modifier × ActualClass level + Summand)` (Eldritch Scion's Fighter Training: magus levels at 0.5 as fighter
/// levels for `BasicFeatSelection`'s feats).
#[test]
fn a_class_level_prerequisite_counts_the_classes_a_held_fact_adds() {
    let training = component_feat(
        "wizard-training",
        vec![Component::PrerequisiteClassLevels {
            class: class("fighter"),
            from: class("wizard"),
            share: Share::Half,
            plus: 0,
            only_for: Some(Pool::Feats),
        }],
    );
    let needs = component_feat(
        "needs-fighter-3",
        vec![requires(Condition::ClassLevel(class("fighter"), 3))],
    );
    let engine = engine_with(vec![training, needs], vec![]);
    let pick = pick_feat("needs-fighter-3");
    // Fighter 1 + wizard 4: 1 + (int)(0.5 × 4) = 3.
    let build = holding(
        "wizard-training",
        &["fighter", "wizard", "wizard", "wizard", "wizard"],
    );
    assert_eq!(status(&engine, &build, &general_slot(5), &pick), Ok(()));
    // Fighter 1 + wizard 3 + rogue 1: 1 + (int)(0.5 × 3) = 2, the half truncated.
    let build = holding(
        "wizard-training",
        &["fighter", "wizard", "wizard", "wizard", "rogue"],
    );
    let Err(Issue::Unmet(unmet)) = status(&engine, &build, &general_slot(5), &pick) else {
        panic!("refused");
    };
    assert_eq!(conditions(&unmet)[0].1, Some(2));
    // Without the training, the wizard levels count for nothing.
    let mut build = rules::test_world::build(
        "human",
        &["fighter", "wizard", "wizard", "wizard", "wizard"],
    );
    build.picks.insert(race_slot(0), pick_feat("power-attack"));
    let Err(Issue::Unmet(unmet)) = status(&engine, &build, &general_slot(5), &pick) else {
        panic!("refused");
    };
    assert_eq!(conditions(&unmet)[0].1, Some(1));
}

// ---- Grants -----------------------------------------------------------------------------------------------------

/// `UnitLogic/FactLogic/AddFacts.cs` `OnActivate` (`UpdateFacts`): the facts are held while their source is, and only
/// held: a selection is offered as a pick only by a progression's row (`LevelUpHelper.AddFeaturesFromProgression`,
/// `LevelUpState.AddSelection`) or as a selection's own option (`SelectFeature.Apply`), never by `AddFacts`.
#[test]
fn a_grant_holds_its_facts_and_a_granted_selection_opens_no_slot() {
    let talents = FeatureSelectionKey::new("rogue/rogue-talents");
    let granting = component_feat(
        "granting",
        vec![Component::Grant(BTreeSet::from([
            fact_feature("granted"),
            Fact::Feat(feat("dodge")),
            Fact::Selection(talents.clone()),
        ]))],
    );
    let engine = engine_with(vec![granting], vec![component_feature("granted", vec![])]);
    let build = holding("granting", &["fighter"]);
    let sheet = walked(&engine, &build);
    let state = sheet.at(1).unwrap();
    assert_eq!(state.feature_count(&feature("granted")), 1);
    assert_eq!(state.feat_count(&feat("dodge")), 1);
    assert_eq!(state.selections.get(&talents), Some(&1));
    // The selection is held and offers no pick: the fighter 1's slots are those of a fighter 1 without the grant.
    let plain = walked(&engine, &holding("dodge", &["fighter"]));
    let ids = |sheet: &rules::Sheet| -> Vec<SlotId> {
        sheet.levels[0]
            .slots
            .iter()
            .map(|slot| slot.id.clone())
            .collect()
    };
    assert_eq!(ids(&sheet), ids(&plain));
    assert!(
        sheet
            .slot(&SlotId::OptionSelection { level: 1, index: 0 })
            .is_none()
    );
    assert_eq!(
        state.option_count(&SelectionOptionKey::new("rogue/rogue-talents/fast-stealth")),
        0
    );
}

/// `Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs` `IsFeatureShouldBeApplied`: the level is
/// `ReplaceCasterLevelOfAbility.CalculateClassLevel`, the class's levels plus each additional class's, a class counting
/// only taken with one of the archetypes when they are all its own (`CheckArchetype`); the feature is held from `Level`
/// on, applied on activation and on each level gained (`Apply`, `HandleUnitGainLevel`).
#[test]
fn a_grant_at_class_level_comes_at_the_level_and_counts_its_additional_classes_and_archetypes() {
    let at_level =
        |fact: &str, classes: &[&str], archetypes: &[&str], level| Component::GrantAtClassLevel {
            fact: fact_feature(fact),
            classes: classes.iter().map(|key| class(key)).collect(),
            archetypes: archetypes
                .iter()
                .map(|key| rules::ArchetypeKey::new(*key))
                .collect(),
            level,
            before: false,
        };
    let granting = component_feat(
        "late-bloomer",
        vec![
            at_level("both-classes", &["fighter", "rogue"], &[], 3),
            at_level("armiger-only", &["fighter"], &["fighter/armiger"], 1),
        ],
    );
    let features = vec![
        component_feature("both-classes", vec![]),
        component_feature("armiger-only", vec![]),
    ];
    let engine = engine_with(vec![granting], features);
    let build = holding("late-bloomer", &["fighter", "rogue", "fighter"]);
    // Fighter 1 (1), fighter 1 + rogue 1 (2), fighter 2 + rogue 1 (3).
    assert!(!holds_feature(&engine, &build, 1, "both-classes"));
    assert!(!holds_feature(&engine, &build, 2, "both-classes"));
    assert!(holds_feature(&engine, &build, 3, "both-classes"));
    // A plain fighter's levels don't count where the archetypes are all the fighter's; an armiger's do.
    assert!(!holds_feature(&engine, &build, 3, "armiger-only"));
    let mut armiger = build.clone();
    armiger.class_choices.insert(
        class("fighter"),
        ClassChoice {
            archetype: Some(rules::ArchetypeKey::new("fighter/armiger")),
            ..ClassChoice::default()
        },
    );
    assert!(holds_feature(&engine, &armiger, 1, "armiger-only"));
}

/// `AddFeatureOnClassLevel.IsFeatureShouldBeApplied` with `BeforeThisLevel`: held while the level is below `Level`, and
/// removed (`Apply`: `RemoveFact`) once it's reached.
#[test]
fn before_this_level_takes_it_away_at_the_level() {
    let granting = component_feat(
        "fading",
        vec![Component::GrantAtClassLevel {
            fact: fact_feature("early"),
            classes: BTreeSet::from([class("fighter")]),
            archetypes: BTreeSet::new(),
            level: 3,
            before: true,
        }],
    );
    let engine = engine_with(vec![granting], vec![component_feature("early", vec![])]);
    let build = holding("fading", &times("fighter", 4));
    assert!(holds_feature(&engine, &build, 1, "early"));
    assert!(holds_feature(&engine, &build, 2, "early"));
    assert!(!holds_feature(&engine, &build, 3, "early"));
    assert!(!holds_feature(&engine, &build, 4, "early"));
}

/// `Designers/Mechanics/Facts/AddFeatureIfHasFact.cs` `Apply`: on activation and on each level gained, the feature is
/// added if the checked fact is held and nothing was added yet; nothing removes it but the source's `OnDeactivate`.
#[test]
fn a_grant_if_held_adds_once_and_never_removes() {
    let granting = component_feat(
        "watchful",
        vec![
            Component::GrantAtClassLevel {
                fact: fact_feature("early"),
                classes: BTreeSet::from([class("fighter")]),
                archetypes: BTreeSet::new(),
                level: 3,
                before: true,
            },
            Component::GrantIf {
                has: fact_feature("early"),
                not: false,
                fact: fact_feature("kept"),
            },
            Component::GrantIf {
                has: Fact::Feat(feat("power-attack")),
                not: false,
                fact: fact_feature("later"),
            },
        ],
    );
    let features = ["early", "kept", "later"]
        .map(|key| component_feature(key, vec![]))
        .to_vec();
    let engine = engine_with(vec![granting], features);
    let mut build = holding("watchful", &times("fighter", 4));
    build
        .picks
        .insert(general_slot(3), pick_feat("power-attack"));
    // Held with its source at 1, and still at 3 when the fact it checked is gone.
    assert!(holds_feature(&engine, &build, 1, "kept"));
    assert!(!holds_feature(&engine, &build, 3, "early"));
    assert!(holds_feature(&engine, &build, 3, "kept"));
    // Power Attack taken at 3 is seen at the next level gained, 4 (`HandleUnitGainLevel` comes with the class level,
    // `UnitProgressionData.AddClassLevel`, before the level's picks).
    assert!(!holds_feature(&engine, &build, 3, "later"));
    assert!(holds_feature(&engine, &build, 4, "later"));
}

// ---- Stat bonuses -----------------------------------------------------------------------------------------------

/// `EntitySystem/Stats/ModifiableValue.cs` `ApplyModifiersFiltered`: of a descriptor that stacks
/// (`Enums/ModifierDescriptorHelper.cs` `DefaultStackingDescriptors`: `None`, `UntypedStackable`, `Racial`, …) every
/// modifier adds; of any other the highest bonus and the lowest penalty.
#[test]
fn a_stat_bonus_of_one_type_counts_the_largest_and_untyped_ones_stack() {
    let amounts = [
        (Descriptor::Insight, 2),
        (Descriptor::Insight, 3),
        (Descriptor::Untyped, 1),
        (Descriptor::Untyped, 1),
        (Descriptor::Racial, 2),
        (Descriptor::Racial, 1),
        (Descriptor::Morale, -1),
        (Descriptor::Morale, -2),
    ];
    assert_eq!(
        game_stacking(&amounts),
        [false, true, true, true, true, true, false, true]
    );
    assert_eq!(game_stacked(&amounts), 3 + 1 + 1 + 2 + 1 - 2);

    // In the walk, to the Will save: insight 2 and 3 count 3, two untyped 1s count 2.
    let will = |amount, descriptor| {
        typed_bonus(
            StatTarget::Save(Save::Will),
            Value::Flat(amount),
            descriptor,
        )
    };
    let steady = component_feat(
        "steady",
        vec![
            will(2, Descriptor::Insight),
            will(3, Descriptor::Insight),
            will(1, Descriptor::Untyped),
            will(1, Descriptor::UntypedStackable),
        ],
    );
    let engine = engine_with(vec![steady], vec![]);
    let without = will_total(&engine, &build("human", &["fighter"]), 1);
    assert_eq!(
        will_total(&engine, &holding("steady", &["fighter"]), 1),
        without + 5
    );
}

/// `ModifiableValue.Modifier.IsPermanent`: a racial modifier, or an enhancement from an item; an ability's permanent
/// value counts only those (`ModifiableValueAttributeStat.CalculatePermanentValue`, `FilterIsPermanent`), and an ability
/// prerequisite reads it (`PrerequisiteStatValue.GetStatValue`). Every number reads the sheet's value. A full one
/// (`Designers/Mechanics/Prerequisites/PrerequisiteFullStatValue.cs` `GetFullStatValue`) reads the permanent value plus
/// the modifiers of descriptor `Feat`, and no other.
#[test]
fn an_untyped_ability_bonus_counts_on_the_sheet_and_not_for_prerequisites() {
    let strength = |descriptor| {
        typed_bonus(
            StatTarget::Ability(Ability::Strength),
            Value::Flat(4),
            descriptor,
        )
    };
    let needs = component_feat(
        "needs-strength-20",
        vec![requires(Condition::Stat {
            stat: StatRef::Ability(Ability::Strength),
            value: 20,
            full: false,
        })],
    );
    let full = component_feat(
        "needs-full-strength-20",
        vec![requires(Condition::Stat {
            stat: StatRef::Ability(Ability::Strength),
            value: 20,
            full: true,
        })],
    );
    let engine = engine_with(
        vec![
            component_feat("legendary", vec![strength(Descriptor::Untyped)]),
            component_feat("ancestral", vec![strength(Descriptor::Racial)]),
            component_feat("trained", vec![strength(Descriptor::Feat)]),
            needs,
            full,
        ],
        vec![],
    );
    let meets = |build: &Build, feat: &str| {
        status(&engine, build, &general_slot(1), &pick_feat(feat)).is_ok()
    };
    // Strength 16 + 4 untyped: 20 on the sheet, 16 for either prerequisite.
    let legendary = holding("legendary", &["fighter"]);
    let state = walked(&engine, &legendary).at(1).unwrap().clone();
    assert_eq!(state.ability_scores.get(Ability::Strength), 20);
    assert_eq!(state.permanent_ability_scores.get(Ability::Strength), 16);
    assert!(!meets(&legendary, "needs-strength-20"));
    assert!(!meets(&legendary, "needs-full-strength-20"));
    // A racial one counts for both.
    let ancestral = holding("ancestral", &["fighter"]);
    let state = walked(&engine, &ancestral).at(1).unwrap().clone();
    assert_eq!(state.permanent_ability_scores.get(Ability::Strength), 20);
    assert!(meets(&ancestral, "needs-strength-20"));
    assert!(meets(&ancestral, "needs-full-strength-20"));
    // A `Feat` one counts for the full value only.
    let trained = holding("trained", &["fighter"]);
    assert!(!meets(&trained, "needs-strength-20"));
    assert!(meets(&trained, "needs-full-strength-20"));
}

/// `Designers/Mechanics/Facts/ReplaceStatForPrerequisites.cs` `ResultStat` with `NewStat`: for a prerequisite the higher
/// of the stat and the other stat's value as a number reads it (`GetStat(NewStat)`, its `ModifiedValue` by
/// `ModifiableValue`'s `implicit operator int`), and for a full one (`fullValue`, from
/// `PrerequisiteFullStatValue.GetFullStatValue`) the other stat's `PermanentValue`.
#[test]
fn a_stat_replacement_reads_the_other_stat_on_the_sheet_and_for_a_full_prerequisite_its_permanent_value()
 {
    let dexterity_for_strength = |key: &str, descriptor| {
        component_feat(
            key,
            vec![
                Component::PrerequisiteStat {
                    stat: StatRef::Ability(Ability::Strength),
                    rule: StatRule::Ability(Ability::Dexterity),
                },
                typed_bonus(
                    StatTarget::Ability(Ability::Dexterity),
                    Value::Flat(8),
                    descriptor,
                ),
            ],
        )
    };
    let needs = |key: &str, full| {
        component_feat(
            key,
            vec![requires(Condition::Stat {
                stat: StatRef::Ability(Ability::Strength),
                value: 20,
                full,
            })],
        )
    };
    let engine = engine_with(
        vec![
            dexterity_for_strength("nimble", Descriptor::Untyped),
            dexterity_for_strength("born-nimble", Descriptor::Racial),
            needs("needs-strength-20", false),
            needs("needs-full-strength-20", true),
        ],
        vec![],
    );
    let meets = |build: &Build, feat: &str| {
        status(&engine, build, &general_slot(1), &pick_feat(feat)).is_ok()
    };
    // Strength 16, Dexterity 14 + 8 untyped: 22 on the sheet, 14 permanent.
    let nimble = holding("nimble", &["fighter"]);
    assert!(meets(&nimble, "needs-strength-20"));
    assert!(!meets(&nimble, "needs-full-strength-20"));
    // Dexterity 14 + 8 racial: 22 permanent too.
    let born = holding("born-nimble", &["fighter"]);
    assert!(meets(&born, "needs-strength-20"));
    assert!(meets(&born, "needs-full-strength-20"));
}

/// `UnitLogic/Mechanics/Components/ContextRankConfig.cs` `GetValue`: `ApplyMinMax(ApplyProgression(GetBaseValue))`.
/// Each progression of `ApplyProgression`, by its formula (C#'s integer division truncates toward zero), then each
/// base the planner knows of `GetBaseValue`, read off a walked Build, and `ApplyMinMax`.
#[test]
fn a_rank_config_reads_each_base_and_applies_each_progression() {
    use RankProgression as P;
    let cases: [(P, i32, i32); 17] = [
        (P::AsIs, 7, 7),
        (P::Div2, 7, 3),
        (P::OnePlusDiv2, 7, 4),
        (P::StartPlusDivStep { start: 3, step: 4 }, 11, 3),
        (P::StartPlusDivStep { start: 3, step: 4 }, 1, 1),
        (P::DivStep(3), 8, 2),
        (P::OnePlusDivStep(5), 12, 3),
        (P::MultiplyBy(3), 4, 12),
        (P::HalfMore, 7, 10),
        (P::BonusValue(2), 7, 9),
        (P::DelayedStartPlusDivStep { start: 5, step: 5 }, 4, 0),
        (P::DelayedStartPlusDivStep { start: 5, step: 5 }, 14, 2),
        (P::StartPlusDoubleDivStep { start: 1, step: 4 }, 9, 5),
        (P::Div2PlusStep(1), 7, 4),
        (P::DoublePlusBonusValue(1), 3, 7),
        (P::SquaredValue, 3, 9),
        (P::Custom(vec![(4, 1), (9, 2)]), 5, 2),
    ];
    for (progression, value, expected) in cases {
        assert_eq!(
            progression.apply(value),
            expected,
            "{progression:?} of {value}"
        );
    }

    // The bases, each to a skill or a save of a fighter 2 / rogue 3 with 2 Mobility ranks and Dodge.
    let bonus = |stat, base, progression| stat_bonus(stat, rank(base, progression));
    let skill = |key: &str| StatTarget::Skill(rules::SkillKey::new(key));
    let reader = component_feat(
        "reader",
        vec![
            bonus(
                skill("athletics"),
                RankBase::ClassLevel {
                    classes: BTreeSet::from([class("fighter")]),
                    except: true,
                },
                P::AsIs,
            ),
            bonus(skill("mobility"), RankBase::CharacterLevel, P::Div2),
            bonus(
                skill("persuasion"),
                RankBase::FeatureListRanks(BTreeSet::from([
                    Fact::Feat(feat("dodge")),
                    Fact::Feat(feat("reader")),
                ])),
                P::AsIs,
            ),
            bonus(
                skill("knowledge-arcana"),
                RankBase::BaseStat(StatRef::Skill(rules::SkillKey::new("mobility"))),
                P::MultiplyBy(2),
            ),
            bonus(
                StatTarget::Save(Save::Fortitude),
                RankBase::StatBonus(Ability::Dexterity),
                P::AsIs,
            ),
            bonus(
                StatTarget::Save(Save::Reflex),
                RankBase::BaseAttack,
                P::AsIs,
            ),
            stat_bonus(
                StatTarget::Save(Save::Will),
                Value::Rank(RankConfig {
                    base: RankBase::BaseStat(StatRef::Ability(Ability::Strength)),
                    progression: P::AsIs,
                    min: None,
                    max: Some(10),
                }),
            ),
            stat_bonus(
                StatTarget::Initiative,
                Value::Rank(RankConfig {
                    base: RankBase::FeatureRank(Fact::Feat(feat("power-attack"))),
                    progression: P::AsIs,
                    min: Some(3),
                    max: None,
                }),
            ),
        ],
    );
    let engine = engine_with(vec![reader], vec![]);
    let mut build = holding("reader", &["fighter", "rogue", "rogue", "rogue", "fighter"]);
    build.picks.insert(general_slot(1), pick_feat("dodge"));
    build.levels[0]
        .skill_ranks
        .insert(rules::SkillKey::new("mobility"), 1);
    build.levels[1]
        .skill_ranks
        .insert(rules::SkillKey::new("mobility"), 1);
    let sheet = walked(&engine, &build);
    let level = &sheet.levels[4];
    let in_force = |target: &rules::BonusTarget| -> i16 {
        level
            .state
            .bonuses_in_force
            .iter()
            .filter(|bonus| bonus.target.number() == Some(target))
            .filter(|bonus| bonus.name == "reader")
            .map(|bonus| bonus.amount)
            .sum()
    };
    let skill_target = |key: &str| rules::BonusTarget::Skill(rules::SkillKey::new(key));
    // `ClassLevel` with `m_ExceptClasses`: every class but the fighter, rogue 3.
    assert_eq!(in_force(&skill_target("athletics")), 3);
    // `CharacterLevel` 5, halved: 2.
    assert_eq!(in_force(&skill_target("mobility")), 2);
    // `FeatureListRanks`: Dodge 1 and the reader itself 1.
    assert_eq!(in_force(&skill_target("persuasion")), 2);
    // `BaseStat` of a skill, its `BaseValue`: the 2 ranks, times 2.
    assert_eq!(in_force(&skill_target("knowledge-arcana")), 4);
    // `StatBonus`: Dexterity 14's modifier, +2.
    assert_eq!(in_force(&rules::BonusTarget::Save(Save::Fortitude)), 2);
    // `BaseAttack`: fighter 2 (+2) and rogue 3 (+2).
    assert_eq!(in_force(&rules::BonusTarget::Save(Save::Reflex)), 4);
    // `BaseStat` of Strength 16, at most 10 (`m_UseMax`).
    assert_eq!(in_force(&rules::BonusTarget::Save(Save::Will)), 10);
    // `FeatureRank` of a feat not held, 0, at least 3 (`m_UseMin`).
    assert_eq!(in_force(&rules::BonusTarget::Initiative), 3);
}

/// `ContextRankConfig.GetBaseValue`'s `BaseStat`: the stat's `BaseValue`, which a race's ability modifiers don't change:
/// they are modifiers of descriptor `Racial` (`ElfRace` 25a5878d125338244896ebd3238226c8: `AddStatBonus` Dexterity +2
/// `Racial`; `SelectRaceStat.Apply`: the chosen +2 as `AddModifier(2, …, Racial)`).
#[test]
fn a_base_stat_rank_reads_the_score_without_its_racial_modifiers() {
    let reader = component_feat(
        "nimble-reader",
        vec![stat_bonus(
            StatTarget::Save(Save::Will),
            rank(
                RankBase::BaseStat(StatRef::Ability(Ability::Dexterity)),
                RankProgression::AsIs,
            ),
        )],
    );
    let engine = engine_with(vec![reader], vec![]);
    // An elf of Dexterity 14 bought, 16 with the race's +2: the base value is 14.
    let mut elf = build("elf", &["fighter"]);
    elf.picks
        .insert(general_slot(1), pick_feat("nimble-reader"));
    let sheet = walked(&engine, &elf);
    assert_eq!(
        sheet.at(1).unwrap().ability_scores.get(Ability::Dexterity),
        16
    );
    assert_eq!(
        bonus_of(&sheet, 1, "nimble-reader", &BonusTarget::Save(Save::Will)),
        14
    );
}

/// `ContextRankConfig.GetBaseValue`: `SummClassLevelWithArchetype` adds the ranks of `m_FeatureList` to the classes'
/// levels; `MaxClassLevelWithArchetype` is the highest class's level alone (`CheckAndSetMaxValue`), no feature read.
#[test]
fn a_max_class_level_with_archetype_adds_no_feature_ranks() {
    let base = |max| RankBase::ClassLevelWithArchetype {
        classes: BTreeSet::from([class("fighter"), class("rogue")]),
        except: false,
        archetypes: BTreeSet::new(),
        features: BTreeSet::from([Fact::Feat(feat("dodge"))]),
        max,
    };
    let counted = component_feat(
        "counted",
        vec![
            stat_bonus(
                StatTarget::Save(Save::Will),
                rank(base(false), RankProgression::AsIs),
            ),
            stat_bonus(
                StatTarget::Save(Save::Reflex),
                rank(base(true), RankProgression::AsIs),
            ),
        ],
    );
    let engine = engine_with(vec![counted], vec![]);
    let mut build = holding("counted", &["fighter", "rogue", "fighter"]);
    build.picks.insert(general_slot(1), pick_feat("dodge"));
    let sheet = walked(&engine, &build);
    // Fighter 2 + rogue 1 + Dodge 1 = 4; the highest class, fighter 2.
    assert_eq!(
        bonus_of(&sheet, 3, "counted", &BonusTarget::Save(Save::Will)),
        4
    );
    assert_eq!(
        bonus_of(&sheet, 3, "counted", &BonusTarget::Save(Save::Reflex)),
        2
    );
}

/// `AddStatBonus.OnTurnOn`: `Value × Fact.GetRank()`, so a second rank of the feature doubles its bonus as it comes; and
/// a value read off the character (`ContextRankConfig`'s `FeatureRank`) follows the pick that changes it.
#[test]
fn a_per_rank_value_and_a_value_another_pick_changes_follow_at_once() {
    let grant = |key: &str, extra: Vec<Component>| {
        let mut components = vec![Component::Grant(BTreeSet::from([fact_feature("mighty")]))];
        components.extend(extra);
        component_feat(key, components)
    };
    let dodge_reader = stat_bonus(
        StatTarget::Ability(Ability::Dexterity),
        rank(
            RankBase::FeatureRank(Fact::Feat(feat("dodge"))),
            RankProgression::AsIs,
        ),
    );
    let engine = engine_with(
        vec![
            grant("first-grant", vec![dodge_reader]),
            grant("second-grant", vec![]),
        ],
        vec![component_feature(
            "mighty",
            vec![stat_bonus(
                StatTarget::Ability(Ability::Strength),
                Value::PerRank(2),
            )],
        )],
    );
    let mut build = holding("first-grant", &times("fighter", 3));
    build
        .picks
        .insert(general_slot(1), pick_feat("second-grant"));
    build.picks.insert(general_slot(3), pick_feat("dodge"));
    let sheet = walked(&engine, &build);
    let level = |at: u8| sheet.at(at).unwrap();
    // Strength 16, Mighty twice at 1: +4.
    assert_eq!(level(1).feature_count(&feature("mighty")), 2);
    assert_eq!(level(1).ability_scores.get(Ability::Strength), 20);
    // Dexterity 14, Dodge picked at 3: +1 at 3.
    assert_eq!(level(2).ability_scores.get(Ability::Dexterity), 14);
    assert_eq!(level(3).ability_scores.get(Ability::Dexterity), 15);
}

/// `ContextRankConfig.ApplyProgression`'s `Custom`: the first item whose `BaseValue` is at or above the value gives its
/// `ProgressionValue`, the last item's past them all. Skill Focus (Mobility)'s shape (`SkillFocusAcrobatics`
/// 52dd89af385466c499338b7297896ded: +3, +6 from 10 ranks): its base is the skill's ranks.
#[test]
fn a_custom_rank_progression_takes_the_first_band_at_or_above_the_value() {
    let bands = RankProgression::Custom(vec![(9, 3), (100, 6)]);
    assert_eq!(bands.apply(0), 3);
    assert_eq!(bands.apply(9), 3);
    assert_eq!(bands.apply(10), 6);
    assert_eq!(bands.apply(101), 6, "the last band's value past them all");
    assert_eq!(RankProgression::Custom(vec![(5, 1), (6, 2)]).apply(7), 2);

    let mobility = rules::SkillKey::new("mobility");
    let focus = component_feat(
        "mobility-focus",
        vec![stat_bonus(
            StatTarget::Skill(mobility.clone()),
            rank(RankBase::BaseStat(StatRef::Skill(mobility.clone())), bands),
        )],
    );
    let engine = engine_with(vec![focus], vec![]);
    let mut build = holding("mobility-focus", &times("rogue", 10));
    for level in &mut build.levels {
        level.skill_ranks.insert(mobility.clone(), 1);
    }
    let sheet = walked(&engine, &build);
    let amount = |level: usize| {
        sheet.levels[level - 1]
            .state
            .bonuses_in_force
            .iter()
            .find(|bonus| bonus.name == "mobility-focus")
            .map(|bonus| bonus.amount)
    };
    assert_eq!(amount(9), Some(3));
    assert_eq!(amount(10), Some(6));
}

/// Perfect Body's shape (`DragonLevel2StrengthOverride` 9ed55389d2079714eab8495957964af2): a shared value of the base
/// Strength and −18 (`ContextCalculateSharedValue.Calculate`), times −1, at least 0 (`AddContextStatBonus.OnTurnOn`
/// with `HasMinimal`, `Minimal` 0): Strength is raised to 18 and never lowered.
#[test]
fn a_shared_value_below_zero_with_at_least_zero_adds_nothing() {
    let strength = StatTarget::Ability(Ability::Strength);
    let body = component_feat(
        "perfect-body",
        vec![Component::StatBonus {
            stat: strength,
            value: Value::Shared(SharedValue {
                count: Some(Box::new(rank(
                    RankBase::BaseStat(StatRef::Ability(Ability::Strength)),
                    RankProgression::AsIs,
                ))),
                bonus: Some(Box::new(Value::Flat(-18))),
                modifier: 1,
            }),
            descriptor: Descriptor::UntypedStackable,
            multiplier: -1,
            at_least: Some(0),
            only_if: None,
        }],
    );
    let engine = engine_with(vec![body], vec![]);
    // Strength 16: 18 − 16 = +2.
    let build = holding("perfect-body", &["fighter"]);
    let state = walked(&engine, &build).at(1).unwrap().clone();
    assert_eq!(state.ability_scores.get(Ability::Strength), 18);
    // Strength 18 at the start, 19 from the increase at 4: 18 − 19 = −1, so nothing.
    let mut strong = holding("perfect-body", &times("fighter", 4));
    strong.start.ability_scores = point_buy([18, 10, 10, 10, 10, 10]);
    strong.levels[3].ability_increase = Some(Ability::Strength);
    let sheet = walked(&engine, &strong);
    assert_eq!(
        sheet.at(1).unwrap().ability_scores.get(Ability::Strength),
        18
    );
    assert_eq!(
        sheet.at(4).unwrap().ability_scores.get(Ability::Strength),
        19
    );
}

/// `AddContextStatBonus.OnTurnOn` with `HasMinimal`: `Math.Max(value, Minimal)`, whatever the minimum. (Every blueprint
/// of the dump with `HasMinimal` set has `Minimal` 0; the eleven with another `Minimal` leave `HasMinimal` unset.)
#[test]
fn a_stat_bonus_with_a_minimum_adds_at_least_it() {
    let will = |amount| Component::StatBonus {
        stat: StatTarget::Save(Save::Will),
        value: Value::Flat(amount),
        descriptor: Descriptor::Untyped,
        multiplier: 1,
        at_least: Some(3),
        only_if: None,
    };
    let engine = engine_with(
        vec![
            component_feat("small", vec![will(1)]),
            component_feat("large", vec![will(5)]),
        ],
        vec![],
    );
    let plain = will_total(&engine, &build("human", &["fighter"]), 1);
    assert_eq!(
        will_total(&engine, &holding("small", &["fighter"]), 1),
        plain + 3
    );
    assert_eq!(
        will_total(&engine, &holding("large", &["fighter"]), 1),
        plain + 5
    );
}

/// `Designers/Mechanics/Buffs/AddStatBonusIfHasFact.cs`: `OnTurnOn` applies the bonus when `ShouldApplyBonus` holds
/// (`Update`); afterwards only two events look again: `HandleUnitGainFact` of a checked fact with `InvertCondition`,
/// and `HandleUnitLostFact` of one without. So a bonus held when its fact is lost goes, an inverted one goes when its
/// fact is gained, and a bonus whose fact is gained later stays off.
#[test]
fn a_stat_bonus_if_held_is_decided_when_turned_on_and_looked_at_again_on_the_events_it_handles() {
    let will = |amount, fact: Fact, not| Component::StatBonus {
        stat: StatTarget::Save(Save::Will),
        value: Value::Flat(amount),
        descriptor: Descriptor::Untyped,
        multiplier: 1,
        at_least: None,
        only_if: Some(IfHeld {
            facts: BTreeSet::from([fact]),
            all: false,
            not,
        }),
    };
    let power_attack = || Fact::Feat(feat("power-attack"));
    let engine = engine_with(
        vec![component_feat(
            "wary",
            vec![
                // Early is held below fighter 3, and lost at 3.
                Component::GrantAtClassLevel {
                    fact: fact_feature("early"),
                    classes: BTreeSet::from([class("fighter")]),
                    archetypes: BTreeSet::new(),
                    level: 3,
                    before: true,
                },
                will(2, fact_feature("early"), false),
                will(5, power_attack(), true),
                will(20, power_attack(), false),
            ],
        )],
        vec![component_feature("early", vec![])],
    );
    let plain = build("human", &times("fighter", 4));
    let mut build = holding("wary", &times("fighter", 4));
    build
        .picks
        .insert(general_slot(3), pick_feat("power-attack"));
    let added = |level| will_total(&engine, &build, level) - will_total(&engine, &plain, level);
    // Turned on at 1 with Early and without Power Attack: 2 and 5.
    assert_eq!(added(1), 7);
    // At 3 Early is lost (the 2 goes) and Power Attack gained (the inverted 5 goes; the 20, not inverted, isn't looked
    // at): nothing, and nothing at 4.
    assert_eq!(added(3), 0);
    assert_eq!(added(4), 0);
}

// ---- Class skills, skill points, proficiencies ------------------------------------------------------------------

/// `UnitLogic/FactLogic/AddClassSkill.cs` `OnTurnOn` (a class skill while held); `AddSkillPointPerCharacterLevel.cs`
/// `HandleUnitCalculateSkillPointsOnLevelup` (the character level more points, counted over every level by
/// `ApplySkillPoints.Apply`: one a level); `AddProficiencies.cs` `OnTurnOn` (the weapons and lists while held).
#[test]
fn class_skills_skill_points_and_proficiencies_from_components_join_the_state() {
    let schooled = component_feat(
        "schooled",
        vec![
            Component::ClassSkill(rules::SkillKey::new("persuasion")),
            Component::SkillPoints(1),
            Component::Proficiency {
                weapons: BTreeSet::from([rules::WeaponKey::new("longsword")]),
                lists: BTreeSet::from([WeaponProficiency::Exotic]),
                armour: BTreeSet::new(),
            },
            Component::Flag(rules::MechanicsFlag::WeaponFinesse),
        ],
    );
    let engine = engine_with(vec![schooled], vec![]);
    let plain = walked(&engine, &build("human", &times("wizard", 2)));
    let held = walked(&engine, &holding("schooled", &times("wizard", 2)));
    let state = held.at(2).unwrap();
    assert!(
        state
            .class_skills
            .contains(&rules::SkillKey::new("persuasion"))
    );
    assert!(
        !plain
            .at(2)
            .unwrap()
            .class_skills
            .contains(&rules::SkillKey::new("persuasion"))
    );
    // Picked at 1, after that level's points (`ApplySkillPoints` comes before `Features`): none at 1, then 2 at 2, the
    // character level less what it gave before.
    let available = |sheet: &rules::Sheet, level: usize| sheet.levels[level].skill_points.available;
    assert_eq!(available(&held, 0), available(&plain, 0));
    assert_eq!(available(&held, 1), available(&plain, 1) + 2);
    assert_eq!(
        state
            .weapon_proficiencies
            .get(&rules::WeaponKey::new("longsword")),
        Some(&1)
    );
    // The exotic list: Exotic Weapon Proficiency held for every exotic weapon.
    assert!(state.has_feat_for(
        &FeatKey::new("exotic-weapon-proficiency"),
        &weapon("dueling-sword")
    ));
    assert_eq!(
        state.flags.get(&rules::MechanicsFlag::WeaponFinesse),
        Some(&1)
    );
}

/// `AddSkillPointPerCharacterLevel.HandleUnitCalculateSkillPointsOnLevelup` is raised by `ApplySkillPoints.Apply`, whose
/// priority (`LevelUpActionPriority.ApplySkillPoints`) comes after `ApplyClass`: `ApplyClassMechanics.ApplyProgressions`
/// adds a level's row features (`LevelUpHelper.UpdateProgression`, `AddFeaturesFromProgression`) first. So a feature a
/// row grants counts at its own level, the character level at once; one picked (`Features`, after) from the next.
#[test]
fn a_row_granted_features_skill_points_count_at_its_own_level() {
    let mut data = world();
    (data.features.iter_mut())
        .find(|entry| entry.key == feature("rogue/sneak-attack"))
        .expect("the rogue's Sneak Attack")
        .components = vec![Component::SkillPoints(1)];
    let with = engine_on(data);
    let without = engine_on(world());
    // Sneak Attack comes with rogue 1, at character level 2.
    let build = build("human", &["fighter", "rogue", "fighter"]);
    let (with, without) = (walked(&with, &build), walked(&without, &build));
    let added = |level: usize| {
        i16::from(with.levels[level - 1].skill_points.available)
            - i16::from(without.levels[level - 1].skill_points.available)
    };
    assert_eq!(added(1), 0);
    assert_eq!(added(2), 2);
    assert_eq!(added(3), 1);
}

/// `PrerequisiteClassSpellLevel.GetClassSpellLevel` (the class's spellbook's highest level) and
/// `PrerequisiteCasterType.CheckInternal` (a class with a spellbook of the type, whatever it casts yet), compiled into
/// the planner's leaves.
#[test]
fn a_class_spell_level_and_a_caster_type_read_the_classes_spellbooks() {
    let engine = engine_with(
        vec![
            component_feat(
                "wizard-second",
                vec![requires(Condition::ClassSpellLevel(class("wizard"), 2))],
            ),
            component_feat(
                "divine-caster",
                vec![requires(Condition::CasterType(SpellType::Divine))],
            ),
            component_feat(
                "arcane-caster",
                vec![requires(Condition::CasterType(SpellType::Arcane))],
            ),
        ],
        vec![],
    );
    let wizard = build("human", &times("wizard", 3));
    let check = |build: &Build, feat: &str| {
        status(&engine, build, &general_slot(3), &pick_feat(feat)).is_ok()
    };
    assert!(check(&wizard, "wizard-second"));
    assert!(check(&wizard, "arcane-caster"));
    assert!(!check(&wizard, "divine-caster"));
    let early = build("human", &["wizard", "fighter", "fighter"]);
    assert!(!check(&early, "wizard-second"));
}

/// `PrerequisiteClassSpellLevel.GetClassSpellLevel` is an `int?`: the class's spellbook's `MaxSpellLevel`, or `null`
/// for a class not taken or without a spellbook, which no level meets (C#'s lifted `>=` with `null` is false), 0
/// included. `Spellbook.GetMaxSpellLevel` is the highest spell level whose count in the table's row is 0 or more, so a
/// spellbook whose row lists level 0 (`BloodragerSpellPerDayTable` caf7018942861664ebe87687893ad05d: `[0]` at caster
/// levels 0 to 3) meets a level 0 before its first spell level. (No blueprint of the dump requires level 0.)
#[test]
fn a_class_spell_level_of_zero_needs_the_classs_spellbook() {
    let mut data = world();
    // A ranger with divine spells from 4: a spellbook from level 1.
    let ranger = (data.all_classes_mut())
        .find(|entry| entry.key == class("ranger"))
        .expect("the test world's ranger");
    ranger.casting_ability = Some(Ability::Wisdom);
    for row in &mut ranger.levels {
        if row.level >= 4 {
            row.divine_spell_level = 1;
        }
    }
    let needs = |key: &str, of: &str| {
        component_feat(
            key,
            vec![requires(Condition::ClassSpellLevel(class(of), 0))],
        )
    };
    data.feats.extend([
        needs("ranger-zero", "ranger"),
        needs("fighter-zero", "fighter"),
        needs("wizard-zero", "wizard"),
    ]);
    let engine = engine_on(data);
    let check = |build: &Build, feat: &str| {
        status(&engine, build, &general_slot(1), &pick_feat(feat)).is_ok()
    };
    let ranger = build("human", &["ranger"]);
    assert!(check(&ranger, "ranger-zero"));
    // The fighter has no spellbook, and the wizard isn't taken.
    assert!(!check(&ranger, "fighter-zero"));
    let fighter = build("human", &["fighter"]);
    assert!(!check(&fighter, "fighter-zero"));
    assert!(!check(&fighter, "wizard-zero"));
}

// ---- The check --------------------------------------------------------------------------------------------------

/// `GameData::check` on components: a key one names must be the data's, a hand field and the component that replaces
/// it can't both be on one entry, a component means something on its entry, and a step is never 0.
#[test]
fn check_refuses_an_unknown_key_in_a_component_and_a_field_beside_its_component() {
    let mut data = world();
    let mut both = component_feat(
        "both",
        vec![requires(has(Fact::Feat(feat("power-attack"))))],
    );
    both.prerequisite = Some(Prerequisite::Feat(feat("dodge")));
    data.feats.push(both);
    data.feats.push(component_feat(
        "unknown",
        vec![Component::Grant(BTreeSet::from([fact_feature("nowhere")]))],
    ));
    data.feats.push(component_feat(
        "chosen",
        vec![Component::Grant(BTreeSet::from([Fact::Class(class(
            "fighter",
        ))]))],
    ));
    data.feats.push(component_feat(
        "zero",
        vec![stat_bonus(
            StatTarget::Initiative,
            rank(RankBase::CharacterLevel, RankProgression::DivStep(0)),
        )],
    ));
    data.features.push(component_feature(
        "picked-never",
        vec![requires(Condition::CharacterLevel(3))],
    ));
    // `PrerequisiteFeaturesFromList` over deities only: its `IsDeity` rule isn't modelled.
    data.feats.push(component_feat(
        "devout",
        vec![requires(Condition::AtLeast(
            1,
            BTreeSet::from([
                Fact::Deity(DeityKey::new("lawgiver")),
                Fact::Deity(DeityKey::new("balance")),
            ]),
        ))],
    ));
    let problems: Vec<(DataAt, DataIssue)> = data
        .check()
        .into_iter()
        .map(|problem| (problem.at, problem.issue))
        .collect();
    let at_feat = |key: &str| DataAt::Feat(feat(key));
    let at_feature = |key: &str| DataAt::Feature(feature(key));
    for expected in [
        (
            at_feat("both"),
            DataIssue::FieldAndComponent("prerequisite"),
        ),
        (
            at_feat("unknown"),
            DataIssue::UnknownKey(DataAt::Feature(FeatureKey::new("nowhere"))),
        ),
        (at_feat("chosen"), DataIssue::NotGrantable),
        (at_feat("zero"), DataIssue::ZeroStep),
        (
            at_feature("picked-never"),
            DataIssue::ComponentOnWrongEntry("Requires"),
        ),
        (at_feat("devout"), DataIssue::DeityList),
    ] {
        assert!(
            problems.contains(&expected),
            "{expected:?} in {problems:#?}"
        );
    }
    assert_eq!(problems.len(), 6, "{problems:#?}");
}

// ---- Agreement --------------------------------------------------------------------------------------------------

/// A prerequisite as `Requires` components, when every leaf has a condition: the test world's own prerequisites in the
/// component form, which compiles back into them.
fn as_requires(prerequisite: &Prerequisite) -> Option<Vec<Component>> {
    let (group, children) = match prerequisite {
        Prerequisite::All(children) => (Group::All, children.as_slice()),
        Prerequisite::Any(children) => (Group::Any, children.as_slice()),
        other => (Group::All, std::slice::from_ref(other)),
    };
    children
        .iter()
        .map(|child| {
            Some(Component::Requires {
                condition: condition(child)?,
                group,
            })
        })
        .collect()
}

fn condition(prerequisite: &Prerequisite) -> Option<Condition> {
    use Prerequisite as P;
    let each =
        |children: &[Prerequisite]| children.iter().map(condition).collect::<Option<Vec<_>>>();
    Some(match prerequisite {
        P::All(children) => Condition::All(each(children)?),
        P::Any(children) => Condition::Any(each(children)?),
        P::Not(child) => Condition::Not(Box::new(condition(child)?)),
        P::AtLeast(n, children) => Condition::AtLeast(
            *n,
            children
                .iter()
                .map(|child| match condition(child)? {
                    Condition::Has(fact) => Some(fact),
                    _ => None,
                })
                .collect::<Option<_>>()?,
        ),
        P::Feat(key) => has(Fact::Feat(key.clone())),
        P::FeatFor { feat, value } => Condition::ParametrizedFeature(feat.clone(), value.clone()),
        P::MythicFeat(key) => has(Fact::MythicFeat(key.clone())),
        P::MythicAbility(key) => has(Fact::MythicAbility(key.clone())),
        P::ClassLevel { class, level } => Condition::ClassLevel(class.clone(), *level),
        P::Archetype(key) => has(Fact::Archetype(key.clone())),
        P::Feature { feature, count: 1 } => has(Fact::Feature(feature.clone())),
        P::Feature { feature, count } => Condition::FactRank(feature.clone(), *count),
        P::SelectionOption(key) => has(Fact::Option(key.clone())),
        P::Selection(key) => has(Fact::Selection(key.clone())),
        P::SkillRanks { skill, ranks } => Condition::Stat {
            stat: StatRef::Skill(skill.clone()),
            value: *ranks,
            full: false,
        },
        P::AbilityScore { ability, score } => Condition::Stat {
            stat: StatRef::Ability(*ability),
            value: *score,
            full: false,
        },
        P::BaseAttackBonus(bab) => Condition::Stat {
            stat: StatRef::BaseAttack,
            value: *bab,
            full: false,
        },
        P::SpellLevel { spell_type, level } => Condition::CasterTypeSpellLevel {
            caster: *spell_type,
            level: *level,
            spontaneous: false,
        },
        P::SpontaneousSpellLevel { spell_type, level } => Condition::CasterTypeSpellLevel {
            caster: *spell_type,
            level: *level,
            spontaneous: true,
        },
        P::CharacterLevel(level) => Condition::CharacterLevel(*level),
        P::MythicRank(rank) => Condition::MythicLevel(*rank),
        P::Alignment(alignment) => Condition::Alignment(BTreeSet::from([*alignment])),
        P::Deity(key) => has(Fact::Deity(key.clone())),
        P::Race(key) => has(Fact::Race(key.clone())),
        // The planner's own leaves stay hand-shaped.
        _ => return None,
    })
}

/// The test world with every prerequisite a component form exists for moved into `Requires` components, and how many
/// entries moved.
fn world_in_components() -> (GameData, usize) {
    let mut data = world();
    let mut moved = 0;
    let mut convert = |prerequisite: &mut Option<Prerequisite>, components: &mut Vec<Component>| {
        if let Some(requires) = prerequisite.as_ref().and_then(as_requires) {
            *components = requires;
            *prerequisite = None;
            moved += 1;
        }
    };
    for feat in &mut data.feats {
        convert(&mut feat.prerequisite, &mut feat.components);
    }
    for option in &mut data.options {
        convert(&mut option.prerequisite, &mut option.components);
    }
    for ability in &mut data.mythic_abilities {
        convert(&mut ability.prerequisite, &mut ability.components);
    }
    for feat in &mut data.mythic_feats {
        convert(&mut feat.prerequisite, &mut feat.components);
    }
    for archetype in &mut data.archetypes {
        convert(&mut archetype.prerequisite, &mut archetype.components);
    }
    for class in data.all_classes_mut() {
        convert(&mut class.prerequisite, &mut class.components);
    }
    (data, moved)
}

/// `agreement::judge` on the test world whose prerequisites are components: no picker disagrees with the walk, and
/// each Build of the sample walks to the very sheet the hand-shaped world gives it.
#[test]
fn every_picker_agrees_with_the_walk_on_a_world_of_components() {
    let (data, moved) = world_in_components();
    assert!(moved > 50, "{moved} entries in components");
    let engine = engine_on(data);
    let hand = rules::test_world::engine();
    let corpus = corpus(&engine, 1..=1, Some(SAMPLE));
    assert!(corpus.len() > 150, "a sample of {} Builds", corpus.len());
    for (name, build) in &corpus {
        assert_eq!(engine.evaluate(build), hand.evaluate(build), "{name}");
    }
    let sweep = judge_all(&engine, &corpus);
    assert!(sweep.judged > 50_000, "{} options judged", sweep.judged);
    assert!(sweep.lines.is_empty(), "{:#?}", sweep.lines);
}
