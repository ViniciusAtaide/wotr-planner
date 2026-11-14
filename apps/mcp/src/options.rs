//! `options`: what a choice can take, from the engine's own picker or stepper for it, each option with its value,
//! its name, whether it can be taken now and, when not, the engine's issue and reason. Each option keeps the edit
//! the engine gives it, so `choose` applies the matching offer's edit and never builds one.

use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Address, Build, Choice, Edit, Engine, Failure, Issue, RankStatus, Sheet, SlotId, Step,
};

use crate::code::{envelope, open};
use crate::error::ToolError;
use crate::gear::{self, GearMark};
use crate::planner::Planner;
use crate::shapes::{
    ChoiceValue, Kind, Open, OptionLine, OptionStatus, OptionsArgs, OptionsResult,
};
use crate::unreached::Unreached;

/// One option the engine offers for a choice.
#[derive(Clone, Debug)]
pub(crate) struct Offer {
    pub value: ChoiceValue,
    pub name: String,
    /// `Ok` when it can be taken now; otherwise the engine's issue.
    pub status: Result<(), Issue>,
    /// The engine's reason for a slot's option (`SlotEntry`'s, worded once for a run of values); `None` for the
    /// other pickers, whose reason is the issue's text ([`issue_reason`]).
    pub reason: Option<String>,
    /// The engine's edit that takes it; `None` for a slot's parameterised entry listed without its values, which
    /// needs one of them.
    pub edit: Option<Edit>,
    /// For an entry listed without its values, the edit of its first value: it stands for them when the entry
    /// breaks a later choice and the reason must name what ([`Breaks::reason`]). No edit is applied from it.
    pub like: Option<Edit>,
    /// For a slot's pick refused because a later rank's pick meets its prerequisite: what the level that would let it
    /// in is told from, worked out only when the reason is shown ([`LaterLevels::tail`]).
    pub opens: Option<Opens>,
    /// The filter chip it's filed under: a feat's category, an option's selection, "Base class", …
    pub category: Option<String>,
    /// A slot entry's summary, from the engine's picker; `None` when empty, and for the other pickers, whose summary
    /// `options` looks up ([`summary_of`]).
    pub summary: Option<String>,
    /// For an entry listed without its values: what a value is, with the article ("a weapon").
    pub takes: Option<&'static str>,
    /// Whether the Build holds it now.
    pub current: bool,
    /// For an open option: whether it is open only because of the Build's gear, or would leave other choices
    /// allowed only because of it.
    pub gear: Option<GearMark>,
    /// A gear item's other effects.
    pub other_effects: Vec<String>,
    /// A gear item's always-on lines that no number counts.
    pub uncounted: Vec<String>,
}

impl Offer {
    pub(crate) fn new(
        value: ChoiceValue,
        name: String,
        status: Result<(), Issue>,
        edit: Edit,
    ) -> Self {
        Self {
            value,
            name,
            status,
            reason: None,
            edit: Some(edit),
            like: None,
            opens: None,
            category: None,
            summary: None,
            takes: None,
            current: false,
            gear: None,
            other_effects: Vec::new(),
            uncounted: Vec::new(),
        }
    }

    fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    /// Marks an open option whose edit would leave these choices allowed only because of the gear (a picker's
    /// `leans_on_gear`).
    fn leaning(mut self, engine: &Engine, build: &Build, leans_on_gear: &[rules::Choice]) -> Self {
        if self.status.is_ok()
            && let Some(edit) = &self.edit
        {
            self.gear = gear::leaning(engine, build, edit, leans_on_gear);
        }
        self
    }

    /// The text matched against a query: its name and its value's keys.
    pub fn search_text(&self) -> String {
        let value = match &self.value {
            ChoiceValue::Number(n) => n.to_string(),
            ChoiceValue::Key(key) => key.clone(),
            ChoiceValue::Parameterised { key, parameter } => format!("{key} {parameter}"),
            ChoiceValue::Class {
                class, archetype, ..
            } => format!("{class} {}", archetype.as_deref().unwrap_or_default()),
            ChoiceValue::Ranks(ranks) => ranks.keys().cloned().collect::<Vec<_>>().join(" "),
        };
        format!("{} {value}", self.name).to_lowercase()
    }
}

/// The reason for an option that would break a later choice, which the engine leaves unworded (the web app greys
/// it out).
pub(crate) const BREAKS_LATER: &str = "Would break a later choice, which must be withdrawn first (`sheet` with \
    `removable: true` shows what can come off now)";

