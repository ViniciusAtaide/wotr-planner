# Hand-fix the extracted mythic data

Type: task
Status: resolved
Blocked by: 10
Map: [WotR Build Planner](../map.md)

## Question

Stage of the split extraction (user, 2026-11-01). Resolve the `## Mythic` section of `game-data/EXTRACTION_REPORT.md` by editing the RON files directly. AFK.

- **Files you may edit:** `game-data/mythic/**` and `game-data/progression.ron` only. Plus the report's `## Mythic` section.
- **Focus:** Mythic abilities and feats and their prerequisites, each path's rank table and path picks (Aeon gazes, Trickster tiers), path switch rules (kept ranks, level cap, further ascensions) for Legend, Gold Dragon, Swarm That Walks and Devil, and the ascension table 6/7/9/11/14/15/16/17×2/19.

**Shared rules for the hand-fix stages** (these four run in parallel, one agent each):
- **Your area:** fix only this stage's section of `game-data/EXTRACTION_REPORT.md`, and only this stage's files (below). Other agents are editing the other files at the same time.
- **Source:** the saved pages in `guide-cache/`. Never fetch GameFAQs or any site.
- **Copyright:** commit facts only (names, numbers, prerequisites, keys, anchors). A `summary` stays empty unless it's one short line in your own words.
- **Validating:** use the database-free check command from ticket 10's Answer. Never seed, reset or migrate the dev database; the main session seeds once all stages are done. No schema or `rules` type changes. Anything that needs one becomes a proposed follow-up in your Answer.
- **Unclear items:** anything you can't resolve accurately from the guide stays in the report, with a note on why. Don't guess.
- **Done when:** your section of the report holds only unresolvable items, the check command passes, `cargo test --workspace` passes, and there are no commits.
- **Recording:** append an `## Answer` with fixed and remaining counts and the follow-ups, set `Status: resolved`, and add one line at the end of map.md's Decisions so far using Edit. Keep a dated `## Progress log`, because runs can be cut off.

## Progress log

- **2026-11-01:** Read the guide pages locally (mythic-paths, the seven path pages, other-mythic-paths). Per the main session, this stage edits neither `EXTRACTION_REPORT.md` nor `map.md`; the main session merges the report and records the map line from this Answer.
- **2026-11-01:** Fixed the two prerequisite lines, five more prerequisites found wrong in the spot-check, the Aeon gaze picks and the Gold Dragon breath picks. `seed --check-only` and `cargo test --workspace` pass. Every key written was checked by hand against `game-data/`.

## Answer

Report `## Mythic`: **3 of 7 fixed, 4 remain**, plus 5 new items the main session should add to the report. Files touched: `game-data/mythic/abilities.ron`, `mythic/feats.ron`, `mythic/paths/aeon.ron`, `mythic/paths/gold-dragon.ron`. `progression.ron` already matched the user's table (6/7/9/11/14/15/16/17/17/19) and is unchanged.

### Fixed

- `mythic-paths#destiny-beyond-birth` ("Racial attribute penalty"): `Any` of the eight races with a penalty (dhampir, dwarf, elf, gnome, halfling, kitsune, oread, tiefling). Every subrace of these has a penalty too, and no subrace of the other four has one, so race alone is exact.
- `mythic-paths#second-bloodrager-bloodline` ("Bloodrager Bloodline"): `ClassLevel(bloodrager, 1)`. Every bloodrager and archetype gets its bloodline at level 1.
- `other-mythic-paths#legend` ("no rank table"): the guide has none either. Legend is fully its switch rule (below), which matches the prose: only MR 1–2 kept, rank stuck at 3, no further ascensions, cap 40. Its own effects are a new item below.

### Remaining (for the report)

