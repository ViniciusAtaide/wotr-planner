# 306e: the bard's, skald's and Inciter's rogue talents are their own lists

## What the data says now

- `selections/bard/rogue-talents.ron` (`BardTalentSelection` 94e2cd84bf3a8e04f8609fe502892f4f, granted by
  `BardProgression` 8127f5ff40f5b484b8be98609358b9d2 at 2, 6, 10, 14, 18): 24 talents.
- `selections/skald/rogue-talents.ron` (`SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57, `SkaldProgression` at
  2, 7, 12, 17): 21 talents.
- `selections/skald/inciter/rogue-talents.ron` (`ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4,
  `ProvocateurArchetype` a1f5ab7eb78942f6889304acbd92e945 `AddFeatures` at 4, 9, 14, 19): 24 talents.
- Each copies `rogue/rogue-talents`' name, guide and `option_levels`, owns no option and shares the rogue's options
  that its game list holds (`shared_options`, written by `game-check --write option-lists --only ...`: 14, 17 and 14
  of the rogue's 38 removed). The summaries are ours.
- The rows of `classes/bard.ron`, `classes/skald.ron` and their archetypes that named `rogue/rogue-talents` name the
  copy of their class; `archetypes/skald/inciter.ron`'s four rows name the Inciter's.

## Decisions

- Selection keys follow the family's plan (306 slices, 306e); picks are carried by slot, so no alias (decision 5).
- Consequences the game shares: Extra Rogue Talent is refused to a bard and a skald (its `needs_slot_of` names only
  the rogue's selections; the game's Extra feat is met by the rogue's list alone, the comment of ticket 264's ignored test `the_extra_feats_the_game_refuses_to_an_underground_chemist_a_bard_a_skald_and_a_hexcrafter`), and a
  Battle Scion's level-12 slots swap places (inference: the merged row orders its selections by key; `rogue/...` came before
  `skald/battle-scion/...`, `skald/rogue-talents` comes after), so a link's two picks there no longer open: both are
  listed in `tools/corpus/tests/links/306-refused.txt`.

## Not modelled

- Each copy's own prerequisites (S4) and the Eldritch Scoundrel's Uncanny Dodge talents (308).

## Test

`the_bard_skald_and_inciter_talents_are_their_own_lists` (`libs/seeds/tests/game_data/importer_lists.rs`).
