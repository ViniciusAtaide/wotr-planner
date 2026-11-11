# The walk judges every choice: a cursor for the pickers

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 1, the top recommendation, *Strong*). The user asked for a ticket for each strong candidate.

The planner holds only possible Builds ([ticket 39](39-impossible-states-not-validation.md)): a picker offers only what the walk accepts. That promise rests on two implementations agreeing. The walk (`engine/walk.rs`, `walk/*`) judges a Build, and the pickers (`engine/pickers.rs`, `slots.rs`, `removable.rs`) build each option's verdict again from three parts:

1. **A probe for the state.** `self.probe(build, Probe::Slot(slot))` or `Probe::Level(level)` pauses the walk and returns its state (`pickers.rs:33`, `pickers.rs:342`).
2. **A check of their own:**
   - `check_pick(&Catalog, state, slot, pick)` for slot options (`slots.rs:213–342`). It picks one of `CharacterState`'s four check modes (`check`, `check_for`, `check_feat_for`, `check_option`; `state.rs:225–280`) at the call site (`slots.rs:326–341`).
   - `class_status` for classes (`pickers.rs:384–411`).
3. **`Recheck::later(&edit)`** for "breaks a later choice". It clones a paused `Walker`, overwrites `walker.build` and drives `walker.start()` and `walker.level()` itself (`removable.rs:308–356`).

**Where they already disagree in shape.** Class entry is judged twice, and each version has checks the other lacks:

| | `walk/level.rs:24–62` `check_class_entry` | `pickers.rs:384–411` `class_status` |
| --- | --- | --- |
| Class and archetype `entry_prerequisites` | ✓ | ✓ |
| Unknown class or archetype key, archetype of another class (`NotAllowed`) | ✓ | (only offers the class's own) |
| `advances` | ✓ | (`advances_options`, separately) |
| Class level cap | elsewhere in the walk | ✓ |

Whether that disagreement is a bug today or only a hazard is part of the ticket's first step.

**The `Walker` (`walk.rs:156–200`, about 25 fields) runs in five modes:** evaluate (the sheet), `failures`, `is_possible`, the three probes (`Slot`, `Level`, `Later`), and a recheck (`recheck: Option<RecheckWalk>`, `sheet: bool`). The probe hooks are interleaved into `fill` (`walk/hold.rs:40–59`) and `level` (`walk.rs:539–590`). Tests reach the paused state through a `#[cfg(feature = "testing")]` back door, `Engine::state_before` (`walk.rs:71–75`).

**The shape the review suggested** is a cursor on the walk:

- `until(slot | level)`: the walk paused just before a slot (or a level's class), with its state and slot kind.
- `try(edit)` on the paused walk: the candidate's verdict, `Result<(), Issue>`. This covers the slot's own check, in the mode the slot needs, and whether it breaks a later choice.

The pickers would ask the paused walk for each candidate's verdict and keep only presentation (grouping, names, chips, summaries: [ticket 97](97-slot-options-presented.md)'s `slot_entries`). Class entry would have one implementation, inside the walk. `Recheck` would use the cursor rather than `Walker`'s fields.

### The design questions (grill the user; take the recommendation where the user gives none)

1. **Is the cursor the right seam?** Alternatives:
   - **A.** The cursor above: `until` plus `try` on a paused walk. *Recommended*: one module judges, and the pickers keep their speed, because the walk to a slot still runs once per picker.
   - **B.** Keep the probe, but move every check the pickers do (`check_pick`, `class_status`, the check mode) into `walk/` as the walk's own functions, which the pickers call. This is a smaller change, but the pickers still assemble legality from parts.
   - **C.** `try(edit)` by re-walking the edited Build for each candidate (`Engine::allows`). This is simplest, but it costs a walk per candidate, and ticket 48's benchmark will refuse it for a feat slot's several hundred candidates.
2. **What the cursor returns.** The paused state for presentation (the pickers' `slot_lists`, `pools` and steppers read it), or only verdicts, with what the presentation needs given by the cursor too? *Recommended:* the state stays readable (`&CharacterState`), and only verdicts go through `try`.
3. **The walk's modes.** Should the `Walker` split along the modes, for example a judging core with the sheet's rows and the probe as observers? Or should the cursor sit over today's `Walker`, with the split left for later? Answer after step 1 of the Build. Split only what the cursor needs.
4. **Tests.** The test-world tests (`libs/rules/tests/engine/`, 54 files) already go through `Engine`'s public interface. Should the cursor be public, so tests can judge a candidate at a slot directly and `state_before` goes? *Recommended:* yes, if the UI or the MCP has a use for it too; otherwise `pub(crate)`, with `state_before` kept as it is.

Use `grilling` and `domain-modeling`, as the map's Notes say for design tickets. Name the cursor in `GLOSSARY.md` if it becomes a public engine term.

## Build

Once the design is decided. A refactor: the engine's answers don't change.
1. First, a test that pins today's agreement. For every test-world Build and every slot and level, each option a picker marks open must make an edit `Engine::allows`, and each one it marks closed must not.
   - Run it on the real data too (`libs/seeds/tests/game_data/`) over ticket 48's Builds.
   - Every disagreement it finds is a bug: record it in the Answer and ask the user before changing behaviour.
   - This test is the oracle the refactor keeps green.
2. Then the cursor, one picker family at a time: slots, then classes, then paths, first ascension and switch, then the start pickers ([ticket 96](96-start-and-level-pickers.md)).
- No schema, data or link-format change. `Engine`'s public interface stays the same, apart from what the design adds. If something has to move, say why.
- [Ticket 151](151-bonuses-in-force.md) also changes the walk: run the two one after the other, not in parallel.

## Done when

- Class entry is judged in one place, and a grep finds no prerequisite check (`state.check*`) outside `walk/` and `state.rs`.
- `Recheck` reads no `Walker` field.
- The agreement test from step 1 passes on the test world and the real data.
- `cargo test --workspace -- --list` lists the same tests, plus the new ones.
- `scripts/proof.sh rules` gives an empty comparison (or only the disagreements the user agreed to fix).
- Ticket 48's benchmark shows an edit and the level-1 picker no slower, within noise.
- `scripts/check.sh` passes.
