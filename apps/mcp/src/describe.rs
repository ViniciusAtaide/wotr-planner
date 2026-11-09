//! `describe`: one item, with what its kind has: a class's or archetype's rows by level, a Mythic path's rows by
//! rank, the Feature selections it defines.

use rules::{
    Archetype, Class, Engine, FeatKey, FeatSlot, FeatValue, FeatureKey, FeatureSelection,
    FeatureSelectionKey, Save, SelectionOptionKey, effective_row,
};

use crate::planner::Planner;
use crate::shapes::{
    Hit, Item, Kind, LevelRow, Named, RankRow, Selection, SpellLevels, Summarised,
};

/// The item of a kind with a key, or why there's none: the closest matches, so the agent can retry.
pub(crate) fn describe(planner: &Planner, kind: Kind, key: &str) -> Result<Item, String> {
    let Some(hit) = planner.index.get(kind, key) else {
        return Err(not_found(planner, kind, key));
    };
    let mut item = Item {
        hit: hit.clone(),
        taken_for: None,
        levels: None,
        ranks: None,
        archetypes: None,
        selections: None,
        features: None,
        feats: None,
        subraces: None,
        alignments: None,
        domains: None,
    };
    let engine = &planner.engine;
    let data = &planner.data;
    match kind {
        Kind::Class => {
            let (class, prestige) = class(planner, key).expect("an indexed class");
            item.levels = Some(class_rows(planner, class, prestige, None));
            item.archetypes = Some(
                class
                    .archetypes
                    .iter()
                    .map(|archetype| named(&archetype.key, &archetype.name))
                    .collect(),
            );
            item.selections = Some(selections(planner, &class.selections));
        }
        Kind::Archetype => {
            let (class, prestige, archetype) = data
                .classes
                .iter()
                .map(|class| (class, false))
                .chain(data.prestige_classes.iter().map(|class| (class, true)))
                .find_map(|(class, prestige)| {
                    let archetype = class
                        .archetypes
                        .iter()
                        .find(|archetype| archetype.key.as_str() == key)?;
                    Some((class, prestige, archetype))
                })
                .expect("an indexed archetype");
            item.levels = Some(class_rows(planner, class, prestige, Some(archetype)));
            item.selections = Some(selections(planner, &archetype.selections));
        }
        Kind::Path => {
            let path = data
                .mythic_paths
                .iter()
                .find(|path| path.key.as_str() == key)
                .expect("an indexed path");
            item.ranks = Some(
                path.ranks
                    .iter()
                    .map(|row| RankRow {
                        rank: row.rank,
                        features: features(
                            planner,
                            engine.path_rank_features(&path.key, row.rank).into_iter(),
                        ),
                        feat_slots: feat_slots(engine, &row.feat_slots),
                        selections: selection_names(engine, &row.selections),
                    })
                    .collect(),
            );
            item.selections = Some(selections(planner, &path.selections));
        }
        Kind::Feat => {
            let feat = data
                .feats
                .iter()
                .find(|feat| feat.key.as_str() == key)
                .expect("an indexed feat");
            item.taken_for = feat.parameter.map(|kind| kind.noun().to_owned());
        }
        Kind::MythicAbility => {
            let ability = data
                .mythic_abilities
                .iter()
                .find(|ability| ability.key.as_str() == key)
                .expect("an indexed mythic ability");
            item.features = Some(features(planner, ability.features.iter()));
            let granted: Vec<&FeatureSelection> = ability
                .selections
                .iter()
                .filter_map(|key| planner.selections().find(|selection| &selection.key == key))
                .collect();
            item.selections = Some(granted.into_iter().map(|s| selection(planner, s)).collect());
        }
        Kind::MythicFeat => {
            let feat = data
                .mythic_feats
                .iter()
                .find(|feat| feat.key.as_str() == key)
                .expect("an indexed mythic feat");
            item.taken_for = feat.parameter.map(|kind| kind.noun().to_owned());
        }
        Kind::SelectionOption => {
            let option = planner
                .selections()
                .flat_map(|selection| &selection.options)
                .find(|option| option.key.as_str() == key)
                .expect("an indexed option");
            item.taken_for = option.parameter.map(|kind| kind.noun().to_owned());
            if !option.levels.is_empty() {
                item.levels = Some(
                    option
                        .levels
                        .iter()
                        .map(|row| LevelRow {
                            level: row.level,
                            bab: None,
                            fort: None,
                            reflex: None,
                            will: None,
                            spell_levels: None,
                            advances: Vec::new(),
                            features: features(planner, row.features.iter()),
                            feats: feats(engine, row.feats.iter(), [].iter()),
                            feat_slots: feat_slots(engine, &row.feat_slots),
                            selections: selection_names(engine, &row.selections),
                        })
                        .collect(),
                );
            }
        }
        Kind::Race => {
            let race = data
                .races
                .iter()
                .find(|race| race.key.as_str() == key)
                .expect("an indexed race");
            item.subraces = Some(
                race.subraces
                    .iter()
                    .map(|subrace| named(&subrace.key, &subrace.name))
                    .collect(),
            );
        }
        Kind::Deity => {
            let deity = data
                .deities
                .iter()
                .find(|deity| deity.key.as_str() == key)
                .expect("an indexed deity");
            item.alignments = Some(
                deity
                    .alignments
                    .iter()
                    .map(|alignment| alignment.name().to_owned())
                    .collect(),
            );
            item.domains = Some(
                deity
                    .domains
                    .iter()
                    .map(|domain| named(domain, engine.domain_name(domain)))
                    .collect(),
            );
        }
        Kind::Background => {
            let background = data
                .backgrounds
                .iter()
                .find(|background| background.key.as_str() == key)
                .expect("an indexed background");
            item.feats = Some(feats(
                engine,
                background.feats.iter(),
                background.feats_for.iter(),
            ));
        }
    }
    Ok(item)
}

