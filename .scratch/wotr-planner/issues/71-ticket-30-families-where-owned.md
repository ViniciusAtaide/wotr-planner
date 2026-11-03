# Build ticket 30's option families where their owners are built

Type: task
Status: resolved
Blocked by: 70
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 69](69-option-families-where-owned.md)'s decisions (its recommendations taken, user, 2026-11-02). Ticket 69 kept `option_levels::apply` (ticket 30's seven families, the Primalist, `other_classes`, Second Spirit's line, Disruptive) because each archetype's decisions are read off its rows in force, known only once the rows are sparse. Record each placeholder cell as a marker when the rows are read, compare markers in `progressions::sparse_archetype_rows`, and decide each archetype right after its rows are made sparse, so nothing is labelled and then replaced. Fold in the follow-ups that edit what these families build (the rest of `follow_ups`, `option_follow_ups`, `second_follow_ups`, `option_class_skills`, `report_gaps`' scoundrel rows and `talent_follow_ups`), the Living Deity and Impossible Domain that ticket 70 kept (the Living Deity's rows are the Magic Deceiver's ways; Impossible Domain's prerequisite names the Living Deity's way, so it must exist before dangling references are dropped), and add the hooks ticket 69 found missing: skills, Perfect Mind, the Sword Saint's and half-elf's lists, Animal Focus, Champion of the Faith. If the shape allows, read the shaman hexes' "Wind" line as the shaman's Wind spirit directly and drop `Pick::SelectionKeepingFeature` (the data already says the spirit; the extraction must stay byte-identical).

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

- **2026-11-03, baseline** (before any edit, HEAD `c89342b`): a fresh extraction with HEAD's extractor into the scratchpad (`t71/before`, identical to `t70/after`), `cargo test --workspace -- --list` saved (738 tests). A throwaway dump of the touched classes' rows as `option_levels::apply` received them (removed again) showed which placeholder features a line resolves to: only the bloodrager archetypes' "Bloodline" cells (Second Bloodline's raw line) and the shaman's "Spirit" (Second Spirit's).
- **One step at a time, the extraction diffed after each** (`t71/check.sh`; all identical):
  1. The registry (`Picks::family`, `option_levels::{Taking, Cell, Planned, TablePlan}`, `Builder::plan`/`cell_of`, `grant` consulting the plan) and `PendingLists::stand_in`, then the families one by one: the cavalier's orders; the wizard's schools; the Magic Deceiver's ways, the shaman's spirits (the spirits' selection is named, so Second Spirit's line resolves to a spirit held: `second_spirit` gone) and the ranger's styles; the bloodrager's bloodlines with the Primalist's choices, and the sorcerer's.
  2. `other_classes` as `option_levels::provide_others` (Dragon Disciple, Eldritch Scion, Dragonheir Scion, Elemental Specialist; Disruptive's line in place). `option_levels::apply`, `take_family` and `drop_unused` gone.
  3. The Living Deity on the ways before they're given (`option_families::living_deity_rows`); `option_families::apply` gone.
  4. Impossible Domain with the domains (`option_families::impossible_domain`).
  5. Ticket 33/35's bloodline rules (groups, arcanas, Elemental Movement, bloodrager groups) and the Razmiri Infiltrator on the families before they're given.
  6. `option_class_skills` (options on the families, archetype cells nothing, the Sohei's list).
  7. The hooks, then ticket 32's owner facts (Zen Archer, Battle Master, Sword Saint, half-elf, Perfect Mind, skills, the slayer's combat style feats with tickets 33 and 35's), Champion of the Faith, Animal Focus. `normalise` was found to do nothing and went; the guards became checks. `option_follow_ups::apply` and `second_follow_ups::apply` gone.
  8. `report_gaps` (the Eldritch Scoundrel's dice, the sorcerers' level-1 slot, Extra Slayer Talent, the deity) and `talent_follow_ups`; both `apply`s gone.
  9. Ticket 32's weapons (where weapons are built), Armor Focus's condition and fixed values (cells as feats granted for one value).
- **Two surprises, both fixed in the new shape:** the Forester's "Animal Focus" cell found its own heading once the hunter's isn't a selection (a rule gives it the hunter's feature, as before); Extra Slayer Talent's check had to allow the Sanctified Slayer's later widening.
- **Checks:** fresh extraction, report included, byte-identical to the baseline (`diff -r t71/before t71/after`); `git diff HEAD -- libs/game-data` empty; test list identical (738); `cargo fmt --all --check`, clippy native and wasm, `cargo test --workspace` (738 pass), the `rules` wasm build and seed `--check-only` pass. `apps/backend/src/main.rs` untouched. No new files.

## Answer

**Ticket 30's seven families are built with their classes, and every follow-up this ticket covers with its owner.** `option_levels::provide` reads the sorcerer's and bloodrager's bloodlines, the wizard's schools, the cavalier's orders, the Magic Deceiver's ways, the shaman's spirits and the ranger's styles before any class is built, asserting the same guide sentences, and gives each to the class stage through `extract::picks`. Before a table of the class is read, its cells are planned (`Taking::plan`): the choice and the ability placeholders are markers, never features, and so is what a fixed option's row gives where an archetype's own table lists it. Once every table of the class is read, the family is taken on the full rows (`Taking::take`): the class's selection, each archetype's own selection where it skips levels or limits the options, the merged bonus slots as option-list slots. No placeholder, choice or covered feature is defined at any point, apart from the six bloodrager "Bloodline" cells a raw line names (decision 3). `option_levels::apply`, `option_families::apply`, `option_follow_ups::apply`, `second_follow_ups::apply`, `option_class_skills::apply`, `report_gaps::apply` and `talent_follow_ups::apply` are gone, and so is `drop_unused`.

**Why the full rows rather than the sparse ones** (the ticket proposed deciding right after the rows are made sparse). The decisions can be made earlier, once every table is read. An archetype row that would be dropped as its class's holds exactly the class's cells, markers included. So "a level whose row differs from the class's and has no placeholder" is computed the same way on the full rows, by the same comparison `sparse_archetype_rows` makes, with the markers counted as the features they'd be. `sparse_archetype_rows` then runs as for every class, and drops the rows the edits made the class's. The extraction is byte-identical, so that equivalence holds on the real data.

**What moved where:**
- **`option_levels`:**
  - The seven families, the Primalist's choices (`Takes::row_choices`), Bloodline Ascendance and the mythic picks are given in `provide`.
  - Dragon Disciple, the Eldritch Scion, the Dragonheir Scion, the Elemental Specialist and Disruptive are given in `provide_others`. Dragon Disciple's entry is stated for the class. Its bloodline feats are option-list slots from the cell. "Strength +2" and the others carry their ability bonus (`FeatureFact`). Disruptive's line is stated in place.
  - Second Spirit's line resolves to a spirit held, since the spirits' selection is named.
  - The Winter Witch's stand-in guard is now in `check`.
- **`option_families`:** the Living Deity's rows on the ways (`living_deity_rows`) and its three selections. Impossible Domain is built beside the domains: its line also accepts the Living Deity's way and Pentamic Faith (`Picks::alternatives`, now also for a line naming one thing).
- **`option_follow_ups`:**
  - On the sorcerer's bloodlines before they're given (`sorcerer_bloodlines`): the groups, the arcanas and Elemental Movement. The Crossblooded's "Bloodline 2 Arcana" cell is nothing.
  - Champion of the Faith is a condition of each alignment blessing.
  - Animal Focus: the heading isn't read as a selection, the feature is a daily choice, and the Forester's cell is the hunter's feature.
  - `features_named_exist` is in `check`.
- **`second_follow_ups`:** the bloodrager groups are on the family. The half-elf and no-druid guards and the Pentamic Faith row assertion are in `check`.
- **`option_class_skills`:** the bloodline and order entries are on the families (`option_rows`). The Empyreal, Sylvan, Overwhelming Mage and Seeker cells are `Pick::Nothing`. The Sohei's list is its class's and Lore (Nature) (`ArchetypeSkills::ClassAnd`). The rest is in `check`.
- **`report_gaps`:** the Eldritch Scoundrel's dice (`Picks::row_feature`). The sorcerers' level-1 slot is the base row's (`Picks::row_slot`) and the archetypes' "Bonus Feat" cell. Extra Slayer Talent's line is stated in place. The deity assertion runs when the providers are read.
- **`talent_follow_ups`:** the Skill Focus talent's and Combat Trick's repeats (`Picks::option_repeat`), Combat Trick's row and the offering selections' option levels. `shared_options` levels its two new selections (`NEW_LEVELS`).
- **`follow_ups`:**
  - Where weapons are built: the weapons.
  - The Tortured Crusader's skills (`ArchetypeSkills::Own`), and the Arcane Trickster's and Dragon Disciple's class skills (`Picks::class_skills`).
  - Perfect Mind (`FeatureFact`).
  - The Sword Saint's list (a cell rule and a list).
  - The half-elf's list (`Picks::race_list`: the races are built after the providers are read).
  - The Zen Archer's bow and the Battle Master (`Picks::option_parameter`).
  - The slayer's combat style feats, built in their final shape (no intermediate lists).
  - Armor Focus's condition (stated for the feat).
  - The fixed values (`Pick::FeatsFor`). The Mantis Zealot's proficiency cell gives its feats and the saber's Exotic Weapon Proficiency.
  - The Razmiri Infiltrator's alignment is on the ways.

**Hooks added:**
- In `Picks`: `family`/`family_of`, `feature_fact`/`shape`, `archetype_skills`, `class_skills`, `race_list`, `option_condition`, `option_parameter`, `option_repeat`, `not_selection`, `row_feature` and `row_slot`.
- New `Pick` variants: `Pick::FeatsFor` and `Pick::Nothing`.
- A slot cell ("… Feat") now consults the rules before it's read as bonus slots.
- `stated_for` is applied to prestige classes and feats.
- `option_levels_of` is applied to provided selections.
- `sparse_archetype_rows` compares `feats_for`.
- `references::prerequisites` (read-only).

**Tests:** the test list is unchanged (738), so no test was moved or renamed.

**Decisions made by recommendation (the user is asleep):**
1. The archetype decisions are made on the full rows once every table of the class is read, not after the rows are made sparse: the same result, without a pass (see above).
2. **Merged bonus lists.** The merged lists still count when a class's lists are keyed, but aren't defined (`PendingLists::stand_in`). Otherwise the Stormwalker's Archery lists would lose their `-2b`/`-6b`/`-10b` keys.
3. **The bloodrager archetypes' "Bloodline" cells** stay defined off the rows (`Planned::Marker { kept: true }`). The raw extraction's Second Bloodline line resolves to them; `libs/game-data/` has a hand fix there.
4. **"Wind" and the Divine Hunter's "Domain" stay `Pick::SelectionKeepingFeature`.** Reading the shaman hexes' "Wind" line as the Wind spirit changes the raw extraction (`Feature("oracle/wind-whisperer/wind")` becomes `SelectionOption("shaman/spirit/wind")`), which byte identity forbids. `libs/game-data/` already says the spirit.
5. **The `settled` report filters are kept as they were.** Some entries now match nothing: Disruptive's line, Extra Slayer Talent's, and five of the six sorcerer "Bonus Feat" items, which are no longer reported. Pruning them means editing `tests/extract/report_gaps.rs`; that's left for a later ticket.
6. **Ticket 32's repeats and mythic values stay in `follow_ups::apply`** (see the list below).

**Passes still in `build.rs`** (the end state), each with its reason:
1. `prose::apply`: joins what several owners' pages define (lists folded into another class's selection, slots and gates the tables can't say, the Trickster's tiers).
2. `place_features`: a feature is defined where a table first names it, and goes to its owner once every table is read.
3. `prose::path_rules`: the paths' prose facts (Legend's switch features, path feature bonuses, the Gold Dragon's list), on placed features. It could go through `FeatureFact` in a later ticket.
4. `resolve_prerequisites`: a line names what any page defines.
5. `references::drop_dangling`: drops and reports a cell or line naming what no page defines.
6. `references::drop_redundant_alternatives`: needs every option's feats.
7. `feat_lists::grant_lone_feats`: a list of one feat is known once dangling feats are gone.
8. `shared_options::apply`: works on resolved prerequisites and full rows.
9. `parameters::apply`: rewrites resolved prerequisites by value.
10. `progressions::sparse_archetype_rows`: needs complete grants.
11. `feat_lists::name_lists`: needs sparse rows.
12. `parameters::add_lists`: added after the lists are named.
13. `follow_ups::apply`, ticket 32's repeats and mythic values. Repeats act on the parameters `parameters::apply` gives the feats; the same-value rule rewrites resolved prerequisites, as `parameters::apply` does. [Fold both into `parameters`' tables (`FEAT_SHAPES`, `PREREQUISITES`), with `tests/extract/parameters.rs` updated: it pins `FEAT_SHAPES`' `Once` for Improved Critical.]
14. `backgrounds`: not a patch. It builds its own owner from the user's list and names the finished data's skills and parameterised feats' values.
15. `extra_picks`: needs every selection.
16. `borrowed_selections::apply`: widens resolved prerequisites (advanced talents, hexes, Extra feats) and the talents' option levels. [Its option levels could now be stated where the talents are built; the prerequisite widening needs resolution.]
17. `borrowed_extra_picks`: needs the rows in force and ticket 49's extra picks.
18. At the end: `picks::check` (read-only), the `settled` report filter, and `data.check()`.

**For the user:**
- Whether to open the two small follow-ups above: folding ticket 32's repeats and mythic values into `parameters`' tables, and pruning the dead `settled` entries.
- Whether to fix "Wind" in the raw extraction as a data change.

Nothing blocks.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run). The nine decisions above stand. Folding ticket 32's repeats and mythic values into `parameters`' tables, and pruning the `settled` filter entries that now match nothing, go to [ticket 72](72-parameters-and-settled-cleanup.md). Reading the shaman hexes' "Wind" line as the shaman's Wind spirit in the raw extraction is left for the user: it changes the extraction (not `libs/game-data/`, which already says the spirit), and no recommendation was made.

**Lead, 2026-11-03:** verified independently; a fresh extraction is byte-identical to HEAD-before-ticket-68's.
