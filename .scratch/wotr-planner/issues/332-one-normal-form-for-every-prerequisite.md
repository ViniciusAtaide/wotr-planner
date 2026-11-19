# One normal form for every prerequisite, at load

Type: build (engine)
Status: resolved
Blocked by: 331
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-P.md, T1](../research/architecture/design-P.md) (interface §1, what goes §2, tests §4, risks §5), in the frame of [overall.md](../research/architecture/overall.md); decisions 1 and 4 in [decisions.md](../research/architecture/decisions.md).

## Problem

Each reader of a Prerequisite re-derives its shape:
- `and_parts`, `all_texts` and three `Not(Not)` guards in labels, and one in ranks;
- `carrier_guard`'s `AtLeast(0 | 1)`;
- game-check's `canon`.

Fixing one place leaves the others. Ticket 329 fixed "and inside and" in three places and left "or inside or" bracketed. `AtLeast(1, [x])` reads "Needs 1 more of: Kinetic Healer" (308f), and "N more of" shows with none held (330 item 3).

## Change

- `Prerequisite::canonicalize` normalises (design-P N1–N5: no and in an and, no or in an or, `AtLeast(1)` is an or, `Not(Not(x))` is `x`, a group of one is its child, children sorted). `compile_prerequisites` applies it to every entry, hand trees included.
- Add `is_canonical`, and the `Unmet::all`/`any` constructors used by `check_in`, `unmet_for` and `Entry::issue`.
- Delete the flattening and the guards listed in design-P §2.
- Make 330 item 3's change in `unmet_text`.

## Done when

- A real-data test shows every tree in the checked data is canonical.
- The form proof's `compare` (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`, recorded on the base in the lane before the first edit) reports 0 status changes. Every changed reason is one of decision 1's first three items, listed in the Answer by count.
- The plain game check prints the same report, byte for byte, and exits 0.
- The exact texts in design-P §4 pass.

## Files

- `libs/rules/src/{prerequisite.rs, component.rs, choice.rs, state.rs}`
- `libs/rules/src/engine/{labels.rs, ranks.rs, walk/judge.rs}`
- `libs/rules/tests/engine/{prerequisites.rs, labels.rs}`
- `libs/seeds/tests/game_data/{wild_talents.rs, barbarian_witch_shaman_prerequisites.rs, canonical_trees.rs (new), main.rs}`
- `apps/mcp/tests/server/wording.rs`

If the build needs `libs/rules/src/engine/catalog.rs`, `libs/rules/src/check.rs` or `libs/seeds/tests/game_data/support.rs`, say so at once: ticket 334 may be running beside this one on those files.

## Checks

- `cargo test -p rules -p seeds -p corpus -p mcp -p game-check`
- `cargo run -p seeds --bin check-data`
- `cargo build -p rules --target wasm32-unknown-unknown`
- the game check
- the form proof as above
- `cargo fmt --all --check`

## Answer

Built as design-P T1 describes. The normal form lives in `Prerequisite::canonicalize`, and every load path calls it.

### What was built
- **`prerequisite.rs`:** `canonicalize` normalises from the leaves up, through a private by-value `into_canonical` and `joined`:
  - N1: no `All` in an `All`, and no `Any` in an `Any`;
  - N2: `AtLeast(1, ..)` is an `Any`;
  - N3: `Not(Not(x))` is `x`;
  - N4: a group of one is its child;
  - N5: children sorted.

  Kept as they are: empty groups, `AtLeast(n ≥ 2)` (`AtLeast(0)` is untouched too), and duplicate children. New `is_canonical`.
- **`component.rs`:** `compile_prerequisites` canonicalises every entry's tree, compiled or hand-written.
- **`choice.rs`:** new `Unmet::all` (debug-asserts at least one failure) and `Unmet::any`. Each takes in the children of a child of its own kind; one failure alone is itself. `and_parts` is no longer recursive.
- **Callers:** `check_in` (`state.rs`), `unmet_for` (`labels.rs`) and `Entry::issue` (`walk/judge.rs`) build with the constructors.
- **Deleted:**
  - `all_texts`' `push`;
  - the `Not(Not)` guards in `negated`, in `all_texts`' sort and in `unmet_negates` (`ranks.rs`);
  - the "Two negations cancel" arm;
  - `and_parts` in `unmet_all_texts`.
