//! `sheet`: a Build as the engine's sheet has it, level by level and rank by rank, each choice by its address.
//! Everything comes from [`Engine::evaluate`]'s [`Sheet`] and the engine's labels; the only work here is saying
//! what each level adds (its feats and features: the state at its end against the one before). A Build with gear
//! also gets its items, the choices only they allow, and the numbers without them where they differ
//! ([`Engine::naked_sheet`]).

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    AbilityScores, Address, Build, CharacterState, Choice, Engine, POINT_BUY_BUDGET, RankStatus,
    Sheet, Slot, SpellTraits, WeaponKey, has_ability_increase,
};

use crate::attack;
use crate::caster;
use crate::code::envelope;
use crate::error::ToolError;
use crate::gear::{dependent_lines, gear_lines, if_differs};
use crate::planner::Planner;
use crate::shapes::ChoiceValue;
use crate::shapes::{
    Gained, LevelLine, NakedLevel, NakedNumbers, Named, PickLine, RankLine, RankState, Scores,
    SheetLevel, SheetResult, SkillPointsLine, SpellLevels, level_rows,
};
use crate::totals;
use crate::unreached::{Unreached, no_such_level};

/// What a `sheet` call asks for besides the numbers' weapon and spell: which levels and ranks, and whether the whole
/// list of open choices comes along.
#[derive(Clone, Debug, Default)]
pub(crate) struct Asked {
    /// The character levels to give (each in full), sorted and once each; `None` for the whole sheet.
    pub levels: Option<Vec<u8>>,
    /// The Mythic ranks to give, sorted and once each; `None` for every rank.
    pub ranks: Option<Vec<u8>>,
    /// Whether `todo` comes with the sheet.
    pub todo: bool,
}

impl Asked {
    /// What the arguments ask: each list sorted and without repeats, each entry one the Build has (the first that
    /// isn't, in the order given, is refused).
    pub(crate) fn of(
        engine: &Engine,
        build: &Build,
        levels: Option<&[u8]>,
        ranks: Option<&[u8]>,
        todo: Option<bool>,
    ) -> Result<Self, ToolError> {
        let levels_in_build = build.levels.len();
        let bad_level = levels.and_then(|levels| {
            (levels.iter()).find(|&&level| level == 0 || usize::from(level) > levels_in_build)
        });
        if let Some(&level) = bad_level {
            return Err(ToolError::invalid_arguments(format!(
                "`levels`: {}",
                no_such_level(level, levels_in_build)
            )));
        }
        // A rank the Build doesn't reach has its `not_reached` row: only a rank that doesn't exist is refused.
        for &rank in ranks.unwrap_or_default() {
            if Unreached::of(engine, build, rank) == Some(Unreached::NoSuchRank) {
                return Err(ToolError::invalid_arguments(format!(
                    "`ranks`: {}",
                    Unreached::no_such_rank(rank)
                )));
            }
        }
        let sorted = |list: Option<&[u8]>| {
            list.map(|list| {
                list.iter()
                    .copied()
                    .collect::<BTreeSet<u8>>()
                    .into_iter()
                    .collect()
            })
        };
        Ok(Self {
            levels: sorted(levels),
            ranks: sorted(ranks),
            todo: todo == Some(true),
        })
    }
}

