# Share selection options between selections

Type: task
Status: resolved
Blocked by: 24
Map: [WotR Build Planner](../map.md)

## Question

**User decision (2026-11-01, [ticket 21](21-schema-questions-real-data.md) S4, option (a)).** 47 options are copies of another selection's (36 rogue talents in `slayer/slayer-talents`, 11 witch hexes in `shaman/hex`), so a rogue/slayer can take one talent twice.

- **Rust:** `FeatureSelection.shared_options: BTreeSet<SelectionOptionKey>` (`#[serde(default)]`). An option keeps the key of the selection that defines it; the engine offers own ∪ shared, and already-held compares the one key.
- **Schema:** `selection_shared_option (selection_key, option_key, PRIMARY KEY (both), CHECK (NOT starts_with(option_key, selection_key || '/')))`.
- **Data:** delete the 47 copies and their prerequisite trees; the slayer shares the 36 rogue talents and the shaman the 11 witch hexes.
- **Class-dependent prerequisites:** where a copy named its own class's feature (the 16 advanced slayer talents need `slayer/advanced-talents`), the shared option's prerequisite accepts either class's feature with an `Any`. Accepted trade-off: a rogue 10 / slayer 2 can then take an advanced talent through the slayer slot.
- **A slot on two lists:** Fey Tricks (rogue talents + witch hexes) and Underground Chemist (rogue talents + alchemist discoveries) become their own selections sharing both lists, if the guide data has their slots; otherwise list them in the report.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB): data by one-off script, extractor emits the new shape, docs updated. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus a test that a rogue/slayer can't take one talent twice.

## Progress log

