//! Each entry's blueprint is in the dump, of a type its kind can be: a class a class (a prestige class a prestige
//! one), a mythic path a mythic class, an archetype one of its class's archetypes, a feat or feature a feature. A
//! feat's, an option's, a mythic ability's or a mythic feat's `ranks` is its blueprint's `Ranks` (ticket 303). An option
//! the planner takes as a selection (`is_selection`, or a pick that names a value) has a selection's or a parametrised
//! feature's blueprint (ticket 306ac).

use std::collections::BTreeMap;

use rules::{BlueprintGuid, DataAt, GameData};

use super::Ctx;
use crate::dump::{flag, int, is_feature_kind, references};
use crate::report::{Area, Out};

const AREA: Area = Area::Mapping;

pub fn compare(ctx: &Ctx, out: &mut Out) {
    let ranks = ranks(ctx.planner.data);
    for (at, guid) in &ctx.planner.entries {
        let Some(guid) = guid else {
            out.skip(AREA, format!("{} with no blueprint", at.kind()));
            continue;
        };
        let mark = out.mark();
        entry(ctx, out, at, guid, ranks.get(at).copied().unwrap_or(1));
        out.compared(AREA, mark);
    }
}

/// `ranks`: the entry's own, 1 for a kind without one.
fn entry(ctx: &Ctx, out: &mut Out, at: &DataAt, guid: &BlueprintGuid, ranks: u8) {
    let Some(blueprint) = ctx.dump.get(guid) else {
        out.finding(AREA, "not in the dump", at, guid, "-", "-", at.kind());
        return;
    };
    let name = blueprint.name.as_str();
    if !fits(at, &blueprint.kind) {
        out.finding(
            AREA,
            "a type its kind can't be",
            at,
            guid,
            name,
            &blueprint.kind,
            at.kind(),
        );
        return;
    }
    let json = ctx.dump.json(guid);
    match at {
        DataAt::Class(key) => {
            let Some(json) = json else { return };
            let game = flag(&json["PrestigeClass"]);
            let planner = ctx
                .planner
                .data
                .prestige_classes
                .iter()
                .any(|class| class.key == *key);
            if game != planner {
                out.finding(
                    AREA,
                    "prestige or not",
                    at,
                    guid,
                    name,
                    prestige(game),
                    prestige(planner),
                );
            }
            if flag(&json["IsMythic"]) {
                out.finding(
                    AREA,
                    "a mythic class as a class",
                    at,
                    guid,
                    name,
                    "mythic",
                    "class",
                );
            }
        }
        DataAt::MythicPath(_) => {
            if json.is_some_and(|json| !flag(&json["IsMythic"])) {
                out.finding(
                    AREA,
                    "a mythic path that isn't mythic",
                    at,
                    guid,
                    name,
                    "not mythic",
                    "mythic path",
                );
            }
        }
        DataAt::Archetype(key) => {
            let data = ctx.planner.data;
            let class = (data.archetypes.iter())
                .find(|archetype| archetype.key == *key)
                .and_then(|archetype| data.class(&archetype.class));
            let Some(class) = class else { return };
            let Some(class_guid) = ctx.planner.guid(DataAt::Class(class.key.clone())) else {
                return;
            };
            let Some(class_json) = ctx.dump.json(class_guid) else {
                return;
            };
            if !references(class_json.get("m_Archetypes")).contains(guid) {
                out.finding(
                    AREA,
                    "not among its class's archetypes",
                    at,
                    guid,
                    name,
                    format!("not in {}'s m_Archetypes", ctx.name(class_guid)),
                    format!("an archetype of {}", class.key),
                );
            }
        }
        DataAt::SelectionOption(key) => {
            let Some(json) = json else { return };
            ranks_differ(out, at, guid, name, json, ranks);
            if let Some(option) = ctx.planner.data.selection_option(key) {
                let game = is_selection_kind(&blueprint.kind);
                let planner = option_is_selection(option);
                if planner && !game {
                    out.finding(
                        AREA,
                        "selection or not",
                        at,
                        guid,
                        name,
                        selection_word(game, &blueprint.kind),
                        selection_word(planner, "a plain feature"),
                    );
                }
            }
        }
        DataAt::Feat(_) | DataAt::MythicAbility(_) | DataAt::MythicFeat(_) => {
            let Some(json) = json else { return };
            ranks_differ(out, at, guid, name, json, ranks);
        }
        _ => {}
    }
}

