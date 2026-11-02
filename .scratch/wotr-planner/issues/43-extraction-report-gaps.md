# Close the extraction report's gaps

Type: task
Status: resolved
Blocked by: 12
Map: [WotR Build Planner](../map.md)

## Question

`game-data/EXTRACTION_REPORT.md` still lists 12 items (Feats 1, Base classes 11). Most let the sheet hold a Build the game doesn't allow, against the rule that the sheet only ever holds a possible Build ([ticket 39](39-impossible-states-not-validation.md)). User, 2026-11-02: close these before the end-to-end tests.

The items:

1. **Sorcerer archetypes' bonus feats** (`sorcerer#crossblooded`, `#empyreal-sorcerer`, `#geomancer`, `#nine-tailed-heir`, `#sage-sorcerer`, `#sylvan-sorcerer`): stored as general slots, so any feat fits. Find each one's list in the saved pages (the bloodline's bonus-feat list, the archetype's own, or Crossblooded's two bloodlines' lists together), and give the slot that list.
2. **Barbarian totem rage powers** (`barbarian#rage-powers`): one totem line per character. Only the Lesser → plain → Greater chain is checked. Make the totems exclusive with the `Not` prerequisite (ticket 32) or an option group, whichever the data already does for similar cases.
3. **Kineticist wild talents' bonus feats** (`kineticist#kineticist-bonus-feat`): see the item; model what the saved pages state, if the current types can hold it.
4. **Eldritch Scoundrel's sneak attack** (`rogue#sneak-attack-eldritch-scoundrel`): read from prose as dice at 1, 3, 7, 11, 15, 19. Check the reading against the saved page.
5. **Shaman's Wandering Hex** (`shaman`): a hex from another spirit. Model it if the saved pages say enough.
6. **Extra Slayer Talent** (`feats#extra-slayer-talent`): "Slayer Talents or Talented Slayer"; nothing named Talented Slayer is on a saved page.
7. **Cleric's deity alignment** (`cleric`): "within 1 place of their chosen deity". Deities aren't data.

