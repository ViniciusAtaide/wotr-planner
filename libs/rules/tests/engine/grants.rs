//! The grant index's static half (ticket 334, design-G §1): every place the game data gives facts, by the entry that
//! gives them and by the fact, read off the data alone (no Build).

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, BackgroundKey, ClassKey, Component, DataAt, DeityKey, Fact, FeatKey, FeatureKey,
    FeatureSelectionKey, GameData, Grant, GrantIndex, GrantRule, GrantValue, MythicAbilityKey,
    MythicFeatKey, MythicPathKey, Parameter, RaceKey, Satisfies, SelectionOptionKey, When,
};

use crate::support::*;

fn feat_fact(key: &str) -> Fact {
    Fact::Feat(FeatKey::new(key))
}

fn feature_fact(key: &str) -> Fact {
    Fact::Feature(FeatureKey::new(key))
}

fn selection_fact(key: &str) -> Fact {
    Fact::Selection(FeatureSelectionKey::new(key))
}

fn every(fact: Fact) -> Grant {
    Grant {
        fact,
        value: GrantValue::Every,
        opens_slot: false,
    }
}

fn slot(key: &str) -> Grant {
    Grant {
        fact: selection_fact(key),
        value: GrantValue::Every,
        opens_slot: true,
    }
}

fn fixed(feat: &str, value: Parameter) -> Grant {
    Grant {
        fact: feat_fact(feat),
        value: GrantValue::Fixed(value),
        opens_slot: false,
    }
}

fn picked(feat: &str) -> Grant {
    Grant {
        fact: feat_fact(feat),
        value: GrantValue::Picked,
        opens_slot: false,
    }
}

/// The test world with a feature that carries each grant component: Grant, GrantAtClassLevel, GrantIf,
/// GrantOnApply and GrantWithValue.
fn world_with_components() -> GameData {
    let mut data = world();
    feature_mut(&mut data, "legend/legend").components.extend([
        Component::Grant(BTreeSet::from([feat_fact("dodge")])),
        Component::GrantAtClassLevel {
            fact: feat_fact("toughness"),
            classes: BTreeSet::new(),
            archetypes: BTreeSet::new(),
            level: 5,
            before: false,
        },
        Component::GrantIf {
            has: feat_fact("dodge"),
            not: true,
            fact: feat_fact("cleave"),
        },
        Component::GrantOnApply(feat_fact("power-attack")),
        Component::GrantWithValue {
            feat: FeatKey::new("weapon-focus"),
            value: weapon("longsword"),
        },
    ]);
    data
}

/// Whether the index has, among what `by` gives at `when`, a site that gives `grant`.
fn has(index: &GrantIndex, by: &DataAt, when: &When, grant: &Grant) -> bool {
    index
        .gives(by)
        .iter()
        .any(|site| &site.when == when && site.grants.contains(grant))
}

