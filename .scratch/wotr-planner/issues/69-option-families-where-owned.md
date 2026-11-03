# Build the option families where their owners are built

Type: task
Status: resolved
Blocked by: 68
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 68](68-build-picks-where-owned.md)'s follow-up (user, 2026-11-03: "we should patch the extraction data directly in code and not in runtime"; its recommendations taken, 2026-11-02). Ticket 68 left the largest label-then-replace passes in `libs/extract/src/build.rs`:

- `option_levels` and `option_families` (tickets 30–31, about 4,400 lines). They replace cells in several classes' rows in force at once, and they depend on which rows are sparse.
- `Picks::late`: the wild talents' and infusions' option prerequisites name element options that `option_families` builds, so today they're set right after it.
- The follow-ups that edit what the families build: `follow_ups`, `option_follow_ups`, `second_follow_ups`, the backgrounds, `option_class_skills`, `report_gaps` and `talent_follow_ups`.

Move each to where its owner is built, through the `Picks` registry or an owner-stage provider, so no label is created and then replaced. Where the sparse-rows order really matters, say how the new shape handles it.

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

- **2026-11-03, baseline** (before any edit, HEAD `d180e06`): a fresh extraction with HEAD's extractor into the scratchpad (`t69/before`; a second run diffed identical), `cargo test --workspace -- --list` saved (738 tests).
- **Registry:** `Pick::OptionRow` (a placeholder cell is the held option's row: nothing on the owner's row); a rule may name the guide selection a cell grants; option levels and option rows of a selection the guide's headings define; a selection a row grants beside its cells; a feature that is a daily choice; a mythic ability's features; an option's stated conditions, joined to its resolved lines. The prestige stage takes provided selections and lists, as the class stage does. `Picks::read` takes the skills.
- **One step per family, the extraction diffed after each** (all identical): the kineticist's elements (ticket 31), with ticket 33's Expanded Element and ticket 37's element class skills, and `Picks::late` removed; the warpriest's blessings; the Hellknight's order and Pentamic Faith, with tickets 33 and 35's Signifer; the oracle's mysteries, with ticket 33's revelations (Cinder Dance, the Dual-Cursed Oracle's, Maneuver Mastery, Second Mystery) and ticket 37's mystery class skills; `option_levels`' owner facts (`spontaneous` into the class stage; Second Bloodline, Second Bloodrager Bloodline and Second Spirit's picks; Bloodline Ascendance) and the daily choices of tickets 30, 33 and 43; ticket 43's totem rage powers.
- **Two tries that differed, both reverted to the recorded shape:** Second Spirit's prerequisite stated in place of its line (the spirits' options don't exist when the lines are resolved: the report gained an item), and the totems' exclusions as extra resolver conditions (the two Beast totems came out as a nested `All`). The totems now join their resolved lines as the pass did; Second Spirit's line stays in `option_levels`.
- **Checks:** fresh extraction, report included, byte-identical to the baseline (`diff -r`); `git diff HEAD -- libs/game-data` empty; test list identical (738); `cargo fmt --all --check`, clippy native and wasm, `cargo test --workspace` (738 pass), the `rules` wasm build and seed `--check-only` pass. `apps/backend/src/main.rs` untouched.

## Answer

**Four of ticket 31's families are built with their classes.** `option_families::provide` reads the kineticist's elements, the warpriest's blessings, the oracle's mysteries and the Hellknight's order and Pentamic Faith before any class is built, asserting the same guide sentences, and gives them to the class and prestige stages through `extract::picks`. Their placeholder cells ("Class Skills", "Defensive Wild Talent", "Expanded Element", the level-10 "Blessings", "Feral Blessing", "Mystery", "Revelation", "Final Revelation", "Order") are the pick or the held option's row from the start: no label feature is made. `option_families::check` asserts the rows in force: each table's element and mystery pick at 1, Expanded Element at 7 and 15, no blessing pick at 10, the Hellknight's and the Signifer's order at 1, and the oracle's and the Dual-Cursed Oracle's revelation levels. `Picks::late` is gone: the element options exist from the class stage, so the wild talents' and infusions' prerequisites are set where their options are.