- `gold-dragon` Dragon Feats: three feats without prerequisites. Still three `General` slots: `GameData::check` allows `ignores_prerequisites` only on class rows, and only on a fixed list.
- `gold-dragon` and `other-mythic-paths#swarm-that-walks`: the switch drops the old path's own powers from MR 3–7 but keeps the mythic abilities and feats. Kept as MR 1–7 in full, since the types keep whole ranks. Over-keeping the old path's features is the lesser error; dropping MR 3–7 would lose the general picks that later prerequisites need.
- `other-mythic-paths#devil`: the switch is open only from Aeon or Azata. A path has no prerequisite.
- New, `other-mythic-paths#devil`: switching from Aeon loses the Enforcing Gaze picks (Azata loses the dragon companion, which isn't modelled). MR 1–7 are kept in full.
- New, `other-mythic-paths#legend`: the Legend effects (+4 to every ability score; Corruption Resistance) are never granted. A switch with no further ascensions applies no rank row, so these can't sit in a rank table. The +4 matters for ability-score prerequisites at levels 21–40.
- New, `other-mythic-paths#legend`: feats unlocked by Trickster's Perception II are kept after becoming a Legend. The data drops the MR 3–7 Trickster picks, so `feat:stat-focus`, `feat:metamagic-completely-normal-spell` and the Improved…Critical chain would show as broken.
- New, `gold-dragon#annihilating-breath`, `#decaying-breath` ("Mythic Rank: 9", "Vow of Dahak"), `#enlightening-breath`, `#rejuvenating-breath` ("Mythic Rank: 9", "Uncorrupted"): left out. There's no mythic-rank or Dahak/uncorrupted condition, and MR 8 and 9 share character level 17, so `CharacterLevel` can't stand in.
- New, `mythic-paths#domain-zealot`: a druid has a domain only if Nature Bond took one over the animal companion, and Nature Bond isn't a choice in the data. So `ClassLevel(druid, 1)` lets every druid in.

### Switch rules as encoded

| Path | `kept_ranks` | `further_ascensions` | `level_cap` | Rank after switch | Guide |
| --- | --- | --- | --- | --- | --- |
| Legend | 1, 2 | false | 40 | 3 | loses MR 3–8 abilities and feats, rank stuck at 3, cap 40 |
| Gold Dragon | 1–7 | true | none | 8, 9, 10 | loses MR 3–7 powers, keeps abilities and feats |
| Swarm That Walks | 1–7 | true | none | 8, 9, 10 | same as Gold Dragon |
| Devil | 1–7 | true | none | 8, 9, 10 | from Aeon or Azata only; Aeon loses gazes |

All switch at MR 8 (level 17). On the three paths with further ascensions, MR 8 gives a mythic feat, 9 an ability and 10 a feat, which matches the even/odd rule in code. Their rank tables (8–10) match the guide.

### Spot-check

17 mythic abilities and feats were checked against `mythic-paths`, and a script compared every "Prerequisites:" line on the page with the data: only the two report items were missing. Rank tables: Aeon and Trickster in full, plus the rows of Angel, Azata, Demon, Lich and the three switch paths. Errors found and fixed:

- Bloodline Ascendance and Second Bloodline ("Bloodline") pointed at bloodrager archetype features. The guide says Bloodline Ascendance is for sorcerers and not bloodragers, and bloodragers have their own Second Bloodrager Bloodline. Both are now `ClassLevel(sorcerer, 1)`. A judgment call: Eldritch Scion magi and Dragonheir Scion fighters also have a bloodline, and the guide doesn't say whether they qualify.
- Domain Zealot and Impossible Domain ("Domain") allowed only the Divine Hunter. They are now `Any` of cleric 1, inquisitor 1 and `archetype:hunter/divine-hunter`, plus druid 1 for Domain Zealot only: the druid page says a druid who chose a domain can't take Impossible Domain.
- Throw Elixir ("Infusion", which extends an alchemist's extracts) allowed kineticist infusions. It now needs `selection:alchemist/discovery/infusion` only.
- Aeon: "one Enforcing Gaze at 3rd rank and another at each subsequent rank", but the data had a gaze pick only at MR 3. Now there is one at each of MR 3–10.
- Gold Dragon: Improved Dragon Breath was a feature granted 1/1/3 times. The guide makes it a pick of a breath effect, so it's now `selection:gold-dragon/improved-gold-dragon-breath` (10 options) at MR 8, 9 and 10 ×3.
- Correct: Abundant Bane, Abundant Ki Pool, Kinetic Overcharge, Any Means Necessary, Full Reservoir, Explosives Expert, Mythic Sneak Attack, Stunning Fist (Mythic), Weapon Focus (Mythic), Mythic Channeling, Improved Abundant Casting, Limitless Rage. The Trickster tiers (Mythic Trick at MR 3–10, Improved at 4/6/8/9, Greater at 7/10, each needing the tier below) match too.

### Follow-ups (need type or rules changes)

1. **A feat slot of any feat without prerequisites on a path rank** (Dragon Feats): for example a flag on `General`, allowed on path rank rows.
2. **Partial keeps on a switch:** split `kept_ranks` into general picks (mythic abilities and feats) and path picks, or list dropped selections. This covers Gold Dragon/Swarm (drop the old path's MR 3–7 powers), Devil from Aeon (drop gazes) and Legend from Trickster (keep the Perception II feats).
3. **Previous-path condition on a switch:** for example `PathSwitch.from: BTreeSet<MythicPathKey>` (Devil: Aeon, Azata).
4. **Effects granted on switching** (Legend's +4 to all ability scores), and ability bonuses from mythic features in general (Gold Dragon's Perfect Body/Mind/Soul).
5. **A mythic-rank prerequisite and story-flag conditions** (Vow of Dahak / Uncorrupted). The Aeon gazes' "Rank 5"/"Rank 8" stand in as `CharacterLevel(14/17)`, which happens to be exact.
6. **Druid Nature Bond** (domain or animal companion) as a choice, for the base-classes data and Domain Zealot.
7. **Question for the rules engine:** every path page lists MR 1–2 features (Aeon "including the first two Mythic Hero levels"). The path is chosen at MR 3, so the engine never applies rows 1–2. Decide whether choosing a path grants them retroactively.
