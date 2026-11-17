# Importer S2, 306h: rage powers

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306g
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306h-rage-powers.md`.

**What was built**
- New selections `barbarian/skald-rage-power` (`SkaldRagePowerSelection` 2476514e…, 33 offers) and `barbarian/instinctual-warrior/rage-power` (`InstinctualWarriorRagePowerSelection` 609f0e53…, 38: its five monk powers are its own options, keys unchanged, the other 33 shared with the barbarian's). `barbarian/rage-power` keeps 33 (`RagePowerSelection` 28710502…). Rows: `classes/skald.ron` 3 to 18, demon-dancer 12, inciter 6, 12, 18 to the skald's list; `archetypes/barbarian/instinctual-warrior.ron` 4 to 20 to its own.
- **Decision (main session's rule, applied by the builder): the skald's key is `barbarian/skald-rage-power`, not the section's `skald/rage-power`.** A row's selections sort by key and slot ids follow; the skald's rows at 12 hold `rogue/skald-talents` too, and `skald/rage-power` sorts after it. The new key sorts before it, as `barbarian/rage-power` did. No other row holds two selections that include these (checked: skald 12, demon-dancer 12; the Instinctual Warrior's and the Inciter's rows hold one each).
- Extra Rage Power lists the game's 38 (`ExtraRagePowerSelection` 0c7f01fb…), is named "Rage Power" (its slot's label) and names the three lists and the Battle Scion's in `needs_slot_of`. Pack Rager's bonus feats gained Shake It Off (the writer). The Battle Scion's shared list is the game's 21 rage powers, written by hand: the writer can't express the list's twelve teamwork feats beside options.
- Writer: `--only barbarian/rage-power,barbarian/skald-rage-power,barbarian/instinctual-warrior/rage-power,barbarian/extra-rage-power,barbarian/pack-rager/bonus-feats`. It skips a selection with no list, so the two new selections and Extra Rage Power were seeded with one shared option first (Clear Mind, which two options hold: the writer needs one the selection already offers to choose).
- `known.txt`: removed 269's `barbarian/rage-power` and `barbarian/pack-rager/bonus-feats` lines. **Not removed: 286's `skald/battle-scion/rage-power-or-teamwork-feat` line:** the 17 extra rage powers are gone, but the game's twelve teamwork feats still show as "in game only" (the planner stands them as one option, `teamwork-feat`), so the line still matches. The game check exits 0.

**Tests:** `importer_lists::the_skald_and_the_instinctual_warrior_have_their_own_rage_powers` (33, 33, 38, 38 offers; the five monk powers only in the Instinctual Warrior's and Extra Rage Power's; Clear Mind and Come and Get Me in all four). Un-ignored: `extra_picks::the_barbarians_rage_power_slot_offers_33_and_extra_rage_power_38` (264), `skald_rage_powers::the_battle_scions_selection_offers_21_rage_powers` (286). Updated for the split: `skald_rage_powers`, `instinctual_rage_powers` (a barbarian is offered no monk power at all now, not a locked one), `extra_picks`, `held_selections`, `mythic`, `nested_selections`, `archetype_losses` (the skald archetypes lose `barbarian/skald-rage-power`; the Instinctual Warrior loses `barbarian/rage-power` at its table's levels), and `tools/corpus/tests/links.rs`'s Battle Scion test.

**Proof (offers, `FORM_PROOF_OFFERS=1`, base `93545ed1`, this lane):** links: 1,996 open, 1,989 the same, 0 through an heir, 7 refused, 0 changed. The section expected 0 refused; the 7 are Battle Scion Builds (two already listed by 306e for a rogue talent, five added), refused for the rage powers (Clear Mind, Come and Get Me, Daemon Totem, Celestial Totem Greater, Powerful Stance) the game's Battle Prowess list lacks, as the un-ignored 21-powers test requires; they are in `tools/corpus/tests/links/306-refused.txt` (appended). No Instinctual Warrior Build is refused. Offers: 18,033,279 recorded, 18,028,871 now; only recorded 4,423, only now 15; 17,485 statuses changed in 214 groups, all in the 6 Battle Scion Builds that hold a refused pick (`FORM_PROOF_OFFER_BUILDS_OUT`). No slot-order refusal. `compare` fails its form assertions as it must for a data change. No gold build changed.
