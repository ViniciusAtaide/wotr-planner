# Class skills and the Defender of the Hearth's skill points (to the user)

Type: question (the user's facts), then build (data)
Status: open, waiting on the user
Blocked by: the user's answer
Model: Sonnet to build once answered: each value is copied from the game.
Map: [WotR Build Planner](../map.md)

## Problem

The game check's class tables tag class skills and skill points "(the user's facts)": they come from [research/skills-from-user.md](../research/skills-from-user.md) (2026-11-01). Since [234](234-game-blueprints-as-a-source.md) the game wins over them too, but a difference with one goes to the user first. The user decided seven 242 differences on 2026-11-15 (the Arcane Trickster's 3 skill points among them, in [261](261-class-tables-saves-bab-and-skill-points.md)); these were not among them.

## To the user

Each row: the game's class skills (`ReplaceClassSkills` or the class's `ClassSkills`) against the data's, from the report.

| Entry | Blueprint | The game adds | The game drops |
| --- | --- | --- | --- |
| alchemist/metamorph | `MetamorphArchetype` ac242ddeaa05af449bfc5230231fab84 | — | Stealth |
| arcanist/magic-deceiver | `MagicDeceiver` 5c77110cd0414e7eb4c2e485659c9a46 | Persuasion | — |
| hunter/divine-hunter | `DivineHunterArchetype` f996f0a18e5d945459e710ee3a6dd485 | Knowledge (Arcana), Lore (Religion) | Knowledge (World) |
| magus/spell-dancer | `SpellDancerArchetype` 1125145639129cf45b6b9b674cbd62b1 | Mobility | Athletics |
| monk/sohei | `SoheiArchetype` fad7c56737ed12e42aacc330acc86428 | — | Lore (Nature) |
| oracle/seeker | `SeekerArchetype` c539d65b7682c8d469e6f122922f1dee | — | Trickery |
| oracle/wind-whisperer | `WIndWhispererArchetype` 77769e43e63647c2b32298efcc577aca | — | Mobility, Stealth |
| ranger/sable-company-marine | `SableMarineArchetype` 475117e837fa461589005a5339f653a0 | Mobility | Knowledge (Arcana) |
| rogue/sylvan-trickster | `SylvanTricksterArchetype` 490394869f666c141bf8647b1a365220 | — | Knowledge (World) |
| sorcerer/empyreal-sorcerer | `EmpyrealSorcererArchetype` aa00d945f7cf6c34c909a29a25f2df38 | — | Lore (Religion) |
| sorcerer/sylvan-sorcerer | `SylvanSorcererArchetype` 711d5024ecc75f346b9cda609c3a1f83 | — | Lore (Nature) |
| witch/witch-of-the-veil | `WitchOfTheVeilArchetype` 08045a390a764d71b2150fd122e05e19 | — | Stealth, Trickery |
| stalwart-defender | `StalwartDefenderClass` d5917881586ff1d4d96d5b7cebda9464 | Intimidate, Mobility | — |

And skill points: **fighter/defender-of-the-hearth** (`DefenderOfTheHearthArchetype` 6c96dc1f5573487399336cd351ef8f2c): the game 3, the data 2.

Caution for the answer: some of the game's drops may be a skill the archetype's own feature or bloodline gives back (the Empyreal's and the Sylvan's bloodline class skill, the Sohei's): the builder checks each against the features before applying.

## Findings (the game check, `known.txt` section 262)

| The check's group (area / kind) | Findings |
| --- | --- |
| class tables / class skills differ (the user's facts) | 13 |
| class tables / skill points differ (the user's facts) | 1 |
| **all** | **14** |

The 14 entries: `archetype alchemist/metamorph`, `archetype arcanist/magic-deceiver`, `archetype fighter/defender-of-the-hearth`, `archetype hunter/divine-hunter`, `archetype magus/spell-dancer`, `archetype monk/sohei`, `archetype oracle/seeker`, `archetype oracle/wind-whisperer`, `archetype ranger/sable-company-marine`, `archetype rogue/sylvan-trickster`, `archetype sorcerer/empyreal-sorcerer`, `archetype sorcerer/sylvan-sorcerer`, `archetype witch/witch-of-the-veil`, `class stalwart-defender`.

The report lists each finding under its group, marked `[known: 262]`, with the entry, the blueprint's name and GUID, and both values.

## Change

Once the user answers: apply each accepted row (data and `research/skills-from-user.md`), keep each refused one with the user's reason in the data's note, and list the refusals in `known.txt` under this ticket.

## Done when

- The user's answer is recorded in the Answer, row by row.
- The game check reports none of this ticket's findings, or only the refused ones, each known under this ticket with the user's reason; the scoped checks pass.
