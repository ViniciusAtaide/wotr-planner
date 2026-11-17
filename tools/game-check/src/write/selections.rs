//! The selections target (tickets 306a, 306b): what a selection offers, brought to its blueprint's `m_AllFeatures`, and
//! whether its picks ignore prerequisites, to `IgnorePrerequisites`.
//!
//! A selection is written all or nothing: only when every member of the game's list is one entry of the selection's
//! kind (importer design, ticket 306 §0 decisions 2 and 3). A member with no entry, or with several, leaves the whole
//! selection unwritten and listed, so a family slice makes the missing entries first and runs the writer after.
//!
//! - **Feats and mythic abilities** (306a): the `feats` or `mythic_abilities` field is the game's list.
//! - **Options** (306b): the options the game's list holds are the selection's `options` where it owns them and its
//!   `shared_options` otherwise. Each option is defined once (`OptionNotDefinedOnce`), so ownership is planned across
//!   all the selections first (decision 4): an option stays with its owner while the owner's game list holds it, else
//!   it moves to the first selection, in key order, whose game list holds it, else it stays and is listed. `only`
//!   limits what is written, not what is resolved: a move is made only when both selections are in the set.

use std::collections::{BTreeMap, BTreeSet};

use rules::{BlueprintGuid, DataAt, FeatureSelection, FeatureSelectionKey, SelectionOptionKey};
use serde_json::Value as Json;

use super::span::Value;
use super::{Change, Unexpressed, entry_file};
use crate::areas::Ctx;
use crate::areas::options::game_list;
use crate::dump::flag;

/// The changes that bring each selection, in key order, to its blueprint's list, and the selections and options
/// left unwritten. `only` limits both to the selections it names.
pub(crate) fn changes(
    ctx: &Ctx,
    only: Option<&BTreeSet<String>>,
) -> (Vec<Change>, Vec<Unexpressed>) {
    let data = ctx.planner.data;
    let in_set = |key: &FeatureSelectionKey| only.is_none_or(|only| only.contains(key.as_str()));
    let mut changes = Vec::new();
    let mut unexpressed = Vec::new();
    let mut selections: Vec<&FeatureSelection> = data.selections.iter().collect();
    selections.sort_by(|a, b| a.key.cmp(&b.key));
    let owning = Ownership::plan(ctx, &selections, only);
    for selection in selections {
        if !selection.has_list() || !in_set(&selection.key) {
            continue;
        }
        let at = DataAt::FeatureSelection(selection.key.clone());
        let kind = if !selection.feats.is_empty() {
            Kind::Feats
        } else if selection.offers_mythic_abilities() {
            Kind::MythicAbilities
        } else {
            options_changes(ctx, selection, &owning, &mut changes, &mut unexpressed);
            continue;
        };
        let (json, game) = match game_of(ctx, selection) {
            Ok(found) => found,
            Err(what) => {
                unexpressed.push(Unexpressed { at, what });
                continue;
            }
        };
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
            changes.push(list_change(
                ctx,
                &at,
                kind.field(),
                Some(&keys),
                &planner,
                &keys,
            ));
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
    unexpressed.extend(owning.listed(&in_set));
    (changes, unexpressed)
}

/// A selection's blueprint JSON and the game's list, or why there is none. An empty list is no list of anything:
/// written, it would leave the selection with none.
fn game_of<'a>(
    ctx: &Ctx<'a>,
    selection: &FeatureSelection,
) -> Result<(&'a Json, BTreeSet<BlueprintGuid>), String> {
    let guid = selection
        .blueprint
        .as_ref()
        .ok_or("no blueprint".to_owned())?;
    let json = ctx
        .dump
        .json(guid)
        .ok_or("blueprint not in the dump".to_owned())?;
    match game_list(json) {
        None => Err("the game's blueprint has no list".to_owned()),
        Some(game) if game.is_empty() => Err("the game's list is empty".to_owned()),
        Some(game) => Ok((json, game)),
    }
}

