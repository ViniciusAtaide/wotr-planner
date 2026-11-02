# Extra feats give their extra pick

Type: task
Status: resolved
Blocked by: 48
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 45](45-kineticist-wild-talents.md)'s decision 2 (user, 2026-11-02): the "Extra …" feats that grant another pick of a class selection give that pick. Today none does. From `libs/game-data/feats.ron`: Extra Arcana, Extra Arcanist Exploit, Extra Discovery, Extra Hex, Extra Mercy, Extra Rage Power, Extra Rogue Talent, Extra Slayer Talent and Extra Wild Talent (check each against its guide entry; the resource ones, such as Extra Ki, Extra Rage or Extra Channel, grant no pick and stay as they are). Each time the feat is taken (they repeat: `Times(3)` or `Unlimited`), the character gets one more slot of that selection, at the level the feat is taken.

Data from the saved guide pages only. If a feat's selection isn't clear (Extra Hex for a witch vs a shaman, Extra Rogue Talent for a slayer's or an Eldritch Scoundrel's talents), write a question in this ticket with a recommendation and go on.

## Build

AFK, test-first (`tdd`); the only agent in the repo. A feat that grants a selection slot is probably new to the types (feats grant feats and features today): add it test-first in `rules`, the schema (one migration, DB reset per `CLAUDE.md`) and the loader, then the data by the extractor-module-plus-rewrite method of tickets 43–45. The new slot needs a `SlotId` (the link format from [ticket 46](46-shorter-links.md) must carry it; no link has shipped, so regenerate the frozen vector if the format changes) and a label on the sheet.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a barbarian with Extra Rage Power at level 3 gets a rage power slot there and its option list is the barbarian's rage powers; taking it a second time gives a second slot; removing the feat removes the slot (and, by the removal rules, isn't offered while the slot's pick depends on it); a witch's Extra Hex gives a hex slot.

## Progress log