- **`unmet_text` (330 item 3):** `met == 0` reads "{needed} of". The `prerequisite_text` doc says it reads a canonical tree.

### Decisions
- **Main session, from the brief:**
  - the normal form in `canonicalize`, applied by `compile_prerequisites` to hand trees too;
  - wording items 1–3 accepted;
  - both refusal orders kept;
  - `needs_unheld` on the raw tree;
  - game-check's `canon` whole.
- **Main session, on this ticket:**
  - the four reorder edits below are accepted;
  - the `AtLeast(1)` trim counts as item 2;
  - the four seeds expectations may be rewritten, shape only;
  - the GLOSSARY sentence is left to the main session.
- **Mine:** `Unmet::any` doesn't assert non-empty, because `Any([])` evaluates to an empty "or". This departs from design-P §1's error modes.

### Form proof
- **Record:** on the base aedfb6ec, in the lane, before the first edit, with `FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`.
- **Compare on the tip** (`FORM_PROOF_REASONS=1`, with the accept file):
  - `links: 1992 the same, 0 through an heir, 0 refused, 0 changed`;
  - `offers: 0 statuses changed in 0 Builds`;
  - `failures: 0 appeared, 0 gone`;
  - `reasons: 15217937 compared, 214231 changed in 52 edits`;
  - `2157 Builds compared (1992 open), 2157 old links reopened, 0 differences, 0 offer differences, 0 reason problems`.

**Changed reasons, by count:**
- **Item 1, an "or" in an "or" loses its brackets** (children in the canonical sort): 104,109 options in 6 edits.
  - Wild Shape (Wolf)/Major Form: 64,072;
  - Extra Rogue Talent: 16,308 + 23;
  - Warrior Priest's domains and mysteries: 12,051;
  - Second Mystery: 5,947;
  - Impossible Domain: 5,708.
- **Item 3, "N more of" with none held reads "N of":** 54,643 options in 1 edit (`more ` removed).
- **Item 2, `AtLeast(1)` reads as an "or":** 55,411 options in 41 edits. Within it:
  - The plain "1 more of: A, B" becomes "A or B" (Bomb, Healer's Way, the rages, Cloud, Plague Mutation, Ice in the Blood).
  - The lone Kinetic Healer reads "Needs Kinetic Healer": 902.
  - About 1,230 options re-sort a reason's parts (the "or" now sorts as an `Any`).
  - **The trim:** 15,033 options in 7 edits now get `reason_for`'s rule for an "or": the alternatives the character's classes give, else all. Examples:
    - raging-brutality for an alchemist: "Needs Demonic Rage, Power Attack, …" (14,415);
    - extra-bombs for a Metamorph: "Needs Bomb" (262);
    - extra-lay-on-hands for a paladin: "Needs Lay On Hands" (221);
    - the Mutagen ones: 15 + 12;
    - the skald's "(Demonic Rage or Inspired Rage)": 99 + 9.

    "Demonic Rage" is true for any class. The Demon path's rank-3 row grants `demon/demonic-rage` (game: DemonRageFeature 6a8af3f208a0fa747a465b70b7043019, granted by DemonProgression 285fe49f7df8587468f676aa49362213, the path's own progression), and a path's grant is always in reach.