/// The sheet of a possible Build; with `removable`, each held choice marked with whether it can be withdrawn now;
/// each level's attack line for the `weapon` given, or else for the Build's Weapon Focus weapon
/// ([`Engine::default_attack_weapon`]), none without one; with a `caster` spell, each level's caster line for it.
/// Every number counts the Build's gear; `naked` gives those the gear changes without it.
fn full_sheet(
    planner: &Planner,
    build: &Build,
    removable: bool,
    weapon: Option<WeaponKey>,
    caster: Option<SpellTraits>,
) -> SheetResult<LevelLine> {
    let engine = &planner.engine;
    let evaluated = engine.evaluate(build);
    let mut result = sheet_of(planner, build, &evaluated, removable);
    let weapon = attack::weapon_or_default(engine, weapon.as_ref(), &evaluated);
    if let Some(weapon) = &weapon {
        for (line, level) in result.levels.iter_mut().zip(&evaluated.levels) {
            line.attack = attack::attack_line(engine, &level.state, weapon);
        }
        result.attack_leaves_out = Some(attack::leaves_out());
    }
    if let Some(spell) = caster {
        for (line, level) in result.levels.iter_mut().zip(&evaluated.levels) {
            line.caster = caster::caster_line(engine, level, spell);
        }
        result.caster_leaves_out = Some(caster::leaves_out());
    }
    if let Some(naked) = engine.naked_sheet(build) {
        result.naked = naked_numbers(engine, &result, &naked, weapon.as_ref(), caster);
    }
    result
}

/// [`full_sheet`] as the `sheet` tool gives it: `todo` only when asked, only the rows asked for, and on a full sheet
/// each level row leaving out what is the same as on the row before ([`level_rows`]); the levels asked for come in
/// full. The numbers read the whole Build.
pub(crate) fn sheet(
    planner: &Planner,
    build: &Build,
    removable: bool,
    weapon: Option<WeaponKey>,
    caster: Option<SpellTraits>,
    asked: &Asked,
) -> SheetResult<SheetLevel> {
    let mut result = full_sheet(planner, build, removable, weapon, caster);
    if !asked.todo {
        result.todo = None;
    }
    // Given either list, the sheet holds only the rows asked for: none of the section whose list is left out.
    let narrow = asked.levels.is_some() || asked.ranks.is_some();
    if narrow {
        let wanted = asked.levels.as_deref().unwrap_or_default();
        result.levels.retain(|line| wanted.contains(&line.level));
        if let Some(naked) = &mut result.naked {
            naked.levels.retain(|line| wanted.contains(&line.level));
            if naked.levels.is_empty() && naked.totals.is_none() {
                result.naked = None;
            }
        }
        let wanted = asked.ranks.as_deref().unwrap_or_default();
        result.ranks.retain(|line| wanted.contains(&line.rank));
    }
    result.map_levels(|lines| level_rows(lines, !narrow))
}

/// The numbers of the sheet without the Build's gear (`naked`, from [`Engine::naked_sheet`]), only where they differ
/// from the sheet's: each level's scores, attack line and caster line, and the last level's totals. `None` when the
/// gear changes none.
fn naked_numbers(
    engine: &Engine,
    result: &SheetResult<LevelLine>,
    naked: &Sheet,
    weapon: Option<&WeaponKey>,
    caster: Option<SpellTraits>,
) -> Option<NakedNumbers> {
    let levels: Vec<NakedLevel> = (result.levels.iter())
        .zip(&naked.levels)
        .filter_map(|(line, level)| {
            let numbers = NakedLevel {
                level: line.level,
                scores: if_differs(Some(&line.scores), Some(scores(level.state.ability_scores))),
                attack: if_differs(
                    line.attack.as_ref(),
                    weapon.and_then(|weapon| attack::attack_line(engine, &level.state, weapon)),
                ),
                caster: if_differs(
                    line.caster.as_ref(),
                    caster.and_then(|spell| caster::caster_line(engine, level, spell)),
                ),
            };
            let any =
                numbers.scores.is_some() || numbers.attack.is_some() || numbers.caster.is_some();
            any.then_some(numbers)
        })
        .collect();
    let totals = if_differs(
        result.totals.as_ref(),
        naked
            .levels
            .last()
            .map(|level| totals::totals(engine, level)),
    );
    (!levels.is_empty() || totals.is_some()).then_some(NakedNumbers { levels, totals })
}

