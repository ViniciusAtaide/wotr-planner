# Nested selections

Type: build (data form and engine)
Status: resolved
Blocked by: 301
Model: Opus to build: the extra picks and the mythic abilities' selections change shape.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** `extra_pick_of`, `MythicExtraPick` and a mythic ability's own selections, where the game nests a selection inside the feat or ability that grants it.

## Change

- **Nested selections replace them:** a feat or ability grants a selection, and that selection's pick is its slot.
- **The engine questions 298's first build found** are this ticket's:
  - `extra_pick_of` gives a slot of the first listed selection reached; the game's extra-talent selections (ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12, the slayer's) name their own lists;
  - `once_per_option` does not reach across selections, which the third elemental focus (ThirdElementalFocusSelection e2c1718828fc843479f18ab4d75ded86) needs.
- **A rewrite script in the scratchpad (`game/302/`)**, from the current data.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

## Answer

**Built** (2026-11-16) on the main session's decisions (`<scratchpad>/game/302/decisions.md`), with the two of mine
below. Note: `libs/game-data/notes/302-nested-selections.md`.

**The form.** `Feat`, `MythicFeat` and `MythicAbility` each name the selection they grant (`selection:
Option<FeatureSelectionKey>`), and a pick of one opens a slot of it, as the game adds a picked selection to the
level-up's (`SelectFeature.Apply`, WrathDecompiled 6ce4a6b). `FeatureSelection` gains `mythic_abilities` (Extra Mythic
Ability's list) and `needs_slot_of` (the planner's rule until 293: the slot comes only after a slot of one of these;
a selection with no list of its own gives a slot of the first one had). `SlotKind` gains `MythicAbilities(key)`.
**Gone:** `Feat::extra_pick_of`, `MythicFeat::extra_pick`, `MythicExtraPick`, `MythicAbility::selections`. **Kept:**
every `SlotId` and its numbering, `selections_had` (what `needs_slot_of` reads), `once_per_option` (Q5), `option_lists`
(Q3, 304), `feat_lists_only`.

**The engine.** One path for a granted selection: `Walker::granted_slot` (the slot of `nested_selection`, unless a
Feature selection granted without a pick) for a feat at a level (`SlotId::FeatSelection`), a mythic feat at a rank
(`MythicExtra`) and a mythic ability at a rank (`MythicSelection { index: 0 }`); `Catalog::slot_kind` and `slot_band`
give a selection's slot kind and band. The judge and the picker read a `MythicAbilities` slot's list
(`Catalog::lists_mythic_ability`); labels give its selection's name ("Mythic ability"). MCP `bare_pick` reads the
mythic feat's selection; `describe` lists a mythic ability's one selection; the game check compares a selection's
mythic abilities and skips a first-had selection (293).

**The data** (rewrite `<scratchpad>/game/302/rewrite.py`, from `1387bbeb`, run again moves nothing): 18 fields moved
(9 feats, 7 mythic abilities, 2 mythic feats), 15 new selections:

| Selection | Granted by | Game blueprint | Form |
| --- | --- | --- | --- |
| `arcanist/extra-arcanist-exploit`, `paladin/extra-mercy`, `slayer/extra-slayer-talent`, `kineticist/extra-wild-talent` | the Extra feats | `ExtraArcanistExploitSelection` 0a7065e1…, `ExtraMercySelection` 8a49abed…, `ExtraSlayerTalentSelection` 53f7237f…, `ExtraWildTalentSelection` 0f73730c… | own list: the one selection each stood for, shared, with its header |
| `barbarian/extra-rage-power`, `rogue/extra-rogue-talent`, `witch/extra-hex`, `magus/extra-arcana`, `alchemist/extra-discovery` | the Extra feats | `ExtraRagePowerSelection` 0c7f01fb…, `ExtraRogueTalentSelection` 14d9089d…, `ExtraWitchHexSelection` b6054088…, `ExtraArcanaSelection` 00727883…, `ExtraDiscoverySelection` 53796587… | first had (marked planner list, 293) |
| `sorcerer/second-bloodline`, `bloodrager/second-bloodline`, `oracle/beneficial-curse`, `shaman/second-spirit` | the mythic abilities | `SecondBloodline` 3cf2ab2c…, `SecondBloodragerBloodline` b7f62628…, `BeneficialCurse` 2dda6742…, `SecondSpirit` 2faa8066… | today's list, the stood-for header, the band of the blueprint's group |
| `witch/wandering-hex` (`planner/`) | Witch Wandering Hex | none: `WitchWanderingHex` fb8490d1… is a plain feature | a pick of a witch hex, with its reason |
| `mythic/extra-mythic-ability` | Extra Mythic Ability | `ExtraMythicAbilityMythicFeat` 8a6a511c… | every mythic ability (63), band MythicAbility |

Extra Feat grants `feats`; Second Mystery and Impossible Domain keep `oracle/second-mystery` and
`cleric/impossible-domain`. Seeds fixture edited by hand; the test world's three granting feats name first-had
selections and Extra Mythic Ability its own.