/// How many of the choices an edit would break its reason names; the rest are counted ("and 4 more").
const NAMED: usize = 3;

/// The verb of a reason that names `count` choices: "needs"/"need", "depends"/"depend".
pub(crate) fn verb<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

/// What the edits of one Build would break, told from the choices the Build already fails (none, in a possible
/// one). It walks the Build once, when the first reason is asked, and each edit once more.
pub(crate) struct Breaks<'a> {
    engine: &'a Engine,
    build: &'a Build,
    failing: OnceCell<BTreeSet<Choice>>,
    /// The Build's sheet, walked when a reason must say where a pick is held.
    sheet: OnceCell<Sheet>,
}

impl<'a> Breaks<'a> {
    pub(crate) fn new(engine: &'a Engine, build: &'a Build) -> Self {
        Self {
            engine,
            build,
            failing: OnceCell::new(),
            sheet: OnceCell::new(),
        }
    }

    /// The choices an edit would refuse, in walk order, each with its issue.
    pub(crate) fn refusals(&self, edit: &Edit) -> Vec<Failure> {
        let failing = self
            .failing
            .get_or_init(|| self.engine.failing_choices(self.build));
        self.engine.refusals(self.build, failing, edit)
    }

    /// Where the Build holds the pick of a slot, when a slot holds one ("Toughness is taken in level 3's Feat slot").
    fn taken_in(&self, slot: &SlotId) -> Option<String> {
        let sheet = self.sheet.get_or_init(|| self.engine.evaluate(self.build));
        self.engine.taken_in(self.build, sheet.slot(slot)?)
    }

    /// Why the choices an edit would refuse are refused, as a reason says it (`depends`/`depend` and the word to
    /// follow, for a withdrawal's and a pick's reason): `Held` for the first that would be held twice, `Needs` when
    /// all fail a prerequisite that isn't a negation, `Other` for anything else.
    pub(crate) fn cause(&self, refusals: &[Failure]) -> Cause {
        let held = refusals.iter().find_map(|failure| {
            let Choice::Slot(slot) = &failure.choice else {
                return None;
            };
            matches!(failure.issue, Issue::AlreadyHeld | Issue::MaxTimes { .. })
                .then(|| self.taken_in(slot))
                .flatten()
        });
        if let Some(text) = held {
            return Cause::Held(text);
        }
        let needs = refusals.iter().all(|failure| {
            matches!(&failure.issue, Issue::Unmet(unmet) if !Engine::unmet_negates(unmet))
        });
        if needs { Cause::Needs } else { Cause::Other }
    }

    /// The first few of the choices, named as a reason names them.
    pub(crate) fn names(&self, refused: &[Choice]) -> String {
        self.engine.choice_names_within(self.build, refused, NAMED)
    }

    /// Why an option that breaks a later choice does, naming what it would break: "Dodge needs Dexterity above 10"
    /// for a score lowered, "…needs more than 2 Stealth ranks at level 3" for a skill's ranks, otherwise "Power
    /// Attack depends on what `slot.feat.1.general.0` holds now". `None` when the edit refuses nothing the walk can
    /// name (the caller falls back to [`BREAKS_LATER`]).
    pub(crate) fn reason(&self, address: &Address, edit: &Edit) -> Option<String> {
        let (engine, build) = (self.engine, self.build);
        let refusals = self.refusals(edit);
        if refusals.is_empty() {
            return None;
        }
        let refused: Vec<Choice> = refusals.iter().map(|f| f.choice.clone()).collect();
        let names = self.names(&refused);
        let count = refused.len();
        let needs = verb(count, "needs", "need");
        Some(match (address, edit) {
            (Address::Score(ability), Edit::Start(after)) => {
                let (was, now) = (
                    build.start.ability_scores.get(*ability),
                    after.ability_scores.get(*ability),
                );
                let side = if now < was { "above" } else { "below" };
                format!("{names} {needs} {} {side} {now}", ability.name())
            }
            (
                Address::Skill(level, _) | Address::SkillPoints(level),
                Edit::SkillRanks { skill, ranks, .. },
            ) => {
                let was = usize::from(*level)
                    .checked_sub(1)
                    .and_then(|index| build.levels.get(index))
                    .and_then(|level_up| level_up.skill_ranks.get(skill))
                    .copied()
                    .unwrap_or(0);
                let side = if ranks < &was { "more" } else { "fewer" };
                let noun = if *ranks == 1 { "rank" } else { "ranks" };
                format!(
                    "{names} {needs} {side} than {ranks} {} {noun} at level {level}",
                    engine.skill_name(skill)
                )
            }
            _ => match self.cause(&refusals) {
                Cause::Held(text) => text,
                // The slot's pick is what they need, so it depends on what the slot holds.
                Cause::Needs if matches!(address, Address::Slot(slot) if build.picks.contains_key(slot)) =>
                {
                    format!(
                        "{names} {} on what `{address}` holds now",
                        verb(count, "depends", "depend")
                    )
                }
                _ => format!("Would break {names}"),
            },
        })
    }
}

