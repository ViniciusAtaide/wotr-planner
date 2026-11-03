//! The Living Deity's mystery, revelations and domains (the Magic Deceiver's selections, granted by its way's rows),
//! and Impossible Domain.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    DataAt, FeatureKey, FeatureSelection, FeatureSelectionKey, LevelFrom, MythicAbilityKey,
    Prerequisite, SelectionOptionKey,
};

use crate::guide::Guide;
use crate::names::norm;
use crate::option_levels::selection;
use crate::picks::Picks;

use super::domain_selections::PENTAMIC_DISCIPLINE;
use super::{Family, sharing};

/// The Living Deity (a way of the Magic Deceiver): "She gains an Oracle's Mystery and Curse class features",
/// "receives Revelations appropriate to the Mystery at 1st, 4th, 12th and 20th levels along with the Final
/// Revelation", and "At 20th level, a Living Deity may select two domains, gaining any abilities granted by these
/// domains" (Hollow Spark). Its mystery, revelations and domains are selections of the Magic Deceiver's, levelled
/// by arcanist levels; its way's rows grant them ([`living_deity_rows`]).
pub(super) fn living_deity(domains: &Family, mysteries: &Family, picks: &mut Picks) {
    let mystery = format!("{DECEIVER}/mystery");
    let selections = [
        sharing(
            &mystery,
            "Mystery (Living Deity)",
            "arcanist#mysterious-gift-mystery".to_owned(),
            &["arcanist"],
            mysteries.all_keys(),
        ),
        FeatureSelection {
            pool_of: Some(FeatureSelectionKey::new(&mystery)),
            ..selection(
                &format!("{DECEIVER}/revelation"),
                "Revelation (Living Deity)",
                "arcanist#mysterious-gift-mystery".to_owned(),
                &["arcanist"],
            )
        },
        sharing(
            &format!("{DECEIVER}/domain"),
            "Domain (Living Deity)",
            "arcanist#hollow-spark".to_owned(),
            &["arcanist"],
            domains.all_keys(),
        ),
    ];
    for selection in selections {
        picks.unnamed(selection.key.as_str());
        picks.selection(DECEIVER, selection);
    }
}

const DECEIVER: &str = "arcanist/magic-deceiver";

/// Impossible Domain (tickets 33 and 35): "You may select any domain" (the guide), levelled by the class that gives
/// the character's other domains (the user's fact), and for a Living Deity by arcanist levels from the pick (the
/// user's decision); a Living Deity, whose way gives domains, and a Hellknight with Pentamic Faith's domain ("She uses
/// her class level to determine which domain abilities she can use") may take it. A selection of every domain,
/// picked through the mythic ability, whose "Prerequisites: Domain" line also accepts the Living Deity's way and the
/// Pentamic Faith discipline. Built here, where the domains and the Living Deity's way are: the line is resolved once
/// both exist.
pub(super) fn impossible_domain(guide: &Guide, domains: &Family, picks: &mut Picks) {
    crate::follow_ups::says(
        guide,
        "mythic-paths",
        "impossible-domain",
        "You may select any domain.",
    );
    crate::follow_ups::says(
        guide,
        "prestige-classes",
        "pentamic-faith",
        "She uses her class level to determine which domain abilities she can use.",
    );
    let living_deity =
        SelectionOptionKey::new("arcanist/magic-deceiver/ways-of-the-deceiver/living-deity");
    let key = "cleric/impossible-domain";
    picks.selection(
        "cleric",
        FeatureSelection {
            shared_options: domains.all_keys(),
            level_from: Some(LevelFrom::OtherOptions),
            level_also_from: Some(living_deity.clone()),
            ..selection(
                key,
                "Impossible Domain",
                "mythic-paths#impossible-domain".into(),
                &[],
            )
        },
    );
    picks.unnamed(key);
    picks.mythic_selection("impossible-domain", FeatureSelectionKey::new(key));
    picks.alternatives(
        DataAt::MythicAbility(MythicAbilityKey::new("impossible-domain")),
        "Domain",
        vec![
            Prerequisite::SelectionOption(living_deity),
            Prerequisite::SelectionOption(SelectionOptionKey::new(PENTAMIC_DISCIPLINE)),
        ],
    );
}

/// The Living Deity's way's rows, as [`living_deity`] says: its Mystery, Curse, Revelation and Hollow Spark cells
/// grant the selections, the Final Revelation is the mystery's row, and their features go.
pub(crate) fn living_deity_rows(ways: &mut crate::option_levels::Family) {
    let mystery = FeatureSelectionKey::new(format!("{DECEIVER}/mystery"));
    let revelation = FeatureSelectionKey::new(format!("{DECEIVER}/revelation"));
    let domain = FeatureSelectionKey::new(format!("{DECEIVER}/domain"));
    let curse = FeatureSelectionKey::new("oracle/oracles-curse");
    let names: BTreeMap<FeatureKey, String> = ways
        .features
        .iter()
        .map(|feature| (feature.key.clone(), norm(&feature.name)))
        .collect();
    let deity = ways
        .selection
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity")
        .expect("the Living Deity");
    let mut revelations = 0;
    for row in &mut deity.levels {
        let name = |key: &FeatureKey| names.get(key).cloned().unwrap_or_default();
        let mut add = Vec::new();
        row.features.retain(|feature| {
            let n = name(feature);
            if n == norm("Mysterious Gift - Mystery") {
                add.push(mystery.clone());
            } else if n == norm("Mysterious Gift - Curse") {
                add.push(curse.clone());
            } else if n == norm("Revelation") {
                add.push(revelation.clone());
                revelations += 1;
            } else if n == norm("Hollow Spark") {
                add.push(domain.clone());
                add.push(domain.clone());
            } else if n != norm("Final Revelation") {
                return true;
            }
            false
        });
        row.selections.extend(add);
    }
    assert_eq!(
        revelations, 4,
        "the Living Deity's revelations at 1, 4, 12 and 20"
    );
    let named: BTreeSet<FeatureKey> = ways
        .selection
        .options
        .iter()
        .flat_map(|option| &option.levels)
        .flat_map(|row| row.features.iter().cloned())
        .collect();
    ways.features.retain(|feature| named.contains(&feature.key));
}
