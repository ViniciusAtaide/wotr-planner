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

/// The changes, in entry order, and the entries left whole with each gap that holds them.
pub(crate) fn changes(
    ctx: &Ctx,
    only: Option<&BTreeSet<String>>,
) -> (Vec<Change>, Vec<Unexpressed>) {
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
    let (mut out, mut left) = (Vec::new(), Vec::new());
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
    (out, left)
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
