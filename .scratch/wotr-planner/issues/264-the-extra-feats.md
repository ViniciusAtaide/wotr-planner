# The Extra feats' prerequisites and picks

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [247](247-a-selection-counts-from-the-level-that-grants-it.md), [250](250-the-extra-feats-caps.md)
Model: Opus to build: two feats split, lists change and each prerequisite is the game's.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `extra_picks.rs`, `borrowed_selections.rs`; half B `skald_rage_powers.rs`, `mercies_and_exploits.rs`) differ on the Extra feats:
- **Their prerequisites name selections** (the game: holding the class's selection, [247](247-a-selection-counts-from-the-level-that-grants-it.md)'s form); the planner lists every option the selection offers, or a class level. Extra Discovery (`ExtraDiscoverySelection` 537965879fc24ad3948aaffa7a1a3a66) needs `DiscoverySelection`; Extra Arcana (`ExtraArcanaSelection` 00727883edf145e2a6bce9ad176ecfd8) needs `MagusArcanaSelection`, `HexcrafterMagusHexArcanaSelection` or the Eldritch Scion's `EldritchMagusArcanaSelection` (d4b54d9db4932454ab2899f931c2042c); Extra Rage Power needs the barbarian's, the Instinctual Warrior's or the skald's rage-power selection; Extra Slayer Talent (`ExtraSlayerTalentSelection` 53f7237f5b1447bb851ba68045a00e41) `SlayerTalentSelection2` or the Sanctified Slayer's feature; Extra Wild Talent (`ExtraWildTalentSelection` 0f73730cf1b44ee882671e55d5f6e471) `WildTalentSelection`; Extra Rogue Talent the Dark Lurker's selection too; Extra Arcanist Exploit the Exploiter's; Extra Bombs `ArcaneBombSelection`; Extra Ki `ScaledFistPowerFeature`; Extra Lay on Hands `HealersWayFeature`; Extra Performance and Lingering Performance the performance resource.
- **Extra Hex is two feats:** Extra Hex (Witch) (`ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35: `WitchHexSelection` or the Winter Witch's, 27 hexes) and Extra Hex (Shaman) (`ExtraShamanHexSelection` d0b4c8245d504b8c9c6d3fccc1f8c5b6: `ShamanHexSelection` or the Winter Witch's, 34). A witch/shaman takes both. The Hexcrafter's hex (`HexcrafterMagusHexMagusSelection`) meets neither.
- **Extra Discovery is two feats:** the alchemist's (28 discoveries) and the Vivisectionist's (`ExtraVivsectionistDiscoverySelection` 10287e7b8cee479e82ea88bd6d2d4dae, 25). The Underground Chemist's selection meets neither.
- **Extra Arcana's pick** is its own 15 magus arcana for every magus, the Hexcrafter included; no hexes.
- **Extra Rage Power's pick** is its own list of 38; the Battle Scion (whose `BattleProwessSelection` meets nothing) can't take it; `barbarian/rage-power` lists the 5 Instinctual Warrior powers the barbarian's own slot (33) doesn't.
- **Extra Rogue Talent's list** is 39 (the rogue's 38 and Evasion); **Extra Wild Talent's** holds Air's Reach (`AirsReachFeature` 9a358a71928952547b885389f5ba2a82), which `WildTalentSelection` lacks.
- **Extra Channel** names the paladin's, the Empyreal's and the Hospitaler's channels; its variants (`ExtraChannelEmpyrealSorcerer`, …) are [244](244-the-game-data-laid-out-by-blueprint.md)'s pool entries.

## Findings (the game check, `known.txt` section 264)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / not in game | 106 |
| prerequisites / missing in planner | 9 |
| prerequisites / missing in planner, a blueprint it doesn't map | 9 |
| **all** | **124** |

The 13 entries: `feat extra-arcana`, `feat extra-arcanist-exploit`, `feat extra-bombs`, `feat extra-channel`, `feat extra-discovery`, `feat extra-hex`, `feat extra-ki`, `feat extra-lay-on-hands`, `feat extra-performance`, `feat extra-rage-power`, `feat extra-rogue-talent`, `feat extra-slayer-talent`, `feat extra-wild-talent`.

The report lists each finding under its group, marked `[known: 264]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Each Extra feat's prerequisite is the game's, with [247](247-a-selection-counts-from-the-level-that-grants-it.md)'s form; Extra Hex and Extra Discovery split into the game's feats (links keep the old key as an alias); each pick's list is the feat's own.

## Tests 242 marks as waiting on this ticket

- `extra_picks.rs`: `the_extra_feats_that_give_a_pick_name_their_selections`, `a_barbarian_with_extra_rage_power_at_3_gets_a_rage_power_slot_there` (its "same as the slot at 2" half), `each_extra_feat_gives_a_slot_of_its_classs_selection` (the Underground Chemist, the witch 1 / shaman 2, the bard and skald, the Hexcrafter cases), `a_hexcrafters_extra_arcana_gives_a_hex_arcana_slot` (15 arcana), `extra_discovery_and_extra_arcana_reach_a_vivisectionist_and_a_hexcrafter_whatever_they_picked` (the Hexcrafter 5 meets Extra Arcana).
- `skald_rage_powers.rs`: `extra_rage_power_reaches_a_skald`; `a_battle_scion_takes_a_teamwork_feat_in_place_of_a_rage_power` (its Extra Rage Power half).
- `mercies_and_exploits.rs`: `extra_arcanist_exploit_gives_an_exploiter_wizard_one_more_exploit_slot`.
- `borrowed_selections.rs`: `a_scoundrel_with_only_uncanny_dodge_may_take_extra_rogue_talent` (the list adds Evasion).
- `wild_talents.rs`: `extra_wild_talent_needs_kineticist_level_2` (the selection, not a level).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 264 (exit 0, run by the main session).
- The tests above hold the game's answers, cited; a data note lists the changes; the scoped checks pass.
