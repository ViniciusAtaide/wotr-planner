# Two text guardrails: one wording per refusal, and no "(s)"

Type: tooling
Status: open
Blocked by: [202](202-the-web-app-says-the-gear-notes-in-the-engines-words.md) (the Offers queue, 199 to 202, moves the reasons' wording into the engine)
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the retrospective of the queue of tickets 188 to 193 (2026-11-14; user: "do every recommendation"). Two kinds of wording defect were each found more than once by a review or a QA session, never by a check:

1. **The same refusal worded differently by two tools.** A rank the Build doesn't reach read one way in `choose` and two other ways in `validate`; a rank level that isn't a number likewise; found by ticket 191's review, again by the code review, again by the QA session. One test now compares the four tools for the unreached rank (`options_choose_validate_and_withdraw_say_alike_why_a_rank_is_not_reached`); nothing covers the other conditions.
2. **"level(s)".** Seven texts printed it until the QA session; `apps/mcp/src/words.rs` words the count now, and nothing stops the next one.

`CODING_STANDARDS.md` holds both as rules for the reviewer until this ticket's tests exist; strike them there when they do.

## Build

- **A parity table** in `apps/mcp/tests/server/`: rows of (a Build, an address, a refused value), each sent through every tool that can refuse it (`options` with `only_open: false`, `choose`, `validate`, `withdraw` where it applies), asserting one reason, allowing only each tool's documented tail (`choose`'s "Nothing was changed.", `validate`'s "declare …"). Start with the conditions the queue of 188 to 193 touched: a rank not reached (both causes), a rank level below its floor and past the last level, a value that isn't a number, an option already held, an unmet prerequisite with an `Any`, a prerequisite held in a later rank, a change that breaks a later choice. Where the tools disagree today, fix the wording to one function and say which.
- **No "(s)" in a text.** A test over the string literals of `apps/mcp/src` and `libs/rules/src` (read the sources as text; exclude comments and tests) fails on `(s)`. Fix what it finds with `words.rs` or the rules crate's own plural helper.

After the Offers queue: its tickets 199 and 200 move the later-rank reasons and the offers into the engine, so the table is written against the wording that queue leaves.

## Done when

The parity table passes with every row's tools saying one thing; the `(s)` test passes and fails when a literal with `(s)` is added (try it); the two rules are struck from `CODING_STANDARDS.md`; `scripts/check.sh` passes.