/// Each grant kind of design-G §1's `When` and `GrantValue`, from one entry of the test world each.
#[test]
fn every_grant_kind_is_a_site() {
    let index = GrantIndex::new(&world_with_components());
    let class_at = |key: &str| DataAt::Class(ClassKey::new(key));
    let legend_feature = DataAt::Feature(FeatureKey::new("legend/legend"));
    let component = |rule| When::Component(rule);
    let expected: Vec<(DataAt, When, Grant)> = vec![
        // A class row's feat and selection.
        (
            class_at("monk"),
            When::ClassRow { level: 1 },
            every(feat_fact("improved-unarmed-strike")),
        ),
        (
            class_at("monk"),
            When::ClassRow { level: 1 },
            slot("monk/bonus-feats"),
        ),
        // An archetype row's `feats_for`: Weapon Focus for the dueling sword.
        (
            DataAt::Archetype(ArchetypeKey::new("warpriest/mantis-zealot")),
            When::ClassRow { level: 1 },
            fixed("weapon-focus", weapon("dueling-sword")),
        ),
        // An option's chosen feat, and its row's chosen feat.
        (
            DataAt::SelectionOption(SelectionOptionKey::new("rogue/rogue-talents/weapon-focus")),
            When::Always,
            picked("weapon-focus"),
        ),
        (
            DataAt::SelectionOption(SelectionOptionKey::new(
                "monk/zen-archer/way-of-the-bow/weapon-focus",
            )),
            When::OptionRow { level: 6 },
            picked("weapon-specialization"),
        ),
        // A path's rank row, and the features a switch to a path grants.
        (
            DataAt::MythicPath(MythicPathKey::new("trickster")),
            When::Rank { rank: 2 },
            every(feature_fact("trickster/hard-to-kill")),
        ),
        (
            DataAt::MythicPath(MythicPathKey::new("legend")),
            When::Always,
            every(feature_fact("legend/legend")),
        ),
        // A feat's and a mythic feat's selection, whose pick is the slot it opens.
        (
            DataAt::Feat(FeatKey::new("extra-hex")),
            When::Always,
            slot("witch/extra-hex"),
        ),
        (
            DataAt::MythicFeat(MythicFeatKey::new("extra-feat")),
            When::Always,
            slot(rules::BASIC_FEAT_SELECTION),
        ),
        // A mythic ability's features and selection.
        (
            DataAt::MythicAbility(MythicAbilityKey::new("bloodline-ascendance")),
            When::Always,
            every(feature_fact("sorcerer/bloodline-ascendance")),
        ),
        (
            DataAt::MythicAbility(MythicAbilityKey::new("second-bloodline")),
            When::Always,
            slot("sorcerer/bloodline"),
        ),
        // A race's selection of feats.
        (
            DataAt::Race(RaceKey::new("human")),
            When::Always,
            slot(rules::BASIC_FEAT_SELECTION),
        ),
        // A background's feats, and its feat for one value.
        (
            DataAt::Background(BackgroundKey::new("martial-disciple")),
            When::Always,
            every(feat_fact("improved-unarmed-strike")),
        ),
        (
            DataAt::Background(BackgroundKey::new("aldori-swordsman")),
            When::Always,
            fixed("exotic-weapon-proficiency", weapon("dueling-sword")),
        ),
        // A deity's favoured weapons: the unarmed strike's Improved Unarmed Strike, an exotic weapon's proficiency for
        // that weapon.
        (
            DataAt::Deity(DeityKey::new("fist")),
            When::Always,
            every(feat_fact("improved-unarmed-strike")),
        ),
        (
            DataAt::Deity(DeityKey::new("lawgiver")),
            When::Always,
            fixed("exotic-weapon-proficiency", weapon("dueling-sword")),
        ),
        // Each grant component, under its own rule.
        (
            legend_feature.clone(),
            component(GrantRule::Held),
            every(feat_fact("dodge")),
        ),
        (
            legend_feature.clone(),
            component(GrantRule::AtClassLevel {
                level: 5,
                before: false,
            }),
            every(feat_fact("toughness")),
        ),
        (
            legend_feature.clone(),
            component(GrantRule::IfHas {
                has: feat_fact("dodge"),
                not: true,
            }),
            every(feat_fact("cleave")),
        ),
        (
            legend_feature.clone(),
            component(GrantRule::OnApply),
            every(feat_fact("power-attack")),
        ),
        (
            legend_feature,
            component(GrantRule::Held),
            fixed("weapon-focus", weapon("longsword")),
        ),
    ];
    let missing: Vec<&(DataAt, When, Grant)> = expected
        .iter()
        .filter(|(by, when, grant)| !has(&index, by, when, grant))
        .collect();
    assert!(missing.is_empty(), "no site: {missing:#?}");
}

/// A class row's site names its class, an archetype row's its base class; no other site names one.
#[test]
fn a_row_site_names_its_class() {
    let index = GrantIndex::new(&world());
    let class = |by: &DataAt| -> BTreeSet<Option<ClassKey>> {
        index
            .gives(by)
            .iter()
            .map(|site| site.class.clone())
            .collect()
    };
    assert_eq!(
        class(&DataAt::Class(ClassKey::new("monk"))),
        BTreeSet::from([Some(ClassKey::new("monk"))])
    );
    assert_eq!(
        class(&DataAt::Archetype(ArchetypeKey::new(
            "warpriest/mantis-zealot"
        ))),
        BTreeSet::from([Some(ClassKey::new("warpriest"))])
    );
    assert_eq!(
        class(&DataAt::Deity(DeityKey::new("fist"))),
        BTreeSet::from([None])
    );
}