/// "No archetype has the key `x`", with the closest matches.
fn not_found(planner: &Planner, kind: Kind, key: &str) -> String {
    let kind_name = serde_json::to_value(kind).expect("a kind serialises");
    let kind_name = kind_name.as_str().unwrap_or_default();
    let mut message = format!("No {kind_name} has the key `{key}`.");
    let query = key.replace(['/', '-'], " ");
    let search = |query: &str| {
        planner
            .index
            .search(query, None, None, Some(5))
            .map(|found| found.matches)
            .unwrap_or_default()
            .into_iter()
            // By name, key or initials: a summary that holds a word of the key isn't close to it.
            .filter(|hit| hit.matched.is_none())
            .collect::<Vec<_>>()
    };
    let mut closest: Vec<Hit> = search(key);
    if closest.is_empty() {
        closest = search(&query);
    }
    if !closest.is_empty() {
        let closest: Vec<String> = closest
            .iter()
            .map(|hit| {
                let kind = serde_json::to_value(hit.kind).expect("a kind serialises");
                format!(
                    "{} `{}` ({})",
                    kind.as_str().unwrap_or_default(),
                    hit.key,
                    hit.name
                )
            })
            .collect();
        message.push_str(&format!(" Closest: {}.", closest.join(", ")));
    }
    message.push_str(" `search` finds keys by name.");
    message
}

fn class<'a>(planner: &'a Planner, key: &str) -> Option<(&'a Class, bool)> {
    let data = &planner.data;
    data.classes
        .iter()
        .map(|class| (class, false))
        .chain(data.prestige_classes.iter().map(|class| (class, true)))
        .find(|(class, _)| class.key.as_str() == key)
}

