# MCP ergonomics from the Demon Greatsword run

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The Demon Greatsword run (2026-11-08, see [112](112-effect-summaries.md)) took more calls than it needed. Some of them went to facts the server already gives.

| What happened | Cause |
| --- | --- |
| 45 one-rank skill steps to fill levels 6–20 | `level.N.skill.<skill>` takes one skill per step |
| A batch refused: alignment before faith | The `todo` order was right; the instructions don't say that faith comes first |
| A batch refused: scores assumed to start at 7 | `sheet` gives `point_buy_spent` and `point_buy_budget`, and the stepper names its points; the instructions don't say scores start at 10 |
| Not knowing which character level MR 4 falls at | `sheet` gives each rank's `character_level`; the agent never called it |
| Improved Critical (Mythic) without its weapon: `not_an_option`, with Sai, Bite and Bomb as the "closest" | A parameterised key without its parameter gets the general error |
| `compare` on the same code twice, just to get a one-Build summary | `compare` takes 2 to 4 codes |

## Scope

1. **Skill plans:** `level.N.skill` also takes a map, `{"athletics": 1, "perception": 1, "mobility": 1}`. Each rank is still one checked step, and the first refused rank stops the step as today.
2. **A missing parameter:** a parameterised key given bare is answered `needs_parameter`, with what it takes ("a weapon") and its open values.
3. **One-Build summary:** `compare` takes 1 to 4 codes; with one it returns the summary and no differences.
4. **Instructions, a "Before you plan" paragraph:**
   - scores start at 10 with a 25-point budget, and `sheet` shows what's spent;
   - the faith comes before the alignment;
   - `sheet` gives each Mythic rank's character level;
   - branch class splits and `compare` them before committing to one ([115](115-plan-ahead-class-splits.md), option A).

## Open questions (with recommendations)

1. **A level range for skills** (`level.6..20.skill`)? *Recommend:* not yet. With item 1, a 20-level plan is 20 steps, and a range adds an address form to the grammar.

## Done when

Server tests cover the skill map (with a refusal partway through), `needs_parameter`, and a one-code `compare`. The user reviews the instructions text, as in ticket 110. `scripts/check.sh` passes.

## Choices settled

The user approved building this with the recommended option (2026-11-08): no level range for skills.

1. **The skill map** is a new value, `{skill: ranks}` (`ChoiceValue::Ranks`), taken only at `level.N.skill`. Each number is the ranks the level puts in that skill, as `level.N.skill.<skill>` takes it (the number to reach, not ranks to add), so sending a map again changes nothing. Each skill is moved with the same stepper walk as `level.N.skill.<skill>`, one checked rank at a time. The skills lowered go first, to free their points, then the rest in key order (a JSON object's order isn't kept). **The map is one step:** the first refused rank refuses all of it, with the stepper's reason naming that rank ("At Perception ranks at level 1 0 → 1: …"), and nothing of the step is applied, as a refused score or rank move is today. A map at any other address is `not_an_option`, naming `level.N.skill`.
2. **`needs_parameter`** (a new server code): a key given alone where the slot offers that key only with parameters. The reason gives what a value is (the entry's own noun, "a weapon"), how to give it (`{"key":"weapon-focus","parameter":"bardiche"}`, with the first open value), and every open value's parameter; it counts the closed ones, or, with none open, points to `options {values: true, only_open: false}`. All open values are listed, however many (73 weapons for Weapon Focus on real data).
3. **`compare`** takes 1 to 4 codes. One code gets its summary and empty differences, which the diffing gives without a special case; only the count in `compare.rs` changed. `invalid_arguments` now says "1 to 4".
4. **Instructions:** a "Before you plan" paragraph sits between the opening line and the workflow (text in the Answer). Its class-split bullet is ticket 115's (option A), committed with 115. The Values paragraph also names the skill map.
5. The `choose` and `compare` tool descriptions, `ChooseStep`'s and `Rejected`'s docs, and `CompareArgs`' doc say the same.

## Progress log

- 2026-11-08: Built on 116 (e5c52fc). Wrote the tests first, and all six failed: `tests/server/choose.rs` gains three (the skill map, a refused rank in a map with the code unchanged, and `needs_parameter`), `tests/server/compare.rs` gains one (one Build) and its count test now takes 1 to 4, and `tests/server/listing.rs` gains one (the paragraph, before the workflow).
- Wrote the `Ranks` value, the map walk and `missing_parameter` in `choose.rs`, the `needs_parameter` code, the count in `compare.rs`, and the paragraph. All passed.
- Smoke test of the binary on real data: a human fighter's `level.1.skill` map of three skills stopped at the third rank (a fighter has 2 points at Intelligence 10) and changed nothing; bare `weapon-focus` came back `needs_parameter` with "takes a weapon" and the open weapons; `compare` with one code gave the summary and empty differences.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. `level.N.skill` takes a map `{skill: ranks}`, each rank one checked step, the whole map one `choose` step that a refused rank stops with nothing applied. A parameterised key given alone gets `needs_parameter`, with what it takes and its open values. `compare` takes 1 to 4 codes; one gets its summary and no differences. The instructions gain "Before you plan". 82 server tests pass (5 new).

The new paragraph, for the user's review (with ticket 115's last bullet, as it stands after 115):

> Before you plan:
> - Point-buy scores start at 10 and run from 7 to 18 on a 25-point budget; `sheet` gives `point_buy_spent` and `point_buy_budget`.
> - Choose the faith (a deity, or atheism) before the alignment: no alignment is open until the faith is chosen, and a deity allows only some.
> - `sheet` gives each Mythic rank's `character_level`, the character level its ascension comes at.
> - A class split decides what the last levels can take: Fighter 5 / Barbarian 15 gives up Barbarian levels 16 to 20 and whatever needs them. Before you commit to a class split, branch the candidates from one code (Barbarian 20, and Fighter 5 / Barbarian 15), add their levels, and `compare` the branches: it lists the feats and features each gains that the other doesn't. `options` with `only_open: false` says what a closed option still needs ("Needs Barbarian level 16 (have 14)").

And the Values paragraph's last sentence, also for review: "A point-buy score or a skill's ranks take the number to reach; `level.N.skill` takes a map `{skill: ranks}` to spend a level's skill points in one step."