/// What the choices an edit would refuse fail for ([`Breaks::cause`]).
pub(crate) enum Cause {
    /// A pick would be held twice: where it is held now.
    Held(String),
    /// A prerequisite the edit would leave unmet.
    Needs,
    /// Anything else (a pick that excludes another, one no longer offered).
    Other,
}

/// An engine issue as a code: the variant's name in snake case.
pub(crate) fn issue_code(issue: &Issue) -> &'static str {
    match issue {
        Issue::Unmet(_) => "unmet",
        Issue::AlreadyHeld => "already_held",
        Issue::MaxTimes { .. } => "max_times",
        Issue::NotAllowed => "not_allowed",
        Issue::UnknownKey => "unknown_key",
        Issue::NotReached => "not_reached",
        Issue::ClassLevelCap { .. } => "class_level_cap",
        Issue::NotFromPath { .. } => "not_from_path",
        Issue::DeityAlignment { .. } => "deity_alignment",
        Issue::NeedsFaith => "needs_faith",
        Issue::DeityDomain { .. } => "deity_domain",
        Issue::BreaksLater => "breaks_later",
    }
}

/// An issue's reason: the engine's own text, or for one that would break a later choice, [`BREAKS_LATER`] (the
/// last fallback: [`Breaks::reason`] names the choices when it can).
pub(crate) fn issue_reason(engine: &Engine, issue: &Issue) -> String {
    engine
        .issue_text(issue)
        .unwrap_or_else(|| BREAKS_LATER.to_owned())
}

