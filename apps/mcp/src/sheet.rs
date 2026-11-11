//! `sheet`: a Build as the engine's sheet has it, level by level and rank by rank, each choice by its address.
//! Everything comes from [`Engine::evaluate`]'s [`Sheet`] and the engine's labels; the only work here is saying
//! what each level adds (its feats and features: the state at its end against the one before).

use std::collections::BTreeSet;

use rules::{
    AbilityScores, Address, Build, CharacterState, Choice, Engine, POINT_BUY_BUDGET, RankStatus,
    Sheet, Slot, SpellTraits, WeaponKey, has_ability_increase,
};

use crate::attack;
use crate::caster;
use crate::code::envelope;
use crate::planner::Planner;
use crate::shapes::ChoiceValue;
use crate::shapes::{
    LevelLine, Named, PickLine, RankLine, RankState, Scores, SheetResult, SkillPointsLine,
    SpellLevels,
};
use crate::totals;

/// The sheet of a possible Build; with `removable`, each held choice marked with whether it can be withdrawn now;
/// each level's attack line for the `weapon` given, or else for the Build's Weapon Focus weapon
/// ([`Engine::default_attack_weapon`]), none without one; with a `caster` spell, each level's caster line for it.
pub(crate) fn sheet(
    planner: &Planner,
    build: &Build,
    removable: bool,
    weapon: Option<WeaponKey>,
    caster: Option<SpellTraits>,
) -> SheetResult {
    let evaluated = planner.engine.evaluate(build);
    let mut result = sheet_of(planner, build, &evaluated, removable);
    if let Some(weapon) = attack::weapon_or_default(&planner.engine, weapon.as_ref(), &evaluated) {
        for (line, level) in result.levels.iter_mut().zip(&evaluated.levels) {
            line.attack = attack::attack_line(&planner.engine, &level.state, &weapon);
        }
        result.attack_leaves_out = Some(attack::leaves_out());
    }
    if let Some(spell) = caster {
        for (line, level) in result.levels.iter_mut().zip(&evaluated.levels) {
            line.caster = caster::caster_line(&planner.engine, &build.start, &level.state, spell);
        }
        result.caster_leaves_out = Some(caster::leaves_out());
    }
    result
}

/// [`sheet`] from the Build's sheet already evaluated (`compare` reads the engine's sheet once per Build).
pub(crate) fn sheet_of(
    planner: &Planner,
    build: &Build,
    sheet: &Sheet,
    removable: bool,
) -> SheetResult {
    let engine = &planner.engine;
    let removable = removable.then(|| engine.removable(build));
    let lines = Lines {
        engine,
        removable: removable.as_ref(),
    };
    let empty = CharacterState::default();
    let last = sheet.levels.last().map_or(&empty, |level| &level.state);
    SheetResult {
        envelope: envelope(engine, build, sheet),
        name: build.name.clone(),
        start: lines.start(build),
        point_buy_spent: build.start.ability_scores.cost(),
        point_buy_budget: POINT_BUY_BUDGET,
        levels: lines.levels(build, sheet),
        ranks: sheet.ranks.iter().map(|rank| lines.rank(rank)).collect(),
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
        totals: totals::totals(engine, &build.start, last),
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
        PickLine {
            choice: address.to_string(),
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

    fn rank(&self, rank: &rules::RankSheet) -> RankLine {
        let engine = self.engine;
        RankLine {
            rank: rank.rank,
            character_level: rank.character_level,
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
            features: engine
                .rank_features(rank)
                .into_iter()
                .map(|feature| named(feature, &engine.feature_name_apart(feature)))
                .collect(),
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

/// The features granted at a level: each held more times at its end than before it.
fn new_features(engine: &Engine, before: &CharacterState, after: &CharacterState) -> Vec<Named> {
    after
        .features
        .iter()
        .filter(|(feature, times)| before.feature_count(feature) < **times)
        .map(|(feature, _)| named(feature, &engine.feature_name_apart(feature)))
        .collect()
}
