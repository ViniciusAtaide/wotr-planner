# Rogue and slayer talents and rows

Type: build (data), with one item to the user
Status: superseded by 293: the user, 2026-11-15, took the game's rule for Improved Uncanny Dodge (class level 12), reversing ticket 190's choice; the importer writes it
Blocked by: [247](247-a-selection-counts-from-the-level-that-grants-it.md), [249](249-selections-and-parametrised-feats-are-taken-again.md), [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: lists, prerequisites and rows read from the selections and progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `borrowed_selections.rs`, `data.rs`, `gold.rs`, `archetype_losses.ron`; half B `negated_prerequisites.rs`, `option_follow_ups.rs`, `follow_ups.rs`, `scaling_features.rs`, `report_gaps.rs`) differ on `rogue.ron` and `slayer.ron`:
- **Uncanny Dodge in the slayer's lists.** The slayer talent selections at 2, 6 and 10 (`SlayerTalentSelection2` 04430ad24988baa4daa0bcd4f1c7d118, `…6` 43d1b15873e926848be2abf0ea3ad9a8, `…10` 913b9cf25c9536949b43a2651b7ffb66) list `UncannyDodgeTalent` (ca5274d057152fa45b7527cad0927840), whose `Any` prerequisites include slayer 4. **The user's decision (2026-11-15):** the game wins over ticket 173's: Uncanny Dodge is a slayer talent from slayer 4, applied without asking again; the gold builds' known gap (173, "write-up mistaken") is deleted, since the Enforcer's write-up is right. The talents' prerequisites become the game's `Any` (Eldritch Scoundrel, slayer, bard or skald 4, the Feyform 4; Improved at 12, no Feyform); the bard's list holds both, the Feyform's (the rogue's) Uncanny Dodge. The Eldritch Scoundrel (`EldritchScoundrelArchetype` 57f93dd8423c97c49989501281296c4a) is offered Uncanny Dodge from 4, not 2.
- **The slayer's lists** are the game's three (39 at 10: three ranger styles, Slayer Camouflage and Trapfinding, 32 of the rogue's talents and two archetype talents, `ExecutionerPainfulStrike` and `VanguardTacticianExtraUseTalent`; not Intimidating Prowess, Weapon Snatcher, Skill Focus, Weapon Focus, Emboldening Strike, Petrifying Strike).
- **The combat-style chain:** `RangerStyleSelection2` (c6d0da9124735a44f93ac31df803b9a9) needs only that it isn't held, with no upper level; 6 needs 2, 10 needs 6. A ranger 10 / slayer 2 is refused the slayer's level-2 style talent.
- **Advanced talents:** 17 need `AdvanceTalents` (a33b99f95322d6741af83e9381b2391c), Emboldening Strike (ba6bea8c93c64764ab7e5bbdb88fb9a6) among them; Crippling Strike (b696bd7cb38da194fa3404032483d1db), Confounding Blades, Dispelling Attack, Wearying Strike, Emboldening and Petrifying Strike also need Sneak Attack; one `AdvanceTalents` feature serves the rogue, the slayer, the Archaeologist and the Vivisectionist (an Archaeologist, who never gains Sneak Attack, can't take Crippling Strike).
- **Borrowed lists:** the bard's (`BardTalentSelection` 94e2cd84bf3a8e04f8609fe502892f4f, 24) and the skald's (`SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57, 21) and the Inciter's (`ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4, 24) are their own; the Feyform (`FeyformShifterShifterArchetype` 1cdfc7d306d1430eac19427539b62091) takes the rogue's own selection at 8 and 14 (all 38, the advanced ones opened by `AdvanceTalents`). The separate-selection shape of the bard's and skald's rows is [244](244-the-game-data-laid-out-by-blueprint.md)'s; their lists' contents are this ticket's.
- **Fey Tricks** (`SylvanTricksterTalentSelection` 290bbcc3c3bb92144b853fd8fb8ff452) offers 64: 37 rogue talents (not Weapon Snatcher), 26 witch hexes and the Hex Channeler's channel.
- **The Underground Chemist** (`UndergroundChemistArchetype` b4f5741d42c4cb04abf5ab3fe33f6fc5) loses the talent at 4 and 10 only; at 10 its own selection (`UndergroundChemistSelection` c3d08fdb481aa2e4a9f71c4349141826: 6 discoveries and 23 rogue talents); 12 to 20 keep the rogue's talents. **The Dark Lurker** (`DarkLurkerArchetype` 33eb002078e5437b8c25db4b80b4a4fe) loses the talents at 2, 8, 14 and gains one at 20.
- **Rows:** the Imitator holds its own Master Slayer (`ImitatorMasterSlayerFeature` 97c7054442504513b0324c00db0290fc) at 20; the Eldritch Scoundrel's sneak attack is the rogue's own `SneakAttack`; Slayer's Advance's extra uses, the Spawn Slayer's size bonuses, the Stygian Slayer's invisibility, the Executioner's assassinate are the game's ([254](254-a-features-steps-follow-the-class-level.md)); the Deliverer's deity row goes; the Master of All's Skill Focus list ignores no prerequisites.
- The rogue talent Weapon Focus repeats per weapon: [249](249-selections-and-parametrised-feats-are-taken-again.md).

## To the user

- **Improved Uncanny Dodge** (`ImprovedUncannyDodgeTalent` e821c61b2711cea4cb993725b910e7e8) needs Eldritch Scoundrel, bard, slayer or skald 12, not Uncanny Dodge held; and the Uncanny Dodge talent taken while holding Uncanny Dodge grants Improved instead. This contradicts the user's choice of the recommended option on 2026-11-13 (ticket 190: "Improved only after Uncanny Dodge"), which the 2026-11-15 decisions didn't cover. Apply the game's rule? Until answered, its finding (`prerequisites / not in game`, `selection option rogue/rogue-talents/improved-uncanny-dodge`, the planner's `UncannyDodgeChecker`) stays known under this ticket and `borrowed_selections.rs`' `an_eldritch_scoundrel_takes_improved_uncanny_dodge_only_after_uncanny_dodge` keeps its value.

## Findings (the game check, `known.txt` section 276)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner | 57 |
| option lists / in planner only | 24 |
| progression rows / in game only | 22 |
| progression rows / in planner only | 19 |
| option lists / in game only | 9 |
| progression rows / a further rank in planner only | 9 |
| prerequisites / not in game | 4 |
| option lists / a choice the game doesn't offer | 1 |
| option lists / ignores prerequisites or not | 1 |
| **all** | **146** |

The 48 entries: `archetype rogue/dark-lurker`, `archetype rogue/eldritch-scoundrel`, `archetype rogue/sylvan-trickster`, `archetype rogue/underground-chemist`, `archetype slayer/bloodseeker`, `archetype slayer/deliverer`, `archetype slayer/executioner`, `archetype slayer/imitator`, `archetype slayer/spawn-slayer`, `archetype slayer/stygian-slayer`, `archetype slayer/vanguard`, `class slayer`, `feat list rogue/master-of-all/skill-focus`, `selection option rogue/rogue-talents/blinding-strike`, `selection option rogue/rogue-talents/confounding-blades`, `selection option rogue/rogue-talents/crippling-strike`, `selection option rogue/rogue-talents/dispelling-attack`, `selection option rogue/rogue-talents/emboldening-strike`, `selection option rogue/rogue-talents/focusing-attack-confused`, `selection option rogue/rogue-talents/focusing-attack-shaken`, `selection option rogue/rogue-talents/focusing-attack-sickened`, `selection option rogue/rogue-talents/improved-evasion`, `selection option rogue/rogue-talents/improved-uncanny-dodge`, `selection option rogue/rogue-talents/mystery-seeker`, `selection option rogue/rogue-talents/petrifying-strike`, `selection option rogue/rogue-talents/slow-reactions`, `selection option rogue/rogue-talents/survivalist`, `selection option rogue/rogue-talents/uncanny-dodge`, `selection option rogue/rogue-talents/weakening-wound`, `selection option rogue/rogue-talents/weapon-focus`, `selection option rogue/rogue-talents/wearying-strike`, `selection option slayer/bloodseeker/nocturnal-abilities/blasphemy`, `selection option slayer/bloodseeker/nocturnal-abilities/enervation`, `selection option slayer/bloodseeker/nocturnal-abilities/haste`, `selection option slayer/bloodseeker/nocturnal-abilities/predatory-might`, `selection option slayer/bloodseeker/nocturnal-abilities/predatory-prowess`, `selection option slayer/bloodseeker/nocturnal-abilities/slow`, `selection option slayer/bloodseeker/nocturnal-abilities/summon-bloody-bones-beast`, `selection option slayer/bloodseeker/nocturnal-abilities/vampiric-shadow-shield`, `selection option slayer/bloodseeker/nocturnal-abilities/vampiric-touch`, `selection option slayer/bloodseeker/nocturnal-abilities/wracking-ray`, `selection option slayer/slayer-talents/combat-style-feat-10`, `selection option slayer/slayer-talents/combat-style-feat-2`, `selection option slayer/slayer-talents/combat-style-feat-6`, `selection rogue/sylvan-trickster/fey-tricks`, `selection rogue/underground-chemist/discovery`, `selection slayer/imitator/imitation`, `selection slayer/slayer-talents`.

The report lists each finding under its group, marked `[known: 276]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The lists, prerequisites and rows become the game's; the gold builds' known gap 173 goes; `archetype_losses.ron`'s dark-lurker and underground-chemist entries follow; a data note lists every change, and the user's answer on Improved Uncanny Dodge.

## Tests 242 marks as waiting on this ticket

- `borrowed_selections.rs`: `the_borrowed_selections_offer_the_guides_lists`, `a_bard_takes_rogue_talents_and_an_archaeologist_advanced_ones_from_10` (24 and 21; Opportunist, not Crippling Strike, at the Archaeologist's 10), `a_feyform_shifter_picks_a_rogue_talent_at_8_and_14_but_no_advanced_one` (38; the rogue 10's advanced talents open), `an_eldritch_scoundrel_is_offered_uncanny_dodge_at_each_talent_level` (from 4), `no_other_class_or_archetype_is_offered_the_scoundrels_talents` and `a_scoundrel_with_bard_levels_may_take_them_through_the_bards_slots` (the game's `Any`), the `ADVANCED_TALENTS` constant (17).
- `data.rs`: `a_slayer_takes_rogue_talents_and_advanced_ones_from_slayer_10` (39), `a_sylvan_trickster_picks_a_rogue_talent_or_a_witch_hex` (64), `an_underground_chemist_may_take_discoveries_in_place_of_rogue_talents_from_level_10` (29 at 10; rogue talents at 12).
- `negated_prerequisites.rs`: `every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out` (the style talent's window); `option_follow_ups.rs`: `a_ranger_10_slayer_2_takes_the_slayers_2_or_4_feat_from_the_first_stage` (refused); `follow_ups.rs`: the slayer style window rows.
- `gold.rs`: `gold_builds_gain_what_the_write_ups_list` (the 173 known gap goes).
- `scaling_features.rs`: the Imitator's Master Slayer row; `report_gaps.rs`: `an_eldritch_scoundrels_sneak_attack_dice_come_at_1_3_7_11_15_and_19` (the rogue's own feature).
- `archetype_losses.rs` (dark-lurker, underground-chemist).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 276 (exit 0, run by the main session), the Improved Uncanny Dodge findings aside until the user answers.
- The tests above hold the game's values, cited; the scoped checks pass.
