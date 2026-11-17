# 306aa: `needs_slot_of` goes; the Extra feats open their own list's slot whenever they are held

## What the data says now

- The field `needs_slot_of` of `FeatureSelection` is gone (302's stand-in), from the rules, the writer's field order and the 11
  `selections/**/extra-*.ron` files that named it. An Extra feat's slot is a slot of the selection the feat grants, at the level the feat is
  held, whatever slots the character had before: the game adds the picked selection to the level-up with its own list
  (`SelectFeature.Apply`, WrathDecompiled 6ce4a6b). The planner's old condition (the character must already have had a slot of one of the
  selections named, and an Extra selection with no list stood for the first of them) was the planner's own.
- `rogue/extra-rogue-talent` (`ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12) was the one Extra selection with no list: it now
  holds the game's 39, the rogue talents' 38 (`shared_options`) and Evasion (`rogue/rogue-talents/evasion`, 576933720c440aa4d8d42b0c54b77e80,
  owned by `rogue/dark-lurker/rogue-talents`). It is named "Rogue Talents" (the label the slot had) and takes its option level from
  `bard`, `rogue`, `shifter` and `skald` levels (the rogue talents' bard, rogue and skald, and the Feyform Shifter's own: the slot
  stood for those before).
- A Sylvan Trickster's or a Feyform Shifter's Extra Rogue Talent no longer takes the slot of its fey tricks or its rogue talents: the game's
  list is the rogue's 39 for everyone, so the fey tricks that aren't rogue talents (the witch hexes and the Hex Channeler's channel) are
  no longer offered there.

## Decisions

- `once_per_option` stays in the rules: `kineticist/expanded-element` and the Seeker's `sorcerer/seeker/second-bloodline` name it (306t,
  deferred, removes the first) (main session's slice: "if no data names it").
- `tools/game-check` no longer skips a selection with no list (`areas/options.rs`): every selection is compared now.
- The test world's three Extra selections (`rogue/extra-rogue-talent`, `witch/extra-hex`, `slayer/slayer-trick`) list what the class
  selection they stand for lists (`own_lists`), as the real data's do.

## Not modelled

- Who may take an Extra feat is its prerequisite's. The planner's still lets a bard or a skald take Extra Rogue Talent, an Underground
  Chemist Extra Discovery, a Hexcrafter Extra Hex and a Battle Scion Extra Rage Power (the game refuses each: its prerequisite needs a
  selection they lack); 308 (S4) writes the prerequisites. The tests that pin the refusals stay ignored, now on 308.

## Tests

`importer_lists::a_pick_in_extra_rogue_talents_slot_stands_for_a_rogue_and_a_bard`, `extra_picks::an_extra_feat_opens_its_own_list_for_any_class`,
`nested_selections::the_extra_feats_list_the_games_own_selections` (`libs/seeds/tests/game_data/`); rules:
`nested_selections::an_extra_feat_opens_its_own_list_for_any_class`, `extra_picks::the_slot_is_the_feats_own_whatever_slots_the_character_had_first`,
`extra_picks::a_character_with_no_slot_of_a_class_selection_still_gets_the_feats_slot`.
