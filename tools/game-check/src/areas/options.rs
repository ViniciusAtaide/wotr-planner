//! What a choice offers: each selection's options (its own and the ones it shares), feats (ticket 301) or mythic
//! abilities (ticket 302) against the blueprint's `m_AllFeatures`, and the planner's feats, mythic abilities and mythic
//! feats against the game's general pools (`BasicFeatSelection`, `MythicAbilitySelection`, `MythicFeatSelection`).

use std::collections::BTreeSet;

use rules::{BlueprintGuid, DataAt};

use super::Ctx;
use super::fixed::{BASIC_FEAT_SELECTION, MYTHIC_ABILITY_SELECTION, MYTHIC_FEAT_SELECTION, guid};
use crate::dump::{flag, references};
use crate::report::{Area, Out};

const AREA: Area = Area::OptionLists;

pub fn compare(ctx: &Ctx, out: &mut Out) {
    let data = ctx.planner.data;
    for selection in &data.selections {
        let at = DataAt::FeatureSelection(selection.key.clone());
        // A filtered pool (`filtered_by`, ticket 243) offers all its options, as the game's one selection does
        // before its options' prerequisites filter it (an oracle's revelations).
        let offered = (selection.offered().into_iter())
            .map(|key| DataAt::SelectionOption(key.clone()))
            .chain(
                selection
                    .feats
                    .iter()
                    .map(|feat| DataAt::Feat(feat.clone())),
            )
            .chain(
                (selection.mythic_abilities.iter())
                    .map(|ability| DataAt::MythicAbility(ability.clone())),
            );
        let planner = guids(ctx, out, offered);
        // Whether a pick ignores prerequisites is a selection of feats' own (a bonus list's).
        let ignores = data
            .offers_feats(selection)
            .then_some(selection.ignores_prerequisites);
        list(
            ctx,
            out,
            &at,
            selection.blueprint.as_ref(),
            planner,
            ignores,
        );
    }
    let pools = [
        (
            "general feats",
            BASIC_FEAT_SELECTION,
            guids(
                ctx,
                out,
                data.feats.iter().map(|feat| DataAt::Feat(feat.key.clone())),
            ),
        ),
        (
            "mythic abilities",
            MYTHIC_ABILITY_SELECTION,
            guids(
                ctx,
                out,
                data.mythic_abilities
                    .iter()
                    .map(|each| DataAt::MythicAbility(each.key.clone())),
            ),
        ),
        (
            "mythic feats",
            MYTHIC_FEAT_SELECTION,
            guids(
                ctx,
                out,
                data.mythic_feats
                    .iter()
                    .map(|each| DataAt::MythicFeat(each.key.clone())),
            ),
        ),
    ];
    for (name, pool, planner) in pools {
        list(
            ctx,
            out,
            &format!("pool {name}"),
            Some(&guid(pool)),
            planner,
            None,
        );
    }
}

/// The blueprints of the planner's entries, by GUID with their keys; an entry with none is left out.
fn guids(
    ctx: &Ctx,
    out: &mut Out,
    entries: impl Iterator<Item = DataAt>,
) -> BTreeSet<(BlueprintGuid, String)> {
    entries
        .filter_map(|at| match ctx.planner.guid(at.clone()) {
            Some(guid) => Some((guid.clone(), at.key().unwrap_or("").to_owned())),
            None => {
                out.skip(AREA, "an option with no blueprint");
                None
            }
        })
        .collect()
}

fn list(
    ctx: &Ctx,
    out: &mut Out,
    entry: &dyn std::fmt::Display,
    guid: Option<&BlueprintGuid>,
    planner: BTreeSet<(BlueprintGuid, String)>,
    ignores_prerequisites: Option<bool>,
) {
    let Some((guid, json)) = ctx.blueprint(out, AREA, guid, "a choice") else {
        return;
    };
    let name = ctx.name(guid);
    let mark = out.mark();
    let Some(game) = game_list(json) else {
        // The planner offers a choice where the game grants one blueprint (Greater Wyrmshifter's breaths).
        out.finding(
            AREA,
            "a choice the game doesn't offer",
            entry,
            guid,
            &name,
            ctx.dump.kind(guid).unwrap_or("-"),
            format!("options with a blueprint: {}", planner.len()),
        );
        out.compared(AREA, mark);
        return;
    };
    let planner_guids: BTreeSet<&BlueprintGuid> = planner.iter().map(|(guid, _)| guid).collect();
    for option in &game {
        if !planner_guids.contains(option) {
            out.finding(
                AREA,
                "in game only",
                entry,
                guid,
                &name,
                ctx.name(option),
                "-",
            );
        }
    }
    for (option, key) in &planner {
        if !game.contains(option) {
            out.finding(
                AREA,
                "in planner only",
                entry,
                guid,
                &name,
                "-",
                format!("{key} ({})", ctx.name(option)),
            );
        }
    }
    if let Some(planner) = ignores_prerequisites {
        let game = flag(&json["IgnorePrerequisites"]);
        if game != planner {
            out.finding(
                AREA,
                "ignores prerequisites or not",
                entry,
                guid,
                &name,
                game.to_string(),
                planner.to_string(),
            );
        }
    }
    out.compared(AREA, mark);
}

/// The blueprints a choice's `m_AllFeatures` lists; `None` when it has none, a choice the game doesn't offer.
pub(crate) fn game_list(json: &serde_json::Value) -> Option<BTreeSet<BlueprintGuid>> {
    let all = json.get("m_AllFeatures")?;
    Some(references(Some(all)).into_iter().collect())
}
