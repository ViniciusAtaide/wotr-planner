# Sorcerer level-1 feat, Extra Slayer Talent, deities

Type: task
Status: resolved
Blocked by: 43
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 43](43-extraction-report-gaps.md)'s decisions 1, 3 and 4 (user, 2026-11-02):

1. **Sorcerer level-1 bonus feat.** Every sorcerer, base included, gets a level-1 bonus feat slot from the held bloodline's bonus-feat list (`OptionList(selection: "sorcerer/bloodline")`, as at 7/13/19, prerequisites as there). Crossblooded picks from either of its two bloodlines' lists; the Empyreal, Sage and Sylvan from their fixed bloodline's; the Seeker and Nine Tailed Heir from their chosen bloodline's (the Seeker's current Magic-feat slot at level 1 is replaced; its 5/10/15 stay); the Geomancer keeps its General slot. The Overwhelming Mage is unchanged (it gives the feats up). The base sorcerer gains the slot it lacks today.
2. **Extra Slayer Talent** needs slayer level 2 (`ClassLevel(class: "slayer", level: 2)`), the level the slayer's table first gives Slayer Talents. Talented Slayer stays out (not on any saved page).
3. **Deities stay out of scope.** The cleric's deity-alignment item leaves the report; add "Deities: a cleric's deity and its alignment and domain limits" to the map's Out of scope.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same method as tickets 37, 38 and 43: rules in the extractor (extend `extract::report_gaps`), a one-off rewrite of `game-data/`, real-data tests first (red run saved), a fresh extraction that matches the rewrite, a before/after proof, the report's items removed (it should end with 0 items: the kineticist item moves to ticket 45, so mark it there instead of in the report, or leave it in the report pointing to ticket 45, whichever the report's conventions suit).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a sorcerer 1 with the Draconic bloodline is offered its bloodline feats and not Power Attack at level 1; a Crossblooded sorcerer is offered both bloodlines' lists; an Empyreal Sorcerer its fixed bloodline's; a Seeker at 1 no longer a Magic feat; a Geomancer still any feat; Extra Slayer Talent locked for a slayer 1 ("Needs Slayer level 2 (have 1)") and open at 2, and locked for a rogue.

## Progress log

- 2026-11-02: started. Baseline: HEAD 7b8f898 (the user's uncommitted doc comment in `server/src/main.rs` left alone); scratch in `t44/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t44/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (byte-identical to `t43/extract-new/`: only tickets changed since ticket 43). Read the decisions against the saved pages: every sorcerer table but the Overwhelming Mage's has "Bonus Feat" at 1 (the nine bloodlines' and seven archetypes'); Crossblooded's "at the bonus feat levels, you may choose a feat from one bloodline or the other"; the Nine Tailed Heir and Seeker choose a bloodline "for the purpose of bonus feats"; the Seeker's own "Bonus Feat" section is its 5th, 10th and 15th; the Geomancer "does not gain bloodline spells or bloodline feats"; the slayer's table first gives Slayer Talents at 2; the cleric's sentence is in the Cleric section.
- 2026-11-02: real-data tests first (`game_data.rs`, 7 new): a sorcerer 1's bloodline slot (Draconic: exactly its 7 feats, Toughness takeable, Dodge and Combat Casting not offered; Abyssal: its list, no Toughness), a Crossblooded's both lists, the Empyreal/Sage/Sylvan's fixed bloodline's list (no Power Attack), a Seeker's bloodline list at 1 and Magic slots at 5/10/15 and a kitsune Nine Tailed Heir's chosen list, a Geomancer's General slot (Power Attack and Dodge offered) and an Overwhelming Mage's no slot, the labels ("Bloodline feat" at 1 and 7, a Geomancer's "Bonus feat", every slot of a sorcerer 20 labelled), Extra Slayer Talent (the `ClassLevel`, "Needs Slayer level 2 (have 1)" at slayer 1, open for a fighter 1 / slayer 2, locked for a rogue 1). Red run on HEAD's data (`t44/real-data-red.txt`, 126 pass, 6 fail, each at the slot kind or the prerequisite the ticket changes); the Geomancer test passes from the start (it pins data that stays). The Nine Tailed Heir test then needed a kitsune (the archetype's `Race("kitsune")`); the red run had stopped at the Seeker before reaching it.
  - The ticket's "a sorcerer 1 with the Draconic bloodline is offered … not Power Attack": the Draconic list has Power Attack ("Blind-Fight, Great Fortitude, Improved Initiative, Quicken Spell, Power Attack, Skill Focus, Toughness"), so the slot offers it, locked at sorcerer 1 by its BAB +1. The test checks exactly the Draconic list and that Power Attack isn't takeable; "no Power Attack" is tested on the Empyreal, Sage and Sylvan (whose lists lack it).
- 2026-11-02: data and extractor. `extract::report_gaps::decisions` (ticket 44's rules; `apply` runs ticket 43's, then it): `sorcerer_level_1_feat` asserts the 16 tables' "Bonus Feat" at 1, the Overwhelming Mage's Overwhelming Penetration there, the five sentences above, the eight archetypes by key, the base row 1 with no slot and the bloodline pick, each archetype's row-1 slot (General; the Seeker's Magic; none for the Overwhelming Mage) and a bloodline it holds there, and the Seeker's Magic at 5/10/15; `extra_slayer_talent` asserts the feat's line, the slayer table's first Slayer Talents at 2, the slayer data's first `slayer/slayer-talents` row at 2 and no prerequisite yet; `deity` asserts the cleric's sentence and changes nothing. `settled` drops the six sorcerer items, Extra Slayer Talent's and the cleric's; the new `moved` drops the kineticist's (ticket 45 carries it; ticket 45's done-when says so). `t44/rewrite/` (from `t43/rewrite/`, calling `decisions`) wrote `game-data/` byte-identical to its dry run: 8 changes in 2 files (`classes/sorcerer.ron`, `feats.ron`; `t44/game-data-log.json`), 0 check problems. Real-data tests green (132, `t44/real-data-green.txt`). A fresh extraction (`t44/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t44/rewrite/` (`t44/dry/extract-old-rewritten/`, the same 8 changes, line for line those in `game-data/`); its report differs by the 9 items only (32 → 23). `extract/tests/extract/report_gaps.rs`: 2 tests (written first; red as a compile error, `moved` not there yet). `cargo test -p extract` 90 pass.
- 2026-11-02: proof, docs, DB, Verification. Proof (`t44/proof/` on `t44/proof-common.rs`, ticket 43's Builds plus `decision_builds`; one program, since ticket 44 changes no rules; `t44/compare.py`, 103894 checks) passes. Docs: `game-data/README.md` (ticket 44's paragraph), `game-data/EXTRACTION_REPORT.md` (9 → 0 items), ticket 45 (the kineticist item), the map (Decisions so far; Deities in Out of scope). Reseeded the dev DB (no schema change, so no reset). Verification below.

## Answer

Done 2026-11-02, uncommitted. Ticket 43's decisions 1, 3 and 4 are built; the report is at 0 items (the kineticist's moved to ticket 45). No schema, `rules` or `Build` change, so the v1 link vector stays.

### Data

`extract::report_gaps::decisions` asserts each guide sentence and the data's shape before changing anything:

- **The sorcerer's level-1 bonus feat:** an `OptionList(selection: "sorcerer/bloodline")` slot on row 1, as at 7, 13 and 19: added for the base sorcerer, and in place of the General slot of Crossblooded (either bloodline's list), the Empyreal (Celestial), Sage (Arcane), Sylvan (Fey) and Nine Tailed Heir (its chosen bloodline's), and of the Seeker's Magic slot (its chosen bloodline's; its magic feats stay at 5, 10 and 15). The Geomancer keeps its General slot, the Overwhelming Mage has none. A pick off the list in an old link (an Empyreal's Toughness at 1) is now an impossible Build, which the sheet treats as a stale link. The sheet labels the slot "Bloodline feat", as at 7.
- **Extra Slayer Talent:** `ClassLevel(class: "slayer", level: 2)`, read as "Needs Slayer level 2 (have 1)" at slayer 1. Talented Slayer stays out.
- **The cleric's deity:** no data; the report item goes and deities are in the map's Out of scope.

`t44/rewrite/` (in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t44/`) ran `report_gaps::decisions` over `game-data/`: 8 changes in 2 files (sorcerer, feats), byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

Real-data tests (`game_data.rs`, 7 new; red on HEAD's data first, `t44/real-data-red.txt`; the Geomancer/Overwhelming Mage test green from the start, since that data stays):

- a sorcerer 1's slot offers exactly the held bloodline's list (Draconic; Abyssal), and Toughness in it is a possible Build;
- a Crossblooded sorcerer 1 is offered both bloodlines' lists;
- the Empyreal, Sage and Sylvan Sorcerers 1 their fixed bloodline's list, no Power Attack;
- a Seeker 1 its bloodline's list and no other Magic feat, Magic slots at 5, 10, 15; a kitsune Nine Tailed Heir 1 its bloodline's list;
- a Geomancer 1 still any feat; an Overwhelming Mage 1 no class feat slot;
- the labels: "Bloodline feat" at 1 and 7, a Geomancer's "Bonus feat", every slot of a sorcerer 20 labelled;
- Extra Slayer Talent: locked at slayer 1 ("Needs Slayer level 2 (have 1)"), open for a fighter 1 / slayer 2 at 3, locked for a rogue 1.

### Extractor

`extract` runs `report_gaps::apply` (ticket 43's rules, then `decisions`) and drops the items `settled` names (the totems'; the six sorcerer archetypes' "Bonus Feat", Extra Slayer Talent's line, the cleric's deity) and the one `moved` names (`kineticist#kineticist-bonus-feat`, which [ticket 45](45-kineticist-wild-talents.md) settles). A fresh extraction (`t44/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t44/rewrite/`; its report differs by those 9 items only (32 → 23 items). An extraction's base sorcerer still has no slots at 7, 13 and 19 (a hand fix only `game-data/` holds, from before this ticket); it now has the level-1 one. `extract/tests/extract/report_gaps.rs`: 2 tests.

### Proof

`t44/proof/` builds `t44/proof-common.rs` (ticket 43's Builds, plus a kitsune Nine Tailed Heir 7, a Geomancer 3, a fighter 1 / slayer 2 and a rogue 1 / slayer 2, and five links: an Empyreal's Toughness and a Seeker's Extend Spell in the level-1 slot, a sorcerer's Toughness in the new slot, Extra Slayer Talent at slayer 1 and at fighter 1 / slayer 2) against the unchanged rules, and walks it over `old-src/game-data` and the new data. `t44/compare.py` (103894 checks pass):

- **Data:** the old data, with the base sorcerer's row-1 slot added, the six archetypes' row-1 slot turned into it, and Extra Slayer Talent's `ClassLevel`, is exactly the new data.
- **Walk:** all 6387 lines are identical but for: the base sorcerer's new slot at its 1st sorcerer level (24 Builds) and the six archetypes' slot there (35 Builds), each offering exactly the held bloodlines' lists with every feat open or closed as the old general (or Magic) slot had it; Extra Slayer Talent closing 2653 times in 356 Builds, always below slayer level 2; three links now impossible (the Empyreal's Toughness, the Seeker's Extend Spell, Extra Slayer Talent at slayer 1: their pickers lock 176, 194 and 148 options) and one now possible (the sorcerer's Toughness in the new slot: 183 options open). The Geomancer's and Overwhelming Mage's slots, and the Extra Slayer Talent link at slayer 2, don't move.

### Docs

- `game-data/README.md`: ticket 44's paragraph.
- `game-data/EXTRACTION_REPORT.md`: 9 → 0 items; the header says what ticket 44 fixed, dropped and moved.
- [Ticket 45](45-kineticist-wild-talents.md): its done-when names the moved kineticist item and `report_gaps::moved`.
- The map: a Decisions-so-far line; "Deities: a cleric's deity and its alignment and domain limits" in Out of scope.
- `GLOSSARY.md`: unchanged (no new term).

### Verification

- `cargo test --workspace`: pass (rules 307 + unit 2, extract 90, data files 25 + game_data 132 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (no query file changed).
- No schema change, so no DB reset: `cargo run -p data --bin seed` reseeded `wotr-planner-postgres` on 5434 only; the postgres tests pass on the reseeded DB.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 39 backgrounds.
- Rows: `feat_slot` 282 (was 281): `option_list` 53 (46), `general` 23 (28), `category` 51 (52); `prerequisite_node` 2008 (was 2007: Extra Slayer Talent's `class_level`).
- Ticket 22's acceptance grep finds nothing (also over `extract` and `data/src`).
- The v1 link vector is unchanged (no `Build` change).
- Proof: passes (above).

### Open questions

None.