/// A deity whose favoured weapon is martial gives no feat: no feat checks proficiency with one martial weapon.
#[test]
fn a_martial_favoured_weapon_gives_nothing() {
    let index = GrantIndex::new(&world());
    assert!(
        index
            .gives(&DataAt::Deity(DeityKey::new("balance")))
            .is_empty()
    );
}

#[test]
fn given_by_is_the_inverse_of_gives() {
    let data = world_with_components();
    let index = GrantIndex::new(&data);
    let mut pairs = 0;
    for (by, _) in data.blueprints() {
        for site in index.gives(&by) {
            assert_eq!(site.by, by);
            for grant in &site.grants {
                pairs += 1;
                assert!(
                    index
                        .given_by(&grant.fact)
                        .any(|(s, g)| std::ptr::eq(s, site) && std::ptr::eq(g, grant)),
                    "{by:?} gives {grant:?}, which given_by doesn't list"
                );
            }
        }
    }
    assert!(pairs > 0);
    let facts: BTreeSet<&Fact> = (data.blueprints().iter())
        .flat_map(|(by, _)| index.gives(by))
        .flat_map(|site| site.grants.iter().map(|grant| &grant.fact))
        .collect();
    for fact in facts {
        for (site, grant) in index.given_by(fact) {
            assert_eq!(&grant.fact, fact);
            assert!(
                index.gives(&site.by).iter().any(|s| std::ptr::eq(s, site)),
                "given_by lists a site of {:?} that gives doesn't",
                site.by
            );
        }
    }
}

/// The order of the data's lists, of their rows and of a row's grants changes nothing: the index sorts its sites and
/// their grants, so the canonical data gives the same index.
#[test]
fn the_index_ignores_the_datas_order() {
    let data = world_with_components();
    let mut shuffled = data.clone();
    shuffled.races.reverse();
    shuffled.subraces.reverse();
    shuffled.feats.reverse();
    shuffled.classes.reverse();
    shuffled.prestige_classes.reverse();
    shuffled.archetypes.reverse();
    shuffled.features.reverse();
    shuffled.selections.reverse();
    shuffled.options.reverse();
    shuffled.mythic_abilities.reverse();
    shuffled.mythic_feats.reverse();
    shuffled.mythic_paths.reverse();
    shuffled.backgrounds.reverse();
    shuffled.deities.reverse();
    shuffled.gear.reverse();
    for class in shuffled.all_classes_mut() {
        class.levels.reverse();
    }
    for path in &mut shuffled.mythic_paths {
        path.ranks.reverse();
    }
    assert_eq!(
        GrantIndex::with_offers(&shuffled),
        GrantIndex::with_offers(&data)
    );
    assert_eq!(
        GrantIndex::with_offers(&data.clone().canonical()),
        GrantIndex::with_offers(&data)
    );
}

/// The engine's index holds no offers: only game-check reads them, and builds its index with them.
#[test]
#[should_panic(expected = "an index built with_offers")]
fn an_index_without_offers_has_none_to_give() {
    let index = GrantIndex::new(&world());
    assert_eq!(index.sites(), GrantIndex::with_offers(&world()).sites());
    index.offered_in(&feat_fact("dodge"));
}

