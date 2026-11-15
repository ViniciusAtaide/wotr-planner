# Every real-data test's expected value, from the game

Type: build
Status: resolved
Blocked by: 235, 236
Model: Opus to build: each value is read from the game and judged.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14). The user chose "b"; the recommendation was to update a test only when a ticket changes its fact.
- **The rule.** A test's expected value comes from the game, cited by blueprint GUID or by the decompiled code's file and method, never from the engine's earlier output. The user: "assume the game code works always".
- **The existing tests** were written from the guide: about 50 files in `libs/seeds/tests/game_data/`, besides the rules crate's own unit tests.

## Change

For each real-data test in `libs/seeds/tests/game_data/`:
- **Read the game** for each fact it asserts: the blueprint, or the code for a rule.
- **Where the game agrees,** add the citation beside the assertion: blueprint name and GUID, or file and method with the WrathDecompiled commit.
- **Where it differs,** change the test to the game's value. The data or engine then fails it, so fix that in the same change, or leave the test marked with the ticket that will fix it.
  - A difference with one of the user's facts (`research/skills-from-user.md`, `backgrounds-from-user.md`, the Aivu and Skeletal Champion tables) goes to the user first.
  - A difference already ticketed from [research/game-source-check-2026-11-14.md](../research/game-source-check-2026-11-14.md) is left to that ticket and named.

The gold builds (`gold.rs`) assert the user's write-ups, not the game. A gain there that the game contradicts goes to the user (234).

The rules crate's unit tests on hand-made data are not in scope: they test the engine's logic on fixtures, not game facts. A rule they encode that the game's code contradicts is listed for a ticket.

## Answer

**Built** (2026-11-15) in two halves by two builders on disjoint test files, merged one after the other. Every real-data assertion in `libs/seeds/tests/game_data/` either cites the game beside it, or moved into its own `#[ignore = "waits on ticket <n>: the game gives …"]` test holding the game's value, while the rest of the original test keeps running. 141 tests wait on tickets 247–291 (63 in half A, 78 in half B), and 45 `archetype_losses.ron` entries wait on their class tickets. No data, engine or `known.txt` changed here: every difference is a difference ticket's (user, 2026-11-15: "Cite, mark, ticket"). The seven facts of the user's the game contradicted are applied by those tickets (user, 2026-11-15).

**Found for other tickets** (not asserted by a test): `magical-tail` names `MagicalTail1` where the feat offered is `MagicalTail` (Ranks 8), for 264; five spells name another blueprint variant than the lists hold (poison ×3, harm, cure critical wounds, heal), for 290; Expanded Arsenal adds once per fact, so the Max DC run's two facts add +8, for 258.

## Half A