/// [`sheet`] from the Build's sheet already evaluated (`compare` reads the engine's sheet once per Build).
pub(crate) fn sheet_of(
    planner: &Planner,
    build: &Build,
    sheet: &Sheet,
    removable: bool,
) -> SheetResult<LevelLine> {
    let engine = &planner.engine;
    let removable = removable.then(|| engine.removable(build));
    let gear_dependent = dependent_lines(engine, build, sheet);
    let lines = Lines {
        engine,
        removable: removable.as_ref(),
        with_gear: (gear_dependent.iter())
            .map(|line| (line.choice.clone(), line.needs.clone()))
            .collect(),
    };
    let gear = gear_lines(engine, build, removable.as_ref());
    let empty = CharacterState::default();
    let last = sheet.levels.last().map_or(&empty, |level| &level.state);
    let envelope = envelope(engine, build, sheet);
    SheetResult {
        build: envelope.build,
        link: envelope.link,
        todo: Some(envelope.todo),
        name: build.name.clone(),
        start: lines.start(build),
        point_buy_spent: build.start.ability_scores.cost(),
        point_buy_budget: POINT_BUY_BUDGET,
        levels: lines.levels(build, sheet),
        ranks: sheet
            .ranks
            .iter()
            .map(|rank| lines.rank(rank, sheet))
            .collect(),
        level_cap: sheet.level_cap,
        caster_level: last.caster_level,
        spell_levels: SpellLevels {
            arcane: last.arcane_spell_level,
            divine: last.divine_spell_level,
        },
        class_skills: last
            .class_skills
            .iter()
            .map(|skill| named(skill, engine.skill_name(skill)))
            .collect(),
        totals: sheet
            .levels
            .last()
            .map(|level| totals::totals(engine, level)),
        totals_leave_out: totals::leaves_out(),
        notes: notes(engine, build, last),
        removable: removable.map(|choices| {
            choices
                .iter()
                .map(Address::from)
                .filter(withdrawable)
                .map(|address| address.to_string())
                .collect()
        }),
        attack_leaves_out: None,
        caster_leaves_out: None,
        gear,
        gear_dependent,
        naked: None,
    }
}

/// Whether `withdraw` takes back the choice at an address at all: not a level's class, which `choose` changes (the
/// engine would withdraw it by giving the level a class no data has). The racial ability has no engine choice, so
/// the engine never lists it.
fn withdrawable(address: &Address) -> bool {
    !matches!(address, Address::Class(_))
}

fn named(key: &impl std::fmt::Display, name: &str) -> Named {
    Named {
        key: key.to_string(),
        name: name.to_owned(),
    }
}

/// What the planner notes beside the Build: the background's note, the favoured weapons the character is proficient
/// with, and the choices the game asks for again each day.
fn notes(engine: &Engine, build: &Build, last: &CharacterState) -> Vec<String> {
    let mut notes = Vec::new();
    if let Some(background) = &build.start.background {
        let note = engine.background_note(background);
        if !note.is_empty() {
            notes.push(format!("{}: {note}", engine.background_name(background)));
        }
    }
    notes.extend(engine.favoured_weapons_note(last));
    let daily = engine.daily_choices(last);
    let daily: Vec<&str> = daily
        .iter()
        .map(|feature| engine.feature_name(feature))
        .collect();
    if !daily.is_empty() {
        notes.push(format!("Chosen each day in the game: {}", daily.join(", ")));
    }
    notes
}

/// Writes the sheet's lines, marking held choices when asked.
struct Lines<'a> {
    engine: &'a Engine,
    removable: Option<&'a BTreeSet<Choice>>,
    /// The note of each choice only the gear allows, by address.
    with_gear: BTreeMap<String, String>,
}

