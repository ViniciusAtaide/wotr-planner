//! Reading a declared Build: each address's value set straight into the Build's fields, and what the walk can't see
//! refused on the way, each refusal a root of the tree while the rest is still walked.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Ability, AbilityScores, Address, Alignment, ArchetypeKey, BackgroundKey, Build, ClassKey,
    Faith, FeatKey, GameData, LevelUp, MythicAbilityKey, MythicExtraPick, MythicFeatKey,
    MythicPathKey, MythicPickKind, Parameter, Pick, PointBuy, RaceKey, SelectionOptionKey,
    SkillKey, SlotId, SubraceKey, has_ability_increase, mythic_pick_kind,
};

use super::tree::{At, Given, Node, pick_gives};
use super::{Refused, closest};
use crate::planner::Planner;
use crate::shapes::ChoiceValue;

/// The declared Build, and what the declaration refused while building it.
pub(super) struct Declared {
    pub(super) build: Build,
    pub(super) refusals: Vec<Node>,
}

/// A key as given: trimmed, in lower case (keys are lower-case slugs, and `choose` matches them ignoring case).
fn key_text(text: &str) -> String {
    text.trim().to_lowercase()
}

/// What a declaration gathers before it becomes the Build's levels, class choices and picks.
#[derive(Default)]
struct Gathered<'a> {
    scores: BTreeMap<Ability, u8>,
    /// Each level's class.
    classes: BTreeMap<u8, ClassKey>,
    increases: BTreeMap<u8, Ability>,
    /// Each level's ranks by skill, with the address that gave them.
    ranks: BTreeMap<(u8, SkillKey), (u8, Address)>,
    archetypes: BTreeMap<ClassKey, Vec<(Address, ArchetypeKey)>>,
    advances: BTreeMap<ClassKey, Vec<(Address, BTreeSet<ClassKey>)>>,
    racial_ability: Option<Ability>,
    picks: Vec<(SlotId, &'a ChoiceValue)>,
}

