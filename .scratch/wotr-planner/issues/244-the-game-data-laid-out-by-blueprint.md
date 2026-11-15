# The game data laid out by blueprint

Type: design (decided), then build
Status: decided; built after the queue of 235–242 and its difference tickets (246 on), before 245
Blocked by: 237, [243, the rank and context splits](243-ranked-features-and-filtered-pools.md)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Question

Mapping each entry to its blueprint ([237](237-a-blueprint-guid-per-entry.md)) showed the planner's data cut the game's objects in ways the game doesn't:
- **Archetype and class copies:** Sneak Attack is 13 entries, one `SneakAttack`.
- **Rank splits:** `oracle/curse-2/3/4`, one `OracleCurse`.
- **Context splits:** per-mystery revelation pools, one `OracleRevelationSelection`.
- **Entries merging several blueprints:** the domains (Primary, Secondary, Separatist), Blessing and Second Blessing, Aeon Gaze tiers.
- **About 336 planner-made entries:** no blueprint behind them.

237 first planned a list of allowed sharings. The user: "creating a blueprint-sharings.ron still makes the old architecture leak into the new", and "why don't we create one file per blueprint in a blueprint folder?" Grilled on 2026-11-15.

**Facts found** (2026-11-15, read in the code at 0d2c2d8):
- **Files:** 160 `.ron` files; the folder decides the kind (`libs/seeds/src/files.rs:104-144`, a whitelist).
- **Keys:** a feature or selection key must start with its owner's key (`DataIssue::KeyOutsideItsOwner`, `check.rs:203`). The sheet's labels parse key prefixes for the holder (`engine/labels.rs:218-238`).
- **Lookups:** the engine finds entries by global key indexes (`engine/catalog.rs`), not by walking the owning class.
- **Cross-class references:** 78 progression entries already take another class's selection (skald and bard take `rogue/rogue-talents`). No feature is borrowed across classes; each class defines its own copy.
- **Links:** they name races, subraces, backgrounds, deities, skills, weapons, classes, archetypes, paths, feats, selection options, mythic abilities, mythic feats and gear, never features or selections (`codec.rs:15-27`).
- **The browser** receives the serialised `GameData` and checks it again (`apps/backend/src/game_data.rs`, `libs/ui/src/game_data.rs`).

## Answer

**Decided** (user, 2026-11-15). Round 1 was put to the user with recommendations; the user: "do everything as planned", so every recommendation below was taken. Each is open to revisit before the build starts.

1. **One file per blueprint, for every kind with a blueprint:** classes, archetypes, features, selections, feats, mythic paths, abilities and feats, races, subraces, backgrounds, deities, items, spells, spell lists and weapons. That is about 3,800 files, in folders by kind under `libs/game-data/`. Two data tickets then rarely touch one file.
2. **The model follows the data.**
   - `GameData` becomes flat catalogs keyed by key.
   - Classes, archetypes, paths and progression rows refer to features and selections by key, with no owned copies.
   - The engine's indexes already work this way. The structs, the loader, `GameData::check`, the MCP server's and the web app's walks of a class's own lists, and the browser's JSON (smaller without the copies) change with it.
3. **Order:** 237 (the GUIDs), then the rank and context splits ticket (the engine learns ranked features, repeated selections and filtered pools), then this layout. Each lands one shape at a time.
4. **Timing:** after the queue of 235–242. 238 reads through `GameData` and the GUIDs, so it needs little rework.
5. **Planner-made entries** go in a `planner/` folder of the same per-entry form. Each file says why no blueprint stands behind it. This ticket questions each one first: a placeholder that is really a rule becomes the rule. The 91 bloodline-spell placeholders, for instance, are "the held bloodline's spell at this level".
6. **File names and keys:**
   - Each file is named by its key: `features/sneak-attack.ron`. Its blueprint is inside.
   - A collapsed feature or selection takes one key without an owner (`sneak-attack`). Two game objects of one name are told apart by the game's own distinction (`sneak-attack`, `sneak-attack-vivisectionist`, if the game has two).
   - Keys that links name never change. A collapsed linked entry keeps the old keys as `aliases`, which only the link decoder reads.
7. **Owners:**
   - `KeyOutsideItsOwner` goes.
   - The sheet's holder comes from the row or selection that grants the entry, not from the key's prefix.
   - A feature granted by two classes is one entry with two holders.
8. **Selection options are features.** As in the game (a selection's `m_AllFeatures`), an option is a feature entry that the selection lists. The link codec's option kind looks its key up among the features. An option's level rows stay with the feature.
9. **Archetypes** are files of their own. Each one's added and removed rows refer to its class's features, as the game's `AddFeatures` and `RemoveFeatures` do.
10. **Entries merging several blueprints split** back into the game's objects: the Primary, Secondary and Separatist domain selections, Blessing and Second Blessing, the Aeon Gaze tiers. The web app may show them together; the data holds them as the game does.
11. **What an entry file holds:** the entry (key, blueprint, name, summary, guide link, prerequisites and its own fields). Progression rows stay in their class's or archetype's file. Data notes stay in `notes/`, one per ticket.
12. **The move is mechanical and proved.**
    - A script, kept outside the repo, writes the new layout from the current files and 237's mapping.
    - The proof is the corpus: its 2,163 Builds' names and `Debug` hash before and after, the way 235 proved its move. It is checked together with the self-checks, the gold builds and every real-data test, with no expected value changed.
    - Link compatibility: every bench and corpus link decodes to the same Build.
13. **One build ticket.** The structs, the loader, the check, the data move and the readers land together, since a half-moved layout doesn't load.

## Done when (for the build)

- `libs/game-data/` holds one file per blueprint in folders by kind, plus `planner/`, and no entry is a copy of another.
- The game check's findings listed under 244 in `tools/game-check/known.txt` are gone, and their lines with them (2026-11-15: 323 at first, then 309 more assigned when tickets 247–291 were filed). Among them are the game's per-selection copies that the planner shares: the domain options' Secondary and Separatist exclusions, the revelations' Divine Herbalist and Enlightened Philosopher mystery prerequisites, the druid and Blight Druid domain lists, the archetype mystery selections, `shaman/hex`, and the cleric's secondary-domain rows. Splitting an entry keeps each copy's own prerequisites, so no behaviour moves.
- The corpus hash, the self-checks, the gold builds and every real-data test agree before and after, with every link still opening.
- `GameData::check`, the MCP server and the web app read the new model; `scripts/check.sh --full` passes.
- `CLAUDE.md`'s "Game data" and `libs/game-data/README.md` describe the layout.
