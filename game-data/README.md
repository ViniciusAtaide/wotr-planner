# Game data

The planner's game data, as RON files that deserialise straight into the `rules` crate types
(`rules/src/game_data.rs`). These files are the source of truth: fix data by editing them, then reseed.

The files were extracted once from the saved guide pages (`cargo run -p extract -- --force`, which never
overwrites `.ron` files). `EXTRACTION_REPORT.md` lists, by category, what the extractor couldn't turn into data;
fix an item in the `.ron` files and delete its line. `data/tests/fixtures/game-data/` is a small hand-written
example of the layout.

## Layout

| File | Holds |
| --- | --- |
| `progression.ron` | `Progression`: the mythic rank table (the Character level of each Mythic rank) |
| `races.ron` | `Vec<Race>`, with subraces |
| `skills.ron` | `Vec<Skill>` |
| `feats.ron` | `Vec<Feat>` |
| `classes/<class>.ron` | one base `Class`, with its features, feature selections, level rows and archetypes |
| `prestige/<class>.ron` | one prestige `Class` |
| `mythic/abilities.ron` | `Vec<MythicAbility>` |
| `mythic/feats.ron` | `Vec<MythicFeat>` |
| `mythic/paths/<path>.ron` | one `MythicPath`: its rank table, picks and switch rule (with the character level cap it sets, if any) |

Any other `.ron` file is an error; other files (like this README) are ignored. Order means nothing anywhere
in the data: catalogs are sorted by key, class levels by level, mythic ranks by rank, and the children of
`All`/`Any`/`AtLeast` are a set, so write entries in whatever order reads best.

What follows from a rule is code in the `rules` crate, not data: each mythic rank's general pick (odd ranks a
mythic ability, even ranks a mythic feat), the path choice at MR 3 and the switch at MR 8, which feat categories
are Combat feats (`FeatCategory::is_combat`), the skill rank cap (the Character level), the rank after a path
switch (one above the highest kept rank), and each class's level cap (its number of level rows).

- Level rows run `level: 1`, `2`, … with none missing; what taking a class grants goes in row 1. An archetype
  repeats every row of its base class, and its BAB, saves and spell levels must match the base class's unless
  `data/tests/files.rs` allow-lists the difference.
- Only prestige classes set `advances_arcane`/`advances_divine`, and only base classes have archetypes.
- A selection option's key extends its selection's: `rogue/rogue-talents/fast-stealth`.
- Feat slots are `General`, `CombatFeat`, `Category(Teamwork)` or a fixed list,
  `OneOf(feats: ["crane-style", "dodge"])`. A list that is the class's own bonus-feat list (a monk's
  bonus feats, a ranger's combat style) adds `ignores_prerequisites: true`: the pick skips the feat's
  prerequisites. Only class and archetype level rows may set it; left out, prerequisites are checked.

- Keys are readable text from the guide's anchors: `power-attack` (a feat), `rogue` (a class),
  `rogue/dark-lurker` (an archetype), `rogue/rogue-talents/fast-stealth` (a selection option). A key has no type
  prefix: where it sits says what it names, so keys of two kinds may be equal (the feature and the selection
  `shaman/spirit`). Things a class or path owns start with the owner's key. A key is lowercase letters, digits
  and `-`, in `/`-separated parts (the schema checks this).
- `guide` fields are `page#anchor` links into the guide. Store a short summary in our own words, never the
  guide's text.
- Prerequisites are `All([...])`, `Any([...])` and `AtLeast(n, [...])` over single conditions such as
  `Feat("power-attack")` or `AbilityScore(ability: Strength, score: 13)`. A missing `prerequisite`
  means there is none.

## Seeding

```sh
cargo run -p data --bin seed -- --check-only   # read and check the files only, no database
cargo run -p data --bin seed            # this folder (or $GAME_DATA_DIR)
cargo run -p data --bin seed -- <dir>   # another folder, e.g. data/tests/fixtures/game-data
```

The seed first runs `GameData::check` and refuses data with problems, listing them. `--check-only` stops
there, never touching Postgres, so several editors can check at once; keys that don't exist are only caught by a
real seed (the database's foreign keys). Then it replaces every game
table in one transaction and stores a hash of the `.ron` files. The server loads the game data once at startup,
so restart it after a reseed. It logs a warning at startup when the stored hash doesn't match the files in
`$GAME_DATA_DIR` (default: this folder); it never reseeds on its own.