/// A change of a list field from `old` to `new`; `value` `None` removes the field.
fn list_change(
    ctx: &Ctx,
    at: &DataAt,
    field: &'static str,
    value: Option<&BTreeSet<String>>,
    old: &BTreeSet<String>,
    new: &BTreeSet<String>,
) -> Change {
    let join = |keys: Vec<&String>| {
        keys.iter()
            .map(|key| key.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    Change {
        file: entry_file(ctx.planner.data, at),
        at: at.clone(),
        field,
        kind: "list",
        value: value.map(|keys| Value::List(keys.iter().map(|key| format!("\"{key}\"")).collect())),
        old: join(old.difference(new).collect()),
        new: join(new.difference(old).collect()),
    }
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
    problems(kind.noun(), &missing, &several, keys)
}

/// `keys` when nothing is missing or ambiguous, else the reason.
fn problems<T>(
    noun: &str,
    missing: &[String],
    several: &[String],
    keys: BTreeSet<T>,
) -> Result<BTreeSet<T>, String> {
    if missing.is_empty() && several.is_empty() {
        return Ok(keys);
    }
    let mut what = Vec::new();
    if !missing.is_empty() {
        what.push(format!("no {noun} entry: {}", missing.join("; ")));
    }
    if !several.is_empty() {
        what.push(format!("several {noun} entries: {}", several.join("; ")));
    }
    Err(what.join("; "))
}

/// The options of each selection of options the game's list holds, or why it can't be read.
type Resolved = BTreeMap<FeatureSelectionKey, Result<BTreeSet<SelectionOptionKey>, String>>;

/// The one entry of options for each blueprint of a selection's game list. A blueprint several options name resolves
/// to those the selection offers today; when it offers none, to the one of them some selection offers; else it is
/// ambiguous.
fn resolve_options(
    ctx: &Ctx,
    selection: &FeatureSelection,
    game: &BTreeSet<BlueprintGuid>,
) -> Result<BTreeSet<SelectionOptionKey>, String> {
    let data = ctx.planner.data;
    let mut keys = BTreeSet::new();
    let mut missing = Vec::new();
    let mut several = Vec::new();
    for member in game {
        let held: Vec<&SelectionOptionKey> = ctx
            .planner
            .entries_of(member)
            .into_iter()
            .filter_map(|at| match at {
                DataAt::SelectionOption(key) => Some(key),
                _ => None,
            })
            .collect();
        match held.as_slice() {
            [one] => {
                keys.insert((*one).clone());
            }
            [] => missing.push(format!("{} ({member})", ctx.name(member))),
            _ => {
                let offered: Vec<&SelectionOptionKey> = held
                    .iter()
                    .copied()
                    .filter(|key| selection.offers(key))
                    .collect();
                let offered_somewhere: Vec<&SelectionOptionKey> = held
                    .iter()
                    .copied()
                    .filter(|key| data.selections.iter().any(|s| s.offers(key)))
                    .collect();
                match (offered.as_slice(), offered_somewhere.as_slice()) {
                    ([], [one]) => {
                        keys.insert((*one).clone());
                    }
                    ([], _) => several.push(format!(
                        "{} ({})",
                        ctx.name(member),
                        held.iter()
                            .map(|k| k.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                    (offered, _) => keys.extend(offered.iter().map(|key| (*key).clone())),
                }
            }
        }
    }
    problems("option", &missing, &several, keys)
}

/// Who owns each option once every selection's list is written (decision 4), and what stays as it is.
struct Ownership {
    resolved: Resolved,
    /// The owner after the plan, for each option some resolved selection owns or holds.
    owner: BTreeMap<SelectionOptionKey, FeatureSelectionKey>,
    /// Options left where they are, and why: (owner, option, reason).
    left: Vec<(FeatureSelectionKey, SelectionOptionKey, String)>,
}

impl Ownership {
    fn plan(
        ctx: &Ctx,
        selections: &[&FeatureSelection],
        only: Option<&BTreeSet<String>>,
    ) -> Ownership {
        let in_set =
            |key: &FeatureSelectionKey| only.is_none_or(|only| only.contains(key.as_str()));
        let mut resolved = Resolved::new();
        for selection in selections {
            let options_kind = selection.feats.is_empty() && !selection.offers_mythic_abilities();
            if !selection.has_list() || !options_kind {
                continue;
            }
            let found = game_of(ctx, selection)
                .and_then(|(_, game)| resolve_options(ctx, selection, &game));
            resolved.insert(selection.key.clone(), found);
        }
        let mut current = BTreeMap::new();
        for selection in selections {
            for option in &selection.options {
                current.entry(option).or_insert(&selection.key);
            }
        }
        // The first selection, in key order, whose game list holds an option.
        let holder = |option: &SelectionOptionKey| {
            resolved
                .iter()
                .find(|(_, found)| found.as_ref().is_ok_and(|keys| keys.contains(option)))
                .map(|(key, _)| key.clone())
        };
        let mut every: BTreeSet<&SelectionOptionKey> = current.keys().copied().collect();
        for keys in resolved.values().flatten() {
            every.extend(keys);
        }
        let mut owner = BTreeMap::new();
        let mut left = Vec::new();
        for option in every {
            let now = current.get(option).copied();
            let wanted = match now {
                // An owner that is no selection of options, or whose list can't be read, keeps its option.
                Some(now) if !matches!(resolved.get(now), Some(Ok(_))) => Some(now.clone()),
                Some(now)
                    if resolved[now]
                        .as_ref()
                        .is_ok_and(|keys| keys.contains(option)) =>
                {
                    Some(now.clone())
                }
                now => match holder(option) {
                    Some(to) => Some(to),
                    None => {
                        if let Some(now) = now {
                            left.push((
                                now.clone(),
                                option.clone(),
                                "no game list holds it; it stays".to_owned(),
                            ));
                        }
                        now.cloned()
                    }
                },
            };
            let made = match (now, &wanted) {
                (Some(from), Some(to)) if from != to => {
                    if in_set(from) && in_set(to) {
                        Some(to.clone())
                    } else {
                        left.push((
                            from.clone(),
                            option.clone(),
                            format!("its move to {to} is not made: a selection is outside --only"),
                        ));
                        Some(from.clone())
                    }
                }
                (None, Some(to)) if !in_set(to) => None,
                _ => wanted,
            };
            if let Some(made) = made {
                owner.insert(option.clone(), made);
            }
        }
        Ownership {
            resolved,
            owner,
            left,
        }
    }

    /// The options left where they are, for the owners in the set.
    fn listed(&self, in_set: &impl Fn(&FeatureSelectionKey) -> bool) -> Vec<Unexpressed> {
        self.left
            .iter()
            .filter(|(owner, _, _)| in_set(owner))
            .map(|(owner, option, why)| Unexpressed {
                at: DataAt::SelectionOption(option.clone()),
                what: format!("{why} (in {owner})"),
            })
            .collect()
    }
}

/// A selection of options: its `options` are those the plan gives it and its `shared_options` the rest of the game's
/// list.
fn options_changes(
    ctx: &Ctx,
    selection: &FeatureSelection,
    owning: &Ownership,
    changes: &mut Vec<Change>,
    unexpressed: &mut Vec<Unexpressed>,
) {
    let at = DataAt::FeatureSelection(selection.key.clone());
    let game = match owning.resolved.get(&selection.key) {
        Some(Ok(game)) => game,
        Some(Err(what)) => {
            unexpressed.push(Unexpressed {
                at,
                what: what.clone(),
            });
            return;
        }
        None => return,
    };
    let owns = |option: &SelectionOptionKey| owning.owner.get(option) == Some(&selection.key);
    let own: BTreeSet<SelectionOptionKey> = selection
        .options
        .iter()
        .chain(game)
        .filter(|option| owns(option))
        .cloned()
        .collect();
    let shared: BTreeSet<SelectionOptionKey> = game
        .iter()
        .filter(|option| !own.contains(option))
        .cloned()
        .collect();
    for (field, old, new) in [
        ("options", &selection.options, own),
        ("shared_options", &selection.shared_options, shared),
    ] {
        if *old != new {
            let texts = |keys: &BTreeSet<SelectionOptionKey>| -> BTreeSet<String> {
                keys.iter().map(|key| key.as_str().to_owned()).collect()
            };
            let (old, new) = (texts(old), texts(&new));
            changes.push(list_change(
                ctx,
                &at,
                field,
                (!new.is_empty()).then_some(&new),
                &old,
                &new,
            ));
        }
    }
}
