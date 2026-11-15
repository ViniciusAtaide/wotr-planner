# Deities, domains and blessings

Type: build (data, and the deity's domain model)
Status: open for its model part only, after 293 (the user, 2026-11-15): its mechanical data is 293's, the importer's
Blocked by: none
Model: Opus to build: the deity's allowed domains change model, and each faith rule is read from its blueprint.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `alignment.rs`, `deities.rs`; half B `negated_prerequisites.rs`, `option_follow_ups.rs`) differ on the faith rules:
- **A deity's domains:** each domain option (`<Domain>DomainProgression`) and each warpriest blessing (`<Domain>BlessingFeature`) needs the deity's `<Domain>DomainAllowed` feature; the planner filters the domain slot by the deity's list, which the check can't see (the `XDomainAllowed` leaves). Death, Murder and Undead exclude each other, Water and Ice likewise; the Animal and Scalykind domains exclude the Sacred Huntsmaster. Decide (recommended): the deity grants the game's `DomainAllowed` facts and the options need them, so the data reads as the game; otherwise teach the check the planner's filter.
- **Blessings drop their alignment rule** (ticket 86's question 7): the Chaos, Evil, Good and Law blessings need only the deity's allowance (`ChaosBlessingFeature` 528e316f9f092954b9e38d3a82b1634a; `EvilBlessingFeature` f38f3abf6ca3a07499a61f96e342bb16), so a lawful neutral follower of Asmodeus (`AsmodeusFeature` a3a5ccc9c670e6f4ca4a686d23b89900) takes Evil. The game has 34 blessings; the data 25 (missing Artifice, Charm, Community, Glory, Knowledge, Plant, Rune, War, Scalykind; `ArtificeBlessingFeature` 04e5d412a7b94f809a28607618b71619): Torag offers five.
- **The Champion of the Faith's** chosen alignment (`ChampionOfTheFaithAlignmentSelection` 4146f0ff2c5e461488b0ace718020670) offers its own four features (`ChampionOfTheFaithEvilFeature` cf6e5f8927653434eab6179cbc76eb04, …), each by alignment alone (Evil: not good; Chaos: not lawful; …), no deity check; the planner shares the blessings. The Feral Champion's blessing is a feature, not a choice.
- **Deities:** Asmodeus also the light mace; Norgorber, Urgathoa and Zon-Kuthon also the Murder subdomain (Zon-Kuthon Undead too) (`NorgorberFeature` 805b6bdc8c96f4749afc687a003f9628, `UrgathoaFeature` 812f6c07148088e41a9ac94b56ac2fc8, `ZonKuthonFeature` f7eed400baa66a744ad361d4df0e6f1b). The deity's weapon goes to every cleric, inquisitor and warpriest, the Demonbane Priest (`DevilbanePriestArchetype` 2537d1312a462834aadbd1153a2a0d54) included; Irori's grant (`IroriFeature` 23a77a5985de08349820429ce1b5a234) counts cleric, inquisitor and warpriest levels only, so a Deliverer gets no Improved Unarmed Strike (`Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs`, `IsFeatureShouldBeApplied`, WrathDecompiled 6ce4a6b).
- **Classes:** the cleric, inquisitor, paladin and warpriest refuse Atheism; the paladin needs one of eight deities; the oracle refuses Apsu.
- **Archetypes:** the Angelfire Apostle (`AngelfireApostleArchetype` 857bc9fadf70f294795a9cba974a48b8) has no alignment rule but 26 deities exclude it (Cayden Cailean, Desna, Erastil, Iomedae, Pulura, Sarenrae, Shelyn, Torag and Apsu remain); the Priest of Balance needs one of seven deities (Abadar, Calistria, Gorum, Gozreh, Irori, Nethys, Pharasma); the Feral Champion (`FeralChampionArchetype` f68ca492c9c15e241ab73735fbd0fb9f) excludes 25 deities rather than allowing three; the Mantis Zealot's `not DeitySelection` branch; the Dark Elementalist (Evil), the Rageshaper (NG, TN, NE, CG, CN, CE, replacing the shifter's) and the Hag of Gyronna (`DarkSisterArchetype` 411fa458481e44f0855d47a19358874b: CN, NE, CE, not a Gyronna requirement) gain their alignment rules; the Deliverer (`DelivererArchetype` 738c5666f8ef46c45942df9100a0d254) has no faith rule.
- **Separatist alignment domains** (`ChaosDomainProgressionSeparatist` 714706ca6958440cbbb217c2d167e6ec, Evil, Good, Law likewise) need that alignment: a TN Separatist takes none of the four.
- **The cleric's rows:** Detect Magic at 1 for the class and its archetypes, the Demonbane Priest's proficiencies, the Herald Caller's and Separatist's own rows; the Demonbane Priest's teamwork list (no Spell Chain or Volley Fire; prerequisites not ignored). The Primary/Secondary domain split is [244](244-the-game-data-laid-out-by-blueprint.md)'s.

## Findings (the game check, `known.txt` section 267)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner, a blueprint it doesn't map | 75 |
| prerequisites / missing in planner | 47 |
| progression rows / in game only | 10 |
| prerequisites / not in game | 9 |
| option lists / in planner only | 8 |
| option lists / in game only | 4 |
| progression rows / in planner only | 4 |
| option lists / a choice the game doesn't offer | 1 |
| option lists / ignores prerequisites or not | 1 |
| **all** | **159** |

The 83 entries: `archetype cleric/angelfire-apostle`, `archetype cleric/crusader`, `archetype cleric/demonbane-priest`, `archetype cleric/ecclesitheurge`, `archetype cleric/herald-caller`, `archetype cleric/priest-of-balance`, `archetype cleric/separatist`, `archetype kineticist/dark-elementalist`, `archetype shifter/rageshaper`, `archetype warpriest/feral-champion`, `archetype warpriest/mantis-zealot`, `archetype witch/hag-of-gyronna`, `class cleric`, `class inquisitor`, `class oracle`, `class paladin`, `class warpriest`, `feat list cleric/demonbane-priest/bonus-feats`, `selection cleric/impossible-domain`, `selection option cleric/domain/air`, `selection option cleric/domain/animal`, `selection option cleric/domain/artifice`, `selection option cleric/domain/chaos`, `selection option cleric/domain/charm`, `selection option cleric/domain/community`, `selection option cleric/domain/darkness`, `selection option cleric/domain/death`, `selection option cleric/domain/destruction`, `selection option cleric/domain/earth`, `selection option cleric/domain/evil`, `selection option cleric/domain/fire`, `selection option cleric/domain/glory`, `selection option cleric/domain/good`, `selection option cleric/domain/healing`, `selection option cleric/domain/ice`, `selection option cleric/domain/knowledge`, `selection option cleric/domain/law`, `selection option cleric/domain/liberation`, `selection option cleric/domain/luck`, `selection option cleric/domain/madness`, `selection option cleric/domain/magic`, `selection option cleric/domain/murder`, `selection option cleric/domain/nobility`, `selection option cleric/domain/plant`, `selection option cleric/domain/protection`, `selection option cleric/domain/repose`, `selection option cleric/domain/rune`, `selection option cleric/domain/scalykind`, `selection option cleric/domain/strength`, `selection option cleric/domain/sun`, `selection option cleric/domain/travel`, `selection option cleric/domain/trickery`, `selection option cleric/domain/undead`, `selection option cleric/domain/war`, `selection option cleric/domain/water`, `selection option cleric/domain/weather`, `selection option warpriest/blessings/air-domain`, `selection option warpriest/blessings/animal-domain`, `selection option warpriest/blessings/chaos-domain`, `selection option warpriest/blessings/darkness-domain`, `selection option warpriest/blessings/death-domain`, `selection option warpriest/blessings/destruction`, `selection option warpriest/blessings/earth`, `selection option warpriest/blessings/evil-domain`, `selection option warpriest/blessings/fire`, `selection option warpriest/blessings/good-domain`, `selection option warpriest/blessings/healing-domain`, `selection option warpriest/blessings/law-domain`, `selection option warpriest/blessings/liberation`, `selection option warpriest/blessings/luck`, `selection option warpriest/blessings/madness`, `selection option warpriest/blessings/magic`, `selection option warpriest/blessings/nobility`, `selection option warpriest/blessings/protection`, `selection option warpriest/blessings/repose`, `selection option warpriest/blessings/strength`, `selection option warpriest/blessings/sun`, `selection option warpriest/blessings/travel`, `selection option warpriest/blessings/trickery`, `selection option warpriest/blessings/water`, `selection option warpriest/blessings/weather`, `selection warpriest/champion-of-the-faith/chosen-alignment`, `selection warpriest/feral-champion/blessing`.

The report lists each finding under its group, marked `[known: 267]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The domain model is decided and recorded; every faith rule above becomes the game's; a data note lists each change.

## Tests 242 marks as waiting on this ticket

- `alignment.rs`: `every_archetype_with_an_alignment_rule_is_buildable_at_an_alignment_it_allows` (`ALIGNMENT_RULES` becomes the game's eight; the Angelfire Apostle's row goes).
- `deities.rs`: `the_26_deities_have_the_tables_alignments_weapons_and_domains` (Asmodeus, Norgorber, Urgathoa, Zon-Kuthon), `a_warpriest_of_torag_is_offered_his_blessings` (five), `an_alignment_domain_needs_its_alignment` (the blessings' half), `some_archetypes_need_one_deity_or_atheism` (the Deliverer), `a_deliverer_of_irori_holds_improved_unarmed_strike_and_can_take_stunning_fist` (he doesn't), `the_angelfire_apostle_needs_a_good_deity_and_the_priest_of_balance_a_neutral_one` (the game's deity lists), `the_champion_of_the_faiths_alignment_is_one_his_deity_offers` (by alignment alone), `the_hag_of_gyronna_is_chaotic_neutral_or_evil`.
- `negated_prerequisites.rs`: `every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out` (the Champion's and the Separatist's rows).
- `option_follow_ups.rs`: `a_champion_of_the_faiths_alignment_blessing_is_his_own_alignment` (TN takes Chaos; a warpriest's Evil blessing needs no evil alignment), `a_lawful_good_separatist_cant_take_chaos_or_evil_as_his_second_domain` (TN refused all four).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 267 (exit 0, run by the main session).
- The tests above hold the game's answers, cited; the scoped checks pass.
