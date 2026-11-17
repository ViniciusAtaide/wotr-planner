# Importer S2, 306g: slayer talents at 2, 6 and 10 (user fact 276)

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306f
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306g-slayer-talents.md`.

**What was built**
- New selections `slayer/slayer-talents-6` (`SlayerTalentSelection6` 43d1b158…, 38 offers) and `slayer/slayer-talents-10` (`SlayerTalentSelection10` 913b9cf2…, 39), copies of `slayer/slayer-talents` (`SlayerTalentSelection2` 04430ad2…, 37). Rows: `classes/slayer.ron` 6, 8 to the list at 6 and 10 to 20 to the list at 10; the Imitator, Arcane Enforcer, Spawn Slayer, Executioner, Stygian Slayer and Sanctified Slayer rows as the section's table (the keys `-6` and `-10` sort after `slayer/slayer-talents` and sit in rows holding one selection, so no slot moves).
- New options (names and summaries ours): `slayer/slayer-talents/vanguard-tactician` (`VanguardTacticianExtraUseTalent` c67db0b6…, needs the Vanguard Tactician feature, `ranks: 10` as the blueprint) and `slayer/slayer-talents/painful-strike` (`ExecutionerPainfulStrike` 052c8252…, grants `slayer/executioner/painful-strike`, `NoArchetype` of the five archetypes the blueprint names). The game check showed those prerequisites and the ranks as new differences, so they are written, not listed.
- Writer: `game-check --write option-lists --only slayer/slayer-talents,slayer/slayer-talents-6,slayer/slayer-talents-10,slayer/extra-slayer-talent`. The writer skips a selection with no list, so the two copies were seeded with one shared option (Combat Trick) first. It wrote the lists (Uncanny Dodge and Improved in all four) and moved the combat style options `-6`, `-10` to `slayer/extra-slayer-talent` (first selection in key order whose list holds them; keys unchanged).
- `user-facts.txt`: four `applied 276` lines (the three lists and `slayer/extra-slayer-talent`). `known.txt`: removed `option lists / * / selection slayer/slayer-talents -> 276` and `… slayer/extra-slayer-talent -> 293`. The game check exits 0.
- `extra-slayer-talent.ron`: `needs_slot_of` names all three lists (306aa removes the field); a Sanctified Slayer's first talent is now at 8 from the list at 6, and its Extra Slayer Talent stayed possible.
- `archetype_losses.ron`: the slayer archetypes' losses split by list (Bloodseeker loses all three; the others by the levels each list covers).

**Decisions:** Painful Strike's and the Vanguard's prerequisites written by hand (above); the slayer takes no Skill Focus talent, as the game's lists (the rogue-only repeating test now covers the rogue alone).

**Not done, deviates from the section:** five of the six tests the section un-ignores stay ignored, with new reasons: `a_slayer_may_take_the_first_combat_style_feat_at_6`, `a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style`, `a_sanctified_slayer_8_may_take_the_first_combat_style_feat` and `a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style` wait on stage S4: the combat style options still carry the old slayer-level prerequisites (`known.txt`, 276); `the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents` waits on 308 (Uncanny Dodge still needs the Eldritch Scoundrel). `a_slayers_talent_list_at_10_is_the_games_39` is un-ignored and passes. The gold build gap for the Enforcer's Uncanny Dodge (ticket 173) stays: the talent is in the list but not yet offered to a slayer (308). No gold build's gains changed.

**Tests:** `importer_lists::the_slayers_three_lists_are_the_games` (37, 38, 39 offers, Uncanny Dodge, Improved Uncanny Dodge, Painful Strike, Vanguard Tactician in each; SlayerTalentSelection2/6/10 and the talents' GUIDs), `importer_lists::the_slayers_extra_talent_list_is_the_games_41` (`ExtraSlayerTalentSelection` 53f7237f…); un-ignored `data::a_slayers_talent_list_at_10_is_the_games_39`. Updated for the split: `borrowed_selections` (the Sanctified Slayer's levels 8 / 16, 17, 20 by list), `extra_picks`, `nested_selections` (Extra Slayer Talent's list is the game's own), `repeating_talents` (Combat Trick's three lists; the slayer is offered no Skill Focus), `parameterised_feats` (the Vanguard talent repeats 10 times, `Ranks` 10), `prerequisites_as_components` (632 entries), `archetype_losses`.

**Proof (offers, `FORM_PROOF_OFFERS=1`, base `b765dbf6`, this lane):** links: 1,998 open, 1,960 the same, 0 through an heir, 38 refused, 0 changed; the 38 are listed in `tools/corpus/tests/links/306-refused.txt` (own block): every refused pick is a Weapon Focus, Skill Focus or Intimidating Prowess talent, which the game's slayer lists lack (the section's 38 open Builds). A first run had 2 more: a Sanctified Slayer's Extra Slayer Talent at 9, refused because `needs_slot_of` named only the list at 2; fixed above. Offers: 18,067,927 recorded, 18,032,450 now; only recorded 37,153, only now 1,676; 69,062 statuses changed in 344 groups, all in 39 Builds: the 38 refused plus `follow stale slayer sf one skill twice` (a slayer Build that isn't open on the base, holding changed slots). No slot-order refusal. `compare` fails its form assertions as it must for a data change.
