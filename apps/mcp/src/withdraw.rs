//! `withdraw`: takes a choice back only by the engine's own Withdrawal ([`Engine::withdrawal`]), and only when the
//! engine says nothing later depends on it; the result passes the same final gate as `choose`'s ([`reopen`]).
//!
//! Two addresses are refused by design: `start.racial-ability`, which the engine keeps with the race (it has no
//! `Choice` of its own), and `level.N.class`, whose engine Withdrawal sets a class key no data has: an agent changes
//! a level's class with `choose`, and drops the last level with `level.N`.
//!
//! A gear item comes off the same way (`gear.belt`, `gear.usable.<item key>`), unless a choice needs it: then the
//! refusal names that choice, as the engine words it ([`crate::gear::needed_reason`]).

use rules::Address;

use crate::code::{envelope, open, reopen};
use crate::error::ToolError;
use crate::planner::Planner;
use crate::shapes::{WithdrawArgs, WithdrawResult};

/// `withdraw`: the Build without the choice, or why it can't come off.
pub(crate) fn withdraw(planner: &Planner, args: WithdrawArgs) -> Result<WithdrawResult, ToolError> {
    let engine = &planner.engine;
    let build = open(planner, &args.build)?;
    let address: Address = crate::gear::parse_address(&args.choice)?;
    let last = build.levels.len();
    match &address {
        Address::RacialAbility => {
            return Err(ToolError::not_removable(
                "The racial ability bonus comes with the race and isn't withdrawn on its own: change the race \
                 (`choose` `start.race`), or choose another ability (`choose` `start.racial-ability`).",
            ));
        }
        Address::Class(level) => {
            let drop_last = match last {
                0 => "the Build has no level to drop".to_owned(),
                last => format!("to drop the last level, withdraw `level.{last}`"),
            };
            return Err(ToolError::not_removable(format!(
                "A level's class isn't withdrawn: to change it, `choose` another class for `level.{level}.class`; \
                 {drop_last}."
            )));
        }
        _ => {}
    }
    let choice = address
        .choice()
        .expect("every address but the racial ability names an engine choice");
    let Some((edit, removable)) = engine.withdrawal(&build, &choice) else {
        let reason = match address {
            Address::Level(level) if (1..=last).contains(&usize::from(level)) => {
                format!("Only the last level, `level.{last}`, can be withdrawn.")
            }
            _ => format!(
                "The Build holds nothing at `{address}`: `sheet` prints what it holds, by address."
            ),
        };
        return Err(ToolError::no_such_choice(reason));
    };
    if !removable {
        // An item a choice needs: the engine names the choice ("Greater Two-Weapon Fighting needs Belt of …").
        if let Address::Gear(spot) = &address
            && let Some(reason) = crate::gear::needed_reason(engine, &build, spot)
        {
            return Err(ToolError::not_removable(format!(
                "{reason}, so `{address}` can't be withdrawn now, nor replaced by an item that doesn't keep that choice: \
                 withdraw the choice first, or `choose` an item that keeps it (`options` on `{address}` lists them \
                 open). `sheet` lists the choices only gear allows under `gear_dependent`."
            )));
        }
        return Err(ToolError::not_removable(format!(
            "A later choice depends on `{address}`, so it can't be withdrawn now: withdraw what depends on it first. \
             `sheet` with `removable: true` lists what can be withdrawn now."
        )));
    }
    let leaning = engine.newly_gear_dependent(&build, &edit);
    let mut next = build;
    engine.apply(&mut next, &edit);
    let next = reopen(engine, &next).ok_or_else(|| {
        ToolError::internal(format!(
            "The planner's own withdrawal of `{address}` made a Build its rules don't allow, so it was not applied. \
             This is a bug in the planner."
        ))
    })?;
    let sheet = engine.evaluate(&next);
    let leans_on_gear = match leaning.is_empty() {
        true => Vec::new(),
        false => crate::gear::dependent_lines(engine, &next, &sheet)
            .into_iter()
            .filter(|line| {
                (leaning.iter()).any(|choice| crate::choose::leaning_address(choice) == line.choice)
            })
            .collect(),
    };
    Ok(WithdrawResult {
        envelope: envelope(engine, &next, &sheet),
        leans_on_gear,
    })
}
