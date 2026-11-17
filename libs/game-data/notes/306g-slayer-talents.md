# 306g: the slayer's talents at 2, 6 and 10 are the game's three lists

## What the data says now

- `selections/slayer/slayer-talents.ron` (`SlayerTalentSelection2` 04430ad24988baa4daa0bcd4f1c7d118, 37 offers) keeps
  the slayer's talents at 2 and 4; new `slayer/slayer-talents-6` (`SlayerTalentSelection6`
  43d1b15873e926848be2abf0ea3ad9a8, 38) and `slayer/slayer-talents-10` (`SlayerTalentSelection10`
  913b9cf25c9536949b43a2651b7ffb66, 39) are the lists at 6 and 8, and from 10. Each copies the first's name, guide and
  `option_levels`, owns no option and shares what its game list holds (written by `game-check --write option-lists
  --only slayer/slayer-talents,slayer/slayer-talents-6,slayer/slayer-talents-10,slayer/extra-slayer-talent`).
- Rows: `classes/slayer.ron` 6 and 8 name the list at 6, 10 to 20 the list at 10; the Imitator, Arcane Enforcer, Spawn
  Slayer, Executioner, Stygian Slayer and Sanctified Slayer rows follow the same split (slice 306g's table).
- New options (names and summaries ours): `slayer/slayer-talents/vanguard-tactician`
  (`VanguardTacticianExtraUseTalent` c67db0b6ede802b43a6cf00136b1a5e6: needs the Vanguard Tactician feature, 10 ranks)
  and `slayer/slayer-talents/painful-strike` (`ExecutionerPainfulStrike` 052c82521039f7b4e978207eb533ead5: grants
  `slayer/executioner/painful-strike`, needs none of five archetypes).
- Uncanny Dodge and Improved Uncanny Dodge are in all three lists and in Extra Slayer Talent's (user fact 276, 2026-11-15,
  "Apply the game's"): `user-facts.txt` has the four `applied 276` lines.
- The combat style options moved to Extra Slayer Talent's own `options` (the first selection in key order whose list holds
  them, `OptionNotDefinedOnce`); their keys are unchanged. Extra Slayer Talent's `needs_slot_of` names all three lists
  (306aa removes the field).

## Not modelled

- The combat style options' prerequisites (known.txt, 276) and the two talents' Eldritch Scoundrel prerequisite (308):
  the five tests that wait on them keep their `#[ignore]`, with the reason.
- The ranks of Vanguard Tactician's talent as uses: nothing counts them.

## Tests

`importer_lists::the_slayers_three_lists_are_the_games`; un-ignored `data::a_slayers_talent_list_at_10_is_the_games_39`.
