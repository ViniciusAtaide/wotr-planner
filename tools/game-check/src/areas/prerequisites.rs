//! Prerequisites of feats, selection options, mythic abilities and feats, classes and archetypes.
//!
//! The game's: a blueprint's `Prerequisite*` components. Those of group `All` must all hold, and of the others at
//! least one (`BlueprintFeature.MeetsPrerequisites`, which counts `ForcedTrue` with `Any`); a class's `ForcedTrue`
//! one lets it through on its own (`BlueprintCharacterClass.MeetsPrerequisites`), which the check leaves out (WrathDecompiled
//! 6ce4a6b). Both sides become one normal form over blueprint GUIDs, sorted and flattened, and are compared there.
//! What either side has that the other can't say is left out and counted.

use std::cmp::Ordering;

use rules::{Alignment, BlueprintGuid, DataAt, Parameter, Prerequisite, SpellType};
use serde_json::Value;

use super::fixed::{ANIMAL_CLASS, guid};
use super::{Ctx, ability_stat, is_ability_stat, skill_stat};
use crate::dump::{component_kind, components, flag, int, reference, references};
use crate::report::{Area, Out};

const AREA: Area = Area::Prerequisites;

/// The lowest ability score a Build can have: 7 bought, less a racial 2. A game prerequisite at or below it always
/// holds (`ShatterDefenses`' Intelligence 3).
const LOWEST_SCORE: i64 = 5;

/// One condition, in blueprint GUIDs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Leaf {
    /// Holds a feature, feat, option, mythic pick, race or deity.
    Has(BlueprintGuid),
    Ability(String, i64),
    Ranks(String, i64),
    Bab(i64),
    ClassLevel(BlueprintGuid, i64),
    Archetype(BlueprintGuid),
    /// Any of the alignments in the mask (`AlignmentMaskType`'s bits).
    Alignments(u16),
    SpellLevel {
        arcane: bool,
        spontaneous: bool,
        level: i64,
    },
    CharacterLevel(i64),
    MythicRank(i64),
}

impl Leaf {
    /// The leaf without its number: two leaves with one subject and different numbers are one value that differs.
    fn subject(&self) -> Leaf {
        match self {
            Leaf::Ability(stat, _) => Leaf::Ability(stat.clone(), 0),
            Leaf::Ranks(stat, _) => Leaf::Ranks(stat.clone(), 0),
            Leaf::Bab(_) => Leaf::Bab(0),
            Leaf::ClassLevel(class, _) => Leaf::ClassLevel(class.clone(), 0),
            Leaf::Alignments(_) => Leaf::Alignments(0),
            Leaf::SpellLevel {
                arcane,
                spontaneous,
                ..
            } => Leaf::SpellLevel {
                arcane: *arcane,
                spontaneous: *spontaneous,
                level: 0,
            },
            Leaf::CharacterLevel(_) => Leaf::CharacterLevel(0),
            Leaf::MythicRank(_) => Leaf::MythicRank(0),
            Leaf::Has(_) | Leaf::Archetype(_) => self.clone(),
        }
    }

