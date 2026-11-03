# Kineticist infusions become a selection

Type: task
Status: resolved
Blocked by: 58
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): the kineticist's infusions (`kineticist#infusion`, at 3, 5, 9, 11, 13, 17 and 19, with each infusion's level and element in the guide's Infusions section) are a feature in the data, not picks. Make them a selection, as [ticket 45](45-kineticist-wild-talents.md) did the wild talents (level limit, element conditions, Expanded Element). Kineticist archetypes that change infusions follow their own sections.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have a kineticist of 3 or more differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: a kineticist 3 has one infusion slot offering only its elements' and universal infusions of allowed level; an infusion of a higher level is locked with a reason and opens at its level; a Fire kineticist isn't offered a Water-only infusion; every kineticist archetype's infusion levels match its table; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-02: started. Baseline: HEAD 3a2602d (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t59/main-rs-user-edit.diff` and left alone); scratch in `t59/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t59/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 58's fresh extraction). Read `kineticist#infusion` ("She gains new Infusions at 3rd, 5th, 9th, 11th, 13th, 17th and 19th levels"), the list `kineticist#infusions` (32 infusions under "Level 1" to "Level 8", each with `Element`, `Type` and sometimes `Prerequisites`; "The minimum Kineticist level to use a given Infusion is twice the Infusion's level"), every kineticist table's "Infusion" cells (the base and four archetypes also at 1; the Blood Kineticist at 3, 13, 17, 19, its own infusions fixed at 1, 5, 9, 11; the Kinetic Knight at 5, 11, 13, 17, 19; the Sharpshooter at 3 to 19), and `kineticist#infusion-kinetic-sharpshooter` (four infusions only it may use, six it cannot use). In the data, `kineticist/infusion` is a feature on those rows and the Sharpshooter has its own "Infusion (Kinetic Sharpshooter)" feature; nothing in `libs/game-data/` needs either. No feat or mythic pick gives an infusion (Over-Infused Blasts applies two at once; Throw Elixir's "Infusion" is the alchemist's discovery).
- 2026-11-02: real-data tests first (`game_data.rs`, 7 new): red on HEAD's data (`t59/real-data-red.txt`: 177 pass, 7 fail, each at the missing selection or slot).
- 2026-11-02: no `rules`, schema, `Build` or codec change was needed (a selection whose options need `ClassLevel`, `SelectionOption`, `Archetype` and `Not` is the wild talents' and mercies' shape). `git diff HEAD -- libs/rules libs/seeds/src libs/seeds/migrations libs/ui apps/frontend` is empty.
- 2026-11-02: data and extractor. New `extract::infusions` (`apply`, `read_infusions`), run after `ki_powers::apply`. The first dry run left the Sharpshooter's rows 17 and 19 equal to the base's (the data check's "leave the level out"); the module now drops such rows. `t59/rewrite/` wrote `libs/game-data/` byte-identical to its dry run from HEAD's data: 64 changes in 1 file (`classes/kineticist.ron`; `t59/game-data-log.json`), 0 check problems; a second run refuses. A fresh extraction first failed on Throw Elixir: an extraction reads its "Infusion" as both kineticist features too (ticket 20 fixed that by hand in `libs/game-data/`), and the Sharpshooter's feature goes, so the module gives an extraction's Throw Elixir ticket 20's prerequisite (a no-op on `libs/game-data/`). Real-data tests green (184, `t59/real-data-green.txt`). A fresh extraction (`t59/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten by `t59/rewrite/` (`t59/dry/extract-old-rewritten/`: 65 changes in 2 files, the 64 plus Throw Elixir); its kineticist lines that changed are exactly those in `libs/game-data/` (`t59/lines-extract.txt`, `t59/lines-game-data.txt`). `extract/tests/extract/infusions.rs`: 1 test.
- 2026-11-02: proof, UI, docs, DB, checks. Proof: `t59/proof/` (ticket 58's program plus ticket 59's Builds, each Build's kineticist levels, the class pickers also without the kineticist's entries, ticket 48's shortcut also on ticket 59's Builds; `t59/patch_proof.py`), `t59/compare.py`, `t59/compare.txt`. UI: `cargo leptos watch` on 3900, headless Chromium (`t59/links/`, `t59/ui-check.js`, `t59/ui-check.txt`, `t59/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, the map. Reseeded (no schema change). Checks below.

## Answer

Done 2026-11-02, uncommitted. Ticket 54's decision 2 for the kineticist is built: the infusions are a selection, each archetype picks where its table says, and the Kinetic Sharpshooter's section decides which infusions it may and may not take. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::infusions` asserts each guide sentence and the data's shape before changing anything:

- **The selection** `kineticist/infusions` ("Infusion", guide `kineticist#infusion`) in place of the "Infusion" feature on the rows that held it after 1st level: the base class's 3, 5, 9, 11, 13, 17, 19 and the archetype rows in force. Keyed apart from the feature, which stays on the 1st-level rows (question 1).
- **32 infusions** (`kineticist/infusions/<name>`), each needing:
  - `ClassLevel(kineticist, 2 × its level)`;
  - one of its elements: `SelectionOption("kineticist/element/…")`, an `Any` for several, none for Universal; an option held through any selection counts, so Expanded Element's element (and the Blood Kineticist's Water) opens its infusions;
  - one of the options its "Prerequisites" names: Eruption, Deadly Earth, Cloud and Fragmentation need Extended Range, Blade Whirlwind Kinetic Blade, Foxfire Infusion the Foxfire wild talent, Grappling Infusion one of Cloud, Deadly Earth and Wall (question 4).
  Level 1: Burning, Dazzling, Fan of Flames (Fire), Pushing (Air, Earth, Water), Exploding Arrows, Extended Range, Kinetic Blade; 2: Bowling (Earth), Entangling (Earth, Water), Magnetic (Air, Earth), Spindle; 3: Eruption, Foxfire Infusion (Fire), Rare-Metal (Earth), Synaptic (Air), Torrent (Air, Fire, Water); 4: Cyclone (Air), Detonation, Flash (Fire), Spray (Water); 5: Chilling (Water), Unraveling (Fire), Blade Whirlwind, Grappling, Rending Arrows, Wall; 6: Deadly Earth (Earth), Honed Infusion; 7: Cloud (Air, Water), Fragmentation (Earth, Water), Pure-Flame (Fire); 8: Chain Arrows. Burn, blasts and saves aren't stored. The guide's "Dazziing Infusion" is named "Dazzling Infusion" (key `…/dazzling-infusion`), its link the guide's anchor `kineticist#dazziing-infusion`.
- **The Kinetic Sharpshooter:** Exploding Arrows, Rending Arrows, Honed Infusion and Chain Arrows ("exclusive to the Sharpshooter"; each entry says so too) need `Archetype("kineticist/kinetic-sharpshooter")`; Extended Range, Kinetic Blade, Eruption, Blade Whirlwind, Deadly Earth and Fragmentation ("the Sharpshooter cannot use") need `Not(Archetype(...))` (question 2). Its "Infusion (Kinetic Sharpshooter)" feature is gone; its rows 3 to 19 hold the slot, and its rows 17 and 19, now the base's, are left out.
- **The archetypes**, by the rows in force: the Dark Elementalist, Elemental Engine, Overwhelming Soul, Psychokineticist and Kinetic Sharpshooter at 3, 5, 9, 11, 13, 17, 19; the Blood Kineticist at 3, 13, 17, 19 (Wrack, Bleeding, Gut-Wrenching and Vampiric Infusion stay its features at 1, 5, 9, 11); the Kinetic Knight at 5, 11, 13, 17, 19 (question 3).
- **No feat gives an infusion**, so ticket 49's rule has nothing to wire. Throw Elixir keeps `SelectionOption("alchemist/discovery/infusion")`.

`t59/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t59/`) ran `infusions::apply` over `libs/game-data/`: 64 changes in 1 file, byte-identical to its dry run, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change was needed. The sheet labels the slot with the selection's name: "Infusion". Ticket 48's shortcut: no feat gains an extra pick, and the new options name no feat.

### Tests

Real-data tests (`game_data.rs`, 7 new; red on HEAD's data first):

- the selection's 32 options and each one's prerequisite; the selection on base rows 3 to 19; the "Infusion" feature only on 1st-level rows; the Sharpshooter's feature gone; no feat's extra pick names it;
- a Fire kineticist 3 has one slot, "Infusion", offering 32 and leaving open exactly Burning, Dazzling, Extended Range, Fan of Flames and Kinetic Blade (an Earth one: Extended Range, Kinetic Blade, Pushing); a kineticist 2 has none; the 1st-level feature stays;
- Detonation reads "Needs Kineticist level 8 (have 3)" at 3 and 5 and is open at 9, Pure-Flame "Needs Kineticist level 14 (have 13)" until 17; every Fire or Universal infusion with no other need opens exactly at twice its level; Detonation at 3 is why the Build is impossible; one taken is "Already taken", and twice is impossible;
- a Fire kineticist's Spray reads "Needs Elemental Focus - Water", Pushing is locked and Torrent open; with Water by Expanded Element both open;
- Eruption "Needs Extended Range", Foxfire Infusion "Needs Foxfire", Blade Whirlwind after Kinetic Blade, Grappling after Wall;
- a Sharpshooter has Exploding Arrows open and Extended Range and Kinetic Blade locked; the base class reads "Needs the Kinetic Sharpshooter archetype" for Exploding Arrows;
- every archetype's sheet has its slots at its table's levels and the 1st-level feature where its table has it; filled, each is possible with every slot filled.

Extractor: `infusions.rs`, 1 test of `read_infusions` (level, elements, prerequisites, the Sharpshooter-only mark, a misspelt name fixed).

### Proof

Ticket 59 changes no rules, so one program (`t59/proof-run`, the working tree's rules) runs on HEAD's data (A) and on ticket 59's (C). It's ticket 58's proof plus 37 ticket-59 Builds (with their full versions and switches: 1,409 Builds in A, 1,412 in C): a Fire kineticist 2, 3, 5, 9 and 20, one with no element, each archetype's 20 (the Blood Kineticist with its own Water), a kineticist 3 / rogue 3, each empty and filled; a Fire kineticist 19 holding eight infusions (Extended Range, Burning, Spray on an expanded Water, Wall, Kinetic Blade, Grappling, Foxfire Infusion after the talent); a Sharpshooter 13 holding five (Exploding Arrows, Burning, Spindle, Rending Arrows, Wall); seven Builds no sheet holds. `t59/compare.py` (`t59/compare.txt`): 0 problems.

- **A checks the program:** stripped of the added field, A equals ticket 58's proof output (same rules, same data) on all 1,326 Builds.
- **The Builds without a kineticist of 3 or more** (1,273) are byte-identical, class pickers included.
- **The 136 that differ** all have a kineticist of 3 or more (95 bases). Where the picks are the same and the Build is possible alike (42 Builds), every slot both have answers the same, and the 153 new slots are all infusion slots (32 options, at 3 to 19). The two holding infusions are impossible in A (no slots) and possible in C; the seven meant to be impossible are impossible in both (Detonation at 3, Spray on Fire alone, Extended Range twice, Exploding Arrows on the base class, Extended Range on the Sharpshooter, an infusion at a Blood Kineticist's 5 or a Kinetic Knight's 3). In "Spray on Fire" two element pickers move (open 1 → 2 at 1, 0 → 1 at 7): Water there would make the Spray possible, which nothing could on HEAD's data. The rest are full Builds, whose fill differs from the first infusion slot on.
- **Ticket 48's shortcut stays exact:** on ticket 55's, 57's, 58's and 59's Builds (and their full versions), each of 373,197 feat options a feat slot accepts is open exactly when `allows` keeps the Build with it picked (368,006 on HEAD's data, also exact).
- Run time: 110 s on A, 116 s on C (8 threads).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t59/ui-check.js`, `t59/ui-check.txt`):

- A Fire kineticist 9's link opens with "INFUSION Extended Range" at 3, "INFUSION Burning Infusion" at 5 and "INFUSION Detonation" at 9; level 1 shows no infusion slot.
- The same with no infusions shows "INFUSION +" at 3, 5 and 9; the one at 3 opens "Infusion · Level 3" with Burning Infusion, Dazzling Infusion, Extended Range, Fan of Flames and Kinetic Blade, and "Show 27 locked" lists the others with their reasons ("Detonation Needs Kineticist level 8 (have 3)", "Exploding Arrows Needs the Kinetic Sharpshooter archetype", "Eruption Needs Kineticist level 6 (have 3), Extended Range").
- A Fire Sharpshooter 5's slot at 3 opens Burning, Dazzling, Exploding Arrows and Fan of Flames; "Extended Range Needs not the Kinetic Sharpshooter archetype" (question 5).
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 59's paragraph.
- `GLOSSARY.md`: unchanged (it names no class's selections).
- The map: the Decisions-so-far line.

### Checks

(`t59/verify.txt`)

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 99, rules 338 + unit 4, seeds files 25 + game_data 184 + postgres 28, server 4).
- `cargo test --workspace -- --list` (`t59/test-list.txt`, 682) differs from ticket 58's (674) only by the 8 new tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- No schema change, so no DB reset and no `cargo sqlx prepare --check` needed. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only. Rows: `feature_selection` 119 (+1), `selection_option` 808 (+32), `prerequisite_node` 2244 (+124), `feature` 1920 (−1: the Sharpshooter's).
- `cargo leptos build` and `--release`: skipped, as asked (the user runs them at the end of the queue). `cargo leptos watch` serves the sheet (UI above).
- The v1 link vector is unchanged (no `Build` or format change).
- Proof: passes. Browser: passes.
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t59/main-rs-user-edit.diff`).

### Questions for the user

1. **No infusion pick at 1st level.** The base table (and the Dark Elementalist's, Elemental Engine's, Overwhelming Soul's and Psychokineticist's) has "Infusion" at 1, but the section says she "gains new Infusions at 3rd, 5th, …", and by "twice the Infusion's level" no infusion is open at kineticist 1 (level-1 infusions need 2). Built as the ticket says: picks at 3 to 19, and the 1st-level "Infusion" stays a feature. **Recommendation:** keep it. If you know the game gives a 1st-level pick, a follow-up would add the slot at 1 and open level-N infusions at 2N − 1, which would change every infusion's level and the guide's own advice (Wall at 11, Deadly Earth at 13).
2. **"Cannot use" is built as "cannot take".** The Sharpshooter isn't offered the six infusions it cannot use, so it can't take Cloud either (Cloud needs Extended Range), nor Grappling through Cloud or Deadly Earth (Wall still opens it). **Recommendation:** keep it; an infusion it can't use is no plan.
3. **The Kinetic Knight is offered Kinetic Blade and Blade Whirlwind.** Its own Elemental Blade (1) and Blade Whirlwind (9) are features, and Elemental Blade doesn't count as Kinetic Blade for Blade Whirlwind's prerequisite. The guide doesn't say the Knight can't take them. **Recommendation:** leave it; if you want, a follow-up gives both a `Not(Archetype(kinetic-knight))`.
4. **Grappling Infusion's "Prerequisites: Cloud, Deadly Earth, Wall"** is built as one of the three, as its text says ("your Cloud, Deadly Earth or Wall Blast"). **Recommendation:** keep it.
5. **The lock reason for an excluded infusion reads "Needs not the Kinetic Sharpshooter archetype"**, the generic `Not` wording. **Recommendation:** a small `rules` label follow-up: `Not(Archetype(x))` reads "Not for the x archetype" (no other data uses it).

## Decisions on the questions

**User, 2026-11-02:** all recommendations ("Yes, run overnight": take every recommendation without asking).

1. No infusion pick at level 1.
2. "Cannot use" stays "cannot take" (so no Cloud for the Sharpshooter).
3. The Kinetic Knight is still offered Kinetic Blade and Blade Whirlwind.
4. Grappling Infusion needs one of Cloud, Deadly Earth or Wall.
5. The lock reason for an option closed to an archetype reads "Not for the … archetype": [ticket 65](65-closed-option-reason.md).