- 2026-11-02: started. Baseline: HEAD 81c16be (the user's uncommitted doc-comment edit in `apps/backend/src/main.rs` left alone); scratch in `t49/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t49/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 47's fresh extraction). Read the feats page's 19 "Extra …" entries: 9 say "gain one additional …" (Arcana, Arcanist Exploit, Discovery, Hex, Mercy, Rage Power, Rogue Talent, Slayer Talent, Wild Talent), the other 10 add uses or pool points. Each pick's selection is what the entry's "Prerequisites" names: one selection for six, two for Extra Discovery (the alchemist's, the Underground Chemist's) and Extra Hex (the witch's, the shaman's), two for Extra Rogue Talent (the rogue's talents, the Sylvan Trickster's Fey Tricks); "Mercy" is the paladin's feature (`paladin#mercy` lists mercies, but the data has no mercy selection), "Talented Slayer" is on no saved page. The types couldn't say "a feat gives a slot": feats grant feats only.
- 2026-11-02: rules, test-first (test world: Extra Rogue Talent names the rogue's and the slayer's talents, a new Extra Hex the witch's hex, an invented Combat feat Slayer Trick the slayer's talents): `extra_picks.rs`, 6 tests (red: `t49/rules-red.txt`, 325 pass, 6 fail, the frozen link among them; the "no slot of its selections" test passes on HEAD's engine, pinning it). Built: `Feat::extra_pick_of`, `SlotId::FeatSelection { level, index }` (codec group 8), the walker's pending slot per feat pick, of the selection it reached a slot of first, and the pickers always walking such a feat (the sixth test shows the shortcut's answer was wrong for one: Slayer Trick in a fighter's bonus slot takes the level's first extra slot from Extra Rogue Talent). `V1_LINK` regenerated (198 characters). Two world tests count the new feats. Green: 331 + 4 (`t49/rules-green.txt`).
- 2026-11-02: schema and DB: `feat_extra_pick (feat_key, selection_key)`, 42 tables; seed and load carry it. DB reset per `CLAUDE.md` (`wotr-planner-postgres` on 5434 only); `cargo sqlx prepare` replaced 1 query file (the TRUNCATE) and added 2. Postgres: 1 new test (round trip, a foreign key refusing an unknown selection).
- 2026-11-02: real-data tests first (`game_data.rs`, 7 new): red on HEAD's data (`t49/real-data-red.txt`: 151 pass, 6 fail, each at the missing `extra_pick_of` or slot; Extra Mercy's "no slot" passes, pinning it).
- 2026-11-02: data and extractor. New `extract::extra_picks` (`apply`), run after `talent_follow_ups::apply`: it asserts which feats' entries say "gain one additional", each one's "Prerequisites" and sentence, the selections those names are in the data, and that none has an extra pick yet. `t49/rewrite/` wrote `libs/game-data/` byte-identical to its dry run: 8 changes in 1 file (`feats.ron`; `t49/game-data-log.json`), 0 check problems; a second run refuses. Real-data tests green (157, `t49/real-data-green.txt`). A fresh extraction (`t49/extract-new/`) is byte-identical to HEAD's extraction rewritten by `t49/rewrite/` (`t49/dry/extract-old-rewritten/`; the same 8 lines as in `game-data/`); its report is HEAD's extraction's (23 items). The RON writer writes `extra_pick_of` after `repeat`; `extract/tests/extract/extra_picks.rs` 1 test.
- 2026-11-02: proof, UI, docs, DB, Verification. Proof (`t49/proof/`, ticket 48's with 45 ticket-49 Builds and each Build's Extra feats in its line; `t49/compare.py`, `t49/compare.txt`). UI: `cargo leptos watch` on 3900, headless Chromium (`t49/ui-check.js`, `t49/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, `GLOSSARY.md` (**Extra pick**, **Slot**'s order), the map. Reseeded. Verification below.

## Answer

Done 2026-11-02, uncommitted. Ticket 45's decision 2 is built: the eight Extra feats whose guide entry gives another pick of a class selection give one more slot of it each time they're taken, at the level they're taken. New in `rules` (a feat that gives a slot, a slot id for it), the schema (one table, DB reset) and the link format (`V1_LINK` regenerated, still version 1).

### Which feats give a pick, and of what

| Feat | Repeat | Selections ("Prerequisites") |
|---|---|---|
| Extra Arcana | 3 | `magus/magus-arcana` |
| Extra Arcanist Exploit | 3 | `arcanist/arcanist-exploits` |
| Extra Discovery | 3 | `alchemist/discovery`, `rogue/underground-chemist/discovery` |
| Extra Hex | 3 | `witch/hex`, `shaman/hex` |
| Extra Rage Power | any | `barbarian/rage-power` |
| Extra Rogue Talent | 3 | `rogue/rogue-talents`, `rogue/sylvan-trickster/fey-tricks` |
| Extra Slayer Talent | 3 | `slayer/slayer-talents` |
| Extra Wild Talent | 3 | `kineticist/wild-talents` |

The resource feats (Extra Arcane Pool, Bane Weapon, Bloodrage, Bombs, Channel, Ki, Lay On Hands, Performance, Rage, Reservoir) give none, and Extra Mercy gives none either (question 1).

### Rules

- `Feat::extra_pick_of`: the selections a pick of the feat gives one more slot of. The slot is of the one of them the walk reached a slot of first (a witch who becomes a shaman gets a witch's hex, question 2); a character who has had a slot of none gets none (a rogue 1, a wizard with Extra Hex; the pick of the feat itself still holds as before).
- The slot is `SlotId::FeatSelection { level, index }`: at the level of the feat's pick (a feat picked at a Mythic rank gives it at the rank's level), after the level's other slots and the options' rows' slots, numbered in the order the level holds those feats. Its kind is `SlotKind::Selection`, so the sheet labels it with the selection's name ("Rage Power", "Hex"), offers what that selection offers, and holds its pick through it (option levels and all).
- Removal: the existing rules hold. A pick in the feat's slot depends on the feat, so the feat isn't removable while the slot has a pick; clearing the pick frees it, and removing the feat removes the slot.
- Pickers (ticket 48's shortcut): `options` gives a feat no later check names the slot-left-empty answer. A feat with an extra pick isn't such a feat: its slot takes the level's next index from a later feat's, so its pick can move another's (a new engine test shows the shortcut's wrong answer). Every feat with an extra pick (8) is now always walked; the shortcut stays exact for the rest.
- Codec: group `8` FeatSelection (step, index), as `OptionFeat`'s. `V1_LINK` regenerated with a comment: the frozen Build gains a rogue talent in a FeatSelection slot (198 characters).

Engine tests (test world): 6 new in `extra_picks.rs`; `codec` (the generator and the frozen Build), `canonical` and `options` updated for the new world feats.

### Schema

`feat_extra_pick (feat_key, selection_key)`, both foreign keys, primary key the pair: 42 tables. Postgres test: 1 new (round trip, an unknown selection refused).

### Data

`extract::extra_picks` asserts each guide sentence and what it finds before changing anything; the eight feats' `extra_pick_of` are the selections named by their "Prerequisites" (the guide's "Dicovery" read as Discovery). `t49/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t49/`) changed 8 lines of `feats.ron`, byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

### Tests

Real-data tests (`game_data.rs`, 7 new, red on HEAD's data first):

- the eight feats name exactly the table's selections; the resource ones and Extra Mercy none;
- a barbarian with a rage power at 2 and Extra Rage Power at 3 gets a "Rage Power" slot at 3 offering the barbarian's 33 rage powers, as the slot at 2 does; filled, the Build is possible;
- taken at 3 and 5, it gives a slot at each;
- the feat isn't removable while its slot holds a pick, is once the pick is cleared, and takes the slot with it;
- a witch 1's Extra Hex gives a hex slot;
- each feat gives its class's slot: barbarian, witch, shaman, magus, arcanist, alchemist, Underground Chemist (rogue 11), rogue, Sylvan Trickster, slayer, kineticist, and a witch/shaman's Extra Hex is a witch's hex;
- Extra Mercy gives a paladin 3 no slot.

### Proof

`t49/proof/` is ticket 48's proof program (every slot's options with statuses, the class, path and switch pickers, `removable`, `allows` over the edit battery, `is_removable`, `removable_classes` and `allows_many` checked against them) over ticket 47's Builds, the benchmark's, 45 ticket-49 Builds (each case of the real-data table before the feat, with it, and filled), the full Builds and their switches: 1,118 Builds.

- **A** HEAD's rules on HEAD's data and **B** ticket 49's rules on HEAD's data: **byte-identical** (6,008,472 bytes).
- **B** vs **C** (ticket 49's rules and data): 1,059 Builds identical; the 59 that differ all pick a feat with an extra pick (in 49 base Builds). Where their picks are the same (16: the 14 ticket-49 `/ feat` Builds, ticket 44's slayer with Extra Slayer Talent and ticket 45's kineticist with Extra Wild Talent), only the sheet and the new FeatSelection slots differ: every other slot's options, the pickers, `removable` and `allows` are identical. The others are filled differently (full Builds whose generator now also fills an extra slot, the `/ filled` Builds). C has 71 FeatSelection slots in 59 Builds; every ticket-49 Build is possible but "Extra Rage Power at 1" (a barbarian holding no rage power yet), which is impossible on HEAD too.
- Run time is the same (40 s for each of A, B and C).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t49/ui-check.js`): a barbarian 5's link (Extra Rage Power at 3 and 5, each extra slot filled) opens with "FEAT Extra Rage Power" then "RAGE POWER Come and Get Me" at 3 and "RAGE POWER Guarded Stance" at 5 (`t49/ui-barbarian.png`); a witch 1's shows its empty extra "HEX +", whose picker opens as "Hex · Level 1" listing the hexes, 14 locked. No console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 49's paragraph.
- `GLOSSARY.md`: **Extra pick**; **Slot**'s checking order.
- The map: a Decisions-so-far line.

### Verification

- `cargo test --workspace`: pass (rules 331 + unit 4, extract 92, data files 25 + game_data 157 + postgres 27, server 4).
- `cargo clippy --workspace --all-targets`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass. `cargo leptos build --release`: pass (release wasm 1,474,277 B).
- `cd libs/seeds && cargo sqlx prepare --check`: pass (the TRUNCATE's query file replaced; the `feat_extra_pick` insert and select added).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source libs/seeds/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 38 backgrounds. `feat_extra_pick` has 11 rows; 42 tables.
- Ticket 22's acceptance grep finds nothing (over `libs/game-data`, `libs/seeds/tests/fixtures`, `libs/rules`, `libs/extract`, `libs/seeds/src`).
- The v1 link vector is regenerated (above).
- Proof: passes (above).

### Questions for the user

1. **Extra Mercy.** "You gain one additional mercy", but the paladin's mercies are a feature in the data (`paladin/mercy` at 3, 6, …), not picks, though `paladin#mercy` lists them (Fatigued, Shaken, … with their levels and prerequisites). Extra Mercy gives no slot. **Recommendation:** a follow-up ticket that makes the mercies a selection (as ticket 45 did the wild talents), and Extra Mercy's extra pick then comes for free (`extra_pick_of: ["paladin/mercy"]`).
2. **Extra Hex (or Discovery, Rogue Talent) with two of its selections.** A witch/shaman's slot is of the selection the character had a slot of first (a witch 1 / shaman 2 gets a witch's hex). The guide doesn't say which. **Recommendation:** keep it (the first class's); the alternative, a slot offering both selections' options, needs a new kind of slot.
3. **Extra Rogue Talent for a slayer.** A slayer who took a rogue talent through its slayer talents meets Extra Rogue Talent's prerequisite in the data (an option is held whichever selection took it), but has had no rogue talent slot, so the feat gives it no slot. An Eldritch Scoundrel's talents are the rogue's own selection, so its Extra Rogue Talent is a rogue talent. **Recommendation:** leave it: a slayer's extra talent is Extra Slayer Talent's; if you want, tighten Extra Rogue Talent's prerequisite to the rogue's own talents in a follow-up.
4. **Mythic Extra Feat and Extra Mythic Ability** ("You can select any normal feat"; "You can select a mythic ability") also give a pick, of a feat and of a mythic ability, not of a class selection, so they're out of this ticket. **Recommendation:** a follow-up ticket for them.
5. **The Arcane Enforcer's exploits** ("can choose an Arcanist Exploit at level 1 and every 3 subsequent levels") are a feature, not picks, so its Extra Arcanist Exploit gives no slot. **Recommendation:** with 1, a follow-up that makes such features picks.