impl Declared {
    /// The declared Build from its entries, in whatever order they come: each is read in `Address` order.
    pub(super) fn read<'a>(
        planner: &Planner,
        choices: impl IntoIterator<Item = (&'a String, &'a ChoiceValue)>,
    ) -> Self {
        let mut declared = Self {
            build: Build::default(),
            refusals: Vec::new(),
        };
        let entries = declared.addresses(planner, choices);
        let mut gathered = Gathered::default();
        for (address, value) in &entries {
            declared.entry(planner, &mut gathered, address, value);
        }
        declared.point_buy(&gathered);
        declared.racial_ability(planner, &gathered);
        declared.levels(planner, &gathered);
        declared.class_choices(planner, &gathered);
        for (slot, value) in &gathered.picks {
            declared.pick(planner, slot.clone(), value);
        }
        // In Build order (text that isn't an address first): a refusal under another comes after it.
        declared
            .refusals
            .sort_by(|one, other| one.at.cmp(&other.at));
        declared
    }

    fn refuse(&mut self, node: Node) {
        self.refusals.push(node);
    }

    fn refuse_at(&mut self, address: &Address, refused: Refused, reason: String) {
        self.refuse(Node::refused(address.clone(), refused, reason));
    }

    /// The declared addresses with their values. Text that isn't an address is refused; two spellings of one
    /// address with different values are declared twice, and neither is taken.
    fn addresses<'a>(
        &mut self,
        planner: &Planner,
        choices: impl IntoIterator<Item = (&'a String, &'a ChoiceValue)>,
    ) -> BTreeMap<Address, &'a ChoiceValue> {
        let mut entries: BTreeMap<Address, BTreeMap<&'a str, &'a ChoiceValue>> = BTreeMap::new();
        for (text, value) in choices {
            match text.parse::<Address>() {
                Err(error) => self.refuse(Node::unparsed(text.clone(), error.to_string())),
                Ok(address) => {
                    entries.entry(address).or_default().insert(text, value);
                }
            }
        }
        let mut taken = BTreeMap::new();
        for (address, given) in entries {
            let mut spellings = given.iter();
            let Some((first, value)) = spellings.next() else {
                continue;
            };
            let Some((other, _)) = spellings.find(|(_, other)| *other != value) else {
                taken.insert(address, *value);
                continue;
            };
            let reason = format!(
                "`{first}` and `{other}` both declare `{address}`, with different values: give it once."
            );
            // A slot declared twice would have given either pick.
            let gives = match &address {
                Address::Slot(slot) => given
                    .values()
                    .filter_map(|value| key_of(value))
                    .flat_map(|(key, _)| {
                        let pick = bare_pick(&planner.data, &BTreeMap::new(), slot, key);
                        pick_gives(&planner.data, &pick)
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let node = Node::refused(address, Refused::DeclaredTwice, reason).giving(gives);
            self.refuse(node);
        }
        taken
    }

    /// One declared choice: set on the Build start, or gathered for the levels, class choices and picks.
    fn entry<'a>(
        &mut self,
        planner: &Planner,
        gathered: &mut Gathered<'a>,
        address: &Address,
        value: &'a ChoiceValue,
    ) {
        let data = &planner.data;
        let not_an_ability =
            |key: &str| format!("`{key}` isn't an ability: one of str, dex, con, int, wis, cha.");
        match (address, value) {
            (Address::Race, ChoiceValue::Key(key)) => {
                let key = key_text(key);
                // A subrace's key names its race too.
                let subrace = data.races.iter().find_map(|race| {
                    race.subraces
                        .iter()
                        .find(|subrace| subrace.key.as_str() == key)
                        .map(|subrace| (race.key.clone(), subrace.key.clone()))
                });
                let start = &mut self.build.start;
                match subrace {
                    Some((race, subrace)) => {
                        start.race = Some(race);
                        start.subrace = Some(subrace);
                    }
                    None => start.race = Some(RaceKey::new(key)),
                }
            }
            (Address::Subrace, ChoiceValue::Key(key)) => {
                let key = SubraceKey::new(key_text(key));
                match self.build.start.subrace.clone() {
                    Some(held) if held != key => {
                        let reason = format!(
                            "`start.race` names the subrace `{}` and `start.subrace` `{}`: give one.",
                            held.as_str(),
                            key.as_str()
                        );
                        self.refuse_at(address, Refused::DeclaredTwice, reason);
                    }
                    _ => {
                        let start = &mut self.build.start;
                        if start.race.is_none() {
                            start.race = data
                                .races
                                .iter()
                                .find(|race| race.subraces.iter().any(|s| s.key == key))
                                .map(|race| race.key.clone());
                        }
                        start.subrace = Some(key);
                    }
                }
            }
            (Address::Faith, ChoiceValue::Key(key)) => match Faith::from_key(key) {
                Some(faith) => self.build.start.deity = Some(faith),
                None => {
                    let reason = "An empty key isn't a faith: a deity's key, or `atheism`.".to_owned();
                    self.refuse_at(address, Refused::NotAnOption, reason);
                }
            },
            (Address::Alignment, ChoiceValue::Key(key)) => {
                match Alignment::from_key(key) {
                    Some(alignment) => self.build.start.alignment = Some(alignment),
                    None => {
                        let key = key_text(key);
                        let keys: Vec<&str> = Alignment::ALL.into_iter().map(Alignment::key).collect();
                        let reason =
                            format!("`{key}` isn't an alignment: one of {}.", keys.join(", "));
                        self.refuse_at(address, Refused::NotAnOption, reason);
                    }
                }
            }
            (Address::Background, ChoiceValue::Key(key)) => {
                self.build.start.background = Some(BackgroundKey::new(key_text(key)));
            }
            (Address::RacialAbility, ChoiceValue::Key(key)) => match Ability::from_key(key) {
                Some(ability) => gathered.racial_ability = Some(ability),
                None => {
                    self.refuse_at(address, Refused::NotAnOption, not_an_ability(key))
                }
            },
            (Address::Score(ability), ChoiceValue::Number(score)) => {
                gathered.scores.insert(*ability, *score);
            }
            (
                Address::Class(level),
                ChoiceValue::Class {
                    class,
                    archetype,
                    advances,
                },
            ) => {
                let class = ClassKey::new(key_text(class));
                if let Some(archetype) = archetype {
                    gathered
                        .archetypes
                        .entry(class.clone())
                        .or_default()
                        .push((address.clone(), ArchetypeKey::new(key_text(archetype))));
                }
                if let Some(advances) = advances {
                    let set = advances
                        .iter()
                        .map(|key| ClassKey::new(key_text(key)))
                        .collect();
                    gathered
                        .advances
                        .entry(class.clone())
                        .or_default()
                        .push((address.clone(), set));
                }
                gathered.classes.insert(*level, class);
            }
            (Address::Archetype(class), ChoiceValue::Key(key)) => {
                gathered
                    .archetypes
                    .entry(class.clone())
                    .or_default()
                    .push((address.clone(), ArchetypeKey::new(key_text(key))));
            }
            (Address::Increase(level), ChoiceValue::Key(key)) => match Ability::from_key(key) {
                Some(ability) => {
                    gathered.increases.insert(*level, ability);
                }
                None => {
                    self.refuse_at(address, Refused::NotAnOption, not_an_ability(key))
                }
            },
            (Address::SkillPoints(level), ChoiceValue::Ranks(map)) => {
                for (skill, number) in map {
                    let skill = SkillKey::new(key_text(skill));
                    self.add_ranks(planner, gathered, address, *level, skill, *number);
                }
            }
            (Address::Skill(level, skill), ChoiceValue::Number(number)) => {
                let skill = SkillKey::new(key_text(skill.as_str()));
                self.add_ranks(planner, gathered, address, *level, skill, *number);
            }
            (Address::FirstAscension | Address::Path | Address::Switch, ChoiceValue::Key(key)) => {
                let path = Some(MythicPathKey::new(key_text(key)));
                match address {
                    Address::FirstAscension => self.build.first_ascension = path,
                    Address::Path => self.build.mythic_path = path,
                    _ => self.build.path_switch = path,
                }
            }
            (Address::Slot(slot), ChoiceValue::Key(_) | ChoiceValue::Parameterised { .. }) => {
                gathered.picks.push((slot.clone(), value));
            }
            (Address::Level(level), _) => self.refuse_at(address,
                Refused::NoSuchChoice,
                format!(
                    "`level.{level}` is the level itself, which only `withdraw` takes: a level is declared by its \
                     class, `level.{level}.class`."
                ),
            ),
            (Address::Advances(class), _) => self.refuse_at(address,
                Refused::NoSuchChoice,
                format!(
                    "A class's advanced classes are declared with it: `advances` in the value of a `level.N.class` \
                     that takes `{}`.",
                    class.as_str()
                ),
            ),
            // Gear's choices arrive with ticket 168; until then a declared Build has none, as `options` says.
            (Address::Gear(_), _) => self.refuse_at(address,
                Refused::NoSuchChoice,
                format!("The Build has no choice `{address}`: `sheet` and `todo` print the addresses it has."),
            ),
            (address, _) => {
                let shape = match address {
                    Address::Score(_) | Address::Skill(..) => "a number",
                    Address::Class(_) => "a class: `{class, archetype?, advances?}`",
                    Address::SkillPoints(_) => "a map `{skill: ranks}`",
                    Address::Slot(_) => "a key, or `{key, parameter}`",
                    _ => "a key",
                };
                let reason = format!("`{address}` takes {shape}.");
                self.refuse_at(address, Refused::NotAnOption, reason);
            }
        }
    }

    /// Adds a skill's ranks at a level. The same skill and level declared again with a different number (by
    /// `level.N.skill`'s map and `level.N.skill.<skill>`) is declared twice, and neither is taken.
    fn add_ranks(
        &mut self,
        planner: &Planner,
        gathered: &mut Gathered,
        address: &Address,
        level: u8,
        skill: SkillKey,
        number: u8,
    ) {
        let ranks = &mut gathered.ranks;
        match ranks.get(&(level, skill.clone())) {
            Some((held, by)) if *held != number => {
                let reason = format!(
                    "`{by}` gives {} {held} rank(s) at level {level} and `{address}` {number}: give them once.",
                    planner.engine.skill_name(&skill)
                );
                ranks.remove(&(level, skill.clone()));
                self.refuse_at(
                    &Address::Skill(level, skill),
                    Refused::DeclaredTwice,
                    reason,
                );
            }
            Some(_) => {}
            None => {
                ranks.insert((level, skill), (number, address.clone()));
            }
        }
    }

    /// The point buy: each score from 7 to 18, then the budget. A score outside the range stays 10; scores over the
    /// budget all stay 10, and the rest of the Build is checked with them.
    fn point_buy(&mut self, gathered: &Gathered) {
        let mut scores = AbilityScores::default();
        for (ability, score) in &gathered.scores {
            if PointBuy::default().with(*ability, *score).is_some() {
                *scores.get_mut(*ability) = *score;
                continue;
            }
            let name = ability.name();
            let reason = format!(
                "{name} {score}: a point-buy score runs from 7 to 18. The rest is checked with {name} 10."
            );
            let node = Node::refused(Address::Score(*ability), Refused::FixedRule, reason)
                .giving(vec![Given::Score(*ability)]);
            self.refuse(node);
        }
        match PointBuy::try_from(scores) {
            Ok(point_buy) => self.build.start.ability_scores = point_buy,
            Err(error) => {
                // Named at the highest score.
                let highest = AbilityScores::ABILITIES
                    .into_iter()
                    .enumerate()
                    .max_by_key(|(order, ability)| {
                        (scores.get(*ability), std::cmp::Reverse(*order))
                    })
                    .map_or(Ability::Strength, |(_, ability)| ability);
                let listed: Vec<String> = AbilityScores::ABILITIES
                    .into_iter()
                    .map(|ability| format!("{} {}", ability.key(), scores.get(ability)))
                    .collect();
                let reason = format!(
                    "The point buy ({}) costs more than the {}-point budget: {error}. The rest is checked with \
                     every score 10.",
                    listed.join(", "),
                    rules::POINT_BUY_BUDGET
                );
                let mut node = Node::refused(Address::Score(highest), Refused::FixedRule, reason)
                    .giving(AbilityScores::ABILITIES.map(Given::Score).to_vec());
                node.label = "Point buy".to_owned();
                self.refuse(node);
            }
        }
    }

    /// The racial ability, only for a race (or subrace) that chooses where its bonus goes.
    fn racial_ability(&mut self, planner: &Planner, gathered: &Gathered) {
        let Some(ability) = gathered.racial_ability else {
            return;
        };
        if planner.engine.chooses_racial_ability(&self.build.start) {
            self.build.start.racial_ability = Some(ability);
        } else {
            let reason = "The Build's race doesn't choose where a racial bonus goes (or no race is declared)."
                .to_owned();
            self.refuse_at(&Address::RacialAbility, Refused::NotAnOption, reason);
        }
    }

    /// The levels from 1 with no gap, each with its class, ability increase and skill ranks. A level declared past a
    /// missing one is refused under it; an increase off every 4th level and ranks above the character level are
    /// fixed-rule breaks, left out.
    fn levels(&mut self, planner: &Planner, gathered: &Gathered) {
        let mut level = 1u8;
        while let Some(class) = gathered.classes.get(&level) {
            self.build.levels.push(LevelUp::new(class.clone()));
            level = level.saturating_add(1);
        }
        let last = level - 1;
        // A level missing below a declared one: the Build stops before it.
        let missing = Address::Class(level);
        let beyond: Vec<u8> = gathered.classes.range(level..).map(|(at, _)| *at).collect();
        if !beyond.is_empty() {
            let refused = self
                .refusals
                .iter()
                .any(|node| node.at == At::Address(missing.clone()));
            if !refused {
                let reason = format!(
                    "Level {level} has no class declared, so the Build stops at level {last}: declare \
                     `level.{level}.class`."
                );
                self.refuse_at(&missing, Refused::MissingLevel, reason);
            }
            for later in beyond {
                let reason = format!("The Build stops at level {last}, below level {later}.");
                let node = Node::refused(Address::Class(later), Refused::NotReached, reason)
                    .under(missing.clone());
                self.refuse(node);
            }
        }
        let past =
            |at: u8| format!("The Build has {last} level(s), so it doesn't reach level {at}.");
        for (&at, &ability) in &gathered.increases {
            let address = Address::Increase(at);
            if at > last {
                let node = Node::refused(address, Refused::NotReached, past(at))
                    .giving(vec![Given::Score(ability)])
                    .under(Address::Class(at));
                self.refuse(node);
            } else if !has_ability_increase(at) {
                let reason =
                    format!("Level {at} has no ability increase: only every 4th level does.");
                let node = Node::refused(address, Refused::FixedRule, reason)
                    .giving(vec![Given::Score(ability)]);
                self.refuse(node);
            } else {
                self.build.levels[usize::from(at) - 1].ability_increase = Some(ability);
            }
        }
        let mut totals: BTreeMap<&SkillKey, u8> = BTreeMap::new();
        for ((at, skill), (number, address)) in &gathered.ranks {
            let (at, number) = (*at, *number);
            if number == 0 {
                continue;
            }
            if at > last {
                let node = Node::refused(address.clone(), Refused::NotReached, past(at))
                    .under(Address::Class(at));
                self.refuse(node);
                continue;
            }
            let total = totals
                .get(skill)
                .copied()
                .unwrap_or(0)
                .saturating_add(number);
            if total > at {
                let reason = format!(
                    "{} would have {total} ranks at level {at}: a skill's ranks never go above the character level.",
                    planner.engine.skill_name(skill)
                );
                self.refuse_at(address, Refused::FixedRule, reason);
                continue;
            }
            totals.insert(skill, total);
            self.build.levels[usize::from(at) - 1]
                .skill_ranks
                .insert(skill.clone(), number);
        }
        debug_assert!(self.build.keeps_fixed_rules());
    }

    /// Each class's archetype and advanced classes, the same wherever they're declared; two that differ are declared
    /// twice, and the class is checked without. Every class a level takes gets its choice, as `choose` gives it.
    fn class_choices(&mut self, planner: &Planner, gathered: &Gathered) {
        let engine = &planner.engine;
        for level_up in &self.build.levels {
            self.build
                .class_choices
                .entry(level_up.class.clone())
                .or_default();
        }
        for (class, given) in &gathered.archetypes {
            let distinct: BTreeSet<&ArchetypeKey> = given.iter().map(|(_, key)| key).collect();
            if distinct.len() > 1 {
                let said: Vec<String> = given
                    .iter()
                    .map(|(address, key)| format!("`{}` at `{address}`", key.as_str()))
                    .collect();
                let reason = format!(
                    "The {} has one archetype, and the declaration gives {}: give the same one wherever it's \
                     given. It's checked without one.",
                    engine.class_name(class),
                    said.join(", ")
                );
                let gives = distinct
                    .into_iter()
                    .cloned()
                    .map(Given::Archetype)
                    .collect();
                let address = Address::Archetype(class.clone());
                let node = Node::refused(address, Refused::DeclaredTwice, reason).giving(gives);
                self.refuse(node);
                continue;
            }
            let archetype = given.first().map(|(_, key)| key.clone());
            self.build
                .class_choices
                .entry(class.clone())
                .or_default()
                .archetype = archetype;
        }
        for (class, given) in &gathered.advances {
            let distinct: BTreeSet<&BTreeSet<ClassKey>> =
                given.iter().map(|(_, set)| set).collect();
            if distinct.len() > 1 {
                let said: Vec<String> = given
                    .iter()
                    .map(|(address, set)| {
                        let keys: Vec<&str> = set.iter().map(ClassKey::as_str).collect();
                        format!("[{}] at `{address}`", keys.join(", "))
                    })
                    .collect();
                let reason = format!(
                    "The {} advances one set of classes, and the declaration gives {}: give the same one wherever \
                     it's given. It's checked advancing none.",
                    engine.class_name(class),
                    said.join(", ")
                );
                let address = Address::Advances(class.clone());
                self.refuse_at(&address, Refused::DeclaredTwice, reason);
                continue;
            }
            if let Some((_, set)) = given.first() {
                self.build
                    .class_choices
                    .entry(class.clone())
                    .or_default()
                    .advances = set.clone();
            }
        }
    }

    /// A slot's pick ([`bare_pick`]), with its value read as its item's kind of value. A pick refused here gives
    /// what it would have given, so the picks that need it sit under it.
    fn pick(&mut self, planner: &Planner, slot: SlotId, value: &ChoiceValue) {
        let engine = &planner.engine;
        let Some((key, parameter)) = key_of(value) else {
            return;
        };
        let address = Address::Slot(slot.clone());
        let bare = bare_pick(&planner.data, &self.build.picks, &slot, key);
        let refuse = |refused: Refused, reason: String| {
            Node::refused(address.clone(), refused, reason).giving(pick_gives(&planner.data, &bare))
        };
        let pick = match (parameter, engine.value_kind(&bare)) {
            // A key the data lacks: the walk refuses it, naming the closest.
            (_, None) => bare.clone(),
            (None, Some(Some(kind))) if !kind.may_stay_open() => {
                let reason = format!(
                    "`{}` takes {}: give `{{key, parameter}}` (`options` with `values: true` lists them).",
                    bare.key(),
                    kind.with_article()
                );
                self.refuse(refuse(Refused::NeedsParameter, reason));
                return;
            }
            (None, _) => bare.clone(),
            (Some(text), Some(None)) => {
                let reason = format!(
                    "`{}` takes no value, so not `{text}`: give the key alone.",
                    bare.key()
                );
                self.refuse(refuse(Refused::NotAnOption, reason));
                return;
            }
            (Some(text), Some(Some(kind))) => match engine.parameter_from_key(kind, text) {
                Some(value) => with_parameter(bare.clone(), value),
                None => {
                    let values = engine.parameter_values(kind);
                    let texts: Vec<&str> = values.iter().map(Parameter::value_key).collect();
                    let near = closest(text, texts.iter().map(|text| (*text, *text)));
                    let reason = format!(
                        "`{text}` isn't {} `{}` takes{near}.",
                        kind.with_article(),
                        bare.key()
                    );
                    self.refuse(refuse(Refused::NotAnOption, reason));
                    return;
                }
            },
        };
        self.build.picks.insert(slot, pick);
    }
}

