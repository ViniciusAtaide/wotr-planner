# Build the nature bond, domains and proficiencies where their owners are built

Type: task
Status: resolved
Blocked by: 69
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 68](68-build-picks-where-owned.md)'s follow-up (its recommendations taken, user, 2026-11-02). Two passes ticket 68 kept still patch finished data:

- **`class_rules::nature_bond`** is label-then-replace. The level-1 companion feature it removes is still what Boon Companion's and Mythical Beast's guide lines resolve to, and a later pass removes that feature. Build the nature bond's slot where the class is built, and resolve those two lines to what they mean without the intermediate feature.
- **`proficiencies`** drops a proficiency feature its feats cover, on the full rows, and its tests drive it on hand-made data. Build only the facts that survive, where the class is built. Keep or adapt its tests so the same cases are covered.

- **The domains** that ticket 69 kept in `option_families::apply`: the druid's domain hangs off the Nature Bond, and the Living Deity, Domain Zealot, Boon Companion and Mythical Beast follow it (ticket 69's decision).

Also fold in any pass ticket 69 kept that this shape now allows.

## Build

AFK; the only agent in the repo. A refactor of `libs/extract` only: no change to `rules`, the schema, `libs/game-data/` or the link format. Use ticket 68's `Picks` registry (`libs/extract/src/picks.rs`) and its provider shape (`provide` reading the guide with its assertions, `check` read-only at the end). Keep every guide-sentence assertion, moved to where its fact is built. The user is asleep and said to take every recommendation (2026-11-02): where a pass can't sensibly move, keep it with a comment giving the reason, record your recommendation, and go on.

**Done when:**
- `libs/game-data/` is byte-identical (`git diff HEAD -- libs/game-data` empty).
- A fresh extraction into the scratchpad, report included, is byte-identical (`diff -r`) to one made with HEAD's extractor before the change. Re-check after each step.
- `cargo test --workspace -- --list` is unchanged apart from moved names, each explained in the Answer.
- No label-then-replace pass is left for what this ticket covers; any pass kept is listed in the Answer with its reason.
- The lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- **2026-11-03, baseline** (before any edit, HEAD `fd4b116`): a fresh extraction with HEAD's extractor into the scratchpad (`t70/before`), `cargo test --workspace -- --list` saved (738 tests). A throwaway dump of the rows before and after the old passes (removed again) showed every archetype's domain and companion cells carry keys of their own, so editing the full rows and letting `sparse_archetype_rows` drop the rows that match is the same as the old pass's edit-then-`normalise`.
- **One step at a time, the extraction diffed after each** (all identical):
  1. `proficiencies` as a provider: a proficiency cell is `Pick::Feats` (covered: the feats, no feature) or `Pick::FeatureAndFeats`; the kineticist's and the Purifier's feats are row feats (`Picks::row_feats`).
  2. The Nature Bond as `class_rules::provide`: the level-1 domain cell is the pick, the companion cell the option's row; Boon Companion's and Mythical Beast's line also accepts the companion option (`Picks::alternatives`).
  3. The domains as `option_families::provide_domains`, with ticket 33's Separatist (31-1) folded in.
  4. Ticket 33's Divine Hunter (31-2): no skipped levels from the start.
  5. Ticket 37's domain class skills and ticket 38's Animal domain move: on the domains' rows from the start.
  6. Ticket 35's Separatist alignment domains (33-6): on the domain options from the start.