/// A class's rows, taken with an archetype or none: [`effective_row`] at each level of its table, with the BAB
/// and saves its progressions give there.
fn class_rows(
    planner: &Planner,
    class: &Class,
    prestige: bool,
    archetype: Option<&Archetype>,
) -> Vec<LevelRow> {
    class
        .levels
        .iter()
        .filter_map(|base| effective_row(class, archetype, base.level))
        .map(|row| {
            let engine = &planner.engine;
            let level = row.level;
            let save = |save| Some(class.save_bonus(save, level, prestige, archetype));
            let spells = row.spell_levels;
            LevelRow {
                level,
                bab: Some(class.base_attack_bonus(level)),
                fort: save(Save::Fortitude),
                reflex: save(Save::Reflex),
                will: save(Save::Will),
                spell_levels: (spells.arcane > 0 || spells.divine > 0).then_some(SpellLevels {
                    arcane: spells.arcane,
                    divine: spells.divine,
                }),
                advances: [
                    (row.advances_arcane, "arcane"),
                    (row.advances_divine, "divine"),
                ]
                .into_iter()
                .filter(|(advances, _)| *advances)
                .map(|(_, kind)| kind.to_owned())
                .collect(),
                features: features(planner, row.features.iter()),
                feats: feats(engine, row.feats.iter(), row.feats_for.iter()),
                feat_slots: feat_slots(engine, row.feat_slots),
                selections: selection_names(engine, row.selections),
            }
        })
        .collect()
}

fn named(key: &impl std::fmt::Display, name: &str) -> Named {
    Named {
        key: key.to_string(),
        name: name.to_owned(),
    }
}

/// Features by key and name, with their summaries.
fn features<'a>(planner: &Planner, keys: impl Iterator<Item = &'a FeatureKey>) -> Vec<Summarised> {
    keys.map(|key| Summarised {
        key: key.to_string(),
        name: planner.engine.feature_name(key).to_owned(),
        summary: planner.feature_summary(key),
    })
    .collect()
}

/// Granted feats, then the ones granted for a value ("Weapon Focus (Longsword)", keyed by the feat).
fn feats<'a>(
    engine: &Engine,
    feats: impl Iterator<Item = &'a FeatKey>,
    feats_for: impl Iterator<Item = &'a FeatValue>,
) -> Vec<Named> {
    feats
        .map(|key| named(key, engine.feat_name(key)))
        .chain(feats_for.map(|granted| {
            named(
                &granted.feat,
                &format!(
                    "{} ({})",
                    engine.feat_name(&granted.feat),
                    engine.parameter_name(&granted.value)
                ),
            )
        }))
        .collect()
}

fn feat_slots(engine: &Engine, slots: &[FeatSlot]) -> Vec<String> {
    slots
        .iter()
        .map(|slot| engine.feat_slot_label(slot))
        .collect()
}

fn selection_names(engine: &Engine, keys: &[FeatureSelectionKey]) -> Vec<Named> {
    keys.iter()
        .map(|key| named(key, engine.selection_name(key)))
        .collect()
}

fn selections(planner: &Planner, list: &[FeatureSelection]) -> Vec<Selection> {
    list.iter()
        .map(|selection| self::selection(planner, selection))
        .collect()
}

/// A Feature selection with its own options, then the ones it shares from elsewhere, each with its summary.
fn selection(planner: &Planner, selection: &FeatureSelection) -> Selection {
    let option = |key: &SelectionOptionKey, name: &str| Summarised {
        key: key.to_string(),
        name: name.to_owned(),
        summary: planner.summary(Kind::SelectionOption, key.as_str()),
    };
    Selection {
        key: selection.key.as_str().to_owned(),
        name: selection.name.clone(),
        summary: (!selection.summary.is_empty()).then(|| selection.summary.clone()),
        options: selection
            .options
            .iter()
            .map(|own| option(&own.key, &own.name))
            .chain(
                selection
                    .shared_options
                    .iter()
                    .map(|key| option(key, planner.engine.option_name(key))),
            )
            .collect(),
    }
}
