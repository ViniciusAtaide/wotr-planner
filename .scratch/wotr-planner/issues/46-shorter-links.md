# Shorter links

Type: task
Status: resolved
Blocked by: 45
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02:** the link grows linearly as options are chosen; make it grow much less. Measured on 2026-11-02 (`scratchpad/linksize/`: every base class to 20, every slot filled with its first open option, two skill ranks a level, a background and a path): today's links are **630–850 characters**. The body is postcard with every key as text, raw DEFLATE, base64url. Unique keys (feats, options) barely repeat, so DEFLATE can't shrink them: they're most of the link. Estimates from the same Builds:

| Encoding of each key | Link |
|---|---|
| Text (today) | 630–850 |
| A 3-byte hash of the key, per kind, then DEFLATE | 330–430 |
| A 2-byte index into the game data's list | ~250–330, but any data change shifts indices and breaks or silently changes old links |
| A preset DEFLATE dictionary of the keys | ~10% shorter only |

**Decision (user, 2026-11-02): hash each key.** The link stores a short hash of each key instead of its text; opening a link looks each hash up in the current game data. Links still survive game-data changes that keep the keys, as today. A hash of the whole Build is not an option: it can't be reversed without storing Builds on a server, which the map rules out.

## Build

AFK, test-first (`tdd`), in `rules` (`codec.rs` and whatever `open_link` needs); no schema or game-data change. No link has shipped, so replace version 1's body rather than adding a version 2, and regenerate the frozen `V1_LINK` with a comment saying why.