**What moved where:**
- **Elements, blessings, the Hellknight and the mysteries** (31): into `option_families::provide`. The class stage defines their selections, options, rows, features and lists.
- **Folded into those providers:**
  - from `option_follow_ups` (33): Expanded Element's `once_per_option`; the Signifer's order; the revelations (Cinder Dance's no-Lame curse, the Dual-Cursed Oracle's revelation pick, Maneuver Mastery levelled like its revelation, the Second Mystery selection and its pick);
  - from `second_follow_ups` (35): the Signifer's `only_without_option`;
  - from `option_class_skills` (37): the elements' and mysteries' class skills, now on their row 1 from the start, so no "Class Skill: …" feature is made for them.
- **Second Mystery:** its "Mystery" line is stated in place (`Picks::prerequisite`, asserting the line), and its pick is given where the mythic abilities are built.
- **`option_levels`:**
  - `spontaneous` is read where each class and archetype is built, keeping its assertion;
  - `provide` gives Second Bloodline's, Second Bloodrager Bloodline's and Second Spirit's picks and Bloodline Ascendance's feature to the mythic stage;
  - Mixed Bloodline, Second Mixed Bloodline and the Wandering Spirit are daily choices where their tables define them.
- **Daily choices:** the Unsworn Shaman's two and the hunter's Second Animal Focus (`option_follow_ups::provide`), and the Wandering Hex (`report_gaps::provide`), are built where the features are defined. The Wandering Hex's rows are asserted in `report_gaps::check`.
- **Totem rage powers** (43): stated where the rage powers are built, joined to their own lines as before.

**Passes kept, and why** (recommendation in brackets):
1. **`option_levels::apply`** (ticket 30's seven families, the Primalist, `other_classes`, Second Spirit's line, Disruptive). Whether an archetype takes a family, which option levels its own selection skips and whether its bonus slots take the held option's list are read off its rows in force: a level it has no row of is its class's, which is only known once the rows are sparse, and the merged lists it rewrites are named only then. Second Spirit's and Disruptive's lines name options this pass builds. [A follow-up ticket: record each table's placeholder cells in the class stage as markers rather than features, let `sparse_archetype_rows` compare them as row content, and take the archetype decisions right after the rows are made sparse. Then fold ticket 30's follow-ups into it:
   - from `option_follow_ups`: `option_groups`, `crossblooded`, `elemental_movement` and the slayer's combat style;
   - from `second_follow_ups`: the bloodrager groups and the slayer's window stages;
   - from `option_class_skills`: the bloodline and order entries;
   - from `report_gaps::decisions`: the sorcerer's level-1 bonus slot;
   - from `follow_ups`: the Razmiri Infiltrator's way, the Battle Master and the slayer's combat style feats.]
2. **`option_families::apply`** (the domains, the Living Deity, Domain Zealot, Boon Companion and Mythical Beast). The cleric's, inquisitor's and druid's "Animal Companion" cells at 4 are what Boon Companion's and Mythical Beast's lines resolve to before this pass rewrites them, as `class_rules::nature_bond` does. The druid's domain hangs off the Nature Bond selection that `nature_bond` builds in the postlude. The Living Deity's rows are the Magic Deceiver's ways, which `option_levels` builds. [Move the domains with [ticket 70](70-nature-bond-and-proficiencies-where-owned.md): state Boon Companion's and Mythical Beast's prerequisites in place of their lines, so the companion cells can be option rows. Then fold the domain follow-ups: the Separatist, the Divine Hunter, Impossible Domain (33 and 35), "no druid takes Impossible Domain", and `option_class_skills`' domain entries and Animal domain cleanup. The Living Deity moves with item 1.]
3. **`follow_ups`** (32). Most rules set the values and repeats of the parameterised feats, which `parameters::apply` builds on the resolved prerequisites; the rest edit facts of item 1. [Owner facts that need only a hook could move in a small ticket: the Tortured Crusader's, the Arcane Trickster's and the Dragon Disciple's skills; Perfect Mind; the Sword Saint's and the half-elf's one-feat lists; the Zen Archer's bow.]
4. **`option_follow_ups`:**
   - `daily_choices`' Animal Focus: the guide's heading lists eight aspects, so the class stage reads it as a selection [a "not a selection" hook in `Builder::selections`];
   - `champion_of_the_faith`: it adds a `Not(PickedThrough)` inside each alignment blessing's `Any`, which no hook states [a hook for an alternative ORed into a resolved prerequisite, now that the blessings are built with the class];
   - `normalise` and `features_named_exist`: tidy and check the passes above;
   - the rest are items 1 and 2.
5. **`second_follow_ups`:** the half-elf subraces, a guard on the races' slots [into the race stage]; the rest are items 1 and 2.
6. **`backgrounds`:** not a patch. It builds its own owner from the user's list, and stays here because it names the parameterised feats' values (Exotic Weapon Proficiency (Dueling Sword)).
7. **`option_class_skills`:** the bloodline, order and domain entries and `cleanup` (items 1 and 2).
8. **`report_gaps`:** the Eldritch Scoundrel's sneak attack (rows in force), and `decisions` (the sorcerer's bonus slots are item 1's; Extra Slayer Talent's line).
9. **`talent_follow_ups`:** the rogue talents' Skill Focus repeat and Combat Trick's rows and option levels, which `borrowed_selections::apply` then widens. [Fold them with `borrowed_selections` (ticket 60), through the option-row and option-levels hooks.]
10. **Ticket 68's other kept steps** stay as ticket 68 recorded: `extra_picks`, `borrowed_selections`, `borrowed_extra_picks`, and the ordered steps.
11. **One label kept, not placed:** the Wind Whisperer's "Wind" cell is its mystery pick, but the cell's feature stays defined, off the row (`Pick::SelectionKeepingFeature`), because the shaman's Air Barrier, Vortex Spells and Wind Ward resolve their "Wind" line to it.

