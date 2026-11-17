# Importer S2, 306m: bonus feat lists II: cavalier

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306l
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306m-bonus-feats-cavalier.md`.

**What was built**
- `mounted-combat-feats` is renamed `cavalier/bonus-feats` (file moved, every row naming it; `CavalierBonusFeatSelection` dd17090d…), now the game's 142 members. New `cavalier/tactician-feats` (`CavalierTacticianFeatSelection` 7bc55b5e…, 12 members) and `cavalier/gendarme/bonus-feats` (`GendarmeFeatSelection` ef1cd58e…, 151 members). `ignores_prerequisites` left at its default (false), as the game's `IgnorePrerequisites` (builder's decision).
- **Written by the writer:** `cavalier/tactician-feats` (log: `scratchpad/306m/lists.log`). **By hand:** the other two, because each game list holds four members with no feat entry (`AdvancedWeaponTraining2` 70a139f0…, `3` ee9ab011…, `4` 0b55d725…, `BleedingCriticalFeature` 42be7d46…: 306q, 306r); the members are `m_AllFeatures` mapped through each feat's `blueprint`, the four left out. The full member set is pinned by GUID.
- Rows: cavalier 6, 12, 18, disciple-of-the-pike 6, 12 and fearsome-leader 6, 12, 18 name `cavalier/bonus-feats`; the Tactician's list is named at cavalier 1, 9, 17, disciple-of-the-pike 1, 9, 17 and beast-rider, standard-bearer, cavalier-of-the-paw 1; the Gendarme's own list is at 1, 3, 9, 15, 20 and **new rows 6, 12, 18** (row 12 carries Demanding Challenge, which it inherited). Cavalier-of-the-paw row 8 keeps naming `cavalier/bonus-feats` (a row question, 309). Slot ids do not move (feat lists number apart from option selections; the compare's slot lines differ only in the selection key).
- `archetype_losses.ron`: the four archetypes that lose the Tactician's list name `cavalier/tactician-feats`; the Gendarme loses `cavalier/bonus-feats` at 6, 12, 18, replaced by its own. `support.rs`: `FEAT_POOLS` loses Mounted Combat; the category test drops that assertion. `data.rs`: 84 feat lists (was 81). README line updated.
- `known.txt`: removed 293's `mounted-combat-feats` and two stale cavalier-of-the-paw lines (273: "a bonus pick of another group" and "at another level"; the game check reported them stale, so the slice's "becomes 309's" has no line to move; the row-8 pick now shows under 273's existing "in planner only" line); added 8 lines (four members for each of the two hand-written lists) under 306q and 306r. The game check exits 0 (3,932 differences in 195 groups, 0 not known).
- **Corpus generator:** no change (nothing under `tools/corpus/src/builds/` names these lists). Builds 2,158 before, 2,158 after, 1,994 open in both. Eleven Builds' generated picks changed with the lists (cavalier, beast-rider, gendarme, knight-of-the-wall, standard-bearer each plain and `/ legend`, and `full option cavalier ... order-of-the-sword`); the same Builds' sheet lines read the new list names.

**Test:** `importer_lists::the_cavaliers_three_feat_lists_are_the_games`: each list's blueprint, `IgnorePrerequisites` false, and its member set equal to the game's GUIDs (CavalierBonusFeatSelection dd17090d… 146 entries less the 4; GendarmeFeatSelection ef1cd58e… 155 less the 4; CavalierTacticianFeatSelection 7bc55b5e… 12), then the rows (CavalierProgression aa70326b… 6, 12, 18 and 1, 9, 17; GendarmeArchetype 48fd6925… adds at 1, 3, 6, 9, 12, 15, 18, 20 and removes the cavalier's at 6, 12, 18; Fearsome Leader, Ghost Rider and Knight of the Wall remove the Tactician's). I wrote it after the data, so I did not watch it fail first. `the_standard_bearer_holds_the_cavaliers_own_banner_from_1` stays ignored: it still fails (banner gains [1, 10, 15, 20], a row question of 273).

**Proof (offers, base `5e8fdd70`, recorded in this lane before the first edit):** links: 1,985 the same, 0 through an heir, 9 refused, 0 changed; the 9 Builds are listed in `306-refused.txt` (own block, 9 link lines; 11 refused picks): `full cavalier ` (spell-chain at 9, not in the Tactician's list) and `/ legend`; `full cavalier cavalier/beast-rider` (spell-chain at 1, lightning-reflexes at 18) and `/ legend`; `full cavalier cavalier/gendarme` (iron-will at 18) and `/ legend`; `full cavalier cavalier/knight-of-the-wall` (lightning-reflexes at 18) and `/ legend`; `full option cavalier cavalier/order/order-of-the-sword` (spell-chain at 1). None of those feats is in the game's list. Offers: 18,039,199 options recorded, 18,038,927 now. Statuses changed: 21,241 in 223 (pick, kinds) groups, in exactly those 9 Builds (`FORM_PROOF_OFFER_BUILDS_OUT` the same set): `ok -> BreaksLater` except 4 groups `ok -> Unmet` (improved-lightning-reflexes, improved-iron-will, lightning-reflexes-mythic, iron-will-mythic, which rest on a refused pick). The 6 Builds whose numbers differ lost Lightning Reflexes or Iron Will to a refusal. No slot-order refusal; no offer moves away from the game. The slice expected 9 open Builds refused: the same. I added the row-12 feature after the compare; it was inherited before, so no offer or link moves (not re-recorded).

**Checks:** see the commit; each ran once after the last edit.