/// Every option the engine offers at an address of a possible Build, in the engine's order. `values` lists a slot's
/// parameterised feats and options once per value; otherwise once, with what a value is (`takes`). `sheet` is the
/// Build's, to tell which Mythic ranks it reaches.
pub(crate) fn offers(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
    address: &Address,
    values: bool,
) -> Result<Vec<Offer>, ToolError> {
    let lacks = || no_such_choice(build, address);
    let start = &build.start;
    Ok(match address {
        Address::Slot(slot) => slot_offers(engine, build, sheet, slot, values).ok_or_else(lacks)?,
        Address::Gear(spot) => gear::gear_offers(engine, build, spot)?,
        // The class picker offers a level past the cap too: the web app shows no add control there.
        Address::Class(level) if *level > sheet.level_cap => {
            return Err(ToolError::no_such_choice(format!(
                "Level {} is the last level a Build can have (the level cap; 40 after a Legend Path switch).",
                sheet.level_cap
            )));
        }
        Address::Class(level) => class_offers(engine, build, *level).ok_or_else(lacks)?,
        Address::Race => engine
            .race_options(build)
            .into_iter()
            .map(|option| {
                let (race, subrace) = (&option.value.race, option.value.subrace.as_ref());
                let name = match subrace {
                    Some(subrace) => engine.subrace_name(subrace),
                    None => engine.race_name(race),
                };
                let current =
                    start.race.as_ref() == Some(race) && start.subrace.as_ref() == subrace;
                Offer::new(
                    ChoiceValue::race(race, subrace),
                    name.to_owned(),
                    option.status,
                    option.edit,
                )
                .current(current)
                .leaning(engine, build, &option.leans_on_gear)
            })
            .collect(),
        Address::Faith => engine
            .faith_options(build)
            .into_iter()
            .map(|option| {
                let current = start.deity.as_ref() == Some(&option.value);
                Offer::new(
                    ChoiceValue::Key(option.value.key().to_owned()),
                    engine.faith_name(&option.value).to_owned(),
                    option.status,
                    option.edit,
                )
                .current(current)
                .leaning(engine, build, &option.leans_on_gear)
            })
            .collect(),
        Address::Alignment => engine
            .alignment_options(build)
            .into_iter()
            .map(|option| {
                Offer::new(
                    ChoiceValue::Key(option.value.key().to_owned()),
                    option.value.name().to_owned(),
                    option.status,
                    option.edit,
                )
                .current(start.alignment == Some(option.value))
                .leaning(engine, build, &option.leans_on_gear)
            })
            .collect(),
        Address::Background => engine
            .background_options(build)
            .into_iter()
            .map(|option| {
                let current = start.background.as_ref() == Some(&option.value);
                Offer::new(
                    ChoiceValue::Key(option.value.as_str().to_owned()),
                    engine.background_name(&option.value).to_owned(),
                    option.status,
                    option.edit,
                )
                .current(current)
                .leaning(engine, build, &option.leans_on_gear)
            })
            .collect(),
        Address::RacialAbility => {
            let options = engine.racial_ability_options(build);
            if options.is_empty() {
                return Err(lacks());
            }
            options
                .into_iter()
                .map(|option| {
                    Offer::new(
                        ChoiceValue::ability(option.value),
                        option.value.name().to_owned(),
                        option.status,
                        option.edit,
                    )
                    .current(start.racial_ability == Some(option.value))
                    .leaning(engine, build, &option.leans_on_gear)
                })
                .collect()
        }
        Address::Increase(level) => engine
            .ability_increase_options(build, *level)
            .ok_or_else(lacks)?
            .into_iter()
            .map(|option| {
                let current =
                    build.levels[usize::from(*level) - 1].ability_increase == Some(option.value);
                Offer::new(
                    ChoiceValue::ability(option.value),
                    option.value.increase_name(),
                    option.status,
                    option.edit,
                )
                .current(current)
                .leaning(engine, build, &option.leans_on_gear)
            })
            .collect(),
        Address::Score(ability) => {
            let stepper = engine
                .point_buy_steps(build)
                .into_iter()
                .find(|stepper| stepper.value == *ability)
                .ok_or_else(lacks)?;
            [stepper.down, stepper.up]
                .into_iter()
                .flatten()
                .filter_map(|step| {
                    let Edit::Start(after) = &step.edit else {
                        return None;
                    };
                    let score = after.ability_scores.get(*ability);
                    let name = format!(
                        "{} {score} ({} of {} points)",
                        ability.name(),
                        after.ability_scores.cost(),
                        rules::POINT_BUY_BUDGET
                    );
                    Some(step_offer(
                        engine,
                        build,
                        ChoiceValue::Number(score),
                        name,
                        step,
                    ))
                })
                .collect()
        }
        Address::Skill(level, skill) => {
            let stepper = engine
                .skill_steps(build, *level)
                .ok_or_else(lacks)?
                .into_iter()
                .find(|stepper| &stepper.value == skill)
                .ok_or_else(lacks)?;
            let name = engine.skill_name(skill);
            [stepper.down, stepper.up]
                .into_iter()
                .flatten()
                .filter_map(|step| {
                    let Edit::SkillRanks { ranks, .. } = step.edit else {
                        return None;
                    };
                    let text = format!(
                        "{name}: {ranks} rank{} at level {level}",
                        if ranks == 1 { "" } else { "s" }
                    );
                    Some(step_offer(
                        engine,
                        build,
                        ChoiceValue::Number(ranks),
                        text,
                        step,
                    ))
                })
                .collect()
        }
        Address::SkillPoints(level) => engine
            .skill_steps(build, *level)
            .ok_or_else(lacks)?
            .into_iter()
            .filter_map(|stepper| {
                let step = stepper.up?;
                let Edit::SkillRanks { ranks, .. } = step.edit else {
                    return None;
                };
                let name = format!(
                    "{} +1 ({ranks} at level {level})",
                    engine.skill_name(&stepper.value)
                );
                Some(step_offer(
                    engine,
                    build,
                    ChoiceValue::Key(stepper.value.as_str().to_owned()),
                    name,
                    step,
                ))
            })
            .collect(),
        Address::FirstAscension | Address::Path | Address::Switch => {
            let rank = match address {
                Address::FirstAscension => rules::FIRST_ASCENSION_RANK,
                Address::Path => rules::PATH_CHOICE_RANK,
                _ => rules::PATH_SWITCH_RANK,
            };
            let reached = sheet
                .ranks
                .iter()
                .any(|r| r.rank == rank && r.status != RankStatus::NotReached);
            if !reached {
                return Err(ToolError::no_such_choice(format!(
                    "The Build doesn't reach MR {rank}, where `{address}` is chosen."
                )));
            }
            let (options, held) = match address {
                Address::FirstAscension => (
                    engine.first_ascension_options(build),
                    &build.first_ascension,
                ),
                Address::Path => (engine.path_options(build), &build.mythic_path),
                _ => (engine.switch_options(build), &build.path_switch),
            };
            options
                .into_iter()
                .map(|option| {
                    let name = match address {
                        Address::FirstAscension => engine.first_ascension_name(&option.path),
                        _ => engine.path_name(&option.path).to_owned(),
                    };
                    let current = held.as_ref() == Some(&option.path);
                    Offer::new(
                        ChoiceValue::Key(option.path.as_str().to_owned()),
                        name,
                        option.status,
                        option.edit,
                    )
                    .current(current)
                })
                .collect()
        }
        Address::RankLevel(rank) => rank_level_offers(engine, build, *rank)?,
        Address::Subrace => {
            return Err(ToolError::no_such_choice(
                "The subrace is chosen with the race: see `options` for `start.race`.",
            ));
        }
        Address::Level(level) => {
            return Err(ToolError::no_such_choice(format!(
                "`level.{level}` is the level itself and has no options; its class is `level.{level}.class`."
            )));
        }
        Address::Archetype(_) | Address::Advances(_) => {
            return Err(ToolError::no_such_choice(
                "A class's archetype and advanced classes are chosen with the class: see `options` for \
                 `level.N.class` at the level that first takes it.",
            ));
        }
    })
}

