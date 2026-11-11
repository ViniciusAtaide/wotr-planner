# Checked game data: a type the engine requires

Type: task
Status: resolved
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

## Progress log

- 2026-11-11: worktree `worktree-agent-ae730a397139730fd` at `40fd29c`, with its own `.env`, database and `target/`. Baseline `cargo test --workspace -- --list` saved in the scratchpad (`tests-before-full.txt`: 1,117 tests with their test binaries). Callers re-grepped (tickets 148, 150 and 151 hadn't landed): `Engine::new` in the MCP `Planner`, the UI's `SheetPage`, the proof tool's `engine_on`, `test_world::engine`, two seeds real-data helpers and 36 sites in 18 files of `libs/rules/tests/engine/`; `GameData::check` in `seeds::check_game_data_files`, `seeds::replace_game_data` and the tests.
- 2026-11-11: `CheckedGameData` in `check.rs`, `GameData::checked`, `Engine::new(CheckedGameData)`; every caller routed through it. The rules tests build their engines through a new `test_world::engine_on(data)`, which panics with the problems. Five tests then failed on worlds that don't check: the three `totals` tests and two `indexed_lookups` tests (see Answer).
- 2026-11-11: browser timing with a release build made with `--lib-features timings`, served on a free port (60689) and loaded nine times each in headless Chromium (Playwright, a new context per load), at 1× and 4× CPU throttling; then native release. The machine was under heavy load from the other agents' builds (load average 85–95). Script and results in the scratchpad (`bench/`). Option A taken.
- 2026-11-11: the main session moved `main` to `4b1f164` (the wotr-build skill, and this ticket's file); the work was rebased onto it through a temporary commit, and the Progress log and Answer written into `main`'s copy of the ticket. Then proof, test list and `scripts/check.sh` (see Answer).
- 2026-11-11: review fixes. The proof records data that fails `check` (through `rules`' new `unchecked` feature, only `tools/proof` turns it on) with a warning, rather than aborting; the seed takes `CheckedGameData` and runs no check of its own; `totals.rs`'s extra items have minimal definitions of their own, not copies; the UI's banner says "1 problem"; `CheckedGameData`'s doc says what `into_data` and the canonical sort do; `class_skills.rs` panics with a message about its edited copy. Throwaway `scripts/proof.sh data` run (a stricter `check` rule that HEAD's data fails, with the data fix in the working tree, neither committed): side A warned with its 1 problem and was recorded; 2,163 Builds each side, only in A 0, only in B 0, differing 0. `scripts/proof.sh rules 4b1f164` (A = `main`, whose `rules` lacks the feature, patched in by `proof.sh`): 2,163 each, differing 0, `compare.txt` empty, `A.jsonl` and `B.jsonl` byte-identical. Test list unchanged (1,119, same as after the rebase); `scripts/check.sh` ALL-OK.

## Answer

**Done, option A.** `Engine::new` takes `CheckedGameData`, which only `GameData::checked` makes. Every caller goes through it, the browser included, and every engine a test builds is on data that passes `check`. The engine's answers are unchanged (the proof is empty) and so is the test list.

### The type

- **`CheckedGameData`** (`libs/rules/src/check.rs`, exported from `rules`): a tuple struct over `GameData` whose field is private to the `check` module, so nothing else in the crate or outside it can build one. It derefs to `&GameData` (no `DerefMut`), derives `Clone` and `Debug`, and has no `Deserialize`, so JSON can't skip the check. Outside `rules` it lends the data only for reading; inside, `pub(crate) fn into_data` hands it owned to the `Catalog`, which sorts it into canonical order (only a sort, so the check's rules still hold), as its doc says.
- **The proof's way past the check:** `impl From<GameData> for CheckedGameData`, behind a new `rules` feature, `unchecked`, which only `tools/proof` turns on (its doc and `Cargo.toml` comment say why). A `From` rather than a named constructor, so the harness's `Engine::new(data.into())` still compiles against crates from before this ticket, whose `Engine::new` takes `GameData` (there `into` is the identity). A workspace-wide build unifies the feature into every crate's `rules`, but an app's own build doesn't have it.
- **`GameData::checked(self) -> Result<CheckedGameData, Vec<DataProblem>>`**, beside `check`, which stays as it is: the tests that list problems (`check.rs`, `pick_values.rs`, …) keep asserting on `check()`.
- **`Engine::new(data: CheckedGameData)`**, and `Catalog::new` with it. The four `expect`s stay, their messages now ending "(CheckedGameData)", and the docs of `slot_list`, `slot_selection` and `slot_accepts` say the type guarantees what they rely on.
- **Naming.** "Game data" is what the glossary and the docs call the data throughout ("the current game data"), and `GameData` is its type; `GameData::check` is the step. `CheckedGameData` is that same game data after the step, so it reads at a call site the way the docs speak (`Engine::new(data.checked()?)`). A bare `Checked` would say nothing about what was checked next to the Build's own checks (a pick checked against its slot). The glossary is unchanged: it lists the game's and the planner's terms, not engine types (as for ticket 56's `Catalog`).

### The callers

- **MCP:** `Planner::from_data` is `Self::new(data.checked()?)`, `Planner::load` gets the checked data from `seeds::check_game_data_files`, and `Planner::new` takes `CheckedGameData`, keeping its own `Arc<GameData>` copy as before.
- **Seed:** `seeds::check_game_data_files` returns `CheckedGameData` (`read_game_data_files(dir)?.checked()`); `seed --check-only` reads its counts through the deref, and `seeds::seed` gets its data from it too: `replace_game_data` takes `&CheckedGameData` and runs no check of its own (the private `replace_unchecked`, behind it, keeps the foreign-key backstop's test).
- **Proof tool:** `engine_on` reads the files (`seeds::read_game_data_files`, as before), runs `check`, and builds the engine on the data whatever it finds, through the `unchecked` feature's `From`: side A of a `data` proof is the older data, which fails a check rule its ticket adds together with the data fix, and that change most needs its comparison. `record` and `export` print a `warning:` line with the problem count and each problem to stderr, and `proof.sh` shows the warning line and where the problems are listed. In `rules` and `both` mode, `proof.sh` adds `unchecked = []` to the ref's `libs/rules/Cargo.toml` when it lacks it, so the harness still compiles against crates from before this ticket.
- **UI:** `game_data::load` parses the JSON and returns `timed("check", || data.checked())`; a data problem shows in the page's existing error banner ("The game data didn't load: it has 1 problem: …", or "it has N problems, the first: …"). `SheetPage` is unchanged.
- **Tests:** the rules tests build every engine through `test_world::engine_on(data)` (new, beside `engine()`), which panics with the problems; the seeds real-data tests check `DATA` once (`CHECKED`, a `LazyLock`), `class_skills.rs` checks its edited copy (panicking with a message about that copy, not the test world), and the two seeds `database.rs` round trips that call `replace_game_data` check their data first. After the change, `GameData::checked` is called only in `rules` (`test_world`), `seeds` (`check_game_data_files` and four test sites), the MCP `Planner` and the UI's `game_data.rs`, and `CheckedGameData(` appears only in `check.rs`.

### The browser: option A

Measured with `libs/ui/src/timings.rs`'s `timed("check", …)` on the real data, in a release build made with `--lib-features timings` (`cargo leptos build --release`, `wasm-release` profile), served by the release server and loaded nine times each in headless Chromium (Playwright, a fresh context per load, the Fold-sized 412×915 viewport), median ms:

| | `check` | `Engine::new` |
| --- | ---: | ---: |
| Chromium, wasm, 1× CPU | **11.7** (11.1–12.2) | 4.0 |
| Chromium, wasm, 4× CPU throttling (ticket 48's phone stand-in) | 48.4 (44.8–50.9) | 18.4 |
| native release, 21 runs | 4.3 (4.2–5.5) | 1.2 |

The machine was busy with the other agents' builds throughout (load average 85–140), so these are on the high side. Under ~50 ms in a release build, so **option A**: the browser checks the data with the same code that guards the engine, and no constructor skips `check`. It adds about 12 ms to the first paint at 1× (about 48 ms at 4×, close to the line), once per page load, before the engine is built. If the 4× number ever matters, option B stays open: one named constructor used by `game_data.rs` alone. Script and results: the scratchpad's `bench/` (`check-timing.js`, `x1.json`, `x4.json`, `check_time.rs`, the native example, removed from the tree).

### Test worlds changed

Five tests built engines over worlds that fail `check`. None was removed or renamed; each now builds a world that checks, with the same assertions unless said:

- **`totals.rs`** (`initiative_is_dexterity_…`, `a_save_is_its_base_…`, `a_skill_is_its_ranks_…`): the bonuses named items the world lacks (Improved Initiative, Iron Will, Improved Initiative (mythic), Grace, the Pickpocket background, the Kindred half-elf subrace), and Stat Focus held for a stat without taking one (`HeldBonusWithoutSkillOrStat`). `engine_with` now starts from `world_with_bonus_items()`, which adds each with a minimal definition of its own (no prerequisite, feature, feat, class skill or slot; a Miscellaneous group or category; named by its key, as the parts already were) and gives Stat Focus a stat parameter (`PerValue`), as the guide's has. Only `totals.rs` needs them, so they stay there. The expected parts are unchanged.
- **`indexed_lookups::whether_a_class_casts_follows_its_archetype`**: the armiger casting arcane spells had no casting ability (`NoCastingAbility`); it now casts with Intelligence.
- **`indexed_lookups::a_class_level_gets_the_row_of_its_own_number_when_the_table_has_a_gap`**: `check` refuses a table with a gap (`LevelRowsNotContiguous`), so no engine can hold one and the engine-level test was pointless. It now asks `rules::effective_row` directly, which takes any class: on the fighter without row 2, level 2 has no row, level 3 is row 3 (no slot) and level 4 is row 4 (a Combat feat slot). The Bravery count it checked through the engine is gone with the engine.

`assumed.rs`'s `armoury()` and its other worlds already checked; they changed only to `engine_on` (a small overlap with ticket 148's changes there).

### Done when

- **The type:** `Engine::new` takes `CheckedGameData`; it is built only in `check.rs` (`GameData::checked`). No test builds an engine on data that fails `check`: the compiler requires the type, and `engine_on` panics otherwise.
- **Test list:** `cargo test --workspace -- --list` with the test binaries is the same before and after this change (1,117 tests at `40fd29c`, `tests-before-full.txt` and `tests-after-full.txt` in the scratchpad, empty diff). After the rebase onto `4b1f164` it has 1,119, and differs only by `308085e`'s MCP tests (`apps/mcp/tests/server/{listing,spells}.rs`): four `the_instructions_…` tests became `the_guide_…`, and `the_build_guide_is_the_skill_without_its_frontmatter` and `the_instructions_point_to_the_build_guide_and_stay_short` are new (`test-list-rebase.diff`). This ticket adds and removes none.
- **Proof:** `scripts/proof.sh rules` (A = `40fd29c`, B = the working tree, own target folder): 2,163 Builds each side, only in A 0, only in B 0, differing 0; `compare.txt` is empty and `A.jsonl` and `B.jsonl` are byte-identical (17,599,587 B). After the review fixes, again against `main` (`4b1f164`, crates from before `CheckedGameData`): the same, empty and byte-identical. A `data` proof on older data that fails a new `check` rule records it with a warning and compares (the review fixes' Progress log line).
- **Browser:** above; option A.
- **`scripts/check.sh`** (on the rebased tree): ALL-OK (fmt, clippy, wasm-clippy, test, rules-wasm, seed-check). `cargo doc -p rules` gives only the three warnings it gave before.
- No schema, data or link-format change; the worktree's database was built by `CLAUDE.md`'s steps, and `libs/seeds/.sqlx` is unchanged.