    fn guid(&self) -> Option<&BlueprintGuid> {
        match self {
            Leaf::Has(guid) | Leaf::ClassLevel(guid, _) | Leaf::Archetype(guid) => Some(guid),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Leaf(Leaf),
    Not(Box<Node>),
    All(Vec<Node>),
    Any(Vec<Node>),
    AtLeast(i64, Vec<Node>),
}

pub fn compare(ctx: &Ctx, out: &mut Out) {
    let data = ctx.planner.data;
    for feat in &data.feats {
        entry(
            ctx,
            out,
            DataAt::Feat(feat.key.clone()),
            feat.blueprint.as_ref(),
            feat.prerequisite.as_ref(),
            false,
        );
    }
    for option in data.selections().flat_map(|selection| &selection.options) {
        let at = DataAt::SelectionOption(option.key.clone());
        entry(
            ctx,
            out,
            at,
            option.blueprint.as_ref(),
            option.prerequisite.as_ref(),
            false,
        );
    }
    for ability in &data.mythic_abilities {
        let at = DataAt::MythicAbility(ability.key.clone());
        entry(
            ctx,
            out,
            at,
            ability.blueprint.as_ref(),
            ability.prerequisite.as_ref(),
            false,
        );
    }
    for feat in &data.mythic_feats {
        let at = DataAt::MythicFeat(feat.key.clone());
        entry(
            ctx,
            out,
            at,
            feat.blueprint.as_ref(),
            feat.prerequisite.as_ref(),
            false,
        );
    }
    for class in data.all_classes() {
        let at = DataAt::Class(class.key.clone());
        entry(
            ctx,
            out,
            at,
            class.blueprint.as_ref(),
            class.prerequisite.as_ref(),
            true,
        );
        for archetype in &class.archetypes {
            let at = DataAt::Archetype(archetype.key.clone());
            entry(
                ctx,
                out,
                at,
                archetype.blueprint.as_ref(),
                archetype.prerequisite.as_ref(),
                false,
            );
        }
    }
}

fn entry(
    ctx: &Ctx,
    out: &mut Out,
    at: DataAt,
    guid: Option<&BlueprintGuid>,
    prerequisite: Option<&Prerequisite>,
    class: bool,
) {
    let Some((guid, json)) = ctx.blueprint(out, AREA, guid, at.kind()) else {
        return;
    };
    let mark = out.mark();
    let game = game_side(json, guid, class, out).and_then(canon);
    let planner = prerequisite
        .and_then(|prerequisite| planner_side(ctx, prerequisite, out))
        .and_then(canon);
    if game != planner {
        differences(ctx, out, &at, guid, game.as_ref(), planner.as_ref());
    }
    out.compared(AREA, mark);
}

fn differences(
    ctx: &Ctx,
    out: &mut Out,
    at: &DataAt,
    guid: &BlueprintGuid,
    game: Option<&Node>,
    planner: Option<&Node>,
) {
    let name = ctx.name(guid);
    let mut game_leaves = Vec::new();
    let mut planner_leaves = Vec::new();
    if let Some(game) = game {
        leaves(game, false, &mut game_leaves);
    }
    if let Some(planner) = planner {
        leaves(planner, false, &mut planner_leaves);
    }
    remove_common(&mut game_leaves, &mut planner_leaves);
    let mark = out.mark();
    for (negated, leaf) in game_leaves {
        let subject = (negated, leaf.subject());
        let same = planner_leaves
            .iter()
            .position(|(other_negated, other)| (*other_negated, other.subject()) == subject);
        if let Some(index) = same {
            let (planner_negated, planner_leaf) = planner_leaves.remove(index);
            out.finding(
                AREA,
                "value differs",
                at,
                guid,
                &name,
                render_leaf(ctx, negated, &leaf),
                render_leaf(ctx, planner_negated, &planner_leaf),
            );
        } else {
            let kind = if leaf.guid().is_some_and(|guid| !ctx.planner.maps(guid)) {
                "missing in planner, a blueprint it doesn't map"
            } else {
                "missing in planner"
            };
            out.finding(
                AREA,
                kind,
                at,
                guid,
                &name,
                render_leaf(ctx, negated, &leaf),
                "-",
            );
        }
    }
    for (negated, leaf) in planner_leaves {
        out.finding(
            AREA,
            "not in game",
            at,
            guid,
            &name,
            "-",
            render_leaf(ctx, negated, &leaf),
        );
    }
    if out.mark() == mark {
        out.finding(
            AREA,
            "grouping differs",
            at,
            guid,
            &name,
            game.map_or_else(|| "-".to_owned(), |node| render(ctx, node)),
            planner.map_or_else(|| "-".to_owned(), |node| render(ctx, node)),
        );
    }
}

/// Removes the leaves both lists hold, as multisets.
fn remove_common(game: &mut Vec<(bool, Leaf)>, planner: &mut Vec<(bool, Leaf)>) {
    game.sort();
    planner.sort();
    let (mut kept_game, mut kept_planner) = (Vec::new(), Vec::new());
    let (mut g, mut p) = (game.drain(..).peekable(), planner.drain(..).peekable());
    loop {
        match (g.peek(), p.peek()) {
            (Some(a), Some(b)) => match a.cmp(b) {
                Ordering::Equal => {
                    g.next();
                    p.next();
                }
                Ordering::Less => kept_game.extend(g.next()),
                Ordering::Greater => kept_planner.extend(p.next()),
            },
            (Some(_), None) => kept_game.extend(g.next()),
            (None, Some(_)) => kept_planner.extend(p.next()),
            (None, None) => break,
        }
    }
    drop((g, p));
    *game = kept_game;
    *planner = kept_planner;
}

fn leaves(node: &Node, negated: bool, out: &mut Vec<(bool, Leaf)>) {
    match node {
        Node::Leaf(leaf) => out.push((negated, leaf.clone())),
        Node::Not(child) => leaves(child, !negated, out),
        Node::All(children) | Node::Any(children) | Node::AtLeast(_, children) => {
            for child in children {
                leaves(child, negated, out);
            }
        }
    }
}

/// The normal form: groups flattened into a parent of their kind, alignments in one group folded into one mask,
/// children sorted and deduplicated, a group of one its child, an empty group nothing.
fn canon(node: Node) -> Option<Node> {
    match node {
        Node::Leaf(_) => Some(node),
        Node::Not(child) => canon(*child).map(|child| match child {
            Node::Not(inner) => *inner,
            child => Node::Not(Box::new(child)),
        }),
        Node::All(children) => group(children, false),
        Node::Any(children) => group(children, true),
        Node::AtLeast(count, children) if count <= 1 => group(children, true),
        Node::AtLeast(count, children) => {
            let mut children: Vec<Node> = children.into_iter().filter_map(canon).collect();
            children.sort();
            children.dedup();
            Some(Node::AtLeast(count, children))
        }
    }
}

fn group(children: Vec<Node>, any: bool) -> Option<Node> {
    let mut flat = Vec::new();
    let mut mask: Option<u16> = None;
    for child in children.into_iter().filter_map(canon) {
        match child {
            Node::Any(inner) if any => flat.extend(inner),
            Node::All(inner) if !any => flat.extend(inner),
            child => flat.push(child),
        }
    }
    flat.retain(|child| match child {
        Node::Leaf(Leaf::Alignments(bits)) => {
            mask = Some(match mask {
                None => *bits,
                Some(mask) if any => mask | bits,
                Some(mask) => mask & bits,
            });
            false
        }
        _ => true,
    });
    if let Some(mask) = mask {
        flat.push(Node::Leaf(Leaf::Alignments(mask)));
    }
    flat.sort();
    flat.dedup();
    match flat.len() {
        0 => None,
        1 => flat.pop(),
        _ if any => Some(Node::Any(flat)),
        _ => Some(Node::All(flat)),
    }
}

/// The game's prerequisites of a blueprint, before the normal form.
fn game_side(json: &Value, own: &BlueprintGuid, class: bool, out: &mut Out) -> Option<Node> {
    let mut all = Vec::new();
    let mut any = Vec::new();
    for component in components(json) {
        let kind = component_kind(component);
        if !kind.starts_with("Prerequisite") {
            continue;
        }
        let group = component["Group"].as_str().unwrap_or("All");
        if class && group == "ForcedTrue" {
            let own_level = kind == "PrerequisiteClassLevel"
                && reference(&component["m_CharacterClass"]).as_ref() == Some(own);
            out.skip(
                AREA,
                if own_level {
                    "a class's forced-true level in itself"
                } else {
                    "a class's other forced-true prerequisite"
                },
            );
            continue;
        }
        // A feature's `PrerequisiteNoFeature` of itself: `BlueprintFeature.MeetsPrerequisites` already refuses a
        // feature held at its `Ranks`, so with one rank it says nothing the feature doesn't.
        if kind == "PrerequisiteNoFeature"
            && group == "All"
            && reference(&component["m_Feature"]).as_ref() == Some(own)
            && json["Ranks"].as_i64().unwrap_or(1) <= 1
        {
            out.skip(AREA, "a feature's own NoFeature, not compared");
            continue;
        }
        if let Some(node) = game_component(component, kind, out) {
            if group == "All" {
                all.push(node);
            } else {
                any.push(node);
            }
        }
    }
    if !any.is_empty() {
        all.push(Node::Any(any));
    }
    Some(Node::All(all))
}

fn game_component(component: &Value, kind: &str, out: &mut Out) -> Option<Node> {
    let has = |field: &str| reference(&component[field]).map(|guid| Node::Leaf(Leaf::Has(guid)));
    let not = |node: Node| Node::Not(Box::new(node));
    let node = match kind {
        "PrerequisiteFeature" => has("m_Feature"),
        "PrerequisiteNoFeature" => has("m_Feature").map(not),
        "PrerequisiteFeaturesFromList" => {
            let features: Vec<Node> = references(component.get("m_Features"))
                .into_iter()
                .map(|guid| Node::Leaf(Leaf::Has(guid)))
                .collect();
            let amount = int(&component["Amount"]);
            let node = if amount <= 1 {
                Node::Any(features)
            } else {
                Node::AtLeast(amount, features)
            };
            Some(if flag(&component["m_RestrictIfNot"]) {
                not(node)
            } else {
                node
            })
        }
        "PrerequisiteStatValue" | "PrerequisiteFullStatValue" => {
            let stat = component["Stat"].as_str().unwrap_or("");
            let value = int(&component["Value"]);
            // Inference: the full value (`PrerequisiteFullStatValue.GetFullStatValue`) is the permanent value plus
            // the stat's `Feat` modifiers, which the planner has none of for an ability score or BAB; read as
            // `PrerequisiteStatValue`. Of another stat it isn't compared.
            let full = kind == "PrerequisiteFullStatValue";
            if is_ability_stat(stat) {
                if value <= LOWEST_SCORE {
                    out.skip(AREA, "an ability score no Build can fail");
                    return None;
                }
                Some(Node::Leaf(Leaf::Ability(stat.to_owned(), value)))
            } else if stat == "BaseAttackBonus" {
                Some(Node::Leaf(Leaf::Bab(value)))
            } else if full {
                out.skip(AREA, format!("a full {stat} prerequisite, not compared"));
                return None;
            } else if stat.starts_with("Skill") {
                Some(Node::Leaf(Leaf::Ranks(stat.to_owned(), value)))
            } else {
                out.skip(AREA, format!("a {stat} prerequisite"));
                return None;
            }
        }
        "PrerequisiteClassLevel" => reference(&component["m_CharacterClass"]).map(|class| {
            let node = Node::Leaf(Leaf::ClassLevel(class, int(&component["Level"])));
            if flag(&component["Not"]) {
                not(node)
            } else {
                node
            }
        }),
        "PrerequisiteNoClassLevel" => match reference(&component["m_CharacterClass"]) {
            Some(class) if class == guid(ANIMAL_CLASS) => {
                out.skip(AREA, "no animal class levels (pets)");
                return None;
            }
            class => class.map(|class| not(Node::Leaf(Leaf::ClassLevel(class, 1)))),
        },
        "PrerequisiteArchetypeLevel" => reference(&component["m_Archetype"]).map(|archetype| {
            let archetype = Node::Leaf(Leaf::Archetype(archetype));
            let level = int(&component["Level"]);
            match reference(&component["m_CharacterClass"]) {
                Some(class) if level > 1 => {
                    Node::All(vec![archetype, Node::Leaf(Leaf::ClassLevel(class, level))])
                }
                _ => archetype,
            }
        }),
        "PrerequisiteNoArchetype" => reference(&component["m_Archetype"])
            .map(|archetype| not(Node::Leaf(Leaf::Archetype(archetype)))),
        "PrerequisiteAlignment" => {
            match alignment_mask(component["Alignment"].as_str().unwrap_or("")) {
                Some(mask) => Some(Node::Leaf(Leaf::Alignments(mask))),
                None => {
                    out.skip(AREA, "an alignment mask the check can't read");
                    return None;
                }
            }
        }
        "PrerequisiteCasterTypeSpellLevel" => Some(Node::Leaf(Leaf::SpellLevel {
            arcane: flag(&component["IsArcane"]),
            spontaneous: flag(&component["OnlySpontaneous"]),
            level: int(&component["RequiredSpellLevel"]),
        })),
        "PrerequisiteCharacterLevel" => {
            Some(Node::Leaf(Leaf::CharacterLevel(int(&component["Level"]))))
        }
        "PrerequisiteMythicLevel" => Some(Node::Leaf(Leaf::MythicRank(int(&component["Level"])))),
        // Inference: the game asks for a class with a spellbook of that type (`PrerequisiteCasterType.CheckInternal`),
        // which the planner says as spells of that type of level 1.
        "PrerequisiteCasterType" => Some(Node::Leaf(Leaf::SpellLevel {
            arcane: flag(&component["IsArcane"]),
            spontaneous: false,
            level: 1,
        })),
        "PrerequisiteParametrizedFeature" => {
            out.skip(
                AREA,
                "the value of a parametrized feature, compared as the feature",
            );
            has("m_Feature")
        }
        "PrerequisiteIsPet" => {
            out.skip(
                AREA,
                if flag(&component["Not"]) {
                    "not a pet, which always holds"
                } else {
                    "a pet's prerequisite"
                },
            );
            return None;
        }
        "PrerequisiteMainCharacter"
        | "PrerequisiteEtude"
        | "PrerequisitePet"
        | "PrerequisitePlayerHasFeature"
        | "PrerequisiteConditionForWeaponCategory" => {
            out.skip(AREA, format!("{kind}, the story's or a pet's"));
            return None;
        }
        other => {
            out.skip(AREA, format!("{other}, not compared"));
            return None;
        }
    };
    if node.is_none() {
        out.skip(AREA, "a prerequisite with an empty reference");
    }
    node
}

/// `AlignmentMaskType` from its names, as the JSON writes flags: `"NeutralGood, TrueNeutral, NeutralEvil, Chaotic"`.
fn alignment_mask(names: &str) -> Option<u16> {
    names.split(", ").try_fold(0, |mask, name| {
        let bits = match name.trim() {
            "LawfulGood" => 0x1,
            "NeutralGood" => 0x2,
            "ChaoticGood" => 0x4,
            "LawfulNeutral" => 0x8,
            "TrueNeutral" => 0x10,
            "ChaoticNeutral" => 0x20,
            "LawfulEvil" => 0x40,
            "NeutralEvil" => 0x80,
            "ChaoticEvil" => 0x100,
            "Good" => 0x7,
            "Evil" => 0x1c0,
            "Lawful" => 0x49,
            "Chaotic" => 0x124,
            "Any" => 0x1ff,
            "None" => 0,
            _ => return None,
        };
        Some(mask | bits)
    })
}

/// An alignment's bit in `AlignmentMaskType`, whose order is `Alignment::ALL`'s.
fn alignment_bit(alignment: Alignment) -> u16 {
    let index = Alignment::ALL
        .iter()
        .position(|each| *each == alignment)
        .unwrap_or(0);
    1 << index
}

/// The planner's prerequisite, before the normal form.
fn planner_side(ctx: &Ctx, prerequisite: &Prerequisite, out: &mut Out) -> Option<Node> {
    let has = |at: DataAt, out: &mut Out| match ctx.planner.guid(at) {
        Some(guid) => Some(Node::Leaf(Leaf::Has(guid.clone()))),
        None => {
            out.skip(AREA, "a planner prerequisite on an entry with no blueprint");
            None
        }
    };
    let leaf = |leaf: Leaf| Some(Node::Leaf(leaf));
    let children = |children: &[Prerequisite], out: &mut Out| -> Vec<Node> {
        children
            .iter()
            .filter_map(|child| planner_side(ctx, child, out))
            .collect()
    };
    match prerequisite {
        Prerequisite::All(inner) => Some(Node::All(children(inner, out))),
        Prerequisite::Any(inner) => match whole_selection(ctx, inner) {
            Some(guid) => Some(Node::Leaf(Leaf::Has(guid.clone()))),
            None => Some(Node::Any(children(inner, out))),
        },
        Prerequisite::AtLeast(count, inner) => {
            Some(Node::AtLeast(i64::from(*count), children(inner, out)))
        }
        Prerequisite::Not(inner) => {
            planner_side(ctx, inner, out).map(|node| Node::Not(Box::new(node)))
        }
        Prerequisite::Feat(feat) => has(DataAt::Feat(feat.clone()), out),
        Prerequisite::FeatFor { feat, value } => match per_skill_feature(ctx, feat, value) {
            Some(guid) => leaf(Leaf::Has(guid)),
            None => {
                out.skip(
                    AREA,
                    "the value of a parametrized feature, compared as the feature",
                );
                has(DataAt::Feat(feat.clone()), out)
            }
        },
        Prerequisite::MythicFeat(feat) => has(DataAt::MythicFeat(feat.clone()), out),
        Prerequisite::MythicAbility(ability) => has(DataAt::MythicAbility(ability.clone()), out),
        Prerequisite::SelectionOption(option) => has(DataAt::SelectionOption(option.clone()), out),
        Prerequisite::Race(race) => has(DataAt::Race(race.clone()), out),
        Prerequisite::Deity(deity) => has(DataAt::Deity(deity.clone()), out),
        Prerequisite::Feature { feature, count } => {
            if *count > 1 {
                out.skip(AREA, "a feature's count, compared as the feature");
            }
            has(DataAt::Feature(feature.clone()), out)
        }
        Prerequisite::ClassLevel { class, level } => {
            match ctx.planner.guid(DataAt::Class(class.clone())) {
                Some(guid) => leaf(Leaf::ClassLevel(guid.clone(), i64::from(*level))),
                None => {
                    out.skip(AREA, "a planner prerequisite on an entry with no blueprint");
                    None
                }
            }
        }
        Prerequisite::Archetype(archetype) => {
            match ctx.planner.guid(DataAt::Archetype(archetype.clone())) {
                Some(guid) => leaf(Leaf::Archetype(guid.clone())),
                None => {
                    out.skip(AREA, "a planner prerequisite on an entry with no blueprint");
                    None
                }
            }
        }
        Prerequisite::SkillRanks { skill, ranks } => {
            leaf(Leaf::Ranks(skill_stat(skill), i64::from(*ranks)))
        }
        Prerequisite::AbilityScore { ability, score } => leaf(Leaf::Ability(
            ability_stat(*ability).to_owned(),
            i64::from(*score),
        )),
        Prerequisite::BaseAttackBonus(bab) => leaf(Leaf::Bab(i64::from(*bab))),
        Prerequisite::SpellLevel { spell_type, level } => leaf(Leaf::SpellLevel {
            arcane: *spell_type == SpellType::Arcane,
            spontaneous: false,
            level: i64::from(*level),
        }),
        Prerequisite::SpontaneousSpellLevel { spell_type, level } => leaf(Leaf::SpellLevel {
            arcane: *spell_type == SpellType::Arcane,
            spontaneous: true,
            level: i64::from(*level),
        }),
        Prerequisite::CharacterLevel(level) => leaf(Leaf::CharacterLevel(i64::from(*level))),
        Prerequisite::MythicRank(rank) => leaf(Leaf::MythicRank(i64::from(*rank))),
        Prerequisite::Alignment(alignment) => leaf(Leaf::Alignments(alignment_bit(*alignment))),
        other => {
            out.skip(
                AREA,
                format!(
                    "the planner's {}, which has no game form",
                    planner_kind(other)
                ),
            );
            None
        }
    }
}

fn planner_kind(prerequisite: &Prerequisite) -> &'static str {
    match prerequisite {
        Prerequisite::FeatForSameValue(_) => "FeatForSameValue",
        Prerequisite::FeatIfValue { .. } => "FeatIfValue",
        Prerequisite::NoOptionFrom(_) => "NoOptionFrom",
        Prerequisite::NoOtherOptionOfGroup(_) => "NoOtherOptionOfGroup",
        Prerequisite::PickedThrough(_) => "PickedThrough",
        Prerequisite::CasterLevel(_) => "CasterLevel",
        Prerequisite::Atheist => "Atheist",
        Prerequisite::Worships => "Worships",
        Prerequisite::DeityOffers(_) => "DeityOffers",
        Prerequisite::Story(_) => "Story",
        _ => "prerequisite",
    }
}

fn render_leaf(ctx: &Ctx, negated: bool, leaf: &Leaf) -> String {
    let text = match leaf {
        Leaf::Has(guid) => ctx.name(guid),
        Leaf::Ability(stat, value) => format!("{stat} {value}"),
        Leaf::Ranks(stat, value) => format!("{stat} ranks {value}"),
        Leaf::Bab(value) => format!("BAB {value}"),
        Leaf::ClassLevel(class, level) => format!("{} level {level}", ctx.name(class)),
        Leaf::Archetype(archetype) => format!("archetype {}", ctx.name(archetype)),
        Leaf::Alignments(mask) => {
            let names: Vec<String> = Alignment::ALL
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1 << index) != 0)
                .map(|(_, alignment)| format!("{alignment:?}"))
                .collect();
            format!("alignment {}", names.join(" or "))
        }
        Leaf::SpellLevel {
            arcane,
            spontaneous,
            level,
        } => format!(
            "{} spells of level {level}{}",
            if *arcane { "arcane" } else { "divine" },
            if *spontaneous { ", spontaneously" } else { "" }
        ),
        Leaf::CharacterLevel(level) => format!("character level {level}"),
        Leaf::MythicRank(rank) => format!("mythic rank {rank}"),
    };
    if negated { format!("not {text}") } else { text }
}

