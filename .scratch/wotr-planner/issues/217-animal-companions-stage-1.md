# Animal companions, stage 1: the animal, its archetype, its level and its numbers

Type: feature
Status: open
Blocked by: none
Model: Sonnet to build the data; Opus to build the engine and review.
Map: [WotR Build Planner](../map.md)

## Build

The first stage of [174](174-animal-companions.md)'s answer (its items 1 to 8, 10 and 11); read that Answer first.

- **Data**, from `character-creation#animal-companions-and-familiars` and its following sections (`guide-cache/txt/character-creation.txt`):
  - the 13 animals, each with its size, speed, attacks, six scores, specials and evolved form (the level it grows at: 7, or 4 for horses and leopards);
  - the levelling table: BAB, saves and the Str/Dex bonuses by companion level;
  - the seven companion archetypes with their rows;
  - for each of the 17 granting features and options, the animals it allows ("Horse only", "Dog or Wolf", the Primal Druid's four) and the class level its companion starts at.
  - A note in `libs/game-data/notes/217-animal-companions.md`.
- **The Build:** a choice of animal, and of archetype, at the feature that grants the companion, at that level, like a selection. It is part of the link (a new choice in the codec) and goes when the feature goes.
- **The companion's level:** the granting class's level from where the companion starts. A second source adds its levels to the same companion. Boon Companion adds 4, capped by the character level. Never above 20, Legend included.
- **Its numbers:** six scores, BAB, saves, and an attack line for each natural attack, from the character's attack engine. Not its HP or AC (AC comes with [223](223-armour-class.md)).
- **Rider bonuses** count in the character's numbers as if always riding, whatever the animal's size: the horse's +1 AC, the triceratops's +2 AC. The horse's +1 attack "vs. Medium or smaller" is a conditional row, shown beside the line and not summed into it ([169](169-gear-second-stage.md)'s Answer, item 9). The riding switch is [143](143-switched-on-abilities.md)'s; the size rule is [220](220-a-companion-big-enough-to-ride.md).
- **MCP:** `sheet` gains a `companion` section (animal, archetype, level, numbers, attack lines); `compare` and `improve` take a `by` of the companion's attack line; `options` lists the animals at the granting feature. The `wotr-build` skill's `martial.md` and `dpr.py` gain a companion row.
- **Web app:** the animal and archetype are picked at the granting feature's chip.
- **Gold builds:** Jack-in-the-Green's "Animal Companion - Elk" known gap is deleted; the Elk (Daredevil) is compared.

## Done when

A Sylvan Sorcerer can choose the Elk and the Daredevil archetype at level 1, and the gold test's entry for it is gone. A Druid 5 / Hunter 5 has one companion of level 10, a Boon Companion adds 4 up to the character level, and no companion goes above 20. A horse companion adds +1 AC to its rider and a conditional +1 attack row. `sheet` gives the companion's attack lines, and `compare` ranks two Builds by them. Real-data tests cover each. The snapshot check is regenerated, every changed line following from the new choice. `scripts/check.sh` passes.
