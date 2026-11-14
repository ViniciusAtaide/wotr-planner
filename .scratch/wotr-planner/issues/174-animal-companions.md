# Animal companions

Type: design
Status: resolved
Blocked by: 160
Model: no agent. A design ticket: the user decides, in a grilling session with the main session.
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists "Animal Companion - Elk" at Jack-in-the-Green's level 1 as a known gap: the planner grants the Sylvan Sorcerer's companion with no choice of animal (ticket [54](54-mercies-and-enforcer-exploits.md)).

The user, 2026-11-11: "Add animal companions as a subsequent ticket". This reopens the map's Out of scope entry, where companions stay out ("Gear and companions"), and which the equipment plan ([163](163-equipment.md)) had left closed.

Both write-ups also carry a companion build: the animal, its archetype ("Elk (Daredevil)"), and a level-up table of feats, tricks and ability increases (`libs/seeds/tests/game_data/gold/source.txt`).

Questions the plan has to answer, none decided:

- **How far it goes:** only the choice of animal (enough for the gold line), or the companion's own build, level by level, as the write-ups have it?
- **What the choice changes:** does the animal open or close any pick of the character (mounted feats need a mount the character's size can ride)? Does it change any number the planner computes?
- **Where it lives:** part of the Build and its link, as gear is; on a tab of its own?
- **The data:** which saved guide pages list the animals, their archetypes and their feats, and what is missing.
- **The gold builds:** once companions exist, is the companion's table compared too?
- **Who gets one:** every class and archetype with an animal companion, a mythic path's companion (the Azata's dragon), a mount.

**Found again by the third martial run** (2026-11-14; see [212](212-athletics-iii-raises-bab.md)). The brief counted a pet's damage toward the total. The MCP agent found that a companion-granting archetype (Mad Dog) grants "Animal Companion" with no species, stats, attacks or companion level, so it couldn't price a pet against a pure fighter. It dropped pets on its own judgement. The guide-only agent priced the Azata's Aivu at about 100–150 damage per round from `azata#aivu`. For a martial goal, the question "What the choice changes" includes the companion's own attack line.

## Answer

**Decided** (user, 2026-11-14, in a grilling session; recommendations taken but where noted):

1. **How far:** the companion's whole build, in two stages. Stage 1: the animal, its companion archetype, its level and its numbers. Stage 2: its level-up picks (feats at odd levels, 2 skill points a level, ability increases), as the write-ups' companion tables have them. The guide holds all of it (`character-creation#animal-companions-and-familiars` and its following sections: 13 animals with their evolved forms, the levelling table, seven archetypes).
2. **Who:** every source of an animal companion the data has (17 features and options), each limited to the animals the guide allows it, the paladin's Divine Bond horse and the Bloodrider's horse included. The mythic paths' companions (Aivu, the Lich's Skeletal Champion) are their own ticket (user: "for aivu and other pets like the lich skeleton create tickets for it").
3. **Where:** a choice at the feature that grants the companion, at that level, like a selection: part of the Build and its link, gone with the feature.
4. **The companion's level** (user's own answer): "when you already have a companion the other class just continues the level of the existing companion up until 20". A second source adds its levels to the one companion. A companion's level never goes above 20, Legend included. Boon Companion's +4 is capped by both the character level and 20.
5. **Numbers in stage 1:** its six scores, BAB, saves and an attack line for each natural attack (from the character's attack engine, so `compare` and `improve` can rank it). Not its HP or AC (169 decides armour class).
6. **Rider bonuses** (user's own answer: "b but assume you're always riding for now. the switch will come in 143"): the animal's rider bonuses (the horse's +1 AC and +1 attack, the triceratops's +2 AC) count in the character's numbers, as if always riding. Ticket [143](143-switched-on-abilities.md) adds the switch.
7. **Gold builds:** the animal and archetype are compared in stage 1, the companion's level-up table in stage 2.
8. **In the MCP:** `sheet` gains a `companion` section (animal, archetype, level, numbers, attack lines); `compare` and `improve` take a `by` of the companion's attack line. A total damage per round stays the `wotr-build` skill's worksheet, which gains a companion row.
10. **The horse's +1 attack against Medium or smaller foes** (user, against the recommendation): counted in the attack line, with a row naming its condition ("vs. Medium or smaller").
11. **Riding size** (user: "b. since this feature will be very complex depending on the size buff we'll deal with it in a subsequent ticket"): a rider bonus counts whenever the Build has that animal, whatever its size against the character's. Whether the animal can carry the character (its evolved form, size buffs) is a ticket of its own.
9. **Stage 2 in the web app:** the companion's level-up shows in its own panel, opened from the granting feature's chip, one row per companion level; its addresses are `companion.level.N.*`.

## Done when

The user has decided each question, the decisions are in the Answer and on the map (its Out of scope entry rewritten), and the build is ticketed.

Built by: [217](217-animal-companions-stage-1.md) stage 1, then [218](218-animal-companions-stage-2.md) stage 2. Follow-ups (design): [219](219-mythic-path-companions.md) the mythic paths' companions, [220](220-a-companion-big-enough-to-ride.md) a companion big enough to ride.
