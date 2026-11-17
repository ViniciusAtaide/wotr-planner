# Importer S2, 306l: bonus feat lists I: magus, Arcane Rider, warpriest

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306k
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306l-bonus-feats-magus-warpriest.md`.

**What was built**
- Three new feat lists, written by hand: the writer skips a selection with a member that has no feat entry, and each list's game selection holds four (`AdvancedWeaponTraining2` 70a139f0…, `3` ee9ab011…, `4` 0b55d725…, `BleedingCriticalFeature` 42be7d46…). `magus/bonus-feats` (`MagusFeatSelection` 66befe7b…, 181 in the game, 177 here), `magus/arcane-rider/bonus-feats` (`ArcaneRiderFeatSelection` 8e627812…, 182 entries with one listed twice, 177 here: the same set as the magus's) and `warpriest/bonus-feats` (`WarpriestFeatSelection` 303fd456…, 155 in the game, 151 here). The game's `IgnorePrerequisites` is false for all three (the field's default). The members were the game's `m_AllFeatures` mapped through each feat's `blueprint`.
- Rows: `classes/magus.ron` 5, 11, 17 and the six magus archetypes' rows that named `feats`; `classes/warpriest.ron` and the five archetypes' rows that named `combat-feats`; each row held one selection, so no slot id moves. **Not in the section:** the Arcane Rider had no row at 5 and 17 (it inherited the magus's); it now has rows 5 and 17 (each keeping that level's Arcane Weapon grant) and row 11, all naming its own list (ArcaneRiderArchetype 2dff80a0… `AddFeatures` at 5, 11, 17).
- `known.txt`: removed 283's `progression rows / a bonus pick of another group / archetype magus/hexcrafter` (stale after the change); added 12 lines (the four members, for each list) under 306q's and 306r's file names, which add the feats. The game check exits 0 (4,096 differences in 197 groups, 0 not known).
- `archetype_losses.ron`: the Bladebound's and Spell Dancer's `feats` at 5 and the warpriest archetypes' `combat-feats` entries name the new lists; the Arcane Rider's loses `magus/bonus-feats`, replaced by its own. `data.rs`: 81 feat lists (was 78).
- **Corpus generator:** no change needed (nothing under `tools/corpus/src/builds/` names the magus's, the Arcane Rider's or the warpriest's list). Builds: 2,158 before, 2,158 after, 1,994 open in both. The generated picks of three Builds changed because the lists changed (`full warpriest`, its `/ legend`, `full t86 warpriest/feral-champion erastil / empty`: a slot's first offer moved to the game's list); they open.
- Decision (builder, no question to the user): `ignores_prerequisites` is left at its default rather than written `false`.

**Test:** `importer_lists::the_magus_arcane_rider_and_warpriest_bonus_feats_are_their_own` (cites MagusFeatSelection 66befe7b…, ArcaneRiderFeatSelection 8e627812…, WarpriestFeatSelection 303fd456…: the counts, `IgnorePrerequisites`, the rider's list equal the magus's, the warpriest's lacking the spell and metamagic feats, MagusProgression 5, 11, 17, WarpriestProgression 062b2157… 3 to 18, ArcaneRiderArchetype 2dff80a0… 5, 11, 17); it prints in `--list`. I wrote it after the data, so I did not watch it fail first.

**Proof (offers, base `c2a691c6`, re-recorded after a reboot in a temporary worktree):** links: 1,994 open, 1,974 the same, 0 through an heir, 20 refused, 0 changed; the 20 link lines are in `306-refused.txt` (own block). Each refused pick is absent from the game's list: none of `skill-focus` c9629ef9…, `extra-arcane-pool` 42f96fc8…, `history-of-terrors` 9e4c7d08…, `iron-will` 175d1577…, `great-fortitude` 79042cb5…, `deceitful` 231a3732…, `stealthy` c7e1d5ef… is in the magus's, the rider's or the warpriest's `m_AllFeatures`:
- magus 8 lines (plain 4: skill-focus at level 5; Arcane Rider skill-focus at 17; Armored Battlemage extra-arcane-pool at 5; Sword Saint history-of-terrors at 17; each plain and `/ legend`);
- Hexcrafter 6 (`t60 witch 1 hexcrafter 6 / filled` deceitful at 6; `t64 hexcrafter arcana thrice / before` and `/ legend` stealthy at 11; `t64 hexcrafter 20 / filled` and `/ legend` history-of-terrors at 17; `t64 hexcrafter hexes only` great-fortitude at 5);
- warpriest 6 (Champion of the Faith iron-will at 6 plus its rank-8 `iron-will-mythic`, which needs it; Feral Champion great-fortitude at 12; Proclaimer iron-will at 18; each plain and `/ legend`).
Offers: 18,058,975 recorded, 18,039,199 now; only recorded 22,098, only now 2,322 (the three Builds the generator now fills differently, and the new lists). Statuses changed: 45,599 in 315 (pick, kinds) groups, in 20 Builds, exactly the 20 refused names (`FORM_PROOF_OFFER_BUILDS_OUT` is the same set); every group but two is `ok -> BreaksLater` (the later picks that rest on the refused one; inference, as in 306j) and the others `AlreadyHeld` to `BreaksLater` (`deceitful`, 20 Builds) and to `Unmet` (`iron-will-mythic`, 3 Builds). No slot-order refusal; no offer moves away from the game. No gold build changed (`cargo test -p seeds` passes). The section expected 20 refused Builds (magus 12, Arcane Rider 2, warpriest 6) and rows −25: the census counts 20 link lines by the new rule.

**Checks:** `cargo test -p seeds -p corpus` ok (570 passed, 115 ignored in the seeds game_data crate), `cargo test -p game-check` (134 passed), `check-data` ok, `cargo fmt --all --check` ok, `scripts/test-modules.sh` OK, the game check exit 0.
