# Importer S4, 308e: the rogue's, slayer's, alchemist's, magus's, arcanist's and small families' prerequisites

Type: build (the importer, stage S4)
Status: resolved
Blocked by: 308d
Model: Opus to build. Its gold change goes to the user before the merge.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [308](308-importer-prerequisites.md), the importer's stage S4, scoped to ranks 1 and 7 of [the queue's scope](../research/queue-scope-2026-11-17.md) (user, 2026-11-17). The goal, the edit list, the tests by name, the proof and the expected counts are under its heading in [research/308/slices.md](../research/308/slices.md); that file's §0 and [research/308/decisions.md](../research/308/decisions.md) hold the decisions. A data slice also follows [research/306/decisions.md](../research/306/decisions.md): the generator follows the game with the Build count held, refusals are counted in link lines, a moved option keeps its gate.

## Done when

Its section in `slices.md` is built and proved as written there, as amended by `decisions.md`, and the scoped checks pass.

## Answer

Built on base `7e3a9a63`. The note is `libs/game-data/notes/308e-rogue-slayer-alchemist-prerequisites.md`.

**What was built**

- **The run:** `game-check --write prerequisites --only 'selection option rogue/*,selection option slayer/*,selection
  option alchemist/*,selection option magus/*,selection option arcanist/*,selection option ranger/*,selection option
  hunter/*,selection option paladin/*,selection option monk/*,selection option stalwart-defender/*' --log` wrote 59
  options. 54 stay written:
  - rogue 6: Uncanny Dodge, Improved Uncanny Dodge (the user's applied facts, 276), Blinding Strike, Improved Evasion,
    Mystery Seeker, Survivalist;
  - slayer 13: the ten nocturnal abilities, the three combat style talents;
  - alchemist 9, magus 10, arcanist 2 (Potent Magic, Swift Consume), hunter 2, paladin 3, monk 1, Stalwart Defender 3;
  - ranger 5: the 6th-level style lists (widened to every planner entry of the 2nd-level blueprint).
- **Put back: the five `ranger/combat-style-feats-10/*`** (mine; finding below). They keep the base's tree.
- **Left whole by the writer, with their owner** (`known.txt` re-homed to what the writer prints):
  - 307: the 13 bomb discoveries (ExplosiveExpert 8a551fcca93446f0adfbe5e57e5bc636 grants the bomb feature,
    `mythic ability explosives-expert` doesn't), as 308h's dry run said;
  - 307: 11 rogue talents (Confounding Blades, Crippling Strike, Dispelling Attack, Emboldening Strike, the three
    Focusing Attacks, Petrifying Strike, Slow Reactions, Weakening Wound, Wearying Strike): TaintedSneakAttackFeature
    e6ce101a94ac9034b8b55c546e74b9dd grants SneakAttack, `lich/lich-powers/tainted-sneak-attack` doesn't;
  - 313: the rogue talent Weapon Focus (PrerequisiteConditionForWeaponCategory);
  - 266: Holy Water Jet (it names the six mythic classes, which no single entry maps);
  - 267: the Razmiri Infiltrator (deity features no entry maps) and the Living Deity (a hand `Atheist`; the game lists
    every deity's feature).
- **Allowlist** (`prerequisite-narrowing.txt`): 48 lines removed, all used by the run: the three mercies (6), Sudden
  Speed (2), combat style talent 2 (2), Mummification (2), Improved Uncanny Dodge (2), Cognatogen (2), Fast Bombs (3),
  Preserve Organs (2), Potent Magic (2), four magus arcana (8), Mystery Seeker (2), Survivalist (3), Blinding Strike (2)
  and the five ranger 10th-level lists (10). The last ten went with their entries left whole, so the writer now leaves
  those as a narrowing.
- **`known.txt`:** the 54 stale lines deleted (274, 275, 276, 279, 280, 282, 283, 287 and the five ranger lines under
  293); 24 lines re-homed to 307, one to 313, one to 266, two to 267. The five ranger 10th-level lines stay 293's, with a
  comment (below). Section comments name what 308e wrote.
- **`improved-uncanny-dodge.ron`:** the stale comment is rewritten to the game's rule and the user's 276 decision. Its
  summary and Uncanny Dodge's said "Eldritch Scoundrel only"; they now name the classes (mine).
- **Game check:** 3,482 differences in 188 groups, all known, no stale line in either file. Prerequisites: 1,447
  compared, 1,029 agree.

**The finding: the ranger's 10th-level style lists (for the main session; no gold change)**

- **The failure.** With the five written, the gold Enforcer was impossible at all three starts (CG, CN, NG). I put the
  written tree back into `ranger/combat-style-feats-10/menacing` alone and printed `engine.failures(&enforcer())`. The
  one failure is `Slot(OptionFeat { level: 12, index: 0 }) NotAllowed`: Dreadful Carnage in the feat slot under the
  slayer's 10th-level combat style talent. That slot nests `ranger/combat-style-feats-10/menacing`, whose written tree
  was `Any([SelectionOption("ranger/combat-style-feats-6/menacing"),
  Selection("ranger/combat-style-feat/menacing-feats-6")])`, so it offered no Menacing list.
- **The game allows the pick.** At slayer 6 the Enforcer takes RangerStyleSelection6 61f82ba786fe05643beb3cd3910233a8
  (combat style talent 6), whose `m_AllFeatures` holds RangerStyleMenacingSelection6
  a9998f05a0472bb4f8950304fe26ec0c (PrerequisiteFeature RangerStyleMenacingSelection2, held from level 2). Taking it
  makes that feature held. At slayer 12 it takes RangerStyleSelection10 78177315fc63b474ea3cbb8df38fafcd, which offers
  RangerStyleMenacingSelection10 7d5bd07830a5e2c478852bf8b860db2c, whose one prerequisite is PrerequisiteFeature
  RangerStyleMenacingSelection6 (`game-check show`). So the game allows Dreadful Carnage there.
- **The cause: the planner's model, not a carrier.** The planner nests each stage's lists in its stage selection and
  stores only the feat (304's note). Neither the option `ranger/combat-style-feats-6/menacing` nor the list
  `ranger/combat-style-feat/menacing-feats-6` is ever held, so `Has` of either never holds. 308g's allowlist reason
  ("classes/ranger.ron grants ranger/combat-style-feats-6") named the stage selection, but the written tree names its
  option and list. The same tree also emptied a ranger 10's style slot (`option_follow_ups::a_ranger_10s_style_slot_offers_the_10th_level_list`)
  and a slayer's 10th-level style feat (`follow_ups::a_slayer_takes_three_combat_style_feats_and_not_a_fourth`).
- **Owner.** 304's note lists "opens the 10th-level style lists with the 6th-level one" as not modelled, known under 293
  (its decision Q4). So the five lines stay `-> 293`, not 307. The writer prints `-> 308` for them, its default when its
  owner function names no ticket.
- The carrier guard couldn't see this: the fact isn't granted by a game carrier the planner lacks. The narrowing guard
  would have, but 308g's allowlist let it through.

**Decisions**

- **Clear Mind (mine).** `stalwart-defender/defensive-powers/clear-mind` shares ClearMindFeature
  4a236ae23d5a4f76b3616ddfa337e724 with the rage power. StalwartDefenderDefensivePowerSelection
  2cd91c501bda80b47ac2df0d51b02973 doesn't list it (`option lists / in planner only`, 287). The writer gave it the
  blueprint's barbarian, skald or shifter 8 or Primalist 8, which moves a planner-only option toward "not offered".
- **Summaries of the two Uncanny Dodges (mine):** they stated the old Eldritch-Scoundrel-only rule.
- **The census log** for `compare` is the run's log less the five ranger rows (55 rows).
- **Generators (mine): none changed, 2,156 Builds before and after, 1,990 open.** `targeted ranger10-slayer2` (ranger 10,
  then the slayer's 2nd-level combat style talent) no longer opens: the game refuses that talent to a holder of
  RangerStyleSelection2, which a ranger 2 holds (`option_follow_ups::a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style`).
  `t60 sanctified 8 combat style` (the same talent at Sanctified Slayer 8) now opens: the base's upper slayer level is
  gone. So `full targeted ranger10-slayer2` is lost and `full t60 sanctified 8 combat style` is new. Neither is a rename:
  `generate.rs` makes a `full` copy only of a Build that opens. Making the ranger Build open again would have added a
  copy (2,157), so it stays the game's refusal, as 306aa's Underground Chemist did.
- **Older links (main session's rule, as 306s and 308d):** `support::without_discoveries_the_game_refuses_since_308e`
  drops Nauseating Flesh only where its failure names nothing but level 12 of a class or the two archetypes.
  `first_ascension::every_older_link_…` uses it (one older link takes it at alchemist 10).
- **Tests re-pointed, not un-ignored:** Emboldening Strike, six advanced talents and the Archaeologist's Crippling Strike
  (`-> 307`, the carrier), Holy Water Jet (`-> 266`), the Razmiri Infiltrator (`-> 267`).

**Gold (to the user before the merge)**

- **The Enforcer's level 8:** before, `selection(8, 0)` (its slayer talent at 8) held no pick, and `KNOWN_GAPS` held
  173's "Uncanny Dodge" `WriteUpMistaken` entry. After, it picks `rogue/rogue-talents/uncanny-dodge`
  (SlayerTalentSelection6 43d1b15873e926848be2abf0ea3ad9a8 lists UncannyDodgeTalent ca5274d057152fa45b7527cad0927840,
  PrerequisiteClassLevel SlayerClass 4), and the entry is deleted. The now unused `Why::WriteUpMistaken` is removed.
  Every other pick is unchanged (2 combat style 2 Menacing, 6 combat style 6, 12 combat style 10, 14 Crippling Strike).
  The gold tests pass. No other gold change.

**Proof** (`compare` against the base's record with offers, `FORM_PROOF_CENSUS`, `FORM_PROOF_OFFER_BUILDS_OUT`)

- **Census:** 1,803 lines in 87 Builds (builds.tsv 85, numbers 25, reopened sheet 310, reopened slots 134, sheets 824,
  slots 425), 0 unattributed. Other differences: the 4 above (one lost and one new `full` copy, one Build closed, one
  opened).
- **Links:** 1,913 the same and 77 refused.
- **Offers:** 156,877 statuses changed in 559 groups and 388 Builds. 83 groups (3,675 statuses) name a written option and
  each moves toward the game:
  - Uncanny Dodge and Improved Uncanny Dodge `Unmet → ok` (153 and 123 Builds): bard, skald, slayer 4 or 12, Feyform 4.
  - Combat style talent 2 `Unmet → ok` (68): no slayer level and no upper level. Talents 6 and 10 `→ Unmet` (54, 45):
    the style before and the level.
  - Magus arcana `→ Unmet` (84): their magus level; Bane, Devoted, Enduring and Ghost Blade also for the Armored
    Battlemage.
  - Nauseating Flesh (75), Swift Consume (52: the Arcane Enforcer and other non-arcanists), Improved Evasion (60: no
    Evasion), the nocturnal abilities (7), Survivalist (20), Mystery Seeker, Cognatogen, Fast Bombs, Preserve Organs,
    Feral Mutagen, Potent Magic, Clear Mind and the fiendish companion (4): `→ Unmet`, each its blueprint's condition.
  - Extra Mercy `NoOptionLeft → ok` (1): the three mercies have no level now.
  - Mythic Skill Focus and Toughness `ok → Unmet` (34, 5) name no written entry. They're in the regenerated `full` Builds
    (inference, below).
- The other 476 groups (153,202 statuses) are `→ BreaksLater` across almost every feat: Weapon Focus 70 Builds, Improved
  Critical 65, and so on. **Inference:** they're in the `full`/`filled` Builds whose random fill drew differently once an
  earlier option's status changed (291 of the 388 Builds are `full`, `filled` or `empty`). The census attributes every
  line of theirs.
- **`306-refused.txt`:** 42 link lines in 6 new blocks:
  - 22: magus arcana below their level (bane-blade-level-15 at 3 to 9, devoted-blade-level-12 at 6, dimension-strike and
    ghost-blade-level-9 at 3 and 6);
  - 11: alchemists' Nauseating Flesh below 12, the Incense Synthesizer's Greater Mutagen (it has no mutagen), the
    Metamorph's Fast Bombs;
  - 5: Bloodseeker nocturnal abilities below their level;
  - 2: `targeted ranger10-slayer2` and its full copy;
  - 1: Survivalist for a Sylvan Trickster;
  - 1: the fiendish companion for Desna's (neutral good) Divine Hunter.

**Tests**

- Un-ignored (10):
  - `borrowed_selections::{the_vivisectionists_advanced_talents_meet_every_advanced_talent, a_rogue_10_feyform_shifter_is_offered_the_advanced_talents, a_sanctified_slayer_8_may_take_the_first_combat_style_feat, an_eldritch_scoundrel_2_isnt_offered_uncanny_dodge, an_eldritch_scoundrel_takes_improved_uncanny_dodge_from_rogue_12, the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents, a_scoundrel_2_with_bard_2_isnt_offered_uncanny_dodge}`;
  - `follow_ups::a_slayer_may_take_the_first_combat_style_feat_at_6`;
  - `negated_prerequisites::a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style`;
  - `option_follow_ups::a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style`.
- New, `rogue_slayer_alchemist_prerequisites.rs` (11, each citing its blueprints):
  - `the_bloodseekers_nocturnal_abilities_need_their_slayer_levels`;
  - `the_magus_arcana_need_their_magus_level_and_four_refuse_the_armored_battlemage`;
  - `the_options_whose_blueprints_refuse_an_archetype_refuse_it`;
  - `the_mutagen_discoveries_accept_the_mutation_warrior`;
  - `mummification_needs_an_underground_chemist_rogue_10`;
  - `the_three_mercies_and_sudden_speed_name_no_class_level`;
  - `the_stalwart_defenders_powers_take_the_games_levels`;
  - `the_otherworldly_companions_name_their_alignments`;
  - `blinding_strike_improved_evasion_and_swift_consume_take_the_games_conditions`;
  - `the_rangers_6th_level_styles_need_the_2nd_level_style`;
  - `the_slayers_later_combat_style_talents_need_the_style_before_and_the_level`.
- Updated:
  - `mercies_and_exploits::mercies_are_a_selection_of_15_with_their_level_and_prerequisite` and
    `ki_powers::ki_powers_are_a_selection_of_14_with_their_level` (no level for the three mercies and Sudden Speed);
  - `prerequisites_as_components` (657 → 684);
  - `repeats_from_the_header::the_selections_with_their_own_no_feature_are_taken_once` (the talents' shared NoFeature);
  - `negated_prerequisites::every_negated_prerequisite_…`: an option's own parenthesised name is set aside before the
    "no parenthesis" check;
  - `gold`, `first_ascension`.

**Checks** (each once after the last source edit)

- `cargo test -p seeds -p corpus`: game_data 632 passed, 94 ignored; files 34; links 6; form_proof 22; self_checks 1.
- `cargo test -p game-check`: 165 passed.
- `check-data`: checked, prerequisites 1,447 compared, 1,029 agree.
- `game-check`: exit 0, 0 not known, no stale line.
- The second dry run of the same `--only`: "nothing to write".
- `cargo fmt --all --check` clean; `scripts/test-modules.sh`: test-modules OK.
- No `libs/rules` change.
