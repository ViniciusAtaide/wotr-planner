# Build classes with their picks from the start

Type: task
Status: resolved
Blocked by: 67
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-03** (in a forked session): "why didn't we started kinetiscits/wild-talents selections from the start in classes? the label servers no purpose on that go", then "yeah lets do this. we should patch the extraction data directly in code and not in runtime".

Today the generic class-table reader from [ticket 10](10-extract-guide.md) turns every level-table cell into a feature. Patch passes appended to `libs/extract/src/build.rs` then find the label, remove it and insert the selection: `wild_talents`, `ki_powers`, `infusions`, `mercies_and_exploits`, `exploits`, `borrowed_selections`, `bonus_feat_choices`, and those of tickets 66 and 67 once they land.

The class-building stage should know which cells are picks and make the slot directly. Each of those modules stops being a patch and becomes a provider that the class stage asks, for a class or archetype and a cell, whether it's a pick, of which selection, and with which options. No label is created and then deleted.

Apply the same idea, where it fits, to the other passes that patch finished data: the extra picks (`extra_picks`, `mythic_extra_picks`, `mythic_ability_picks`, `borrowed_extra_picks`), `archetype_alignments`, `option_levels`, `option_families`, the follow-ups (`follow_ups`, `option_follow_ups`, `second_follow_ups`, `talent_follow_ups`), `class_rules`, `report_gaps` and the rest. Each fact is built where its owner is built.

The few passes whose order is real stay as a short ordered prelude, with a comment giving the reason for each. Examples: resolving prerequisites, dropping dangling grants (`references::drop_dangling`), and sparsifying archetype rows before naming lists (`progressions::sparse_archetype_rows` before `feat_lists::name_lists`). The ticket-history comments in `build.rs` go.

## Build

AFK; the only agent in the repo. A refactor of `libs/extract` only: no change to `rules`, the schema, `libs/game-data/` or the link format. Keep each module's assertions of the guide's sentences (they're the extractor's checks), moved to where the fact is now built. Keep the `settled` report filters working, or replace them with the same effect. Use the RustRover IDE tools for moves and renames where they fit.

**Done when:**
- `libs/game-data/` is byte-identical: `git diff HEAD -- libs/game-data` is empty.
- A fresh extraction into the scratchpad, report included, is byte-identical to one made with HEAD's extractor before the change. Save both and `diff -r` them.
- `cargo test --workspace -- --list` is unchanged apart from moved names. Save the diff, and explain each moved name in the Answer.
- `build.rs` reads as: the stages that build each owner, then the short ordered prelude or postlude with a reason per step. There are no label-then-replace passes left. List any patch pass you kept, and why, in the Answer.
- The lighter per-ticket checks pass: fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- **2026-11-03, baseline** (before any edit, HEAD `5280ed0`): a fresh extraction with HEAD's extractor into the scratchpad (`t68/before`; a second run diffed identical, so it's deterministic), `cargo test --workspace -- --list` saved (738 tests), a `git archive HEAD` copy.
- **Infrastructure:** new `libs/extract/src/picks.rs`. `Builder::grant` asks `Picks::pick` (table, cell, the feature the cell names, level) before it defines a feature; the class stage adds each owner's provided selections (their options' prerequisites queued as the guide's lines are), features and feat lists; `resolve_prerequisites` asks `Picks::prerequisite_of`; `add_options` adds provided options. `feat_lists::name_lists` extends an owner's lists instead of replacing them. Checked byte-identical.
- **One step per module, the extraction diffed after each** (all identical): 67 `second_curse`; 57 `exploits` and 54's Arcane Enforcer; 54's mercies (Extra Mercy's extra pick joins ticket 49's table, its prerequisite the generic rule's); 58 `ki_powers`; 45 `wild_talents` (with ticket 47's wild-talent parts); 59 `infusions`; 61 `bonus_feat_choices`; 60 `borrowed_selections` (ticket 47's Combat Trick list gains the two sharing selections, which now exist when it runs); 66 `instinctual_rage_powers`; 63 `archetype_alignments`; 55 `mythic_extra_picks` and 62 `mythic_ability_picks`; 28's `class_rules::apply_prestige` into the prestige stage; `build.rs` rewritten as stages then the commented postlude, the four `settled` filters as one.
- **Checks:** fresh extraction, report included, byte-identical to the baseline (`diff -r`); `git diff HEAD -- libs/game-data` empty; test list identical; `cargo fmt --all --check`, clippy native and wasm, `cargo test --workspace` (738 pass), the `rules` wasm build and seed `--check-only` pass. `apps/backend/src/main.rs` untouched.

## Answer

**The class stage builds the picks.** `libs/extract/src/picks.rs` holds what the providers give the stages. Each provider module keeps its ticket's name and now has `provide` (reads its guide sections and asserts their sentences, before any class is built) and `check` (what can only be asserted on the finished data: the rows in force; it changes nothing). While a class or archetype table is read, a cell naming a pick's feature is the slot (`Pick::Selection`), the slots (`Pick::Slots`) or the owner's own feature (`Pick::Feature`, the Traditional Monk's fixed ki powers). No label feature is created at any point: `kineticist/wild-talents`, `paladin/mercy`, `monk/ki-power`, the Enforcer's and the Exploiter's exploit features, the Sharpshooter's infusion, the bard/skald/Feyform/Vivisectionist/Sanctified Slayer/Hexcrafter features, the four bonus-feat choices and the second curse are never defined.

