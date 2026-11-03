# Monk ki powers become a selection

Type: task
Status: resolved
Blocked by: 57
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): the monk's ki powers (`monk#ki-power`, at 4 and every 2 levels after) are a feature in the data, not picks. Make them a selection from the guide's list, each with its level and prerequisites, as [ticket 45](45-kineticist-wild-talents.md) did the wild talents. Monk archetypes that change or replace ki powers follow their own sections. If a feat (such as an Extra feat) names ki powers, check whether its guide entry gives a pick and wire it per ticket 49.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have a monk of 4 or more differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: a monk 4 has one ki power slot offering only the powers of level ≤ 4; a power of a higher level is locked with a reason and opens at its level; every monk archetype's ki power levels match its table; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-02: started. Baseline: HEAD 81cb13c (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t58/main-rs-user-edit.diff` and left alone); scratch in `t58/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t58/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 57's fresh extraction). Read `monk#ki-power` ("At 4th level and every 2 levels thereafter a Monk may select a ki power"), the list `monk#ki-powers` (14 powers, `Lvl | Ki Power | Cost`, "The Monk must be a certain level to unlock them"; 7 have a section of their own), every monk table's "Ki Power" cells (the Scaled Fist has none at 12, the Sohei none at 4, the Student of Stone none at 6 or 12, the Zen Archer none at 10) and the Traditional Monk's ("Gains fixed Ki Powers": "Ki Power: Wholeness of Body" at 6, Abundant Step 8, Diamond Body 10, Diamond Soul 12, Quivering Palm 14, Perfect Self 20). In the data, `monk/ki-power` is a feature nothing names, and the Traditional Monk's rows 6–14 were the base class's, so it held that feature there. The feats page names no ki power (Extra Ki: "Your ki pool increases by 2"), nor does the mythic page (Abundant Ki Pool).
- 2026-11-02: real-data tests first (`game_data.rs`, 5 new): red on HEAD's data (`t58/real-data-red.txt`: 172 pass, 5 fail, each at the missing selection, slot or Traditional Monk feature).
- 2026-11-02: no `rules`, schema, `Build` or codec change was needed (a selection with `ClassLevel` options is the mercies' shape). `git diff HEAD -- libs/rules libs/seeds/src libs/seeds/migrations libs/ui apps/frontend` is empty.
- 2026-11-02: data and extractor. New `extract::ki_powers` (`apply`, `read_powers`), run after `exploits::apply`. `t58/rewrite/` wrote `libs/game-data/` byte-identical to its dry run: 52 changes in 1 file (`classes/monk.ron`; `t58/game-data-log.json`), 0 check problems; a second run refuses. The first version linked the 7 powers with no section to `monk#ki-powers`, which `no_selection_option_is_a_copy_of_another` caught (seven options, one anchor); they link to the page now, the data's rule for an option with no heading. Real-data tests green (177, `t58/real-data-green.txt`). A fresh extraction (`t58/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten by `t58/rewrite/` (`t58/dry/extract-old-rewritten/`): the same 52 changes, and the changed lines are those in `libs/game-data/` but where the inputs already differed (the Sohei's and the Student of Stone's hand-fixed bonus feats). `extract/tests/extract/ki_powers.rs`: 1 test.
- 2026-11-02: proof, UI, docs, DB, Verification. Proof: `t58/proof/` (ticket 57's program plus ticket 58's Builds, each Build's monk levels, the class pickers also without the monk's entries, ticket 48's shortcut checked on ticket 55's, 57's and 58's Builds), `t58/compare.py`, `t58/compare.txt`. UI: `cargo leptos watch` on 3900, headless Chromium (`t58/links/`, `t58/ui-check.js`, `t58/ui-check.txt`, `t58/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, the map. Reseeded (no schema change). Verification below.

## Answer

Done 2026-11-02, uncommitted. Ticket 54's decision 2 for the monk is built: the ki powers are a selection, each archetype picks where its table says, and the Traditional Monk holds its fixed powers instead of picking. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::ki_powers` asserts each guide sentence and the data's shape before changing anything:

- **The selection** `monk/ki-power` ("Ki Power") in place of the "Ki Power" feature, which goes, on the base class's rows 4 to 20 (every 2) and the archetype rows that held it.
- **14 ki powers** (`monk/ki-power/barkskin`, …), each needing `ClassLevel(monk, its level)`: Barkskin, Scorching Ray, Sudden Speed, True Strike (4); Abundant Step, Poison, Restoration, Wholeness of Body (8); Spit Venom (10); Diamond Body (12); Cold Ice Strike, Diamond Soul, Shout (14); Quivering Palm (16). The ki cost isn't stored. A power with a section of its own links to it (`monk#sudden-speed`); the 7 without one link to the page.
- **The archetypes**, by the rows in force: the Drunken Master, Quarterstaff Master and Sensei at 4 to 20; the Scaled Fist not at 12 (Dragon Breath); the Sohei from 6; the Student of Stone at 4, 8, 10, 14, 16, 18, 20; the Zen Archer not at 10.
- **The Traditional Monk** picks none. Its rows 6, 8, 10, 12 and 14 (new rows; they were the base's) hold the base row less the pick, plus its own feature for the table's power: `monk/traditional-monk/ki-power-wholeness-of-body` ("Ki Power: Wholeness of Body"), `…-abundant-step`, `…-diamond-body`, `…-diamond-soul`, `…-quivering-palm`, each linking to the power's section. Its bonus feats, Stunning Fist: Sicken and Ki Strike - Lawful stay. Perfect Self at 20 is the base class's feature already.
- **No feat gives a ki power.** Extra Ki and Abundant Ki Pool grow the pool; no section of the feats or mythic pages names ki powers, and nothing needed the feature, so ticket 49's rule has nothing to wire.

`t58/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t58/`) ran `ki_powers::apply` over `libs/game-data/`: 52 changes in 1 file, byte-identical to its dry run, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change was needed. The sheet labels a selection slot with the selection's name: "Ki Power". Ticket 48's shortcut: no feat gains an extra pick, and the new options name no feat.

### Tests

Real-data tests (`game_data.rs`, 5 new; red on HEAD's data first):

- the selection's 14 options and each one's `ClassLevel`; no "Ki Power" feature on any row; base rows 4 to 20; no feat's extra pick names it;
- every archetype's sheet has its ki power slots at its table's levels (the Traditional Monk none); filled, each is possible but the Drunken Master's (question 1);
- a monk 4 has one slot, "Ki Power", offering 14 and leaving open exactly the four of level 4; a monk 3 has none;
- Abundant Step reads "Needs Monk level 8 (have 4)" at 4 and is open at 8, Quivering Palm "Needs Monk level 16 (have 14)" until 16; every power opens exactly at its level; Diamond Body picked at 4 is why the Build is impossible ("Needs Monk level 12 (have 4)"); a power taken is "Already taken" in the next slot, and twice is impossible;
- a Traditional Monk 20 has no slot and holds each fixed power from its level on, with the rest of the base rows.

Extractor: `ki_powers.rs`, 1 test of `read_powers` (a power's level, and its section only where the guide gives one).

### Proof

Ticket 58 changes no rules, so one program (`t58/proof-run`, the working tree's rules) runs on HEAD's data (A) and on ticket 58's (C). It's ticket 57's proof over 1,325 Builds (1,326 in C): ticket 47's, 49's, 54's, 55's and 57's, the benchmark's (`t56/bench/links.json`), the full Builds and their switches, plus 31 ticket-58 Builds (a monk 3, 4, 8 and 20 and each archetype's 20, a monk 4 / rogue 4, each empty and filled; a monk 16 holding Barkskin, Abundant Step and Quivering Palm; four Builds no sheet holds). `t58/compare.py` (`t58/compare.txt`): 0 problems.

- **A checks the program:** stripped of the added field, A equals ticket 57's proof output (same rules, same data) on all 1,254 Builds.
- **The Builds without a monk of 4 or more** (1,243) are byte-identical, class pickers included.
- **The 82 that differ** all have a monk of 4 or more (54 bases). Where the picks are the same and the Build is possible alike (24 Builds), every slot both have answers the same, and the 170 new slots are all ki power slots (14 options). The monk 16 holding three powers is impossible in A (no slots) and possible in C; the four meant to be impossible are impossible in both (Abundant Step at 4, Barkskin twice, a Traditional Monk's or a Sohei's ki power where it picks none). The Drunken Master 20 is impossible in both (question 1). The rest are full Builds, whose fill differs from the first ki power slot on.
- **Ticket 48's shortcut stays exact:** on ticket 55's, 57's and 58's Builds (and their full versions), each of 240,774 feat options a feat slot accepts is open exactly when `allows` keeps the Build with it picked (238,240 on HEAD's data, also exact).
- Run time: 70 s on A, 77 s on C (8 threads).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t58/ui-check.js`, `t58/ui-check.txt`):

- A monk 8's link opens with "KI POWER Sudden Speed" at 4, "KI POWER Barkskin" at 6 and "KI POWER Abundant Step" at 8.
- A monk 8 with no picks shows "KI POWER +" at 4, 6 and 8; the one at 4 opens "Ki Power · Level 4" with Barkskin, Scorching Ray, Sudden Speed and True Strike, and "Show 10 locked" lists the others with their reasons ("Abundant Step Needs Monk level 8 (have 4)", "Quivering Palm Needs Monk level 16 (have 4)").
- A Traditional Monk 14's link opens with no "Ki Power" slot.
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 58's paragraph.
- `GLOSSARY.md`: unchanged (it names no class's selections).
- The map: the Decisions-so-far line.

### Verification

Ticket 53's Done when (`t58/verify.txt`):

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 98, rules 338 + unit 4, seeds files 25 + game_data 177 + postgres 28, server 4).
- `cargo test --workspace -- --list` (`t58/test-list.txt`, 674) differs from ticket 57's (668) only by the 6 new tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass. `cargo leptos build --release`: pass.
- `cd libs/seeds && cargo sqlx prepare --check`: pass (no query changed).
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- No schema change, so no DB reset. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only. Rows: `feature_selection` 118 (+1), `selection_option` 776 (+14), `prerequisite_node` 2120 (+14), `feature` 1921 (+4: the five fixed powers in, the Ki Power feature out).
- `cargo leptos watch` serves the sheet (UI above).
- The acceptance grep (`\bdata::`, `\bapp::`, `-p data`, `"app/` outside `.scratch/`) finds nothing.
- The v1 link vector is unchanged (no `Build` or format change).
- Proof: passes (above). Browser: passes (above).
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t58/main-rs-user-edit.diff`).

### Questions for the user

1. **No Drunken Master can be built.** The monk's "Alignment must be Lawful" and the Drunken Master's "Cannot be Lawful" are both checked when the class is taken, so no alignment meets them; HEAD's data is the same (the proof's Drunken Master 20 is impossible on both). This ticket's tests read its slots from the sheet without asking it to be possible. **Recommendation:** a follow-up ticket in which an archetype's alignment requirement replaces its class's (the guide's archetype section is the more specific), checking every archetype for the same clash.
2. **The Traditional Monk's fixed powers are features, not the selection's options.** Nothing names a ki power, so holding "Diamond Body" as a feature or as the option makes no difference today. **Recommendation:** keep them features; revisit only if something comes to need a ki power held.
3. **The Instinctual Warrior's monk rage powers.** `barbarian#rage-power-instinctual-warrior` lets it take Deflect Arrows, Purity of Body (8th), Diamond Body (12th), Diamond Soul (16th) and Perfect Self (20th) as rage powers; none is in `barbarian/rage-power`. **Recommendation:** a follow-up (or ticket 60, if you'd rather group borrowed picks) that adds them as Instinctual Warrior-only rage power options with those levels, as options of their own rather than the monk's ki powers, since the levels differ.

## Decisions on the questions

**User, 2026-11-02:** question 1: "drunken master can be built as long as you check the alignment per archetype and not on the base class". An archetype's alignment requirement replaces its class's: [ticket 63](63-archetype-alignment.md). Questions 2 and 3 wait for the end of the queue.
