# Witch Wandering Hex and Beneficial Curse give their pick

Type: task
Status: resolved
Blocked by: 61
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 55](55-mythic-extra-picks.md)'s decisions 1 and 2 (user, 2026-11-02):

1. **Witch Wandering Hex** ("You can select any hex for which you meet the prerequisites") gives one slot of `witch/hex`, as Second Spirit gives a spirit, and needs a hex held (the witch's or the shaman's): its "Prerequisites: Hex" is missing from the data today.
2. **Beneficial Curse** ("You gain an additional Curse with no penalties") gives one slot of the oracle's curses (`oracle/oracles-curse`), offering a curse not already held, as Second Mystery gives a mystery.

Reuse ticket 55's machinery where it fits; otherwise say what was added and why.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that take Witch Wandering Hex or Beneficial Curse differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: Witch Wandering Hex is locked without a hex and, once taken, gives one witch hex slot offering hexes whose prerequisites are met; Beneficial Curse gives one oracle curse slot offering only curses not held; removing either is refused while its slot holds a pick and frees once it's cleared; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD 3e0efc5 (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t62/main-rs-user-edit.diff`, identical to ticket 61's, and left alone). Scratch in `t62/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t62/`): `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 61's fresh extraction). Read the mythic page's Class Abilities (Witch Wandering Hex, Beneficial Curse, Second Spirit, Second Mystery, Second Bloodline) and the oracle page's Dual-Cursed Oracle. Found: Second Spirit and Second Bloodline give their pick through `MythicAbility::selections` with the class's own selection, so ticket 55's machinery (a `MythicSelection` slot at the rank, its label, removal, Extra Mythic Ability's slot granting it too) fits whole. Witch Wandering Hex's guide line is spelt "Prequisites: Hex", which the prerequisite reader passed over; Beneficial Curse's "Prerequisites: Oracle's Curse" is already an `Any` of the nine curses. Both are the same in HEAD's extraction and in `libs/game-data/`.
- 2026-11-03: real-data tests first (`game_data.rs`, 5 new), red on HEAD's data (`t62/real-data-red.txt`: 201 pass, 5 fail, each at the missing selection, prerequisite or slot).
- 2026-11-03: no `rules`, schema, `Build` or codec change was needed. `git diff HEAD -- libs/rules libs/seeds/src libs/seeds/migrations libs/seeds/.sqlx libs/ui apps/frontend` is empty.
- 2026-11-03: data and extractor. New `extract::mythic_ability_picks` (`apply`, `holding_one_of`), run after `bonus_feat_choices::apply`; `extra_picks::named_selections` and `selections` are `pub(crate)` for it, and ticket 55's `mythic_extra_picks` docs point to it. `t62/rewrite/` wrote `libs/game-data/` byte-identical to its dry run (`t62/dry/`): 3 changes in 1 file (`mythic/abilities.ron`; `t62/game-data-log.json`), 0 check problems. A second run refuses ("grants something already"). Real-data tests green (206, `t62/real-data-green.txt`). A fresh extraction (`t62/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten by `t62/rewrite/` (`t62/dry/extract-old-rewritten/`), and its two changed lines are exactly `libs/game-data/`'s (`t62/lines-extract.txt`, `t62/lines-game-data.txt`). `extract/tests/extract/mythic_ability_picks.rs`: 1 test.
- 2026-11-03: proof, UI, docs, DB, checks. Proof: `t62/proof/` (ticket 61's program plus ticket 62's Builds; `t62/patch_proof.py`), `t62/compare.py`, `t62/compare.txt`. UI: `cargo leptos watch` on 3900, headless Chromium (`t62/links/`, `t62/links.txt`, `t62/ui-check.js`, `t62/ui-check.txt`, `t62/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, `GLOSSARY.md` (**Extra pick**), the map. Reseeded (no schema change). Checks below (`t62/test-workspace.txt`, `t62/test-list.txt`).

## Answer

Done 2026-11-03, uncommitted. Ticket 55's decisions 1 and 2 are built, through ticket 55's machinery as it stands: both are mythic abilities with `selections`, as Second Spirit and Second Mystery are. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### What the guide says

| Mythic ability | Entry | Slot | Needs |
|---|---|---|---|
| Witch Wandering Hex | "You can select any hex for which you meet the prerequisites." | one of `witch/hex` ("Hex"), each hex's own prerequisites checked | "Prequisites: Hex": a hex held, the witch's or the shaman's, the same `Any` as Extra Hex's "Prerequisites: Hex" (49 options: 23 shaman hexes, 26 witch hexes) |
| Beneficial Curse | "You gain an additional Curse with no penalties." | one of `oracle/oracles-curse` ("Oracle's Curse"); a curse held is "Already taken" | "Prerequisites: Oracle's Curse" (in the data already) |

### Rules, schema, codec, UI

No code change. The rank's mythic ability grants the slot `MythicSelection { rank, 0 }` after the general pick, as Second Spirit's, and through Extra Mythic Ability's slot too. Removal follows the existing rules: the ability isn't removable while its slot holds a pick, is once it's cleared, and takes the slot with it. The sheet labels the slots by their selections' names, "Hex" and "Oracle's Curse". Ticket 48's shortcut needs no change and stays exact (Proof below). A hex held through a Hexcrafter's or a Sylvan Trickster's hex slots is a `witch/hex` option, so it opens Witch Wandering Hex too.

### Data

`extract::mythic_ability_picks` asserts both guide sentences (ticket 55's `LEFT`) and prerequisite lines, that each selection is its class's own selection of that name, that neither ability grants anything yet, that Beneficial Curse's prerequisite is a curse held, and that Witch Wandering Hex's new one equals Extra Hex's, before changing anything. `t62/rewrite/` changed 2 lines of `mythic/abilities.ron`, byte-identical to its dry run, 0 check problems. The extractor was never run over `libs/game-data/`.

### Tests

Real-data tests (`game_data.rs`, 5 new, red on HEAD's data first):

- **Data:** Witch Wandering Hex gives `witch/hex` and needs Extra Hex's prerequisite; Beneficial Curse gives `oracle/oracles-curse` and needs one of its nine curses; the mythic abilities with selections are these two and ticket 55's five.
- **Locked without a hex:** a fighter 6's MR 1 picker says `Unmet`, and a Build taking it anyway is refused; a witch's or a shaman's hexes open it.
- **Witch Wandering Hex's slot:** a witch 6 gets a "Hex" slot at MR 1 offering the witch's hexes; her four hexes are "Already taken", a major hex is `Unmet` there and refused, and is open at MR 5 (level 14, after her Major Hexes at 10). A pick there is held from level 6. A shaman's slot is a witch hex slot too, and a witch hex she took through her own hexes is "Already taken".
- **Beneficial Curse's slot:** an oracle 6 gets an "Oracle's Curse" slot offering the nine curses, all open but the one held ("Already taken"); a second curse there is held from 6, the held one refused. A fighter's MR 1 picker locks it.
- **Removal:** either one isn't removable while its slot holds a pick, is once it's cleared, and takes the slot with it.

Extractor: `mythic_ability_picks.rs`, 1 test of `holding_one_of` (each option once, shared ones included, in key order; a selection the data lacks offers none).

### Proof

Ticket 62 changes no rules, so one program (`t62/proof-run`, the working tree's rules) runs on HEAD's data (A) and ticket 62's (C). It is ticket 61's proof plus ticket 62's Builds, 1,724 Builds in A and 1,731 in C counting their full versions and switches:

- a witch, a shaman, an oracle and a Hexcrafter, each 20, empty and filled;
- nine with fixed picks: Witch Wandering Hex on a witch 6 (slot empty and filled), a shaman 6 and a Hexcrafter 6; Beneficial Curse on an oracle 6 (empty and filled); Witch Wandering Hex through Extra Mythic Ability on a witch 7; both on a Trickster at 20;
- four that no sheet holds: Witch Wandering Hex on a fighter 6, a hex or a curse held twice, Agony at 6.

Witch Wandering Hex's new prerequisite changes a mythic ability picker's statuses wherever no hex is held, so the program also hashes each slot's options without it (`without62`). Its generator counts Witch Wandering Hex among a slot's choices when its only issue is an unmet prerequisite, and leaves the slot empty when it draws it then; so a full Build draws alike on both data sets until one of them picks it.

`t62/compare.py` (`t62/compare.txt`): 0 problems.

- **A checks the program:** stripped of the added fields, A equals ticket 61's proof output on all 1,684 Builds.
- **Builds picking neither ability** (in either run): of the Builds both runs have, 503 are byte-identical; the 1,071 others picking neither differ only in Witch Wandering Hex's own entry in 4,573 mythic ability pickers (4,516 rank picks, 57 Extra Mythic Ability slots): the same total, one fewer open, everything else equal.
- **The 148 that differ otherwise** all pick one of them in A or C (74 bases; mostly full Builds whose generator picked Witch Wandering Hex without a hex on HEAD's data, or picked either and then filled its slot). Where the picks are the same and the Build is possible alike, every slot both have answers the same, and the 5 new slots are 3 `witch/hex` and 2 `oracle/oracles-curse` `MythicSelection` slots; no slot goes.
- **Fixed picks:** the seven filled ones are impossible in A (their slot pick has no slot) and possible in C; the two with the slot empty are possible in both. Witch Wandering Hex on a fighter is possible in A and impossible in C. The hex held twice, the curse held twice and Agony at 6 are impossible in both; in the first two, the level-1 picker of the held option opens more options in C (1 → 10 of 26 hexes, 1 → 9 of 9 curses), since the walk now fails at the mythic slot's pick, not at a pick with no slot.
- Every full Build picking either in C (17) is possible, and so are the 8 empty and filled ticket-62 Builds.
- **Ticket 48's shortcut stays exact:** on ticket 55's to 62's Builds (and their full versions), each of 1,010,542 feat options a feat slot accepts is open exactly when `allows` keeps the Build with it picked. On HEAD's data the count is 1,000,084, also exact.
- Run time: 228 s on A, 191 s on C (8 threads; A shared the machine with other checks).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t62/ui-check.js`, `t62/ui-check.txt`), Mythic track:

