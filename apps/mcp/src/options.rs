//! `options`: what a choice can take, from the engine's own picker or stepper for it, each option with its value,
//! its name, whether it can be taken now and, when not, the engine's issue and reason. Each option keeps the edit
//! the engine gives it, so `choose` applies the matching offer's edit and never builds one.

use rules::{Build, Edit, Engine, Issue, RankStatus, Sheet, SlotId, Step};

use crate::address::{Address, NotAnAddress};
use crate::code::{envelope, open};
use crate::error::ToolError;
use crate::planner::Planner;
use crate::shapes::{
    ChoiceValue, Kind, Open, OptionLine, OptionStatus, OptionsArgs, OptionsResult,
};
use crate::values::{
    ability_value, alignment_key, class_name, class_value, faith_key, pick_value, race_value,
};

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
    /// The filter chip it's filed under: a feat's category, an option's selection, "Base class", …
    pub category: Option<String>,
    /// A slot entry's summary, from the engine's picker; `None` when empty, and for the other pickers, whose summary
    /// `options` looks up ([`summary_of`]).
    pub summary: Option<String>,
    /// For an entry listed without its values: what a value is, with the article ("a weapon").
    pub takes: Option<&'static str>,
    /// Whether the Build holds it now.
    pub current: bool,
}

impl Offer {
    fn new(value: ChoiceValue, name: String, status: Result<(), Issue>, edit: Edit) -> Self {
        Self {
            value,
            name,
            status,
            reason: None,
            edit: Some(edit),
            category: None,
            summary: None,
            takes: None,
            current: false,
        }
    }

    fn current(mut self, current: bool) -> Self {
        self.current = current;
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

/// An issue's reason: the engine's own text, or for one that would break a later choice, [`BREAKS_LATER`].
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
        Address::Slot(slot) => slot_offers(engine, build, slot, values).ok_or_else(lacks)?,
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
                    race_value(race, subrace),
                    name.to_owned(),
                    option.status,
                    option.edit,
                )
                .current(current)
            })
            .collect(),
        Address::Faith => engine
            .faith_options(build)
            .into_iter()
            .map(|option| {
                let current = start.deity.as_ref() == Some(&option.value);
                Offer::new(
                    ChoiceValue::Key(faith_key(&option.value)),
                    engine.faith_name(&option.value).to_owned(),
                    option.status,
                    option.edit,
                )
                .current(current)
            })
            .collect(),
        Address::Alignment => engine
            .alignment_options(build)
            .into_iter()
            .map(|option| {
                Offer::new(
                    ChoiceValue::Key(alignment_key(option.value)),
                    option.value.name().to_owned(),
                    option.status,
                    option.edit,
                )
                .current(start.alignment == Some(option.value))
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
                        ability_value(option.value),
                        option.value.name().to_owned(),
                        option.status,
                        option.edit,
                    )
                    .current(start.racial_ability == Some(option.value))
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
                    ability_value(option.value),
                    format!("+1 {}", option.value.name()),
                    option.status,
                    option.edit,
                )
                .current(current)
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
                    Some(step_offer(ChoiceValue::Number(score), name, step))
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
                    Some(step_offer(ChoiceValue::Number(ranks), text, step))
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

fn step_offer(value: ChoiceValue, name: String, step: Step) -> Offer {
    Offer::new(value, name, step.status, step.edit)
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
fn slot_offers(engine: &Engine, build: &Build, slot: &SlotId, values: bool) -> Option<Vec<Offer>> {
    let held = build.picks.get(slot);
    let mut offers = Vec::new();
    for entry in engine.slot_entries(build, slot)? {
        let category = (!entry.category.is_empty()).then(|| entry.category.clone());
        let summary = (!entry.summary.is_empty()).then(|| entry.summary.clone());
        match entry.noun {
            Some(noun) if !values => {
                // One line for the entry: open when any value is.
                let first = &entry.options[0].option.pick;
                let status = match entry.open {
                    true => Ok(()),
                    false => Err(entry_issue(&entry.options)),
                };
                offers.push(Offer {
                    value: ChoiceValue::Key(first.key().to_owned()),
                    name: entry.name.clone(),
                    status,
                    reason: entry.reason.clone(),
                    edit: None,
                    category,
                    summary,
                    takes: Some(noun),
                    current: held.is_some_and(|pick| pick.key() == first.key()),
                });
            }
            _ => {
                for option in entry.options {
                    let pick = option.option.pick;
                    offers.push(Offer {
                        value: pick_value(&pick),
                        name: engine.pick_name(&pick),
                        status: option.option.status,
                        reason: option.reason,
                        edit: Some(option.option.edit),
                        category: category.clone(),
                        summary: summary.clone(),
                        takes: None,
                        current: held == Some(&pick),
                    });
                }
            }
        }
    }
    Some(offers)
}

/// The issue a locked entry shows, chosen as the engine chooses its one reason: the first option's that isn't
/// already holding it (Weapon Focus held for a longsword but needing BAB +1 for the rest says the BAB), else the first
/// option's.
fn entry_issue(options: &[rules::EntryOption]) -> Issue {
    let failing = || {
        options
            .iter()
            .filter_map(|option| option.option.status.as_ref().err())
    };
    failing()
        .find(|issue| !matches!(issue, Issue::AlreadyHeld | Issue::MaxTimes { .. }))
        .or_else(|| failing().next())
        .cloned()
        .unwrap_or(Issue::NotAllowed)
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
                    class_value(&option.class, archetype, None),
                    class_name(engine, &option.class, archetype, None),
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
                    class_value(&option.class, archetype, Some(&answer.advances)),
                    class_name(engine, &option.class, archetype, Some(&answer.advances)),
                    Ok(()),
                    answer.edit,
                )
                .current(current && held_advances == Some(&answer.advances));
                offer.category = Some(category.to_owned());
                offers.push(offer);
            }
        } else {
            let mut offer = Offer::new(
                class_value(&option.class, archetype, None),
                class_name(engine, &option.class, archetype, None),
                option.status,
                option.edit,
            )
            .current(current);
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
    let address: Address = args
        .choice
        .parse()
        .map_err(|error: NotAnAddress| ToolError::no_such_choice(error.to_string()))?;
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
            status: match &offer.status {
                Ok(()) => OptionStatus::Open(Open::Ok),
                Err(issue) => OptionStatus::Closed {
                    issue: issue_code(issue).to_owned(),
                    reason: offer
                        .reason
                        .clone()
                        .unwrap_or_else(|| issue_reason(engine, issue)),
                },
            },
            summary: match compact {
                true => None,
                false => offer
                    .summary
                    .or_else(|| summary_of(planner, &address, &offer.value)),
            },
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
