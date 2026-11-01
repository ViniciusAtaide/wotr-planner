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
switch (one above the highest kept rank), each class's level cap (its number of level rows; an archetype has its
class's), and BAB and saves (from the class's progressions).

- A class's level rows run `level: 1`, `2`, … with none missing; what taking a class grants goes in row 1. Rows
  hold no BAB or saves: the class names its progressions (`bab: Full`/`ThreeQuarters`/`Half`, and `fort`, `reflex`,
  `will`: `Good`/`Poor`) and the rules compute the numbers, with the prestige save formula for prestige classes. A
  save cell the guide prints differently goes in `save_exceptions: [(level: 20, save: Will, value: 20)]`, exactly as
  printed; the check refuses one the progression gives anyway.
- An archetype has a row only at the levels where it differs from its base class. A row's `features`, `feats`,
  `feat_slots` and `selections` replace the base row's in full (a row with none takes them away), and
  `spell_levels: (arcane: 1, divine: 0)` replaces the base row's spell levels when set. A level without a row is
  the base class's. The check refuses a row that changes nothing and spell levels equal to the base row's.
- Only prestige classes set `advances_arcane`/`advances_divine`, and only base classes have archetypes.
- A selection option's key extends its selection's: `rogue/rogue-talents/fast-stealth`. A feature's, a
  selection's and a feat list's key starts with its owner's (the archetype's when an archetype defines it), and
  the schema checks it: Hard to Kill is each path's own `aeon/hard-to-kill`, not one shared key.
- An option is defined once, in the selection whose list it is, and another selection that offers it lists its
  key in `shared_options: [...]` after its own `options` (one key per line): the slayer's talents share the 36
  rogue talents, the shaman's hexes the witch's 11 shared hexes, the Sylvan Trickster's Fey Tricks the rogue
  talents and every witch hex, and the Underground Chemist's Discovery (her rogue-talent slots from 10th level)
  the rogue talents and six alchemist discoveries. Never copy an option: a copy is a second key, so a character
  could take it twice. A shared option's prerequisite is the same in every selection, so where it names the
  defining class's feature or level it accepts each sharing class's with an `Any`
  (`Any([Feature(feature: "rogue/advanced-talents", count: 1), Feature(feature: "slayer/advanced-talents", count: 1)])`).
  A selection may share no option of its own (the schema checks it), and one made only of shared options has
  `options: []`.
- Mythic abilities and feats name their group as the guide heads it: `group: Melee`, `ArmourAndShield`,
  `CombatManoeuvre` and so on.
- Feat slots are `General`, `CombatFeat`, `Category(Teamwork)` or a named list, `OneOf(list: "monk/bonus-feats-1")`.
  The lists are in the file of the class, archetype or path that owns them, in a `feat_lists: [...]` block before
  its `levels`/`ranks`: `(key: "monk/bonus-feats-1", name: "Monk Bonus Feats", guide: "monk#monk-bonus-feats",
  ignores_prerequisites: true, feats: ["crane-style", "dodge"])`. The key starts with the owner's key; any row may
  name any list. A list that grows with level is one list per stage. A list that is the class's own bonus-feat
  list (a monk's bonus feats, a ranger's combat style) sets `ignores_prerequisites: true`: a pick from it skips
  the feat's prerequisites. Only class and archetype rows may name such a list; left out, prerequisites are
  checked. The check refuses a list of fewer than two feats and one no slot names.
- A feat the class gives outright is a granted feat, not a slot: `feats: ["scribe-scrolls"]` on the level row
  (class or archetype). It's held with no pick, whatever its prerequisites, and it's in no Build or link. Classes
  grant their Armor, Shield and Weapon Proficiency feats this way, from each class page's proficiency text; a
  `<class>-proficiencies` feature stays only where it says more than those feats (named weapons, bombs, casting
  in armour, or a prerequisite names it).

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
