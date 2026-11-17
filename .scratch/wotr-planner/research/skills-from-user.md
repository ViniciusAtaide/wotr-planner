# Class skills, skill points and backgrounds (from the user)

Facts the user gave on 2026-11-01, checked against `game-data/` and the saved guide pages the same day. Script: `scratchpad/t32/compare_cs.py`.

## Impossible Domain

The mythic ability grants one domain, chosen from the full list of domains.

## Skill points per level

| Class | Skill points | Archetypes with a different value |
| --- | --- | --- |
| Alchemist, Barbarian, Bloodrager, Cavalier, Monk, Oracle, Shifter, Skald, Slayer | 3 | — |
| Arcanist, Kineticist, Magus, Shaman, Sorcerer, Wizard | 2 | — |
| Bard, Hunter, Inquisitor, Ranger | 4 | — |
| Cleric | 2 | Herald Caller, Ecclesitheurge: 3 |
| Druid | 3 | Feyspeaker: 4 |
| Fighter | 2 | Armiger, Mutation Warrior: 3 |
| Paladin | 2 | Divine Scion, Tortured Crusader: 3 |
| Rogue | 5 | Eldritch Scoundrel, Rowdy: 3 |
| Warpriest | 2 | Cult Leader: 3 |
| Witch | 2 | Stigmatized Witch: 3 |
| Prestige: Aldori Swordlord, Dragon Disciple, Eldritch Knight, Hellknight, Hellknight Signifer, Mystic Theurge, Stalwart Defender, Winter Witch | 2 | — |
| Prestige: Assassin, Duelist, Loremaster | 3 | — |
| Prestige: Arcane Trickster, Student of War | 4 | — |

**Against the data:** everything matches except **Tortured Crusader**. The data has no skill points or class skills for it. The guide's paladin page agrees with the user on 3 points and gives its own class skills: Athletics, Knowledge (Arcana), Knowledge (World), Lore (Nature), Lore (Religion), Perception, Persuasion.

## Class skills

The user's table covers the 16 Kingmaker base classes and 7 prestige classes. It doesn't cover Arcanist, Bloodrager, Cavalier, Hunter, Oracle, Shaman, Shifter, Skald, Warpriest, Witch, Assassin, Hellknight, Hellknight Signifer, Loremaster, Student of War or Winter Witch.

| Skill | Base classes | Prestige classes |
| --- | --- | --- |
| Athletics | Barbarian, Bard, Druid, Fighter, Inquisitor, Magus, Monk, Ranger, Rogue, Slayer | Arcane Trickster, Eldritch Knight, Stalwart Defender |
| Mobility | Barbarian, Bard, Inquisitor, Kineticist, Monk, Rogue, Slayer | Aldori Swordlord, Arcane Trickster, Dragon Disciple, Duelist |
| Stealth | Bard, Inquisitor, Kineticist, Monk, Ranger, Rogue, Slayer | Arcane Trickster |
| Trickery | Alchemist, Bard, Inquisitor, Rogue | Arcane Trickster |
| Knowledge (Arcana) | Alchemist, Bard, Cleric, Druid, Inquisitor, Magus, Paladin, Ranger, Sorcerer, Wizard | Arcane Trickster, Dragon Disciple, Eldritch Knight, Mystic Theurge |
| Knowledge (World) | Alchemist, Bard, Cleric, Druid, Fighter, Inquisitor, Magus, Monk, Paladin, Ranger, Rogue, Slayer, Wizard | Aldori Swordlord, Arcane Trickster, Dragon Disciple, Eldritch Knight |
| Lore (Nature) | Alchemist, Barbarian, Bard, Druid, Fighter, Inquisitor, Paladin, Ranger, Slayer, Wizard | Arcane Trickster, Dragon Disciple |
| Lore (Religion) | Bard, Cleric, Inquisitor, Monk, Paladin, Wizard | Arcane Trickster, Dragon Disciple, Mystic Theurge |
| Perception | Alchemist, Barbarian, Bard, Druid, Inquisitor, Kineticist, Monk, Ranger, Rogue, Slayer | Aldori Swordlord, Arcane Trickster, Dragon Disciple, Duelist, Eldritch Knight, Mystic Theurge, Stalwart Defender |
| Persuasion | Barbarian, Bard, Cleric, Fighter, Inquisitor, Kineticist, Magus, Monk, Paladin, Ranger, Rogue, Slayer, Sorcerer | Aldori Swordlord, Arcane Trickster, Dragon Disciple, Duelist |
| Use Magic Device | Alchemist, Bard, Inquisitor, Kineticist, Magus, Rogue, Sorcerer | — |

**Against the data:** all 23 covered classes match except two, where the user's table and the guide disagree:

