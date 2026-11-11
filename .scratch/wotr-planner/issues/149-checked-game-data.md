# Checked game data: a type the engine requires

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 4, *Strong*). The user asked for a ticket for each strong candidate.

The engine relies on `GameData::check`'s invariants, but nothing in its interface asks for them. `Engine::new(data: GameData)` (`libs/rules/src/engine.rs:69`) takes any data. The invariants are then assumed in four `expect`s:

- `engine/slots.rs:118`: "a slot or an option row names a feat list of the data (GameData::check)"
- `engine/slots.rs:125`: "a slot names a selection of the data (GameData::check)"
- `engine/catalog.rs:99`: "an option's pool is a selection of the data (GameData::check)"
- `engine/pickers.rs:62`: "a slot names a selection of the data (GameData::check)"

Each caller decides on its own whether to check:

- **MCP:** `Planner::load` checks through `seeds::check_game_data_files`, and `Planner::from_data` calls `check()` before `Engine::new` (`apps/mcp/src/planner.rs:26–40`).
- **UI:** the planner calls `Engine::new(data)` on the JSON the server sends (`libs/ui/src/planner.rs:193`). That data was checked once, at seed time.
- **Proof tool:** `tools/proof/src/lib.rs:72` calls `Engine::new(data.clone())`.
- **Tests:** about 25 sites call `Engine::new`, some on data that doesn't pass `check`, and say so: "The bonuses may name items the test world lacks" (`libs/rules/tests/engine/totals.rs:22`). A test like that exercises an engine no app can build.

Make the invariant part of the interface:

1. `GameData::check` (`libs/rules/src/check.rs:531`, returning `Vec<DataProblem>` today) gives a checked value: `Result<Checked, Vec<DataProblem>>` (or keeps `check` as it is and adds a constructor beside it).
   - `Checked` is a suggestion. Pick the name that reads best next to the glossary's terms, and say why.
   - It derefs or lends `&GameData` for reading and can't be built any other way.
2. `Engine::new` takes only the checked value. The four `expect`s stay, but their message now refers to an invariant the types guarantee.
3. Every caller goes through it: the MCP `Planner`, the UI, the proof tool and the seed (`libs/seeds/src/{seed.rs:34, lib.rs:108}` already call `check`).
4. The tests build checked data. A test world extended with bonuses for items it lacks (`totals.rs:22`, `assumed.rs`) either adds those items or names real ones, so every engine a test builds is one an app could build.

**The browser.** Running `check` in wasm on every page load may cost time; ticket 48 cared about the first paint. Measure it with `libs/ui/src/timings.rs` (`timed("check", …)`) on the real data, and choose one:

- **A.** The browser checks the data too. *Recommended* when it costs under about 50 ms in a release build: the data the browser holds is then checked by the same code that guards the engine.
- **B.** A narrow way past `check` for data that was checked where it was made: one named constructor (`Checked::checked_at_seed(data)` or similar), documented, used only by `libs/ui/src/game_data.rs`, with a grep in the Done when that finds no other caller.

Out of scope: [ticket 98](98-check-validates-references.md)'s reference checks and anything `check` validates, which stay the same; the SQLite path (the review's speculative candidate 8, against the decision of 2026-11-06).

## Build

AFK. A refactor: the engine's answers don't change.
- `apps/mcp/src/planner.rs` changes too. Ticket 148 doesn't touch it, so the two can run in parallel worktrees.
- No schema, data or link-format change.
- Use the RustRover tools (`analyze_calls` on `Engine::new` and `GameData::check`) to find every caller. Re-grep, since tickets 148, 150 and 151 may add or remove some.

## Done when

- `Engine::new` takes the checked type, and no code outside the type's own module can build that type without running `check` (for option B, the one named constructor, with a grep showing a single caller).
- No test builds an engine over data that fails `check`.
- `cargo test --workspace -- --list` lists the same tests before and after, apart from tests added or ones a changed test world made pointless (each named in the Answer).
- `scripts/proof.sh rules` gives an empty comparison.
- The Answer gives the browser's measured `check` time and the option taken.
- `scripts/check.sh` passes.
