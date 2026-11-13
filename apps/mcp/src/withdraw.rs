//! `withdraw`: takes a choice back only by the engine's own Withdrawal ([`Engine::withdrawal`]), and only when the
//! engine says nothing later depends on it; the result passes the same final gate as `choose`'s ([`reopen`]).
//!
//! Two addresses are refused by design: `start.racial-ability`, which the engine keeps with the race (it has no
//! `Choice` of its own), and `level.N.class`, whose engine Withdrawal sets a class key no data has: an agent changes
//! a level's class with `choose`, and drops the last level with `level.N`.
//!
//! A Mythic rank taken later than the rank table's level comes back the same way (`mythic.R.level`: the rank is taken
//! as early as it comes again), unless a pick needs it where it is: then the refusal names that pick.
//!
//! A gear item comes off the same way (`gear.belt`, `gear.usable.<item key>`), unless a choice needs it: then the
//! refusal names that choice, as the engine words it ([`crate::gear::needed_reason`]).

use rules::{Address, Build, Engine};

use crate::code::{envelope, open, reopen};
use crate::error::ToolError;
use crate::options::{Breaks, verb};
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
            Address::RankLevel(rank) => unplaced_rank(engine, &build, rank),
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
        // A rank taken later that a pick needs there: the engine names the pick ("Shield Focus (Mythic) needs MR
        // 10 later than level 19").
        if let Address::RankLevel(rank) = &address
            && let Some(reason) = (engine.rank_level_options(&build, *rank).into_iter())
                .find(|option| option.earliest)
                .and_then(|option| option.reason)
        {
            return Err(ToolError::not_removable(format!(
                "{reason}, so `{address}` can't be withdrawn now: withdraw that choice first. `options` on \
                 `{address}` lists the levels MR {rank} can be taken at."
            )));
        }
        // The choices that would fail without it, by name.
        let breaks = Breaks::new(engine, &build);
        let refused = breaks.refused(&edit);
        if !refused.is_empty() {
            let count = refused.len();
            return Err(ToolError::not_removable(format!(
                "{} {} on `{address}`, so it can't be withdrawn now: withdraw {} first.",
                breaks.names(&refused),
                verb(count, "depends", "depend"),
                verb(count, "that choice", "those choices")
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

/// Why a Mythic rank's level has nothing to withdraw: the rank is taken where it comes at the earliest (the rank
/// table's level, or the rank below's when that one is taken later), the Build doesn't reach it, or there is no such
/// rank.
fn unplaced_rank(engine: &Engine, build: &Build, rank: u8) -> String {
    let Some(table) = engine.rank_table_level(rank) else {
        return format!("There is no MR {rank}: the Mythic ranks are 1 to 10.");
    };
    if !engine.rank_reached(build, rank) {
        let levels = build.levels.len();
        let comes_at = engine.rank_floor(build, rank).unwrap_or(table);
        return match usize::from(comes_at) > levels {
            true => format!(
                "The Build doesn't reach MR {rank}, so there is nothing to withdraw: MR {rank} comes at level \
                 {comes_at} at the earliest, and the Build has {levels} level(s)."
            ),
            false => format!(
                "The Build doesn't reach MR {rank}, so there is nothing to withdraw: its Path switch (Legend) ends \
                 the ascensions at MR {}.",
                rules::PATH_SWITCH_RANK
            ),
        };
    }
    let at = engine.rank_level(build, rank).unwrap_or(table);
    let nothing = format!("there is nothing to withdraw at `mythic.{rank}.level`");
    if at > table {
        let below = rank - 1;
        format!(
            "MR {rank} comes at level {at} at the earliest (MR {below} is taken at level {at}, and ranks keep their \
             order) and is taken there: {nothing}. To take MR {rank} earlier, take MR {below} earlier: `options` on \
             `mythic.{below}.level` lists its levels."
        )
    } else {
        format!(
            "MR {rank} comes at level {at} at the earliest (the rank table's level) and is taken there: {nothing}."
        )
    }
}