fn render(ctx: &Ctx, node: &Node) -> String {
    let list = |children: &[Node]| {
        children
            .iter()
            .map(|child| render(ctx, child))
            .collect::<Vec<_>>()
            .join("; ")
    };
    match node {
        Node::Leaf(leaf) => render_leaf(ctx, false, leaf),
        Node::Not(child) => format!("not ({})", render(ctx, child)),
        Node::All(children) => format!("all of ({})", list(children)),
        Node::Any(children) => format!("one of ({})", list(children)),
        Node::AtLeast(count, children) => format!("{count} of ({})", list(children)),
    }
}

/// The game's feature for a feat taken for one skill, where the game has one per skill: the option of the feat's
/// selection that adds to that skill (`SkillFocusSelection`'s `SkillFocusStealth` adds to `SkillStealth`).
fn per_skill_feature(ctx: &Ctx, feat: &rules::FeatKey, value: &Parameter) -> Option<BlueprintGuid> {
    let Parameter::Skill(skill) = value else {
        return None;
    };
    let stat = skill_stat(skill);
    let selection = ctx.planner.guid(DataAt::Feat(feat.clone()))?;
    references(ctx.dump.json(selection)?.get("m_AllFeatures"))
        .into_iter()
        .find(|option| {
            ctx.dump.json(option).is_some_and(|json| {
                components(json).iter().any(|part| {
                    matches!(component_kind(part), "AddContextStatBonus" | "AddStatBonus")
                        && part["Stat"].as_str() == Some(stat.as_str())
                })
            })
        })
}

/// A selection all of whose options an `Any` names, when it has a blueprint: the game's prerequisite on such a
/// selection is the selection itself (`ExtraDiscoverySelection` asks for `DiscoverySelection`), where the planner
/// lists its options.
fn whole_selection<'a>(ctx: &Ctx<'a>, inner: &[Prerequisite]) -> Option<&'a BlueprintGuid> {
    let keys: Vec<&rules::SelectionOptionKey> = inner
        .iter()
        .map(|each| match each {
            Prerequisite::SelectionOption(key) => Some(key),
            _ => None,
        })
        .collect::<Option<_>>()?;
    let first = keys.first()?;
    let selection = ctx
        .planner
        .data
        .selections()
        .find(|selection| selection.options.iter().any(|option| option.key == **first))?;
    let named: std::collections::BTreeSet<&rules::SelectionOptionKey> = keys.into_iter().collect();
    (named == selection.offered()).then_some(())?;
    selection.blueprint.as_ref()
}
