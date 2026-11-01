# Game data

The planner's game data, as RON files that deserialise straight into the `rules` crate types
(`rules/src/game_data.rs`). These files are the source of truth: fix data by editing them, then reseed.

Empty for now: the files are extracted once from the saved guide pages in a later ticket. Until then,
`data/tests/fixtures/game-data/` is a small hand-written example of the layout.

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
- A selection option's key extends its selection's: `selection:rogue/rogue-talents/fast-stealth`.

- Keys are readable text from the guide's anchors: `feat:power-attack`, `class:rogue`,
  `archetype:rogue/dark-lurker`, `selection:rogue/rogue-talents/fast-stealth`.
- `guide` fields are `page#anchor` links into the guide. Store a short summary in our own words, never the
  guide's text.
- Prerequisites are `All([...])`, `Any([...])` and `AtLeast(n, [...])` over single conditions such as
  `Feat("feat:power-attack")` or `AbilityScore(ability: Strength, score: 13)`. A missing `prerequisite`
  means there is none.

## Seeding

```sh
cargo run -p data --bin seed            # this folder (or $GAME_DATA_DIR)
cargo run -p data --bin seed -- <dir>   # another folder, e.g. data/tests/fixtures/game-data
```

The seed first runs `GameData::check` and refuses data with problems, listing them. Then it replaces every game
table in one transaction and stores a hash of the `.ron` files. The server loads the game data once at startup,
so restart it after a reseed. It logs a warning at startup when the stored hash doesn't match the files in
`$GAME_DATA_DIR` (default: this folder); it never reseeds on its own.
