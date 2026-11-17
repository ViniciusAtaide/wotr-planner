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

- **The three feats' prerequisites are the game's now** (fix round; main session, 2026-11-17, from the review: the slot opened for characters
  the game refuses the feat). Each names the class selection the game's feat names, as `Has(Selection)`, one `Requires` per game component
  (`PrerequisiteFeature`, Group Any), a blueprint several entries share as an `Any` of `Has` each (308's rule), then the cap:
  - `extra-rogue-talent`: ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12 needs SylvanTricksterTalentSelection
    290bbcc3c3bb92144b853fd8fb8ff452 (`rogue/sylvan-trickster/fey-tricks`, the selection and the feature that share its blueprint),
    DarkLurkerRogueTalentSelection 1edbb68422c046f98a86f4726f6721b2 (`rogue/dark-lurker/rogue-talents`) or RogueTalentSelection
    c074a5d615200494b8f2a9c845799d93 (`rogue/rogue-talents`, and the Feyform Shifter's `shifter/feyform-shifter/rogue-talent`, which has the
    same blueprint: FeyformShifterShifterArchetype 1cdfc7d306d1430eac19427539b62091 adds it at 8). A bard (BardTalentSelection
    94e2cd84bf3a8e04f8609fe502892f4f) and a skald (SkaldTalentSelection d2a8fde8985691045b90e1ec57e3cc57) are refused.
  - `extra-hex`: ExtraWitchHexSelection b6054088b4ab4be286724127cbf48b35 needs WitchHexSelection 9846043cf51251a4897728ed6e24e76f
    (`witch/hex`) or WinterWitchWitchHexSelection b921af3627142bd4d9cf3aefb5e2610a. **Inference (decision 2 of `research/308/decisions.md`):
    the second is left out**: it is an S2 split no Build can hold (no row grants it), so leaving it out refuses nothing a Build holds; its
    `known.txt` line is `-> 306o`. A Hexcrafter (HexcrafterMagusHexMagusSelection a18b8c3d6251d8641a8094e5c2a7bc78) is refused.
  - `extra-discovery`: ExtraDiscoverySelection 537965879fc24ad3948aaffa7a1a3a66 needs DiscoverySelection cd86c437488386f438dcc9ae727ea2a6
    (`alchemist/discovery`) only. VivisectionistArchetype 68cbcd9fbf1fb1d489562f829bb97e38 removes that selection at 2 and every even level, so
    a Vivisectionist is refused it and takes `extra-discovery-vivisectionist`; an Underground Chemist's UndergroundChemistSelection
    c3d08fdb481aa2e4a9f71c4349141826 is not named, so it is refused too.
  - Because the selection is held from the level its class grants it, whatever is picked in it
    (`LevelUpHelper.AddFeaturesFromProgression`, WrathDecompiled 6ce4a6b), a plain alchemist 2 meets Extra Discovery with nothing picked, as a
    rogue 2 meets Extra Rogue Talent: the old option-named prerequisites didn't.
- `rogue/rogue-talents/evasion` carries the game's own gate: Evasion 576933720c440aa4d8d42b0c54b77e80 has a `PrerequisiteNoFeature` of itself,
  written `Requires(HasNot(Feature("evasion")))` (the class rows grant `"evasion"`, as the option's level does; "an option moved into a wider
  list keeps its owner's gating", `research/306/decisions.md`). A rogue 2 and up is refused Evasion in Extra Rogue Talent's slot; a Dark
  Lurker, whose archetype removes it, is offered it, and once it holds it a second is refused. `tools/game-check/known.txt`: the check doesn't
  read a NoFeature of a blueprint several entries share, so it reports the planner's `not Evasion` as not in the game, `-> 308` (a new line);
  the `extra-discovery` and `extra-rogue-talent` lines (`-> 264`) went (they agree), and `extra-hex`'s narrowed to `-> 306o`.

## Not modelled

- A Battle Scion's Extra Rage Power (the game refuses it: BattleProwessSelection is not named) waits on 308, as does Extra Arcana's
  prerequisite (306o/306q).

## Tests

`importer_lists::a_pick_in_extra_rogue_talents_slot_stands_for_a_rogue_and_a_bard_is_refused_the_feat`,
`importer_lists::a_bard_and_a_skald_cant_take_extra_rogue_talent`, `extra_picks::an_extra_feat_opens_its_own_list_for_any_class`,
`extra_picks::the_extra_feats_the_game_refuses_to_an_underground_chemist_a_bard_a_skald_and_a_hexcrafter` (adds a Vivisectionist's Extra
Discovery), `extra_picks::a_plain_alchemists_and_magus_extra_picks_are_unchanged`,
`borrowed_selections::extra_rogue_talent_offers_the_rogue_talents_and_evasion` (Evasion refused to a rogue 3, offered to a Dark Lurker 5),
`option_lists::extra_feats_name_every_option_of_their_selections`,
`nested_selections::the_extra_feats_list_the_games_own_selections` (`libs/seeds/tests/game_data/`); rules:
`nested_selections::an_extra_feat_opens_its_own_list_for_any_class`, `extra_picks::the_slot_is_the_feats_own_whatever_slots_the_character_had_first`,
`extra_picks::a_character_with_no_slot_of_a_class_selection_still_gets_the_feats_slot`.