- **Key hashes.** Each key is written as a fixed-width hash of its text, per kind (feats, options, classes, …): a specified, stable hash (for example FNV-1a 32-bit, truncated), documented in the module docs. Choose the width so that no two keys of one kind collide in the real data, with a real-data test that fails if a future data change makes two collide (then widen it or change the seed). 24 bits is the starting estimate.
- **Decoding needs the game data** to turn hashes back into keys, so the link boundary changes: `Engine::open_link` decodes against its data. A hash no key of its kind has is a stale link (the one generic error). Keep a pure round-trip (encode → decode with the same data) for the tests and proptests.
- **Structural trims, measured:** try each and keep those that shorten links without making the codec fragile:
  - a run of levels of one class stored once (Fighter × 20) rather than the class at every level;
  - picks without their full `SlotId` (for example level-relative or in the Sheet's slot order), only if decoding stays simple and exact;
  - anything else the measurement shows is large.
- **Measure** with the `scratchpad/linksize/` Builds (copy and extend it to `t46/`), before and after, and put the table in the Answer.
- **Docs:** the codec module docs, ticket 13's encoding paragraph, `GLOSSARY.md` if it describes the link, a map "Decisions so far" line.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; the measured links are at most half today's length; a real-data test shows no hash collisions within any kind; round-trip tests and the proptest pass; a link naming a key the data no longer has opens as a stale link; `V1_LINK` is regenerated.

## Progress log

- 2026-11-02: started. Baseline: HEAD 86ffafa (`server/src/main.rs` has the user's uncommitted edit, left alone). Scratch in `t46/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t46/`): `linksize/` is the lead's measurement, copied and extended; `old-src/` is `git archive HEAD`.
- 2026-11-02: measured candidates before building (`t46/measure-candidates.txt`, `cargo run --release -- candidates`): every key as a 3-byte FNV-1a hash in place of its text, then each structural trim alone and on top of the others, on all 26 base classes (monk and paladin lawful good; the lead's run had skipped them). Means over the 26, in characters of `#b=…`: before 759; hashes only 391; picks grouped by kind of slot with levels as steps 347; the hashes listed apart from the structure 383; both 320. On top of both: runs of one class 324, an ability-increase byte only at every 4th level 321, skill ranks per skill rather than per level 319, 2-byte hashes for the small kinds 308, no pick-kind byte 317, no byte for an absent parameter 319. 4-byte hashes and no trims: 449. Without DEFLATE: 772. An upper bound for dropping slot ids altogether (picks in the Sheet's slot order, without even a map of filled slots) was about 36 characters (11%) under groups + hashes apart, on the lead's 24 classes.
- 2026-11-02: tests first.
  - `rules/tests/engine/codec.rs`:
    - The proptest reads each Build back with `LinkKeys::of_build` (the keys it names). It skips the rare Build whose random keys of one kind share a hash.
    - A link naming a key the game data lacks doesn't decode (`UnknownKey`): a feat, class, skill, background, weapon value or switch.
    - A key of another kind doesn't stand in for a missing one.
    - A key's length doesn't change the link's length.
    - The hash is FNV-1a cut to 24 bits (checked against FNV's published vectors).
    - Two keys of one kind with one hash are a `KeyCollision`, and a link naming either doesn't decode. Keys of two kinds may share one.
    - The 40-level fixture Build's link is at most half its old 604 characters.
  - `rules/src/codec.rs` unit tests: the frozen body layout, the point buy, and bodies the encoder never writes (a slot twice, two groups of one kind, an empty group, bytes or hashes left over, a hash missing).
  - `data/tests/game_data.rs`: `no_two_keys_of_one_kind_share_a_link_hash`, and `a_full_build_of_every_base_class_opens_from_a_short_link` (the measurement's 26 Builds open through `Engine::open_link` unchanged, each link at most 400 characters).
  - Red: `t46/red-compile.txt` (27 errors: no `LinkKeys`, `KeyKind`, `KeyCollision`, `key_hash`, `UnknownKey`, `decode_build` with keys or `body`). `t46/red-run.txt`, with the new codec built but `V1_LINK` still empty: the two V1 tests fail.
- 2026-11-02: built.
  - `rules`' codec writes version 1's new body by hand (`Writer`, `Reader`) instead of through postcard, and the postcard dependency is gone.
  - New: `LinkKeys`, `KeyKind`, `KeyCollision`, `key_hash`, `KEY_HASH_BITS` and `DecodeError::UnknownKey`. `decode_build` takes the keys.
  - `Engine::new` builds the game data's `LinkKeys` once, and `Engine::open_link` reads links with them.
  - `V1_LINK` regenerated (314 characters before, 196 after). The module docs are the format's spec.
  - Docs updated: `build.rs`'s module docs, ticket 13's encoding paragraph, `GLOSSARY.md`'s Build entry.
  - Green: rules 314 + unit 4, data `game_data` 143.
- 2026-11-02: measured after (`t46/measure-final.txt`, `cargo run --release`), then ran the Verification checks and the browser check (below).

## Answer

Done 2026-11-02, uncommitted. A link is now 0.39 to 0.45 of its old length: 269–367 characters instead of 629–894 for every base class to 20 with every slot filled. Each key is a 24-bit hash of its text, looked up in the current game data by kind when the link opens. Two structural trims were kept: picks grouped by kind of slot with levels as steps, and the hashes listed apart from the structure. Version 1's body was replaced (no link has shipped), and `V1_LINK` was regenerated.

### Measured

The Builds: every base class to 20, every slot filled with its first open option until nothing more opens, two skill ranks a level (the first two skills), Strength at every 4th level, human, Mercenary, Angel; true neutral, or lawful good for the monk and the paladin. Lengths are characters of the fragment `#b=…`. Before is version 1 as at 86ffafa (postcard, DEFLATE level 9); after is `encode_build` now. The program is `t46/linksize/` (`cargo run --release`). Every after link also opens through `Engine::open_link` unchanged.

| Class | Picks | Before | After | After / before |
|---|---|---|---|---|
| alchemist | 38 | 831 | 326 | 0.39 |
| arcanist | 39 | 837 | 341 | 0.41 |
| barbarian | 38 | 814 | 329 | 0.40 |
| bard | 28 | 629 | 269 | 0.43 |
| bloodrager | 34 | 747 | 323 | 0.43 |
| cavalier | 35 | 783 | 326 | 0.42 |
| cleric | 30 | 657 | 282 | 0.43 |
| druid | 29 | 658 | 278 | 0.42 |
| fighter | 39 | 799 | 338 | 0.42 |
| hunter | 35 | 729 | 330 | 0.45 |
| inquisitor | 36 | 777 | 335 | 0.43 |
| kineticist | 42 | 894 | 367 | 0.41 |
| magus | 37 | 802 | 337 | 0.42 |
| monk | 34 | 733 | 318 | 0.43 |
| oracle | 36 | 801 | 331 | 0.41 |
| paladin | 28 | 641 | 269 | 0.42 |
| ranger | 34 | 751 | 337 | 0.45 |
| rogue | 38 | 845 | 330 | 0.39 |
| shaman | 37 | 782 | 343 | 0.44 |
| shifter | 33 | 711 | 309 | 0.43 |
| skald | 29 | 633 | 275 | 0.43 |
| slayer | 38 | 825 | 331 | 0.40 |
| sorcerer | 33 | 751 | 305 | 0.41 |
| warpriest | 37 | 783 | 345 | 0.44 |
| witch | 40 | 829 | 346 | 0.42 |
| wizard | 34 | 715 | 323 | 0.45 |
| **mean** | | 759 | 320 | 0.42 |
| **range** | | 629-894 | 269-367 | 0.39-0.45 |

The 40-level fixture Build in `codec.rs`: 604 → 252 characters. The frozen `V1_LINK`: 314 → 196.

### The format

The module docs of `rules/src/codec.rs` are the spec. In short:

- **Link:** base64url (no padding) over a format-version byte (1), then the body compressed with raw DEFLATE.
- **Body:** a varint count, that many 3-byte key hashes, then the structure. The list holds one hash per key the structure names, in the order it names them. A key named twice is listed twice, and DEFLATE shrinks the repeats.
- **Hash:** FNV-1a 32-bit over the key's UTF-8 bytes (offset basis `0x811c9dc5`, prime `0x01000193`), cut to its low 24 bits, least significant byte first (`rules::key_hash`).
- **Per-kind namespaces:** the structure says each key's kind, and a hash is looked up only among the game data's keys of that kind (`LinkKeys`, `KeyKind`). The kinds are races, subraces, backgrounds, skills, weapons, classes (base and prestige together), archetypes, mythic paths, feats, selection options, mythic abilities and mythic feats. Keys of two kinds may share a hash; two keys of one kind may not. The real data has no such pair (the largest kind is 747 selection options).
- **Structure:** in this order:
  - the name (a tag, then its length and UTF-8 bytes);
  - the Build start (race, subrace, racial ability, alignment, background, six scores);
  - the levels (class, ability increase, skill ranks);
  - the class choices (class, archetype, advances);
  - the path, then the switch;
  - the picks. Each pick names its kind, its key and its parameter: none, a skill or weapon key, or a school, energy or armour-type index.
- **Picks:** grouped by `SlotId` variant (0 Feat … 7 OptionFeat), in that order. Each group gives its variant and a count. Each pick in it then gives a one-byte step (its level or rank less the previous pick's in the group), the slot's other fields, and the pick. The other fields are a Feat slot's source and index, nothing for a Mythic slot, and the index for the rest.
- **Exact:** `decode(encode(b)) == b` given the keys `b` names, and every Build has one body. Decoding refuses (`Format`) whatever the encoder never writes:
  - an unknown tag;
  - slots out of order or named twice, two groups of one kind, an empty group;
  - a map or set out of key order or naming a key twice;
  - a varint in more bytes than it needs;
  - a listed hash left unused, or bytes left over;
  - a Build that breaks the fixed rules, as before (point buy, ranks above the level, a misplaced increase).
- **Stale link:** a hash no key of its kind has, or one that two keys of its kind share, is `DecodeError::UnknownKey`. `Engine::open_link` turns it, like every other failure, into the one generic `StaleLink`.

### Trims kept and dropped

- **Kept: picks grouped by kind of slot, with levels as steps** (−44 on average against hashes alone). The variant byte appears once per group, and a pick's level or rank shrinks to a small step. Decoding stays exact and needs no game data: the steps rebuild each `SlotId`.
- **Kept: the hashes listed apart from the structure** (−27 more on top of the groups). Random hash bytes no longer break up the structure's small, repetitive bytes, so DEFLATE codes the structure better. The cost is one count at the front, and the reader takes the structure and the hashes in step.
- **Dropped: a run of levels of one class stored once** (+4). DEFLATE already turns the repeated class hash into back-references, and the run counts cost more than they save.
- **Dropped: picks without their `SlotId`, in the Sheet's slot order.** Dropping slot ids altogether, with no map of which slots are filled, would save at most about 36 characters (11%). Decoding would then need the engine's walk, because picks open slots. And old links would silently change meaning whenever slot order changed in the rules or the data, which is the fragility the ticket rules out.
- **Dropped after measuring:** an ability-increase byte only at every 4th level (+1), skill ranks per skill (−1), 2-byte hashes for the small kinds (−12, but a collision risk for 39 classes and 73 weapons), no pick-kind byte (−3), and no byte for an absent parameter (−1). Each saves too little to justify a less obvious layout or a weaker guarantee.

### API

- `rules::encode_build(&Build) -> String`: unchanged, and it needs no game data (it hashes the key text).
- `rules::decode_build(code, &LinkKeys) -> Result<Build, DecodeError>` now takes the keys:
  - `LinkKeys::new(&GameData)` gives the game data's keys.
  - `LinkKeys::of_build(&Build)` gives the keys one Build names, for a pure round trip.
  - `LinkKeys::collisions()` lists any `KeyCollision { kind, keys }`.
- `Engine::new` builds the game data's `LinkKeys` once, and `Engine::open_link` reads links with them. The app doesn't change: it writes links with `encode_build` and opens them through `open_link`.
- `DecodeError::UnknownKey`, `key_hash`, `KEY_HASH_BITS` and `KeyKind` are new.
- `postcard` is no longer a `rules` dependency. `Build` still derives serde.

### Decisions

1. **A collision makes a hash unreadable rather than first-come.** If two keys of one kind ever share a hash, `LinkKeys` reports the pair, and a link naming either fails to decode as a stale link. A link never silently opens with the wrong pick. The real-data test fails first, and the fix is a wider hash or another offset basis: before links ship that edits version 1, after it needs a version 2.
2. **Every key kind is hashed, the small ones too** (races, skills, classes, paths). One width and one rule keep the spec simple, and their repeats compress anyway. 2-byte hashes for them would save about 12 characters.
3. **The encoder destructures `Build`, `BuildStart`, `LevelUp` and `ClassChoice` field by field,** so a new field fails to compile until the codec writes it. Before, postcard followed `Build`'s serde shape, and a new field changed the format silently.
4. **One body per Build:** decoding refuses every non-canonical body (listed above). Nothing needs this today, but it makes the frozen vector and the round trip a complete statement of the format.
5. **The name stays text.** It's the player's own, and empty by default.

### Docs

- The codec's module docs (the format's spec) and `build.rs`'s module docs.
- Ticket 13's encoding paragraph.
- `GLOSSARY.md`'s **Build**: the link holds a short hash of each key.
- The map's Decisions line.

### Verification

- `cargo test --workspace`: pass (rules 314 + unit 4, data files 25 + `game_data` 143 + postgres 25, extract 90). Before: rules 310 + unit 2, `game_data` 141. The proptest also passed 20,000 cases in release (`PROPTEST_CASES=20000`).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass.
- No schema or game-data change, so the dev DB wasn't reset. `--check-only` and the real seed (`wotr-planner-postgres` on 5434, same data) both give 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 38 backgrounds.
- The acceptance grep finds nothing, also over `extract` and `data/src`.
- A read-only sample:
  - `feat` starts with `ability-focus-bombs`, `accomplished-sneak-attacker`.
  - `selection_option` starts with `aeon/enforcing-gaze/enforcing-gaze-area-effects`.
  - No `feat`, `feature`, `selection_option` or `class` key contains `:`.
- Browser check, headless (`t46/smoke.js`, output in `t46/smoke.txt`), against `cargo leptos watch` on 3900, stopped afterwards. All 14 checks pass:
  - A blank Build has no fragment.
  - Adding a Fighter level, adding a second, and picking Alertness each rewrite `#b=`.
  - Reloading keeps the fragment and restores the same 2-level sheet.
  - The measurement's full fighter 20 (a 335-character code) opens all 20 levels, and its link stays as it is.
  - `#b=garbage`, and the same fighter link with a feat the data lacks, each show "This link no longer works with the current game data" and a blank Build.
