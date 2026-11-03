# Talents, hexes and discoveries taken through another class become picks

Type: task
Status: resolved
Blocked by: 59
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): several classes and archetypes pick from another class's list, but the data holds a feature with no slot. Give each a slot of an existing selection (the rogue's, the slayer's, the witch's, the alchemist's), or of one sharing their options as [ticket 25](25-shared-selection-options.md) did, whichever the guide's list matches:

- bard talents (`bard#bard-talents`) and skald talents;
- the Archaeologist's advanced talents;
- the Feyform Shifter's rogue talent at 8 and 14;
- the Vivisectionist's medical discoveries;
- the Sanctified Slayer's slayer talents at 8, 16, 17 and 20;
- the Hexcrafter's hex arcana and Hex Magus.

Where the slot is of a selection an Extra feat names, ticket 49's rule gives that feat's pick there too.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have one of the listed classes or archetypes differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: each listed class or archetype has its slots at its table's levels, offering the guide's list; an option taken through one class is not offered again through another where the selection is shared; an Extra feat that names the slot's selection gives its extra pick there; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-02: started. Baseline: HEAD 14fa582 (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t60/main-rs-user-edit.diff` and left alone); scratch in `t60/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t60/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 59's fresh extraction). Read `bard#bard-talents` ("The Bard can select from the Rogue Talent list"), `bard#advanced-talents`, `skald#skald-talents`, `shifter#rogue-talent` ("Note that you cannot choose Advanced Talents"), `alchemist#medical-discovery` (three lists), `inquisitor#slayer-talents`, `magus#hex-magus`, `magus#hex-arcana`, `rogue#rogue-talents` and `#advanced-rogue-talents` (20 and 16), the slayer's combat style note, the feats page's Extra entries, and every table's "Bard Talent", "Skald Talent", "Rogue Talent", "Medical Discovery", "Advanced Talents", "Slayer Talent", "Hex Magus", "Hex Arcana" and "Magus Arcana" cells; the data's rows in force already sit exactly there. The seven features nothing needs. Found: `rogue/rogue-talents` and `slayer/slayer-talents` are levelled by the rogue and the slayer only, so Combat Trick picked in a bard's or a Sanctified Slayer's slot would get no feat slot; the slayer's 2nd-or-4th combat style talent (`Not(slayer 6)`) would open to an inquisitor.
- 2026-11-02: real-data tests first (`game_data.rs`, 10 new; 4 extended: ticket 47's Combat Trick, ticket 49's two Extra-pick tables, ticket 44's Extra Slayer Talent): red on HEAD's data (`t60/real-data-red.txt`: 180 pass, 14 fail, each at a missing slot, selection, gate or extra pick).
- 2026-11-02: no `rules`, schema, `Build` or codec change was needed (shared options, option levels, `extra_pick_of`, `Archetype` and `ClassLevel` prerequisites already exist). `git diff HEAD -- libs/rules libs/seeds/src libs/seeds/migrations libs/ui apps/frontend` is empty.
- 2026-11-02: data and extractor. New `extract::borrowed_selections` (`apply`, `read_medical_discoveries`), run after `infusions::apply`. `t60/rewrite/` wrote `libs/game-data/` byte-identical to its dry run: 44 changes in 10 files (`t60/game-data-log.json`), 0 check problems; a second run refuses. Real-data tests green (194, `t60/real-data-green.txt`). A fresh extraction (`t60/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten by `t60/rewrite/` (`t60/dry/extract-old-rewritten/`: the same 44 changes); its changed lines are those in `libs/game-data/` (`t60/lines-extract.txt`, `t60/lines-game-data.txt`) but for the Sanctified Slayer's row 16, whose inputs already differed (its Sneak Attack, hand-added in ticket 19). `extract/tests/extract/borrowed_selections.rs`: 1 test.
- 2026-11-02: proof, UI, docs, DB, checks. Proof: `t60/proof/` (ticket 59's program plus ticket 60's Builds; `t60/patch_proof.py`), `t60/compare.py`, `t60/compare.txt`. UI: `cargo leptos watch` on 3900, headless Chromium (`t60/links/`, `t60/ui-check.js`, `t60/ui-check.txt`, `t60/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, the map. Reseeded (no schema change). Checks below (`t60/verify.txt`, `t60/test-workspace.txt`).

## Answer

Done 2026-11-02, uncommitted. Ticket 54's decision 2 for borrowed lists is built: each listed class or archetype picks where its table says, from the existing selection where the guide's list is one, or from a selection sharing the lists it mixes. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::borrowed_selections` asserts each guide sentence, every table's cells against the rows in force, and the data's shape before changing anything. Each feature below goes (nothing needed it):

| Who | Guide | Slots at | Selection |
|---|---|---|---|
| Bard and its 7 archetypes | "The Bard can select from the Rogue Talent list" | 2, 6, 10, 14, 18 (Beast Tamer 6, 14, 18) | `rogue/rogue-talents` |
| Skald and its 7 archetypes | "…from the Rogue Talent list at 2nd, 7th, 12th and 17th levels" | 2, 7, 12, 17 (Inciter 4, 9, 14, 19) | `rogue/rogue-talents` |
| Feyform Shifter | "can choose a Rogue Talent. … you cannot choose Advanced Talents" | 8, 14 | new `shifter/feyform-shifter/rogue-talent` ("Rogue Talent"), sharing the 20 non-advanced rogue talents |
| Vivisectionist | its three lists | 2, 4, …, 18 | new `alchemist/vivisectionist/medical-discovery` ("Medical Discovery"), sharing 14 discoveries, 8 rogue talents and 3 advanced ones |
| Sanctified Slayer | "can select a Slayer talent" | 8, 16, 17, 20 | `slayer/slayer-talents` |
| Hexcrafter, Hex Magus | "may select a Witch hex" | 4 | `witch/hex` |
| Hexcrafter, Hex Arcana | "Besides the usual Magus Arcana abilities, a Hexcrafter can select a hex" | 6, 9, 12, 15, 18 | new `magus/hexcrafter/hex-arcana` ("Hex Arcana"), sharing the 15 magus arcana and 26 witch hexes |

Also:

- **Advanced talents.** The Archaeologist's Advanced Talents (10) joins the `Any` of all 16 advanced rogue talents; the Vivisectionist's (10) that of its three (Confounding Blades, Crippling Strike, Dispelling Attack). Both features stay.
- **Major and grand hexes.** "From 12th level … major hexes and at 18th level … a grand hex": the 9 major and 4 grand hexes' `Any` gains `All([ClassLevel(magus, 12 or 18), Archetype(magus/hexcrafter)])`.
- **Option levels.** Combat Trick's Combat feat comes from its option row, which needs an option level from the classes its holding selection names. `rogue/rogue-talents` is now levelled by the bard, rogue and skald, and `slayer/slayer-talents` by the inquisitor and slayer. Every row those options have is at level 1 (asserted), so this changes only whether the row comes at all, never which row (question 1).
- **Extra feats** (ticket 49's rule):
  - Extra Rogue Talent reaches the bard and the skald through `rogue/rogue-talents`. Its "Prerequisites: Rogue Talents" names the Feyform Shifter's "Rogue Talent" too, so its `extra_pick_of` gains that selection.
  - Extra Hex reaches the Hexcrafter through `witch/hex`.
  - Extra Slayer Talent's "Prerequisites: Slayer Talents" names the Sanctified Slayer's own "Slayer Talents" section, so it is now `Any([ClassLevel(slayer, 2), All([ClassLevel(inquisitor, 8), Archetype(inquisitor/sanctified-slayer)])])`. Its slot is the slayer's.
  - Extra Discovery and Extra Arcana give the Vivisectionist and the Hexcrafter none (questions 2 and 3).
- **The slayer's combat style talents** are the slayer's windows ("he can only do so … at certain levels: 2 or 4, 6 or 8 and 10 or greater"). The 2nd-or-4th one needed only `Not(slayer 6)`, which would open it to a Sanctified Slayer; it now also needs `ClassLevel(slayer, 2)`. No slayer Build changes: a slayer's slots start at 2 (question 4).

`t60/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t60/`) ran `borrowed_selections::apply` over `libs/game-data/`: 44 changes in 10 files, byte-identical to its dry run, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change was needed. The sheet labels each slot with its selection's name: "Rogue Talents" (bard, skald), "Rogue Talent", "Medical Discovery", "Slayer Talents", "Hex", "Hex Arcana" (question 6). Ticket 48's shortcut is unaffected: the new options name no feat, and Extra Rogue Talent and Extra Slayer Talent were already always walked.

### Tests

Real-data tests (`game_data.rs`, 10 new and 4 extended; red on HEAD's data first):

- **The features and selections:** the seven features are gone from features and rows, and both Advanced Talents features stay. The three new selections offer exactly the guide's lists, with their option levels. The 16 advanced talents accept the Archaeologist; exactly the Vivisectionist's three accept the Vivisectionist.
- **Slot levels:** every listed class or archetype (the bard's and skald's 7 archetypes each) has its slots at its table's levels and, filled, is possible. There is no alchemist discovery slot for the Vivisectionist and no magus arcana slot for the Hexcrafter; other archetypes have none of these slots.
- **Bard and skald:** a bard 2 and a skald 2 are offered the 36 rogue talents, with Crippling Strike locked. For an Archaeologist 6 it reads "Needs Advanced Talents (Archaeologist)"; it opens at 10, and the Build holding it is possible.
- **Feyform Shifter:** its slot offers 20 talents, and no advanced one even for a rogue 10.
- **Vivisectionist:** its slot offers exactly its 25. Crippling Strike reads "Needs Advanced Talents (Vivisectionist)" at 2 and opens at 10; Blinding Strike isn't offered.
- **Sanctified Slayer:** at 8 it is offered the slayer's talents. Trapfinding and Combat Trick are open; the 2nd-or-4th combat style talent reads "Needs Slayer level 2 (have 0)". Extra Slayer Talent is locked at 7 and open at 9, where it gives a slayer talent slot.
- **Hexcrafter:** it has 26 hexes at 4 and 41 options from 6. Agony reads "Needs Magus level 12 (have 9)" and Death Curse "Needs Magus level 18 (have 15)"; each opens at its level.
- **Already taken across classes:** an option taken through one class is "Already taken" through another, and taking it twice is impossible. Cases: rogue/bard, bard/skald, rogue/Feyform, rogue/Vivisectionist, slayer/Sanctified Slayer, witch/Hexcrafter at 4 and at 6.
- **Extra feats:** ticket 49's table gains bard, skald, Feyform (Extra Rogue Talent), Sanctified Slayer (Extra Slayer Talent) and Hexcrafter (Extra Hex). Extra Discovery and Extra Arcana give a Vivisectionist and a Hexcrafter no slot.
- **Combat Trick** gives its Combat feat slot through every selection offering it, the new ones and a bard's, skald's and Sanctified Slayer's slots included.

Extractor: `borrowed_selections.rs`, 1 test of `read_medical_discoveries` (headings, names, "Greater / Grand Mutagen" read as two).

### Proof

Ticket 60 changes no rules, so one program (`t60/proof-run`, the working tree's rules) runs on HEAD's data (A) and ticket 60's (C). It is ticket 59's proof plus 65 ticket-60 Builds, 1,569 Builds in A and 1,576 in C counting their full versions and switches:

- each listed class and archetype at 20, and six multiclass pairs, each empty and filled;
- six with fixed picks;
- seven that no sheet holds.

`t60/compare.py` (`t60/compare.txt`): 0 problems.

- **A checks the program:** stripped of the added fields, A equals ticket 59's proof output on all 1,412 Builds both have.
- **Builds with no listed class or archetype (1,352):** 67 are byte-identical. In the other 1,285, each slot's options differ only by the lock reasons of Extra Slayer Talent and the 2nd-or-4th combat style talent; the open count, option count and kind are equal. In 4 of them the class picker also differs, only in the borrowing classes' entries. Their sheets are identical.
- **The 217 that differ** all take a listed class or archetype (137 bases). Where the picks are the same and the Build is possible alike (53 Builds), every slot both have answers the same, apart from Extra Slayer Talent opening to a Sanctified Slayer from 9. All 199 new slots are slots of the borrowed selections: 145 rogue talents, 20 medical discovery, 13 hex arcana, 10 slayer talents, 6 Feyform, 5 witch hex.
- **Fixed picks:** the six fixed-pick Builds are impossible in A (no slots) and possible in C. The seven meant to be impossible are impossible in both: Crippling Strike on a bard 2, a Feyform 8 and a Vivisectionist 2; the combat style talent on a Sanctified Slayer; Agony at 9; Fast Stealth twice; Cackle twice. In the last two, an earlier picker opens more in C, because their second pick now has a slot.
- **Ticket 48's shortcut stays exact:** on ticket 55's, 57's, 58's, 59's and 60's Builds (and their full versions), each of 698,947 feat options a feat slot accepts is open exactly when `allows` keeps the Build with it picked (658,400 on HEAD's data, also exact).
- Run time: 135 s on A, 143 s on C (8 threads).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t60/ui-check.js`, `t60/ui-check.txt`):

- A bard 10's link opens with "ROGUE TALENTS Combat Trick" (and its "COMBAT FEAT +") at 2, "FEAT Extra Rogue Talent · ROGUE TALENTS Iron Guts" at 3, "ROGUE TALENTS Fast Stealth" at 6, and "ROGUE TALENTS +" at 10.
- A Hexcrafter 12's link: "HEX Slumber" at 4, "HEX ARCANA Arcane Accuracy" at 6, "HEX ARCANA Cackle" at 9, "HEX ARCANA Restless Slumber" at 12.
- A Sanctified Slayer 8's link: "SLAYER TALENTS Trapfinding" at 8.
- A Feyform Shifter 8's empty "ROGUE TALENT +" opens "Rogue Talent · Level 8" with the 20 talents and no advanced one.
- A Vivisectionist 2's "MEDICAL DISCOVERY +" opens with its discoveries and talents. "Show 9 locked" gives the reasons ("Crippling Strike Needs Advanced Talents (Vivisectionist)", "Greater Mutagen Needs Alchemist level 12 (have 2)").
- A Hexcrafter 9's "HEX ARCANA +" opens with the 15 arcana and the open hexes. "Show 13 locked" gives the reasons ("Agony Needs Magus level 12 (have 9)", "Death Curse Needs Magus level 18 (have 9)").
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 60's paragraph.
- `GLOSSARY.md`: unchanged (its Feature selection and Extra pick entries hold).
- The map: the Decisions-so-far line.

### Checks

(`t60/verify.txt`, `t60/test-workspace.txt`)

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean (after one test-code lint fix). `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 100, rules 338 + unit 4, seeds files 25 + game_data 194 + postgres 28, server 4).
- `cargo test --workspace -- --list` (`t60/test-list.txt`, 693) differs from ticket 59's (682) only by the 11 new tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only. Rows: `feature_selection` 122 (+3), `selection_shared_option` 670 (+86), `prerequisite_node` 2308 (+64), `feature` 1913 (−7), `feat_extra_pick` 13 (+1).
- `cargo leptos build` and `--release`: skipped, as asked. `cargo leptos watch` serves the sheet (UI above).
- The v1 link vector is unchanged (no `Build` or format change).
- Proof: passes. Browser: passes.
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t60/main-rs-user-edit.diff`).

