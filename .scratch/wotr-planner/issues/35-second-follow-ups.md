# Second round of follow-ups

Type: task
Status: resolved
Blocked by: 33
Map: [WotR Build Planner](../map.md)

## Question

Build the user's decisions (2026-11-01, "all recommended") from the open questions of tickets [32](32-feat-and-rule-follow-ups.md) and [33](33-option-follow-ups.md). Each ticket's `## Decisions on the open questions` is the spec.

- **Half-elf subraces** (32-1): Dual Heritage and Kindred Raised don't get Adaptability.
- **Impossible Domain** (33-2): a Living Deity's extra domain is levelled by arcanist levels from the pick; a Hellknight with Pentamic Faith's domain qualifies, levelled by Hellknight levels.
- **Signifer's order** (33-3): offered only to a character holding no order.
- **Separatist** (33-6): no alignment domain that conflicts with its alignment.
- **Ranger/slayer** (33-7): one shared style; the slayer's feats come from its talent window's stage.
- **Bloodrager groups** (33-8): its draconic and elemental bloodlines are grouped.

Data from the saved guide pages only (offline). If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a Dual Heritage half-elf has no Adaptability pick; a Living Deity with Impossible Domain holds the third domain's level-1 power before arcanist 20; a Hellknight 3 with Pentamic Faith's domain can take Impossible Domain; a Hellknight who becomes a Signifer gets no second order; a lawful good Separatist can't take Chaos or Evil; a ranger 10 / slayer 2 takes the slayer's 2-or-4 feat from the first stage; a bloodrager with a dragon bloodline can't take a second dragon through Second Bloodline.

## Progress log

