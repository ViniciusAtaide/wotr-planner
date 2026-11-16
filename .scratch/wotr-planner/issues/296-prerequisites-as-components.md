# Prerequisites as components

Type: build (data form)
Status: open
Blocked by: 295
Model: Opus to build: every entry's prerequisite changes form, and the proof must show no behaviour moves.
Map: [WotR Build Planner](../map.md)

## Problem

The component vocabulary ([295](295-the-component-vocabulary.md)) gives the engine the game's prerequisite components. The user decided its staged plan on 2026-11-15. Each blueprint entry still holds the hand-shaped `prerequisite` field. This ticket moves them to the vocabulary's `Requires` form, in the same values and a new form, so the importer ([293](293-the-importer.md)) later writes one form.

The design is in the main session's scratch `vocab/design.md`, §3 and §4.

## Change

- **A rewrite script, kept outside the repo,** turns every blueprint entry's `prerequisite` into `Requires` components from the current data, not from the game.
- **`GameData::check`** refuses an entry holding both the field and the component.
- **The field goes** where nothing still needs it. Planner-made entries keep what they hold.

## Done when

- No blueprint entry holds `prerequisite` but through `Requires`.
- The proof is clean:
  - the corpus Builds' names, `Debug` and sheets hashes, and every link's decode, all identical before and after (as 244 did);
  - the self-checks, the gold builds and every real-data test unchanged;
  - `allows` identical over the corpus.
- The scoped checks pass, and the game check exits 0.