/// A Mythic rank's level picker (`mythic.R.level`): each character level the rank can be taken at, from the earliest
/// it comes at to the Build's last level. A level's name says when it is the earliest and which ranks above move
/// with it; a move that would refuse a choice is closed with the engine's reason, which names the choice.
fn rank_level_offers(engine: &Engine, build: &Build, rank: u8) -> Result<Vec<Offer>, ToolError> {
    let options = engine.rank_level_options(build, rank);
    if options.is_empty() {
        // No levels: the rank doesn't exist, or the Build doesn't reach it.
        let reason = match Unreached::of(engine, build, rank) {
            Some(unreached) => unreached.reason(rank, &Unreached::no_level(rank)),
            None => format!("MR {rank} has no level to take it at."),
        };
        return Err(ToolError::no_such_choice(reason));
    }
    Ok(options
        .into_iter()
        .map(|option| {
            let mut name = format!("Level {}", option.level);
            if option.earliest {
                name.push_str(" (the earliest)");
            }
            if !option.moves.is_empty() {
                name.push_str(&format!(
                    ", which moves {}",
                    Engine::moves_text(&option.moves)
                ));
            }
            let mut offer = Offer::new(
                ChoiceValue::Number(option.level),
                name,
                option.status,
                option.edit,
            )
            .current(option.held);
            offer.reason = option.reason;
            offer
        })
        .collect())
}

fn step_offer(
    engine: &Engine,
    build: &Build,
    value: ChoiceValue,
    name: String,
    step: Step,
) -> Offer {
    Offer::new(value, name, step.status, step.edit).leaning(engine, build, &step.leans_on_gear)
}

/// Why the Build has no choice at an address.
fn no_such_choice(build: &Build, address: &Address) -> ToolError {
    let levels = build.levels.len();
    let reason = match address {
        Address::Class(_) => format!(
            "The Build has {levels} level(s): `level.N.class` takes N from 1 to {}, the last adding a level.",
            levels + 1
        ),
        Address::Increase(level) if usize::from(*level) <= levels => {
            format!("Level {level} has no ability increase: only every 4th level does.")
        }
        Address::RacialAbility => {
            "The Build's race doesn't choose where a racial bonus goes (or no race is chosen yet)."
                .to_owned()
        }
        Address::Skill(level, skill) if usize::from(*level) <= levels => {
            format!("There is no skill `{}`.", skill.as_str())
        }
        _ => format!(
            "The Build has no choice `{address}`: `sheet` and `todo` print the addresses it has."
        ),
    };
    ToolError::no_such_choice(reason)
}

