# `sheet` has no narrow form

Type: feature
Status: open
Blocked by: [202](202-the-web-app-says-the-gear-notes-in-the-engines-words.md) (the Offers queue, 199 to 202, edits `apps/mcp`)
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the retrospective of the queue of tickets 188 to 193 (2026-11-14; user: "do every recommendation"). `sheet` returns every level, every Mythic rank and the totals. For a level-20 Build that is too large for a spot check: the main session skipped a one-fact verification through it (does a Defender of the Hearth gain Weapon Training once at level 9?), and the QA session, which reads sheets all day, used about 664,000 tokens.

## Build

- `sheet` takes `levels` (a list of character levels) and `ranks` (a list of Mythic ranks): given either, the answer holds only those levels' and ranks' entries, with the Build start, `totals` and the envelope as now. Left out, the answer is as today, byte for byte.
- A level or rank the Build doesn't have is refused with a reason in the words the other tools use for it (`apps/mcp/src/unreached.rs`).
- The tool's description gains one clause; the `wotr-build` skill's text says to ask for the levels a question is about (the server embeds the skill, and a test holds the instructions' length).

## Done when

`sheet` with `levels: [9]` on a fighter 20 returns level 9 alone; without the arguments an existing sheet test's output is unchanged; tests pin both and the refusal; `scripts/check.sh` passes.
