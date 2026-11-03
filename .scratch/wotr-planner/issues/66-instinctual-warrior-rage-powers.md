# The Instinctual Warrior's monk rage powers

Type: task
Status: resolved
Blocked by: 65
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 58](58-monk-ki-powers.md)'s decision 3 (user, 2026-11-02): the Instinctual Warrior (a barbarian archetype) can take monk powers as rage powers: Deflect Arrows, Purity of Body, Diamond Body, Diamond Soul and Perfect Self, at the levels its guide section gives. Add them as rage power options only an Instinctual Warrior can take, with those levels: separate options, not the monk's ki powers, since the levels differ.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same data method as tickets 54–60: extractor module, one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, before/after proof with the latest proof program (only Builds that have an Instinctual Warrior differ). Ticket 48's picker shortcut must stay exact. Data from the saved guide pages in `guide-cache/` only; a question the guide doesn't settle is written in this ticket with a recommendation, and the recommendation is taken (user, 2026-11-02).

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: an Instinctual Warrior's rage power picker offers each power from its level and not before; a plain barbarian isn't offered them; Extra Rage Power's slot offers them to an Instinctual Warrior; the sheet shows one picked (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD c964c02 (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t66/main-rs-user-edit.diff`, identical to ticket 65's, and left alone). Scratch in `t66/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t66/`): `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 64's fresh extraction). Read `barbarian#instinctual-warrior` ("Gains fewer Rage Powers"; "Rage Power" at 4, 8, 10, 12, 16, 18, 20) and `barbarian#rage-power-instinctual-warrior` ("Besides the normal Barbarian rage powers, an Instinctual Warrior can select a number of Monk abilities and Ki powers: Deflect Arrows (requires Improved Unarmed Strike), Purity of Body (requires 8th level), Diamond Body (requires 12th level), Diamond Soul (requires 16th level) and Perfect Self (requires 20th level)"), and Extra Rage Power ("Prerequisites: Rage Power"; its `Any` lists all 33 rage powers). The data's Instinctual Warrior rows already pick a rage power at its table's levels; `barbarian/rage-power` is also picked by the Primalist bloodrager's "Two Rage Powers".
- 2026-11-03: real-data tests first (`game_data.rs`, 4 new): red on HEAD's data (`t66/real-data-red.txt`: 217 pass, 4 fail, each at the missing options).
- 2026-11-03: no `rules`, schema, `Build` or codec change was needed. Data and extractor: new `extract::instinctual_rage_powers` (`apply`, `read_monk_powers`), run after `borrowed_extra_picks::apply`. `t66/rewrite/` wrote `libs/game-data/` byte-identical to its dry run (`t66/dry/game-data/`): 6 changes in 2 files (`classes/barbarian.ron`, 5 lines; `feats.ron`, 1 line; `t66/game-data-log.json`), 0 check problems; a second run refuses (`t66/second-run.txt`). Real-data tests green (221, `t66/real-data-green.txt`; ticket 49's Extra Rage Power slot now counts 38 options). A fresh extraction (`t66/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten (`t66/dry/extract-old-rewritten/`, the same 6 changes), and its changed lines are `libs/game-data/`'s (`t66/lines-extract.txt`, `t66/lines-game-data.txt`). Extractor tests: `instinctual_rage_powers.rs`, 1.
- 2026-11-03: proof (`t66/proof/`, `t66/patch_proof.py`, `t66/proof-run/`, `t66/compare.py`, `t66/compare.txt`), UI (`cargo leptos watch` on 3900, headless Chromium: `t66/links/`, `t66/links.txt`, `t66/ui-check.js`, `t66/ui-check.txt`, `t66/ui-*.png`; server stopped), docs (`libs/game-data/README.md`, the map), reseed (no schema change), checks (`t66/verify.txt`, `t66/test-workspace.txt`, `t66/test-list.txt`, `t66/test-list.diff`, `t66/seed.txt`).

## Answer

Done 2026-11-03, uncommitted. Ticket 58's decision 3 is built: the Instinctual Warrior's five monk powers are rage power options only it can take, at the guide's levels. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::instinctual_rage_powers` asserts the guide's sentences, the archetype's table ("Rage Power" at 4, 8, 10, 12, 16, 18, 20) and its rows in force, and Extra Rage Power's `Any` (exactly the 33 rage powers), then:

| Option (`barbarian/rage-power/…`) | Name | Needs |
|---|---|---|
| `deflect-arrows-instinctual-warrior` | Deflect Arrows (Instinctual Warrior) | `All([Archetype(instinctual-warrior), Feat(improved-unarmed-strike)])` |
| `purity-of-body-instinctual-warrior` | Purity of Body (Instinctual Warrior) | `All([Archetype, ClassLevel(barbarian, 8)])` |
| `diamond-body-instinctual-warrior` | Diamond Body (Instinctual Warrior) | `All([Archetype, ClassLevel(barbarian, 12)])` |
| `diamond-soul-instinctual-warrior` | Diamond Soul (Instinctual Warrior) | `All([Archetype, ClassLevel(barbarian, 16)])` |
| `perfect-self-instinctual-warrior` | Perfect Self (Instinctual Warrior) | `All([Archetype, ClassLevel(barbarian, 20)])` |

Each links to `barbarian#rage-power-instinctual-warrior`, which lists them; each is taken once. Extra Rage Power's `Any` names the five too (38). `t66/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t66/`) changed 5 lines of `barbarian.ron` and 1 of `feats.ron`, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change. The archetype's own rage power slots and Extra Rage Power's slot (ticket 49: it offers what the character's rage power slots offer) offer the five; a character without the archetype sees them locked ("Needs the Instinctual Warrior archetype", with the level first when that's missing too). The sheet shows a pick by its option name ("RAGE POWER Purity of Body (Instinctual Warrior)"). Ticket 48's shortcut needs nothing new: Improved Unarmed Strike is named once Deflect Arrows is picked (it's in the pick's prerequisite), and checked exact below.

### Tests

Real-data tests (`game_data.rs`, 4 new; red on HEAD's data first):

- the five options' names, link, repeat and prerequisites; the selection has 38 options; Extra Rage Power's `Any` names exactly the selection's options;
- an Instinctual Warrior 20 with Improved Unarmed Strike has rage power slots at 4, 8, 10, 12, 16, 18, 20; in each slot each power is open exactly from its level (Deflect Arrows from 4); the reasons before ("Needs Barbarian level 8 (have 4)", "… 12 (have 10)", "… 16 (have 12)", "… 20 (have 18)"); without Improved Unarmed Strike Deflect Arrows reads "Needs Improved Unarmed Strike"; all five taken at their levels is possible, the slot reads "Rage Power" and the option "Purity of Body (Instinctual Warrior)", a power taken is "Already taken" later; Diamond Body at 8 is impossible for "Needs Barbarian level 12 (have 8)";
- a plain barbarian 20 and an Armored Hulk 20 (Improved Unarmed Strike held) have all five locked in every rage power slot for the archetype (and the level), and taking one is impossible;
- an Instinctual Warrior 9 holding only Deflect Arrows may take Extra Rage Power at 5 and 9; each gives a "Rage Power" slot offering what its slot at 8 offers, Purity of Body open at 9 and not at 5, Deflect Arrows taken; filled, it's possible; a plain barbarian's Extra Rage Power slot has the five locked.
- Changed: ticket 49's `a_barbarian_with_extra_rage_power_at_3_gets_a_rage_power_slot_there` counts 38 options (33 and the five, locked there).

Extractor (`instinctual_rage_powers.rs`, 1): `read_monk_powers` reads a feat requirement and ordinal levels from a sentence in the guide's style.

### Proof

Ticket 66 changes no rules, so one program (`t66/proof-run`, the working tree's rules) runs on HEAD's data (A) and ticket 66's (C). It is ticket 65's proof plus, per slot, the options hashed without the five and Extra Rage Power, the lock reasons hashed without them, the five's and Extra Rage Power's open flags; per Build its Instinctual Warrior, whether it has a barbarian or bloodrager, and Extra Rage Power's distinct lock reasons; and 37 ticket-66 Builds (an Instinctual Warrior 3, 4, 8, 12, 16, 20 empty and filled; one with each power at its level; one holding only Deflect Arrows with Extra Rage Power at 5 and 9; one with a plain power and Extra Rage Power; a fighter 5 / Instinctual Warrior 8 with Purity of Body at barbarian 8; a plain barbarian 20; and Builds no sheet holds: Diamond Body at 8, Deflect Arrows without Improved Unarmed Strike, Perfect Self on a plain barbarian 20 and an Armored Hulk 20, Purity of Body at barbarian 4 on the fighter 5 / Instinctual Warrior 8, character level 9; each also filled). 1,945 Builds in A, 1,953 in C. `t66/compare.py` (`t66/compare.txt`): 0 problems.

- **A checks the program:** with the added fields removed, A equals ticket 65's proof B (the same rules on the same data) on all 1,892 Builds both have.
- **Builds without an Instinctual Warrior (1,895):** possibility, sheets, `removable`, class, path and switch pickers and `allows` are identical. Per slot, the open counts, the options and reasons apart from the five and Extra Rage Power, Extra Rage Power's open flag and the negated reasons are identical. Two Builds are byte-identical. The rest differ only as expected: 443 rage power slots (69 Builds, every one with a barbarian or bloodrager) gain the five, always locked; and 15,391 feat slots list Extra Rage Power's lock reason with the five names among its alternatives (each C text is its A text with the five added).
- **Builds with an Instinctual Warrior (35 in both, 8 full Builds only in C):** 23 keep their sheet; their slots gain open monk powers from their levels (Purity of Body 32 times, Diamond Body 17, Diamond Soul 9, Perfect Self 2). The 12 others are filled Builds whose generator draws a monk power once one opens.
- **Builds picking one of the five (15, all impossible on HEAD's data, which has no such option):** possible on C are the each-power, the Deflect-Arrows-only-with-Extra-Rage-Power and the fighter 5 / Instinctual Warrior 8 Builds (and their filled versions); the seven meant to be impossible stay impossible.
- **Ticket 48's shortcut stays exact:** on ticket 55's to 66's Builds and their full versions, every one of 1,312,773 feat options in C (1,297,355 in A) is open exactly when `allows` keeps the Build with it picked.
- **Run time:** 3 min 51 s (A), 3 min 50 s (C), 8 threads.

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t66/ui-check.txt`):

- An Instinctual Warrior 20's link shows "RAGE POWER Deflect Arrows (Instinctual Warrior)" at 4, "Purity of Body (Instinctual Warrior)" at 8, "Diamond Body (Instinctual Warrior)" at 12, "Diamond Soul (Instinctual Warrior)" at 16 and "Perfect Self (Instinctual Warrior)" at 20 (`t66/ui-each.png`).
- An Instinctual Warrior 8's rage power picker at 8 lists Deflect Arrows and Purity of Body as open; "Show 14 locked" gives "Diamond Body (Instinctual Warrior) Needs Barbarian level 12 (have 8)", and the same for Diamond Soul (16) and Perfect Self (20).
- A plain barbarian 8's picker at 8: "Show 16 locked" gives "Deflect Arrows (Instinctual Warrior) Needs the Instinctual Warrior archetype", "Diamond Body (Instinctual Warrior) Needs Barbarian level 12 (have 8), the Instinctual Warrior archetype", and so on.
- An Instinctual Warrior 9 with Extra Rage Power at 9: the sheet shows "FEAT Extra Rage Power RAGE POWER +" at 9. Its picker opens as "Rage Power · Level 9" with Purity of Body open, Deflect Arrows "Already taken", and Diamond Body "Needs Barbarian level 12 (have 9)".
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 66's paragraph.
- `GLOSSARY.md`: unchanged (Feature selection and Extra pick already cover it).
- The map: the Decisions-so-far line.

### Checks

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 106, rules 347 + unit 4, seeds files 25 + game_data 221 + postgres 28, server 4).
- `cargo test --workspace -- --list` (735) vs ticket 65's (730) (`t66/test-list.diff`): exactly the 5 new tests (4 real-data, 1 extractor).
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds).
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only (`selection_option` 813, `prerequisite_node` 2,392).
- `cargo leptos build` and `--release`: skipped, as asked.
- The v1 link vector is unchanged (no `Build` or format change).
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t66/main-rs-user-edit.diff`).

### Questions decided by recommendation

1. **"Requires 8th level": of what?**
   - **Recommendation, built:** barbarian level (`ClassLevel(barbarian, N)`), as every rage power's level is in the data, and since the archetype gains rage powers by its barbarian levels. A fighter 5 / Instinctual Warrior 8 takes Purity of Body at barbarian 8, not at character level 8.
2. **Deflect Arrows' requirement.**
   - **What was there:** the feat Deflect Arrows also needs Dexterity 13; the guide's sentence names only Improved Unarmed Strike.
   - **Recommendation, built:** the option needs the archetype and Improved Unarmed Strike only, as the sentence says. It's an option of its own, not the feat: holding it isn't holding the feat Deflect Arrows (nothing in the data needs that feat), and a character holding the feat may still take it.
3. **Names and links.**
   - **Recommendation, built:** "… (Instinctual Warrior)", linking to the section that lists them, as the archetype's other features are named ("Greater Focused Rage (Instinctual Warrior)").
   - The monk's sections couldn't serve: Diamond Body and Diamond Soul already link there as ki powers, so they'd read as copies (`no_selection_option_is_a_copy_of_another`).
   - The suffix also tells them apart from the monk's ki powers and the feat in the picker and on the sheet.
4. **Extra Rage Power's prerequisite.**
   - **Recommendation, built:** its `Any` (the data's "holds a rage power") names the five too, so an Instinctual Warrior holding only a monk power may take it.
   - **Cost:** the feat's lock reason, shown wherever it's locked, lists five more names (`compare.txt`, section 3). Ticket 64's alternative, the archetype from its first slot whatever it picked, would hide them from other characters. But it would also let an Instinctual Warrior 4 with an empty slot take the feat, which the plain barbarian's encoding doesn't allow.
5. **Shown locked to everyone else.**
   - **Recommendation, built:** every barbarian's (and Primalist bloodrager's) rage power picker lists the five as locked, needing the archetype, as other archetype-only options are listed (the Kinetic Sharpshooter's infusions, the Wildland Shaman's hexes).
   - Hiding them would be a `rules` change for all such options.
6. **A monk / Instinctual Warrior may hold a power twice** (Diamond Body as a ki power and as a rage power; Purity of Body as the monk's feature and as a rage power).
   - **Recommendation, built:** allowed, since the user's decision makes them separate options. Nothing in the guide forbids it, and nothing in the data needs either.
   - Only a Drunken Master can be both (a barbarian can't be lawful; ticket 63 lets the Drunken Master's "Cannot be Lawful" replace the monk's "Lawful").

None needs the user.