**Built** (2026-11-15, branch `ticket/242a-test-values-from-the-game`). Every real-data assertion of half A's 21 files (`libs/seeds/tests/game_data/`: alignment, archetype_losses .rs/.ron, archetype_saves, athletics_iii, backgrounds, bloodline_ascendance, bonus_feat_choices, borrowed_selections, build_sheets, caster_line, class_rules, class_skills, data, deities, extra_picks, finesse_training, first_ascension, follow_ups, gear, gear_engine, gold) either cites the game beside it (`// Game: <Blueprint> <guid>` or `// Game: <File>.<Method>, WrathDecompiled 6ce4a6b`) or, where the data differs, moved into its own `#[ignore = "waits on ticket <n>: the game gives …"]` test holding the game's value, with the rest of the original test still running. No data, engine or `known.txt` change. md`, `notes-A-borrowed.md`, `notes-A-picks.md`.

### Decisions and who made them

- **The form** (main session, the user's on 2026-11-15): citation beside the assertion; `(equivalent model)` where the planner's shape differs but the legal picks don't; a trailing comment per row of a const table; full GUIDs only.
- **Differences** (same): no data or engine fix here; the expected value becomes the game's; the differing assertion moves into its own ignored test naming the ticket (247–291, filed by the main session), the rest keeps running.
- **The user's facts** (the user, 2026-11-15): the seven approved items are treated as ordinary differences (Uncanny Dodge in the slayer talents → 276; Incredible Heft replaces Armor Training → the Titan's losses entry, 285; Belt/Headband of Perfection +8 → 288; the sorcerer's level-1 list, shield bashes, AT skill points, Weapon Focus per weapon: no half-A assertion).
- **`gold.rs`** (same): expectations unchanged; citations only.
- **Mine (the builder's):**
  - `archetype_losses`: an entry gains `waits_on: "ticket <n>"` and then holds the game's value; `count: 0` on a waiting entry means the game keeps the item. `check(…, view)` reads the data's view (waiting entries and their items left out) for the two running tests and the game's view for the ignored `each_archetype_row_loses_what_the_games_archetype_removes`.
  - `deities.rs`' `TABLE` holds the game's values; the four cells that differ are listed in `WAITING` and skipped by the running loop.
  - `extra_picks.rs`' `EXTRA_PICKS` holds the game's rows; the four that differ are in `WAITING`.
  - Running tests keep what agrees: set checks less the waiting keys (alignment), "every data save override is one of the game's four" (saves), "one power at least at Ascendance's rank" (bloodlines), exact DC breakdown less the Spell Focus parts (caster line), "the Seeker holds at least the sorcerer's and the bloodline's skills" (class skills).
  - Fixtures moved to game-legal picks where a running test used a pick the game refuses: the hunter's level-2 Precise Companion is Outflank; the Archaeologist 10 takes Opportunist; the Scoundrel's talents are at 8/12 with Extra Rogue Talent at 9/13; the plain rogue's Finesse Training picks short sword, dagger and sickle.
  - Expanded Arsenal (caster_line): read the code to settle 242's open inference. Each Expanded Arsenal fact's handler checks the shared part's school list, not its own parameter (`ExpandedArsenalMagicSchools.OnEventAboutToTrigger`), so the Max DC run's two facts each add Enchantment's doubled focus feats: +8. The game value is 8, not "4 at least". Ticket 258 should record it.

### Counts

| file | cited sites | of them equivalent model | moved (ignored tests) | not a game fact (left) |
| --- | --- | --- | --- | --- |
| alignment.rs | 12 (7 tests, 5 table rows) | 0 | 2 | the error text |
| archetype_losses.rs/.ron | 196 lists (one `// Game:` per archetype) | 117 (101 own-copy, 16 substitute selections) | 1 test, 45 waiting entries in 37 archetypes | the made-up-row tests (engine logic) |
| archetype_saves.rs | 4 | 0 | 1 | the QA link |
| athletics_iii.rs | 7 | 0 | 0 | slot order within a level |
| backgrounds.rs | 6 | 0 | 0 | — |
| bloodline_ascendance.rs | 11 | 4 (Empyreal Apotheosis = the Celestial's Ascension, one blueprint) | 3 | the Legend switch |
| bonus_feat_choices.rs | 10 | 0 | 3 | list names, guide anchors, labels |
| borrowed_selections.rs | 24 | 6 | 15 | 4 |
| build_sheets.rs | 3 | 0 | 0 | the sheet format, links, bench input, the Legend switch |
| caster_line.rs | 10 | 0 | 3 | — |
| class_rules.rs | 14 | 0 | 1 | the story-note policy, cleric.ron's note |
| class_skills.rs | 20 (incl. table rows) | 0 | 4 | names read apart, labels |
| data.rs | 12 | 3 (Adaptability, Sword Saint, the shaman's shared hex keys) | 3 | data check, link hash, links, list count, slot shape, copies |
| deities.rs | 20 (incl. 26 table rows) | 0 | 8 | issue texts, the no-faith state, allows/removable |
| extra_picks.rs | 17 | 3 | 5 | removability, labels, the empty-slot state |
| finesse_training.rs | 7 | 1 | 2 | links, withdrawals, labels |
| first_ascension.rs | 10 | 1 | 0 | older links |
| follow_ups.rs | 18 | 1 | 5 | the parameterised-feat count |
| gear.rs | 26 | 0 | 7 | scope, bonus types, guide pages, counts, text checks |
| gear_engine.rs | 3 | 0 | 0 | gear notes, withdrawal, link |
| gold.rs | 38 comment sites | 0 | 0 (expectations unchanged) | name tables, parsing, MR 10's level 19 |

**Moved: 63 ignored tests, all failing today** (`cargo test -p seeds --test game_data -- --ignored <half A's modules>`: 0 passed, 63 failed, each on its assertion).

### Moved tests by ticket

- **252** (1): `follow_ups::perfect_mind_raises_the_bab_itself_to_15_and_meets_no_skill_rank_prerequisite` (a real BAB of 15; no skill ranks).
- **257** (3): `bloodline_ascendance::a_second_bloodlines_ascendance_grants_one_power_of_the_two`, `an_eldritch_scions_ascendance_grants_one_power_of_the_two` (one of the two), `dragon_disciple_levels_dont_count_towards_an_empyreal_sorcerers_apotheosis` (none by level 20).
- **258** (3): `caster_line::mythic_spell_focus_doubles_the_max_dc_runs_spell_focus_feats` (focus feats +4; DC 26 + level, 35 heightened, 36 metamagic), `the_max_dc_runs_dcs_from_level_7_count_mythic_spell_focus_doubled` (25, 27, 32, 34, 35), `the_max_dc_runs_expanded_arsenal_gives_conjuration_enchantments_spell_focus_feats` (+8).
- **261** (4): `archetype_saves::four_archetypes_replace_a_save_progression`; `class_skills::the_animal_domains_class_skill_is_on_its_first_row`, `the_animal_domains_lore_nature_comes_with_the_domain_at_its_first_level` (Lore (Nature) at the domain's level 1, a Separatist's too), `a_seeker_sorcerer_holds_trickery_and_its_bloodlines_class_skill`.
- **263** (1): `class_skills::advanced_weapon_training_needs_the_fighters_weapon_training`.
- **264** (6): `borrowed_selections::extra_rogue_talent_offers_the_rogue_talents_and_evasion` (39); `extra_picks::the_extra_feats_that_give_a_pick_name_the_games_selections`, `the_barbarians_rage_power_slot_offers_33_and_extra_rage_power_38`, `the_extra_feats_the_game_refuses_to_an_underground_chemist_a_bard_a_skald_and_a_hexcrafter`, `a_hexcrafters_extra_arcana_gives_a_magus_arcana` (15 arcana), `a_hexcrafter_5_meets_extra_arcana`.
- **265** (1): `class_rules::domain_zealot_is_closed_to_a_druids_domain`.
- **267** (10): `alignment::the_archetypes_with_an_alignment_rule_are_the_games_eight`, `a_lawful_neutral_angelfire_apostle_of_torag_is_possible`; `deities::four_deities_have_the_games_weapons_and_domains`, `an_atheist_deliverer_is_possible`, `a_true_neutral_warpriest_of_calistria_is_offered_the_chaos_blessing`, `a_warpriest_of_torag_is_offered_five_blessings` (34 blessings), `a_deliverer_of_irori_gets_no_improved_unarmed_strike`, `a_demonbane_priest_of_irori_holds_improved_unarmed_strike`, `a_priest_of_balance_follows_one_of_seven_deities`, `a_true_neutral_champion_of_the_faith_chooses_any_alignment`.
- **275** (2): `bonus_feat_choices::the_stormwalkers_lists_are_the_rangers_archery_lists`, `a_hunters_teamwork_slot_at_3_offers_no_precise_shot`.
- **276** (16): `borrowed_selections::a_feyform_shifters_rogue_talent_is_the_rogues_selection` (38), `the_vivisectionists_advanced_talents_meet_every_advanced_talent`, `emboldening_strike_is_an_advanced_talent`, `six_advanced_talents_need_sneak_attack`, `a_bard_and_a_skald_pick_from_their_own_talent_lists` (24, 21, Inciter 24), `an_archaeologist_10_has_no_sneak_attack_for_crippling_strike`, `a_rogue_10_feyform_shifter_is_offered_the_advanced_talents`, `a_sanctified_slayer_8_may_take_the_first_combat_style_feat`, `an_eldritch_scoundrel_2_isnt_offered_uncanny_dodge`, `an_eldritch_scoundrel_takes_improved_uncanny_dodge_from_rogue_12` (the user's, see below), `the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents`, `a_scoundrel_2_with_bard_2_isnt_offered_uncanny_dodge`; `data::a_slayers_talent_list_at_10_is_the_games_39`, `a_sylvan_tricksters_fey_tricks_are_the_games_64`, `an_underground_chemist_takes_a_discovery_at_10_only` (29 at 10, none at 12); `follow_ups::a_slayer_may_take_the_first_combat_style_feat_at_6`.
- **278** (1): `follow_ups::a_battle_master_hex_names_no_weapon_and_8_and_16_each_choose_one`.
- **280** (1): `borrowed_selections::a_vivisectionist_keeps_one_alchemist_discovery_at_20`.
- **282** (1): `follow_ups::the_razmiri_infiltrator_is_any_alignment_but_good` (LN, TN, CN, LE, NE, CE).
- **283** (2): `borrowed_selections::a_hexcrafter_keeps_its_magus_arcana_at_3`; `follow_ups::a_sword_saint_chooses_a_melee_weapon`.
- **285** (1): `bonus_feat_choices::the_titans_massive_attacks_lists_hold_the_style_extras`.
- **288** (7): `gear::the_game_has_a_headband_of_mental_perfection_8`, `the_elixir_of_inconceivable_transmutations_gives_alchemical_bonuses`, `trusted_friend_is_a_usable_toggle`, `the_half_plate_of_vigor_is_a_2_half_plate_with_con_2_enhancement`, `the_venerated_champions_fortitude_bonus_is_always_on`, `mangling_frenzys_damage_reduction_is_while_raging`, `dravens_hat_has_no_once_a_day_quicken`.
- **289** (2): `finesse_training::finesse_training_offers_the_games_31_weapons`, `a_plain_rogue_trains_only_weapons_it_is_proficient_with`.
- **268, 269, 273–279, 281–286** (1 test): `archetype_losses::each_archetype_row_loses_what_the_games_archetype_removes`. Its 45 waiting entries in `archetype_losses.ron` each name the class's ticket: warpriest 268, barbarian 269, cavalier 273, paladin 274, ranger 275, rogue 276, druid 277, witch and shaman 278, monk 279, bloodrager 281, arcanist 282, magus 283, inquisitor 284, fighter 285 (Titan's Armor Mastery, the user's approved fact), bard and skald 286.

**Picked by area, not named in the ticket's waiting list:** `the_vivisectionists_advanced_talents_meet_every_advanced_talent`, `six_advanced_talents_need_sneak_attack` and `a_sanctified_slayer_8_may_take_the_first_combat_style_feat` (all rogue/slayer talents, 276).

### The user's items

- **To the user, ticket 276:** Improved Uncanny Dodge (`ImprovedUncannyDodgeTalent` e821c61b2711cea4cb993725b910e7e8) needs Eldritch Scoundrel/bard/slayer/skald 12, not Uncanny Dodge first; against the user's recommended-option decision of 2026-11-13 (ticket 190). The running test keeps what both agree on.
- **No other open user item in half A.** The game agrees with the user's facts asserted here: the 38 backgrounds in their groups, the Shackles Corsair's class skills, the Tortured Crusader's 3 points and 7 class skills, the Arcane Trickster's lack of UMD, the Dragon Disciple's Knowledge (World), the Mantis Zealot's LN/LE/NE, the 22 generic items. The Belt of Physical Perfection +8 is already in the data, so only the Headband +8 waits (288).
- **gold.rs and the known gaps:**
  - Gap 174 (Jack's Elk): the game agrees with the write-up (`SylvanSorcererArchetype` 711d5024ecc75f346b9cda609c3a1f83 gives `AnimalCompanionSelectionSylvanSorcerer`); the gap is the engine's.
  - Gap 173 (the Enforcer's Uncanny Dodge at 8): the game agrees with the write-up, against the entry's reason. `SlayerTalentSelection2`/`6`/`10` list `UncannyDodgeTalent` from slayer 4. The user approved the game's value; the entry stays until 276 deletes it.
  - The Enforcer's level 13: the game has one exploit pick (`ArcanistExploitSelection`, which lists the greater exploits), not two slots (260). Noted in a comment.
  - Every other write-up gain checked agrees.

### Rules the code contradicts (for their tickets)

- Mythic Spell Focus doubles each Spell Focus and Greater Spell Focus of its school (`SpellFocusParametrized.GetBonusDC`); Expanded Arsenal adds once per fact (258).
- Bloodline Ascendance is a selection of one power, not a grant of every bloodline's 20th row (257).
- Perfect Mind raises the BAB itself; a skill floor on totals meets no rank prerequisite (`PrerequisiteStatValue.GetStatValue` reads ranks) (252).
- A parametrized feature's weapon list follows its category filter: the Sword Saint's melee only (`BlueprintParametrizedFeature.ExtractItemsWeaponCategory`) (283).
- A progression's selection is a fact, whatever was picked (`LevelUpHelper.AddFeaturesFromProgression`): Domain Zealot and the Extra feats test the selection, not a held option (265, 264).
- `AddFeatureOnClassLevel` counts only its named classes' levels; an archetype of another class is ignored (Irori's Improved Unarmed Strike for a Deliverer) (267).
- An archetype prerequisite in the game reads that archetype's class levels (Eldritch Scoundrel 4/12); the engine's `Archetype` reads the whole Build (276).
- One shared Advanced Talents feature, not one per class or archetype (276).
- The Champion of the Faith's alignment choice has no deity check (267).

### Not settled

- **Shieldbearer, Mad Dog and Proclaimer** (losses):
  - which data enhancement the Shieldbearer's level-4 swap is;
  - what `MadDogDamageReductionMaster` grants, and to whom;
  - the Proclaimer's fervor selection, which has no data key.
- **The 16 substitute selections' option lists** (the Instinctual Warrior's rage powers and others) are the game check's.
- **Weapons:** the Tentacle and the Spiked Light Shield have no `weapons.ron` keys, so the 31-weapon test checks the count and 29 keys.
- **Gear test wording:** the gear ignored tests' line texts follow the data's format, an inference.
- **The slayer's 39 at 10** counts the two archetype talents listed in the selection: an inference that the picker lists every option of a selection.
- **Unasserted (no test, no ignored test):** the wizard's bonus feat slot as the game's `WizardFeatSelection` list against the data's `Category(Magic)`; the half-elf's two missing heritages (291); the Azata after a Trickster first ascension (story, not in the blueprints).

### Checks (after the last edit)

- `cargo test -p seeds`: `test result: ok. 477 passed; 0 failed; 65 ignored` (game_data; the other targets ok). The 65 ignored are these 63 and the two agreement long runs.
- `cargo test -p seeds --test game_data -- --ignored --list`: `65 tests, 0 benchmarks`.
- `cargo fmt --all --check`: exit 0.

## Half B

### Answer, half B (ticket 242)

**Built** (2026-11-15) on `ticket/242b-test-values-from-the-game`, base `d3b08b3`: the 26 files of half B in `libs/seeds/tests/game_data/` plus 243's `ranks_and_pools.rs` and `blueprints.rs`. Tests only: no data, no engine, no `known.txt`. Every real-data assertion in them now cites the game beside it (`// Game: <Blueprint> <guid>`, or `// Game: <path>/<File>.<Method>, WrathDecompiled 6ce4a6b`; `(equivalent model: …)` where the planner's shape differs and the legal picks don't; a const table cites per row), or moved into a test of its own named for what the game does, `#[ignore = "waits on ticket N: the game gives …"]`, failing today. Not a game fact (engine logic, data shape, guide extraction, labels and texts): left uncited.

**Decisions:** the main session's of 2026-11-15 (citation forms; differences moved, not fixed; the user's approved facts treated as differences; the ticket numbers 247–291). Ours: a moved assertion's running test keeps the rest of its checks, comparing both sides less the waiting keys where it compared a whole set (Extra Feat, Extra Mythic Ability, Life-Bonding Friendship, the sorcerer's bonus feat, the spell lists' counts, the weapon table, the repeat map); `assert_gains_of_option`'s `WAITING` levels skip only the gains comparison, each site naming its ticket.

### Counts (assertion rows of the research, after re-reading what 237 and 243 changed)

| part | files | agree, cited | equivalent model | moved and waiting (tests) | the user's (approved) | not a game fact | unsettled |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | scaling_features | 106 | 67 | 20 | 0 | 2 | ~18 |
| 2 | infusions, wild_talents, instinctual and skald rage powers, ki_powers, repeating_talents, mercies_and_exploits | 111 | 25 | 39 rows in 20 tests | 2 rows (rogue talent Weapon Focus) | 16 | 3 |
| 3 | mythic, life_bonding_friendship, negated_prerequisites, option_* | ~68 | 16 | 23 | 0 | ~16 | 7 |
| 4 | races, value_keys, pick_order, pick_values, proficiency, parameterised_feats, spells, weapons, report_gaps, summaries, totals | ~60 rows (~170 citation lines, the weapon table's 57 rows and the metamagic tables among them) | 21 | 15 | 3 rows (shield bashes, unarmed finesse, the sorcerer's 21 and the Seeker's 26) | ~14 | 8 |
| all | 28 | ~345 | ~129 | **78 tests** | 5 | ~48 | ~36 |

`// Game` lines added: 817 (the spell lists' 86 additions and the weapon table's 57 rows each count one). `ranks_and_pools.rs`: four citations completed (`SpecializationSchoolTransmutation` c459c8200e666ef4c990873d3e501b91 and `AssassinCreatePoisonAbilityStrBuffEffect` 285290cc80941bc4c97461d6f50aaad9 gained their GUIDs; `BuffScaling.ScaleBuff` and `BlueprintProgression.cs` their paths and the commit). `blueprints.rs`: all 30 GUIDs full and right.

**Waiting, per ticket (78):** 247: 2, 248: 1, 249: 2, 250: 1, 251: 1, 253: 2, 254: 1, 259: 3, 260: 1, 264: 1, 265: 6, 266: 4, 267: 4, 268: 2, 269: 4, 270: 5, 271: 3, 272: 2, 273: 1, 274: 2, 276: 4, 277: 4, 278: 1, 279: 4, 280: 1, 281: 5, 282: 1, 286: 3, 287: 2, 289: 3, 290: 2. No `to-the-user` tag is left: the only user facts in half B are the three the user approved (the weapon table's bashes and unarmed finesse, ticket 92's level-1 list, the rogue talent's Weapon Focus), each waiting on its ticket. Each test fails today for the game's reason (run with `--ignored`, 78 of 78 fail).

**Tickets picked by area** (no waiting list names the test or the test it came from): `parameterised_feats.rs` `plague_mutation_new_strain_repeats_four_times` → 278 (the Prophet of Pestilence is a shaman's; New Strain's blueprint has Ranks 4, the data Unlimited: a difference the research didn't list); `scaling_features.rs` `a_seeker_holds_the_bloodline_steps_its_own_progressions_give` → 281 (281 names the bloodrager's tests); `the_purifiers_celestial_armor_steps_at_7_11_15_and_19` → 272 (from the oracle archetypes' steps test 272 names, a difference the research didn't list).

**Renamed or reshaped tests the tickets name by their old names:** `favorite_metamagic_offers_each_metamagic_at_a_mythic_rank` → `favorite_metamagic_takes_a_metamagic` (running) + `favorite_metamagic_offers_ten_metamagics_each_needing_its_feat` (265); `an_older_link_…`'s second half → `favorite_metamagic_is_taken_again_for_another_metamagic` (249); `greater_spell_focus_in_the_sorcerers_magic_slot_names_spell_focus_in_the_later_general_slot` → `greater_spell_focus_in_the_general_slot_leans_on_spell_focus_in_the_sorcerers_magic_slot` (running, the swapped half) + `greater_spell_focus_in_the_sorcerers_magic_slot_leans_on_spell_focus_in_the_same_levels_general_slot` (248); `an_eldritch_scoundrels_sneak_attack_dice_come_at_1_3_7_11_15_and_19` → `an_eldritch_scoundrel_holds_the_rogues_own_sneak_attack_at_1_3_7_11_15_and_19` (276); the Dragon Disciple's text → `the_dragon_disciples_prerequisite_reads_without_a_character_level` (287); parts 1–3's renames are in their rows below (e.g. `the_standard_bearers_own_banner_steps_at_10_15_and_20` → `the_standard_bearer_holds_the_cavaliers_own_banner_from_1`, `the_rogue_weapon_focus_talent_cant_be_taken_twice` → `…_is_taken_once_per_weapon`, `a_hellknight_3_with_pentamic_faiths_domain_can_take_impossible_domain` → `…_cant_take_…`).

### The moved tests (file, test, ticket, the game's value)

| file | test | ticket | the game gives |
| --- | --- | --- | --- |
| `infusions.rs` | `the_kineticist_picks_an_infusion_at_1_and_at_each_later_infusion_level` | 270 | the game gives an infusion pick at kineticist 1 |
| `infusions.rs` | `each_infusion_needs_the_kineticist_level_its_blueprint_names` | 270 | the game gives no kineticist level for a level-1 infusion, 3 for Exploding Arrows and 11 for Rending Arrows |
| `infusions.rs` | `infusions_need_a_blast_not_an_element` | 270 | the game gives infusion prerequisites by blast, not by element |
| `infusions.rs` | `a_kinetic_knight_is_refused_the_infusions_that_need_the_elemental_focus_selection` | 270 | the game gives the Kinetic Knight no infusion that needs the Elemental Focus selection |
| `infusions.rs` | `the_kineticist_and_the_archetypes_that_keep_it_pick_an_infusion_at_1` | 270 | the game gives an infusion pick at kineticist 1 |
| `instinctual_rage_powers.rs` | `the_instinctual_warriors_monk_powers_need_the_levels_and_dex_their_blueprints_name` | 269 | the game gives Purity of Body level 5, Diamond Soul 14, Deflect Arrows Dex 13 |
| `ki_powers.rs` | `a_drunken_master_is_offered_neither_diamond_body_nor_diamond_soul` | 279 | the game gives the Drunken Master 12 ki powers, without Diamond Body or Diamond Soul |
| `life_bonding_friendship.rs` | `the_slots_offer_every_teamwork_feat_but_spell_chain` | 266 | the game gives Life-Bonding Friendship 12 teamwork feats, no Spell Chain |
| `life_bonding_friendship.rs` | `the_slots_ignore_prerequisites` | 266 | the game gives a sorcerer Shield Wall there |
| `mercies_and_exploits.rs` | `the_divine_hunter_picks_a_mercy_at_3_9_12_15_and_18` | 274 | the game gives the Divine Hunter a mercy at 18 |
| `mercies_and_exploits.rs` | `holy_water_jet_needs_no_alignment_and_swift_consume_needs_arcanist_1` | 282 | the game gives Holy Water Jet no alignment and Swift Consume arcanist 1 |
| `mercies_and_exploits.rs` | `greater_exploits_open_in_the_exploit_slots_from_13_with_no_slot_of_their_own` | 260 | the game gives no greater-exploit slot; greater exploits open in the exploit slots from 13 |
| `mercies_and_exploits.rs` | `an_exploiter_1_takes_extra_arcanist_exploit_with_no_exploit_picked` | 247 | the game gives Extra Arcanist Exploit to an Exploiter 1 with no exploit picked |
| `mythic.rs` | `the_aeons_mr_2_row_holds_its_second_ability_not_force_reality_again` | 266 | the game gives the Aeon's MR 2 its own second ability, Force Reality once |
| `mythic.rs` | `a_gold_dragons_dragon_feats_leave_out_stat_focus` | 266 | the game gives the dragon feats as the basic feats but eight, no Stat Focus |
| `mythic.rs` | `all_skilled_grants_every_exotic_weapon_but_the_sawtooth_saber` | 265 | the game gives All-Skilled 16 exotic weapons, not the sawtooth saber |
| `mythic.rs` | `extra_feat_offers_the_general_slots_feats_but_the_tricksters_five` | 265 | the game gives Extra Feat the basic feats but the Trickster's five |
| `mythic.rs` | `extra_mythic_ability_leaves_out_emergency_help_and_any_means_necessary` | 265 | the game gives Extra Mythic Ability no Emergency Help, Any Means Necessary |
| `mythic.rs` | `witch_wandering_hex_gives_a_daily_hex_ability_and_no_slot` | 259 | the game gives Witch Wandering Hex a daily hex ability, no hex slot |
| `mythic.rs` | `a_shamans_hex_doesnt_open_witch_wandering_hex` | 259 | the game gives Witch Wandering Hex to witch 1, Sylvan Trickster 2, Hexcrafter 4 |
| `mythic.rs` | `witch_wandering_hex_gives_a_witch_no_hex_slot` | 259 | the game gives Witch Wandering Hex a daily hex ability, no hex slot |
| `negated_prerequisites.rs` | `a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style` | 276 | the game gives the 2nd-level combat style talent no upper slayer level |
| `negated_prerequisites.rs` | `a_champion_of_the_faiths_evil_alignment_needs_only_a_non_good_one` | 267 | the game gives the Evil chosen alignment to any non-good alignment, no deity |
| `option_families.rs` | `domain_zealot_isnt_open_to_a_druids_domain` | 265 | the game gives Domain Zealot no druid's domain |
| `option_follow_ups.rs` | `the_earth_bloodline_has_its_own_elemental_movement_at_15` | 281 | the game gives the Earth bloodline an elemental movement at 15 |
| `option_follow_ups.rs` | `a_separatists_second_domain_gives_its_1st_level_power_from_cleric_1` | 253 | the game gives the Separatist's second domain power from cleric 1 |
| `option_follow_ups.rs` | `a_divine_hunters_domain_runs_two_levels_behind_its_hunter_level` | 253 | the game gives the Divine Hunter's domain hunter level - 2 |
| `option_follow_ups.rs` | `a_true_neutral_champion_of_the_faith_takes_chaos` | 267 | the game gives a true neutral Champion of the Faith Chaos |
| `option_follow_ups.rs` | `a_warpriests_evil_blessing_needs_the_deity_not_the_alignment` | 267 | the game gives a warpriest's alignment blessing a deity condition only |
| `option_follow_ups.rs` | `a_hellknight_3_with_pentamic_faiths_domain_cant_take_impossible_domain` | 265 | the game gives Impossible Domain no Pentamic Faith |
| `option_follow_ups.rs` | `a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style` | 276 | the game refuses the talent to a ranger who holds the stage-2 style |
| `option_follow_ups.rs` | `a_true_neutral_separatist_takes_no_alignment_domain` | 267 | the game gives a Separatist's alignment domain only to that alignment |
| `option_levels.rs` | `a_seekers_abyssal_bloodline_keeps_strength_of_the_abyss` | 281 | the game gives the Seeker's Abyssal bloodline Strength of the Abyss |
| `option_levels.rs` | `a_sorcerer_5_enters_dragon_disciple_at_6` | 287 | the game gives Dragon Disciple no character-level prerequisite |
| `parameterised_feats.rs` | `the_martial_weapons_include_the_spiked_shields_and_the_natural_attacks_include_sting_and_tentacle` | 289 | the game gives 29 martial weapons, and the sting and tentacle natural attacks |
| `parameterised_feats.rs` | `the_extra_feats_repeat_up_to_their_blueprints_ranks` | 250 | the game gives Extra Ki 20 ranks, and Extra Rage and six others 10 |
| `parameterised_feats.rs` | `fencing_grace_takes_weapon_focus_in_any_one_handed_piercing_weapon` | 289 | the game gives Fencing Grace every one-handed piercing weapon |
| `parameterised_feats.rs` | `a_warpriests_weapon_focus_slot_opens_only_the_weapons_he_is_proficient_with` | 251 | the game gives the warpriest's slot only the weapons he is proficient with |
| `parameterised_feats.rs` | `plague_mutation_new_strain_repeats_four_times` | 278 | the game gives New Strain 4 ranks |
| `pick_order.rs` | `greater_spell_focus_in_the_sorcerers_magic_slot_leans_on_spell_focus_in_the_same_levels_general_slot` | 248 | the game gives no order among one level's picks |
| `pick_values.rs` | `favorite_metamagic_offers_ten_metamagics_each_needing_its_feat` | 265 | the game gives ten metamagics, each needing its feat |
| `pick_values.rs` | `favorite_metamagic_is_taken_again_for_another_metamagic` | 249 | the game gives a second Favorite Metamagic for another metamagic |
| `repeating_talents.rs` | `the_rogue_weapon_focus_talent_is_taken_once_per_weapon` | 249 | the game gives the rogue talent Weapon Focus once per weapon |
| `report_gaps.rs` | `an_eldritch_scoundrel_holds_the_rogues_own_sneak_attack_at_1_3_7_11_15_and_19` | 276 | the game gives the Eldritch Scoundrel the rogue's own Sneak Attack |
| `report_gaps.rs` | `every_sorcerers_level_1_bonus_feat_is_one_of_the_games_21` | 281 | the game gives the sorcerer's level-1 bonus feat 21 feats |
| `report_gaps.rs` | `the_seekers_feats_at_5_10_and_15_are_its_own_26` | 281 | the game gives the Seeker's feats at 5, 10 and 15 a list of 26 |
| `report_gaps.rs` | `the_dragon_disciples_prerequisite_reads_without_a_character_level` | 287 | the game gives the Dragon Disciple no character-level prerequisite |
| `scaling_features.rs` | `a_human_imitator_holds_its_own_master_slayer_at_20_not_the_slayers` | 276 | the game gives the Imitator its own Master Slayer at 20, not the slayer's |
| `scaling_features.rs` | `a_seeker_holds_the_bloodline_steps_its_own_progressions_give` | 281 | the game gives the Seeker the Draconic resistances from 9, Death's Gift at 9, Snakeskin and Strength of the Abyss at 9, 13 and 17 |
| `scaling_features.rs` | `the_metamorphs_adaptive_physiology_is_granted_once_at_3` | 280 | the game gives Adaptive Physiology once, at 3 |
| `scaling_features.rs` | `the_weretouched_lycanthropic_aspect_steps_at_1_8_and_15` | 277 | the game gives Lycanthropic Aspect at 1, 8 and 15 |
| `scaling_features.rs` | `a_primalists_traded_power_never_steps_and_a_kept_one_steps_with_the_class_level` | 254 | the game gives Infernal Resistance none when traded at 4 and 4, 8 when traded at 8, and no Abyssal Bloodrage when traded at 12 |
| `scaling_features.rs` | `the_flesheaters_devour_essence_steps_at_2_7_13_and_19` | 269 | the game gives Devour Essence at 2, 7, 13 and 19 |
| `scaling_features.rs` | `the_chelaxian_divas_devastating_aria_steps_at_2_4_8_12_16_and_20` | 286 | the game gives Devastating Aria at 2, 4, 8, 12, 16 and 20 |
| `scaling_features.rs` | `the_winter_childs_arctic_native_steps_at_4_8_and_12` | 277 | the game gives Arctic Native at 4, 8 and 12, on character level |
| `scaling_features.rs` | `the_drunken_masters_drunken_ki_pool_rises_at_every_even_level` | 279 | the game gives Drunken Ki at 3, its pool rising at 4, 6, ..., 20 |
| `scaling_features.rs` | `the_drunken_masters_drunken_resilience_is_granted_once_at_9` | 279 | the game gives Drunken Resilience once, at 9 |
| `scaling_features.rs` | `the_soheis_ki_weapon_steps_at_4_5_9_13_and_17` | 279 | the game gives Ki Weapon at 4, 5, 9, 13 and 17 |
| `scaling_features.rs` | `the_wind_whisperers_haste_of_the_wind_steps_at_7_12_and_18` | 272 | the game gives Haste of the Wind at 7, 12 and 18 |
| `scaling_features.rs` | `the_child_of_the_manticores_manticore_shape_steps_at_4_8_and_14` | 277 | the game gives Manticore Shape at 4, 8 and 14 |
| `scaling_features.rs` | `the_mantis_zealots_fading_steps_at_7_10_13_and_16` | 268 | the game gives Fading at 7, 10, 13 and 16 |
| `scaling_features.rs` | `the_purifiers_celestial_armor_steps_at_7_11_15_and_19` | 272 | the game gives Celestial Armor at 7, 11, 15 and 19 |
| `scaling_features.rs` | `the_pack_rager_keeps_the_barbarians_damage_reduction_at_10` | 269 | the game gives the Pack Rager the barbarian's Damage Reduction at 10 |
| `scaling_features.rs` | `the_standard_bearer_holds_the_cavaliers_own_banner_from_1` | 273 | the game gives the Standard Bearer the cavalier's own Banner at 1, 5, 10, 15 and 20 |
| `scaling_features.rs` | `the_hospitaler_holds_the_paladins_own_smite_evil_at_1_7_13_and_19` | 274 | the game gives the Hospitaler the paladin's own Smite Evil at 1, 7, 13 and 19 |
| `scaling_features.rs` | `the_five_druid_archetypes_with_wild_shape_hold_the_classs_own_wolf` | 277 | the game gives these five the druid's own Wild Shape (Wolf), the Feyspeaker's from 6 |
| `scaling_features.rs` | `the_proclaimers_fervor_keeps_its_rank_at_8_only` | 268 | the game gives the Proclaimer Fervor at 2 and 8 only |
| `skald_rage_powers.rs` | `the_battle_scions_battle_prowess_feature_is_on_row_3_only` | 286 | the game gives the Battle Prowess feature on row 3 only |
| `skald_rage_powers.rs` | `a_rage_powers_level_gate_reads_barbarian_skald_primalist_and_shifter_levels` | 269 | the game gives 21 level gates, each read from barbarian, skald, Primalist or shifter levels |
| `skald_rage_powers.rs` | `a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot` | 247 | the game gives Extra Rage Power to a skald 3 with no rage power picked |
| `skald_rage_powers.rs` | `the_battle_scions_selection_offers_21_rage_powers` | 286 | the game gives Battle Prowess 21 rage powers |
| `skald_rage_powers.rs` | `extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection` | 264 | the game gives Extra Rage Power no Battle Prowess slot and closes it to a Battle Scion |
| `spells.rs` | `hold_monster_and_hold_person_carry_the_paralysis_descriptor` | 290 | the game gives Hold Monster and Hold Person the Paralysis descriptor |
| `spells.rs` | `the_spell_lists_hold_the_games_spells` | 290 | the game gives each list the spells of its SpellsByLevel |
| `weapons.rs` | `the_shield_bashes_and_the_unarmed_strike_have_the_games_stats` | 289 | the game gives the heavy shield bash 1d6, the light 1d4, and unarmed finesse |
| `wild_talents.rs` | `flame_shield_needs_kineticist_10_and_searing_flesh` | 271 | the game gives Flame Shield kineticist 10 |
| `wild_talents.rs` | `the_first_bonus_feat_talent_offers_its_elements_entry_feats_and_the_talent_repeats_down_the_chain` | 271 | the game gives the bonus-feat talent as a per-element chain of lists, repeatable |
| `wild_talents.rs` | `expanded_defense_gives_no_defensive_talent_to_an_element_taken_after_it` | 271 | the game gives Expanded Defense one defensive talent, none for a later element |

### Rules the code contradicts (all WrathDecompiled 6ce4a6b)

- **No order among one level's picks** (248): `UnitLogic/Class/LevelUp/Actions/SelectFeature.CalculatePriority` gives every feat selection the priority Features; `LevelUpController.AddAction` applies one priority's actions in the player's order, each checked with the earlier ones applied. The planner's fixed slot order (MetLater) refuses what the game allows.
- **A second Favorite Metamagic is legal** (249), settled from the code: `MythicAbilitySelection` (Mode Default) offers `FavoriteMetamagicSelection`, an `IFeatureSelection`; `BlueprintFeatureSelection.CanSelect` skips the rank check for a selection and refuses it only through `NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable`, when none of its options is left; each option (`FavoriteMetamagicQuicken` …) has Ranks 1 and `FavoriteMetamagicSelection` no prerequisite. `FeatureSelectionState` adds no check of its own (`CanSelectAnything` calls the selection's `CanSelect`). High confidence.
- **A selection is held once its level is reached** (247): Extra Rage Power and Extra Arcanist Exploit need the slot, not a pick held (`LevelUpHelper.AddFeaturesFromProgression`).
- **A parametrised feature repeats per value** (249, 255): the rogue talent Weapon Focus (`BlueprintParametrizedFeature.CanSelect`).
- **Proficiency holds where prerequisites are ignored** (251): Weapon Focus's `RequireProficiency` is checked on the feature's own flag, so the warpriest's slot opens only proficient weapons.
- **A selection flagged IgnorePrerequisites skips `MeetsPrerequisites`** (266): Life-Bonding Friendship.
- **Archetype level offsets** (253): `BlueprintProgression.CalcLevel` runs the Divine Hunter's domains at hunter level − 2.
- **A step on class level follows the class level** (254): `ContextRankConfig`/`AddFeatureOnClassLevel` steps don't follow traded rows (the Primalist).
- **Alignment masks** (267): `PrerequisiteAlignment.CheckInternal` applies the Champion of the Faith's chosen alignments and the Separatist's alignment domains by mask, with no deity condition; a warpriest's blessing checks only its deity.
- **The planner keeps copies of class features the game keeps whole** (273, 274, 276, 277): the Standard Bearer's banner, the Hospitaler's smite, the five druids' Wild Shape (Wolf), the Eldritch Scoundrel's sneak attack.

### Not settled

- **Part 1 (steps the blueprints don't show):** Primal Druid's Primal Size, Defender of the Hearth's Shield Block, Soul of Stone, the Inciter's Caustic Ridicule and Prompted Backstabbing (granted once, no step found); Greenrager Furious Summoning, Standard Bearer Banner of Solace, Bones of Stone (step on caster level); Flamewarden Touch of Flame, Stormwalker Wind Treader, Shadow Shaman Shadows (where the data's steps come from); Abyssal Bloodrage at 16 and 20, Inhibit Courage and Inhibit Greatness (no step found); not read: Mythic Second Bloodline, Second Bloodrager Bloodline and Second Spirit steps, Bloodline Ascendance's 20th row, the Elemental Rampager's granted-once case, the warpriest Channel Energy step levels, the Overwhelming Soul's wild-talent rows, the Primalist in the bloodrager keeping loop. These are 254's code pass.
- **Part 2:** the Instinctual Warrior powers' and the Battle Scion selection's display names (the planner's, uncited); the kineticist blast choice (Air/Electric, Water/Cold) the planner doesn't model, so Pushing, Entangling, Synaptic, Torrent, Cyclone, Spray and Chilling hold for one blast only (noted beside the running assertions; 270).
- **Part 3:** Sarenrae's and Gorum's domains (deity data); Boon Companion's prerequisite; the corrupted-branch gate on Annihilating Breath (story state); the Devil's gaze picks (written as equivalent, an open question); the sorcerer/bloodrager draconic crossing; whether the data limits the Hag bloodline to the Hag-Riven; seen, not asserted: the Gold Dragon's breath selections at rank 10 (2 in the game, 4 in all; the planner has 5).
- **Part 4:** Power Attack's −3/+9 at BAB 8 (`PowerAttackBuff` read, its arithmetic not traced); the Gold Dragon's spell levels (its list holds them at 0); the Keen-Eyed Adventurer's Acid Splash, the Drovier's Aspect of the Stag (a toggle feature in the game, not a list entry), the Chirurgeon's Breath of Life and the Flamewarden's Summon Small Elemental footnotes; each bloodline's feat list at 7, 13 and 19 (255 re-reads them).

### Seen beside the tests (for 237's mapping or the game check, not asserted)

- `magical-tail` names `MagicalTail1` 5114829572da5a04f896a8c5b67be413; the feat the selection offers is `MagicalTail` febb8fe9a2d142fb80c1be6b0b539d9d (Ranks 8).
- Five spells name another variant than the lists hold: the witch's, shaman's and lich's `poison` (lists: `Poison` d797007a142a6c0409a74b064065a15e), the shaman's `harm` (`Harm` 137af566f68fd9b428e2e12da43c1482), the aeon's `cure-critical-wounds` (`CureCriticalWounds` 0d657aa811b310e4bbd8586e60156a2d), the angel's `heal` (`Heal` ff8f1534f66559c478448723e16b6624). Left out of the list comparison as the same spell.
- `WizardSpellList` has a 281st entry, `FingerOfDeathSithhud` e03024c8a03f454db5b78660f524757d at 7, Finger of Death again.
- The research's "Thirsting Entanglement isn't on the druid's list" doesn't hold by GUID: `DruidSpellList` has `ThirstingEntangle` at 4; the druid's difference is Burst of Nettles at 3 (not on the game's list), and Bone Fists 2, Nature's Exile 3, Stabilize 0 added.
- A Human-Raised half-orc keeps the double axe as martial (keyed to `HalfOrcRace`); the halfling's Sling Staff as martial is missing from `races.ron` (the research's, 291).

### Tests

`cargo test -p seeds` passes (game_data: 476 passed, 80 ignored, of which 78 are this ticket's and 2 the agreement sweeps'); `cargo test -p seeds --test game_data -- --ignored --skip agreement::` fails all 78 for the game's reason; each of the 78 is in both `-- --list` and `-- --ignored --list`; `cargo fmt --all --check` passes.


## Done when

- Every real-data assertion either cites its game source, or names the ticket that brings the data to the game's value.
- The Answer lists the differences found, with what each became.
- `scripts/check.sh` passes, apart from tests marked as waiting on a named ticket.
