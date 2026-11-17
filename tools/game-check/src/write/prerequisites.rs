//! The prerequisites target (ticket 308a, the importer's stage S4): each entry's `Requires` components from its
//! blueprint's `Prerequisite*` components, one `Requires` a component, its `group` the component's `Group` (`All`, and
//! anything else `Any`, as `BlueprintFeature.MeetsPrerequisites` counts it). The table of what each component writes
//! is `research/308/slices.md` §1.1.
//!
//! The entries: feats, selection options, mythic abilities and feats, classes (prestige too) and archetypes, the ones
//! the prerequisites area compares. Its decisions (`research/308/slices.md` §0):
//! - **All or nothing per entry.** An entry with any component the writer has no form for (a [`Gap`]) keeps its file
//!   and is listed, one line a gap, with the ticket that owns it. An entry with a hand `prerequisite` is never written.
//! - **Equality is the compiled tree.** The entry's `Requires` now and the game's are each compiled
//!   ([`rules::compile_requires`]) and canonical; equal, the file is untouched, whatever its `Requires` say in form.
//! - **A blueprint several entries share** is held once, by whichever entry holds it: a `Has` of it is an `Any` of
//!   each entry's fact, a `HasNot` a `Not` of that `Any`.
//! - **A rewrite never narrows unasked** (the fix round): one that drops a fact the old tree accepts, or refuses a
//!   fact the old tree didn't, leaves the entry whole, listed `narrows: … -> <ticket>` with the fact's owner (308
//!   when [`owner`] names none), unless `prerequisite-narrowing.txt` names it with the reason it moves toward the
//!   game ([`Narrowings`]). The check compares GUIDs, so it can't see a GUID turned into the wrong entry; this guard
//!   does. A line of that file no rewrite makes fails the check ([`stale`], 308g).
//!
//! The written components are the `Requires` in the blueprint's component order, then the entry's other components in
//! theirs.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Ability, Alignment, ArchetypeKey, BlueprintGuid, ClassKey, Component, Condition, DataAt, Fact,
    FeatKey, GameData, Group, Parameter, Prerequisite, SpellSchool, SpellType, StatRef, WeaponKey,
    compile_requires,
};
use serde_json::Value;

use super::carriers::{fact_at, facts_of, words};
use super::span::{Value as Field, ron_text};
use super::{Change, Unexpressed};
use crate::areas::fixed::{ANIMAL_CLASS, guid};
use crate::areas::prerequisites::{
    LOWEST_SCORE, alignment_mask, armour_groups, holds_for_every_build, is_own_no_feature,
};
use crate::areas::{Ctx, game_skill};
use crate::dump::{component_kind, components, flag, int, reference, references};

/// The finding kind the target answers: the log's `prerequisite` row (308b).
pub const KIND: &str = "requires";

/// A component the writer has no form for: what it is, naming the blueprint, and the ticket that owns it.
#[derive(Debug)]
struct Gap {
    what: String,
    ticket: &'static str,
}

impl Gap {
    fn new(what: impl Into<String>, ticket: &'static str) -> Gap {
        Gap {
            what: what.into(),
            ticket,
        }
    }
}

/// One entry the target reads.
struct Entry<'a> {
    at: DataAt,
    blueprint: Option<&'a BlueprintGuid>,
    prerequisite: Option<&'a Prerequisite>,
    components: &'a [Component],
    /// Its header's `ranks` (1 for a class or an archetype), which an Extra feat's own cap must equal.
    ranks: u8,
    /// A class, whose `ForcedTrue` prerequisites let it through on their own and are left out, as the check does.
    class: bool,
}

fn entries(data: &GameData) -> Vec<Entry<'_>> {
    let mut out = Vec::new();
    for feat in &data.feats {
        out.push(Entry {
            at: DataAt::Feat(feat.key.clone()),
            blueprint: feat.blueprint.as_ref(),
            prerequisite: feat.prerequisite.as_ref(),
            components: &feat.components,
            ranks: feat.ranks,
            class: false,
        });
    }
    for option in &data.options {
        out.push(Entry {
            at: DataAt::SelectionOption(option.key.clone()),
            blueprint: option.blueprint.as_ref(),
            prerequisite: option.prerequisite.as_ref(),
            components: &option.components,
            ranks: option.ranks,
            class: false,
        });
    }
    for ability in &data.mythic_abilities {
        out.push(Entry {
            at: DataAt::MythicAbility(ability.key.clone()),
            blueprint: ability.blueprint.as_ref(),
            prerequisite: ability.prerequisite.as_ref(),
            components: &ability.components,
            ranks: ability.ranks,
            class: false,
        });
    }
    for feat in &data.mythic_feats {
        out.push(Entry {
            at: DataAt::MythicFeat(feat.key.clone()),
            blueprint: feat.blueprint.as_ref(),
            prerequisite: feat.prerequisite.as_ref(),
            components: &feat.components,
            ranks: feat.ranks,
            class: false,
        });
    }
    for class in data.all_classes() {
        out.push(Entry {
            at: DataAt::Class(class.key.clone()),
            blueprint: class.blueprint.as_ref(),
            prerequisite: class.prerequisite.as_ref(),
            components: &class.components,
            ranks: 1,
            class: true,
        });
        for archetype in data.archetypes_of(&class.key) {
            out.push(Entry {
                at: DataAt::Archetype(archetype.key.clone()),
                blueprint: archetype.blueprint.as_ref(),
                prerequisite: archetype.prerequisite.as_ref(),
                components: &archetype.components,
                ranks: 1,
                class: false,
            });
        }
    }
    out
}