/// Each feat's, option's, mythic ability's and mythic feat's `ranks`: how often it can be held when it isn't a
/// selection.
fn ranks(data: &GameData) -> BTreeMap<DataAt, u8> {
    let feats = (data.feats.iter()).map(|feat| (DataAt::Feat(feat.key.clone()), feat.ranks));
    let options = (data.options.iter())
        .map(|option| (DataAt::SelectionOption(option.key.clone()), option.ranks));
    let abilities = (data.mythic_abilities.iter())
        .map(|ability| (DataAt::MythicAbility(ability.key.clone()), ability.ranks));
    let mythic_feats =
        (data.mythic_feats.iter()).map(|feat| (DataAt::MythicFeat(feat.key.clone()), feat.ranks));
    feats
        .chain(options)
        .chain(abilities)
        .chain(mythic_feats)
        .collect()
}

fn prestige(prestige: bool) -> &'static str {
    if prestige {
        "prestige class"
    } else {
        "base class"
    }
}

/// Whether a blueprint of type `kind` can be an entry of this kind.
fn fits(at: &DataAt, kind: &str) -> bool {
    let feature = is_feature_kind(kind);
    match at {
        DataAt::Race(_) => kind == "BlueprintRace",
        DataAt::Subrace(_) => feature || kind == "BlueprintRace",
        DataAt::Feat(_)
        | DataAt::Feature(_)
        | DataAt::FeatureSelection(_)
        | DataAt::SelectionOption(_)
        | DataAt::MythicAbility(_)
        | DataAt::MythicFeat(_)
        | DataAt::Background(_)
        | DataAt::Deity(_) => feature,
        DataAt::Class(_) | DataAt::MythicPath(_) => kind == "BlueprintCharacterClass",
        DataAt::Archetype(_) => kind == "BlueprintArchetype",
        DataAt::Weapon(_) => kind == "BlueprintWeaponType",
        DataAt::Spell(_) => kind == "BlueprintAbility",
        DataAt::SpellList(_) => kind == "BlueprintSpellList",
        DataAt::Gear(_) => kind.starts_with("BlueprintItem"),
        DataAt::Progression | DataAt::Skill(_) => false,
    }
}

/// The finding "ranks differ": the blueprint's `Ranks` against the entry's.
fn ranks_differ(
    out: &mut Out,
    at: &DataAt,
    guid: &BlueprintGuid,
    name: &str,
    json: &serde_json::Value,
    ranks: u8,
) {
    let game = json.get("Ranks").map_or(1, int);
    let planner = i64::from(ranks);
    if game != planner {
        out.finding(
            AREA,
            "ranks differ",
            at,
            guid,
            name,
            game.to_string(),
            planner.to_string(),
        );
    }
}

/// Whether a blueprint's type is one the game picks from: `BlueprintFeatureSelection` and its kin, or a
/// `BlueprintParametrizedFeature` (ticket 306ac).
fn is_selection_kind(kind: &str) -> bool {
    kind.contains("FeatureSelection") || kind == "BlueprintParametrizedFeature"
}

/// Whether the planner takes an option as a selection: the hand `is_selection`, a pick that names a value (a chosen
/// feat or a parameter). A game selection with no hand field is no difference: the engine derives it from the
/// blueprint (ticket 306ac).
fn option_is_selection(option: &rules::SelectionOption) -> bool {
    option.is_selection || option.takes_value()
}

fn selection_word(selection: bool, kind: &str) -> String {
    if selection {
        format!("a selection ({kind})")
    } else {
        format!("not a selection ({kind})")
    }
}
