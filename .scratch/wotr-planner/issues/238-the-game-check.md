# The game check: the data against its blueprints

Type: build
Status: open
Blocked by: 236, 237
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14): the game wins on rules, and a check against the game replaces both the spot checks of [146](146-checking-the-data-against-the-guide.md) and the regression checks removed by [235](235-remove-the-regression-checks.md).
- It is code of our own in the repo that reads the community's `.binz`. The user took the recommendation, and went further on the regression checks.
- It runs by hand, since worktrees and CI have no dump.

## Change

A tool in `tools/`, a binary such as `cargo run -p game-check`:
- **Input:** it reads the `.binz` (or its extraction) from `blueprints/` ([236](236-the-game-sources-in-the-repo.md)), and resolves GUID references and the strings file for names.
- **What it compares.** For each entry with a `blueprint` ([237](237-a-blueprint-guid-per-entry.md)):
  - **prerequisites:** the `Prerequisite*` components, with `Group` All or Any; stat values, BAB, feats and features, class levels, caster level, `PrerequisiteNoFeature`, parametrized weapon subcategories, alignment, race;
  - **class tables:** hit die, BAB and save progressions, skill points, class skills;
  - **progression rows:** each level's features and selections, for classes, archetypes (added and removed rows) and prestige classes;
  - **selection option lists:** a selection's options against the blueprint's `m_AllFeatures`;
  - **mythic paths:** each rank's grants;
  - **items:** the always-on bonuses, with their type and value.

  For each area it says what agrees and lists each difference, with the entry's key, the blueprint's name and GUID, and the two values.
- **Output.** A plain-text report grouped by area and kind of difference, largest group first. The exit status is non-zero when anything differs, unless an area is listed as known with its ticket.
- **Deterministic.** The same dump and the same data give a byte-identical report:
  - **Order.** Groups by count, descending, then by area and kind. Within a group, by entry key, then blueprint GUID. Never in a hash map's iteration order.
  - **Content.** No timestamps, timings, absolute paths, thread counts or other varying text in the report. A run's timing, if printed, goes to stderr.
  - **Its input.** The check reads only the pinned dump. It verifies the `.binz`'s SHA-256 against the pin of [236](236-the-game-sources-in-the-repo.md) before reading, and refuses a different file.
- **Words.** It prints numbers, keys, names and GUIDs, never the game's description texts.

`CLAUDE.md` says when to run it: after any data ticket and once per queue, by the main session. It also says that each difference is applied without asking unless it touches one of the user's facts.

## Done when

- The check runs over the whole data and prints its report.
- A test on a small fixture (hand-written blueprint JSON in our own words: no game text) proves each comparison finds a planted difference.
- Two runs over the whole data give byte-identical reports (`cmp`), and so does a test on the fixture, run with entries in a shuffled order.
- A dump whose hash isn't the pinned one is refused before anything is compared.
- Its first run's differences are each covered by a ticket: those already filed from [research/game-source-check-2026-11-14.md](../research/game-source-check-2026-11-14.md), or new ones.
- `scripts/check.sh` passes.
