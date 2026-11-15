# Class tables: saves, BAB and the Arcane Trickster's skill points

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: none
Model: Sonnet to build: each value is read from the class's stat progression, per level.
Map: [WotR Build Planner](../map.md)

## Problem

The game check's class tables (and 242's research) differ from the data:
- **Saves and BAB** are compared per level with the class's or archetype's `BlueprintStatProgression` (`SavesHigh` ff4662bde9e75f145853417313842751, `SavesLow` dc0c7c1aba755c54f96c089cdf7d14a3, the prestige classes' own tables, …). Each finding names the first level that differs and how many do: the hunter's and its archetypes' Will at 7, the paladin's and its archetypes' Will at 20, the warpriest's and its archetypes' Will at 20, the shifter's and its archetypes' Reflex at 2, the four prestige classes' Will at 4, the Stalwart Defender's three saves, the Feyspeaker's BAB.
- **Archetypes that replace a save** (242, `archetype_saves.rs`): besides the Arcane Enforcer, the Dark Elementalist (`DarkElementalistArchetype` f12f18ae8842425489d29f302e69134c: Fortitude and Reflex low, Will high), the Priest of Balance (`PriestOfBalanceArchetype` a4560e3fb5d247d68fb1a2738fcc0855: Fortitude low) and the Traditional Monk (`TraditionalMonk` 8f3242d209994884bbb6c0999660f4df: Will high). Kinetic Sharpshooter, Psychokineticist and Weretouched restate their class's saves.
- **The Arcane Trickster's skill points** (`ArcaneTricksterClass` 9c935a076d4fe4d4999fd48d853e3cf3): 3, not 4. **The user's decision (2026-11-15):** the game's 3 is applied without asking again, against `research/skills-from-user.md`.
- **Class skills given by a feature** (242, `class_skills.rs`; not the user's facts): the sorcerer's Seeker (`SeekerArchetype` 7229db6fc0b07af4180e783eed43c4d9) gains Trickery at 1 through `SorcererSeekerTrapfinding` (b1c4f14bc87534e4da7ecd8e0b0658d2); the Animal domain's Lore (Nature) comes with `AnimalDomainBaseFeature` (d577aba79b5727a4ab74627c4c6ba23c) at the domain's level 1 (`AnimalDomainProgression` 23d2f87aa54c89f418e68e790dba11e0, `…Separatist` e3d58ae932554987a6e247e281efc2f0, `…Druid` a75ad4936e099c54881cf553e2110703), not with the companion at 4.

## Findings (the game check, `known.txt` section 261)

| The check's group (area / kind) | Findings |
| --- | --- |
| class tables / Will differs | 33 |
| class tables / Reflex differs | 11 |
| class tables / Fortitude differs | 3 |
| class tables / BAB differs | 1 |
| class tables / skill points differ (the user's facts) | 1 |
| **all** | **49** |

The 44 entries: `archetype cleric/priest-of-balance`, `archetype druid/feyspeaker`, `archetype hunter/colluding-scoundrel`, `archetype hunter/divine-hound`, `archetype hunter/divine-hunter`, `archetype hunter/forester`, `archetype hunter/tandem-executioner`, `archetype hunter/urban-hunter`, `archetype hunter/wandering-marksman`, `archetype kineticist/dark-elementalist`, `archetype monk/traditional-monk`, `archetype paladin/divine-guardian`, `archetype paladin/divine-hunter-paladin`, `archetype paladin/divine-scion`, `archetype paladin/hospitaler`, `archetype paladin/martyr`, `archetype paladin/stonelord`, `archetype paladin/tortured-crusader`, `archetype paladin/warrior-of-the-holy-light`, `archetype shifter/child-of-the-manticore`, `archetype shifter/dragonblood-shifter`, `archetype shifter/feyform-shifter`, `archetype shifter/fiendflesh-shifter`, `archetype shifter/griffonheart-shifter`, `archetype shifter/rageshaper`, `archetype shifter/weretouched`, `archetype shifter/wild-effigy`, `archetype warpriest/champion-of-the-faith`, `archetype warpriest/cult-leader`, `archetype warpriest/disenchanter`, `archetype warpriest/feral-champion`, `archetype warpriest/mantis-zealot`, `archetype warpriest/proclaimer`, `archetype warpriest/shieldbearer`, `class arcane-trickster`, `class hunter`, `class loremaster`, `class mystic-theurge`, `class paladin`, `class shifter`, `class stalwart-defender`, `class student-of-war`, `class warpriest`, `class winter-witch`.

The report lists each finding under its group, marked `[known: 261]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Each save and BAB takes the game's progression, per class and archetype; the archetype save overrides above are added.
- The Arcane Trickster has 3 skill points; the Seeker's Trickery and the Animal domain's Lore (Nature) come at their game levels. A data note lists every change.

## Tests 242 marks as waiting on this ticket

- `archetype_saves.rs`: `only_the_arcane_enforcer_replaces_a_save_progression` (the list becomes the four).
- `class_skills.rs`: `options_class_skills_are_the_guides_on_their_first_row_and_the_animal_domains_on_its_4th` (the Animal row's special case goes; the test is renamed), `the_animal_domains_lore_nature_comes_with_its_companion_at_the_holders_4th_domain_level` (from domain level 1, for the cleric, the Separatist and the druid), `an_overwhelming_mage_and_a_seeker_hold_their_bloodlines_class_skill_and_no_class_skill_feature` (the Seeker also holds Trickery).
- `follow_ups.rs`: `the_arcane_trickster_lacks_use_magic_device_and_the_dragon_disciple_has_knowledge_world` gains the skill-point citation.

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 261 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