/// A slot's value as its key and the parameter given with it; `None` for a value of another shape.
fn key_of(value: &ChoiceValue) -> Option<(&str, Option<&str>)> {
    match value {
        ChoiceValue::Key(key) => Some((key, None)),
        ChoiceValue::Parameterised { key, parameter } => Some((key, Some(parameter.trim()))),
        _ => None,
    }
}

/// A slot's pick of a key, without a value: the kind of item its slot takes (a feat, an option, a mythic ability or
/// feat by the rank, and for an extra slot a mythic ability after Extra Mythic Ability, the general pick in `picks`
/// at its rank, else a feat).
fn bare_pick(data: &GameData, picks: &BTreeMap<SlotId, Pick>, slot: &SlotId, key: &str) -> Pick {
    let key = key_text(key);
    match *slot {
        SlotId::Feat { .. } | SlotId::OptionFeat { .. } | SlotId::PathFeat { .. } => {
            Pick::feat(FeatKey::new(key))
        }
        SlotId::Selection { .. }
        | SlotId::OptionSelection { .. }
        | SlotId::PathSelection { .. }
        | SlotId::MythicSelection { .. }
        | SlotId::FeatSelection { .. } => Pick::option(SelectionOptionKey::new(key)),
        SlotId::Mythic { rank } => match mythic_pick_kind(rank) {
            MythicPickKind::Ability => Pick::mythic_ability(MythicAbilityKey::new(key)),
            MythicPickKind::Feat => Pick::mythic_feat(MythicFeatKey::new(key)),
        },
        SlotId::MythicExtra { rank } => {
            let extra = match picks.get(&SlotId::Mythic { rank }) {
                Some(Pick::MythicFeat { feat, .. }) => data
                    .mythic_feats
                    .iter()
                    .find(|known| &known.key == feat)
                    .and_then(|feat| feat.extra_pick),
                _ => None,
            };
            match extra {
                Some(MythicExtraPick::MythicAbility) => {
                    Pick::mythic_ability(MythicAbilityKey::new(key))
                }
                _ => Pick::feat(FeatKey::new(key)),
            }
        }
    }
}

