# Importer S2, 306n: bonus feat lists III: sorcerer, Seekers, skald (user fact 281)

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306m
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306n-bonus-feats-sorcerer-seekers-skald.md`.

**What was built**
- Four feat lists of their own, written by the writer (`game-check --write option-lists --only` the four keys; every member of each game list maps to a feat; each has `ignores_prerequisites` false as the game's `IgnorePrerequisites`; each is named "Magic feat", so the sheet's label stays): `sorcerer/bonus-feat` (`SorcererBonusFeat` d6dd06f4…, 21), `sorcerer/seeker-sorcerer/bonus-feats` and `oracle/seeker/bonus-feats` (both `SeekerFeatSelection` c6b60927…, 26), `skald/bonus-feat` (`SkaldFeatSelection` 0a199953…, the same 26).
- Rows: sorcerer 1 and the Crossblooded, Empyreal, Geomancer, Nine-Tailed Heir, Sage and Sylvan 1 name `sorcerer/bonus-feat`. **The Seeker sorcerer's row 1 too** (the slice omits it; the game's `SeekerArchetype` removes only `SorcererBloodlineSelection` at 1, so it keeps `SorcererBonusFeat`); its 5, 10, 15 name its own list, as the oracle Seeker's do. Skald 1 and Battle Scion, Court Poet, Herald of the Horn and Inciter 1 name `skald/bonus-feat`. `archetype_losses.ron`: the Overwhelming Mage loses `sorcerer/bonus-feat` (the game's `OverwhelimgMageArchetype` 734917ec… removes `SorcererBonusFeat` at 1). Slot ids do not move (feat lists number apart from option selections; the new keys sort where `magic-feats` did among the row's feat lists, and no refusal is a slot-order one).
- `known.txt`: no line moves. The slice's finding is none today (the rows paired feat pools); `magic-feats -> 293` stays for the wizard's list. The game check exits 0 (no stale line).
- **Corpus generator:** no change (nothing under `tools/corpus/src/builds/` names these lists). Builds 2,156 before, 2,156 after, 1,990 open in both; `compare` names no lost or new Build.
- `libs/seeds/tests/game_data/data.rs`: the feat-list count 84 → 88 (found by the full run: `pinning-tests.sh` did not name it). `tools/corpus/tests/links.rs`: a Battle Scion's refused pick may be one of the ten Magic feats the skald's list lacks (`SKALD_BONUS_FEAT_LACKS`).

**User fact 281: applied, not held.** Ticket 281 (user, 2026-11-15) put the game's 21 over ticket 92's Magic feats for the sorcerer's level-1 feat and gave the Seeker its own 26, "applied without asking again". `user-facts.txt` holds `option lists / * / selection … -> applied 281` for `sorcerer/bonus-feat`, `sorcerer/seeker-sorcerer/bonus-feats` and `oracle/seeker/bonus-feats`; no `held` fact is touched. The skald's list is the game's, no fact of the user's. The game changes nothing the user decided: it is what they decided.

**Tests**
- `importer_lists::the_sorcerers_seekers_and_skalds_bonus_feats_are_the_games`: each list's blueprint and full member set by GUID (21 and 26), and the rows by level (sorcerer, seven archetypes, both Seekers at 5, 10, 15, skald, four archetypes; no row names `magic-feats`).
- Un-ignored: `report_gaps::every_sorcerers_level_1_bonus_feat_is_one_of_the_games_21` and `the_seekers_feats_at_5_10_and_15_are_its_own_26`, whose expected values were already the game's (SorcererBonusFeat's 21 and SeekerFeatSelection's 26, cited there). I wrote the data before running them, so I did not watch them fail first (they were ignored: the data held the 35 Magic feats). Sources: `m_AllFeatures` of the three selections in the pinned dump; `SorcererProgression` 997665565ca80a649aedd72455c4df1f at 1, both `SeekerArchetype`s' `AddFeatures` at 5, 10, 15, `SkaldProgression` 26418fed2bc153245972a5b54204ed75 at 1.
- `--list` prints all three.

**Proof (offers, base `c4eea6bf`, recorded in this lane before the first edit)**
- Links: 1,957 the same, 0 through an heir (no feat or option key changed; a selection key needs no alias), **33 refused**, 0 changed. The 33 link lines are one block of `306-refused.txt` (35 refused picks): sorcerer 21 lines (a sorcerer's, Sage's, Geomancer's or Seeker sorcerer's level-1 pick, or the Seeker sorcerer's at 5), skald 10, oracle Seeker 2.
- The game's reason, for every one: the pick is a feat the game's list doesn't hold. Level-1 sorcerer and skald picks refused: `dispel-synergy`, `destructive-dispel`, `arcane-armor-training`, `scribe-scrolls`, `brew-potions` (in neither of the game's lists), and `dispel-focus`, `ambuscading-spells`, `arcane-spirit` at a sorcerer's or Seeker sorcerer's level 1 (in the Seekers' and skald's 26 but not SorcererBonusFeat's 21: the Seeker sorcerer's level 1 is SorcererBonusFeat). The Seeker's picks at 5 and 10 (`brew-potions`, `dispel-synergy`) are not in SeekerFeatSelection's 26. Two General-slot picks of Arcane Armor Mastery (Draconic White and Fey sorcerers) are refused because Arcane Armor Training, its prerequisite, was the refused level-1 pick. None is a slot-order refusal.
- Offers: 18,251,092 options recorded, 18,248,426 now (2,760 only before: the Magic feats the new lists lack, each in a slot of the 209 Builds that hold one of the four slots; 94 only now: Spell Chain, which the Magic feats lacked and the Seekers' and skald's lists hold). Statuses changed: 39,317 in 328 (pick, kinds) groups, in exactly the 33 refused Builds (`FORM_PROOF_OFFER_BUILDS_OUT` names the same set): `ok -> BreaksLater` 39,031 statuses in 315 groups (the refused pick leaves the slot empty, so a later pick breaks it); `AlreadyHeld -> BreaksLater` 211 in 8 groups (the refused feat is no longer held); `ok -> Unmet` 71 in 3 groups (Arcane Armor Mastery, Greater Dispel Focus, Mythic Brew Potions rest on a refused feat); `BreaksLater -> Unmet` 3 in 1; `AlreadyHeld -> Unmet` 1 in 1. Every group is toward the game: the game's lists don't offer the picks the 33 Builds held.
- The other differences `compare` counts: 209 Builds' slot lines differ only in the selection key (checked by recording after and mapping the five keys apart: the remaining 84 differing slot lines are the same 33 Builds, whose generated picks follow the new lists), 17 `numbers` lines (a Spell Focus or Elemental Focus pick now counts), 33 sheet lines. No offer moves away from the game.
- Gold builds: no expected value changes (the gold tests pass; Jack's level-1 Spell Penetration is in the 21).

**Checks** (each ran once after the last source edit; fmt and the full run reran after the feat-list count): `cargo test -p seeds -p corpus`: all ok (seeds' game_data 585 passed, 0 failed, 110 ignored); `cargo test -p game-check`: 134 passed; `cargo run -p seeds --bin check-data`: checked; `cargo fmt --all --check`: 0; `scripts/test-modules.sh`: `test-modules OK`; the game check exits 0.