- **Arcane Trickster:** the guide says "Class Skills: All", so the data has Use Magic Device. The user's table doesn't list it.
- **Dragon Disciple:** the user's table has Knowledge (World). The guide's list (Mobility, Knowledge (Arcana), Lore (Nature), Lore (Religion), Perception, Persuasion) doesn't, and neither does the data.

## Replaced by the game's values (2026-11-16)

The user chose the game's values over these facts where the two differ: ticket 261 for the Arcane Trickster's skill points, ticket 262 ("Apply the game's") for the rest. The importer wrote them in tickets 293i and 293l, and `tools/game-check/user-facts.txt` lists each as `applied`. The tables above keep the user's facts as given; the data now holds:

| Entry | Field | Before | Now (the game's) |
| --- | --- | --- | --- |
| Arcane Trickster | skill points | 4 | 3 (261) |
| Defender of the Hearth (fighter) | skill points | the fighter's 2 | 3 |
| Stalwart Defender | class skills | Athletics, Perception | Athletics, Mobility, Perception (its Intimidate waits on ticket 312) |
| Magic Deceiver (arcanist) | class skills | the arcanist's | its own: Knowledge (Arcana), Knowledge (World), Lore (Nature), Lore (Religion), Persuasion, Use Magic Device |
| Divine Hunter (hunter) | class skills | the hunter's | its own: Athletics, Knowledge (Arcana), Lore (Nature), Lore (Religion), Perception, Persuasion, Stealth |
| Spell Dancer (magus) | class skills | the magus's | its own: Knowledge (Arcana), Knowledge (World), Mobility, Persuasion, Use Magic Device |
| Sable Company Marine (ranger) | class skills | the ranger's | its own: Athletics, Knowledge (World), Lore (Nature), Mobility, Perception, Persuasion, Stealth |
| Sylvan Trickster (rogue) | class skills | its own, with Knowledge (World) | its own, without Knowledge (World) |
| Seeker (oracle) | class skills | its own list | the oracle's; its Trickery comes from Seeker's Trapfinding (293i) |
| Wind Whisperer (oracle) | class skills | its own list | the oracle's |
| Witch of the Veil (witch) | class skills | its own list | the witch's; its Stealth and Trickery come from its Secrecy feature (293i) |
| Metamorph (alchemist) | class skills | its own, with Stealth | the alchemist's; Stealth comes from a level-1 feature (293i) |
| Sohei (monk) | class skills | its own, with Lore (Nature) | the monk's; Lore (Nature) comes from a level-1 feature (293i) |
| Empyreal Sorcerer | class skills | its own list | the sorcerer's; Lore (Religion) comes from its bloodline's feature (293i) |
| Sylvan Sorcerer | class skills | its own list | the sorcerer's; Lore (Nature) comes from its bloodline's feature (293i) |

## Expertise (a bonus of half the class level to a skill)

Scaling numbers stay in summaries, so this isn't modelled. Sources the user listed, by skill:

- **Athletics:** Earth or Water Kineticist (feat); Strength domain (cleric, inquisitor, Divine Hunter).
- **Trickery:** Archaeologist Bard, Seeker Oracle, Seeker Sorcerer.
- **Knowledge (Arcana):** most Bard and Skald archetypes, Loremaster, Disciple of the Pike, Demonbane Priest, Divine Scion, Demonslayer, Master of All.
- **Knowledge (World):** most Bard and Skald archetypes, Loremaster, Divine Scion, Master of All, Earth Kineticist (feat).
- **Lore (Nature):** most Bard and Skald archetypes, Shifter (all but Wild Effigy), Loremaster, Beastkin Berserker, Disciple of the Pike, Divine Herbalist, Divine Scion, Master of All, Air, Fire or Water Kineticist (feat).
- **Lore (Religion):** most Bard (Dirge Bard twice) and Skald archetypes, Loremaster, Divine Scion, Master of All.
- **Perception:** Inquisitor (all but Tactical Leader, morale bonus), Rogue (all but Thug and Knife Master), all Slayer archetypes (feat), Archaeologist Bard, Urban Hunter (insight bonus), Espionage Expert.
- **Persuasion:** Inquisitor (all but Tactical Leader: Intimidate; Tactical Leader: Diplomacy), Beastkin Berserker (Intimidate), Chelaxian Diva (Diplomacy, Bluff), Dirge Bard (Intimidate), Fearsome Leader (Intimidate), Court Poet (all), Battle Scion (Intimidate), Inciter (Intimidate, Bluff), Espionage Expert (Bluff), Aldori Swordlord (Intimidate).
- **Use Magic Device:** Scroll Savant Wizard, Magic Deceiver Arcanist (Magic Inventor).

## Backgrounds

The user's full list, with what each background grants, is in [backgrounds-from-user.md](backgrounds-from-user.md). It replaces the per-skill background column the user gave first, which named some backgrounds only by group and spelt a few differently.
