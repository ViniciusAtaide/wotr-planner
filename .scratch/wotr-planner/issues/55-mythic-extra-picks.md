# Mythic Extra Feat and Extra Mythic Ability give their pick

Type: task
Status: resolved
Blocked by: 54
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 49](49-extra-feats-extra-picks.md)'s decision 4 (user, 2026-11-02): Mythic Extra Feat ("You can select any normal feat") and Extra Mythic Ability ("You can select a mythic ability") give one more pick each time they're taken: a feat slot and a mythic ability slot, at the level (Mythic rank) they're taken. Today they give nothing but themselves.

Check both against their guide entries and the saved mythic pages: whether each repeats, whether the feat slot takes any feat whose prerequisites are met (or ignores them), and whether the mythic ability slot offers what a Mythic rank's ability slot offers (path abilities included or not). Look for other mythic feats or abilities that give a pick the same way and include them if the guide states it plainly. If the guide doesn't settle one, write a question with a recommendation and go on.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Prefer reusing ticket 49's machinery (`Feat::extra_pick_of`, `SlotId::FeatSelection`) if a feat slot and a mythic ability slot fit it; otherwise say what was added and why. Same data method as tickets 43–45 and 49, with before/after proof (only Builds taking these picks differ). Ticket 48's picker shortcut must stay exact (add these feats to the always-walked ones if their slot shifts later indices).

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes (ticket 22's Verification with the current paths and package names), plus real-data tests: Mythic Extra Feat taken at a Mythic rank gives one feat slot at that rank's level, offering what a normal feat slot offers there; Extra Mythic Ability gives one mythic ability slot; removing either is refused while its slot holds a pick and frees once it's cleared; the sheet labels the new slots (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-02: started. Baseline: HEAD 02f7d98 (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t55/main-rs-user-edit.diff` and left alone); scratch in `t55/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t55/`), `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 54's fresh extraction). Read the mythic page's Mythic Feats and Class Abilities and the path pages. Extra Feat ("You can select any normal feat") and Extra Mythic Ability ("You can select a mythic ability") are the only mythic feats whose entry says "You can select"; neither says it can be taken again (the feats page says "This feat may be taken multiple times" where a feat repeats; the mythic page never does). In the data both are plain `Once` mythic feats with no prerequisite. Mythic feats are only ever picked in a rank's general slot (even ranks), so a rank holds at most one. Of the mythic abilities whose entry reads like a pick, five already give it through `selections` (Impossible Domain, Second Bloodline, Second Bloodrager Bloodline, Second Mystery, Second Spirit) and two don't: Witch Wandering Hex and Beneficial Curse (questions 1 and 2). The path pages' "Extra Minor Aspect" (Demon) is simultaneous use, not a pick.
- 2026-11-02: rules, test-first. Ticket 49's machinery didn't fit: `Feat::extra_pick_of` names Feature selections and belongs to feats, and `SlotId::FeatSelection` is a selection slot at a character level, numbered among that level's; these slots are a feat slot and a mythic ability slot, given by a mythic feat at a rank. Added `MythicFeat::extra_pick: Option<MythicExtraPick>` (`Feat`, `MythicAbility`) and `SlotId::MythicExtra { rank }` (one per rank at most, so no index). Test world: `extra-mythic-ability` gives its pick and a new `extra-feat`. `mythic_extra_picks.rs`, 7 tests: red on HEAD's walk (`t55/rules-red.txt`: 330 pass, 8 fail: 6 new, the world count and the frozen link; `other_mythic_feats_give_no_slot` passes, pinning it). Built: the walk fills the slot after the general pick (kind `Feat(General)` or `MythicAbility`), a mythic ability picked there grants its `selections` as the general pick's would, its picks are general picks for a Path switch; "Feat" label; codec group 9. `V1_LINK` regenerated. Green: 338 + 4 (`t55/rules-green.txt`).
- 2026-11-02: schema and DB: domain `mythic_extra_pick` and nullable column `mythic_feat.extra_pick`; seed, load and `seeds::columns` carry it. DB reset per `CLAUDE.md` (`wotr-planner-postgres` on 5434 only); `cargo sqlx prepare` replaced the mythic feat insert and select query files (2 out, 2 in). Postgres: 1 new test (round trip; another value refused by the domain).
- 2026-11-02: real-data tests first (`game_data.rs`, 5 new): red on HEAD's data (`t55/real-data-red.txt`: 163 pass, 5 fail, each at the missing `extra_pick` or slot).
- 2026-11-02: data and extractor. New `extract::mythic_extra_picks` (`apply`, `LEFT`), run after `mercies_and_exploits::apply`: it asserts the two sentences, that only those two mythic feats' entries say "You can select", that neither says it repeats, that each is `Once` with no parameter, prerequisite or extra pick yet, and that the mythic abilities whose entry reads like a pick are the five with selections plus the two left. `ron_out` writes `extra_pick` after `removes_racial_penalty`. `t55/rewrite/` wrote `libs/game-data/` byte-identical to its dry run (`t55/dry/`): 2 changes in 1 file (`mythic/feats.ron`; `t55/game-data-log.json`), 0 check problems; a second run refuses. Real-data tests green (168, `t55/real-data-green.txt`). A fresh extraction (`t55/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten by `t55/rewrite/` (`t55/dry/extract-old-rewritten/`): the same 2 lines as in `game-data/`. `extract/tests/extract/mythic_extra_picks.rs`: 1 test.
- 2026-11-02: proof, UI, docs, DB, Verification. Proof: `t55/proof/` (ticket 54's program with ticket 55's Builds and each Build's Extra Feat / Extra Mythic Ability picks in its line), built against HEAD's rules (`proof-old`) and the working tree's (`proof-new`); `t55/compare.py`, `t55/compare.txt`. UI: `cargo leptos watch` on 3900, headless Chromium (`t55/links/`, `t55/ui-check.js`, `t55/ui-check.txt`, `t55/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, `GLOSSARY.md` (**Extra pick**), the map. Reseeded. Verification below.

## Answer

Done 2026-11-02, uncommitted. Ticket 49's decision 4 is built: Extra Feat and Extra Mythic Ability each give one more pick at the Mythic rank they're taken. New in `rules` (a mythic feat's extra pick, a slot id for it), the schema (a domain and a column, DB reset) and the link format (`V1_LINK` regenerated, still version 1).

### What the guide says

| Mythic feat | Entry | Repeats | Slot |
|---|---|---|---|
| Extra Feat | "You can select any normal feat." | no (the entry doesn't say so) | a feat slot: what the general slot offers, prerequisites checked |
| Extra Mythic Ability | "You can select a mythic ability." | no | a mythic ability slot: what a rank's general pick offers |

- **Prerequisites:** "any normal feat" is what the general slot offers ("any feat whose prerequisites are met"); nothing says they're ignored, and only a class's bonus list skips them (ticket 16).
- **Path abilities:** a rank's general mythic ability pick offers the 63 general mythic abilities; the paths' own abilities are their rows' selection slots (Trickster tricks, Aeon gazes, Demon aspects), which no general pick offers. The extra slot offers the same as the general pick.
- **Others:** no other mythic feat gives a pick. Five mythic abilities already give theirs through `selections`; two read like one but give none (questions 1 and 2).

### Rules

- `MythicFeat::extra_pick: Option<MythicExtraPick>`: `Feat` (a `FeatSlot::General` slot) or `MythicAbility` (a `SlotKind::MythicAbility` slot).
- `SlotId::MythicExtra { rank }`: right after the rank's general pick, before the slots its mythic ability grants and the path's. A rank has one general pick, so one such slot at most: no index. Its kind is the general slot's or a rank's general pick's, so it offers and checks the same; its pick is held from the rank's level.
- A mythic ability picked there grants its selection slots as the general pick's would (Second Bloodline: `MythicSelection { rank, 0 }`, another bloodline). A feat picked there with an `extra_pick_of` gives its `FeatSelection` slot at the rank's level (Extra Rage Power through Extra Feat: a rage power at 7).
- Path switch: its pick is a general pick of the rank (`from_path: false`), so a switch that drops the rank's general pick drops it too (a Legend loses an MR 4 Extra Feat's feat from MR 8 on), and the track marks it.
- Removal: the existing rules hold. The slot's pick depends on the mythic feat, so the mythic feat isn't removable while the slot holds a pick; clearing the pick frees it, and removing it removes the slot. The rank's mythic feat picker locks every other mythic feat meanwhile (`BreaksLater`).
- Labels: the feat slot reads "Feat" (as the general slot), the ability slot "Mythic ability".
- Ticket 48's shortcut: it only answers feat options. A feat in the extra slot takes no index from another slot's, so it's answered as in any feat slot, and mythic feats are always walked. No feat joins the always-walked ones; the proof checks the shortcut stays exact.
- Codec: group `9` MythicExtra, a step and no slot fields (as Mythic). `V1_LINK` regenerated with a comment: the frozen Build gains Dodge in MR 2's extra slot (207 characters).

Engine tests (test world): 7 new in `mythic_extra_picks.rs`; `codec` (the generator and the frozen Build), `canonical` (10 mythic feats) and `mythic` (MR 2's Extra Mythic Ability now shows its slot) updated.

### Schema

Domain `mythic_extra_pick` (`feat`, `mythic_ability`), nullable column `mythic_feat.extra_pick`. Still 42 tables. Postgres test: 1 new.

### Data

`extract::mythic_extra_picks` asserts each guide sentence and what it finds before changing anything. `t55/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t55/`) changed 2 lines of `mythic/feats.ron` (`extra_pick: Feat`, `extra_pick: MythicAbility`), byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

### Tests

Real-data tests (`game_data.rs`, 5 new, red on HEAD's data first):

- Extra Feat and Extra Mythic Ability are the only mythic feats with an extra pick, a feat and a mythic ability, each taken once;
- a fighter 7 with Extra Feat at MR 2 gets a "Feat" slot at MR 2 (level 7) after the general pick, whose options and statuses are exactly level 7's general slot's; Toughness there is held from 7; Great Cleave there is refused (`Unmet`);
- Extra Mythic Ability gives a "Mythic ability" slot offering the 63 mythic abilities, as MR 1's does; a sorcerer's Second Bloodline there gives another bloodline at MR 2;
- either one isn't removable while its slot holds a pick, is once it's cleared, and takes the slot with it;
- Extra Rage Power through Extra Feat gives a barbarian 7 a rage power slot at 7.

### Proof

`t55/proof/` is ticket 54's proof program (every slot's options with statuses, the class, path and switch pickers, `removable`, `allows` over the edit battery, `is_removable`, `removable_classes` and `allows_many` checked against them, and ticket 48's shortcut checked against `allows`) with 24 ticket-55 Builds (Extra Feat and Extra Mythic Ability on a fighter 7, empty and filled; both on a Trickster fighter 20; Second Bloodline through Extra Mythic Ability; Extra Rage Power through Extra Feat; two Builds no sheet holds) and their full and switched versions.

- **A** HEAD's rules on HEAD's data and **B** ticket 55's rules on HEAD's data: **byte-identical** on all 1,197 Builds A builds (HEAD's rules can't name the new slot, so the 16 ticket-55 Builds are B's only).
- **B** vs **C** (ticket 55's rules and data): 1,050 identical; the 163 that differ all pick Extra Feat or Extra Mythic Ability (96 bases, mostly full Builds whose generator picked one at an even rank and then filled its slot), but one: the impossible "Emergency Potion at MR 2, Toughness in its extra slot", where only MR 2's picker differs, with one more open option (Extra Feat, which gives that slot). Where the picks are the same and both sides answer possible alike, the only new slots are 5 `MythicExtra` ones and every other slot answers the same. C has 161 MythicExtra slots in 148 Builds; every full Build picking either is possible; the ticket-55 Builds are possible in C but the two meant not to be.
- **Ticket 48's shortcut stays exact:** on the 24 ticket-55 Builds, each of 47,030 feat options a feat slot accepts (the extra slots' included) is open exactly when `allows` keeps the Build with it picked.
- Run time is the same (48 s for each of A, B and C, 8 threads).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t55/ui-check.js`, `t55/ui-check.txt`), Mythic track:

- A fighter 7's link: "MR 2 Level 7 · MYTHIC FEAT Extra Feat · FEAT Toughness".
- A sorcerer 7's: "MR 2 Level 7 · MYTHIC FEAT Extra Mythic Ability · MYTHIC ABILITY Second Bloodline · BLOODLINE Abyssal".
- A fighter 7 with Extra Feat and the slot empty: "FEAT +", whose picker opens titled "Feat", "Mythic rank 2", listing the feats by category chips, "Show 196 locked".
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 55's paragraph.
- `GLOSSARY.md`: **Extra pick** covers the two mythic feats.
- The map: the Decisions-so-far line.

### Verification

Ticket 53's Done when:

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 96, rules 338 + unit 4, seeds files 25 + game_data 168 + postgres 28, server 4). `cargo test --workspace -- --list` (`t55/test-list.txt`) differs from ticket 54's only by the 14 new tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass. `cargo leptos build --release`: pass (release wasm 1,476,422 B).
- `cd libs/seeds && cargo sqlx prepare --check`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source libs/seeds/migrations`, `cargo run -p seeds --bin seed`, `cargo sqlx prepare`; reseeded after the data change. `mythic_feat` has 2 rows with an `extra_pick`; 42 tables.
- The acceptance grep (`\bdata::`, `\bapp::`, `-p data`, `"app/` outside `.scratch/`) finds nothing.
- The v1 link vector is regenerated (above).
- Proof: passes (above). Browser: passes (above).
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t55/main-rs-user-edit.diff`).

### Questions for the user

1. **Witch Wandering Hex** ("You can select any hex for which you meet the prerequisites"; its "Prerequisites: Hex" isn't in the data either). It reads like a pick, but "Wandering" is also the shaman's Wandering Hex, which ticket 43 made a daily choice, and the entry doesn't say whose hexes it offers. It gives nothing here. **Recommendation:** a follow-up that gives it a slot of `witch/hex` (as Second Spirit gives a spirit), needing a hex held (the witch's or the shaman's), unless you know it's swapped daily, in which case mark it a daily choice like the shaman's.
2. **Beneficial Curse** ("You gain an additional Curse with no penalties"). Its sentence matches Second Mystery's ("You gain a second Oracle Mystery"), which the data makes a pick of `oracle/second-mystery`, but it doesn't say the curse is chosen. It gives nothing here. **Recommendation:** a follow-up that gives it a slot of the oracle's curses (`oracle/oracles-curse`, a curse not held), as Second Mystery does.