Data from the saved guide pages only (offline; never fetch GameFAQs or fextralife). For each item: if the saved pages settle it, build it; if they don't, don't guess: write a numbered question for the user in this ticket with what the pages do say and a recommendation, and go on to the next item. Items 6 and 7 are likely questions (7 may well stay out of scope: deities aren't modelled).

## Build

AFK, test-first (`tdd`). The only agent in the repo, so it may edit the one migration and reset the DB per `CLAUDE.md` if a schema change is needed. As in tickets 30–38: the rules go in an extractor module, applied to `game-data/` by a one-off scratchpad program (never re-run the extractor over `game-data/`); a fresh extraction must be byte-identical to the rewrite; real-data tests first (red on the current data); a before/after proof that nothing else moved; each item settled is removed from `EXTRACTION_REPORT.md` by the extractor.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; every item is either built (with a real-data test showing the slot, lock or row now matches the guide: e.g. an Empyreal Sorcerer's bonus feat slot no longer offers Power Attack, a barbarian with one totem's power can't take another totem's) or written up as a question; the report's counts match.

## Progress log

- 2026-11-02: started. Baseline: HEAD 8e3727d, clean tree; scratch in `t43/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t43/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (byte-identical to `t38/extract-new/`: no extractor or data commit since ticket 38). Read every item against the saved pages (text dumps in the scratchpad's `pages/`):
  - **Settled by the pages:** the totems (barbarian#rage-power says they're exclusive, a line closing off the others), the Eldritch Scoundrel's dice (its section lists them: 1st level, 3rd, then 7, 11, 15, 19; `game-data/` already matches), the Wandering Hex (a hex slot "temporarily filled" from the main spirit or the current Wandering Spirit: a daily choice, as the Wandering Spirit is).
  - **Not settled:** the sorcerer archetypes' level-1 "Bonus Feat" (the guide never says what it is, and its sentences pull both ways), the kineticist's bonus-feat wild talent (which elements' lists, how often, and wild talents aren't data), Talented Slayer (on no saved page), the cleric's deity (the Deities and Domains page wasn't saved). Questions below.
  - Totems: `Not` per power, not an option group. A group (`NoOtherOptionOfGroup`) allows one option of the group, which would forbid a line's own chain; the totems need "nothing from another line". `All([Not(SelectionOption(..)) ×9, …])` rather than `Not(Any([..]))`, so the lock names the power held ("Needs not Beast Totem, Lesser") instead of listing all nine. No schema, `rules` or `Build` change.
- 2026-11-02: real-data tests first (`game_data.rs`, 3 new): the totems (each power rules out exactly the other lines' nine, no other rage power rules anything out; a barbarian with Beast Totem, Lesser at 2 is offered no other line's power at 4 and the Beast chain at 6 and 10; Fiend Totem, Lesser after it is locked with the reason "Needs not Beast Totem, Lesser"; with no totem every line is open); the Eldritch Scoundrel's dice by level (1 to 20: 1, 1, 2, … 6, 6) and no `rogue/sneak-attack`; the Wandering Hex a daily choice from shaman 6 (not 5), and a Possessed Shaman's from 14 (not 13). Red run on HEAD's data (`t43/real-data-red.txt`): the totem and Wandering Hex tests fail, each for the reason meant; the Eldritch Scoundrel test passes (it pins data that already matched the guide).
- 2026-11-02: data and extractor. New `extract::report_gaps` (`apply`, `settled`): each rule asserts its guide sentence and the data's shape (four lines of three powers, no `Not` there yet; the dice levels parsed from the sentence's brackets, at most one die a row and none off those levels; the Wandering Hex on shaman rows 6 and 14, not yet daily, the Wandering Spirit daily). `extract` runs it after `option_class_skills::apply` and drops the totems' report item. `t43/rewrite/` (from `t38/rewrite/`, calling `report_gaps::apply`) wrote `game-data/` byte-identical to its dry run (`t43/dry/game-data/`): 13 changes in 2 files (barbarian, shaman; `t43/game-data-log.json`), the Eldritch Scoundrel a no-op, 0 check problems. Real-data tests green (125, `t43/real-data-green.txt`). A fresh extraction (`t43/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t43/rewrite/` (`t43/dry/extract-old-rewritten/`, 18 changes in 3 files: the same 13, plus the Eldritch Scoundrel's dice at 3, 7, 11, 15 and 19, which an extraction lacked); its report differs only by the totems' item (33 → 32). The barbarian's and shaman's changes are line for line the same as `game-data/`'s. New `extract/tests/extract/report_gaps.rs` (1: only the totems' item is settled). `cargo test -p extract` 89 pass.
- 2026-11-02: proof, docs, DB, Verification. Proof (`t43/proof/` on `t43/proof-common.rs`, ticket 38's Builds plus `gap_builds`; one program, since ticket 43 changes no rules, walked over `old-src/game-data` and the new data; `t43/compare.py`, 88881 checks) passes. Docs: `game-data/README.md` (ticket 43's paragraph), `game-data/EXTRACTION_REPORT.md` (12 → 9 items; the 9 left are the questions). Reseeded the dev DB (no schema change, so no reset). Verification below.

## Questions for the user

1. **The sorcerers' level-1 "Bonus Feat"** (`sorcerer#crossblooded`, `#empyreal-sorcerer`, `#geomancer`, `#nine-tailed-heir`, `#sage-sorcerer`, `#sylvan-sorcerer`; 6 report items). What the pages say:
   - Every sorcerer table (the nine bloodlines' and every archetype's but the Overwhelming Mage's) has "Bonus Feat" at level 1, in the column that holds "Bloodline Feat" at 7, 13 and 19. No section explains the level-1 one.
   - character-creation#bloodlines names only 7, 13 and 19 as the levels a sorcerer picks a bonus feat related to the bloodline.
   - The Overwhelming Mage "does not receive bonus feats" and gives up four feats; its table has Overwhelming Penetration at 1, 7, 13 and 19 in that column. So the level-1 feat is real, and counted with the bloodline ones.
   - The Nine Tailed Heir and the Overwhelming Mage choose a standard bloodline "for the purpose of bonus feats"; Crossblooded picks "at the bonus feat levels" from one bloodline or the other.
   - The Geomancer "does not gain bloodline spells or bloodline feats" (no slot at 7, 13 or 19) but keeps "Bonus Feat" at level 1.
   - The data: the base sorcerer has no level-1 slot at all (the bloodline tables' cell never became one); the six archetypes have a General slot (any feat, so Power Attack); the Seeker has a Magic-feat slot at 1 (from its "Bonus Feat" section, which is about its 5, 10 and 15).

   Does the game give every sorcerer a bonus feat at level 1, and from which list? **Recommendation:** if you can check in the game, go by it. Otherwise: every sorcerer (base included) takes it from the held bloodline's bonus-feat list (`OptionList(selection: "sorcerer/bloodline")`, ignoring nothing, as at 7/13/19), so Crossblooded picks from either bloodline's list, the Empyreal, Sage and Sylvan from their fixed bloodline's, and the Seeker and Nine Tailed Heir from their chosen one's; the Geomancer, whose text denies it bloodline feats, keeps a General slot until you say otherwise. That reading fits the Overwhelming Mage's four feats and the "for the purpose of bonus feats" wording; the character-creation sentence is the one it contradicts.

2. **The kineticist's bonus-feat wild talent** (`kineticist#kineticist-bonus-feat`). What the pages say: a level-1, Universal wild talent lets the kineticist select a bonus feat associated with an Elemental Focus, ignoring its prerequisites. Wild talents are picked at 2nd and every even level, from talents whose level is at most half the class level, each limited to its elements (`kineticist#wild-talents`). Each Elemental Focus section lists five "Wild Talent bonus feats" (already data, one list per element). In the data, the even-level "Wild Talents" is a feature, not a pick, so nothing can offer this talent. Not settled: whether the feat comes from the held elements' lists or any element's, and whether the talent can be taken more than once. **Recommendation:** a follow-up ticket that makes wild talents a selection (the guide's 26 talents with their level and element conditions; the four defensive talents stay the element's 2nd-level grant), with the bonus-feat talent taken once (the guide doesn't say it repeats) and granting a slot from the held elements' lists (`OptionList(selection: "kineticist/element")`, which with Expanded Element offers both elements' lists).

3. **Talented Slayer** (`feats#extra-slayer-talent`). The prerequisite reads "Slayer Talents or Talented Slayer"; nothing named Talented Slayer is on any saved page, so the whole line is left out and Extra Slayer Talent has no prerequisite (anyone can take it). **Recommendation:** use the half the pages name: Slayer Talents, read as slayer level 2 (`ClassLevel(class: "slayer", level: 2)`, the level the slayer's table first gives Slayer Talents; "any slayer talent held" can't be told apart from a rogue's, since the slayer shares the rogue talents' keys). Add Talented Slayer as an `Any` alternative if you know what it is.

4. **The cleric's deity alignment** (`cleric`). The page says the cleric's alignment must be within one step of the chosen deity's. The guide's Deities and Domains page is in its table of contents but wasn't saved (ticket 08), and the planner has no deities, so there's nothing to check against. **Recommendation:** keep deities out of scope: drop the report item and add a line to the map's Out of scope ("Deities: a cleric's deity and its alignment and domain limits"). The alternative is to save that page and add a deity choice to the Build (a schema change and a new link vector).

## Answer

Done 2026-11-02, uncommitted. Of the report's 7 items, 3 are built (the totems, the Eldritch Scoundrel's dice, the Wandering Hex) and 4 are questions above (the sorcerers' level-1 bonus feat, the kineticist's bonus-feat wild talent, Talented Slayer, the cleric's deity). No schema, `rules` or `Build` change, so the v1 link vector stays.

### Data

`extract::report_gaps` asserts each guide sentence and the data's shape before changing anything:

- **Totem rage powers:** each of the 12 (Beast, Celestial, Daemon, Fiend: Lesser, plain, Greater) needs `Not(SelectionOption(..))` of each of the other lines' nine, in an `All` beside what it needed (the Beast chain's level and earlier power). One `Not` per power, so a locked power's reason names the one held: "Needs not Beast Totem, Lesser". A barbarian (or a Primalist through its rage powers) holds one line. The Celestial, Daemon and Fiend lines still have no chain: the guide gives prerequisites only for the Beast line.
- **Eldritch Scoundrel:** its section gives a die at 1st level, at 3rd, then at 7, 11, 15 and 19, which is what `game-data/` held. No change there; an extraction now has the same dice (it had only the 1st).
- **Wandering Hex:** `daily_choice: true` on `shaman/wandering-hex` (base 6 and 14; the Possessed Shaman 14; the Spirit Warden and Witch Doctor as the base). The sheet shows it as a daily note, as it does the Wandering Spirit.

