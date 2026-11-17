# Importer S2, 306ab: the rows compare feat selections by GUID

Type: build (the importer, stage S2)
Status: open
Blocked by: 306l–306o
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

**Also, from 306j's rescue retro (main session, 2026-11-17):** `game-check --write option-lists` skips a selection named in `--only` that holds no list, and says nothing. 306j's `witch/extra-hex` had none and was skipped. List each such selection as unexpressed with the reason "holds no list to write (seed one shared option first)", and test it.