impl Lines<'_> {
    /// A choice's line, labelled as the engine labels it ([`Engine::choice_label`]), with the sheet's slot at a slot
    /// address: open when `held` is `None`.
    fn line(
        &self,
        address: Address,
        slot: Option<&Slot>,
        held: Option<(ChoiceValue, String)>,
    ) -> PickLine {
        let label = self.engine.choice_label(&address, slot);
        let removable = match (&held, self.removable, address.choice()) {
            (Some(_), Some(removable), Some(choice)) if withdrawable(&address) => {
                Some(removable.contains(&choice))
            }
            _ => None,
        };
        let (value, name) = held.unzip();
        // A number's name only repeats its digits, which `value` says.
        let name = name.filter(|name| {
            !matches!(&value, Some(ChoiceValue::Number(number)) if *name == number.to_string())
        });
        let choice = address.to_string();
        PickLine {
            with_gear: self.with_gear.get(&choice).cloned(),
            choice,
            label,
            value,
            name,
            removable,
        }
    }

    fn slot(&self, slot: &Slot) -> PickLine {
        let held = slot
            .pick
            .as_ref()
            .map(|pick| (ChoiceValue::pick(pick), self.engine.pick_name(pick)));
        self.line(Address::Slot(slot.id.clone()), Some(slot), held)
    }

    fn start(&self, build: &Build) -> Vec<PickLine> {
        let engine = self.engine;
        let start = &build.start;
        let mut lines = vec![self.line(
            Address::Race,
            None,
            start.race.as_ref().map(|race| {
                let name = match &start.subrace {
                    Some(subrace) => engine.subrace_name(subrace),
                    None => engine.race_name(race),
                };
                (
                    ChoiceValue::race(race, start.subrace.as_ref()),
                    name.to_owned(),
                )
            }),
        )];
        if let Some(subrace) = &start.subrace {
            lines.push(self.line(
                Address::Subrace,
                None,
                Some((
                    ChoiceValue::Key(subrace.as_str().to_owned()),
                    engine.subrace_name(subrace).to_owned(),
                )),
            ));
        }
        if engine.chooses_racial_ability(start) {
            lines.push(
                self.line(
                    Address::RacialAbility,
                    None,
                    start
                        .racial_ability
                        .map(|ability| (ChoiceValue::ability(ability), ability.name().to_owned())),
                ),
            );
        }
        lines.push(self.line(
            Address::Faith,
            None,
            start.deity.as_ref().map(|faith| {
                (
                    ChoiceValue::Key(faith.key().to_owned()),
                    engine.faith_name(faith).to_owned(),
                )
            }),
        ));
        lines.push(self.line(
            Address::Alignment,
            None,
            start.alignment.map(|alignment| {
                (
                    ChoiceValue::Key(alignment.key().to_owned()),
                    alignment.name().to_owned(),
                )
            }),
        ));
        lines.push(self.line(
            Address::Background,
            None,
            start.background.as_ref().map(|background| {
                (
                    ChoiceValue::Key(background.as_str().to_owned()),
                    engine.background_name(background).to_owned(),
                )
            }),
        ));
        for ability in AbilityScores::ABILITIES {
            let score = start.ability_scores.get(ability);
            lines.push(self.line(
                Address::Score(ability),
                None,
                Some((ChoiceValue::Number(score), score.to_string())),
            ));
        }
        lines
    }

    fn levels(&self, build: &Build, sheet: &Sheet) -> Vec<LevelLine> {
        let engine = self.engine;
        let empty = CharacterState::default();
        let mut before = &empty;
        let mut lines = Vec::new();
        for (level_up, level) in build.levels.iter().zip(&sheet.levels) {
            let n = level.character_level;
            let state = &level.state;
            let archetype = state
                .classes
                .iter()
                .find(|taken| taken.class == level.class)
                .and_then(|taken| taken.archetype.as_ref());
            let advances = engine.asks_advances(build, n, &level.class).then(|| {
                build
                    .class_choices
                    .get(&level.class)
                    .map(|choice| choice.advances.clone())
                    .unwrap_or_default()
            });
            let mut picks = vec![self.line(
                Address::Class(n),
                None,
                Some((
                    ChoiceValue::class(&level.class, archetype, advances.as_ref()),
                    engine.class_value_name(&level.class, archetype, advances.as_ref()),
                )),
            )];
            if has_ability_increase(n) {
                picks.push(
                    self.line(
                        Address::Increase(n),
                        None,
                        level_up.ability_increase.map(|ability| {
                            (ChoiceValue::ability(ability), ability.increase_name())
                        }),
                    ),
                );
            }
            for (skill, ranks) in &level_up.skill_ranks {
                let name = engine.skill_name(skill);
                picks.push(self.line(
                    Address::Skill(n, skill.clone()),
                    None,
                    Some((ChoiceValue::Number(*ranks), format!("{name} +{ranks}"))),
                ));
            }
            picks.extend(level.slots.iter().map(|slot| self.slot(slot)));
            lines.push(LevelLine {
                level: n,
                class: named(&level.class, engine.class_name(&level.class)),
                archetype: archetype.map(|key| named(key, engine.archetype_name(key))),
                class_level: level.class_level,
                bab: state.bab,
                fort: state.fort,
                reflex: state.reflex,
                will: state.will,
                scores: scores(state.ability_scores),
                skill_points: SkillPointsLine {
                    available: level.skill_points.available,
                    spent: level.skill_points.spent,
                },
                skill_ranks: state
                    .skill_ranks
                    .iter()
                    .filter(|(_, ranks)| **ranks > 0)
                    .map(|(skill, ranks)| (skill.as_str().to_owned(), *ranks))
                    .collect(),
                feats: new_feats(engine, before, state),
                features: new_features(engine, before, state),
                mythic_ranks: level.mythic_ranks.clone(),
                attack: None,
                caster: None,
                picks,
            });
            before = state;
        }
        lines
    }

    fn rank(&self, rank: &rules::RankSheet, sheet: &Sheet) -> RankLine {
        let engine = self.engine;
        // The state at the rank's level names a feature for the Build's own path.
        let state = sheet.at(rank.character_level);
        RankLine {
            rank: rank.rank,
            character_level: rank.character_level,
            // Only a rank that is taken somewhere: one a Legend's switch ended the ascensions before shows the
            // table's level and no move, though the switch's own rank (never reached as a rank) says where it is.
            table_level: engine
                .rank_table_level(rank.rank)
                .filter(|earliest| rank.character_level > *earliest)
                .filter(|_| {
                    rank.status != RankStatus::NotReached || rank.rank == rules::PATH_SWITCH_RANK
                }),
            status: match rank.status {
                RankStatus::Reached => RankState::Reached,
                RankStatus::PartlyDropped => RankState::PartlyDropped,
                RankStatus::Dropped => RankState::Dropped,
                RankStatus::NotReached => RankState::NotReached,
            },
            path: rank
                .path
                .as_ref()
                .map(|path| named(path, engine.path_name(path))),
            picks: rank.slots.iter().map(|slot| self.slot(slot)).collect(),
            features: {
                // A row that lists a feature twice gives it twice: one entry, `times` 2.
                let mut counted: Vec<(&rules::FeatureKey, usize)> = Vec::new();
                for feature in engine.rank_features(rank) {
                    match counted.iter_mut().find(|(seen, _)| *seen == feature) {
                        Some((_, times)) => *times += 1,
                        None => counted.push((feature, 1)),
                    }
                }
                (counted.into_iter())
                    .map(|(feature, times)| {
                        let name = match state {
                            Some(state) => engine.feature_name_for(feature, state),
                            None => engine.feature_name_apart(feature),
                        };
                        gained(feature, &name, times, None)
                    })
                    .collect()
            },
            dropped: rank
                .dropped_slots
                .iter()
                .map(|slot| Address::Slot(slot.clone()).to_string())
                .collect(),
        }
    }
}