/// Whether an `--only` name takes an entry: the same name, or a name ending in `*` the entry's begins with the rest of.
fn takes(name: &str, entry: &str) -> bool {
    match name.strip_suffix('*') {
        Some(prefix) => entry.starts_with(prefix),
        None => name == entry,
    }
}

/// The first `--only` name that takes no entry of the target, if any.
pub(crate) fn unmatched<'a>(data: &GameData, only: &'a BTreeSet<String>) -> Option<&'a String> {
    let names: Vec<String> = entries(data)
        .iter()
        .map(|entry| entry.at.to_string())
        .collect();
    only.iter()
        .find(|name| !names.iter().any(|entry| takes(name, entry)))
}

/// The rewrites that narrow who qualifies and are written all the same (`prerequisite-narrowing.txt`): each an entry,
/// as the report names it, and one of its narrowings, `drops <fact>` or `adds Not <fact>`, with its line (number and
/// text) for [`stale`].
#[derive(Debug, Default)]
pub struct Narrowings {
    allowed: BTreeMap<(String, String), (usize, String)>,
}

impl Narrowings {
    /// The repo's own list, `tools/game-check/prerequisite-narrowing.txt`.
    pub fn embedded() -> Result<Narrowings, crate::Error> {
        Narrowings::parse(include_str!("../../prerequisite-narrowing.txt"))
    }

    /// One line each, `#` starting a comment: a narrowing, `<entry> / drops <fact>` or `<entry> / adds Not <fact>`,
    /// with ` -> <reason>`; or an entry's reason for each of its narrowings, `<entry> -> <reason>`. A narrowing with
    /// no reason of its own nor its entry's is refused, as is an entry's reason with no narrowing of that entry.
    pub fn parse(text: &str) -> Result<Narrowings, crate::Error> {
        let bad = |number: usize, why: &str| {
            crate::Error::Known(format!("prerequisite-narrowing.txt line {number}: {why}"))
        };
        let mut allowed = BTreeMap::new();
        // Each entry's reason, with its line.
        let mut reasoned = BTreeMap::new();
        // Each narrowing without a reason of its own, with its line, for its entry's.
        let mut unreasoned = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            // The line is trimmed, so a reason left empty ends it in ` ->`.
            let split = line
                .split_once(" -> ")
                .or_else(|| line.strip_suffix(" ->").map(|said| (said, "")));
            let (said, reason) = match split {
                Some((_, reason)) if reason.trim().is_empty() => {
                    return Err(bad(number, "an empty reason"));
                }
                Some((said, _)) => (said, true),
                None => (line, false),
            };
            let Some((entry, narrowing)) = said.split_once(" / ") else {
                if !reason {
                    return Err(bad(number, "neither a narrowing nor a reason"));
                }
                reasoned.insert(said.trim().to_owned(), number);
                continue;
            };
            let (entry, narrowing) = (entry.trim().to_owned(), narrowing.trim().to_owned());
            if !(narrowing.starts_with("drops ") || narrowing.starts_with("adds Not ")) {
                return Err(bad(
                    number,
                    "a narrowing is `drops <fact>` or `adds Not <fact>`",
                ));
            }
            if !reason {
                unreasoned.push((number, entry.clone()));
            }
            if allowed
                .insert((entry, narrowing), (number, line.to_owned()))
                .is_some()
            {
                return Err(bad(number, "a narrowing said twice"));
            }
        }
        if let Some((number, _)) = unreasoned
            .iter()
            .find(|(_, entry)| !reasoned.contains_key(entry))
        {
            return Err(bad(
                *number,
                "a narrowing with no reason, its own or its entry's",
            ));
        }
        if let Some(number) = reasoned
            .iter()
            .filter(|(entry, _)| !allowed.keys().any(|(each, _)| each == *entry))
            .map(|(_, number)| *number)
            .min()
        {
            return Err(bad(number, "a reason with no narrowing of its entry"));
        }
        Ok(Narrowings { allowed })
    }

    fn allows(&self, entry: &str, narrowing: &str) -> bool {
        self.allowed
            .contains_key(&(entry.to_owned(), narrowing.to_owned()))
    }
}

