//! `choose`: changes a Build only by what the engine offers. Each step recomputes the choice's options, finds the
//! offered one whose value matches, requires it open, applies *that option's own edit* with [`Engine::apply`], and
//! then opens the result's code again ([`reopen`]) as the final gate, so an impossible Build can never come back. No
//! edit is ever built from the agent's input. A point-buy score or a skill's ranks move one stepper step at a time,
//! each step checked the same way; a map of skills to ranks spends a level's skill points so, in one step.

use std::collections::BTreeMap;

use rules::{Address, Build, Engine, POINT_BUY_BUDGET, SkillKey};

use crate::code::{envelope, open, reopen};
use crate::error::ToolError;
use crate::options::{Offer, issue_code, issue_reason, offers};
use crate::planner::Planner;
use crate::shapes::{ChoiceValue, ChooseArgs, ChooseResult, Rejected};
use crate::similar::{closeness, closest_note, distance, nearest};

/// `choose`: the steps applied in order up to the first refused, with the code as the last applied one left it.
pub(crate) fn choose(planner: &Planner, args: ChooseArgs) -> Result<ChooseResult, ToolError> {
    let engine = &planner.engine;
    let mut build = open(planner, &args.build)?;
    let mut applied = 0;
    let mut rejected = None;
    for (index, step) in args.steps.iter().enumerate() {
        match take(engine, &build, &step.choice, &step.value, index) {
            Ok(next) => {
                build = next;
                applied += 1;
            }
            Err(refusal) => {
                rejected = Some(Rejected {
                    step: applied,
                    choice: step.choice.clone(),
                    issue: refusal.code.to_owned(),
                    reason: refusal.reason,
                });
                break;
            }
        }
    }
    let sheet = engine.evaluate(&build);
    Ok(ChooseResult {
        envelope: envelope(engine, &build, &sheet),
        applied,
        rejected,
    })
}

/// One step: the Build with the value taken at the address, or why not. `index` names the step in an `internal`
/// refusal.
fn take(
    engine: &Engine,
    build: &Build,
    choice: &str,
    value: &ChoiceValue,
    index: usize,
) -> Result<Build, ToolError> {
    let address: Address = crate::gear::parse_address(choice)?;
    if let (Address::Score(_) | Address::Skill(..), ChoiceValue::Number(target)) = (&address, value)
    {
        return step_to(engine, build, &address, *target, index);
    }
    if let ChoiceValue::Ranks(ranks) = value {
        let Address::SkillPoints(level) = address else {
            return Err(ToolError::not_an_option(format!(
                "A map of skills to ranks is for `level.N.skill`, a level's skill points, not `{address}`."
            )));
        };
        return skill_map(engine, build, level, ranks, index);
    }
    let offers = offers_at(engine, build, &address)?;
    let offer = find(value, &offers).ok_or_else(|| {
        missing_parameter(engine, build, &address, value, &offers)
            .unwrap_or_else(|| not_offered(&address, value, &offers))
    })?;
    if offer.current {
        // What the Build holds already: nothing to change.
        return Ok(build.clone());
    }
    apply_offer(engine, build, &address, offer, index)
}

/// The offer a value names ([`ChoiceValue::matches`]). One leniency: a class taken before with an archetype is
/// offered at later levels only with it, so `{class}` alone names that one offer when it's the class's only one (a
/// class not taken yet is offered without an archetype too, and that offer matches first).
fn find<'a>(value: &ChoiceValue, offers: &'a [Offer]) -> Option<&'a Offer> {
    if let Some(offer) = offers.iter().find(|offer| value.matches(&offer.value)) {
        return Some(offer);
    }
    if !matches!(
        value,
        ChoiceValue::Class {
            archetype: None,
            ..
        }
    ) {
        return None;
    }
    let mut same_class = offers.iter().filter(|offer| {
        let ChoiceValue::Class {
            class,
            archetype: Some(_),
            advances,
        } = &offer.value
        else {
            return false;
        };
        value.matches(&ChoiceValue::Class {
            class: class.clone(),
            archetype: None,
            advances: advances.clone(),
        })
    });
    let only = same_class.next()?;
    same_class.next().is_none().then_some(only)
}

/// Every option at an address, each value of a parameterised feat on its own line (so each has its edit).
fn offers_at(engine: &Engine, build: &Build, address: &Address) -> Result<Vec<Offer>, ToolError> {
    let sheet = engine.evaluate(build);
    offers(engine, build, &sheet, address, true)
}

