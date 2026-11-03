//! Taking the domains: the cleric's domain selection and the selections that share it (inquisitor, Sacred
//! Huntsmaster, druid, Blight Druid, Winter Child, Divine Hunter, the Separatist's second domain and alignments), and
//! their check.

use std::collections::BTreeSet;

use rules::{
    DataAt, FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, MythicAbilityKey,
    Prerequisite, SelectionOptionKey,
};

use crate::guide::Guide;
use crate::names::norm;
use crate::picks::{Pick, Picks, Rule};

use super::{Family, sharing, starts};

/// The druid's domains: "If you choose a domain, you can choose from: Air, Animal, Earth, Fire, Ice, Plant, Water and
/// Weather."
const DRUID_DOMAINS: [&str; 8] = [
    "air", "animal", "earth", "fire", "ice", "plant", "water", "weather",
];
/// "May choose from the following domains: Air, Animal, Darkness, Death, Destruction, Earth, Fire, Murder, Plant,
/// Undead, Water, Weather." (Blight Druid)
const BLIGHT_DOMAINS: [&str; 12] = [
    "air",
    "animal",
    "darkness",
    "death",
    "destruction",
    "earth",
    "fire",
    "murder",
    "plant",
    "undead",
    "water",
    "weather",
];
/// "The Hellknight gains access to one of the following domains: Artifice, Glory, Knowledge, Law, Magic, Nobility,
/// Protection, Rune, Strength, Travel or War." (Pentamic Faith)
pub(super) const PENTAMIC_DOMAINS: [&str; 11] = [
    "artifice",
    "glory",
    "knowledge",
    "law",
    "magic",
    "nobility",
    "protection",
    "rune",
    "strength",
    "travel",
    "war",
];

const DOMAIN: &str = "cleric/domain";
const INQUISITOR_DOMAIN: &str = "inquisitor/domain";
const HUNTSMASTER_DOMAIN: &str = "inquisitor/sacred-huntsmaster/domain";
const DRUID_DOMAIN: &str = "druid/domain";
const BLIGHT_DOMAIN: &str = "druid/blight-druid/domain";
const WINTER_DOMAIN: &str = "druid/winter-child/domain";
const DIVINE_HUNTER_DOMAIN: &str = "hunter/divine-hunter/domain";
const SEPARATIST: &str = "cleric/separatist";
const SEPARATIST_DOMAIN: &str = "cleric/separatist/domain";
pub(super) const PENTAMIC_DISCIPLINE: &str = "hellknight/disciplines/pentamic-faith";

/// The cells a domain table names its picks and its powers with: "Domain Abilities" at 1 (one or two, "(1)" and
/// "(2)"), "Domain Ability" at 6 and 8.
const PICK_CELLS: [&str; 3] = [
    "domain-abilities",
    "domain-abilities-1",
    "domain-abilities-2",
];
const POWER_CELLS: [&str; 3] = ["domain-ability", "domain-ability-1", "domain-ability-2"];

/// Each cell `{table}/{slug}` of `table` at `levels` is `pick`.
fn cells(picks: &mut Picks, table: &str, slugs: &[&str], levels: &[u8], pick: &Pick) {
    for slug in slugs {
        picks.rule(Rule {
            table: Some(table.to_owned()),
            levels: Some(levels.to_vec()),
            ..Rule::every(&format!("{table}/{slug}"), pick.clone())
        });
    }
}