/// Each narrowing line no rewrite of `data` makes, `line <n>: <line>` in the file's order: its entry since written
/// or its data changed, it would allow that narrowing again unseen, so the check lists it and fails (308g), as it
/// does a stale `known.txt` line. `data` is read as the files hold it (`seeds::read_game_data_files`), not compiled:
/// the writer reads an entry's `Requires`.
pub fn stale(dump: &crate::Dump, data: &GameData, narrowings: &Narrowings) -> Vec<String> {
    let planner = crate::planner::Planner::new(data);
    let ctx = Ctx {
        dump,
        planner: &planner,
    };
    let (_, _, made) = plan(&ctx, None, narrowings);
    let mut stale: Vec<&(usize, String)> = narrowings
        .allowed
        .iter()
        .filter(|(said, _)| !made.contains(*said))
        .map(|(_, line)| line)
        .collect();
    stale.sort();
    stale
        .into_iter()
        .map(|(number, line)| format!("line {number}: {line}"))
        .collect()
}

/// The changes, in entry order, and the entries left whole with each gap that holds them.
pub(crate) fn changes(
    ctx: &Ctx,
    only: Option<&BTreeSet<String>>,
    narrowings: &Narrowings,
) -> (Vec<Change>, Vec<Unexpressed>) {
    let (changes, left, _) = plan(ctx, only, narrowings);
    (changes, left)
}

/// [`changes`], and every narrowing a rewrite makes, by entry, allowed or not.
fn plan(
    ctx: &Ctx,
    only: Option<&BTreeSet<String>>,
    narrowings: &Narrowings,
) -> (Vec<Change>, Vec<Unexpressed>, BTreeSet<(String, String)>) {
    let data = ctx.planner.data;
    let classes: BTreeMap<&ArchetypeKey, &ClassKey> = data
        .archetypes
        .iter()
        .map(|archetype| (&archetype.key, &archetype.class))
        .collect();
    let compiled = |components: &[Component]| {
        compile_requires(components, |archetype| classes.get(archetype).copied()).map(|mut tree| {
            tree.canonicalize();
            tree
        })
    };
    let (mut out, mut left, mut made) = (Vec::new(), Vec::new(), BTreeSet::new());
    for entry in entries(data) {
        let name = entry.at.to_string();
        if only.is_some_and(|only| !only.iter().any(|each| takes(each, &name))) {
            continue;
        }
        let Some(own) = entry.blueprint else {
            continue;
        };
        let Some(json) = ctx.dump.json(own) else {
            continue;
        };
        if entry.prerequisite.is_some() {
            left.push(Unexpressed {
                at: entry.at.clone(),
                what: "a hand prerequisite, which the writer never writes".to_owned(),
            });
            continue;
        }
        let requires = match target(ctx, &entry, own, json) {
            Ok(requires) => requires,
            Err(gaps) => {
                left.extend(gaps.into_iter().map(|gap| Unexpressed {
                    at: entry.at.clone(),
                    what: format!("{} -> {}", gap.what, gap.ticket),
                }));
                continue;
            }
        };
        let mut all = requires;
        all.extend(
            entry
                .components
                .iter()
                .filter(|part| !part.is_requires())
                .cloned(),
        );
        let (before, after) = (compiled(entry.components), compiled(&all));
        if before == after {
            continue;
        }
        let found = narrowings_of(ctx, own, before.as_ref(), after.as_ref());
        made.extend(
            found
                .iter()
                .map(|narrowing| (name.clone(), narrowing.said.clone())),
        );
        let blocked: Vec<Unexpressed> = found
            .into_iter()
            .filter(|narrowing| !narrowings.allows(&name, &narrowing.said))
            .map(|narrowing| Unexpressed {
                at: entry.at.clone(),
                what: format!(
                    "narrows: {} ({}) -> {}",
                    narrowing.said, narrowing.blueprint, narrowing.owner
                ),
            })
            .collect();
        if !blocked.is_empty() {
            left.extend(blocked);
            continue;
        }
        let tree =
            |tree: Option<Prerequisite>| tree.map_or_else(|| "-".to_owned(), |t| format!("{t:?}"));
        out.push(Change {
            file: super::entry_file(data, &entry.at),
            at: entry.at,
            field: "components",
            kind: KIND,
            value: (!all.is_empty()).then(|| Field::List(all.iter().map(ron_text).collect())),
            old: tree(before),
            new: tree(after),
        });
    }
    (out, left, made)
}

