# Importer S2, 306e: rogue-talent copies for bard, skald and Inciter

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306b, 306c, 313–320 filed
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306e-rogue-talent-copies.md`.
**What was built**
- New `selections/bard/rogue-talents.ron` (`BardTalentSelection` 94e2cd84bf3a8e04f8609fe502892f4f), `selections/rogue/skald-talents.ron` (`SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57) and `selections/skald/inciter/rogue-talents.ron` (`ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4): each a copy of the rogue's name, guide and `option_levels`, with our own summary, `options: []` and the rogue's options as `shared_options`, then written by `game-check --write option-lists --only bard/rogue-talents,rogue/skald-talents,skald/inciter/rogue-talents` (3 fields, 3 files; log rows: bard 14 removed, skald 17, Inciter 14, so 24, 21 and 24 talents).
- Rows: `classes/bard.ron` and its seven archetypes name `bard/rogue-talents`; `classes/skald.ron` and court-poet, battle-scion, demon-dancer, herald-of-the-horn, hunt-caller name `rogue/skald-talents`; `archetypes/skald/inciter.ron` 4, 9, 14, 19 name `skald/inciter/rogue-talents`.
- Existing tests re-pointed to the new keys: `archetype_losses.ron`, `borrowed_selections.rs`, `repeating_talents.rs`, `skald_rage_powers.rs`. `known.txt`: nothing to delete, as the section says.
- `form_proof.rs` `compare` now also counts the Builds a status changed in (`FORM_PROOF_OFFER_BUILDS_OUT` writes their names).

**Decisions:** the skald's copy is keyed `rogue/skald-talents`, not the plan's `skald/rogue-talents` (main session, fix round): rows' selections are sorted by key (`game_data.rs` normalise) and slot ids number them in that order, so the plan's key swapped a Battle Scion's two level-12 slots and refused 7 links; `rogue/...` keeps the old order. No alias (picks are carried by slot). Summaries ours.

**Tests:** `importer_lists::the_bard_skald_and_inciter_talents_are_their_own_lists` (each count and removed key, cited by the three lists' GUIDs); `importer_lists::a_bard_and_a_skald_cant_take_extra_rogue_talent` (a rogue's pick in the feat's slot stands, a bard's or a skald's is refused; `ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12 needs `RogueTalentSelection` c074a5d6…, `SylvanTricksterTalentSelection` 290bbcc3… or `DarkLurkerRogueTalentSelection` 1edbb684…); `borrowed_selections::a_bard_and_a_skald_pick_from_their_own_talent_lists` un-ignored (276); `corpus` `links::a_battle_scion_link_from_the_s2_base_still_opens`; `skald_rage_powers` slot order restored.

**Proof (offers, `FORM_PROOF_OFFERS=1`, base `4570c9da`, this lane):** links 2000: 1970 the same, 0 through an heir, 30 refused, 0 changed; listed in `tools/corpus/tests/links/306-refused.txt`: bard 14, skald 10, Battle Scion 2 (they hold Underhanded Trick, which the skald's list lacks), Inciter 2 (28, as the section expects) and 2 bards with Extra Rogue Talent at 3, which the game refuses (above); 306aa reworks the feat's `needs_slot_of`. Offers: 18,079,504 recorded, 18,069,368 now; only recorded 10,136 (the options the three lists no longer offer), **only now 0**; 62,182 statuses changed in 326 groups, all in 30 Builds, every one a bard, skald or Inciter Build (names from `FORM_PROOF_OFFER_BUILDS_OUT`; exactly the 30 refused Builds: later picks go BreaksLater, Extra Rogue Talent goes Unmet). The first record had 1,278 only-now offers: they were the Battle Scion slot swap, gone with the key change. `compare` fails its form assertions as it must for a data change.

**Checks:** game check exit 0; the scoped checks' last lines are in the report.
