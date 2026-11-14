# Every real-data test's expected value, from the game

Type: build
Status: open
Blocked by: 235, 236
Model: Opus to build: each value is read from the game and judged.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14). The user chose "b"; the recommendation was to update a test only when a ticket changes its fact.
- **The rule.** A test's expected value comes from the game, cited by blueprint GUID or by the decompiled code's file and method, never from the engine's earlier output. The user: "assume the game code works always".
- **The existing tests** were written from the guide: about 50 files in `libs/seeds/tests/game_data/`, besides the rules crate's own unit tests.

## Change

For each real-data test in `libs/seeds/tests/game_data/`:
- **Read the game** for each fact it asserts: the blueprint, or the code for a rule.
- **Where the game agrees,** add the citation beside the assertion: blueprint name and GUID, or file and method with the WrathDecompiled commit.
- **Where it differs,** change the test to the game's value. The data or engine then fails it, so fix that in the same change, or leave the test marked with the ticket that will fix it.
  - A difference with one of the user's facts (`research/skills-from-user.md`, `backgrounds-from-user.md`, the Aivu and Skeletal Champion tables) goes to the user first.
  - A difference already ticketed from [research/game-source-check-2026-11-14.md](../research/game-source-check-2026-11-14.md) is left to that ticket and named.

The gold builds (`gold.rs`) assert the user's write-ups, not the game. A gain there that the game contradicts goes to the user (234).

The rules crate's unit tests on hand-made data are not in scope: they test the engine's logic on fixtures, not game facts. A rule they encode that the game's code contradicts is listed for a ticket.

## Done when

- Every real-data assertion either cites its game source, or names the ticket that brings the data to the game's value.
- The Answer lists the differences found, with what each became.
- `scripts/check.sh` passes, apart from tests marked as waiting on a named ticket.