/// The facts a compiled tree names, by the report's name (`feat power-attack`), each with its entry: those it accepts
/// (under an even number of `Not`s) and those it refuses (an odd number). A refused leaf that is no fact (an ability
/// score) is named by its form, with no entry.
#[derive(Default)]
struct Named {
    accepted: BTreeMap<String, DataAt>,
    refused: BTreeMap<String, Option<DataAt>>,
}

impl Named {
    fn of(tree: Option<&Prerequisite>) -> Named {
        let mut named = Named::default();
        if let Some(tree) = tree {
            named.walk(tree, false);
        }
        named
    }

    fn walk(&mut self, node: &Prerequisite, refused: bool) {
        match node {
            Prerequisite::All(children)
            | Prerequisite::Any(children)
            | Prerequisite::AtLeast(_, children) => {
                for child in children {
                    self.walk(child, refused);
                }
            }
            Prerequisite::Not(child) => self.walk(child, !refused),
            leaf => match (leaf_at(leaf), refused) {
                (Some(at), false) => {
                    self.accepted.insert(at.to_string(), at);
                }
                (Some(at), true) => {
                    self.refused.insert(at.to_string(), Some(at));
                }
                (None, true) => {
                    self.refused.insert(format!("{leaf:?}"), None);
                }
                (None, false) => {}
            },
        }
    }

    fn names(&self, fact: &str) -> bool {
        self.accepted.contains_key(fact) || self.refused.contains_key(fact)
    }
}

/// The entry a fact leaf names, whatever its count, rank or value: a feat for `Weapon Focus (Longsword)`, a class for
/// a class level.
fn leaf_at(leaf: &Prerequisite) -> Option<DataAt> {
    Some(match leaf {
        Prerequisite::Feat(feat)
        | Prerequisite::FeatFor { feat, .. }
        | Prerequisite::FeatForSameValue(feat)
        | Prerequisite::FeatIfValue { feat, .. } => DataAt::Feat(feat.clone()),
        Prerequisite::MythicFeat(key) => DataAt::MythicFeat(key.clone()),
        Prerequisite::MythicAbility(key) => DataAt::MythicAbility(key.clone()),
        Prerequisite::ClassLevel { class, .. } => DataAt::Class(class.clone()),
        Prerequisite::Archetype(key) => DataAt::Archetype(key.clone()),
        Prerequisite::Feature { feature, .. } => DataAt::Feature(feature.clone()),
        Prerequisite::SelectionOption(key) => DataAt::SelectionOption(key.clone()),
        Prerequisite::Selection(key) => DataAt::FeatureSelection(key.clone()),
        Prerequisite::Race(key) => DataAt::Race(key.clone()),
        Prerequisite::Deity(key) => DataAt::Deity(key.clone()),
        Prerequisite::Rank { fact, .. } => match fact {
            Fact::Feat(key) => DataAt::Feat(key.clone()),
            Fact::Feature(key) => DataAt::Feature(key.clone()),
            Fact::Option(key) => DataAt::SelectionOption(key.clone()),
            Fact::Selection(key) => DataAt::FeatureSelection(key.clone()),
            Fact::MythicAbility(key) => DataAt::MythicAbility(key.clone()),
            Fact::MythicFeat(key) => DataAt::MythicFeat(key.clone()),
            Fact::Class(key) => DataAt::Class(key.clone()),
            Fact::Archetype(key) => DataAt::Archetype(key.clone()),
            Fact::Race(key) => DataAt::Race(key.clone()),
            Fact::Deity(key) => DataAt::Deity(key.clone()),
        },
        _ => return None,
    })
}

/// One way a rewrite narrows who qualifies: `drops <fact>` or `adds Not <fact>`, the fact's blueprint, and the ticket
/// that owns it ([`owner`] of the fact's blueprint, 308 when none does).
struct Narrowing {
    said: String,
    blueprint: String,
    owner: &'static str,
}

