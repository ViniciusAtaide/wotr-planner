//! `withdraw`: takes a choice back only by the engine's own Withdrawal ([`Engine::withdrawal`]), and only when the
//! engine says nothing later depends on it; the result passes the same final gate as `choose`'s ([`reopen`]).
//!
//! Two addresses are refused by design: `start.racial-ability`, which the engine keeps with the race (it has no
//! `Choice` of its own), and `level.N.class`, whose engine Withdrawal sets a class key no data has: an agent changes
//! a level's class with `choose`, and drops the last level with `level.N`.

use crate::address::{Address, NotAnAddress};
use crate::code::{envelope, open, reopen};
use crate::error::ToolError;
use crate::planner::Planner;
use crate::shapes::{BuildResult, WithdrawArgs};

/// `withdraw`: the Build without the choice, or why it can't come off.
pub(crate) fn withdraw(planner: &Planner, args: WithdrawArgs) -> Result<BuildResult, ToolError> {
    let engine = &planner.engine;
    let build = open(planner, &args.build)?;
    let address: Address = args
        .choice
        .parse()
        .map_err(|error: NotAnAddress| ToolError::no_such_choice(error.to_string()))?;
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
        return Err(ToolError::not_removable(format!(
            "A later choice depends on `{address}`, so it can't be withdrawn now: withdraw what depends on it first. \
             `sheet` with `removable: true` lists what can be withdrawn now."
        )));
    }
    let mut next = build;
    engine.apply(&mut next, &edit);
    let next = reopen(engine, &next).ok_or_else(|| {
        ToolError::internal(format!(
            "The planner's own withdrawal of `{address}` made a Build its rules don't allow, so it was not applied. \
             This is a bug in the planner."
        ))
    })?;
    let sheet = engine.evaluate(&next);
    Ok(BuildResult {
        envelope: envelope(engine, &next, &sheet),
    })
}
