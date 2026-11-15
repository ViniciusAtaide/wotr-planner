# Items

Type: build (data, and a glasses slot)
Status: open for its model part only, after 293 (the user, 2026-11-15): its mechanical data is 293's, the importer's
Blocked by: [252](252-what-a-prerequisite-reads.md) (which bonuses count for prerequisites)
Model: Sonnet to build: each value is read from the item's blueprint and enchantments.
Map: [WotR Build Planner](../map.md)

## Problem

The game check's items and 242's research (half A `gear.rs`) differ on `gear.ron`:
- **Slots:** the Broken Trickster's glasses (`DLC3_BrokenTricksterGlassesArtifactItem` 692b59ae0433471aa675b5b1b80db863) and Razmir's Mask (`RazmirMask` 9338510a8d5c4b43b1284470b706128d) are `BlueprintItemEquipmentGlasses`, a slot of their own, not headwear; Trusted Friend (`TokenOfHonor` 905ec00744cd4627b0e00f187e215d4b) is a usable item (a quick-slot toggle, Knight Commander only), not a necklace, whose Charisma +2 and Persuasion +5 circumstance come from its toggle's buff (`TokenOfHonorBuff` ab092ad237384b46b2fb192e6305f673), not always on.
- **Bonus types:** the Dirge for Sarkoris (`SarkorisDirgeItem` cffc4037c224475ca2a0c0b6bfda8688) gives Strength +2 Enhancement, and the Half-Plate of Vigor (`HalfplateOfVigorItem` 053873b9eca18854696dfc0e49d260fe) Constitution +2 Enhancement, not untyped; the half-plate is +2, not +1.
- **Always-on bonuses:** the Belt of the Venerated Champion (`BeltOfVeneratedChampionItem` 87c5f3e1b5a01d0448d550b727f45cb4, `BeltOfVeneratedChampionFeature` 49fb12e9d6cac0041b6de3507049e7d0) gives an always-on untyped Fortitude +2 (and +1 insight AC against evil, a formation AC bonus); Mangling Frenzy's DR 3/- is in its rage-only buff (`ManglingFrenzyBuff` 1581c5ceea24418cadc9f26ce4d391a9), not always on; Draven's Hat (`DravensHatItem` 0160c8eabe704db41b6309af0d26a8e0) has no once-a-day quicken and gives Intelligence +2 profane; the Elixir of Inconceivable Transmutations (`ElixirMasterpieceFeature` e52dae8a5a68b5040a0c3c8761a412d7) is Alchemical, not Inherent; Ustalavic Pride (`FullplateOfUstalavianOfficerArmorItem` 59a33c8783ad4de8b547022ec274c930) is the game's spelling.
- **The Belt and Headband of Perfection +8** (`BeltOfPerfection8` dc2f2613634c2f947b306fa3379232c2, `HeadbandOfPerfection8` f94d69b19fe07ca4a8ade7993a1a7510) exist. **The user's decision (2026-11-15):** they are added to the user's 22 generic items, without asking again.

## Findings (the game check, `known.txt` section 288)

| The check's group (area / kind) | Findings |
| --- | --- |
| items / slot differs | 3 |
| items / bonus not in game | 2 |
| items / type differs | 2 |
| items / bonus missing in planner | 1 |
| **all** | **8** |

The 6 entries: `gear item belt-of-the-venerated-champion`, `gear item broken-trickster`, `gear item dirge-for-sarkoris`, `gear item half-plate-of-vigor`, `gear item razmirs-mask`, `gear item trusted-friend`.

The report lists each finding under its group, marked `[known: 288]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- A glasses slot (`gear.glasses`, the MCP's and the web app's places with it); each item's slot, bonuses and summary become the game's; the two +8 items join; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `gear.rs`: `exactly_the_users_twenty_two_generic_items_have_no_guide_reference` (24 with the +8s), `the_manuals_tomes_and_all_score_consumables_are_usable_inherent_bonuses` (the Elixir Alchemical), `trusted_friend_is_a_necklace` (usable), `an_armour_and_a_weapon_name_their_own_enhancement_and_base_type` (Half-Plate +2), `the_venerated_champions_fortitude_is_an_effect_not_a_bonus` (an always-on bonus), `an_item_with_other_effects_matches_its_page_line` (the half-plate's Constitution Enhancement), `mangling_frenzys_damage_reduction_is_in_its_summary_and_its_crit_line_stays_in_the_detail` (while raging), `an_item_keeps_its_conditional_lines_and_shows_its_always_on_ones` (Ustalavic Pride's spelling, Draven's Hat).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 288 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