/// How a rewrite narrows who qualifies:
/// - a fact the old tree accepts and the new one doesn't name: where the planner holds the game's blueprint under
///   another entry, or a grantor's rows are still the planner's, the rewrite would refuse a character the game allows;
/// - a fact the new tree refuses and the old one didn't, but an entry of the entry's own blueprint (itself, or another
///   entry sharing its GUID), which is the game's own `PrerequisiteNoFeature` exactly.
fn narrowings_of(
    ctx: &Ctx,
    own: &BlueprintGuid,
    before: Option<&Prerequisite>,
    after: Option<&Prerequisite>,
) -> Vec<Narrowing> {
    let (old, new) = (Named::of(before), Named::of(after));
    let itself: BTreeSet<String> = ctx
        .planner
        .entries_of(own)
        .into_iter()
        .map(ToString::to_string)
        .collect();
    let narrowing = |said: String, at: Option<&DataAt>| {
        let guid = at.and_then(|at| ctx.planner.guid(at.clone()));
        Narrowing {
            said,
            blueprint: blueprint_of(ctx, at),
            owner: guid.and_then(|guid| owner(ctx, guid)).unwrap_or("308"),
        }
    };
    let dropped = old
        .accepted
        .iter()
        .filter(|(fact, _)| !new.names(fact))
        .map(|(fact, at)| narrowing(format!("drops {fact}"), Some(at)));
    let refused = new
        .refused
        .iter()
        .filter(|(fact, _)| !old.refused.contains_key(*fact) && !itself.contains(*fact))
        .map(|(fact, at)| narrowing(format!("adds Not {fact}"), at.as_ref()));
    dropped.chain(refused).collect()
}

/// A fact's blueprint for a narrowing's line: its asset name and GUID, or why there is none.
fn blueprint_of(ctx: &Ctx, at: Option<&DataAt>) -> String {
    let Some(at) = at else {
        return "no fact".to_owned();
    };
    match ctx.planner.guid(at.clone()) {
        Some(guid) if ctx.dump.kind(guid).is_some() => format!("{} {guid}", ctx.name(guid)),
        Some(guid) => format!("{guid}, not in the dump"),
        None => "no blueprint, the planner's own entry".to_owned(),
    }
}

/// The entry's `Requires`, in the blueprint's component order, or every gap.
fn target(
    ctx: &Ctx,
    entry: &Entry,
    own: &BlueprintGuid,
    json: &Value,
) -> Result<Vec<Component>, Vec<Gap>> {
    let mut written: Vec<(Condition, Group)> = Vec::new();
    let mut gaps = Vec::new();
    // A component every Build meets, in the `Any` group, holds the whole group (the check's reading).
    let mut any_holds = false;
    for part in components(json) {
        let kind = component_kind(part);
        if !kind.starts_with("Prerequisite") {
            continue;
        }
        let group = part["Group"].as_str().unwrap_or("All");
        if entry.class && group == "ForcedTrue" {
            continue;
        }
        if holds_for_every_build(part, kind) {
            any_holds |= group != "All";
            continue;
        }
        match condition(ctx, entry, own, json, part) {
            Ok(Some(condition)) => written.push(condition),
            Ok(None) => {}
            Err(gap) => gaps.push(gap),
        }
    }
    if !gaps.is_empty() {
        return Err(gaps);
    }
    Ok(written
        .into_iter()
        .filter(|(_, group)| !(any_holds && *group == Group::Any))
        .map(|(condition, group)| Component::Requires { condition, group })
        .collect())
}