- 2026-11-01: baseline in the session scratchpad `t25/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t25/`): `old-src/` is `git archive HEAD` (d73e098); `extract-old/` is the HEAD extractor's output (identical to ticket 24's `t24/extract-new/`); `proof-old/` dumps every selection's offered options (guide anchor, name, feat, canonical prerequisite with option keys shown as anchors) and every class/archetype level's and path rank's selection slots: `old-proof.jsonl` (4688 lines, 49 selections), `old-fixture-proof.jsonl` (31). Findings: the data has the Fey Tricks slots (Sylvan Trickster's even levels, a `fey-tricks` feature beside a `rogue/rogue-talents` slot) and the Underground Chemist's Discovery slot (level 10; the guide: "unlocks certain discoveries that he can take in place of Rogue Talents", report item `rogue#discovery`: "from 10th level"), so both become selections. Only `slayer-camouflage` references a copy outside the copies (`slayer/slayer-talents/terrain-mastery`); no test Build, fixture or link vector does. Next: rules types and engine, test-first.
- 2026-11-01: `rules` done. `FeatureSelection.shared_options: BTreeSet<SelectionOptionKey>` (`#[serde(default)]`) and `FeatureSelection::offers(key)` (own or shared); `Engine::options` lists own ∪ shared in key order (a shared key the data lacks offers nothing); `check_pick` accepts an offered key and already-held compares the one key. `GameData::check` adds `DataIssue::SharesOwnOption(key)` (the schema CHECK's twin). Tests first: new `shared_options.rs` (6: the slayer slot offers own ∪ shared in key order; a shared talent is held under the rogue's key with its feat; a rogue/slayer can't take one talent twice, either order; an own option needing a shared one sees it from either class's slot; a selection refuses an option it neither defines nor shares; an `Any` over each class's level lets either qualify, including the accepted rogue 10 / slayer 2 trade-off) and a check test; the world gains a slayer (talents share the rogue's three, plus Slayer Camouflage needing Fast Stealth). `cargo test -p rules` 132 + 1 pass, clippy clean, wasm builds. Next: `data` crate.
- 2026-11-01: `data` done and the data rewritten. Migration: new `selection_shared_option (selection_key → feature_selection, option_key → selection_option, PRIMARY KEY both, CHECK NOT starts_with(option_key, selection_key || '/'))` (29 tables); the seed inserts each selection's shared keys, the loader groups them back (a row naming no selection is an error). Tests first: files (`shared_options` read, defaulted, `offers`), postgres (round trip through the fixture's new sharing selection; the CHECK, the FK and the primary key), `game_data.rs` on the real files (no option copies another's anchor; a rogue/slayer can't take one talent twice; a slayer 10 takes advanced rogue talents; the shaman shares 11 witch hexes; Fey Tricks offers 36 talents + 26 hexes, major from 10, grand from 18; the chemist's Discovery from 10 offers 36 + 6 and Mummification after Preserve Organs). Fixture by hand: Eldritch Scoundrel gets `rogue/eldritch-scoundrel/arcane-talents` (one own option, shares Combat Trick and Crippling Strike) at level 3. `t25/rewrite.py` ran over `game-data/` (same as the dry run in `t25/dry/`) and the fixture (no-op): 47 copies deleted, 1 reference redirected, 30 gates widened, 2 new selections, 16 slots. Dev DB reset (only `wotr-planner-postgres`, 5434), migrated, seeded (same counts), `cargo sqlx prepare`. **Proof passes**: `t25/compare.py old-proof.jsonl new-proof.jsonl` finds all 49 selections and 4639 slot rows equal except 46 gate changes, 1 name (the shaman's Healing is now the witch's "Healing (Hex)") and the 16 Fey Tricks / Discovery slots. Next: extractor.
- 2026-11-01: extractor done: new `extract::shared_options` (`SHARES`, the same table as `rewrite.py`, quoting the guide; `apply_shares` deletes copies by anchor and points references at the originals, widens the gates, adds `shared_options`, and for Fey Tricks and Discovery adds the archetype's selection and takes over the rows its `SlotRule` picks), run after `proficiencies::apply` on the full rows; `references::prerequisites_mut` factored out of `drop_redundant_alternatives`; `ron_out` writes `shared_options` one key per line. Tests first: `shared_options.rs` (7). `cargo test -p extract` 57 pass. A fresh extraction (`t25/extract-new/`) is byte-identical to `rewrite.py` applied to the HEAD extraction (`t25/dry/extract-old/`), and `t25/same-data` says same data; `EXTRACTION_REPORT.md` unchanged (82 items, 0 check problems). Next: docs, full verification.
- 2026-11-01: docs (`game-data/README.md`, `GLOSSARY.md`: Feature selection, new Shared option; `EXTRACTION_REPORT.md`: `rogue#discovery` and `rogue#fey-tricks` fixed, 77 items), final DB reset and reseed, `cargo sqlx prepare`, every Verification check passes. Done.

## Answer

Done 2026-11-01, uncommitted. Selections share options instead of copying them: the 47 copies (36 slayer talents, 11 shaman hexes) and their prerequisite trees are gone, a selection lists the other selections' options it offers in `shared_options`, and a shared option keeps its one key, so a rogue/slayer can't take one talent twice. Where a copy named its own class's gate, the original now accepts either class's with an `Any` (decision S4, option (a)). The data has the Fey Tricks and Underground Chemist slots, so both are now selections sharing two lists.

### Rust (`rules`)

- `FeatureSelection.shared_options: BTreeSet<SelectionOptionKey>` (`#[serde(default)]`), and `FeatureSelection::offers(key)`: one of its own options or one it shares.
- `Engine::options` lists own ∪ shared, in key order; a shared key the data lacks offers nothing (missing keys are the database's foreign keys' job, as everywhere). `check_pick` accepts any key the slot's selection offers, and already-held compares that one key, so a second pick of a talent in either class's slot is `AlreadyHeld`. Holding, releasing, the option's feat and would-break are unchanged: they already worked by key.
- `GameData::check` adds `DataIssue::SharesOwnOption(key)`: a selection sharing a key under its own (the schema CHECK's twin).
- Tests first: new `shared_options.rs` (6): the slayer slot offers own ∪ shared in key order; a shared talent taken in a slayer slot is held under the rogue's key with its feat; **a rogue/slayer can't take one talent twice**, in either order; an own option that needs a shared one (Slayer Camouflage needs Fast Stealth) sees it taken in either class's slot; a selection refuses an option it neither defines nor shares; an `Any` over each class's level lets either qualify, including the accepted rogue 10 / slayer 2 trade-off. One check test. The test world gains a slayer (talents share the rogue's three and add Slayer Camouflage); three tests that list the world's classes were updated for it.

### Schema (the one migration)

- `selection_shared_option (selection_key → feature_selection, option_key → selection_option, PRIMARY KEY (selection_key, option_key), CHECK (NOT starts_with(option_key, selection_key || '/')))`. 29 tables.
- The seed inserts each selection's shared keys; the loader groups them back by selection (a row naming no selection the loader read is an error, like the other grouped tables).
- Tests: the fixture round trip now carries a sharing selection; the CHECK, the foreign key and the primary key reject what they should; `files.rs` reads `shared_options` and defaults it; `data/tests/game_data.rs` on the real files: no option copies another's guide anchor, a rogue/slayer can't take one talent twice, a slayer 10 takes advanced rogue talents, the shaman shares 11 witch hexes, Fey Tricks offers 36 talents + 26 hexes (major from 10, grand from 18), and the chemist's Discovery offers 36 + 6 from 10th level, with Mummification after Preserve Organs.

### Data rewrite

Script: `t25/rewrite.py` in the session scratchpad (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t25/`), with a `SHARES` table quoting the guide. Line based: a copy is a sharing selection's option with a shared option's guide anchor; before deleting one, the script asserts the copy equals its original (name aside) once its references point at originals and its class's gate reads as the original's, which proves the `Any`s below cover every difference. Everything else stays byte for byte. The extractor was never run over `game-data/`.

| | Real data | Fixture |
| --- | --- | --- |
| copies deleted, with their prerequisite trees | 47 (36 `slayer/slayer-talents`, 11 `shaman/hex`) | 0 |
| references to a copy pointed at the original | 1 (Slayer Camouflage's Terrain Mastery); none in tests, test Builds or link vectors | 0 |
| shared options | 151: slayer talents 36, shaman hexes 11, Fey Tricks 62, Discovery 42 | 2 (by hand) |
| gates widened to an `Any` | 30 | 0 |
| new selections | 2 | 1 (by hand) |
| slots moved to a new selection | 16 (5 of them new Underground Chemist rows, 12–20) | 1 (by hand) |

- **Slayer talents** share all 36 rogue talents and keep their own Trapfinding and Slayer Camouflage ("counting his Slayer class level as his Rogue level").
- **Shaman hexes** share the 11 the guide names ("The following hexes are shared with the Witch: …"). The copy called Healing is now the witch's option, named "Healing (Hex)".
- **Fey Tricks** (`rogue/sylvan-trickster/fey-tricks`, `options: []`) shares the 36 rogue talents and all 26 witch hexes, and is the Sylvan Trickster's slot at every even level in place of `rogue/rogue-talents`. The `fey-tricks` feature stays on those rows (Extra Rogue Talent's prerequisite names it).
- **Discovery** (`rogue/underground-chemist/discovery`) shares the 36 rogue talents and the six discoveries the guide lists, and is the chemist's slot at 10, 12, …, 20: "At 10th level, an Underground Chemist unlocks certain discoveries that he can take in place of Rogue Talents", as the report item read it ("from 10th level"). The guide's table names Discovery only at 10; see open question 1.
- **The intended `Any` changes** (every one checked by the proof; the alternative is the sharing class's):
  - 16 rogue talents gated on Advanced Talents (Blinding Strike, Confounding Blades, Crippling Strike, Dispelling Attack, Double Debilitation, Familiar, Hunter's Surprise, Improved Evasion, Opportunist, Petrifying Strike, Skill Mastery ×3, Slippery Mind, Weapon Snatcher, Wearying Strike): `Feature(rogue/advanced-talents)` → `Any([rogue/advanced-talents, slayer/advanced-talents])`. These are exactly the 16 copies that named the slayer's feature.
  - 9 major hexes (Agony, Animal Skin, Beast's Gift, Delicious Fright, Hoarfrost, Major Ameliorating, Major Healing, Regenerative Sinew, Restless Slumber): `Feature(witch/major-hexes)` → `Any([rogue/sylvan-trickster/major-hexes, witch/major-hexes])`.
  - 4 grand hexes (Animal Servant, Death Curse, Lay to Rest, Life Giver): `Feature(witch/grand-hexes)` → `Any([rogue/sylvan-trickster/grand-hex, witch/grand-hexes])`.
  - Mummification: `ClassLevel(alchemist, 10)` → `Any([ClassLevel(alchemist, 10), Feature(rogue/underground-chemist/discovery)])`.
- **Fixture**, by hand: the Eldritch Scoundrel gets `rogue/eldritch-scoundrel/arcane-talents` (one own option, shares Combat Trick and Crippling Strike) as its level-3 slot.

### Proof

`t25/proof-old/` (built against `git archive HEAD`) and `t25/proof-new/` dump, for one data folder, every selection's offered options (guide anchor, name, feat, canonical prerequisite, with option references shown as the option's anchor so a copy's reference and the original's compare equal) and the selection slots of every class and archetype level and path rank. `t25/compare.py` checks that every selection that existed offers the same options by anchor, each with the same feat and prerequisite, allowing only a prerequisite equal to the old one with its gate widened (a copy's own-class gate read as the original's), and that every slot row offers the same options, allowing only the Fey Tricks rows and the chemist's rows from 10. **Result: all 49 selections and 4639 slot rows match**, with 46 widened gates allowed (the 30 above, plus the 16 advanced slayer talents seen from the slayer's side), 1 name (Healing) and the 16 slots, each +26 (Fey Tricks) or +6 (Discovery) options. The fixture matches except its hand-added selection.

### Extractor

- New `extract::shared_options`: `SHARES` (the script's table, same quotes). `apply_shares` deletes a sharing selection's copies by anchor and points every prerequisite at the originals, widens the gates, adds `shared_options`, and for a new selection adds it to its archetype and moves the rows its `SlotRule` picks (`WithFeature`, `FromLevel`), adding the slot where the extractor's row carries only the feature. It runs after `proficiencies::apply`, on the full rows. `references::prerequisites_mut` is factored out of `drop_redundant_alternatives`; `ron_out` writes `shared_options` one key per line.
- Tests first: `shared_options.rs` (7, including a write-and-read-back of a sharing selection).
- A fresh extraction (`t25/extract-new/`) is **byte-identical** to `rewrite.py` applied to the HEAD extractor's output (`t25/dry/extract-old/`), and `t25/same-data` reads them as the same data; `EXTRACTION_REPORT.md` is unchanged (82 items, 0 check problems).

### Docs

- `game-data/README.md`: shared options, never copying, gates as an `Any`.
- `GLOSSARY.md`: Feature selection updated; new Shared option.
- `game-data/EXTRACTION_REPORT.md`: `rogue#discovery` and `rogue#fey-tricks` fixed and removed (77 items).

### Verification

- `cargo test --workspace`: pass (rules 132 + unit 1, extract 57, data files 16 + game_data 10 + postgres 15).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (one query file replaced, the TRUNCATE; two new).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths (unchanged). Rows: `selection_option` 579 → 532, `selection_shared_option` 151, `feature_selection` 49 → 51, `class_level` 2272 → 2277, `granted_selection` 325 → 330.
- No file outside the tickets names a deleted copy's key, except the extractor test that deletes copies.
- The v1 link vector needed no regeneration: the `Build` type didn't change and the vector picks only `rogue/rogue-talents/combat-trick`.
- Proof: matches (above).

### Open questions (for the user)

1. **The Underground Chemist's Discovery: from 10th level, or at 10 only?** The data follows the text and the report ("unlocks … in place of Rogue Talents"): every talent slot from 10 offers the six discoveries. The guide's table puts Discovery only in the level-10 cell (Rogue Talent at 12–20). If only the level-10 pick may be a discovery, the chemist's rows at 12–20 go.
2. **Option (a)'s trade-offs beyond the slayer's**, from the same rule: a witch 2 / Sylvan Trickster 10 can take a major hex in the witch's slot (and a witch 10 / Sylvan Trickster 2 through Fey Tricks); an alchemist 2 / Underground Chemist 10 can take Mummification in the alchemist's slot. Option (b) or (c) would close them if they matter.
3. The `rogue/sylvan-trickster/fey-tricks` feature now sits beside the selection of the same key on every Fey Tricks row. It stays because Extra Rogue Talent's prerequisite names it; it could go if that `Any` is dropped (a trickster's talents now hold the rogue's keys anyway).
4. Still not modelled (report items, outside this ticket): the shaman's Second Spirit and Wandering Hex, Winter Witch's hexes, and the slayer's ranger combat style feats.

## Decisions on the open questions

**User, 2026-11-01 ("all recommended"):**
1. Underground Chemist's Discovery applies from level 10 on (the text describes a lasting unlock). No data change.
2. The wider "any of" trade-offs (witch 2 / Sylvan Trickster 10, alchemist 2 / Underground Chemist 10) are accepted. No data change.
