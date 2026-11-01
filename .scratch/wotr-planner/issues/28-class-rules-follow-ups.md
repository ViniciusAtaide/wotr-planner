# Class rules from the hand-fix follow-ups

Type: task
Status: resolved
Blocked by: 27
Map: [WotR Build Planner](../map.md)

## Question

**User decisions (2026-11-01, [ticket 21](21-schema-questions-real-data.md) R5 and R6)**, plus ticket 21's implementation follow-ups 8 and 11:

- **Prestige levels counting as another class:** a `counts_as (class, as_class, for)` fact, where `for` is feat prerequisites or a feature's progression. Covers Diverse Training (Eldritch Knight levels as fighter levels for feats) and Frost Power (Winter Witch stacks with witch/shaman for hexes). Dragon Disciple's bloodline continuation waits on [ticket 29](29-design-option-level-grants.md).
- **Advances one class, arcane or divine** (Signifer, Loremaster, Winter Witch): a per-class `advances: Either | Each` that replaces the always-true flags on these three.
- **Story conditions** (Vow of Dahak, Uncorrupted): out of scope. The option is allowed, and the sheet shows a note. Choose the smallest shape (for example a note on the prerequisite that never blocks).
- **Crusader:** keeps skipping all prerequisites (no change; note it in the data).
- **Assassin:** "neutral or evil" means true neutral or any evil alignment; neutral good is excluded. Fix the data and add a test.
- **Druid Nature Bond:** a `druid/nature-bond` selection (animal companion or domain) at L1, with Domain Zealot requiring the domain option. The domain's own powers wait on ticket 29.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus a test per bullet.

## Progress log