/// One component as a condition and its group; `None` for one left out because it always holds (§1.1).
fn condition(
    ctx: &Ctx,
    entry: &Entry,
    own: &BlueprintGuid,
    json: &Value,
    part: &Value,
) -> Result<Option<(Condition, Group)>, Gap> {
    let kind = component_kind(part);
    let group = if part["Group"].as_str().unwrap_or("All") == "All" {
        Group::All
    } else {
        Group::Any
    };
    let field = |name: &str| {
        reference(&part[name])
            .ok_or_else(|| Gap::new(format!("{kind} with an empty {name}"), "308"))
    };
    let number = |name: &str| {
        u8::try_from(int(&part[name])).map_err(|_| {
            Gap::new(
                format!("{kind}'s {name} {} out of range", int(&part[name])),
                "308",
            )
        })
    };
    let condition = match kind {
        "PrerequisiteFeature" | "PrerequisitePlayerHasFeature" => {
            has(&facts(ctx, &field("m_Feature")?)?)
        }
        "PrerequisiteNoFeature" => {
            let feature = field("m_Feature")?;
            if feature == *own && group == Group::All {
                return Ok(own_no_feature(ctx, entry, own, json, part).map(|c| (c, group)));
            }
            has_not(facts(ctx, &feature)?)
        }
        "PrerequisiteFeaturesFromList" => {
            let amount = number("Amount")?.max(1);
            let mut all = BTreeSet::new();
            for listed in references(part.get("m_Features")) {
                let held = facts(ctx, &listed)?;
                if held.len() > 1 && amount >= 2 {
                    return Err(Gap::new(
                        format!(
                            "PrerequisiteFeaturesFromList of {amount} over {} {listed}, which several entries share",
                            ctx.name(&listed)
                        ),
                        "308",
                    ));
                }
                all.extend(held);
            }
            if all.is_empty() {
                return Err(Gap::new("PrerequisiteFeaturesFromList of nothing", "308"));
            }
            let at_least = Condition::AtLeast(amount, all);
            if flag(&part["m_RestrictIfNot"]) {
                Condition::Not(Box::new(at_least))
            } else {
                at_least
            }
        }
        "PrerequisiteStatValue" | "PrerequisiteFullStatValue" => {
            let full = kind == "PrerequisiteFullStatValue";
            let stat = part["Stat"].as_str().unwrap_or("");
            let value = int(&part["Value"]);
            let stat = if let Some(ability) = Ability::from_key(stat) {
                if value <= LOWEST_SCORE {
                    return Ok(None);
                }
                StatRef::Ability(ability)
            } else if stat == "BaseAttackBonus" {
                StatRef::BaseAttack
            } else if let (false, Some(skill)) = (full, game_skill(stat)) {
                StatRef::Skill(skill)
            } else {
                return Err(Gap::new(
                    format!("{kind} of {stat}, which no stat of the planner's reads"),
                    "314",
                ));
            };
            Condition::Stat {
                stat,
                value: number("Value")?,
                full,
            }
        }
        "PrerequisiteClassLevel" => {
            let level =
                Condition::ClassLevel(class(ctx, &field("m_CharacterClass")?)?, number("Level")?);
            if flag(&part["Not"]) {
                Condition::Not(Box::new(level))
            } else {
                level
            }
        }
        "PrerequisiteNoClassLevel" => {
            let class_guid = field("m_CharacterClass")?;
            if class_guid == guid(ANIMAL_CLASS) {
                return Ok(None);
            }
            Condition::NoClassLevel(class(ctx, &class_guid)?)
        }
        // A level of 1 is the archetype taken: `ArchetypeLevel(_, 1)` would compile to a class level 1 more.
        "PrerequisiteArchetypeLevel" => {
            let archetype = archetype(ctx, &field("m_Archetype")?)?;
            match number("Level")? {
                0 | 1 => Condition::Has(Fact::Archetype(archetype)),
                level => Condition::ArchetypeLevel(archetype, level),
            }
        }
        "PrerequisiteNoArchetype" => {
            Condition::NoArchetype(archetype(ctx, &field("m_Archetype")?)?)
        }
        "PrerequisiteAlignment" => {
            let names = part["Alignment"].as_str().unwrap_or("");
            let set: BTreeSet<Alignment> = alignment_mask(names)
                .map(|mask| {
                    Alignment::ALL
                        .iter()
                        .enumerate()
                        .filter(|(bit, _)| mask & (1 << bit) != 0)
                        .map(|(_, alignment)| *alignment)
                        .collect()
                })
                .unwrap_or_default();
            if set.is_empty() {
                return Err(Gap::new(
                    format!("PrerequisiteAlignment of `{names}`"),
                    "308",
                ));
            }
            Condition::Alignment(set)
        }
        "PrerequisiteCasterTypeSpellLevel" => Condition::CasterTypeSpellLevel {
            caster: caster(part),
            level: number("RequiredSpellLevel")?,
            spontaneous: flag(&part["OnlySpontaneous"]),
        },
        "PrerequisiteCasterType" => Condition::CasterType(caster(part)),
        "PrerequisiteCharacterLevel" => Condition::CharacterLevel(number("Level")?),
        "PrerequisiteMythicLevel" => Condition::MythicLevel(number("Level")?),
        "PrerequisiteProficiency" | "PrerequisiteNotProficient" => {
            let groups = armour_groups(part)
                .map_err(|ticket| Gap::new(format!("{kind} of a weapon or a barding"), ticket))?;
            if groups.is_empty() {
                return Err(Gap::new(format!("{kind} of nothing"), "313"));
            }
            if kind == "PrerequisiteProficiency" {
                Condition::Proficient(groups)
            } else {
                Condition::NotProficient(groups)
            }
        }
        "PrerequisiteParametrizedFeature" => parametrized(ctx, part, &field("m_Feature")?)?,
        "PrerequisiteIsPet" if flag(&part["Not"]) => return Ok(None),
        "PrerequisiteIsPet" | "PrerequisitePet" => {
            return Err(Gap::new(format!("{kind}, a pet's"), "174"));
        }
        "PrerequisiteCondition" => match own_cap(own, part) {
            Some(cap) if i64::from(entry.ranks) == cap => {
                let Some(fact) = fact_at(&entry.at) else {
                    return Err(Gap::new("an own cap on an entry that is no fact", "308"));
                };
                Condition::Not(Box::new(Condition::FactRank(fact, entry.ranks)))
            }
            Some(cap) => {
                return Err(Gap::new(
                    format!(
                        "an own cap of {cap}, not the header's ranks {}",
                        entry.ranks
                    ),
                    "308",
                ));
            }
            None => return Err(Gap::new("a PrerequisiteCondition's tree", "308")),
        },
        "PrerequisiteParametrizedWeaponSubcategory" | "PrerequisiteConditionForWeaponCategory" => {
            return Err(Gap::new(format!("{kind}, a weapon category"), "313"));
        }
        "PrerequisiteAnySpellsInSpellbook" => {
            return Err(Gap::new(format!("{kind}, a spellbook's spells"), "317"));
        }
        other => return Err(Gap::new(format!("{other}, no form the planner has"), "308")),
    };
    Ok(Some((condition, group)))
}

