# What an agent couldn't look up in the third martial run

Type: feature
Status: open
Blocked by: [202](202-the-web-app-says-the-gear-notes-in-the-engines-words.md) (the Offers queue, 199 to 202, edits `apps/mcp`)
Model: Sonnet to build; Opus to review the wording.
Map: [WotR Build Planner](../map.md)

## Problem

Found by the third martial run (2026-11-14; see [212](212-athletics-iii-raises-bab.md) for the run). The MCP agent ran into three gaps that cost it options or a lot of context. The guide-only agent had none of them, since it could grep the pages.

1. **No weapon lookup.** A weapon's dice, critical range, multiplier and proficiency group show up only in an attack line, and only once a Build has Weapon Focus in it or the call names a `weapon`. `search` has no weapon kind, and `kinds: ["gear"]` holds only the ability-bonus items. The agent picked the fauchard from its own knowledge and spent a feat on Exotic Weapon Proficiency for it. The guide-only agent chose the falchion (martial, 18–20) by reading `items#weapons`. `libs/game-data/weapons.ron` already holds these facts.
2. **Path features can't be searched.** `search` has no kind for a Mythic path's own features (Demonic Rage, the aspects, Sneak Attack, Bit of Fun). To find each path feature that adds attack or damage, the agent had to `describe` every path and read it whole. It skipped Angel, Azata, Gold Dragon and Swarm for the cost, so they were never weighed. Class features have the same gap: they are found only through `describe` of their class.
3. **`compare` returns everything.** Three 20-level Builds compared `by` the attack line gave about 25k characters of per-level differences. The agent wanted only the ranking. `sheet` gained `levels` and `ranks` for the same reason ([206](206-sheet-for-some-levels.md), [210](210-sheet-says-each-level-once.md)).

## Build

- **A `weapon` kind** for `search` and `describe`, from `weapons.ron`: name, key, dice, critical range and multiplier, the hands it takes, its proficiency (simple, martial, exotic) and its weapon groups (ticket [155](155-weapon-groups-and-proficiency.md), if built). `category` filters by proficiency or group. No Build choice names a weapon, so it is reference data, as spells are (ticket 121).
- **A `feature` kind** for `search`: class, archetype and path features with their summaries and owner (`parent`), matched by name and by summary words like any other kind ("damage" finds Demonic Rage). `describe` of a feature gives its summary and where it is gained (class level or Mythic rank).
- **`compare` gives only the ranking** when it is asked: a `differences: false` argument (or a name the reviewer prefers) leaves out the per-level differences and keeps `by`'s ranking and each Build's numbers.
- The tool descriptions stay within the instructions' length test. The `wotr-build` skill's Steps 3 and 6 say to use these.

## Done when

`search` with `kinds: ["weapon"]` and `query: "falchion"` returns its dice and 18–20/×2; `search` with `kinds: ["feature"]` and `query: "demonic rage"` returns the Demon's feature; `compare` of three Builds with `differences: false` stays under 2k characters. MCP tests pin each. `scripts/check.sh` passes.
