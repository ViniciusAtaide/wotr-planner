# 306e: the bard's, skald's and Inciter's rogue talents are their own lists

## What the data says now

- `selections/bard/rogue-talents.ron` (`BardTalentSelection` 94e2cd84bf3a8e04f8609fe502892f4f, granted by
  `BardProgression` 8127f5ff40f5b484b8be98609358b9d2 at 2, 6, 10, 14, 18): 24 talents.
- `selections/rogue/skald-talents.ron` (`SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57, `SkaldProgression` at
  2, 7, 12, 17): 21 talents.
- `selections/skald/inciter/rogue-talents.ron` (`ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4,
  `ProvocateurArchetype` a1f5ab7eb78942f6889304acbd92e945 `AddFeatures` at 4, 9, 14, 19): 24 talents.
- Each copies `rogue/rogue-talents`' name, guide and `option_levels`, owns no option and shares the rogue's options
  that its game list holds (`shared_options`, written by `game-check --write option-lists --only ...`: 14, 17 and 14
  of the rogue's 38 removed). The summaries are ours.
- The rows of `classes/bard.ron`, `classes/skald.ron` and their archetypes that named `rogue/rogue-talents` name the
  copy of their class; `archetypes/skald/inciter.ron`'s four rows name the Inciter's.
## Decisions
## Decisions

- The skald's copy is keyed `rogue/skald-talents`, not `skald/rogue-talents` as the slice plan has it (main session, fix round): `game_data.rs` `normalise` sorts each row's selections by key, and slot ids number them in that order. The Battle Scion's level-12 row holds this selection and `skald/battle-scion/rage-power-or-teamwork-feat`; `rogue/...` sorts before it as the old key did, `skald/rogue-talents` would not and swapped the two slots. Slice 306h meets the same sort for the skald's rage-power copy.
- Picks are carried by slot, so no alias (decision 5).
- Extra Rogue Talent gives a bard or a skald no pick: `ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12 needs `RogueTalentSelection`, `SylvanTricksterTalentSelection` or `DarkLurkerRogueTalentSelection` (PrerequisiteFeature, Group Any), and its `needs_slot_of` names the rogue's selections. The feat's own prerequisite still reads the talents held, so the planner lets a bard take the feat itself; 306aa reworks `needs_slot_of`, S4 the prerequisite.


## Not modelled

- Each copy's own prerequisites (S4) and the Eldritch Scoundrel's Uncanny Dodge talents (308).

## Test

`the_bard_skald_and_inciter_talents_are_their_own_lists` (`libs/seeds/tests/game_data/importer_lists.rs`).