/// A feature's `PrerequisiteNoFeature` of its own blueprint (`All` group): none on a feature of one rank whose
/// blueprint is the entry's alone (the check's reading, `is_own_no_feature`); `HasNot` of itself on a selection or a
/// feature of more ranks (303); on a blueprint several entries share, `HasNot` of the others too (306f's Evasion).
fn own_no_feature(
    ctx: &Ctx,
    entry: &Entry,
    own: &BlueprintGuid,
    json: &Value,
    part: &Value,
) -> Option<Condition> {
    let shared = ctx.planner.entries_of(own).len() > 1;
    let plain = is_own_no_feature(json, own, part);
    if plain && !shared {
        return None;
    }
    let itself = fact_at(&entry.at);
    let refused: Vec<Fact> = facts_of(ctx, own)
        .into_iter()
        .filter(|fact| !plain || Some(fact) != itself.as_ref())
        .collect();
    (!refused.is_empty()).then(|| has_not(refused))
}

/// The facts of the entries that map a blueprint, or the gap of a blueprint no entry maps.
fn facts(ctx: &Ctx, listed: &BlueprintGuid) -> Result<Vec<Fact>, Gap> {
    if !ctx.planner.maps(listed) {
        return Err(unmapped(ctx, listed));
    }
    let facts = facts_of(ctx, listed);
    if facts.is_empty() {
        return Err(Gap::new(
            format!(
                "{} {listed}, whose entries are no fact a character holds",
                ctx.name(listed)
            ),
            "308",
        ));
    }
    Ok(facts)
}

/// `Has` of the one fact, or `Any` of each.
fn has(facts: &[Fact]) -> Condition {
    match facts {
        [one] => Condition::Has(one.clone()),
        many => Condition::Any(many.iter().cloned().map(Condition::Has).collect()),
    }
}

/// `HasNot` of the one fact, or `Not` of `Any` of each.
fn has_not(facts: Vec<Fact>) -> Condition {
    match <[Fact; 1]>::try_from(facts) {
        Ok([one]) => Condition::HasNot(one),
        Err(many) => Condition::Not(Box::new(has(&many))),
    }
}

/// The class a blueprint is.
fn class(ctx: &Ctx, class: &BlueprintGuid) -> Result<ClassKey, Gap> {
    entry_of(ctx, class, |at| match at {
        DataAt::Class(key) => Some(key.clone()),
        _ => None,
    })
}

/// The archetype a blueprint is.
fn archetype(ctx: &Ctx, archetype: &BlueprintGuid) -> Result<ArchetypeKey, Gap> {
    entry_of(ctx, archetype, |at| match at {
        DataAt::Archetype(key) => Some(key.clone()),
        _ => None,
    })
}

/// The feat a blueprint is.
fn feat(ctx: &Ctx, feat: &BlueprintGuid) -> Result<FeatKey, Gap> {
    entry_of(ctx, feat, |at| match at {
        DataAt::Feat(key) => Some(key.clone()),
        _ => None,
    })
}

/// The one entry of a kind that maps a blueprint, or the gap.
fn entry_of<T>(
    ctx: &Ctx,
    guid: &BlueprintGuid,
    pick: impl Fn(&DataAt) -> Option<T>,
) -> Result<T, Gap> {
    if !ctx.planner.maps(guid) {
        return Err(unmapped(ctx, guid));
    }
    let mut found = ctx.planner.entries_of(guid).into_iter().filter_map(pick);
    match (found.next(), found.next()) {
        (Some(one), None) => Ok(one),
        _ => Err(Gap::new(
            format!(
                "{} {guid}, which no single entry of its kind maps",
                ctx.name(guid)
            ),
            owner(ctx, guid).unwrap_or("308"),
        )),
    }
}

fn caster(part: &Value) -> SpellType {
    if flag(&part["IsArcane"]) {
        SpellType::Arcane
    } else {
        SpellType::Divine
    }
}

