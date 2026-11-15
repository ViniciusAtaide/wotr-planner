# A parametrised feat at one value, in a feat list or a grant

Type: build (engine and data)
Status: open
Blocked by: none
Model: Opus to build: a new shape in the data and its readers.
Map: [WotR Build Planner](../map.md)

## Problem

The game has a feature per skill for Skill Focus (`SkillFocusAcrobatics`, …, `SkillFocusUseMagicDevice`, under `SkillFocusSelection`), and a list or a row names one of them. The planner holds one `skill-focus` feat taken for any skill, so its lists and rows offer every skill where the game offers one:
- **Bloodline feat lists:** each of the sorcerer's bloodline lists holds one or two given skills (`BloodlineAbyssalFeatSelection` 7781831170d794a4d8f147a1326c35fd: `SkillFocusLoreReligion`; `BloodlineDraconicFeatSelection` f4b011d090e8ae543b1441bd594c7bf7: Acrobatics and Knowledge (Arcana); …).
- **Adaptability** (`Adaptability` 26a668c5a8c22354bac67bcd42e09a3f): the 11 per-skill features.
- **The Disenchanter's lists** (`DisenchanterFeatSelection` 234772e17012d554f8661e3f365b68ae, `DisenchanterFeatSelection9` f20d10e43ed9ede43959678bb246c531, `DisenchanterFeatSelection12` 6b0dd75a4f3872543842179cf6e456e0): `SkillFocusKnowledgeArcana` and `DisenchanterSpellFocusAbjuration`, a feature of its own, where the planner offers `skill-focus` and `spell-focus`.
- **Grants:** the Chirurgeon's Skill Focus (Lore Religion) at 5 and the Enlightened Philosopher's Skill Focus (Knowledge World) at 1.
- **The ranger's level-10 style lists** hold `skill-focus` in the planner and no Skill Focus in the game.
- [238](238-the-game-check.md)'s Answer left these "unchanged and not known"; Mythic Skill Focus's same-value prerequisite stays with [244](244-the-game-data-laid-out-by-blueprint.md) (`known.txt`).

## Findings (the game check, `known.txt` section 255)

| The check's group (area / kind) | Findings |
| --- | --- |
| option lists / in game only | 27 |
| option lists / in planner only | 21 |
| progression rows / in game only | 2 |
| progression rows / in planner only | 2 |
| **all** | **52** |

The 20 entries: `archetype alchemist/chirurgeon`, `archetype oracle/enlightened-philosopher`, `feat list half-elf/adaptability`, `feat list ranger/combat-style-feat/archery-feats-10`, `feat list ranger/combat-style-feat/menacing-feats-10`, `feat list ranger/combat-style-feat/two-handed-weapon-feats-10`, `feat list ranger/combat-style-feat/two-weapon-combat-feats-10`, `feat list ranger/combat-style-feat/weapon-and-shield-feats-10`, `feat list sorcerer/bloodline/abyssal/feats`, `feat list sorcerer/bloodline/arcane/feats`, `feat list sorcerer/bloodline/celestial/feats`, `feat list sorcerer/bloodline/draconic/feats`, `feat list sorcerer/bloodline/elemental/feats`, `feat list sorcerer/bloodline/fey/feats`, `feat list sorcerer/bloodline/infernal/feats`, `feat list sorcerer/bloodline/serpentine/feats`, `feat list sorcerer/bloodline/undead/feats`, `feat list warpriest/disenchanter/bonus-feats-12`, `feat list warpriest/disenchanter/bonus-feats-3`, `feat list warpriest/disenchanter/bonus-feats-9`.

The report lists each finding under its group, marked `[known: 255]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- A feat list's option, and a row's grant, may name a parametrised feat at one value (Skill Focus (Lore Religion)); the pickers offer it at that value only, and the sheet and the link read it.
- Each list and grant above takes the game's values; the Disenchanter's Spell Focus (Abjuration) maps to `DisenchanterSpellFocusAbjuration`; the ranger's level-10 lists lose `skill-focus`.
- The game check compares such an option with the game's per-value feature.

## Tests 242 marks as waiting on this ticket

- None of 242's rows assert these lists; `report_gaps.rs`' `bloodline_feats` helpers and `follow_ups.rs`' Adaptability rows are re-read against the new shape.

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 255 (the game check exits 0, run by the main session).
- A real-data test per list kind (a bloodline list, Adaptability, a Disenchanter list, a grant) asserts the game's value with its blueprint and GUID.
- The scoped checks pass.
