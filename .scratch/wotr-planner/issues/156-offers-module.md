# The MCP's offers in one module

Type: task
Status: open
Blocked by: 151
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 6, *worth exploring*) and its spike, [research/offer-module-spike.md](../research/offer-module-spike.md). The user asked for this ticket, scoped to the spike's smaller piece.

The review proposed one module for "what can be taken at a choice". The spike found that ticket [152](152-one-name-per-choice.md) had already done the biggest cut by deleting `values.rs`: a `choose` now goes through 5 MCP modules on a miss (4 on a hit), down from 6. It also found that `options.rs::offers` is already deep (about 400 lines behind one interface, 4 call sites in 3 modules), and that a whole new module would save no files on a realistic next change. What's left is locality: about 70 lines in the wrong place and a few duplicates.

- **Value matching lives in a JSON shape.** `ChoiceValue::matches` (`apps/mcp/src/shapes.rs:525`) holds matching logic, though `choose` is its only caller (through `choose::find`, `choose.rs:88`).
- **The error mapping is misplaced.** `issue_code` and `issue_reason` (`options.rs:79`, `:97`) are the general mapping from `Issue` to the wire, with 13 call sites in 5 modules (`validate` among them), but they live in `options.rs`. `choose` and `options` each have a 5-line copy of building a `ToolError` from them.
- **The value text is written twice.** `Offer::search_text` (`options.rs:59`) and `choose::plain_text` (`choose.rs:409`) are copies of each other.
- **One step is repeated in three tools.** "Parse the address, list the offers, filter by the query" is written out in `options`, `choose` and `improve` (`rank.rs`).
- **`shapes.rs`'s header is wrong.** It says every JSON shape is defined there, but 19 shapes now live in `rank.rs`, `caster.rs`, `attack.rs` and `totals.rs`.
- **The rules are tested only end to end.** All 33 `options`/`choose` tests go through a full rmcp round trip. `find`'s archetype leniency (`{class}` alone names a class's archetype offer when it's the only one) is covered only in passing, by one real-data test.

**Scope. Behaviour unchanged.**

1. **An `offers.rs`** holding:
   - `Offer`, `offers` and its picker helpers, moved from `options.rs`;
   - `choose::find`, with the body of `ChoiceValue::matches` (the shape keeps only its serde form);
   - one value text, merging `Offer::search_text` and `plain_text`;
   - one helper for "parse the address, list the offers, filter by the query", which `options`, `choose` and `improve` share.
2. **`issue_code`/`issue_reason` move into `error.rs`,** as a `ToolError` built from an `Issue`. That replaces `choose`'s and `options`' copies.
3. **Unit tests in `offers.rs`** for `find`'s archetype leniency and `matches`' advances rule, without an rmcp round trip.
4. **`shapes.rs`'s header** says what the file holds now.

Out of scope:
- the stepper walk (`step_to`, `skill_map`: `choose` is its only user, so moving it would concentrate nothing);
- `similar.rs`, which is deep and in the right place;
- `code.rs::reopen`, which is shallow but names the final gate;
- any split of `shapes.rs`.

## Build

AFK. A refactor.
- **Blocked by [151](151-bonuses-in-force.md),** which also edits `rank.rs` (`Number::of`). Start once 151 has merged.
- **Running alongside 153:** it may run in parallel with [153](153-stale-mcp-server-at-session-start.md) in its own worktree. The two share only `apps/mcp/src/lib.rs`'s module lines.
- **No change** to the MCP tools' arguments, results, error codes or messages. No schema, data or link-format change.

## Done when

- `offers.rs` holds the offers, matching, the value text and the shared listing step.
- `issue_code`/`issue_reason` live in `error.rs`, and a grep finds no copy of building a `ToolError` from an `Issue` outside it.
- `ChoiceValue` has no matching logic.
- The new unit tests appear in `cargo test -p mcp -- --list`.
- `cargo test --workspace -- --list` is otherwise the same.
- Every existing MCP server test passes unchanged.
- The snapshot check passes with the snapshot files untouched (`CLAUDE.md`, "Behaviour checks").
- `scripts/check.sh` passes.