- 2026-11-01: baseline in the session scratchpad `t28/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t28/`): `old-src/` is `git archive HEAD` (d7492b3); `extract-old/` is the HEAD extractor's output (identical to `t27/extract-new/`). Guide read offline (`t24/pages/*.txt`, made from `guide-cache/`). Shape decided: `Class.advances: Advancement` (`Each`, the default: one class per spell type its rows advance; `Either`: one class, arcane or divine) on Signifer, Loremaster and Winter Witch, the rows keeping their flags as the spell types a level can advance; `Class.counts_as: BTreeSet<CountsAs { class, for: CountsFor }>` with `CountsFor::FeatPrerequisites` (Eldritch Knight → fighter: `ClassLevel` in a feat's prerequisite counts both) and `CountsFor::Feature(key)` (Winter Witch → witch for `witch/major-hexes` and `witch/grand-hexes`: the witch's rows that grant it are reached at witch + Winter Witch levels); `Prerequisite::Story(Story)` (`VowOfDahak`, `Uncorrupted`) that always holds, with `Engine::story_conditions(pick)` for the sheet's note; Assassin's alignment TN or any evil (the extractor's grammar: next to another part, a lone "neutral" is true neutral); `druid/nature-bond` (Animal Companion or Domain) at L1 for the druid, Defender of the True World and Feyspeaker, replacing their L1 companion and domain features, with Domain Zealot needing the domain option (or Blight Druid / Winter Child, whose domain is fixed) and Boon Companion / Mythical Beast accepting the companion option; Crusader: a RON comment on its bonus lists. No `Build` change, so no new link vector. Next: rules, tests first.
EOF # ide-unavailable- 2026-11-01: `rules` done. `Advancement` (`Each` default, `Either`) on `Class.advances`; `CountsAs { class, toward: CountsToward }` (`FeatPrerequisites`, `Feature(key)`) in `Class.counts_as`; `Prerequisite::Story(Story)` (`VowOfDahak`, `Uncorrupted`; always holds), `Prerequisite::stories()`, `Engine::story_conditions(pick)`; `CharacterState.feat_class_levels`, `feat_class_level`, `check_feat_for` (a feat pick's prerequisites see them); the walker's `count_toward_features` (rows of the other class past its own level, up to its level plus the counting levels, hold the feature; its own row takes over when reached); `check_advances` allows one class for `Either`; check: `EitherWithOneSpellType`, `CountsAsItself`, `CountsTowardUngrantedFeature`, `StoryNotRequired`. Tests first: new `class_rules.rs` (12) and 4 check tests; the world gains a witch (hexes, Major Hexes at 10, Grand at 18), an Eldritch Knight and a Winter Witch, and the annihilating breath needs Vow of Dahak. Red run: 14 failures (the Either test for one class passed already). `cargo test -p rules` 187 + 1 pass, clippy clean. Next: `data` crate and migration.
- 2026-11-01: `data` done and the data rewritten. Migration: `class.advances` (`each`/`either`), table `class_counts_as (class_key, as_class_key, toward feat_prerequisites|feature, feature_key)` (CHECKs: a feature only and always with `feature`, never itself; FKs to `class` and `feature`), `prerequisite_node` kind `story` with a `story` column (`vow_of_dahak`, `uncorrupted`). 35 tables. Seed, loader, columns. Tests first: files (the fixture's new Loremaster and Eldritch Knight, the Story leaf), postgres (every variant incl. both stories; the new CHECKs and FKs), `game_data.rs` on the real files (8 new; red on the HEAD data). `t28/rewrite.py` ran over `game-data/` (21 changes, byte-identical to the dry run in `t28/dry/`; log `t28/game-data-log.json`) and the fixture (3 changes: the annihilating breath's story, a Loremaster and an Eldritch Knight; `t28/fixture-log.json`). Two older tests updated: `no_selection_option_is_a_copy_of_another` skips options that link to a page, not an anchor (Nature Bond's two have no heading in the guide); the seed-refuses test also names the fixture's Eldritch Knight (the removed fighter row granted the Bravery it counts toward). Dev DB reset (only `wotr-planner-postgres`, 5434), migrated, seeded, `cargo sqlx prepare`. Next: extractor, proof.
- 2026-11-01: extractor done: new `extract::class_rules` (the same tables as `rewrite.py`, quoting the guide): `apply_prestige` (`Either` on the three, after asserting their rows advance both types; the three "counts as" facts), `nature_bond` (the L1 rows granting both companion and domain pick `druid/nature-bond` instead, the domain features go, Boon Companion and Mythical Beast accept the companion option; each asserts it found what it names), run after `parameters::apply` on full rows; `feat_lists_note`/`CRUSADER_NOTE` for `ron_out`, which writes `advances` (when `Either`), `counts_as` (when set) and the Crusader's comment. Grammar: `Cond::Story` ("Vow of Dahak", "Uncorrupted"), and a lone "neutral" next to any other alignment is true neutral (the Assassin's line now gives TN + evil). Tests: `class_rules.rs` (3, including a write-and-read-back) and 2 grammar tests (one updated to R6); written with the code in one pass (the rules come from `rewrite.py`), so no separate red run apart from the grammar expectation changed from HEAD's reading. `cargo test -p extract` 71 pass. A fresh extraction (`t28/extract-new/`) is **byte-identical** to `rewrite.py --extraction` applied to the HEAD extraction (`t28/dry/extract-old/`; 16 changes: the story and Domain Zealot rules have no target there, since the breath options and Domain Zealot's class list are ticket 20's hand-fixes), report unchanged (77 items, 0 check problems). `rewrite.py` re-run on HEAD's `game-data/` and fixture gives the files on disk. Next: proof, docs, final DB reset, Verification.
EOF # ide-unavailable- 2026-11-01: proof (`t28/proof-{old,new}`, shared `t28/proof-common.rs`, `t28/compare.py`) passes: the new data with its new facts undone equals the old; of 5481 walked levels (222 base classes and archetypes × 20, each prestige class after wizard, cleric or witch 10, plus a problems line per Build) 5388 are identical and the 93 others differ only as expected (the three Nature Bond druids, the Eldritch Knight's fighter level for feats, Witch 10 / Winter Witch 8's Grand Hexes). Next: docs, final DB reset, Verification.
- 2026-11-01: docs (`game-data/README.md`, `GLOSSARY.md`, `game-data/EXTRACTION_REPORT.md` 68 → 59); final DB reset and reseed, `cargo sqlx prepare`; every Verification check passes. Done.