/// The domains, built with the classes that take them. Each table's domain cells at 1 are the picks, and the cells
/// the domains' rows stand for (the "Animal domain only" companion at 4, the powers at 6 and 8) are the held
/// domain's rows ([`Pick::OptionRow`]), so none of those features is made. [`check_domains`] asserts the rows in
/// force.
pub(super) fn provide_domains(guide: &Guide, domains: &Family, picks: &mut Picks) {
    let domain = Pick::Selection(FeatureSelectionKey::new(DOMAIN));
    let held = Pick::OptionRow;
    picks.selection("cleric", separatist_alignments(guide, domains));
    // A "Domain" line (Impossible Domain's) names the Divine Hunter's "Domain" feature, as it did when the domains
    // were built after the lines were resolved.
    picks.unnamed(DOMAIN);
    for feature in &domains.features {
        picks.feature("cleric", feature.clone());
    }

    // Cleric: the domain cells at 1 are the picks (two, or one for an archetype that "May only select a single
    // domain"); the cells at 4 ("Animal domain only"), 6 and 8 are the domains' rows.
    for table in crate::build::class_tables(guide, "cleric") {
        cells(picks, &table, &PICK_CELLS, &[1], &domain);
        cells(picks, &table, &["animal-companion"], &[4], &held);
        cells(picks, &table, &POWER_CELLS, &[6, 8], &held);
    }
    // The Separatist (ticket 33's decision on 31-1): "Higher level abilities are delayed by two class levels." Its
    // second domain is its own selection, two levels behind, granted beside its first at 1; that domain's cells (the
    // companion at 6, a Domain Ability at 8 and 10) are its rows.
    crate::follow_ups::says(
        guide,
        "cleric",
        "forbidden-rites",
        "Higher level abilities are delayed by two class levels.",
    );
    assert_eq!(domains.selection.options.len(), 37, "the 37 domains");
    picks.selection(
        SEPARATIST,
        FeatureSelection {
            levels_behind: 2,
            ..sharing(
                SEPARATIST_DOMAIN,
                "Domain (Separatist)",
                "cleric#forbidden-rites".into(),
                &["cleric"],
                domains.all_keys(),
            )
        },
    );
    picks.row_selection(SEPARATIST, 1, SEPARATIST_DOMAIN);
    cells(picks, SEPARATIST, &["animal-companion"], &[6], &held);
    cells(picks, SEPARATIST, &["domain-ability"], &[10], &held);

    // Inquisitor: one domain, levelled by inquisitor levels; the Sacred Huntsmaster "May not select the Animal
    // domain" (and, character-creation says, not Scalykind); the Faith Hunter "Cannot select a domain".
    picks.selection(
        "inquisitor",
        sharing(
            INQUISITOR_DOMAIN,
            "Domain",
            "inquisitor".to_owned(),
            &["inquisitor"],
            domains.all_keys(),
        ),
    );
    picks.selection(
        "inquisitor/sacred-huntsmaster",
        sharing(
            HUNTSMASTER_DOMAIN,
            "Domain (Sacred Huntsmaster)",
            "inquisitor#sacred-huntsmaster".to_owned(),
            &["inquisitor"],
            domains
                .all_keys()
                .into_iter()
                .filter(|key| {
                    !key.as_str().ends_with("/animal") && !key.as_str().ends_with("/scalykind")
                })
                .collect(),
        ),
    );
    for table in crate::build::class_tables(guide, "inquisitor") {
        let pick = if table == "inquisitor/sacred-huntsmaster" {
            HUNTSMASTER_DOMAIN
        } else {
            INQUISITOR_DOMAIN
        };
        let pick = Pick::Selection(FeatureSelectionKey::new(pick));
        cells(picks, &table, &PICK_CELLS, &[1], &pick);
        cells(picks, &table, &["animal-companion"], &[4], &held);
        cells(picks, &table, &POWER_CELLS, &[6, 8], &held);
    }

    // Druid: Nature Bond's domain option grants the pick (levelled by druid levels; the bond is
    // [`crate::class_rules`]'), the druid's cells at 4 ("Animal domain only"), 6 ("Air, Earth, Fire, Plant and
    // Water") and 8 (Lightning Lord: "Weather domain") are the domains' rows; the Blight Druid picks from its twelve
    // and "Does not receive an Animal Companion" (it skips the companion at 4); the Winter Child "must select the Ice
    // Subdomain".
    picks.selection(
        "druid",
        sharing(
            DRUID_DOMAIN,
            "Domain",
            "druid".to_owned(),
            &["druid"],
            domains.keys(&DRUID_DOMAINS),
        ),
    );
    picks.selection(
        "druid/blight-druid",
        FeatureSelection {
            skipped_levels: BTreeSet::from([4]),
            ..sharing(
                BLIGHT_DOMAIN,
                "Domain (Blight Druid)",
                "druid#blight-druid".to_owned(),
                &["druid"],
                domains.keys(&BLIGHT_DOMAINS),
            )
        },
    );
    picks.selection(
        "druid/winter-child",
        sharing(
            WINTER_DOMAIN,
            "Domain (Winter Child)",
            "druid#winter-child".to_owned(),
            &["druid"],
            domains.keys(&["ice"]),
        ),
    );
    for table in [
        "druid",
        "druid/defender-of-the-true-world",
        "druid/feyspeaker",
    ] {
        cells(picks, table, &["animal-companion"], &[4], &held);
        cells(picks, table, &["domain-ability"], &[6], &held);
        cells(picks, table, &["lightning-lord"], &[8], &held);
    }
    let blight = Pick::Selection(FeatureSelectionKey::new(BLIGHT_DOMAIN));
    cells(
        picks,
        "druid/blight-druid",
        &["domain-abilities"],
        &[1],
        &blight,
    );
    cells(
        picks,
        "druid/blight-druid",
        &["domain-ability"],
        &[6, 8],
        &held,
    );
    // The Winter Child's two cells are the Ice Subdomain's two rows.
    let winter = Pick::Selection(FeatureSelectionKey::new(WINTER_DOMAIN));
    cells(
        picks,
        "druid/winter-child",
        &["domain-ability-icicle"],
        &[1],
        &winter,
    );
    cells(
        picks,
        "druid/winter-child",
        &["domain-ability-body-of-ice"],
        &[8],
        &held,
    );
    let ice = domains
        .selection
        .options
        .iter()
        .find(|o| o.key.as_str() == "cleric/domain/ice")
        .expect("Ice");
    assert_eq!(
        ice.levels.iter().map(|row| row.level).collect::<Vec<_>>(),
        [1, 8]
    );

    // Divine Hunter: "May select a domain" (its Domain cell at 1), levelled by hunter levels, every domain power by
    // hunter level (ticket 33's decision on 31-2: no skipped levels). The cell's feature stays defined, off the row:
    // Impossible Domain's "Domain" line names it.
    picks.selection(
        "hunter/divine-hunter",
        sharing(
            DIVINE_HUNTER_DOMAIN,
            "Domain (Divine Hunter)",
            "hunter#domain".to_owned(),
            &["hunter"],
            domains.all_keys(),
        ),
    );
    cells(
        picks,
        "hunter/divine-hunter",
        &["domain"],
        &[1],
        &Pick::SelectionKeepingFeature(FeatureSelectionKey::new(DIVINE_HUNTER_DOMAIN)),
    );

    // Domain Zealot's "Domain" is a domain held (Second Mystery's "Mystery" is stated where the mysteries are built).
    picks.prerequisite(
        DataAt::MythicAbility(MythicAbilityKey::new("domain-zealot")),
        "Domain",
        Prerequisite::Any(
            domains
                .all_keys()
                .into_iter()
                .map(Prerequisite::SelectionOption)
                .collect(),
        ),
    );
    // Boon Companion's and Mythical Beast's "Animal Companion" accepts the domains' companion, whose rows the classes'
    // "Animal domain only" companions are, and the Nature mystery's Animal Companion revelation.
    let companions = vec![
        Prerequisite::Feature {
            feature: FeatureKey::new("cleric/domain/animal/animal-companion"),
            count: 1,
        },
        Prerequisite::Feature {
            feature: FeatureKey::new("cleric/domain/scalykind/animal-companion"),
            count: 1,
        },
        Prerequisite::SelectionOption(SelectionOptionKey::new(
            "oracle/revelations/nature/animal-companion",
        )),
    ];
    picks.alternatives(
        DataAt::Feat(rules::FeatKey::new("boon-companion")),
        "Animal Companion",
        companions.clone(),
    );
    picks.alternatives(
        DataAt::MythicAbility(MythicAbilityKey::new("mythical-beast")),
        "Animal Companion",
        companions,
    );
}

