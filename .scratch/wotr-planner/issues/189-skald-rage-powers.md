# The skald can't choose its rage powers

Type: bugfix
Status: resolved
Blocked by: none
Model: Sonnet to build (data against the guide); Opus to review the archetype table.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. The skald's table in the guide has a Rage Power cell at 3, 6, 9, 12, 15 and 18, and its text says the skald adds the effects of a Barbarian's rage powers to Inspired Rage from 3rd level. The data grants `skald/rage-powers` as a feature on those rows (`libs/game-data/classes/skald.ron`), with no selection: a skald 20 is offered no rage power anywhere (`options` on `slot.selection.3.0` is `no_such_choice`). The feature's own summary says "chosen Barbarian rage powers".

The guide treats them as picks elsewhere too: the Battle Scion "whenever a Battle Scion can add a Rage Power to her song, … can instead take any teamwork feat", and the archetype summaries count them ("Gains fewer Rage Powers", "No Rage Powers", "gives up two Rage Powers").

## Build

- Read `guide-cache/skald.html` for the skald's and each archetype's Rage Power cells and any limit on which rage powers a skald may take; ticket [179](179-dragonheir-scion-gains-draconic-resistances.md) already read the Battle Singer's (6, 12, 18) and the Demon Dancer's (12, 15, 18).
- Give the skald's rows a slot of `barbarian/rage-power` (or a selection sharing its options, as ticket [25](25-shared-selection-options.md) did) at the guide's levels, levelled by the skald so a rage power's level prerequisite reads skald levels; the `skald/rage-powers` feature stays only if something still needs it.
- Each archetype at its own table's cells: Battle Singer, Court Poet (none), Demon Dancer, Inciter (gives up two), Herald of the Horn (keeps them), Hunt Caller, and the Battle Scion's teamwork-feat alternative (ask the user if the guide's sentence doesn't settle whether it is the same slot).
- Ticket [49](49-extra-feats-extra-picks.md)'s rule: whether Extra Rage Power then reaches the skald.

**Not a bug:** the skald's Rogue Talents at 2, 7, 12 and 17 are the guide's ("The Skald can select from the Rogue Talent list at 2nd, 7th, 12th and 17th levels"; ticket [60](60-borrowed-class-selections.md)).

## Done when

A skald 20 is offered a rage power at each of its table's levels and each archetype at its own; a real-data test pins the levels for the class and every archetype; the snapshot diff shows only the new slots; `scripts/check.sh` passes.

## Answer

Done 2026-11-13 (branch `ticket/189-skald-rage-powers`). Data and tests only: no engine, schema or codec change.

