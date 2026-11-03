# An archetype's alignment requirement replaces its class's

Type: task
Status: resolved
Blocked by: 62
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 58](58-monk-ki-powers.md)'s question 1, answered by the user (2026-11-02): "drunken master can be built as long as you check the alignment per archetype and not on the base class". Today a Drunken Master can't be built at all: the monk's prerequisite needs a Lawful alignment (`classes/monk.ron`, `Any([Alignment(LawfulGood), …])`), the archetype's needs a non-Lawful one, and both are checked when the class is taken.

The rule (a user fact): when an archetype states an alignment requirement, it replaces the class's alignment requirement; the class's other prerequisites still hold. An archetype without one keeps the class's.

More examples from the user (2026-11-02): "the mantis archetype and the vampire slayer must be evil".

- **Bloodseeker** (the slayer's vampire archetype: "Alignment must be evil" in `slayer#bloodseeker`) already has `Any([LawfulEvil, NeutralEvil, ChaoticEvil])` in `classes/slayer.ron`. The slayer has no alignment rule of its own, so it's buildable today; keep it enforced (a test).
- **Mantis Zealot** (`warpriest/mantis-zealot`) has no alignment rule in the data: the guide says only "May only select Achaekek as a deity", and deities are out of scope. The user's fact adds one. The exact alignments: see the Decisions below.

Check every archetype with an alignment requirement for the same clash with its class (and for one that narrows rather than replaces it: the rule is the same, the archetype's wins). List what you find in the Answer.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Probably a rules change (where a class level's prerequisite is checked with its archetype), maybe a data change if the alignment leaves need separating; schema change and DB reset allowed per `CLAUDE.md` if the types need one. Engine code reaches game data through ticket 56's `Catalog`. Before/after proof with ticket 58's program: only Builds with an archetype that states an alignment may differ. Ticket 48's picker shortcut must stay exact. Reasons on the sheet name the archetype's alignment rule, not the class's, for such an archetype.

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: a Chaotic Neutral monk 1 can take Drunken Master; a Lawful Neutral one can't, with the archetype's alignment as the reason; a plain monk still needs Lawful; a Bloodseeker needs an evil alignment; a Mantis Zealot is open at LE, LN and NE and locked at CE and LG; every archetype found with a clash is buildable at an alignment its own rule allows; the sheet shows the Drunken Master's alignment reason (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD 2019f7f (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t63/main-rs-user-edit.diff`, identical to ticket 62's, and left alone). Scratch in `t63/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t63/`): `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 62's fresh extraction). Found where a class is entered: `Walker::check_class_entry` (the walk) and `Engine::class_status` (the class picker) each check the class's prerequisite and the archetype's, both always. Swept every saved page for alignment lines ("Alignment must be", "Cannot be", "Alignement", "restricted to"): seven archetypes have one (Answer).
- 2026-11-03: rules tests first (`libs/rules/tests/engine/alignment.rs`, 7, a world whose monk needs Lawful and Wisdom 12, with a Drunken Master and a Lawful Good archetype): red on HEAD's rules (`t63/rules-red.txt`: 5 fail, 2 pass; the two that pass, a Lawful Drunken Master locked by its own rule and the Zen Archer keeping the monk's, already held). Built the rule; green.
- 2026-11-03: data and extractor. New `extract::archetype_alignments` (`apply`, `alignment_rule`), run after `mythic_ability_picks::apply`. Real-data tests (`game_data.rs`, 6 new) red on HEAD's data with the new rules (`t63/real-data-red-new-rules.txt`: 2 fail, the Mantis Zealot's and the sweep's) and on HEAD's rules too (`t63/real-data-red-head.txt`: 3 fail, plus the Drunken Master's). `t63/rewrite/` wrote `libs/game-data/` byte-identical to its dry run (`t63/dry/`): 2 changes in 2 files (`t63/game-data-log.json`), 0 check problems; a second run refuses ("has a prerequisite already"). A fresh extraction (`t63/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten (`t63/dry/extract-old-rewritten/`), and its two changed lines are `libs/game-data/`'s (`t63/lines-extract.txt`, `t63/lines-game-data.txt`). Real-data tests green (212, `t63/real-data-green.txt`).
- 2026-11-03: proof (`t63/proof/`, `t63/patch_proof.py`, `t63/compare.py`, `t63/compare.txt`), UI (`cargo leptos watch` on 3900, headless Chromium: `t63/links/`, `t63/links.txt`, `t63/ui-check.js`, `t63/ui-check.txt`, `t63/ui-*.png`; server stopped), docs (`libs/game-data/README.md`, `GLOSSARY.md`, the map), reseed (no schema change), checks (`t63/verify.txt`, `t63/test-workspace.txt`, `t63/test-list.txt`, `t63/test-list.diff`).