/// A slot's options from [`Engine::slot_entries`]: each entry once, or with `values` each of its options; `None`
/// when the Build has no such slot.
fn slot_offers(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
    slot: &SlotId,
    values: bool,
) -> Option<Vec<Offer>> {
    let held = build.picks.get(slot);
    let mut offers = Vec::new();
    for entry in engine.slot_entries_in(build, Some(sheet), slot)? {
        let category = (!entry.category.is_empty()).then(|| entry.category.clone());
        let summary = (!entry.summary.is_empty()).then(|| entry.summary.clone());
        match entry.noun {
            Some(noun) if !values => {
                // One line for the entry: open when any value is.
                let first = &entry.options[0].option.pick;
                // The value whose refusal the entry shows: its issue, pick and edit stand for the entry.
                let shown = entry.shown();
                let status = match shown {
                    None => Ok(()),
                    Some(option) => Err(option
                        .option
                        .status
                        .clone()
                        .expect_err("a locked entry's shown value is locked")),
                };
                // A later rank's pick that the entry needs says where it is (ticket 192).
                let clause = match (&status, &entry.reason, shown) {
                    (Err(issue @ Issue::Unmet(_)), Some(reason), Some(option)) => {
                        later_rank_clause(
                            engine,
                            build,
                            sheet,
                            (slot, &option.option.pick),
                            issue,
                            reason,
                        )
                    }
                    _ => None,
                };
                let (reason, opens) = match clause {
                    Some((reason, opens)) => (Some(reason), Some(opens)),
                    None => (entry.reason.clone(), None),
                };
                offers.push(Offer {
                    value: ChoiceValue::Key(first.key().to_owned()),
                    name: entry.name.clone(),
                    status,
                    reason,
                    edit: None,
                    like: shown.map(|option| option.option.edit.clone()),
                    opens,
                    category,
                    summary,
                    takes: Some(noun),
                    current: held.is_some_and(|pick| pick.key() == first.key()),
                    gear: entry.gear_note.clone().map(GearMark::WithGear),
                    other_effects: Vec::new(),
                    uncounted: Vec::new(),
                });
            }
            _ => {
                for option in entry.options {
                    let gear = (option.option.with_gear.as_ref())
                        .map(|needs| GearMark::WithGear(engine.gear_note(needs)));
                    let pick = option.option.pick;
                    // A Mythic rank's pick that a later level of the rank would open says so (ticket 171).
                    let mut opens = None;
                    let reason = match (&option.option.status, option.reason) {
                        (Err(issue @ Issue::Unmet(_)), Some(reason)) => {
                            Some(match later_rank_level(engine, build, sheet, slot, &pick) {
                                Some(later) => format!("{reason}. {later}"),
                                // Or a prerequisite that a later rank's pick meets (ticket 192).
                                None => match later_rank_clause(
                                    engine,
                                    build,
                                    sheet,
                                    (slot, &pick),
                                    issue,
                                    &reason,
                                ) {
                                    Some((reason, found)) => {
                                        opens = Some(found);
                                        reason
                                    }
                                    None => reason,
                                },
                            })
                        }
                        (_, reason) => reason,
                    };
                    offers.push(Offer {
                        value: ChoiceValue::pick(&pick),
                        name: engine.pick_name(&pick),
                        status: option.option.status,
                        reason,
                        edit: Some(option.option.edit),
                        like: None,
                        opens,
                        category: category.clone(),
                        summary: summary.clone(),
                        takes: None,
                        current: held == Some(&pick),
                        gear,
                        other_effects: Vec::new(),
                        uncounted: Vec::new(),
                    });
                }
            }
        }
    }
    Some(offers)
}

/// For a Mythic rank's slot and a pick it refuses for an unmet prerequisite: the later level of the rank that opens
/// it, as a sentence naming the address to `choose` and the ranks above that the move takes along ("MR 10 taken at
/// level 20 opens it: `choose` `mythic.10.level` 20 first"; "MR 2 taken at level 13 opens it, which moves MR 3 to
/// level 13, MR 4 to level 13: …").
fn later_rank_level(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
    slot: &SlotId,
    pick: &rules::Pick,
) -> Option<String> {
    let rank = slot.rank()?;
    let (level, moves) = engine.rank_level_that_opens(build, sheet, slot, pick)?;
    Some(opens_text(rank, level, &moves))
}

/// "MR 10 taken at level 20 opens it: `choose` `mythic.10.level` 20 first", with ", which moves MR 3 to level 13, …"
/// when the move takes ranks above along.
pub(crate) fn opens_text(rank: u8, level: u8, moves: &[(u8, u8)]) -> String {
    let along = match moves.is_empty() {
        true => String::new(),
        false => format!(", which moves {}", Engine::moves_text(moves)),
    };
    format!(
        "MR {rank} taken at level {level} opens it{along}: `choose` `{}` {level} first",
        Address::RankLevel(rank)
    )
}