`t43/rewrite/` (in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t43/`) ran `report_gaps::apply` over `game-data/`: 13 changes in 2 files (barbarian, shaman), byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

Real-data tests (`game_data.rs`, 3 new; red on HEAD's data first, `t43/real-data-red.txt`; the Eldritch Scoundrel test was green from the start, since the data already matched):

- the totems: each power rules out exactly the other lines' nine; a barbarian with Beast Totem, Lesser isn't offered another line's power and keeps his chain; Fiend Totem, Lesser after it is locked with "Needs not Beast Totem, Lesser";
- an Eldritch Scoundrel's sneak attack dice by level, 1 to 6, at 1, 3, 7, 11, 15 and 19;
- a shaman's Wandering Hex is a daily choice from 6, a Possessed Shaman's from 14.

### Extractor

`extract` runs `report_gaps::apply` after `option_class_skills::apply` and drops the totems' item from its report (`report_gaps::settled`). A fresh extraction (`t43/extract-new/`) is byte-identical in its RON files to HEAD's extraction rewritten by `t43/rewrite/` (18 changes in 3 files: the 13, plus the Eldritch Scoundrel's 5 later dice); its report differs by the totems' item only (33 → 32). `extract/tests/extract/report_gaps.rs`: 1 test.

### Proof

`t43/proof/` builds `t43/proof-common.rs` (ticket 38's Builds, plus a barbarian 20 on each totem line, one holding two lines, a Primalist bloodrager, an Eldritch Scoundrel 20 and three shamans 14) against the unchanged rules, and walks it over `old-src/game-data` and the new data. `t43/compare.py` (88881 checks pass):

- **Data:** the old data, with the 12 totems' `Not`s added and the Wandering Hex a daily choice, is exactly the new data; nothing outside the classes changed.
- **Walk:** all 6347 lines are identical but for: the Wandering Hex among the daily choices on the 71 lines (7 Builds) that hold it; 261 totem options that close in a rage power slot, in the 4 one-line barbarians, each a power of a line the Build doesn't hold; and the Build holding Beast and Fiend Lesser, now impossible (a stale link: its walk doesn't hold Fiend Totem, Lesser, and its pickers lock every option, 33 totem and 344 others).

### Docs

- `game-data/README.md`: ticket 43's paragraph.
- `game-data/EXTRACTION_REPORT.md`: 12 → 9 items (Feats 1, Base classes 8); the 9 are the questions above.
- `GLOSSARY.md`: unchanged (no new term).

### Verification

- `cargo test --workspace`: pass (rules 307 + unit 2, extract 89, data files 25 + game_data 125 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (no query file changed).
- No schema change, so no DB reset: `cargo run -p data --bin seed` reseeded `wotr-planner-postgres` on 5434 only.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 39 backgrounds.
- Rows: `prerequisite_node` 2007 (226 more: 108 `not` nodes and their 108 leaves, and 10 new `all` roots); `feature` with `daily_choice` 9 (was 8).
- The v1 link vector is unchanged (no `Build` change).
- Proof: passes (above).

### Open questions

The four questions above.

## Decisions on the questions

**User, 2026-11-02:** all four recommendations.

1. Every sorcerer, base included, takes the level-1 bonus feat from the held bloodline's bonus-feat list; Crossblooded from either bloodline's; the Empyreal, Sage and Sylvan from their fixed bloodline's; the Seeker and Nine Tailed Heir from their chosen one's (the Seeker's Magic-feat slot at 1 is replaced); the Geomancer keeps a General slot. Built by [ticket 44](44-sorcerer-feat-slayer-deities.md).
2. Wild talents become a selection, in a follow-up: [ticket 45](45-kineticist-wild-talents.md).
3. Extra Slayer Talent needs slayer level 2. Built by ticket 44.
4. Deities stay out of scope; the cleric item leaves the report. Done by ticket 44.