- 2026-11-01: started. Baseline: HEAD 5795e00, clean tree; scratch in `t35/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t35/`), `old-src/` is `git archive HEAD`. Guide read offline (`t24/pages/*.txt`). Milestones: (1) `rules`, test world first; (2) schema, seed, loader, fixture; (3) data through a new `extract::second_follow_ups` module run over `game-data/` by a one-off `t35/rewrite/`, real-data tests first; (4) extractor wiring and a fresh extraction; (5) proof, docs, DB reset, Verification. Shape decided:
  - **Subraces:** the guide lists each subrace's own racial traits (Dual Heritage and Kindred Raised list no Adaptability), so a subrace's feat slots replace the race's, as its ability modifiers do: `Subrace.feat_slots` (none in the data: the two half-elf subraces get no slot). Its slots may name the race's lists.
  - **Impossible Domain:** `FeatureSelection.level_also_from: Option<SelectionOptionKey>`, only beside `level_from: OtherOptions`: that option's level counts as one of the other options' (`cleric/impossible-domain`: the Living Deity's way, by arcanist levels from the pick). The Hellknight's Pentamic Faith domain is levelled by Hellknight levels and is one of the options Impossible Domain offers, so `OtherOptions` already gives it; its prerequisite accepts the Pentamic Faith discipline (whose row grants the domain).
  - **Signifer:** `hellknight-signifer/order` gets `only_without_option`, as the Hellknight's.
  - **Separatist:** the Chaos, Evil, Good and Law domains: `Any([Not(PickedThrough("cleric/separatist/domain")), the six alignments that don't conflict])`, as the Champion's blessings.
  - **Ranger/slayer:** `FeatSlot::OptionList { selection, at_level: Option<u8> }`: the list in force at that option level of each option held that the selection offers, whatever level the option has reached. The slayer's three talents' slots name their window's stage (2, 6, 10).
  - **Bloodrager groups:** new `OptionGroup::BloodragerDraconic` and `BloodragerElemental` (the guide's "single draconic bloodline" is the sorcerer's, about its arcana; the decision groups the bloodrager's own, so a sorcerer's and a bloodrager's dragon don't exclude each other: open question), on the bloodrager's 10 dragons and 4 elements with `NoOtherOptionOfGroup`.
- 2026-11-01: milestone 1 (`rules`) done. Types: `Subrace.feat_slots`, `FeatSlot::OptionList.at_level`, `FeatureSelection.level_also_from`, `OptionGroup::BloodragerDraconic`/`BloodragerElemental`, `DataIssue::LevelAlsoFromWithoutOtherOptions`. Engine: level 1's race slots are the subrace's when one is chosen (`race_slots`); `LevelFrom::OtherOptions` counts the `level_also_from` option's holding beside the other options; an `OptionList` slot with `at_level` offers the list in force at that option level of each option held that its selection offers. Check: a subrace's slots may name only its race's bonus lists (`MisplacedBonusList` at the subrace); `level_also_from` only beside `OtherOptions`. Tests first: the world gains two half-elf subraces (Dual Heritage with no slot, an invented Adaptable keeping the race's), a False God witch archetype (the world's Living Deity: a way levelled by witch levels, granted as its selection's one option) that Impossible Domain levels beside the domains held, and the slayer's three talents' slots at their stage; new `second_follow_ups.rs` (4) and 2 check tests. Red run (`t35/rules-red.txt`): 7 failures, the 5 new tests that need code and 2 listings (archetype count, class options with the False God; updated); a subrace's own slot naming the race's list passed at once. The Living Deity test first failed on a pick into a slot the one-option grant never opened; fixed, it was red against the engine without the `level_also_from` hunk (option level 0, not 6). Bloodrager groups, the Signifer and the Separatist need no engine change (data, tested on the real data). `cargo test -p rules` 263 + 1, clippy clean. The `Build` type is unchanged: the v1 link vector stays. Next: schema, seed, loader.
- 2026-11-01: milestone 2 (`data`) done. Migration: domain `option_group` gains `bloodrager_draconic`, `bloodrager_elemental`; `feature_selection.level_also_from_key` (FK `selection_option`, added by an `ALTER` since that table comes later; CHECK: only with `level_from = 'other_options'`); `feat_slot.subrace_key` (a fifth owner, exactly one) and `at_level` (u8, CHECK: only on `option_list`). Still 40 tables. Seed (`SlotOwner::Subrace`), loader (a subrace's slots, a slot's `at_level`, a selection's `level_also_from`), `columns` (the two groups). `rules`: `at_level` is left out when serialised while unset (`skip_serializing_if`; `FeatSlot` is in no link), so the files keep writing `OptionList(selection: …)`; `ron_out` writes a subrace's `feat_slots` and a selection's `level_also_from`. Fixture: the half-elf's Dual Heritage (no slot) and an invented Adaptable subrace naming the race's list, the Abyssal powers counting the Seeker's Power beside the other options, a second `OptionList` at option level 1 on the sorcerer's row 3. Tests first: files (1 new, 1 updated for the second slot), postgres (the bloodrager groups in every-variant; a CHECK/FK test of 7 statements). Red run with a loader that defaulted the new fields (`t35/data-red.txt`): 7 postgres failures (the files test passed at once: reading the RON is serde's, done in milestone 1). Dev DB reset with `t35/reset-db.sh --no-seed` (only `wotr-planner-postgres`, 5434). `cargo test -p data` 24 + 94 + 22, `-p extract` 83, `-p rules` 263 + 1 pass; `--check-only` passes on `game-data/`. Next: the data.
- 2026-11-01: milestone 3 (data) done. One implementation: new `extract::second_follow_ups::apply(guide, data)` (`option_follow_ups`' `class_selection` and `and` made `pub(crate)`), each rule asserting the guide phrase it rests on and what it finds, run over `game-data/` by the one-off `t35/rewrite/` (Rust, from `t35/old-src/game-data/`; dry run in `t35/dry/game-data/`, byte-identical to the real run; log `t35/game-data-log.json`, 26 changes in 5 files; 0 check problems). The half-elf rule changes no data (an extraction gives a subrace no slots; the guide's subrace tables list no Adaptability) and only logs it. `ron_out` writes a feat slot list itself so a fixed stage is `at_level: 2` (as the files read it), not `Some(2)`. Real-data tests first (`game_data.rs`, 7 new; red on HEAD's data: 6 failures, `t35/real-data-red.txt`; the half-elf test passed at once, since a subrace's slots replacing the race's is the engine's rule from milestone 1); 1 older test's comment updated (the Living Deity's third domain: arcanist levels). `cargo test --workspace` passes (rules 263 + 1, extract 83, data 24 + 101 + 22). Next: extractor wiring, a fresh extraction.
- 2026-11-01: milestone 4 (extractor) done: `extract` runs `second_follow_ups::apply` after `option_follow_ups::apply` (it settles no item of an extraction's report). A fresh extraction (`t35/extract-new/`) is **byte-identical** (RON and report, 33 items) to `rewrite` applied to HEAD's extraction (`t33/extract-new/` → `t35/dry/extract-old/`, 26 changes, the same log as `game-data/`'s). Tests: new `extract/tests/extract/second_follow_ups.rs` (1: the files write a subrace's slots, a fixed `at_level` as a number, `level_also_from` and a bloodrager group, leave unset ones out, and read back), written after the code (the dry run had found the `Some(2)`). `cargo test -p extract` 84 pass. Next: proof, docs, DB reset, Verification.
- 2026-11-01: milestone 5 done. Proof (`t35/proof-{old,new}` on one shared `t35/proof-common.rs`, ticket 33's plus 10 targeted Builds, one or more per change; `t35/compare.py`) passes. Docs: `game-data/README.md` (ticket 35's shapes, the slayer's `at_level`), `GLOSSARY.md` (Build start, Option level, Option list slot, Option group, Prerequisite updated; no new term), `game-data/EXTRACTION_REPORT.md` unchanged (no item settled). Final DB reset and reseed (`t35/reset-db.sh`), `cargo sqlx prepare` (4 query files new, 4 gone); every Verification check passes. Nothing staged. Done.

## Answer

Done 2026-11-01, uncommitted. Every decision in the Question is built. A half-elf's Dual Heritage and Kindred Raised subraces get no Adaptability: a subrace's feat slots replace its race's, as its ability modifiers do, and the guide lists none for them. Impossible Domain levels a Living Deity's extra domain by arcanist levels from the pick (its level-1 power at character level 6, its level-8 power at arcanist 8), and a Hellknight holding Pentamic Faith's domain may take it, its domain levelled by Hellknight levels (a Hellknight 3's is level 3). The Signifer's order is offered only to a character holding none, so a Hellknight who becomes a Signifer keeps his order and picks no second one. A Separatist's second domain is no alignment domain that conflicts with his alignment (a lawful good one can't take Chaos or Evil; his first domain isn't checked here). A ranger/slayer shares the ranger's style, and each slayer combat style feat comes from the style's list of the talent's window's stage (a ranger 10 / slayer 2 takes the 2-or-4 feat from Archery's first list). The bloodrager's ten Draconic and four Elemental bloodlines are groups, so Second Bloodrager Bloodline can't add a second dragon or element. The `Build` type is unchanged, so the v1 link vector stays.

### Rust (`rules`)

- **A subrace's slots.** `Subrace.feat_slots`: the subrace's slots at level 1, in place of the race's (`Walker::race_slots`). The check lets them name only the race's bonus lists (`MisplacedBonusList` at the subrace).
- **An option counted beside the others.** `FeatureSelection.level_also_from: Option<SelectionOptionKey>`, only beside `level_from: OtherOptions` (`LevelAlsoFromWithoutOtherOptions` otherwise): that option's holding counts among the other options held, so the selection's option takes its level when it's the highest (Impossible Domain: the Living Deity's way, levelled by arcanist).
- **An option list slot at a fixed stage.** `FeatSlot::OptionList { selection, at_level: Option<u8> }`: with `at_level`, the slot offers each held option's list in force at that option level, not at the level the option has reached. Left out when serialised while unset (`skip_serializing_if`; `FeatSlot` is in no link).
- **Groups.** `OptionGroup::BloodragerDraconic` and `BloodragerElemental` ("Draconic (Bloodrager)", "Elemental (Bloodrager)").
- Tests first, in the test world (new `second_follow_ups.rs`, 4, plus 2 check tests): two half-elf subraces (Dual Heritage with no slot, an invented Adaptable naming the race's list), a False God witch archetype (the world's Living Deity, its way granted as its selection's one option) that Impossible Domain levels, the slayer's three talents' slots at their stage with a ranger 6 / slayer 2. Red run: 7 failures (the 5 new tests that need code, 2 listings updated); a subrace's own slot passed at once. The Living Deity test was red against the engine without the `level_also_from` hunk (option level 0, not 6).

### Schema (the one migration)

- Domain `option_group` gains `bloodrager_draconic`, `bloodrager_elemental`.
- `feature_selection.level_also_from_key` (FK `selection_option`, by an `ALTER`, as that table comes later; CHECK: only with `level_from = 'other_options'`).
- `feat_slot.subrace_key` (a fifth owner, exactly one) and `at_level` (u8, CHECK: only on `option_list`).
- Still 40 tables. Seed (`SlotOwner::Subrace`), loader, `columns`. Tests: the fixture has every new shape (a subrace with no slot and one naming its race's list, a selection counting another option beside the others, an `OptionList` at option level 1); files (1 new, 1 updated), postgres (the bloodrager groups in every-variant; a CHECK/FK test of 7 statements). Red run with a loader that defaulted the fields: 7 postgres failures (`t35/data-red.txt`).

### Data rewrite

One implementation: `extract::second_follow_ups::apply(guide, data)`; each rule asserts the guide phrase it rests on (the user's decisions aside) and what it finds. The one-off `t35/rewrite/` (Rust, in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t35/`) ran it from `t35/old-src/game-data/` and wrote only the 5 files whose data changed (`t35/game-data-log.json`, 26 changes; byte-identical to the dry run in `t35/dry/game-data/`). The extractor was never run over `game-data/`.

| Change | Real data | Guide |
| --- | --- | --- |
| half-elf subraces | none: an extraction gives a subrace no slots; the rule checks the guide and logs it | Dual Heritage's and Kindred Raised's "Racial Traits" tables list no Adaptability |
| Impossible Domain | `cleric/impossible-domain`: `level_also_from: "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity"`; the ability's prerequisite gains `SelectionOption("hellknight/disciplines/pentamic-faith")` | "Prerequisites: Domain"; Pentamic Faith "uses her class level to determine which domain abilities she can use"; arcanist levels by the user's decision |
| Signifer | `hellknight-signifer/order`: `only_without_option` | the Hellknight's "Otherwise, they retain their current order." (the Signifer's by the user's decision) |
| Separatist | Chaos, Evil, Good, Law domains: `Any([Not(PickedThrough("cleric/separatist/domain")), the six alignments off the opposite side])` | "This second domain cannot be an alignment domain that conflicts with the cleric's own alignment." |
| slayer | the three combat style talents' slots: `OptionList(selection: "slayer/combat-style", at_level: 2)` (6, 10) | "2 or 4, 6 or 8 and 10 or greater" |
| bloodrager | the 10 Draconic and 4 Elemental bloodlines: `group` and `NoOtherOptionOfGroup` | the user's decision (the guide states the rule for the sorcerer's) |

Real-data tests (`game_data.rs`, 7 new, red on HEAD's data first: 6 failures, `t35/real-data-red.txt`; the half-elf's passed at once, since a subrace's slots replacing the race's is the engine's rule). `ron_out` writes a feat slot list itself so the stage reads `at_level: 2`, not `Some(2)` (found by the dry run).

### Proof

`t35/proof-old/` (against `git archive HEAD`, 5795e00) and `t35/proof-new/` build one shared source, `t35/proof-common.rs` (ticket 33's, plus 10 targeted Builds: a half-elf with each subrace, a ranger 10 / slayer 2, a Living Deity 8 with Impossible Domain, a Pentamic Faith Hellknight with and without it, a lawful good Separatist taking Chaos, a bloodrager taking a second dragon, a Hellknight who becomes a Signifer); `t35/compare.py` checks:

- **Data** (206 checks): each change above is checked and undone; with the new fields' defaults dropped, what's left **equals the old data**, and the 35 untouched classes are identical.
- **Walk:** every base class and archetype through 20 levels, each prestige class after wizard, cleric or witch 10, and the targeted Builds: 5534 of 5549 lines identical (104 option list slots print their unset stage, read as the old kind). The 15 others are 8 targeted Builds, each as expected: Dual Heritage and Kindred Raised lose the race slot; the ranger/slayer's level-12 feat slot loses Archery's later feats (Improved Critical, Improved Precise Shot, Manyshot, Point-Blank Master, Skill Focus); the Living Deity holds Nobility from 6 (option level 6, 7, 8); the Pentamic Hellknight's Impossible Domain is no longer refused and War is held at option level 3; the Separatist's Chaos is refused (and Evil closed in his slot); the bloodrager's second dragon is refused; the Signifer has no order slot. No standard Build changes.

### Extractor

- `extract` runs `second_follow_ups::apply` after `option_follow_ups::apply`; it settles no item of an extraction's report. A fresh extraction (`t35/extract-new/`) is **byte-identical** (RON and report, 33 items) to `rewrite` applied to HEAD's extraction (`t33/extract-new/` → `t35/dry/extract-old/`, the same 26 changes).
- `ron_out` writes a subrace's `feat_slots`, a selection's `level_also_from`, and an option list slot's `at_level` as a number.
- Tests: `second_follow_ups.rs` (1: the files write every new shape, leave unset ones out, and read back), written after the code.

### Docs

- `game-data/README.md`: ticket 35's shapes, the slayer's `at_level`.
- `GLOSSARY.md`: Build start (a subrace's slots), Option level (an option counted beside the others), Option list slot (the window's stage), Option group (the bloodrager's), Prerequisite (the Separatist's second domain). No new term.
- `game-data/EXTRACTION_REPORT.md`: unchanged.

### Verification

- `cargo test --workspace`: pass (rules 263 + unit 1, extract 84, data files 24 + game_data 101 + postgres 22).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (4 query files new, 4 gone).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434 (`t35/reset-db.sh`): drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths. Rows: 40 tables; options with a group 14 → 28, `no_other_option_of_group` nodes 14 → 28, `picked_through` nodes 4 → 8, selections with `level_also_from` 1, `only_without_option` selections 3 → 4, feat slots with `at_level` 3, subrace slots 0.
- The v1 link vector is unchanged (the `Build` type didn't change).
- Proof: passes (above).

### Open questions (for the user)

1. **The bloodrager's groups are apart from the sorcerer's.** The guide's "single draconic bloodline" is the sorcerer's (about doubling the Bloodline Arcana); the decision groups the bloodrager's own. So a sorcerer/bloodrager may hold a sorcerer dragon and a bloodrager dragon. Should one Draconic group span both classes?
2. **Impossible Domain's levels with two sources.** A Living Deity with cleric levels too takes the higher of its arcanist levels and its cleric domains' levels. Fine? (Equal at arcanist 20, where the way's own domains come.)
3. **Who else has a "Domain".** The prerequisite now accepts a cleric, inquisitor, Divine Hunter, Living Deity or Pentamic Faith Hellknight (by the discipline whose row grants the domain). Other domain holders (a druid's Nature Bond domain, the Blight Druid, the Winter Child) still don't qualify. Should they ("Prerequisites: Domain")?

## Decisions on the open questions

**User, 2026-11-01 ("all recommended"):**
1. The sorcerer's and bloodrager's groups stay apart: the guide states the one-dragon rule only for the sorcerer.
2. A Living Deity with cleric levels levels its extra domain by the higher of the two, as built.
3. The druid's Nature Bond domain, the Blight Druid's and the Winter Child's domains qualify for Impossible Domain ("Prerequisites: Domain"), its domain levelled by druid levels. [Ticket 36](36-impossible-domain-druids.md).

**User, 2026-11-01 (reversing decision 3):** the druid page says "If you choose a domain, you can't take the Impossible Domain mythic ability", so no druid domain qualifies. [Ticket 36](36-impossible-domain-druids.md) pins it with tests and changes no data.