/// `PrerequisiteParametrizedFeature`: the feat held for a school, or for a weapon the data has.
fn parametrized(ctx: &Ctx, part: &Value, feature: &BlueprintGuid) -> Result<Condition, Gap> {
    let feat = feat(ctx, feature)?;
    let value = match part["ParameterType"].as_str().unwrap_or("") {
        "SpellSchool" => {
            let name = part["SpellSchool"].as_str().unwrap_or("");
            let school = SpellSchool::ALL
                .into_iter()
                .find(|school| school.name() == name)
                .ok_or_else(|| {
                    Gap::new(
                        format!("a parametrized feature of the school `{name}`"),
                        "308",
                    )
                })?;
            Parameter::SpellSchool(school)
        }
        "WeaponCategory" => {
            let category = part["WeaponCategory"].as_str().unwrap_or("");
            let key = words(category).join("-");
            if !ctx
                .planner
                .data
                .weapons
                .iter()
                .any(|weapon| weapon.key.as_str() == key)
            {
                return Err(Gap::new(
                    format!(
                        "a parametrized feature of the weapon category {category}, which the data has no weapon for"
                    ),
                    "313",
                ));
            }
            Parameter::Weapon(WeaponKey::new(key.as_str()))
        }
        other => {
            return Err(Gap::new(
                format!("a parametrized feature of a {other}"),
                "313",
            ));
        }
    };
    Ok(Condition::ParametrizedFeature(feat, value))
}

/// The cap a selection puts on its own rank: a `PrerequisiteCondition` holding exactly `Not` of
/// `ContextConditionBuffRank` on its own blueprint at a simple value (`ExtraWitchHexSelection`
/// b6054088b4ab4be286724127cbf48b35 at 3).
fn own_cap(own: &BlueprintGuid, part: &Value) -> Option<i64> {
    let condition = &part["Condition"];
    if flag(&condition["Not"]) {
        return None;
    }
    let [rank] = condition["ConditionsChecker"]["Conditions"]
        .as_array()?
        .as_slice()
    else {
        return None;
    };
    (component_kind(rank) == "ContextConditionBuffRank"
        && flag(&rank["Not"])
        && flag(&rank["UseFactInsteadBuff"])
        && reference(&rank["Fact"]).as_ref() == Some(own)
        && rank["RankValue"]["ValueType"].as_str() == Some("Simple"))
    .then(|| int(&rank["RankValue"]["Value"]))
}

/// The gap of a blueprint no entry maps, with the ticket that owns it ([`owner`]), S3's carriers (307) by default.
fn unmapped(ctx: &Ctx, guid: &BlueprintGuid) -> Gap {
    Gap::new(
        format!("{} {guid}, which no entry maps", ctx.name(guid)),
        owner(ctx, guid).unwrap_or("307"),
    )
}

/// The ticket that owns a blueprint the writer can't name, by its kind (§1.2; the writer's inference from the asset
/// name and, for a deity, its `ForbidSpellbookOnAlignmentDeviation`), when one does.
fn owner(ctx: &Ctx, guid: &BlueprintGuid) -> Option<&'static str> {
    let name = ctx.name(guid);
    let deity = ctx.dump.json(guid).is_some_and(|json| {
        components(json)
            .iter()
            .any(|part| component_kind(part) == "ForbidSpellbookOnAlignmentDeviation")
    });
    let has = |part: &str| name.contains(part);
    let ticket = if deity || name.ends_with("DomainAllowed") || name.starts_with("Atheism") {
        "267"
    } else if has("DomainProgression") || has("SubdomainProgression") {
        "294"
    } else if ["Sage", "Sylvan", "Empyreal"]
        .iter()
        .any(|copy| name.starts_with(&format!("{copy}Bloodline")))
    {
        "316"
    } else if has("Bloodline") || name.starts_with("Dragonheir") || name.starts_with("Crossblooded")
    {
        "306x/y"
    } else if has("MysteryFeature") {
        "306v"
    } else if has("SecondaryElement") || has("ThirdElement") || has("KineticKnightElementalFocus") {
        "306t"
    } else if name.ends_with("CurseNoProgression") {
        "306w"
    } else if name.starts_with("OppositionSchool") {
        "306p"
    } else if ["WinterWitch", "Eldritch", "LoremasterTrickster"]
        .iter()
        .any(|split| name.starts_with(split))
    {
        "306o"
    } else if ["SkillFocus", "ArmorFocus", "ElementalFocus"]
        .iter()
        .any(|per_value| name.starts_with(per_value))
    {
        "315"
    } else if name.starts_with("Animal") || name.contains("Barding") {
        "174"
    } else if name.ends_with("MythicClass")
        || name == "GoldenDragonClass"
        || name == "SwarmThatWalksClass"
        || name.starts_with("LichSkeletal")
    {
        "266"
    } else {
        return None;
    };
    Some(ticket)
}