**A finding, not changed:** in `libs/game-data/`, Air Barrier, Vortex Spells and Wind Ward need `Feature("oracle/wind-whisperer/wind")`, which no shaman can hold. Their line presumably means the shaman's Wind spirit. Fixing it is a data change, so it's out of this refactor; once it's fixed, `Pick::SelectionKeepingFeature` can go.

**Done-when:**
- Extraction diff: empty (`diff -r t69/before t69/after`, report included, after every step and at the end).
- Game-data diff: empty.
- Test-list diff: empty (738 tests), so no test was moved or renamed.
- fmt, clippy native and wasm, `cargo test --workspace` (738 pass), the `rules` wasm build and seed `--check-only` all pass.

**Decisions made by recommendation (the user is asleep):**
1. The ticket-31 providers stay in `option_families`, behind one `provide` and one `check`, beside the readers they share (`Family`, `domains`, `mysteries`), so each family keeps its sentences.
2. The follow-ups folded into a provider keep their guide-phrase assertions there; each module's doc says where its rule is now built.
3. The Wind Whisperer's label is kept defined, off the row, rather than fixing the shaman's lines: byte identity, and the fix is the user's call.
4. An option's stated conditions join its resolved prerequisite after resolution, as the pass did (an `All` is extended, not nested): adding them as extra resolver conditions changed two prerequisites' shape.
5. Daily choices are given by feature key, not by name as `option_levels::mythic` matched them: the same three features.
6. Second Spirit's prerequisite stays in `option_levels` (item 1), since its options don't exist when lines are resolved. The domains stay with ticket 70, and ticket 30's families wait on a new follow-up ticket.

**For the user:**
- whether to open the follow-up for ticket 30's families (item 1) and the small hooks ticket (items 3–5);
- whether to fix the shaman's "Wind" lines as data.

Ticket 70 now also carries the domains (item 2). Nothing blocks.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run).

- **Ticket 30's families and the missing hooks** go to [ticket 71](71-ticket-30-families-where-owned.md): placeholder cells recorded as markers, compared in `sparse_archetype_rows`, each archetype decided right after its rows are sparse; plus the hooks still missing (skills, Perfect Mind, the Sword Saint's and half-elf's lists, Animal Focus, Champion of the Faith).
- **The domains** (the Living Deity, Domain Zealot, Boon Companion, Mythical Beast) go with [ticket 70](70-nature-bond-and-proficiencies-where-owned.md).

**Lead, 2026-11-03, on the "Wind" finding:** not a data bug. In `libs/game-data/classes/shaman.ron`, Air Barrier, Vortex Spells and Wind Ward already require `SelectionOption("shaman/spirit/wind")` (a hand fix); only a raw extraction resolves their "Wind" line to the Wind Whisperer's feature. Nothing to fix in the data. Ticket 71 may make the extraction read it as the spirit directly, if the shape allows. Verified independently: a fresh extraction is byte-identical to HEAD-before-ticket-68's.

**User, 2026-11-03, on the "Wind" line:** leave it. The raw extraction keeps reading the line as the Wind Whisperer's feature, and `Pick::SelectionKeepingFeature` stays; the game data's hand fix is the answer.
