//! The selections target (ticket 306a): the feats or mythic abilities a selection offers, and whether its picks ignore
//! prerequisites, brought to the blueprint's `m_AllFeatures` and `IgnorePrerequisites`.
//!
//! A selection is written all or nothing: only when every member of the game's list is one entry of the selection's
//! kind (importer design, ticket 306 §0 decisions 2 and 3). A member with no entry, or with several, leaves the whole
//! selection unwritten and listed, so a family slice makes the missing entries first and runs the writer after.
//! Selections of options are the next slice's (306b).

use std::collections::BTreeSet;

use rules::{BlueprintGuid, DataAt, FeatureSelection};

use super::span::Value;
use super::{Change, Unexpressed, entry_file};
use crate::areas::Ctx;
use crate::areas::options::game_list;
use crate::dump::flag;
use crate::report::{Area, Out};

/// The changes that bring each selection of feats or of mythic abilities, in key order, to its blueprint's list, and
/// the selections a member with no one entry leaves unwritten. `only` limits both to the selections it names.
pub(crate) fn changes(
    ctx: &Ctx,
    only: Option<&BTreeSet<String>>,
) -> (Vec<Change>, Vec<Unexpressed>) {
    let data = ctx.planner.data;
    let mut changes = Vec::new();
    let mut unexpressed = Vec::new();
    let mut selections: Vec<&FeatureSelection> = data.selections.iter().collect();
    selections.sort_by(|a, b| a.key.cmp(&b.key));
    for selection in selections {
        if !selection.has_list() || only.is_some_and(|only| !only.contains(selection.key.as_str()))
        {
            continue;
        }
        let kind = if !selection.feats.is_empty() {
            Kind::Feats
        } else if selection.offers_mythic_abilities() {
            Kind::MythicAbilities
        } else {
            continue;
        };
        let at = DataAt::FeatureSelection(selection.key.clone());
        // The check's skips are its own business; the target only needs what the game gives.
        let mut skipped = Out::default();
        let Some((_, json)) = ctx.blueprint(
            &mut skipped,
            Area::OptionLists,
            selection.blueprint.as_ref(),
            "a choice",
        ) else {
            continue;
        };
        let Some(game) = game_list(json) else {
            unexpressed.push(Unexpressed {
                at,
                what: "the game's blueprint has no list".to_owned(),
            });
            continue;
        };
        // A list the game leaves empty is no selection of .noun()s: writing it would leave the selection with no list.
        if game.is_empty() {
            unexpressed.push(Unexpressed {
                at,
                what: "the game's list is empty".to_owned(),
            });
            continue;
        }
        let keys = match resolve(ctx, kind, &game) {
            Ok(keys) => keys,
            Err(what) => {
                unexpressed.push(Unexpressed { at, what });
                continue;
            }
        };
        let planner: BTreeSet<String> = match kind {
            Kind::Feats => selection
                .feats
                .iter()
                .map(|k| k.as_str().to_owned())
                .collect(),
            Kind::MythicAbilities => selection
                .mythic_abilities
                .iter()
                .map(|k| k.as_str().to_owned())
                .collect(),
        };
        if keys != planner {
            changes.push(Change {
                file: entry_file(data, &at),
                at: at.clone(),
                field: kind.field(),
                kind: "list",
                value: Some(Value::List(
                    keys.iter().map(|key| format!("\"{key}\"")).collect(),
                )),
                old: planner
                    .difference(&keys)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                new: keys
                    .difference(&planner)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
        // Whether a pick ignores prerequisites is a selection of feats' own (a bonus list's).
        let ignores = flag(&json["IgnorePrerequisites"]);
        if data.offers_feats(selection) && ignores != selection.ignores_prerequisites {
            changes.push(Change {
                file: entry_file(data, &at),
                at,
                field: "ignores_prerequisites",
                kind: "list",
                value: Some(Value::One(ignores.to_string())),
                old: selection.ignores_prerequisites.to_string(),
                new: ignores.to_string(),
            });
        }
    }
    (changes, unexpressed)
}

/// What a selection's list holds.
#[derive(Clone, Copy)]
enum Kind {
    Feats,
    MythicAbilities,
}

impl Kind {
    fn field(self) -> &'static str {
        match self {
            Kind::Feats => "feats",
            Kind::MythicAbilities => "mythic_abilities",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Kind::Feats => "feat",
            Kind::MythicAbilities => "mythic ability",
        }
    }

    /// The key of an entry of this kind.
    fn key(self, at: &DataAt) -> Option<&str> {
        match (self, at) {
            (Kind::Feats, DataAt::Feat(_)) | (Kind::MythicAbilities, DataAt::MythicAbility(_)) => {
                at.key()
            }
            _ => None,
        }
    }
}

/// The key of the one entry of `kind` for each blueprint of the game's list; or what stops the selection: the members
/// with no entry of that kind and those with several.
fn resolve(
    ctx: &Ctx,
    kind: Kind,
    game: &BTreeSet<BlueprintGuid>,
) -> Result<BTreeSet<String>, String> {
    let mut keys = BTreeSet::new();
    let mut missing = Vec::new();
    let mut several = Vec::new();
    for member in game {
        let held: Vec<&str> = ctx
            .planner
            .entries_of(member)
            .into_iter()
            .filter_map(|at| kind.key(at))
            .collect();
        match held.as_slice() {
            [one] => {
                keys.insert((*one).to_owned());
            }
            [] => missing.push(format!("{} ({member})", ctx.name(member))),
            _ => several.push(format!("{} ({})", ctx.name(member), held.join(", "))),
        }
    }
    if missing.is_empty() && several.is_empty() {
        return Ok(keys);
    }
    let mut what = Vec::new();
    if !missing.is_empty() {
        what.push(format!("no {} entry: {}", kind.noun(), missing.join("; ")));
    }
    if !several.is_empty() {
        what.push(format!(
            "several {} entries: {}",
            kind.noun(),
            several.join("; ")
        ));
    }
    Err(what.join("; "))
}
