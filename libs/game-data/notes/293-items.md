# 293k: the items run and the +8 headband

**What the data says now.** `game-check --write items` (ticket 293, stage S1) wrote the bonuses of four gear items as the game has them; the Headband +8 was added by hand.

- `headband-of-mental-perfection-8` (new, no `guide`): `HeadbandOfPerfection8` f94d69b19fe07ca4a8ade7993a1a7510, `slot: Headwear`, Intelligence, Wisdom and Charisma +8 Enhancement (its enchantments `Intelligence8` bc66f5712a147874aaac6b397b83f874, `Wisdom8` 02a37956195faf645bb956f2e6c1ef57, `Charisma8` 7669e7a9af237314dbb2a22e78e1a0b4). The user's, applied under ticket 288 line 15 (the user's 22 generic items plus the +8s, "without asking again"); `tools/game-check/user-facts.txt` carries `applied 288`. The writer's run finds it already agrees.
- `dirge-for-sarkoris`: Strength +2 is Enhancement (`SarkorisDirgeEnchantment` e02dedda1d5248b3aa9016f918722c2e), it was untyped.
- `half-plate-of-vigor`: Constitution +2 is Enhancement (`HalfplateOfVigorEnchantment` 2b5234450dfbd7e4a844820674a20f39). Its "A +2 half-plate" line (the data says +1) still waits on 288.
- `belt-of-the-venerated-champion`: Fortitude +2 untyped is a `GearStat` (`BeltOfVeneratedChampionFeature` 49fb12e9d6cac0041b6de3507049e7d0); `other_effects` keeps only the conditional +1 insight bonus to AC against evil.
- `trusted-friend` (decision 2 of research/293/decisions.md, main session): the game's `TokenOfHonor` has no always-on bonus, its Charisma +2 and Persuasion +5 circumstance bonuses come from a toggle (`TokenOfHonorBuff`). `components` is empty and an `other_effects` line says so. Its slot stays `Necklace` (the guide's), a known difference (288).

**Inferences and decisions.** The data check (`DataIssue::GearWithoutBonus`) accepted no item without a bonus; it now accepts one whose `other_effects` is not empty, so Trusted Friend can have the game's form (my decision, forced by decision 2). A gear item with no bonus has an empty summary, so its description is only its `other_effects`.

**Not modelled.** The toggle's bonuses count nowhere; the Venerated Champion's formation AC +1.

**Tests.** `libs/seeds/tests/game_data/gear.rs`: `the_game_has_a_headband_of_mental_perfection_8`, `exactly_the_users_twenty_three_generic_items_have_no_guide_reference`, `the_venerated_champions_fortitude_bonus_is_always_on`, `the_half_plate_of_vigor_gives_con_2_enhancement`, `sarkoris_dirge_gives_str_2_enhancement`, `trusted_friends_bonuses_are_its_toggles_not_always_on`.
