//! Each entry's blueprint is in the dump, of a type its kind can be: a class a class (a prestige class a prestige
//! one), a mythic path a mythic class, an archetype one of its class's archetypes, a feat or feature a feature.

use rules::{BlueprintGuid, DataAt};

use super::Ctx;
use crate::dump::{flag, is_feature_kind, references};
use crate::report::{Area, Out};

const AREA: Area = Area::Mapping;

pub fn compare(ctx: &Ctx, out: &mut Out) {
    for (at, guid) in &ctx.planner.entries {
        let Some(guid) = guid else {
            out.skip(AREA, format!("{} with no blueprint", at.kind()));
            continue;
        };
        let mark = out.mark();
        entry(ctx, out, at, guid);
        out.compared(AREA, mark);
    }
}

fn entry(ctx: &Ctx, out: &mut Out, at: &DataAt, guid: &BlueprintGuid) {
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
            let class = ctx.planner.data.all_classes().find(|class| {
                class
                    .archetypes
                    .iter()
                    .any(|archetype| archetype.key == *key)
            });
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
        _ => {}
    }
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
        DataAt::FeatList(_) => kind == "BlueprintFeatureSelection",
        DataAt::Class(_) | DataAt::MythicPath(_) => kind == "BlueprintCharacterClass",
        DataAt::Archetype(_) => kind == "BlueprintArchetype",
        DataAt::Weapon(_) => kind == "BlueprintWeaponType",
        DataAt::Spell(_) => kind == "BlueprintAbility",
        DataAt::SpellList(_) => kind == "BlueprintSpellList",
        DataAt::Gear(_) => kind.starts_with("BlueprintItem"),
        DataAt::Progression | DataAt::Skill(_) => false,
    }
}
