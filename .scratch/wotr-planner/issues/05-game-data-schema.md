# Game data schema and seeding

Type: grilling
Status: resolved
Blocked by: 01, 04
Map: [WotR Build Planner](../map.md)

## Question

What are the relational Postgres schema and the checked-in data file format for game data, and how do the files get into the database?

- Tables for races, classes, class progression, archetypes, feats, mythic paths, mythic feats and abilities, feature selections and skills.
- **Relational prerequisite trees:** how nested AND/OR prerequisite expressions are stored without JSONB, and how they load into a Rust prerequisite enum.
- **Data file format:** RON, JSON or TOML, and the layout of the files (one per category, or one per entity).
- **Seed pipeline:** re-runnable after migrations, and idempotent or truncate-and-reload.
- **Stable IDs:** how game entities are identified so saved Builds don't break when the data is re-seeded.

## Comments

**2026-10-31, user (round 1):**

- **Q2 = (a):** extract once, then edit the extracted files directly by hand. There is no separate overrides layer and no merge step; the extractor is a one-off. This replaces the "override file" idea from [Filling the guide's gaps](07-guide-gaps.md).
- Not answered, so the recommendations are taken unless the user objects:
  - **Q1:** RON files.
  - **Q3:** readable text keys as primary keys (`feat:power-attack`).
  - **Q4:** one `prerequisite_node` table (each node points to its parent; nodes are all/any/at_least n or a single condition with typed reference columns, enforced by a check).
  - **Q5:** each entity points to the root of its prerequisite tree (`prerequisite_root_id`).
  - **Q6:** full per-level progression rows for every class and archetype, with no table comparison.
  - **Q7:** a `seed` command that deletes and reloads every game table in one transaction, plus a startup warning when the stored data hash is stale.
  - **Q8:** the `rules` types are the single source of truth, plus a round-trip test.

**2026-10-31, user (round 2):**

- **Q9:** no `source` field. Facts aren't marked with where they came from, and facts from the user sit in the data files like any other. This replaces every earlier `source: user` mention (tickets 04 and 07).
- Not answered, so the recommendations are taken unless the user objects:
  - **Q10:** a small extractor crate kept in the repo. Its prerequisite parser is built test-first against short hand-written examples, and it refuses to overwrite existing data files.
  - **Q11:** the RON file layout:
    - `races.ron`, `skills.ron`, `feats.ron`, `progression.ron`
    - `classes/<class>.ron`
    - `prestige/<class>.ron`
    - `mythic/{abilities,feats}.ron`
    - `mythic/paths/<path>.ron`

## Answer

> **Superseded in part, 2026-10-31,** by [Apply the schema and layering review](14-review-fixes.md) and
> [Apply the normalization review](15-normalization-fixes.md). `progression.ron` now holds only the mythic
> ascension table (the Character level at which each Mythic rank is reached); the combat feat categories and the skill rank cap are
> rules in the `rules` crate. In the key examples, `selection:rogue/rogue-talents/<slug>` is a Selection option's
> key; the Feature selection itself is `selection:rogue/rogue-talents`. The rest of this answer is the state when
> 05 closed.

**Data files.** These are RON files under `game-data/`, deserialised straight into the `rules` types:

- `races.ron`, `skills.ron`, `feats.ron`
- `progression.ron`: the mythic rank table, the combat feat categories and the skill rank cap
- `classes/<class>.ron`: one class with its archetypes and feature selections
- `prestige/<class>.ron`
- `mythic/abilities.ron`, `mythic/feats.ron`
- `mythic/paths/<path>.ron`: the path's rank table, its picks and its switch rule

A small extractor crate generates them once from `guide-cache/`. Its prerequisite parser is built test-first, and it refuses to overwrite existing files. After that, fix data by editing the files directly. Facts don't record where they came from.

**Postgres**

- **Keys:** readable text primary keys derived from the guide's anchors, such as `feat:power-attack`, `class:rogue`, `archetype:rogue/dark-lurker` and `selection:rogue/rogue-talents/<slug>`.
- **Progression:** one row per level for every class, archetype and prestige class. Each row holds BAB, Fort/Ref/Will, the highest arcane and divine spell level castable, and what that level grants (features, bonus feat slots with their allowed list, feature-selection slots). An archetype's rows replace the base class's, so no table comparison is needed.
- **Prerequisites:** one `prerequisite_node` table, with each node pointing to its parent. A node is either a group (`all`, `any`, `at_least n`) or a single condition. Conditions use typed columns: references to feats, classes, archetypes, class features or skills, and values for ability score, BAB, spell level with arcane or divine, alignment and race. Each kind of condition must fill exactly its own columns, enforced by a CHECK. Each entity with prerequisites has a nullable `prerequisite_root_id` pointing to the root of its tree.

**Seeding.** `cargo run -p data --bin seed` deletes and reloads every game table in one transaction, and stores a hash of the data files. At startup the server logs a warning when the stored hash doesn't match the files. It never reseeds automatically.

**Types.** The `rules` crate types are the single source of truth. A round-trip test checks that loading the files directly gives the same result as files → Postgres → types.