/// A key the data lacks gives nothing and is given by nothing; data naming keys it lacks (unchecked data, as
/// `GameData::check` reads it) still makes an index.
#[test]
fn an_unknown_key_gives_nothing() {
    let mut data = world();
    class_mut(&mut data, "monk").levels[0]
        .features
        .push(FeatureKey::new("no-such-feature"));
    data.deities[0]
        .favoured_weapons
        .insert(rules::WeaponKey::new("no-such-weapon"));
    let index = GrantIndex::with_offers(&data);
    assert!(
        index
            .gives(&DataAt::Feat(FeatKey::new("no-such-feat")))
            .is_empty()
    );
    assert!(index.given_by(&feat_fact("no-such-feat")).next().is_none());
    assert!(index.offered_in(&feat_fact("no-such-feat")).is_empty());
    let givers: Vec<&DataAt> = (index.given_by(&feature_fact("no-such-feature")))
        .map(|(site, _)| &site.by)
        .collect();
    assert_eq!(givers, [&DataAt::Class(ClassKey::new("monk"))]);
    let weapons: Vec<&GrantValue> = (index.gives(&DataAt::Deity(data.deities[0].key.clone())))
        .iter()
        .flat_map(|site| &site.grants)
        .map(|grant| &grant.value)
        .collect();
    assert!(
        !weapons.contains(&&GrantValue::Fixed(weapon("no-such-weapon"))),
        "{weapons:?}"
    );
}

/// The selections that offer a fact as a pick: their own options, the options they share, their feats and their
/// mythic abilities.
#[test]
fn offered_in_lists_the_selections_whose_lists_offer_a_fact() {
    let index = GrantIndex::with_offers(&world());
    // Its own selection's, and Extra Rogue Talent's, which shares the rogue talents.
    assert_eq!(
        index.offered_in(&Fact::Option(SelectionOptionKey::new(
            "rogue/rogue-talents/weapon-focus"
        ))),
        [
            FeatureSelectionKey::new("rogue/extra-rogue-talent"),
            FeatureSelectionKey::new("rogue/rogue-talents"),
        ]
    );
    assert!(
        index
            .offered_in(&feat_fact("dodge"))
            .contains(&FeatureSelectionKey::new(rules::BASIC_FEAT_SELECTION))
    );
    assert!(
        index
            .offered_in(&Fact::MythicAbility(MythicAbilityKey::new("mythic-charge")))
            .iter()
            .any(|selection| selection.as_str().starts_with("mythic/"))
    );
}

#[test]
fn a_refused_leaf_is_never_satisfied() {
    let grant = every(feat_fact("dodge"));
    assert_eq!(
        grant.satisfies(&feat_fact("dodge"), None, true, None),
        Satisfies::No
    );
    assert_eq!(
        grant.satisfies(&feat_fact("dodge"), None, false, None),
        Satisfies::Yes
    );
    assert_eq!(
        grant.satisfies(&feat_fact("toughness"), None, false, None),
        Satisfies::No
    );
}

#[test]
fn a_grant_for_one_value_meets_only_that_value() {
    let grant = fixed("weapon-focus", weapon("longsword"));
    let fact = feat_fact("weapon-focus");
    assert_eq!(
        grant.satisfies(&fact, Some(&weapon("longsword")), false, None),
        Satisfies::Yes
    );
    assert_eq!(
        grant.satisfies(&fact, Some(&weapon("dagger")), false, None),
        Satisfies::No
    );
    assert_eq!(grant.satisfies(&fact, None, false, None), Satisfies::Yes);
}

#[test]
fn a_grant_for_every_value_meets_any() {
    let grant = every(feat_fact("weapon-focus"));
    let fact = feat_fact("weapon-focus");
    assert_eq!(
        grant.satisfies(&fact, Some(&weapon("dagger")), false, None),
        Satisfies::Yes
    );
    assert_eq!(grant.satisfies(&fact, None, false, None), Satisfies::Yes);
}

#[test]
fn a_picked_value_meets_the_picks_value() {
    let grant = picked("weapon-focus");
    let fact = feat_fact("weapon-focus");
    let longsword = weapon("longsword");
    assert_eq!(
        grant.satisfies(&fact, Some(&longsword), false, Some(&longsword)),
        Satisfies::Yes
    );
    assert_eq!(
        grant.satisfies(&fact, Some(&longsword), false, Some(&weapon("dagger"))),
        Satisfies::No
    );
    // No pick yet: some pick can.
    assert_eq!(
        grant.satisfies(&fact, Some(&longsword), false, None),
        Satisfies::Yes
    );
}