/// Applies an offer's own edit, if it's open, and passes the result through the final gate.
fn apply_offer(
    engine: &Engine,
    build: &Build,
    address: &Address,
    offer: &Offer,
    index: usize,
) -> Result<Build, ToolError> {
    if let Err(issue) = &offer.status {
        return Err(ToolError {
            code: issue_code(issue),
            reason: offer
                .reason
                .clone()
                .unwrap_or_else(|| issue_reason(engine, issue)),
        });
    }
    let Some(edit) = &offer.edit else {
        // Offers listed with their values all carry their edit.
        return Err(ToolError::internal(format!(
            "Step {index} (`{address}`): the planner offered `{}` without its edit, so nothing was applied. This is \
             a bug in the planner.",
            value_text(&offer.value)
        )));
    };
    let mut next = build.clone();
    engine.apply(&mut next, edit);
    reopen(engine, &next).ok_or_else(|| {
        ToolError::internal(format!(
            "Step {index} (`{address}`): the planner's own edit for `{}` made a Build its rules don't allow, so it \
             was not applied. This is a bug in the planner.",
            value_text(&offer.value)
        ))
    })
}

/// A point-buy score or a skill's ranks at a level moved to `target` one stepper step at a time, the options
/// recomputed after each. A step that isn't offered, or isn't open, refuses the whole move, naming that step.
fn step_to(
    engine: &Engine,
    build: &Build,
    address: &Address,
    target: u8,
    index: usize,
) -> Result<Build, ToolError> {
    let mut build = build.clone();
    loop {
        let offers = offers_at(engine, &build, address)?;
        let current = stepper_value(&build, address);
        if current == target {
            return Ok(build);
        }
        let next = if target > current {
            current + 1
        } else {
            current - 1
        };
        let what = match address {
            Address::Score(ability) => ability.name().to_owned(),
            Address::Skill(level, skill) => {
                format!("{} ranks at level {level}", engine.skill_name(skill))
            }
            _ => unreachable!("only scores and skill ranks step"),
        };
        let at = format!("At {what} {current} → {next}");
        let Some(offer) = offers
            .iter()
            .find(|offer| offer.value == ChoiceValue::Number(next))
        else {
            let limits = match address {
                Address::Score(_) => format!(
                    "a point-buy score runs from 7 to 18 within the {POINT_BUY_BUDGET}-point budget, and {} points \
                     are spent now",
                    build.start.ability_scores.cost()
                ),
                _ => "a level adds ranks only up to its skill points, and no skill's total goes above the \
                      character level"
                    .to_owned(),
            };
            return Err(ToolError::not_an_option(format!(
                "{at}: there is no such step ({limits}). Nothing was changed."
            )));
        };
        build =
            apply_offer(engine, &build, address, offer, index).map_err(|refusal| ToolError {
                reason: format!("{at}: {}. Nothing was changed.", refusal.reason),
                ..refusal
            })?;
    }
}

/// A level's skill points spent from a map `{skill: ranks}`: each skill's ranks at the level moved to its number as
/// [`step_to`] moves them, one checked step at a time; the skills lowered go first, to free their points, then the
/// rest in key order. The first refused rank refuses the whole step, so nothing of it is applied.
fn skill_map(
    engine: &Engine,
    build: &Build,
    level: u8,
    ranks: &BTreeMap<String, u8>,
    index: usize,
) -> Result<Build, ToolError> {
    let held = |skill: &SkillKey| {
        usize::from(level)
            .checked_sub(1)
            .and_then(|at| build.levels.get(at))
            .and_then(|level_up| level_up.skill_ranks.get(skill))
            .copied()
            .unwrap_or(0)
    };
    let mut moves: Vec<(SkillKey, u8)> = ranks
        .iter()
        .map(|(skill, ranks)| (SkillKey::new(skill.trim().to_lowercase()), *ranks))
        .collect();
    moves.sort_by_key(|(skill, target)| *target >= held(skill));
    let mut next = build.clone();
    for (skill, target) in moves {
        next = step_to(engine, &next, &Address::Skill(level, skill), target, index)?;
    }
    Ok(next)
}

/// A parameterised feat or option given by its key alone, where the slot offers it only with a parameter: what a value
/// is, how to give one, and the values open now. `None` for any other value.
fn missing_parameter(
    engine: &Engine,
    build: &Build,
    address: &Address,
    value: &ChoiceValue,
    offered: &[Offer],
) -> Option<ToolError> {
    let ChoiceValue::Key(given) = value else {
        return None;
    };
    let parameters: Vec<(&str, &str, bool)> = offered
        .iter()
        .filter_map(|offer| match &offer.value {
            ChoiceValue::Parameterised { key, parameter }
                if key.eq_ignore_ascii_case(given.trim()) =>
            {
                Some((key.as_str(), parameter.as_str(), offer.status.is_ok()))
            }
            _ => None,
        })
        .collect();
    let (key, first, _) = *parameters.first()?;
    // The entry listed once, without its values, says what a value is.
    let sheet = engine.evaluate(build);
    let takes = offers(engine, build, &sheet, address, false)
        .ok()
        .and_then(|entries| {
            entries
                .into_iter()
                .find(|entry| entry.value.matches(value))
                .and_then(|entry| entry.takes)
        })
        .unwrap_or("a value");
    let open: Vec<&str> = parameters
        .iter()
        .filter(|(_, _, open)| *open)
        .map(|(_, parameter, _)| *parameter)
        .collect();
    let example = ChoiceValue::Parameterised {
        key: key.to_owned(),
        parameter: open.first().copied().unwrap_or(first).to_owned(),
    };
    let values = match open.is_empty() {
        true => "None of its values is open now: `options` with `values: true` and `only_open: false` gives each \
                 one's reason."
            .to_owned(),
        false => {
            let closed = parameters.len() - open.len();
            let closed = match closed {
                0 => String::new(),
                n => format!(" {n} more aren't open now."),
            };
            format!("Open values: {}.{closed}", open.join(", "))
        }
    };
    Some(ToolError::needs_parameter(format!(
        "`{key}` takes {takes}: give `{{key, parameter}}`, as `{}`. {values}",
        value_text(&example)
    )))
}