- **One try reverted:** Impossible Domain (tickets 33 and 35) into the domain provider. Its prerequisite names the Living Deity's way, which doesn't exist until `option_levels::apply`, so `drop_dangling` dropped the whole `Any` (the run panicked in ticket 36's guard). It stays in the follow-ups.
- **Checks:** fresh extraction, report included, byte-identical to the baseline (`diff -r t70/before t70/after`); `git diff HEAD -- libs/game-data` empty; test list identical (738); `cargo fmt --all --check`, clippy native and wasm, `cargo test --workspace` (738 pass), the `rules` wasm build and seed `--check-only` pass. `apps/backend/src/main.rs` untouched.

## Answer

**The nature bond, the domains and the proficiencies are built with their classes.** No label feature is made for them: no "… Proficiencies" feature that its feats cover, no druid "Domain Abilities", no cleric, inquisitor or druid "Domain Abilities"/"Domain Ability"/"Lightning Lord" cell feature, no "Animal domain only" companion, no Separatist second-domain cell, no domain "Class Skill: …" feature.

**What moved where:**
- **`proficiencies::apply` → `proficiencies::provide`/`check`.** A cell naming a proficiency feature is `Pick::Feats` (a feature its feats cover in full: the feats alone) or `Pick::FeatureAndFeats` (the feature and its feats). The Herald Caller's and the Eldritch Scoundrel's overrides are rules for their tables. The magus's armour features are rules per table, and `check` asserts that none is missed and that each row in force holding one holds its feat. The kineticist's and the Purifier's feats are row feats (`Picks::row_feats`, `Rows::Of`/`Rows::EveryArchetypeOf`).
- **`class_rules::nature_bond` → `class_rules::provide`/`check`.** In the druid's, the Defender's and the Feyspeaker's level-1 rows, the domain cell is the Nature Bond pick and the companion cell is `Pick::OptionRow`. The selection is the druid's from the start, its domain option granting `druid/domain`, levelled by druid levels. Boon Companion's and Mythical Beast's "Animal Companion" line resolves to the features the tables still define, plus what `Picks::alternatives` says it also accepts: the companion option.
- **`option_families::take_domains` and `prerequisites` → `option_families::provide_domains`/`check_domains`.**
  - The cleric's and the inquisitor's tables (every table the class stage reads, `build::class_tables`): the domain cells at 1 are the picks (the Sacred Huntsmaster's its own), and the companion at 4 and the powers at 6 and 8 are `Pick::OptionRow`.
  - The druid's, the Defender's and the Feyspeaker's cells at 4, 6 and 8, and the Blight Druid's and the Winter Child's cells, likewise.
  - The domain selections (cleric, inquisitor, Sacred Huntsmaster, druid, Blight Druid, Winter Child, Divine Hunter) are provided.
  - Domain Zealot's "Domain" is stated in place (`Picks::prerequisite`, asserting the line).
  - Boon Companion and Mythical Beast also accept the Animal and Scalykind companions and the Nature revelation (`Picks::alternatives`). The "Animal domain only" companions they named before the pass are never defined now, so the line no longer resolves to them.
- **Folded in with the domains:** ticket 33's Separatist (31-1: its second domain two levels behind, row 1 granting it, its companion at 6 and Domain Abilities at 8 and 10 as its rows) and Divine Hunter (31-2: no skipped levels); ticket 35's Separatist alignment domains (33-6); ticket 37's domain class-skill entries and ticket 38's Animal domain move (the class skills are on the domains' rows from the start, the Animal domain's on its 4th).
- **Left of `option_families::apply`:** the Living Deity alone, plus dropping its five labels (`drop_unused` on the arcanist only). `normalise` did nothing any more and is gone.

**Tests:** the test list is unchanged (738), so no test was moved or renamed. Two tests keep their names and cover the same cases on the provider:
- `proficiencies`' three tests now read the picks: what a cell is, whether the feature stays, and the row feats.
- `class_rules::a_druid_chooses_an_animal_companion_or_a_domain_at_level_1` now reads the picks: the rules for the three tables, the selection, its domain option's row, the Primal Druid and the 4th-level companion untouched, and the companion alternative.

**Passes kept, and why** (recommendation in brackets):
1. **`option_families::apply`, the Living Deity:** its rows are the Magic Deceiver's ways, which `option_levels::apply` builds. [With ticket 71, once the ways are built in the class stage.]
2. **Impossible Domain** (`option_follow_ups::impossible_domain`, `second_follow_ups::impossible_domain`): its prerequisite names the Living Deity's way, which doesn't exist when `drop_dangling` runs (tried: it dropped the whole prerequisite). [With ticket 71, as for item 1.]
3. **`second_follow_ups::no_druid_impossible_domain`:** a read-only guard, not a patch, kept beside the rule it guards.

**Hooks added:** `Pick::Feats`, `Pick::FeatureAndFeats`, `Picks::row_feats`, `Picks::alternatives`, `Picks::unnamed`. Also `build::class_tables`, which lists a base class's tables as the class stage reads them.

**Decisions made by recommendation (the user is asleep):**
1. **The cleric's domain selection isn't a name lines resolve to** (`Picks::unnamed`). Registering it, as other provided selections are, would turn Impossible Domain's "Domain" line into 37 more alternatives. So that line still resolves to the Divine Hunter's "Domain" feature, which stays defined off the row (`Pick::SelectionKeepingFeature`, as the Wind Whisperer's "Wind"). Domain Zealot's line is stated in place.
2. **The cleric and inquisitor rules are generated per table** from the cell slugs ("domain-abilities(-1/-2)", "domain-ability(-1/-2)", "animal-companion"), as the old pass matched by name. `check_domains` asserts that no such feature is left and that every cleric table picks a domain at 1.
3. **Only Boon Companion's and Mythical Beast's "Animal Companion" gets the alternatives.** The resolver still finds the companions the tables define, rather than having the whole line stated in place.
4. **The magus's armour copies are listed per table**, as the module's `FEATURES` lists the bloodrager copies, with `check` guarding against a missed one.
5. **The Nature Bond stays in `class_rules`.** It is built there whole (its domain option's row and levels included), and the domains provider only provides `druid/domain`.

**For the user:** in `libs/game-data/`, Impossible Domain already needs a cleric, an inquisitor, the Divine Hunter archetype, the Living Deity or Pentamic Faith (a hand fix). Only a raw extraction has the Divine Hunter's "Domain" feature there, like ticket 69's "Wind". Ticket 71 may make the extraction read it that way once the Living Deity's way exists in the class stage. Nothing blocks.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run). The five decisions above stand; the Living Deity and Impossible Domain go to [ticket 71](71-ticket-30-families-where-owned.md), with the Magic Deceiver's ways they hang off.

**Lead, 2026-11-03:** verified independently; a fresh extraction is byte-identical to HEAD-before-ticket-68's.