**What moved where:**
- Picks from cells, with their selections, options, features and lists, built in `Builder::base_class`: 45, 54, 57, 58, 59, 60, 61, 67. Ticket 66's five options are added where `barbarian/rage-power` is read (`add_options`).
- Prerequisites of other owners now come from the generic rule, since the selection exists when lines are resolved: Extra Mercy (any mercy), Extra Arcanist Exploit (any exploit), Extra Rage Power (the five too). Extra Wild Talent's and Throw Elixir's are stated where they're resolved (`Picks::prerequisite_of`, asserting the guide's line).
- 63: the Priest of Balance's and the Mantis Zealot's rules are queued where the archetype is built. 55: `MythicFeat::extra_pick` set where the mythic feats are built. 62: `MythicAbility::selections` set where the abilities are built, and Witch Wandering Hex's misspelt "Prequisites: Hex" read as its line, which resolves as Extra Hex's.
- 47's wild-talent parts (Skill Focus `PerValue`, the "Wild talent feat" label) are built with the wild talents. 28's prestige advancement and "counts as" run where the prestige classes are built.

**`build.rs`** reads as: the stages (skills, races, feats, `Picks::read`, classes, prestige classes, mythic abilities, feats, paths), then the postlude, one comment per step giving its reason. The ticket-history comments are gone.

**Passes kept, and why** (recommendation in brackets):
1. `prose::apply`, `place_features`, `resolve_prerequisites`, `drop_dangling`, `drop_redundant_alternatives`, `grant_lone_feats`, `sparse_archetype_rows`, `name_lists`, `add_lists`: the real ordered steps.
2. `shared_options`, `parameters`: they rewrite resolved prerequisites.
3. `proficiencies::apply` drops a proficiency feature its feats cover in full, on the full rows. It's small, but its tests drive it on hand-made data. [A follow-up ticket: read the "… Proficiencies" sections through the cell hook.]
4. `class_rules::nature_bond` is label-then-replace. The level-1 companion feature it takes off is still what Boon Companion's and Mythical Beast's lines resolve to, and a later pass removes that feature. [Follow-up, after item 5.]
5. `option_levels`, `option_families` (tickets 30–31, about 4,400 lines) are the largest label-then-replace passes left: placeholder cells such as "Bloodline Ability" and "Domain Abilities". Each family replaces cells in several classes' rows in force at once, and the archetype selections it makes depend on which rows are sparse. [Their own ticket: build the option families in the class stage.]
6. `Picks::late`: the wild talents' and infusions' prerequisites name the element options that `option_families` builds. The options exist from the class stage; their prerequisites are set right after the families.
7. `follow_ups`, `option_follow_ups`, `second_follow_ups`, `backgrounds`, `option_class_skills`, `report_gaps`, `talent_follow_ups`: the user's decisions on facts the option families build, each asserting what it finds there. [Fold each into its owner once item 5 is done.]
8. `extra_picks` needs every selection, the families' included. `borrowed_selections::apply` widens what tickets 44 and 47 set (the rogue's and the slayer's option levels, the advanced talents' and hexes' prerequisites, Extra Rogue/Slayer Talent, the combat style feat). `borrowed_extra_picks` needs the rows in force and ticket 49's extra picks.

**Done-when:** the extraction diff is empty (`diff -r t68/before t68/after`, report included, after every step and at the end). The game-data diff is empty. The test-list diff is empty, so no test was moved or renamed: every function a test imports kept its name. fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build and seed `--check-only` all pass.

**Decisions made by recommendation (the user is asleep):**
1. The providers stay in their tickets' modules behind one registry (`picks`), rather than in one new module, so each keeps its sentences.
2. Assertions on the finished data moved to read-only `check` functions run at the end, since the rows in force exist only once rows are sparse. The assertions that only checked a label existed (`needing`, `take`) went with the label.
3. A provided selection registers its name in the catalog, as a guide selection does, so a line naming it means holding one of its options (Extra Mercy).
4. The four `settled` report filters became one `settled` at the end: same report, byte for byte.
5. `name_lists` extends an owner's feat lists rather than replacing them, so the class stage can give lists by key (the bonus-feat choices').

**For the user:** whether to open the follow-up tickets in items 3–5 and 7, the option families first. Nothing blocks.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run). The six decisions above stand. The passes kept go to follow-ups: [ticket 69](69-option-families-where-owned.md) (the option families, option levels and the follow-ups that edit what they build) and [ticket 70](70-nature-bond-and-proficiencies-where-owned.md) (`class_rules::nature_bond` and `proficiencies`).

**Lead, 2026-11-03:** verified independently. HEAD's extractor, built from `git archive HEAD` in its own target directory, and the working tree's both extract byte-identical output, report included.