pub(crate) fn scores(scores: AbilityScores) -> Scores {
    Scores {
        str: scores.strength,
        dex: scores.dexterity,
        con: scores.constitution,
        int: scores.intelligence,
        wis: scores.wisdom,
        cha: scores.charisma,
    }
}

/// The feats held at the end of a level and not before it: a feat taken for nothing in particular once, a
/// parameterised one once per new value ("Weapon Focus (Longsword)").
fn new_feats(engine: &Engine, before: &CharacterState, after: &CharacterState) -> Vec<Named> {
    let mut feats = Vec::new();
    for feat in after.feats.keys() {
        let name = engine.feat_name(feat);
        match after.feat_values.get(feat) {
            Some(values) => {
                for value in values.keys() {
                    if !before.has_feat_for(feat, value) {
                        let value = engine.parameter_name(value);
                        feats.push(named(feat, &format!("{name} ({value})")));
                    }
                }
            }
            None if !before.feats.contains_key(feat) => feats.push(named(feat, name)),
            None => {}
        }
    }
    feats
}

/// The features granted at a level: each held more times at its end than before it, with how many more when that is
/// more than one, and its rank when that is more than one and more than it gained there; and each already held before
/// the level whose level reached one of its steps there, with its step (`times` left out when it gained no grant). A
/// feature granted at the level is gained with no `step`, whatever steps its level has already passed.
fn new_features(engine: &Engine, before: &CharacterState, after: &CharacterState) -> Vec<Gained> {
    after
        .features
        .iter()
        .filter_map(|(feature, times)| {
            let more = times.saturating_sub(before.feature_count(feature));
            let held_before = before.feature_count(feature) > 0;
            let stepped = held_before
                && engine.steps_reached(after, feature) > engine.steps_reached(before, feature);
            (more > 0 || stepped).then(|| Gained {
                step: stepped.then(|| engine.feature_step(after, feature)),
                ..gained(
                    feature,
                    &engine.feature_name_for(feature, after),
                    more.into(),
                    Some(*times),
                )
            })
        })
        .collect()
}

