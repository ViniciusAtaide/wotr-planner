# Mythic abilities' and mythic feats' prerequisites and lists

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [247](247-a-selection-counts-from-the-level-that-grants-it.md), [249](249-selections-and-parametrised-feats-are-taken-again.md), [256](256-the-game-checks-misses.md)
Model: Opus to build: each prerequisite and list is read from its blueprint.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B part 3 `mythic.rs`, `option_families.rs`, `option_follow_ups.rs`; part 4 `pick_values.rs`) differ on `mythic/abilities.ron`, `mythic/feats.ron` and the two mythic pools:
- **Prerequisites naming selections or other blueprints:** Domain Zealot (`DomainMastery` 2de64f6a1f2baee4f9b7e52e3f046ec5) needs `DomainsSelection`, `DivineHunterDomainsSelection`, `HellknightDisciplinePentamicFaith`, `TricksterLoreReligionTier2Parametrized` or `FakeDivineSparkFeature`, so a druid's domain doesn't count (a druid with Weather, a Blight Druid, a Winter Child are refused); Impossible Domain (`ExtraDomain` 213a8480d22206b45acbfa0619ca5aaf) needs `DomainsSelection`, `DivineHunterDomainsSelection` or `FakeDivineSparkSelection`, so a Hellknight 3 with Pentamic Faith's domain can't take it. Second Bloodline (`SecondBloodline` 3cf2ab2c320b73347a7c21cf0d0995bd) and Bloodline Ascendance name the bloodline selections and progressions (the Empyreal's, the Sage's, the Sylvan's, the Seeker's, the Nine-Tailed Heir's) where the planner asks sorcerer level 1; Second Bloodrager Bloodline names `BloodragerBloodlineSelection`; Second Mystery names the mystery-gift selection; Second Spirit no spirit selection; Abundant Bane, Abundant Ki Pool, Everlasting Judgement, Kinetic Overcharge, Over-Infused Blasts, Limitless Rage, School Tolerance, Defensive Study, Mythical Beast and Mythic Sneak Attack name the game's features.
- **Values:** Any Means Necessary (`FreeFromEvil` a0a814897cd33ed4ea6ffca4b0e1197a) needs Hellknight 7, not 1; Greater Enduring Spells needs Enduring Spells; Destiny Beyond Birth needs the Mongrelman race.
- **Mythic Armor Focus:** the game's pool offers one selection (`ArmorFocusMythicSelection` 068f833e8aea4cd58df003543e11b207) whose options need the armour's Armor Focus (`ArmorFocusLight`/`Medium`/`Heavy`); the planner lists the nine variants in the pool, each needing `ArmorFocusSelection`.
- **The pools:** `MythicAbilitySelection` (ba0e5a900b775be4a99702f1ed08914d) holds the three archetype Second Mysteries the planner's pool lacks. Extra Feat (`ExtraFeatMythicFeat` e10c4f18a6c8b4342afe6954bde0587b) offers the basic feats less five Trickster feats (Stat Focus, Completely Normal Spell, the three Improved² Critical feats); Extra Mythic Ability (`ExtraMythicAbilityMythicFeat` 8a6a511c55e67d04db328cc49aaad2b8) offers 63 of the 67, leaving out Emergency Help (`EmergencyHelpFeature` 22ce17c5253183b42845a5bfc54f8afa), Any Means Necessary and the Herbalist's and Lone Strider's Second Mysteries.
- **Favorite Metamagic** (`FavoriteMetamagicSelection` 503fb196aa222b24cb6cfdc9a284e838) has 10 options, no Completely Normal and no Heighten, each needing its metamagic feat (`FavoriteMetamagicPersistent` eb6483b366a14a83afda1963799c3387 needs Persistent Spell). Taken again: [249](249-selections-and-parametrised-feats-are-taken-again.md).
- **All-Skilled** (`AllSkilledFeature` f3bc6f9c855b2fb4e9aea364b8163aca) grants 16 exotic weapon proficiencies: not the sawtooth sabre.
- Witch Wandering Hex is [259](259-witch-wandering-hex-is-a-daily-ability.md)'s.

## Findings (the game check, `known.txt` section 265)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / not in game | 40 |
| prerequisites / missing in planner, a blueprint it doesn't map | 26 |
| prerequisites / missing in planner | 18 |
| option lists / in planner only | 9 |
| option lists / in game only | 4 |
| prerequisites / value differs | 1 |
| **all** | **98** |

The 32 entries: `mythic ability abundant-bane`, `mythic ability abundant-ki-pool`, `mythic ability abundant-smite-smite-chaos`, `mythic ability any-means-necessary`, `mythic ability bloodline-ascendance`, `mythic ability defensive-study`, `mythic ability domain-zealot`, `mythic ability everlasting-judgement`, `mythic ability greater-enduring-spells`, `mythic ability impossible-domain`, `mythic ability kinetic-overcharge`, `mythic ability limitless-rage`, `mythic ability mythical-beast`, `mythic ability over-infused-blasts`, `mythic ability school-tolerance`, `mythic ability second-bloodline`, `mythic ability second-bloodrager-bloodline`, `mythic ability second-mystery`, `mythic ability second-spirit`, `mythic feat destiny-beyond-birth`, `mythic feat mythic-armor-focus-heavy-armor-assault`, `mythic feat mythic-armor-focus-heavy-armor-avoidance`, `mythic feat mythic-armor-focus-heavy-armor-endurance`, `mythic feat mythic-armor-focus-light-armor-assault`, `mythic feat mythic-armor-focus-light-armor-avoidance`, `mythic feat mythic-armor-focus-light-armor-endurance`, `mythic feat mythic-armor-focus-medium-armor-assault`, `mythic feat mythic-armor-focus-medium-armor-avoidance`, `mythic feat mythic-armor-focus-medium-armor-endurance`, `mythic feat mythic-sneak-attack`, `pool mythic abilities`, `pool mythic feats`.

The report lists each finding under its group, marked `[known: 265]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Each prerequisite, pool and list becomes the game's, with [247](247-a-selection-counts-from-the-level-that-grants-it.md)'s form where the game names a selection; Mythic Armor Focus becomes the game's selection (links keep the nine keys as aliases).
- A data note lists every change.

## Tests 242 marks as waiting on this ticket

- `class_rules.rs`: `domain_zealot_needs_a_druids_domain_and_boon_companion_its_companion` (a druid with Weather, a Blight Druid with Death and a Winter Child are refused).
- `option_families.rs`: `domain_zealot_needs_a_domain_held_whoever_gave_it` (the Winter Child case), `impossible_domain_at_mr_1`, `no_druid_can_take_impossible_domain_whatever_its_domain` (re-read).
- `option_follow_ups.rs`: `a_hellknight_3_with_pentamic_faiths_domain_can_take_impossible_domain` (refused).
- `mythic.rs`: `all_skilled_grants_every_weapon_armour_and_shield_proficiency` (16 exotics), `extra_feat_at_mr_2_gives_a_feat_slot_at_level_7_offering_what_the_general_slot_there_does` (less the five), `extra_mythic_ability_gives_a_mythic_ability_slot_offering_what_a_ranks_does` (63).
- `pick_values.rs`: `favorite_metamagic_offers_each_metamagic_at_a_mythic_rank` (10, each needing its feat), `favorite_metamagic_for_persistent_spell_is_named_and_round_trips` (needs Persistent Spell first).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 265 (exit 0, run by the main session).
- The tests above hold the game's answers, cited; the scoped checks pass.