- **An "and" in an "and" is one "and"** (main session's decision): 68 options in 4 edits. The same parts read in the sorted order:
  - `barbarian/rage-power/*-instinctual-warrior` reads "Improved Unarmed Strike, Barbarian level 4 (have N), the Instinctual Warrior archetype": 5 + 8 + 8;
  - `kineticist/infusions/exploding-arrows` reads "(the Elemental Focus (Blood Kineticist) selection or the Elemental Focus selection), the Kinetic Sharpshooter archetype": 47.

### Tests (each in `--list`)
- **`libs/rules/tests/engine/prerequisites.rs`:**
  - `an_and_inside_an_and_is_one_and`, `an_or_inside_an_or_is_one_or`, `at_least_one_is_an_or_and_a_lone_alternative_is_itself`, `a_double_negation_is_its_condition`, `a_group_of_one_is_its_child`, `at_least_two_keeps_its_count`, `empty_groups_and_never_stay_as_they_are`, `normalising_twice_changes_nothing`;
  - `a_normal_tree_is_met_as_its_raw_tree_is`: raw and canonical trees agree in five test-world states;
  - `a_failing_or_inside_an_or_reads_as_one_or`;
  - `a_class_and_its_archetype_failing_read_as_one_and`.

  They test the engine's form, so no game citation applies.
- **`libs/rules/tests/engine/labels.rs`:**
  - `none_held_reads_n_of` (new).
  - `an_and_inside_an_and_reads_its_parts_once_and_its_refusals_last` moved as §4 says: it canonicalises its tree and evaluates its `Unmet` on a fighter 1 holding Dodge. Its refusal line now reads "Not with dodge; needs toughness, Fighter level 4 (have 1), the Eldritch Scoundrel archetype", the needs in the tree's order.
  - The hand `Not(Not)` assertion moved to `a_double_negation_is_its_condition`.
- **`libs/seeds/tests/game_data/canonical_trees.rs` (new):**
  - `every_checked_prerequisite_is_canonical`: 761 trees;
  - `no_tree_nests_a_not_under_a_not`: design-P risk 5's pin.
- **Exact texts:**
  - **Item 2**, `wild_talents.rs`: Healing Burst reads "Needs Kinetic Healer". Game: HealingBUrstFeature c73b37aaa2b82b44686c56db8ce14e7f, `PrerequisiteFeaturesFromList` Amount 1 of KineticHealerFeature 3ef666973adfa8f40af6c0679bd98ba5.
  - **Item 2**, `barbarian_witch_shaman_prerequisites.rs`: the expected tree is now `has_option(kinetic-healer)` beside the level, the same citation.
  - **Item 1**, `apps/mcp/tests/server/wording.rs`: Extra Rogue Talent reads "Needs the Dark Lurker's Rogue Talents selection or the Rogue's Rogue Talents selection or the Fey Tricks selection or the Rogue Talent selection" (ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12).
  - **Item 1**, same file: Impossible Domain reads "the Domain (Living Deity) selection or the Cleric's Domain selection or the Domain (Divine Hunter) selection or the Inquisitor's Domain selection or the Domain (Sacred Huntsmaster) selection" (ExtraDomain 213a8480d22206b45acbfa0619ca5aaf). Both orders are the canonical sort, not a game fact.
  - **Item 3**, new `real_data_n_of_with_none_held_reads_n_of`: Diva Strike for a Battle Scion 5 reads "Needs BAB +7 (have +3), (3 of: Combat Expertise, Diva Style, Feint), Cha 15 (have 10)". Before, it read "3 more of". Game: DivaStrike 3306f4c6913d4d429358b907cdb4ffd1, `PrerequisiteFeaturesFromList` Amount 3 of DivaStyle b46bf228d6b2427abd028e2fe7b84600, Feint c610310d31414edabcedf0c8a6fe32c4 and CombatExpertiseFeature 4c44724ffa8844f4d9bedb5bb27d144a.
- **Shape only** (each expected tree or `Unmet` rewritten into the normal form; the members, the meaning and the citations unchanged):
  - `mythic_class_prerequisites.rs`, `limitless_rage_counts_the_rampage_and_the_devastating_form`: `AtLeast(1, five)` becomes `Any(five)`;
  - `mythic_class_prerequisites.rs`, `the_second_abilities_need_their_selection`: the nested Second Mystery "or" becomes one "or" of four;
  - `option_follow_ups.rs`, `a_living_deity_below_arcanist_20_cant_take_impossible_domain`: the nested `Unmet::Any` becomes one "or" of five;
  - `rogue_slayer_alchemist_prerequisites.rs`, `the_mutagen_discoveries_accept_the_mutation_warrior`: `AtLeast(1, two mutagens)` becomes `Any`.

### Checks
- `cargo test -p rules -p corpus -p mcp -p game-check`: every binary ok (rules engine 768 passed, mcp 251 passed).
- `cargo test -p seeds`: every binary ok, run after the shape-only rewrites.
- `cargo run -p seeds --bin check-data`: exit 0.
- `cargo build -p rules --target wasm32-unknown-unknown`: Finished.
- `cargo fmt --all --check`: exit 0.
- **The game check:** exit 0, its report byte-identical to the base's (`cmp`). Last line: "Carrier hits (ticket 322): 176, each listed in carriers-known.txt".
- **Design-P risk 2:** `--write prerequisites --dry-run` printed output byte-identical to the base's, exit 0.

### Left
- The GLOSSARY "normal form" sentence, which the main session adds.
- The self-checks' `--ignored` long runs, which are the main session's.