**Decisions.**
- Main session: Q1–Q6 as in `decisions.md`.
- Mine: Extra Rage Power is first-had, not an own list. Q1's condition ("the game's equals today's") fails for it:
  its pick stood for two selections, and a Battle Scion skald's slot was the Battle Prowess selection, whose offer
  includes a teamwork option; an own list would change that offer (the game closes the feat to a Battle Scion, 264).
- Mine: the own lists keep `needs_slot_of`. Without it a Bloodseeker (never), Vanguard (from 6) or Deliverer (from 4)
  slayer taking Extra Slayer Talent before a talent slot would gain a slot it lacks today (the game gives it).
- Mine: the mythic abilities' new selections take the band of their blueprint's group
  (`SelectFeature.CalculatePriority`: Group None, Features; SecondSpirit ShamanSpirit, Path), not the stood-for
  selection's; a rank's slots are filled in order, so no pick moves (the proof below). Keys, and Extra Feat naming
  `feats`.
- Mine, from 301's review: the category-selection test, and the two long comment lines rewrapped.

**Proof.** `record` on `1387bbeb` in a temporary worktree, `compare` on the tip: 2,162 Builds (2,001 open), 2,162 old
links reopened, 396 differences (198 Builds' slots, and the same 198 reopened); `builds.tsv`, `sheets.txt`
and `numbers.txt` byte-identical; `slots.txt` differs on 226 of 49,631
lines, in 198 Builds, and the classifier (`<scratchpad>/game/302/classify.py` over the rewrite's `kind-map.tsv`) finds
all 226 differ only in the kind column as the map allows, 0 otherwise: 112 `MythicExtra` `MythicAbility` →
`MythicAbilities("mythic/extra-mythic-ability")`; `MythicSelection` 23 `witch/hex` → `witch/wandering-hex`, 20
`oracle/oracles-curse` → `oracle/beneficial-curse`, 10 `sorcerer/bloodline` → `sorcerer/second-bloodline`, 2
`shaman/spirit` → `shaman/second-spirit`, 1 `bloodrager/bloodline` → `bloodrager/second-bloodline`; `FeatSelection` 26
arcanist exploits, 16 mercies, 8 slayer talents, 8 wild talents → their Extra selections. Every corpus and bench link
opens with the same picks (`cargo test -p corpus --test links`).

**Game check:** exits 0. New: the six new selections whose list differs from the game's own (98 findings:
`arcanist/extra-arcanist-exploit` 6, `bloodrager/second-bloodline` 43, `kineticist/extra-wild-talent` 19,
`mythic/extra-mythic-ability` 4, `oracle/beneficial-curse` 18, `slayer/extra-slayer-talent` 8) are known under 293;
`paladin/extra-mercy`, `sorcerer/second-bloodline` and `shaman/second-spirit` agree with the game; the five first-had
selections are skipped (293); the option-list skip now names 304.

**Tests.** `libs/seeds/tests/game_data/nested_selections.rs`: `each_extra_feat_grants_a_selection_with_its_blueprint`
(the nine GUIDs above, each a `BlueprintFeatureSelection`), `the_own_list_extra_feats_list_the_selection_they_stand_for`
(`ExtraArcanistExploitSelection` = `ArcanistExploitSelection` b8bf3d50…'s 22, `ExtraMercySelection` = `SelectionMercy`
02b18703…'s 15), `the_first_had_extra_feats_name_the_selections_they_reach`,
`the_mythic_abilities_that_grant_a_pick_name_their_selection` (the six GUIDs; `WitchWanderingHex` a
`BlueprintFeature`), `extra_mythic_ability_and_extra_feat_grant_their_selections` (8a6a511c…, Group MythicAbility;
`ExtraFeatMythicFeat` e10c4f18… stood for by `feats`), `extra_mythic_ability_then_second_bloodline_nest_at_one_rank`
(`SelectFeature.Apply`), `the_category_selections_list_the_feats_of_their_category` (301's review; 293 replaces it).
`libs/rules/tests/engine/nested_selections.rs`:
`a_feat_granting_a_selection_with_its_own_list_gives_a_slot_of_that_selection`,
`a_selection_needing_a_slot_not_had_gives_none`, `a_first_had_selection_gives_a_slot_of_the_first_selection_had`,
`extra_mythic_abilitys_slot_offers_its_selections_list`, `a_mythic_ability_granting_a_selection_of_feats_gives_a_feat_slot`.
Rewritten onto the form with their values unchanged but the slot kind: `extra_picks`, `mythic`, `mercies_and_exploits`,
`borrowed_selections`, `option_lists`, `pick_values`, `ki_powers`, `infusions`, `skald_rage_powers` (`picks_of` reads
`needs_slot_of`), `pick_order` (+2 non-Features bands: `shaman/second-spirit` Path, `mythic/extra-mythic-ability`
MythicAbility, from the game's groups), `blueprints` (+1 selection without a blueprint, Wandering Hex's), and the
engine tests' struct literals and Extra Mythic Ability's kind.