/// The number a stepper address holds now: a point-buy score, or the ranks a level adds to a skill. The level is
/// one the Build has ([`offers_at`] checked).
fn stepper_value(build: &Build, address: &Address) -> u8 {
    match address {
        Address::Score(ability) => build.start.ability_scores.get(*ability),
        Address::Skill(level, skill) => build.levels[usize::from(*level) - 1]
            .skill_ranks
            .get(skill)
            .copied()
            .unwrap_or(0),
        _ => unreachable!("only scores and skill ranks step"),
    }
}

/// Why a value isn't taken at an address: a prestige class that needs its `advances`, or the closest values offered
/// ([`nearest`]: only those close enough, so a value sharing one word of two isn't named).
fn not_offered(address: &Address, value: &ChoiceValue, offers: &[Offer]) -> ToolError {
    if let ChoiceValue::Class {
        advances: None,
        class,
        ..
    } = value
    {
        let answers: Vec<String> = offers
            .iter()
            .filter(|offer| value.matches(&without_advances(&offer.value)))
            .filter_map(|offer| match &offer.value {
                ChoiceValue::Class {
                    advances: Some(advances),
                    ..
                } => Some(format!(
                    "{} ({})",
                    serde_json::to_string(advances).expect("a list serialises"),
                    offer.name
                )),
                _ => None,
            })
            .collect();
        if !answers.is_empty() {
            return ToolError::not_an_option(format!(
                "`{class}` at `{address}` asks which classes its levels advance: give `advances` too, `[]` for \
                 none. Offered: {}.",
                answers.join(", ")
            ));
        }
    }
    if offers.is_empty() {
        return ToolError::not_an_option(format!("`{address}` offers nothing now."));
    }
    let given = value_text(value);
    let plain = plain_text(value);
    let ranked = nearest(
        offers,
        |offer| closeness(&plain, &plain_text(&offer.value)).max(closeness(&plain, &offer.name)),
        |one, other| {
            distance(&given, &value_text(&one.value))
                .cmp(&distance(&given, &value_text(&other.value)))
                .then_with(|| one.status.is_err().cmp(&other.status.is_err()))
        },
    );
    let closest: Vec<String> = ranked
        .iter()
        .map(|offer| {
            let open = if offer.status.is_ok() {
                ""
            } else {
                ", not open now"
            };
            format!("{} (`{}`{open})", offer.name, value_text(&offer.value))
        })
        .collect();
    let closest = closest_note(&closest)
        .unwrap_or_else(|| ", and no offered value is close to it".to_owned());
    ToolError::not_an_option(format!(
        "`{given}` isn't an option for `{address}`{closest}. `options` lists what it takes, with `values: true` for a \
         parameterised feat's values."
    ))
}

/// A class value without its `advances`, to tell a prestige class given without them.
fn without_advances(value: &ChoiceValue) -> ChoiceValue {
    match value {
        ChoiceValue::Class {
            class, archetype, ..
        } => ChoiceValue::Class {
            class: class.clone(),
            archetype: archetype.clone(),
            advances: None,
        },
        other => other.clone(),
    }
}

/// A value as JSON, as the agent would write it.
fn value_text(value: &ChoiceValue) -> String {
    serde_json::to_string(value).expect("a value serialises")
}

/// A value's keys, numbers and parameter as plain text, to measure how close it is to another.
fn plain_text(value: &ChoiceValue) -> String {
    match value {
        ChoiceValue::Number(n) => n.to_string(),
        ChoiceValue::Key(key) => key.clone(),
        ChoiceValue::Parameterised { key, parameter } => format!("{key} {parameter}"),
        ChoiceValue::Class {
            class, archetype, ..
        } => format!("{class} {}", archetype.as_deref().unwrap_or_default()),
        ChoiceValue::Ranks(ranks) => ranks.keys().cloned().collect::<Vec<_>>().join(" "),
    }
}