## Answer

Done 2026-11-03, uncommitted. An archetype's alignment requirement now replaces its class's, so a Drunken Master can be built; the Mantis Zealot has the user's alignment rule, and the Priest of Balance the guide's. No schema, `Build` or link change, so `V1_LINK` stays.

### Rules

- **Where:** `Catalog::entry_prerequisites(class, archetype)` (`engine/catalog.rs`) gives what entering a class with an archetype needs: the class's prerequisite and the archetype's. When the archetype's states an alignment requirement, the class's is taken without its own (precomputed once per class in the catalog); its other conditions still hold. An archetype without one keeps the class's. The walk (`Walker::check_class_entry`) and the class picker (`Engine::class_status`) both read it, so `is_possible`, `allows`, `removable` and the picker's reasons agree. A failing archetype rule fails the archetype choice, as before; the class's other conditions fail the class choice.
- **What an alignment requirement is:** an `Alignment`, or a group (or `Not`) of nothing but alignments, standing as the whole prerequisite or as one of a top-level `All`'s conditions (`Prerequisite::is_alignment`, `states_alignment`, `without_alignment`, `names_alignment`, crate-private). An archetype that narrows its class's (Lawful Good where the class says Lawful) works the same: its rule is the one checked.
- **New data check:** `DataIssue::AlignmentNotOnItsOwn`: a class's or archetype's prerequisite naming an alignment inside a group with other conditions ("an alignment sits in a group with other conditions; the alignment requirement must be a condition of its own"), so the rule above is always well defined. The real data passes (feats' and options' prerequisites may still mix them: the Champion's blessings).
- **Reasons:** a Lawful Neutral Drunken Master reads "Needs Neutral Good alignment or Chaotic Good alignment or True Neutral alignment or Chaotic Neutral alignment or Neutral Evil alignment or Chaotic Evil alignment": the archetype's rule only.
- **Ticket 48's shortcut** needs no change (`named_feats` collects the feats of the class's whole prerequisite and every archetype's, a superset) and stays exact (Proof).

### The archetypes with an alignment rule

Every saved page swept; seven, all base-class archetypes (prestige classes have none):

| Archetype | Rule | Source | Class's rule | Clash |
|---|---|---|---|---|
| Drunken Master (`monk`) | not Lawful (NG, CG, TN, CN, NE, CE) | guide, "Cannot be Lawful" (data already) | Lawful | **yes**: no alignment met both; now buildable at any non-lawful one |
| Bloodseeker (`slayer`) | evil (LE, NE, CE) | guide, "Alignment must be evil" (data already) | none | no |
| Angelfire Apostle (`cleric`) | good | guide (data already) | none (deities out of scope) | no |
| Reformed Fiend (`bloodrager`) | good, beside Race tiefling | guide (data already) | none | no |
| Prophet of Pestilence (`shaman`) | TN, NE, CE | guide (data already) | none | no |
| Priest of Balance (`cleric`) | LN, TN, CN | guide, "Alignement restricted to Lawful Neutral, Neutral and Chaotic Neutral": **new** (the misspelling made the extraction pass it by; question 1) | none | no |
| Mantis Zealot (`warpriest`) | LE, LN, NE | **the user's fact** (decision 1; the guide says only that it worships Achaekek): **new** | none | no |

The barbarian, paladin, druid, shifter and monk have class rules; none of their other archetypes states one, so they keep the class's. No archetype narrows its class's rule in the data today (the rules test covers the case).

### Data

`extract::archetype_alignments` asserts each archetype's guide line, the five existing rules' alignments (`alignment_rule`), that the two new ones had no prerequisite, and that no other archetype names an alignment, before adding `Any([LN, TN, CN])` to `cleric/priest-of-balance` and `Any([LN, LE, NE])` to `warpriest/mantis-zealot`. `t63/rewrite/` changed one line in each of `classes/cleric.ron` and `classes/warpriest.ron`, 0 check problems. The extractor was never run over `libs/game-data/`. `libs/game-data/README.md` marks the Mantis Zealot's as the user's fact, not the guide's.

### Schema, codec, UI

No change. The sheet's class picker and the alignment wheel already show what the engine answers.

### Tests

- `rules` (`tests/engine/alignment.rs`, 7): a non-lawful Drunken Master needs only its own rule; a lawful one is locked by it; the class keeps its other condition (Wisdom 12); a plain monk and the Zen Archer keep the monk's; an archetype narrowing the alignment is checked alone; the Build start's alignment can change only within the archetype's rule; the new data check.
- Real data (`game_data.rs`, 6 new, 1 changed): a Chaotic Neutral monk 1 can take Drunken Master (and a Neutral Evil one 20, filled, is possible); a Lawful Neutral one can't, with the archetype's alignment as the reason and its sheet text; a plain monk (and a Zen Archer) still needs Lawful; a Bloodseeker needs an evil alignment (a slayer doesn't); a Mantis Zealot is open at LE, LN and NE and locked at the other six, CE and LG among them; the seven archetypes are exactly those naming an alignment, only the Drunken Master clashed, and each is open exactly at its rule's alignments and buildable to 20 there, filled. Ticket 58's ki power test now walks the Drunken Master as possible too, and ticket 32's Mantis Zealot feats test sets a lawful neutral alignment (a Build with no alignment fails any alignment rule, the class's or the archetype's, as before).
- Extractor (`archetype_alignments.rs`, 1): `alignment_rule` (whole, beside other conditions, a `Not`, mixed: none).

### Proof

Ticket 63 changes `rules`, so ticket 62's program (plus each Build's archetypes with a rule, `aligned63`, the class pickers without those archetypes' entries, `classes_without63`, and 33 ticket-63 Builds: each of the seven at 1 at an allowed and a refused alignment and at 20 empty and filled, a plain monk 1 at CN and LN, a slayer 1 at LG, a Drunken Master 4 / rogue 4) was compiled twice: `t63/proof-old` against `git archive HEAD`'s rules, `t63/proof-new` against the working tree's. A is proof-old on HEAD's data (1,797 Builds), C proof-new on ticket 63's (1,805), and B proof-new on HEAD's data (1,809: the rules change alone). `t63/compare.py` (`t63/compare.txt`): 0 problems.

- **A checks the program:** stripped of the added fields, A equals ticket 62's proof output on all 1,731 Builds both have.
- **Builds without an archetype stating an alignment** answer the same in A, B and C, byte for byte but the class pickers, whose only difference is those archetypes' entries (`classes_without63` equal): A vs C 83 identical and 1,638 differing only so.
- **A vs B (the rules):** beyond the class pickers only Drunken Master Builds differ (8): 7 go from impossible to possible (ticket 47's and 58's Drunken Master 20, ticket 63's CN 1 and 20, the monk 4 / rogue 4), the LN one stays impossible; their full versions exist only in B (12). The Bloodseeker, Angelfire Apostle, Reformed Fiend and Prophet Builds differ only in class pickers, possible as before.
- **B vs C (the data):** beyond the class pickers only Priest of Balance (12) and Mantis Zealot (10) Builds differ: the Lawful Good Priest of Balance 1 and the Chaotic Evil Mantis Zealot 1 become impossible; ticket 47's true neutral Mantis Zealot 20 too (so its full versions exist only in B); the rest answer the same but for `allows` and `removable` (the alignment edits).
- **Ticket 48's shortcut stays exact:** on ticket 55's to 63's Builds and their full versions, each of 1,125,177 feat options (C; 1,103,363 in A, 1,125,493 in B) a feat slot accepts is open exactly when `allows` keeps the Build with it picked.
- Run time: 3 min 31 s (A), 3 min 23 s (C), 3 min 11 s (B), 8 threads.

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t63/ui-check.txt`):

- A Chaotic Neutral Drunken Master 4's link opens ("Monk 1 🔒 Drunken Master", "KI POWER Barkskin" at 4), with no stale-link error; its alignment wheel locks Lawful Good, Lawful Neutral and Lawful Evil and leaves the six others open.
- A Lawful Neutral Build start's level-1 class picker: the Mantis Zealot open; the Drunken Master among the 44 locked, "Drunken Master Monk archetype Needs Neutral Good alignment or Chaotic Good alignment or True Neutral alignment or Chaotic Neutral alignment or Neutral Evil alignment or Chaotic Evil alignment" (the archetype's rule, not the monk's).
- A Chaotic Neutral one: the Drunken Master open; the monk locked, "Needs Lawful Good alignment or Lawful Neutral alignment or Lawful Evil alignment"; the Mantis Zealot locked, "Needs Lawful Neutral alignment or Lawful Evil alignment or Neutral Evil alignment".
- No console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 63's paragraph (the rule, the seven, the Mantis Zealot's as the user's fact).
- `GLOSSARY.md`: **Prerequisite** says an archetype's alignment requirement replaces its class's.
- The map: the Decisions-so-far line.

### Checks

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 103, rules 345 + unit 4, seeds files 25 + game_data 212 + postgres 28, server 4).
- `cargo test --workspace -- --list` (721) differs from ticket 62's (707) only by the 14 new tests (`t63/test-list.diff`).
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds).
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only.
- `cargo leptos build` and `--release`: skipped, as asked. `cargo leptos watch` serves the sheet (UI above).
- The v1 link vector is unchanged (no `Build` or format change).
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t63/main-rs-user-edit.diff`).

### Questions decided by recommendation

1. **The Priest of Balance's alignment** ("Alignement restricted to Lawful Neutral, Neutral and Chaotic Neutral") was missing from the data: the misspelt line passed the extraction by, as ticket 62's "Prequisites" did. The ticket asks for every archetype with an alignment requirement, and the guide states this one. **Recommendation, built:** add it (`Any([LN, TN, CN])`). The cleric has no class rule (deities are out of scope), so nothing clashes; a cleric Priest of Balance Build outside the three is now impossible.
2. **What counts as a class's "alignment requirement"** when a prerequisite holds other conditions too. **Recommendation, built:** the conditions about alignment alone, as the whole prerequisite or a top-level `All`'s conditions; the data check refuses an alignment mixed into a group with other conditions on a class or archetype, so the rule is never ambiguous.

None needs the user.

## Decisions

**User, 2026-11-02:**

1. **The Mantis Zealot allows Lawful Evil, Lawful Neutral or Neutral Evil**: one step from Achaekek (Lawful Evil). A user fact; add it to `warpriest/mantis-zealot` as its alignment rule (`Any([Alignment(LawfulEvil), Alignment(LawfulNeutral), Alignment(NeutralEvil)])`), with a note in `libs/game-data/README.md` that it comes from the user, not the guide.