/// A feature gained `times` times at once, its rank after `rank` when known: left out for 1, and when it is `times`
/// (all its grants come at once, which `times` says).
fn gained(key: &impl std::fmt::Display, name: &str, times: usize, rank: Option<u8>) -> Gained {
    Gained {
        key: key.to_string(),
        name: name.to_owned(),
        times: u8::try_from(times).ok().filter(|times| *times > 1),
        rank: rank.filter(|rank| *rank > 1 && usize::from(*rank) != times),
        step: None,
    }
}

#[cfg(test)]
mod tests {
    use rmcp::serde_json::{Value, to_value};
    use rules::Ability;
    use rules::test_world::{build, class_slot, pick_feat, pick_feat_for, times, weapon, world};

    use super::*;

    /// A value without its empty arrays and maps, which a row leaves out.
    fn without_empties(value: Value) -> Value {
        match value {
            Value::Array(items) => Value::Array(items.into_iter().map(without_empties).collect()),
            Value::Object(fields) => Value::Object(
                fields
                    .into_iter()
                    .map(|(key, value)| (key, without_empties(value)))
                    .filter(|(_, value)| !matches!(value, Value::Array(a) if a.is_empty()))
                    .filter(|(_, value)| !matches!(value, Value::Object(o) if o.is_empty()))
                    .collect(),
            ),
            other => other,
        }
    }

    /// Rows with no trimming are the full lines, but for what a row leaves out when empty: a field the rows drop in
    /// both modes shows here.
    #[test]
    fn rows_without_trimming_say_all_the_full_lines_say() {
        let planner = Planner::from_data(world()).expect("the test world checks");
        let mut fighter = build("human", &times("fighter", 5));
        fighter
            .picks
            .insert(class_slot(1, 0), pick_feat("power-attack"));
        fighter.picks.insert(
            class_slot(2, 0),
            pick_feat_for("weapon-focus", weapon("longsword")),
        );
        fighter.levels[3].ability_increase = Some(Ability::Strength);
        let full = full_sheet(&planner, &fighter, false, None, None);
        let rows = level_rows(full.levels.clone(), false);
        assert_eq!(rows.len(), 5);
        for (line, row) in full.levels.iter().zip(&rows) {
            let line = without_empties(to_value(line).unwrap());
            let row = without_empties(to_value(row).unwrap());
            assert_eq!(row, line, "level {}", row["level"]);
        }
        // The trimmed rows are smaller, so the trimming is doing something here.
        let trimmed = level_rows(full.levels, true);
        assert!(
            to_value(&trimmed).unwrap().to_string().len()
                < to_value(&rows).unwrap().to_string().len()
        );
    }
}
