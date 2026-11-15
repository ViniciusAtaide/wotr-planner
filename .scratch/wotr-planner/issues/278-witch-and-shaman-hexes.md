# Witch and shaman hexes and rows

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: hex prerequisites and rows read from the blueprints.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `borrowed_selections.rs`, `follow_ups.rs`, `archetype_losses.ron`; half B `scaling_features.rs`) differ on `witch.ron` and `shaman.ron`:
- **The major and grand hex gates are features:** the hexes that need a level (Agony, Death Curse, …) need `WitchMajorHex` (8ac781b33e380c84aa578f1b006dd6c5) or `WitchGrandHex` (d24c2467804ce0e4497d9978bafec1f9), which the witch gets at 10 and 18 and the Hexcrafter at 12 and 18; the planner asks the Hexcrafter archetype and magus 12 or 18. A witch 10 / Hexcrafter 6 holds the major hex.
- **Shaman hexes** need the spirit's base or wandering feature (`Shaman<Spirit>SpiritBaseFeature`, `…WanderingFeature`), where the planner names the spirit progression or the Wildland Shaman; the Prophet of Pestilence's hexes need the archetype (`ProphetOfPestilenceArchetype` bcd758a75fb54651a7f668fe2661a307).
- **The Battle Master hex** (`ShamanHexBattleMasterProgression` 840651aac9b854a4980ec19e2fced74f) gives a Weapon Specialization pick at 8 and a Greater Weapon Focus pick at 16, each its own weapon, prerequisites ignored; the planner ties one weapon to the hex (242 inference: whether the values are limited by Weapon Focus under the ignore flag needs `BlueprintParametrizedFeature` read).
- **Rows:** the Wildland Shaman (`WildlandShamanArchetype` bb14b164c2ce4e2bb05434a3218ff73d) keeps the Wandering Hex (`ShamanWanderingHexFeature` 2674f98616188814fbba9f8fccf43564) and its spirit selection offers the game's spirits; the Witch Doctor (`WitchDoctorArchetype` 96c2b16450cc98548a0e82f743fbee80) also loses the hex at 12; the Elemental Witch (`ElementalWitchArchetype` d7c7cdbc760041fabaedc1623a53a431) also at 6; the Keen-Eyed Adventurer (`RayMasterWitchArchetype` d37f299738894b97bbc0808c1c44e6d2) keeps the cantrips (`WitchCantripsFeature` c213af60aba83ed4993948dce6b947b8); the Spirit Hunter's weapon enchantments, the Unsworn Shaman's minor spirit, the Shadow Shaman's shadows and stealth, the Battle Spirit's progression, the Possessed Shaman's shared skill and the Hag of Gyronna's Sunder Hope follow [254](254-a-features-steps-follow-the-class-level.md).
- The shaman's hex copies of the witch's (`ShamanHex<Name>Feature`) are [244](244-the-game-data-laid-out-by-blueprint.md)'s; Extra Hex's split is [264](264-the-extra-feats.md)'s.

## Findings (the game check, `known.txt` section 278)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner | 33 |
| prerequisites / not in game | 30 |
| prerequisites / missing in planner, a blueprint it doesn't map | 19 |
| progression rows / a further rank in planner only | 16 |
| progression rows / in game only | 10 |
| option lists / in game only | 9 |
| progression rows / in planner only | 9 |
| progression rows / a further rank in game only | 1 |
| **all** | **127** |

The 65 entries: `archetype shaman/possessed-shaman`, `archetype shaman/prophet-of-pestilence`, `archetype shaman/shadow-shaman`, `archetype shaman/spirit-hunter`, `archetype shaman/spirit-warden`, `archetype shaman/unsworn-shaman`, `archetype shaman/wildland-shaman`, `archetype shaman/witch-doctor`, `archetype witch/elemental-witch`, `archetype witch/hag-of-gyronna`, `archetype witch/keen-eyed-adventurer`, `archetype witch/ley-line-guardian`, `archetype witch/stigmatized-witch`, `class shaman`, `selection option shaman/hex/air-barrier`, `selection option shaman/hex/battle-master`, `selection option shaman/hex/battle-ward`, `selection option shaman/hex/beckoning-chill`, `selection option shaman/hex/bone-lock`, `selection option shaman/hex/bone-ward`, `selection option shaman/hex/entangling-curse`, `selection option shaman/hex/fearful-gaze`, `selection option shaman/hex/fire-nimbus`, `selection option shaman/hex/flame-curse`, `selection option shaman/hex/friend-to-animals`, `selection option shaman/hex/hampering-hex`, `selection option shaman/hex/hypothermia`, `selection option shaman/hex/intimidating-display`, `selection option shaman/hex/metal-curse`, `selection option shaman/hex/stone-stability`, `selection option shaman/hex/vortex-spells`, `selection option shaman/hex/ward-of-flame`, `selection option shaman/hex/wind-ward`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-addle-mind`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-contagiousness`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-defenseless-host`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-drain-body`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-drain-mind`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-dulled-reactions`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-erode-body`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-erode-mind`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-muscle-atrophy`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-muscle-feebleness`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-new-strain`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-staggered-body`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-stiffened-limbs`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-virulence`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-vulnerable-host`, `selection option shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-withering`, `selection option witch/hex/agony`, `selection option witch/hex/animal-servant`, `selection option witch/hex/animal-skin`, `selection option witch/hex/beasts-gift`, `selection option witch/hex/death-curse`, `selection option witch/hex/delicious-fright`, `selection option witch/hex/hoarfrost`, `selection option witch/hex/iceplant`, `selection option witch/hex/lay-to-rest`, `selection option witch/hex/life-giver`, `selection option witch/hex/major-ameliorating`, `selection option witch/hex/major-healing`, `selection option witch/hex/regenerative-sinew`, `selection option witch/hex/restless-slumber`, `selection shaman/wildland-shaman/spirit`, `selection witch/hex`.

The report lists each finding under its group, marked `[known: 278]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Hex prerequisites and the rows become the game's; `archetype_losses.ron`'s wildland-shaman, witch-doctor, elemental-witch and keen-eyed-adventurer entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `borrowed_selections.rs`: `a_hexcrafter_picks_a_witch_hex_at_4_and_an_arcana_or_hex_from_6` (Agony and Death Curse gated by the features).
- `follow_ups.rs`: the Battle Master shaman test (two weapon choices at 8 and 16).
- `scaling_features.rs`: `the_shaman_archetypes_step_the_features_the_guide_gives_steps_to`, `the_battle_spirit_steps_at_8_and_16_for_every_shaman_that_takes_it` ([254](254-a-features-steps-follow-the-class-level.md)'s pass).
- `report_gaps.rs`: `a_shamans_wandering_hex_is_a_daily_choice_from_6` (re-read for the Wildland Shaman).
- `archetype_losses.rs` (the four entries above).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 278 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