/// What a pick refused because a later rank's pick meets its prerequisite needs to say which level would let it in.
#[derive(Clone, Debug)]
pub(crate) struct Opens {
    later: rules::LaterRankPick,
    slot: SlotId,
    pick: rules::Pick,
}

/// For a slot's pick refused for a prerequisite that a pick of a Mythic rank taken at a later level meets: the
/// engine's reason with where that pick is (as the web app says it), and what the level that would let the pick in
/// is told from ([`LaterLevels::tail`]). `None` when no later rank's pick is what it needs (ticket 192). No walk.
pub(crate) fn later_rank_clause(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
    (slot, pick): (&SlotId, &rules::Pick),
    issue: &Issue,
    reason: &str,
) -> Option<(String, Opens)> {
    let later = engine.later_rank_pick(build, sheet, slot, pick, issue)?;
    let text = format!("{reason}: {}", engine.later_rank_text(&later));
    let opens = Opens {
        later,
        slot: slot.clone(),
        pick: pick.clone(),
    };
    Some((text, opens))
}

/// The levels later ranks can be taken at before a slot, for the Build of one call: worked out once for a rank, and
/// only when a reason that needs them is shown.
pub(crate) struct LaterLevels<'a> {
    engine: &'a Engine,
    /// The Build the levels are tried on: a possible one.
    build: &'a Build,
    levels: RefCell<BTreeMap<u8, rules::LaterRankLevels>>,
}

impl<'a> LaterLevels<'a> {
    pub(crate) fn new(engine: &'a Engine, build: &'a Build) -> Self {
        Self {
            engine,
            build,
            levels: RefCell::new(BTreeMap::new()),
        }
    }

    /// ". MR 4 taken at level 12 opens it: `choose` `mythic.4.level` 12 first", or nothing when no level before the
    /// slot lets the pick in.
    pub(crate) fn tail(&self, opens: &Opens) -> Option<String> {
        let rank = opens.later.rank;
        let mut levels = self.levels.borrow_mut();
        let levels = levels
            .entry(rank)
            .or_insert_with(|| self.engine.later_rank_levels(self.build, &opens.later));
        let (level, moves) = self
            .engine
            .level_that_lets_in(levels, &opens.slot, &opens.pick)?;
        Some(format!(". {}", opens_text(rank, level, &moves)))
    }
}

/// The class picker at a level from [`Engine::class_options`]; a spellcasting prestige class first taken there is
/// offered once per answer [`Engine::advances_options`] gives. `None` for a level that isn't in the Build or next.
fn class_offers(engine: &Engine, build: &Build, level: u8) -> Option<Vec<Offer>> {
    let held = usize::from(level)
        .checked_sub(1)
        .and_then(|index| build.levels.get(index));
    let held_archetype = held.and_then(|level_up| {
        build
            .class_choices
            .get(&level_up.class)
            .and_then(|choice| choice.archetype.as_ref())
    });
    let mut offers = Vec::new();
    for option in engine.class_options(build, level)? {
        let category = if engine.is_prestige(&option.class) {
            "Prestige class"
        } else {
            "Base class"
        };
        let current = held.is_some_and(|level_up| level_up.class == option.class)
            && held_archetype == option.archetype.as_ref();
        let archetype = option.archetype.as_ref();
        if option.status.is_ok() && engine.asks_advances(build, level, &option.class) {
            let answers = engine.advances_options(build, level, &option);
            if answers.is_empty() {
                // No advancement keeps the Build possible.
                let mut offer = Offer::new(
                    ChoiceValue::class(&option.class, archetype, None),
                    engine.class_value_name(&option.class, archetype, None),
                    Err(Issue::BreaksLater),
                    option.edit.clone(),
                );
                offer.category = Some(category.to_owned());
                offers.push(offer);
            }
            for answer in answers {
                let held_advances = build
                    .class_choices
                    .get(&option.class)
                    .map(|choice| &choice.advances);
                let mut offer = Offer::new(
                    ChoiceValue::class(&option.class, archetype, Some(&answer.advances)),
                    engine.class_value_name(&option.class, archetype, Some(&answer.advances)),
                    Ok(()),
                    answer.edit,
                )
                .current(current && held_advances == Some(&answer.advances));
                offer.category = Some(category.to_owned());
                offers.push(offer);
            }
        } else {
            let mut offer = Offer::new(
                ChoiceValue::class(&option.class, archetype, None),
                engine.class_value_name(&option.class, archetype, None),
                option.status,
                option.edit,
            )
            .current(current)
            .leaning(engine, build, &option.leans_on_gear);
            offer.category = Some(category.to_owned());
            offers.push(offer);
        }
    }
    Some(offers)
}