1. **The guide** (`skald`, each table's Rage Power column; the Battle Scion's "Battle Prowess" column):

   | Class / archetype | Guide's levels | The data gives |
   | --- | --- | --- |
   | Skald | 3, 6, 9, 12, 15, 18 | a `barbarian/rage-power` slot at each |
   | Battle Scion | Battle Prowess at 3, 6, 9, 12, 15, 18 | a slot of its own selection at each, offering the rage powers and Teamwork Feat |
   | Battle Singer | 6, 12, 18 (3, 9, 15 are its verses) | 6, 12, 18 (rows 3, 9, 15 drop the slot) |
   | Court Poet | none | none |
   | Demon Dancer | 12, 15, 18 (3, 6 and 9 hold Fiendish Maw, Fiend Totem and nothing) | 12 (own row), 15, 18 |
   | Herald of the Horn | 3, 6, 9, 12, 15, 18 | all six, from the base rows |
   | Hunt Caller | 3, 9, 12, 15 (6 and 18 hold its own powers) | 3, 9, 12, 15 |
   | Inciter | 3, 6, 12, 18 | 3 (base), 6, 12, 18 (own rows); 9 and 15 drop it |

   The page puts no limit on which rage powers a skald takes, and says nothing of which level a prerequisite reads ("Barbarian level: N" on the barbarian's page).
2. **What was built.** `skald.ron`: the `skald/rage-powers` feature and its references are gone; the skald's rows 3, 6, 9, 12, 15, 18, the Demon Dancer's 12 and the Inciter's 6, 12, 18 hold `selections: ["barbarian/rage-power"]` (a row lists all its selections, so 12 also lists `rogue/rogue-talents`). The Battle Scion has its own selection, `skald/battle-scion/rage-power-or-teamwork-feat`, named "Rage Power or Teamwork Feat": every `barbarian/rage-power` option in `shared_options`, one own option `.../teamwork-feat` ("Teamwork Feat", `repeat: Unlimited`, level-1 row `feat_slots: [Category(Teamwork)]`, no prerequisite) and `option_levels: ["skald"]`. Its rows 3, 6, 9, 12, 15, 18 hold it in place of the barbarian's selection (12 with the rogue talents) and keep Battle Prowess. `barbarian.ron`: the 14 level-gated non-Instinctual-Warrior rage powers read `Any([ClassLevel(barbarian, N), ClassLevel(skald, N)])`; nothing else (no option level, no new option; the selection has its 38 options). `feats.ron`: Extra Rage Power's `extra_pick_of` names the Battle Scion's selection too, so its extra slot is one of its own; its prerequisite doesn't name the Teamwork Feat.
3. **Decisions** (user, 2026-11-13, "do as the recommended for all questions", with the main session's): barbarian and skald levels don't stack (either class's level); the feature is deleted; the Battle Scion's teamwork feat shares the rage power slot (the guide's "instead"); the Demon Dancer's totem exclusivity is not modelled (its Fiend Totem and Greater are features); the rogue talent slot at skald 12 moves from index 0 to 1; the Battle Prowess feature stays on its rows; Extra Rage Power reaches the Battle Scion's selection. **Review fix** (Opus review of the first commit): the first shape put the Teamwork Feat option inside `barbarian/rage-power` with `option_levels: ["skald"]`. That was wrong twice over. `Archetype` is Build-wide and every pick through a selection with `option_levels` is a holding at the skald level, so a Battle Scion with barbarian levels got a real teamwork feat slot from a barbarian's, a Primalist's or an Extra Rage Power slot, which the guide doesn't give (the first Answer said it got none: wrong); and the option level wrote a state entry for every rage power any character holds, changing sheet hashes for no behaviour. The Battle Scion has its own selection instead.
4. **Tests.** New `libs/seeds/tests/game_data/skald_rage_powers.rs` (13): `the_skald_and_each_archetype_have_a_rage_power_slot_at_their_tables_levels` (also the Battle Scion's own slots at the six levels), `the_skald_rage_powers_feature_is_gone`, `a_rage_powers_level_prerequisite_reads_skald_levels_too`, `a_skalds_rage_powers_open_at_skald_levels`, `a_barbarians_rage_powers_are_as_before` (a plain barbarian's reason for Clear Mind at 4 is "Needs Barbarian level 8 (have 4)"), `barbarian_and_skald_levels_do_not_stack`, `a_rage_power_is_taken_once_across_barbarian_and_skald`, `extra_rage_power_reaches_a_skald`, `the_battle_scions_selection_shares_every_rage_power_and_adds_a_teamwork_feat` (shared options equal the barbarian selection's, read from the data; that selection has no option level and 38 options), `a_battle_scion_takes_a_teamwork_feat_in_place_of_a_rage_power`, `a_battle_scion_with_barbarian_levels_is_offered_no_teamwork_feat_in_a_barbarian_slot` (and a rage power is taken once across both), `the_rogue_talent_slot_at_skald_12_follows_the_rage_power_slot`, `a_locked_level_gated_powers_texts_name_the_classes_that_can_open_it`. Adjusted: `extra_picks` (Extra Rage Power's selections), `parameterised_feats` (the repeat table gains the Teamwork Feat, Unlimited). `instinctual_rage_powers` is as before.
5. **Snapshots.** Only `class-skald.txt` changed (1 of 44 Builds; the barbarian's didn't). The check's message, before regenerating, in its grouped form: the six `level.N.feature/skald/rage-powers = Rage Powers` lines gone (N = 3, 6, 9, 12, 15, 18); `slot.selection.12.0: Rogue Talents: - → Rage Power: -`; `+ slot.selection.N.0 = Rage Power: -` for 3, 6, 9, 15, 18 (5 lines); the new `.open` lists, wildcarded `slot.selection.*.0.open/barbarian/rage-power/animal-fury: + in 1 Build, 6 slots` (and the same for the other 17 no-prerequisite powers), `renewed-vigor` 5 slots, `clear-mind`, `increased-damage-reduction` and `internal-fortitude` 4 slots, `fearless-rage` 3 slots; and the rogue talents' `open` lines of the slot at 12, which moved to `slot.selection.12.1` (shown as `slot.selection.*.0.open/rogue/rogue-talents/…: - in 1 Build, 1 slot`). Every line follows from the ticket; regenerated.
6. **For the queue's proof** (what the review expects, and what was checked here):
   - (a) *Not checked directly.* Loading sorts a prerequisite's children, so for a barbarian below level 6 or 10 who holds another totem, Beast Totem's and Beast Totem, Greater's failed children come out in another order: the status differs, the reason text shouldn't.
   - (b) *Checked* (`a_locked_level_gated_powers_texts_name_the_classes_that_can_open_it`): the prerequisite as the data states it (the MCP `search` and `describe` text) reads "Barbarian level 8 or Skald level 8"; a Primalist's picker, which holds neither class, reads "Needs Barbarian level 8 (have 0) or Skald level 8 (have 0)". A barbarian's or a skald's picker, and the raw text of a failure, name only the class held ("Needs Barbarian level 8 (have 4)"). The MCP `validate` wording was not run.
   - (c) The rogue talent at skald 12 (base, Demon Dancer) moved to index 1. The Battle Scion's stays at index 0, its own selection being index 1.