/// The Separatist (ticket 35's decision on 33-6): "This second domain cannot be an alignment domain that conflicts
/// with the cleric's own alignment." Each alignment domain, picked through the Separatist's second-domain selection,
/// needs an alignment off the opposite side of its axis.
fn separatist_alignments(guide: &Guide, domains: &Family) -> FeatureSelection {
    use rules::Alignment::*;
    crate::follow_ups::says(
        guide,
        "cleric",
        "forbidden-rites",
        "This second domain cannot be an alignment domain that conflicts with the cleric's own alignment.",
    );
    let mut selection = domains.selection.clone();
    for (domain, alignments) in [
        (
            "chaos",
            [
                NeutralGood,
                ChaoticGood,
                TrueNeutral,
                ChaoticNeutral,
                NeutralEvil,
                ChaoticEvil,
            ],
        ),
        (
            "evil",
            [
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
                LawfulEvil,
                NeutralEvil,
                ChaoticEvil,
            ],
        ),
        (
            "good",
            [
                LawfulGood,
                NeutralGood,
                ChaoticGood,
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
            ],
        ),
        (
            "law",
            [
                LawfulGood,
                NeutralGood,
                LawfulNeutral,
                TrueNeutral,
                LawfulEvil,
                NeutralEvil,
            ],
        ),
    ] {
        let key = format!("{DOMAIN}/{domain}");
        let option = selection
            .options
            .iter_mut()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert_eq!(option.prerequisite, None, "{key}");
        let mut children: Vec<Prerequisite> = alignments.map(Prerequisite::Alignment).into();
        children.push(Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
            FeatureSelectionKey::new(SEPARATIST_DOMAIN),
        ))));
        children.sort();
        option.prerequisite = Some(Prerequisite::Any(children));
    }
    selection
}