/// A pick with its value.
fn with_parameter(pick: Pick, value: Parameter) -> Pick {
    match pick {
        Pick::Feat { feat, .. } => Pick::feat_for(feat, value),
        Pick::SelectionOption { option, .. } => Pick::option_for(option, value),
        Pick::MythicAbility { ability, .. } => Pick::mythic_ability_for(ability, value),
        Pick::MythicFeat { feat, .. } => Pick::mythic_feat_for(feat, value),
    }
}

#[cfg(test)]
mod tests {
    use rules::test_world::world;

    use super::*;

    fn planner() -> Planner {
        Planner::from_data(world()).expect("the test world checks")
    }

    /// What reading gives: the Build, and each refusal's place, code and reason.
    fn read(planner: &Planner, entries: &[(String, ChoiceValue)]) -> (Build, Vec<String>) {
        let declared = Declared::read(planner, entries.iter().map(|(text, value)| (text, value)));
        let refusals = declared
            .refusals
            .iter()
            .map(|node| format!("{} {:?} {}", node.at, node.why, node.reason))
            .collect();
        (declared.build, refusals)
    }

    #[test]
    fn the_entries_read_the_same_in_any_order() {
        let entries: Vec<(String, ChoiceValue)> = [
            ("start.race", ChoiceValue::Key("human".into())),
            ("start.faith", ChoiceValue::Key("atheism".into())),
            ("start.score.str", ChoiceValue::Number(16)),
            ("start.score.Strength", ChoiceValue::Number(17)),
            (
                "level.1.class",
                ChoiceValue::Class {
                    class: "fighter".into(),
                    archetype: None,
                    advances: None,
                },
            ),
            (
                "level.2.class",
                ChoiceValue::Class {
                    class: "fighter".into(),
                    archetype: None,
                    advances: None,
                },
            ),
            (
                "level.1.skill",
                ChoiceValue::Ranks(BTreeMap::from([("athletics".into(), 1)])),
            ),
            ("level.1.skill.athletics", ChoiceValue::Number(1)),
            ("level.2.skill.athletics", ChoiceValue::Number(3)),
            ("level.3.increase", ChoiceValue::Key("str".into())),
            (
                "slot.feat.1.class.0",
                ChoiceValue::Key("power-attack".into()),
            ),
            (
                "slot.feat.1.general.0",
                ChoiceValue::Key("weapon-focus".into()),
            ),
            ("slot.feat.1.race.0", ChoiceValue::Key("cleave".into())),
            ("not.an.address", ChoiceValue::Key("x".into())),
        ]
        .into_iter()
        .map(|(text, value)| (text.to_owned(), value))
        .collect();
        let planner = planner();
        let forward = read(&planner, &entries);
        let reversed: Vec<_> = entries.iter().rev().cloned().collect();
        let backward = read(&planner, &reversed);
        // Some of each: what's set, and what's refused (declared twice, a fixed rule, a missing value).
        assert!(!forward.0.picks.is_empty(), "{forward:?}");
        assert!(forward.1.len() >= 4, "{forward:?}");
        assert_eq!(forward, backward);
        // Every rotation too.
        for start in 1..entries.len() {
            let rotated: Vec<_> = entries[start..]
                .iter()
                .chain(&entries[..start])
                .cloned()
                .collect();
            assert_eq!(read(&planner, &rotated), forward, "starting at {start}");
        }
    }
}