## Answer

Done 2026-11-01, uncommitted. A prestige class says whether it advances one class of each spell type or one class, arcane or divine; a class's levels can count as another class's, toward a feat's prerequisites or toward one of that class's features; a story condition is a prerequisite that always holds and shows as a note; the Assassin is true neutral or evil; the druid picks an animal companion or a domain at level 1 (Nature Bond), which Domain Zealot, Boon Companion and Mythical Beast check; the Crusader keeps skipping every prerequisite, and the data says so. No `Build` change, so the v1 link vector stands.

### Rust (`rules`)

- **Advancement.** `Class.advances: Advancement` (`Each`, the default; `Either`). The rows keep `advances_arcane`/`advances_divine`, now read as the spell types a level can advance; `Either` lets the Build name one class of either type, `Each` one per type (Mystic Theurge). `check_advances` refuses a second class for `Either`.
- **Counts as.** `Class.counts_as: BTreeSet<CountsAs { class, toward: CountsToward }>`, `CountsToward::FeatPrerequisites` or `Feature(key)`.
  - Feat prerequisites: `CharacterState.feat_class_levels` (the class level a feat's prerequisites see, for each class another class counts as), `feat_class_level`, and `check_feat_for`, which `check_pick` uses for a feat pick only. A mythic pick, an option, a class entry see the plain class level.
  - A feature: the walker's `count_toward_features` holds the feature once for each of the other class's rows (its archetype's, in force) past its own level and up to its level plus the counting levels; when the class reaches such a row itself, its own grant takes over (Witch 9 / Winter Witch 1 holds Major Hexes; Witch 10 then holds it once). Nothing while the other class isn't held.
- **Story conditions.** `Prerequisite::Story(Story)` (`VowOfDahak`, `Uncorrupted`, with `name()`); it always holds, so it is never an `Unmet` reason. `Prerequisite::stories()` and `Engine::story_conditions(pick)` give the note the sheet shows beside a pick.
- **Data check.** `EitherWithOneSpellType`, `CountsAsItself`, `CountsTowardUngrantedFeature(feature)` (no row of the other class or its archetypes grants it), `StoryNotRequired` (a story leaf directly under `Any` or `AtLeast`, where it would hold the group by itself).
- Tests first, in the test world (new `class_rules.rs`, 12, plus 4 check tests): the world gains a witch (a hex at 1 and every even level, Major Hexes at 10, Grand Hexes at 18, hexes needing them), an Eldritch Knight (counts as fighter for feats, Combat feat slots at 1, 5, 9) and a Winter Witch (`Either`, counts toward the witch's Major and Grand Hexes), and the annihilating breath needs Vow of Dahak. Red run: 14 failures.

### Schema (the one migration)

- `class.advances` (`each`, `either`).
- New `class_counts_as (class_key, as_class_key, toward feat_prerequisites|feature, feature_key)`: FKs to `class` (both) and `feature`; CHECKs: `feature_key` set iff `toward = 'feature'`, never the class itself; unique.
- `prerequisite_node` kind `story` with a `story` column (`vow_of_dahak`, `uncorrupted`), filling only it.
- 35 tables (34 + `class_counts_as`). Seed, loader and columns cover each. Tests: files (`advances`, `counts_as` and `Story` read, and their defaults), postgres (every prerequisite variant incl. both stories round-trips; the new CHECKs and FKs reject what they should), `data/tests/game_data.rs` on the real files (8 new; red on the HEAD data).

### Data rewrite

Script: `t28/rewrite.py` in the session scratchpad (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t28/`), line based, quoting the guide for each rule; it changes only the lines it logs (`t28/game-data-log.json`, 21 changes; `t28/fixture-log.json`, 3). The extractor was never run over `game-data/`.

| Change | Real data | Fixture |
| --- | --- | --- |
| `advances: Either` ("advances one class, arcane or divine"; rows already advance both) | Hellknight Signifer, Loremaster, Winter Witch | a Loremaster, by the script |
| `counts_as` | Eldritch Knight → fighter, feat prerequisites ("Eldritch Knight levels count as Fighter levels for the purposes of unlocking feats"); Winter Witch → witch, toward `witch/major-hexes` and `witch/grand-hexes` ("... with Witch levels to determine ... access to major and grand hexes") | an Eldritch Knight → fighter, feats and Bravery |
| story conditions | the four MR 9 breaths: Annihilating and Decaying `All([MythicRank(9), Story(VowOfDahak)])`, Enlightening and Rejuvenating `Story(Uncorrupted)` | the annihilating breath |
| Assassin | true neutral or any evil (was 7 alignments with Neutral Good) | none |
| Crusader | a `//` comment above its three bonus lists: the guide skips only BAB and fighter level, the data every prerequisite (R5) | none |
| Nature Bond | `druid/nature-bond` (Animal Companion, Domain; guide `druid`, which has no heading for it) at level 1 of the druid, Defender of the True World and Feyspeaker, replacing each one's level 1 `animal-companion` and `domain-abilities` features; the three `domain-abilities` features (granted nowhere else) are gone | none |
| prerequisites | Domain Zealot: the druid's class level → the domain option, Blight Druid ("May choose from the following domains") or Winter Child ("Must select a specific domain"); Boon Companion and Mythical Beast also accept the companion option | none |

Two older tests changed: `no_selection_option_is_a_copy_of_another` skips options that link to a page rather than an anchor (Nature Bond's two), and the seed-refuses test also names the fixture's Eldritch Knight (the fighter row it removes granted the Bravery the knight counts toward).

### Proof

`t28/proof-old/` (against `git archive HEAD`, d7492b3) and `t28/proof-new/` build one shared source, `t28/proof-common.rs`; `t28/compare.py` checks:

- **Data:** the whole `GameData` as JSON from each side. Each new fact is checked against `rewrite.py`'s tables and undone (`advances`, `counts_as`, the four story leaves, the Assassin's alignments, the Nature Bond selection and rows and the three domain features, Domain Zealot, the companion option in Boon Companion and Mythical Beast); what's left **equals the old data**, and no other story leaf exists.
- **Walk:** every base class and archetype through 20 levels, and each prestige class after 10 wizard, cleric or witch levels (advancing that class), dumping per level the slots, each slot's options by key (open if any value is), the feats, features, options and fighter level for feats, plus each Build's problems; the new side fills Nature Bond with its first open option (the companion). 5388 of 5481 lines are identical. The 93 others differ only as expected: the three Nature Bond druids (the slot at 1, the option held instead of the two features, and Boon Companion closed in the level-1 feat slots, which come before the class's selections: question 1), the Eldritch Knight's fighter level for feats (3 Builds × 10), and Witch 10 / Winter Witch 8 holding Grand Hexes from 18.

### Extractor

- New `extract::class_rules`, with the same tables as `rewrite.py`: `apply_prestige` (`Either` on the three after asserting their rows advance both types; the three `counts_as`), `nature_bond` (the level 1 rows granting both the companion and the domain pick `druid/nature-bond`, the domain features go, Boon Companion and Mythical Beast accept the companion option; each asserts it found what it names), run after `parameters::apply` on full rows. `ron_out` writes `advances` (when `Either`), `counts_as` (when set) and the Crusader's comment (`class_rules::CRUSADER_NOTE`).
- Grammar: `Cond::Story` ("Vow of Dahak", "Uncorrupted" lines) resolves to `Prerequisite::Story`; next to any other alignment a lone "neutral" is true neutral, so the Assassin's line now reads TN + evil.
- Tests: `class_rules.rs` (3, including a write-and-read-back) and grammar (1 new, 1 updated to R6). These were written with the code in one pass (the rules come from `rewrite.py`'s tables); the red is the grammar expectation that changed from HEAD's reading.
- A fresh extraction (`t28/extract-new/`) is **byte-identical** to `rewrite.py --extraction` applied to the HEAD extraction (`t28/dry/extract-old/`, 16 changes); the report is unchanged (77 items, 0 check problems). The story and Domain Zealot rules have no target in an extraction: the breath options and Domain Zealot's class list are ticket 20's hand-fixes.

### Docs

- `game-data/README.md`: `advances: Either`, `counts_as`, Nature Bond, `Story(..)` and where it may go, `//` comments (the Crusader's).
- `GLOSSARY.md`: new Counts as and Story condition; Class choice and Prerequisite updated.
- `game-data/EXTRACTION_REPORT.md`: 9 items fixed or settled (Assassin, Diverse Training, Frost Power, the three "advances one class", the breaths' story conditions, Domain Zealot, the Crusader): 68 → 59.

### Verification

- `cargo test --workspace`: pass (rules 187 + unit 1, extract 71, data files 20 + game_data 34 + postgres 17).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (5 query files replaced).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 63 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths. Rows: `class.advances = 'either'` 3, `class_counts_as` 3, `story` nodes 4 (2 each), Nature Bond options 2, `feature` 1748 → 1745; 35 tables.
- The v1 link vector is unchanged (the `Build` type didn't change).
- Proof: passes (above).

### Open questions (for the user)

1. **Level-1 order for a druid.** Nature Bond is a class selection, checked after the level's feat slots (race, class, general), so a druid can't take Boon Companion at level 1 any more (it could while the companion was granted outright). Same root as ticket 27's question 7 (the warpriest's Weapon Focus): accept it, or check a level's class selections before its feat slots (a rules change for every class).
2. **The druid's domain footnotes.** The druid's (and Defender's, Feyspeaker's) "Animal Companion" at 4 ("Animal domain only"), "Domain Ability" at 6 (Air, Earth, Fire, Plant and Water) and "Lightning Lord" at 8 (Weather) are still granted to every druid, whatever Nature Bond picked, so a domain druid holds a companion from 4 (Boon Companion opens). They are domain option rows: ticket 31.
3. **Diverse Training's caster-level half** ("his class level counts as spellcaster levels for the purposes of unlocking feats from his spellcasting class") isn't stored: no feat in the data checks a caster level. If one does later, it needs a "counts as" toward the advanced class.
4. **Frost Power and the shaman.** The guide stacks Winter Witch with shaman levels only "to determine the power of Shaman hexes and Frost Spirit abilities": power, which a Build doesn't hold, so nothing is stored. The Winter Witch's entry still takes any shaman for "Shaman (Frost Spirit)": the spirit selection is granted nowhere yet (ticket 30).
5. **Razmiri Infiltrator.** The arcanist archetype says "Her alignment must be Neutral or Evil" too, but its table isn't read (a report item) and it has no alignment prerequisite. Does R6's reading (true neutral or any evil) apply there? The extractor's grammar would read it that way now.
6. **Other domains for Domain Zealot.** The Hellknight's Pentamic Faith gives a domain, but orders aren't a selection (a report item), so Domain Zealot doesn't name it.

## Decisions on the open questions

**User, 2026-11-01 ("all recommended"):**
1. Level-1 slot order: folded into [ticket 27](27-repeatable-and-parameterised-feats.md)'s decision 7, built in ticket 32.
5. The Razmiri Infiltrator way needs true neutral or any evil, the Assassin's reading. Ticket 32.
