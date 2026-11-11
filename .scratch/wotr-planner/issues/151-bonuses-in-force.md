# The walk records the Item bonuses in force

Type: task
Status: open
Blocked by: 148, 150
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 3, *Strong*). The user asked for a ticket for each strong candidate.

What a character holds is worked out twice:

- **In the walk:** the walk tracks holdings as it fills each slot (`Held`, `walk.rs:80–91`; `holdings`, `class_skill_holds`, `counted_features` on the `Walker`).
- **Again for the bonuses:** `Engine::held_bonuses` (`engine/caster.rs:282`) works it out from a `CharacterState` and a `BuildStart`, through `holdings()` (`caster.rs:305–392`). Race, subrace and background come from the start, and feats and features from the state.

Because of that:

- **`Engine::totals(start, state)`** (`totals.rs:97`) and **`Engine::caster_line(start, state, spell)`** (`caster.rs:100`) take a pair that callers must keep in step. The callers are `apps/mcp/src/totals.rs:60`, `caster.rs:188`, `compare.rs:128` and `rank.rs:96`: each passes `&build.start` beside a state taken from the sheet.
- **`totals.rs` imports `caster.rs`'s private helpers:** `use super::caster::{amount, modifier}` (`totals.rs:17`).
- **Expanded Arsenal is a special case:** `BonusAmount::SpellFocusOfAnotherSchool` gives 0 in the generic `amount` (`caster.rs:457`) and is summed apart (`caster.rs:420–443`).
- **The totals and caster tests build `CharacterState` by hand** (`libs/rules/tests/engine/totals.rs`). So nothing tests that the walk fills the state these sums read: `class_skills`, `feat_values`, `skill_ranks_at_least_mythic_rank`.
- **A new kind of bonus touches many files:** a new `BonusTarget` changes `bonus.rs`, `check.rs`, the summing code in `caster.rs` or `totals.rs`, and its consumer.

Tickets [139](139-attack-line-weapon-features-and-mythic-feats.md), [141](141-caster-line-scaling-bonuses-and-second-class.md), [143](143-switched-on-abilities.md) and [144](144-bonus-types-and-stacking.md) all add to these sums. 144's stacking rule ("the highest per type and target") needs every bonus in force in one list.

**Shape.** As the walk holds an item, it records the bonuses in force on the `CharacterState`:

- each bonus's item, its Parameter value, the amount resolved at that point (the Mythic rank, an ability modifier, ranks), and the part's display name;
- Expanded Arsenal's copy, resolved there too.

Totals and the Caster line then become sums over that list, and `start` leaves their interfaces: `totals(state)`, `caster_line(state, spell)`.

- An amount that depends on the final ability scores (a modifier) may stay unresolved and be summed at read time. Say which way, and why.
- The field's name is a suggestion. Pick one that reads well next to the glossary's **Item bonus**, and add it to `GLOSSARY.md` if it's a term callers see.

Out of scope: bonus types and stacking (144, which this prepares); the Attack line, which counts no item bonuses today (`attack_line(state, weapon)` has no `start` already). If 139 has landed by then, its weapon bonuses use the same list.

## Build

AFK. A refactor: the numbers don't change.
- Blocked by [148](148-naked-numbers.md), which removes `assumed_totals`, `assumed_caster_line` and `Assumed` from these files, and by [150](150-the-walk-judges-every-choice.md), which also reshapes the walk. Run it after both.
- Run it before 139, 141 and 143 where the queue allows, so their sums go into the one list.
- No schema, data or link-format change.
- The tests that built `CharacterState` by hand build a Build and walk it instead, through `Engine` (`evaluate`, then `totals` or `caster_line` on a level's state). They keep their assertions.

## Done when

- `Engine::totals` and `Engine::caster_line` take no `BuildStart`.
- `caster.rs`'s `holdings()` is gone, and `totals.rs` imports nothing private from `caster.rs`.
- No test of totals or the caster line builds a `CharacterState` by hand.
- `cargo test --workspace -- --list` lists the same tests, plus any new ones.
- `scripts/proof.sh rules` gives an empty comparison. If the proof doesn't record totals or caster lines, add them to its record first (one commit of its own) and say so.
- `scripts/check.sh` passes.
