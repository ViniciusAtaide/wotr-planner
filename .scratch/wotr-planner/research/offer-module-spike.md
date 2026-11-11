# One module for what a choice can take: spike (2026-11-11)

Report only, on `main` at `933be29` (after 148, 150 and 152). Ticket 151 is in flight in another worktree and changes `rank.rs` (`Number::of`, its `caster_line` call); that matters only where noted.

## Question

The architecture review of 2026-11-11 (at `40fd29c`) rated "one module for what can be taken at a choice" in the MCP server as worth exploring, and the map left it unticketed. After ticket 152 moved value keys and labels into `rules::Address`, is it still worth a ticket?

## What changed since the review

- `values.rs` (95 lines) and `address.rs` (370 lines) are gone. What `values.rs` built is now `ChoiceValue::{pick, ability, race, class}` in `shapes.rs`, with parameter keys from `rules::Parameter::value_key` and the class value's name from `Engine::class_value_name`. Addresses parse with `rules::Address::from_str`.
- `options.rs` is 606 lines (610 before), `choose.rs` 419 (420), `shapes.rs` 1,099 (1,076), `rank.rs` 350 (304, from 148's weapon pin in `improve`).
- `options.rs::offers` is the same 240-line dispatch (lines 106–345, plus `slot_offers`, `class_offers` and `entry_issue` to line 504). 150 left the pickers' interface as it was, so nothing in the MCP changed for it.
- `issue_code`/`issue_reason` still live in `options.rs`, and `ChoiceValue::matches` still lives in `shapes.rs`.

## Call path (before and after)

**`choose`, a Feature selection slot (`slot.selection.1.0`, `"value": "<option key>"`):**

1. `server.rs::choose` → `choose.rs::choose` → `code.rs::open`
2. `choose.rs::take` → `rules::Address::from_str`
3. `choose.rs::offers_at` → `options.rs::offers` → `options.rs::slot_offers` → `Engine::slot_entries`, `shapes.rs::ChoiceValue::pick`, `Engine::pick_name`
4. `choose.rs::find` → `shapes.rs::ChoiceValue::matches`
5. `choose.rs::apply_offer` → (refused) `options.rs::issue_code`, `issue_reason`; (open) `Engine::apply` → `code.rs::reopen`
6. on a miss: `choose.rs::missing_parameter` (calls `options.rs::offers` again, without values) or `choose.rs::not_offered` → `similar.rs::{nearest, closeness, distance, closest_note}`
7. `code.rs::envelope`

**`choose`, a class level (`level.2.class`, `{"class": "arcanist"}`):** the same, except step 3 goes `options.rs::class_offers` → `Engine::class_options`, `is_prestige`, `asks_advances`, `advances_options`, `class_value_name`, `shapes.rs::ChoiceValue::class`; step 4 adds `find`'s archetype leniency (a class taken with an archetype is named by `{class}` alone when that offer is its only one); a miss on a prestige class goes to `not_offered`'s "give `advances` too".

**`options`, either address:** `server.rs::options` → `options.rs::options` → `code.rs::open` → `Address::from_str` → `Engine::evaluate` → `options.rs::offers` (as above) → `Offer::search_text` → `issue_code`/`issue_reason` → `summary_of` (`Planner::summary`, class level only) → `code.rs::envelope` (with `envelope: true`).

**Module count** (MCP modules a reader opens, not counting `server.rs` and `error.rs`):

| Path | At `40fd29c` | Now |
| --- | --- | --- |
| `choose`, a hit | 5: choose, options, values, shapes, code | 4: choose, options, shapes, code |
| `choose`, a miss | 6: + similar | 5: + similar |
| `options` | 4: options, values, shapes, code | 3: options, shapes, code |

152 already took out the hop the review counted most against (`values.rs`). The bounces left are `choose.rs` → `options.rs` (for `Offer`, `offers` and the issue mapping) → `shapes.rs` (for `matches`), and back.

## Deletion test

| Piece | Size, callers | Deleting it would… | Verdict |
| --- | --- | --- | --- |
| `options.rs::offers` (+ `slot_offers`, `class_offers`, `entry_issue`, `no_such_choice`) | ~400 lines; 4 call sites in 3 modules (`options`, `choose` ×2, `rank::improve`) | spread 18 `Address` arms, 12 engine pickers, the steppers and the advances expansion into each caller | **Deep.** One small interface (engine, Build, sheet, address, `values`) over a lot. It is already the offer module in all but its file. |
| `ChoiceValue::matches` | 48 lines; one caller module (`choose`: `find` ×2, `missing_parameter`, `not_offered`) | put the matching back into `choose.rs`, its only user | **Shallow where it is, deep where it would go.** The rule is real (trim and case, archetype equal, advances as a set) but it sits in a JSON shape far from the `find` leniency that wraps it. Moving it concentrates; deleting it just moves it. |
| `issue_code` / `issue_reason` | 23 lines; 13 call sites in 5 modules (`options` 2, `choose` 2, `validate/mod` 1, `validate/tree` 2, `validate/wording` 6) | copy a 12-arm mapping into 3 places | **Shallow but earns its keep.** It's misplaced: it maps an engine `Issue` to the wire, as `error.rs`'s `ToolError` codes do, and `validate` imports it from `options`. |
| `code.rs::reopen` | 1 line; 3 callers (`choose`, `withdraw`, `validate`) | inline `engine.open_link(&encode_build(b)).ok()` three times and lose the name of the final gate | **Shallow, keep it where it is.** It names a concept (ticket 39's gate) and `code.rs` is the right home. Not part of this candidate. |
| `similar.rs` | 144 lines; 3 caller modules (`search`, `choose`, `validate`); its own unit tests | copy edit distance and word closeness into three tools | **Deep, and well placed.** Not part of this candidate. |

Small leaks the deletion test turned up:
- `Offer::search_text` (`options.rs:59`) and `choose.rs::plain_text` (`choose.rs:409`) are the same function on a value.
- "Parse the address, or `no_such_choice`" is written 4 times (`options`, `choose`, `rank::improve`, `withdraw`); "keep offers whose text holds every query word" twice (`options`, `improve`).
- "Take an offer" exists twice: `choose::apply_offer` (status check, `Engine::apply`, `reopen`) and inline in `improve` (status check, `Engine::apply`, no gate, since it returns no code).

## Change cost

(a) A new kind of value a choice takes. Two cases: a new parameter kind, as 125's stat and element were, and a new value shape (a `ChoiceValue` variant).
(b) A new tool that ranks a choice's options by another number, as `improve` does.

| Change | Files today | Files with the offer module |
| --- | --- | --- |
| (a) new parameter kind | rules only (`Parameter`, `value_key`, `parameter_from_key`, `value_left_open`); in the MCP, `server.rs` tool text and `shapes.rs`'s `ChoiceValue` doc. 125 touched `values.rs` and `code.rs`; both paths are generic now. | the same |
| (a) new value shape | `shapes.rs` (variant, `matches`), `options.rs` (`search_text`, a picker arm), `choose.rs` (`plain_text`, `take`'s routing), `validate/declare.rs`, `sheet.rs`, `server.rs`: **6** | `shapes.rs` (variant), the offer module (`matches`, the one text, the arm), `choose.rs` only if it needs its own walk, `declare.rs`, `sheet.rs`, `server.rs`: **5–6** |
| (b) new ranking tool | a new module (or `rank.rs`), `server.rs`, `lib.rs`, its shapes, `tests/server/<tool>.rs` + `main.rs`; and it copies ~35 lines of `improve` (address parse, query filter, skip closed and current, apply, evaluate) | the same files; it copies nothing (asks the module for the open offers, filtered, and for each one's Build) |
| (b) new number for `improve`'s `by` | `rank.rs` (`Number`), the line's module | the same |

The module saves no files on either change. It saves duplicated lines and one drift risk on (b): a second ranking tool copying `improve`'s loop.

## `shapes.rs`

- 1,099 lines, 54 pub types (53 serde derives); 17 of the 21 other source files import it.
- 4 tool modules define their own shapes now (the review counted 3): `rank.rs` 9 types, `caster.rs` 4, `attack.rs` 3, `totals.rs` 3, so 19 in all. `lib.rs` already says attack and caster carry "their own shapes", but `shapes.rs`'s header still claims "every shape a tool takes or returns is defined here".
- 25 commits by 21 tickets have touched it; 3 since the review (148, 150, 152).

Still a hub by churn and size, but not a hub a reader of `choose` must cross more than once: the choice-related part is `ChoiceValue` and the `options`/`choose`/`validate` shapes (lines ~445–916). The convention has already moved towards each tool's shapes beside its handler, one tool at a time. That move doesn't need this candidate. The one cheap fix is the header's false claim.

## Tests

- `options` and `choose` are tested only through rmcp: `tests/server/options.rs` (15 tests) and `choose.rs` (18), each through `Client::on_the_test_world()` (a duplex pipe, `Server::serve`, a client), plus `walks.rs`'s 500-walk property test. Of the crate's 150 server tests, none calls `offers`, `find` or `matches` directly. Only `similar.rs` and `validate/declare.rs` have `#[cfg(test)]` unit tests.
- One narrow rule that would test without a round trip: **`find`'s archetype leniency.** `{"class": "arcanist"}` names the offer `arcanist/brown-fur-transmuter` at a later level when it's the class's only offer, and names nothing when two archetype offers share the class. Today it's covered only in passing, by one real-data test (`real_data_augment_summoning_waits_for_spell_focus_conjuration`, its `level.2.class` step). `Offer`'s fields are public to the crate and `edit` is an `Option`, so a unit test can build offers by hand, with no engine.
- The seam already exists: `find` could get a unit test inside `choose.rs` today. The module would make it the natural place for such tests (`matches`' advances-as-a-set and `entry_issue`'s choice of reason too). It doesn't create a seam that's missing.

## Verdict

**Ticket a smaller piece.** 152 already did the review's biggest cut (6 modules → 5 on a miss, 4 on a hit), `offers` is deep and already one interface, and the change-cost test finds no file saved. What's left is locality, not depth: about 70 lines in the wrong place and a few duplicates. Scope, behaviour unchanged: move `Offer`, `offers` and its picker helpers out of `options.rs` into an `offers.rs`, together with `choose::find` and the body of `ChoiceValue::matches` (the shape keeps only its serde form), one value text (merging `Offer::search_text` and `plain_text`), and one helper for "parse the address, list the offers, filter by the query" that `options`, `choose` and `improve` share; move `issue_code`/`issue_reason` into `error.rs` (as a `ToolError` built from an `Issue`, which also folds `choose`'s and `options`' 5-line copies); add unit tests in `offers.rs` for `find`'s archetype leniency and `matches`' advances rule. Leave out the stepper walk (`step_to`, `skill_map`: `choose` is its only user, so moving it would not concentrate anything), `similar.rs`, `reopen` and any `shapes.rs` split, but correct its header. Run it after 151 merges: it edits `improve` in `rank.rs`, which 151 also touches (`Number::of`), and `options.rs`/`choose.rs` are otherwise quiet. It's small enough to ride with the next ticket that touches `options` or `choose` rather than queue on its own.
