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
- New `selections/bard/rogue-talents.ron` (`BardTalentSelection` 94e2cd84bf3a8e04f8609fe502892f4f), `selections/skald/rogue-talents.ron` (`SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57) and `selections/skald/inciter/rogue-talents.ron` (`ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4): each a copy of the rogue's name, guide and `option_levels`, with our own summary, `options: []` and the rogue's options as `shared_options`, then written by `game-check --write option-lists --only bard/rogue-talents,skald/rogue-talents,skald/inciter/rogue-talents` (3 fields, 3 files; log rows: bard 14 removed, skald 17, Inciter 14, so 24, 21 and 24 talents).
- Rows: `classes/bard.ron` and its seven archetypes name `bard/rogue-talents` (38 rows with the class's); `classes/skald.ron` and court-poet, battle-scion, demon-dancer, herald-of-the-horn, hunt-caller name `skald/rogue-talents`; `archetypes/skald/inciter.ron` 4, 9, 14, 19 name `skald/inciter/rogue-talents`.
- Existing tests re-pointed to the new keys: `archetype_losses.ron` (beast-tamer's loss is `bard/rogue-talents`), `borrowed_selections.rs` (`each_borrowing_class_picks_at_its_tables_levels`), `repeating_talents.rs` (the selections offering Combat Trick), `skald_rage_powers.rs` (talent keys; the Battle Scion's level-12 slots swap, see below).
- `known.txt`: nothing to delete, as the section says.

**Decisions:** keys as the section names them (user-approved slice list, main session); no alias (a link carries picks by slot, decision 5); summaries ours.

**Tests:** `importer_lists::the_bard_skald_and_inciter_talents_are_their_own_lists` (each count and each removed key, cited by GUID: the three lists above); `borrowed_selections::a_bard_and_a_skald_pick_from_their_own_talent_lists` un-ignored (276; 24, 21 and 24 offers).

**Proof (offers, `FORM_PROOF_OFFERS=1`, base `4570c9da`, this lane):** links 2000: 1965 the same, 0 through an heir, 35 refused, 0 changed. The 35 are listed in `tools/corpus/tests/links/306-refused.txt`. By name: bard 14, Inciter 2, skald 10, Battle Scion 7, and 2 bards with Extra Rogue Talent (the section expects bard 14, skald 12, Inciter 2: 28). The extra ones are consequences the game shares or the slot order makes:
- 2 names, a bard with Extra Rogue Talent at 3 (`t60 bard 3 extra rogue talent` and its full form): the feat's `needs_slot_of` names the rogue's selections, so it is now refused to a bard (offers: `extra-rogue-talent` ok -> Unmet, 33 Builds). The game refuses Extra Rogue Talent to a bard and a skald (comment of 264's ignored test); 306aa reworks the field.
- The Battle Scion's level-12 selections swap places (the merged row orders them by key: `skald/battle-scion/...` now sorts before `skald/rogue-talents`, where `rogue/...` sorted before it), so a link's two picks there no longer open (the 7 Battle Scion names, some of which also hold a talent the Battle Scion's list lacks).
Offers: 18,079,504 recorded, 18,067,121 now; only recorded 13,661, only now 1,278; 74,275 statuses changed in 340 groups, none in more than 35 Builds (the refused ones' later picks go BreaksLater, and Extra Rogue Talent). `compare` fails its form assertions as it must for a data change (1 corpus Build fewer, 346 decoded-pick differences).

**Checks:** game check exit 0 (1.2 s). The scoped checks are in the report.
