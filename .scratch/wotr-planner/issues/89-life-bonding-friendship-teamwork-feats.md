# Life-Bonding Friendship gives its two teamwork feats

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). The Azata superpower Life-Bonding Friendship is a plain option in the data (`azata/superpowers/life-bonding-friendship` in `libs/game-data/mythic/paths/azata.ron`): it grants nothing, so a Build that takes it gets no teamwork feat pick.

**Repro:** open the Jack-in-the-Green Build from ticket 88 (`#b=AeWOv0tCURTH77n3vqfW60bRllRrkUNj1BCEFQQVZBGJooOLOjxQBx3cdNHJQXRyEAU3UVBc3N1ERREUfyCI-Efo9YHK2908cL7nez58ORyf7C_3gqls_8XWeTK5D7N3s-nR-Z74m3ZifPVX_77VGZje8OqFyeJN4IEfyaF5jAv3pwHWtLpyn5df5sZDJN3ymD6cXTH03qkVtL82OaMPz-VKfWAxPl9ca3AtPzdWi__O7ug4Wq4mh6VY3m5GAIgASIxJ2pOzlYWtiOp1N4x7QAJifPLCq14LoZhDUEOEVUGeUUG-HgAGgjZCiKJUAZRS5eC2ReUBHWC8BA`), Mythic track. MR 6 holds "Superpowers: Life-Bonding Friendship" and a Mythic feat slot; MR 6 and MR 7 have no other slot. The same holds for the Enforcer Build (LBF at MR 6). Both write-ups take a teamwork feat at each of those ranks: "Life-Bonding Friendship (Shake It Off)" at 6 and "(Allied Spellcaster)" at 7; the Enforcer "(Shield Wall)" at 6 and "(Shake It Off)" at 7.

**The guide** (`azata#life-bonding-friendship`): "You choose a teamwork feat this rank and the next rank." Its "Playing an Azata" adds that LBF is very powerful on a class that receives free teamwork feats.

**Expected:** taking Life-Bonding Friendship gives a teamwork-feat slot (any feat of category Teamwork, prerequisites as for a normal slot unless the guide says otherwise) at the rank it's taken and another at the next rank. Taken at MR 10, the second slot has no rank to land on; say what happens (none, as the game can't give it).

## Build

- A data change in `azata.ron` (the option's rank rows, the shape Combat Trick's `levels` with a `feat_slots` row uses, if a superpower option can carry rank rows; otherwise the smallest rules change that lets it), with a `libs/game-data/README.md` note and a real-data test in `libs/seeds/tests/game_data/`.
- The checks in the repo's `CLAUDE.md` (and `--schema` if the migration changes).

**Done when:** a Build with LBF at MR 6 shows a teamwork-feat slot at MR 6 and MR 7 that offers Shake It Off, Allied Spellcaster and Shield Wall (Shield Wall needs Shield Proficiency) and not a non-teamwork feat; a real-data test pins it; an LBF taken at MR 10 behaves as the ticket decides.
