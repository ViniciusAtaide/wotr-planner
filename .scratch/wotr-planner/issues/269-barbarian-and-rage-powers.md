# Barbarian rows and rage powers

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: each rage power's gates are read from its blueprint.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B `instinctual_rage_powers.rs`, `skald_rage_powers.rs`, `scaling_features.rs`; half A `archetype_losses.ron`) differ on `barbarian.ron`:
- **Level gates read four classes:** each level-gated rage power is open at barbarian, skald, the Primalist's bloodrager level (`PrimalistArchetype`) and, all but Protect Vitals, shifter level (`ClearMindFeature` 4a236ae23d5a4f76b3616ddfa337e724, …). The planner reads barbarian or skald. A Primalist 8 then opens Clear Mind.
- **Totems:** 21 powers carry a gate, not 14: the Celestial, Daemon and Fiend Totems (6) and their Greater (10) (`CelestialTotemFeature` 5156331dc888e9347ae6fc81ad3f3cec, …) and Come and Get Me (12; `ComeAndGetMeFeature` 032402bde3fb47e39e6ebb6724ae9c22); each totem needs its lesser one; the totems' mutual exclusions are the game's (the planner's `not <other totem>` leaves are not the game's form).
- **The Instinctual Warrior** (`InstinctualWarriorArchetype` adffdd8a99094a89823a79292a503ee9): Purity of Body (`RagePurityOfBodyFeature` a4418aa778cb4ebd965d78af19becced) at archetype level 5, not 8; Diamond Soul (`RageDiamondSoulFeature` d4c929bffb80464bbe6daf1471088bf8) at 14, not 16; Deflect Arrows (`RageDeflectArrowsFeature` d971bbb15f354621ac37fd51d0ab9044) also Dexterity 13 and Intelligence 3. The game's names of these powers are an open question (242, item 9). Its rows take its own selection; `barbarian/rage-power` (`RagePowerSelection` 28710502f46848d48b3f0d6132817c4e, 33) doesn't list the five monk powers.
- **Damage reduction** (`DamageReduction` cffb5cddefab30140ac133699d52a8f8): the Pack Rager (`PackRagerArchetype` 3f0a548044dbad04b82535237584772b) removes it at 7, 13, 16, 19 and keeps 10; the Mad Dog (`MadDogArchetype` 063da4c77fa59454a9828c9ece407ea1) removes it at 7 to 19 and gains its own (`MadDogDamageReductionMaster` a0d4a3295224b8f4387464a4447c31d5) at 10, 13, 16, 19.
- **Steps:** Devour Essence (`DevourEssenceFeature` 492abe84bb2c4e4ebf5fbc83f054ef78) at 2, 7, 13, 19; the Instinctual Warrior's, the Flesheater's and the Pack Rager's other repeated grants follow [254](254-a-features-steps-follow-the-class-level.md). The Pack Rager's teamwork list holds Shake It Off.

## Findings (the game check, `known.txt` section 269)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / not in game | 96 |
| prerequisites / missing in planner | 84 |
| progression rows / a further rank in planner only | 15 |
| option lists / in planner only | 5 |
| progression rows / in game only | 5 |
| prerequisites / missing in planner, a blueprint it doesn't map | 4 |
| progression rows / in planner only | 4 |
| prerequisites / value differs | 2 |
| option lists / in game only | 1 |
| **all** | **216** |

The 34 entries: `archetype barbarian/flesheater`, `archetype barbarian/instinctual-warrior`, `archetype barbarian/mad-dog`, `archetype barbarian/pack-rager`, `feat list barbarian/pack-rager/bonus-feats`, `selection barbarian/rage-power`, `selection option barbarian/rage-power/beast-totem`, `selection option barbarian/rage-power/beast-totem-greater`, `selection option barbarian/rage-power/beast-totem-lesser`, `selection option barbarian/rage-power/celestial-totem`, `selection option barbarian/rage-power/celestial-totem-greater`, `selection option barbarian/rage-power/celestial-totem-lesser`, `selection option barbarian/rage-power/clear-mind`, `selection option barbarian/rage-power/come-and-get-me`, `selection option barbarian/rage-power/crippling-blows`, `selection option barbarian/rage-power/daemon-totem`, `selection option barbarian/rage-power/daemon-totem-greater`, `selection option barbarian/rage-power/daemon-totem-lesser`, `selection option barbarian/rage-power/deadly-accuracy`, `selection option barbarian/rage-power/deflect-arrows-instinctual-warrior`, `selection option barbarian/rage-power/diamond-soul-instinctual-warrior`, `selection option barbarian/rage-power/fearless-rage`, `selection option barbarian/rage-power/fiend-totem`, `selection option barbarian/rage-power/fiend-totem-greater`, `selection option barbarian/rage-power/fiend-totem-lesser`, `selection option barbarian/rage-power/increased-damage-reduction`, `selection option barbarian/rage-power/internal-fortitude`, `selection option barbarian/rage-power/lethal-accuracy`, `selection option barbarian/rage-power/protect-vitals`, `selection option barbarian/rage-power/purity-of-body-instinctual-warrior`, `selection option barbarian/rage-power/reflexive-dodge`, `selection option barbarian/rage-power/renewed-vigor`, `selection option barbarian/rage-power/rolling-dodge`, `selection option barbarian/rage-power/wrecking-blows`.

The report lists each finding under its group, marked `[known: 269]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Each rage power's gate and the rows become the game's; `archetype_losses.ron`'s barbarian entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `skald_rage_powers.rs`: `a_rage_powers_level_prerequisite_reads_skald_levels_too` (four classes; 21 gates), `a_locked_level_gated_powers_texts_name_the_classes_that_can_open_it` (the texts; a Primalist's own level counts).
- `instinctual_rage_powers.rs`: `the_instinctual_warriors_monk_powers_are_rage_power_options_for_it_alone` (gates 5 and 14, Deflect Arrows' Dexterity), `an_instinctual_warrior_is_offered_each_monk_power_from_its_level_and_not_before` (Purity of Body at 5).
- `scaling_features.rs`: `the_barbarians_damage_reduction_is_not_the_invulnerable_ragers` (the Pack Rager keeps 10), `the_flesheaters_devour_essence_and_power_from_flesh_step`, `the_instinctual_warriors_cunning_elusion_steps_every_four_levels`.
- `archetype_losses.rs` (`archetype_losses.ron`'s instinctual-warrior, mad-dog, pack-rager entries).
- `report_gaps.rs`: `the_totem_rage_powers_are_exclusive_one_line_per_character` (re-read with the game's exclusions).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 269 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