- A witch 6's link: "MR 1 Level 6 · MYTHIC ABILITY Witch Wandering Hex · HEX Fortune".
- An oracle 6's: "MR 1 Level 6 · MYTHIC ABILITY Beneficial Curse · ORACLE'S CURSE Demonic".
- The witch with the slot empty: "HEX +", whose picker opens titled "Hex", "Mythic rank 1", with 9 open hexes and "Show 17 locked": her four hexes "Already taken", the major ones "Needs Major Hexes (Witch)", the grand ones "Needs Grand Hexes".
- The oracle with the slot empty: "ORACLE'S CURSE +", picker "Oracle's Curse", 8 open curses and "Show 1 locked": Blackened "Already taken".
- A fighter 6's MR 1 picker lists Witch Wandering Hex among the locked, "Needs Air Barrier or Battle Master or …" (the 49 hexes, as Extra Hex's reason reads).
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 62's paragraph.
- `GLOSSARY.md`: **Extra pick** says a mythic ability's selection slot (Second Bloodline … Witch Wandering Hex, Beneficial Curse) comes from its own data, not as an Extra pick.
- The map: the Decisions-so-far line.

### Checks

(`t62/test-workspace.txt`, `t62/test-list.txt`)

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 102, rules 338 + unit 4, seeds files 25 + game_data 206 + postgres 28, server 4).
- `cargo test --workspace -- --list` (707) differs from ticket 61's (701) only by the 6 new tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only; `granted_selection` holds the two new rows (`beneficial-curse` → `oracle/oracles-curse`, `witch-wandering-hex` → `witch/hex`) and both abilities have a prerequisite.
- `cargo leptos build` and `--release`: skipped, as asked. `cargo leptos watch` serves the sheet (UI above).
- The v1 link vector is unchanged (no `Build` or format change).
- Proof: passes. Browser: passes.
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t62/main-rs-user-edit.diff`).

### Questions for the user

None of ticket 62's choices needed one: the guide's sentences and ticket 55's decisions settle the slots and the prerequisite. "With no penalties" needs nothing: the data gives no curse a penalty or a row. One thing found on the way, not built:

1. **The Dual-Cursed Oracle's "Second Curse"** is a feature (`oracle/dual-cursed-oracle/second-curse`), but the guide says "The Oracle must choose a second curse". So Beneficial Curse's slot can offer a Dual-Cursed Oracle the curse she took as her second, which the data can't see. **Recommendation:** a follow-up that makes it a slot of `oracle/oracles-curse` at the Dual-Cursed Oracle's level 1, in place of the feature.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run). The Dual-Cursed Oracle's "Second Curse" becomes a slot of `oracle/oracles-curse` at level 1, in place of the feature: [ticket 67](67-dual-cursed-second-curse.md).