/// The domain cells' features are made nowhere: no cleric, inquisitor or druid feature is a domain cell's or an
/// "Animal domain only" companion (the Sacred Huntsmaster's own companion at 1 aside), and each cleric table picks
/// a domain at 1. Changes nothing.
pub(super) fn check_domains(data: &GameData) {
    for class in data
        .classes
        .iter()
        .filter(|c| ["cleric", "inquisitor", "druid"].contains(&c.key.as_str()))
    {
        let features = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features));
        for feature in features {
            let name = norm(&feature.name);
            let companion = name == norm("Animal Companion")
                && feature.key.as_str() != "inquisitor/sacred-huntsmaster/animal-companion"
                && !feature.key.as_str().starts_with("cleric/domain/")
                && !feature.key.as_str().starts_with("druid/drovier/");
            assert!(
                !starts(&name, "Domain Abilit") && !companion,
                "{}: a domain cell's feature",
                feature.key
            );
        }
    }
    let cleric = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "cleric")
        .expect("the cleric");
    let domain = FeatureSelectionKey::new(DOMAIN);
    let base = cleric
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("row 1");
    assert_eq!(
        base.selections.iter().filter(|s| **s == domain).count(),
        2,
        "the cleric's two domains"
    );
    for archetype in &cleric.archetypes {
        let first = archetype
            .levels
            .iter()
            .find(|row| row.level == 1)
            .map_or(&base.selections, |row| &row.selections);
        assert!(first.contains(&domain), "{} picks no domain", archetype.key);
    }
}