### Questions for the user

1. **Option levels for Combat Trick.** A pick in a bard's, skald's or Sanctified Slayer's slot gets its Combat Trick feat only if the selection is levelled by that class. `rogue/rogue-talents` is now levelled by the bard, rogue and skald, and `slayer/slayer-talents` by the inquisitor and slayer. Every option row there is at level 1, so a multiclass character's option level grows but nothing it gets changes. An inquisitor who isn't a Sanctified Slayer counts too; this is harmless, since it holds slayer talents only through slayer levels. **Recommendation:** keep it. The alternative is a selection of their own for the bard, the skald and the Sanctified Slayer, sharing the lists as the Feyform's does. That would drop Extra Rogue Talent's and Extra Slayer Talent's reach to them, as ticket 49's decision 3 did for the slayer.
2. **Extra Discovery for the Vivisectionist.** Its "Medical Discovery" replaces the alchemist's Discovery, but the feat's "Prerequisites: Discovery" names a different heading. A Vivisectionist holding a discovery meets the prerequisite and gets no slot. **Recommendation:** add `alchemist/vivisectionist/medical-discovery` to Extra Discovery's extra pick, as ticket 49 did the Underground Chemist's Discovery. Its slot then offers the Vivisectionist's list.
3. **Extra Arcana for the Hexcrafter.** Its Hex Arcana is "the usual Magus Arcana abilities" plus hexes, but the Hexcrafter never has a slot of `magus/magus-arcana`, so Extra Arcana gives it none. **Recommendation:** add `magus/hexcrafter/hex-arcana` to Extra Arcana's extra pick. Its slot would then offer hexes too, which is what Hex Arcana means; a slot of arcana only would need a rules change.
4. **The Sanctified Slayer's combat style talents.** Built as the slayer's own (they need slayer levels). The guide doesn't say whether its slots at 8/16/17/20 read the windows by inquisitor level. **Recommendation:** keep it. The windows and the ranger style are the slayer's; the Sanctified Slayer's section gives "a Slayer talent" only.
5. **Shared gates, ticket 25's trade-off.** The advanced talents' and the hexes' `Any`s are on the one shared option, so some multiclass characters may take them in another class's slot: a rogue 10 / bard 2 an advanced talent in the bard's slot (and an Archaeologist 10 / rogue 2 in the rogue's); a witch 2 / Hexcrafter 12 a major hex in the witch's slot. **Recommendation:** accept, as ticket 25's decision 2 did.
6. **The bard's and skald's slots read "Rogue Talents"**, the selection's name, where their tables say "Bard Talent" and "Skald Talent". **Recommendation:** keep it; the guide calls it the Rogue Talent list.

## Decisions on the questions

**User, 2026-11-02:** all recommendations ("Yes, run overnight": take every recommendation without asking).

1. The rogue's and slayer's selections stay levelled by the bard, skald and inquisitor too.
2. and 3. Extra Discovery gives the Vivisectionist a Medical Discovery slot, and Extra Arcana gives the Hexcrafter a Hex Arcana slot: [ticket 64](64-vivisectionist-hexcrafter-extra-picks.md).
4. The Sanctified Slayer gets no combat style talents.
5. The shared-gate trade-offs are accepted.
6. The bard's and skald's slot reads "Rogue Talents".