/// The summary of what a value names at an address outside a slot (a slot's offers carry their entry's): a class's
/// or its archetype's, a race's or subrace's, a background's note, a path's. `None` when the data has none.
fn summary_of(planner: &Planner, address: &Address, value: &ChoiceValue) -> Option<String> {
    match (address, value) {
        (
            Address::Class(_),
            ChoiceValue::Class {
                class, archetype, ..
            },
        ) => match archetype {
            Some(archetype) => planner.summary(Kind::Archetype, archetype),
            None => planner.summary(Kind::Class, class),
        },
        (Address::Race, ChoiceValue::Key(key)) => planner.summary(Kind::Race, key).or_else(|| {
            planner
                .data
                .races
                .iter()
                .flat_map(|race| &race.subraces)
                .find(|subrace| subrace.key.as_str() == key)
                .and_then(|subrace| (!subrace.summary.is_empty()).then(|| subrace.summary.clone()))
        }),
        (Address::Background, ChoiceValue::Key(key)) => planner.summary(Kind::Background, key),
        (Address::FirstAscension | Address::Path | Address::Switch, ChoiceValue::Key(key)) => {
            planner.summary(Kind::Path, key)
        }
        _ => None,
    }
}

/// `options`: the offers at an address of a Build code, filtered by the query and, unless `only_open` is false, to
/// those open now, with the envelope.
pub(crate) fn options(planner: &Planner, args: OptionsArgs) -> Result<OptionsResult, ToolError> {
    let engine = &planner.engine;
    let build = open(planner, &args.build)?;
    let address: Address = crate::gear::parse_address(&args.choice)?;
    let sheet = engine.evaluate(&build);
    let offers = offers(
        engine,
        &build,
        &sheet,
        &address,
        args.values.unwrap_or(false),
    )?;
    let words: Vec<String> = args
        .query
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let only_open = args.only_open.unwrap_or(true);
    let compact = args.compact.unwrap_or(false);
    let breaks = Breaks::new(engine, &build);
    let later = LaterLevels::new(engine, &build);
    let mut closed = 0;
    let mut lines = Vec::new();
    for offer in offers {
        let text = offer.search_text();
        if !words.iter().all(|word| text.contains(word.as_str())) {
            continue;
        }
        if only_open && offer.status.is_err() {
            closed += 1;
            continue;
        }
        lines.push(OptionLine {
            status: match (&offer.status, &offer.gear) {
                (Ok(()), None) => OptionStatus::Open(Open::Ok),
                (Ok(()), Some(GearMark::WithGear(note))) => OptionStatus::WithGear {
                    with_gear: note.clone(),
                },
                (Ok(()), Some(GearMark::LeansOnGear(note))) => OptionStatus::LeansOnGear {
                    leans_on_gear: note.clone(),
                },
                (Err(issue), _) => OptionStatus::Closed {
                    issue: issue_code(issue).to_owned(),
                    reason: {
                        let reason = offer.reason.clone().unwrap_or_else(|| {
                            match (issue, offer.edit.as_ref().or(offer.like.as_ref())) {
                                (Issue::BreaksLater, Some(edit)) => breaks.reason(&address, edit),
                                _ => None,
                            }
                            .unwrap_or_else(|| issue_reason(engine, issue))
                        });
                        // The level of a later rank that would let a pick in, only for the lines shown.
                        let tail = offer.opens.as_ref().and_then(|opens| later.tail(opens));
                        format!("{reason}{}", tail.unwrap_or_default())
                    },
                },
            },
            summary: match compact {
                true => None,
                false => offer
                    .summary
                    .or_else(|| summary_of(planner, &address, &offer.value)),
            },
            other_effects: (!compact && !offer.other_effects.is_empty())
                .then_some(offer.other_effects),
            uncounted: (!compact && !offer.uncounted.is_empty()).then_some(offer.uncounted),
            value: offer.value,
            name: offer.name,
            category: offer.category,
            takes: offer.takes.map(str::to_owned),
            current: offer.current,
        });
    }
    Ok(OptionsResult {
        envelope: args
            .envelope
            .unwrap_or(false)
            .then(|| envelope(engine, &build, &sheet)),
        choice: address.to_string(),
        options: lines,
        closed,
    })
}
