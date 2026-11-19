# Design P: Prerequisite reading (T1, T2)

(Explorer, read-only, main at 790eff76, 2026-11-18. Designed to the architect's recommendations: the normal form applies at load, wording items 1–3 are accepted, item 5 is kept. **[Otherwise: …]** marks each place that would change if the user answers differently.)

Two points in `overall.md` were unverified; I checked both:
- **The browser's JSON carries the compiled tree.** `apps/backend/src/main.rs` serialises `check_game_data_files`'s `CheckedGameData` through `GameDataFile::new` (`apps/backend/src/game_data.rs:31-33`). The browser runs `checked()` on it again (`libs/ui/src/game_data.rs:50-51`).
- **The writer's before/after compile does not go through `compile_prerequisites`.** It calls `compile_requires` and then `tree.canonicalize()` (`tools/game-check/src/write/prerequisites.rs:338-350`). Hand trees are canonicalised by `seeds::files` (`libs/seeds/src/files.rs:173`, `data.canonical()`) before `checked()`.

So the one function every load path already calls is `Prerequisite::canonicalize`. P puts the normal form there. `compile_prerequisites` also applies it to every entry's tree, compiled or hand-written, so that `CheckedGameData` guarantees it whatever the caller did.

## 1. Interface

```rust
// libs/rules/src/prerequisite.rs: T1
impl Prerequisite {
    /// The normal form, then sorted (the existing name, so every load path and test helper gets it):
    /// N1 no `All` directly in an `All`, no `Any` directly in an `Any`;
    /// N2 no `AtLeast(1, ..)`: it is an `Any`;
    /// N3 no `Not(Not(x))`: it is `x`;
    /// N4 no `All`/`Any` of one child: it is the child;
    /// N5 every group's children sorted (as today).
    /// Kept as they are: empty groups (`All([])` holds, `Any([])` fails, `Not(All([]))` never holds),
    /// `AtLeast(n >= 2, ..)` (not rewritten to `All` when n == len), duplicate children.
    pub fn canonicalize(&mut self);
    /// Whether `canonicalize` would change nothing.
    pub fn is_canonical(&self) -> bool;
}

// libs/rules/src/choice.rs: T1
impl Unmet {
    /// An "and" of failures: an `All` child's children taken in, one failure alone is itself.
    /// Panics in debug on no failure.
    pub(crate) fn all(failed: Vec<Unmet>) -> Unmet;
    /// An "or" of failures: an `Any` child's children taken in, one alone is itself.
    pub(crate) fn any(failed: Vec<Unmet>) -> Unmet;
    /// The parts of an "and": its children, or itself. No longer recursive.
    pub fn and_parts(&self) -> Vec<&Unmet>;
}

// libs/rules/src/prerequisite.rs: T2
/// One condition of a tree, with whether the tree needs it or refuses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leaf<'p> {
    /// Never a group or a `Not`.
    pub condition: &'p Prerequisite,
    /// Under an odd number of `Not`s.
    pub refused: bool,
}
impl Leaf<'_> {
    /// The Fact whose holding it reads (see `Prerequisite::fact`).
    pub fn fact(&self) -> Option<Fact>;
    /// The number it needs: a class level, a count, a rank, ranks, a score, BAB, a spell, caster,
    /// character level or Mythic rank.
    pub fn threshold(&self) -> Option<u8>;
    /// The value it names: `FeatFor`'s or `FeatIfValue`'s.
    pub fn value(&self) -> Option<&Parameter>;
}

/// A tree with each `Not` pushed onto its leaves (De Morgan: `Not(Any)` is an `All` of refused
/// leaves, `Not(All)` an `Any`, `Not(AtLeast(n, k children))` an `AtLeast(k - n + 1, ..)`). It holds
/// N1–N4 itself: a pushed group merges into a parent of its kind. The stored tree keeps its `Not`s,
/// so "not with A or B" keeps its wording.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading<'p> {
    All(Vec<Reading<'p>>),
    Any(Vec<Reading<'p>>),
    AtLeast(u8, Vec<Reading<'p>>),
    Leaf(Leaf<'p>),
}
impl<'p> Reading<'p> {
    /// An `All`'s children, or itself alone.
    pub fn parts(&self) -> &[Reading<'p>];
    /// Left to right.
    pub fn leaves(&self) -> impl Iterator<Item = &Leaf<'p>>;
}

impl Prerequisite {
    /// Allocates the tree: for load-time and tool callers, not the per-option path.
    pub fn reading(&self) -> Reading<'_>;
    /// Every leaf with its polarity, left to right, without building a tree (a stack, as `leaves` is).
    pub fn polar_leaves(&self) -> impl Iterator<Item = Leaf<'_>>;
    /// Kept and now one line, `polar_leaves().map(|leaf| leaf.condition)`: polarity-blind.
    pub fn leaves(&self) -> impl Iterator<Item = &Prerequisite>;
    /// The Fact a leaf reads the holding of:
    /// - `Feat`, `FeatFor`, `FeatForSameValue`, `FeatIfValue` → Feat
    /// - `MythicFeat`, `MythicAbility`
    /// - `ClassLevel` → Class
    /// - `Archetype`
    /// - `Feature` → Feature
    /// - `Rank` → its fact
    /// - `SelectionOption` → Option
    /// - `Selection`
    /// - `Race`, `Deity`
    ///
    /// `None` for a group, a `Not`, and every other leaf (`NoOptionFrom` names a selection but reads
    /// that none of its options is held, not the selection). Today's `leaf_at`, exactly.
    pub fn fact(&self) -> Option<Fact>;
}
impl Unmet {
    /// The leaves it fails on, with their polarity: a `Condition`'s own `needs` read by `polar_leaves`.
    pub fn leaves(&self) -> impl Iterator<Item = Leaf<'_>>;
}

// libs/rules/src/component.rs: T2, the Fact↔DataAt pair
impl From<&Fact> for DataAt { /* check.rs:2594 fact_at, moved */ }
impl TryFrom<&DataAt> for Fact { /* game-check write/carriers.rs:708 fact_at, moved */ }
// Fact::leaf (component.rs:329) stays: the inverse of `Prerequisite::fact`.
```

**Invariants and enforcement.**
- `canonicalize` normalises bottom-up, then sorts. It is total and idempotent.
- `compile_prerequisites` (`component.rs:1192`) canonicalises every entry's `prerequisite`, not only compiled ones. Every `CheckedGameData` tree is therefore canonical, and so is every tree `Engine` and game-check hold.
- `check_in` on a canonical tree yields an `Unmet` with no `All` directly in an `All` and no `Any` directly in an `Any`. It builds groups with `Unmet::all`/`any`, because an `All` with a single failing `Any` collapses into a parent `Any` (`state.rs:664-667`).
- Every other place that assembles an `Unmet` also uses the constructors: `unmet_for` (`labels.rs:316-356`) and `Entry::issue` (`walk/judge.rs:301`, which today nests the class's `All` and the archetype's `All` inside another `All`).
- The label functions read a canonical tree. A hand tree passed to them reads with its nesting shown: their doc says so, and tests canonicalise first.
- `GameData::check` does not refuse a non-normal hand tree (`overall.md` §6).

**Error modes.** None at runtime. `Unmet::all`/`any` `debug_assert!` that they get at least one failure.

**Cost.**
- `canonicalize`: O(n log n) per tree, at load. It already runs two or three times per load (files, compile, `Catalog::new:84`). Zero per call.
- `check_in`: one extra `match` per failing `Any` child.
- `polar_leaves`: one small stack `Vec`, as `leaves` does today.
- `reading()`: allocates. Callers are `Repeat::derive` (once per selection at load), game-check, and MCP `validate` (once per failure).
- `fact()`: clones one `String` key.
- The browser runs `checked()` again, so it pays one more normalising pass, which finds nothing to change.

**[Otherwise]**
- **Item 1 refused:** drop "no `Any` in an `Any`" from N1 and from `Unmet::any`. `group_texts` keeps its brackets. Game-check's `canon` already flattens `Any` on its own side.
- **Item 2 refused:** drop N2. `carrier_guard::walk` keeps its `AtLeast(0 | 1)` arm, and `Reading` maps `AtLeast(1)` to `Any` itself.
- **Normal form wanted only in the engine:** move the call from `canonicalize` to `Catalog::new`. MCP `search`/`describe` and game-check then see raw trees, so labels keeps its flattening code and T1 deletes little.

## 2. Behind the seam

### T1: deleted or shrunk

| Function | Now | After |
|---|---|---|
| `Prerequisite::canonicalize` | `prerequisite.rs:268-281` | grows: normalise, then sort |
| `compile_prerequisites` closure | `component.rs:1196-1204` | canonicalises every entry's tree |
| `check_in` `All`/`Any` | `state.rs:659-679` | `Unmet::all(failed)` / `Unmet::any(failed)` |
| `Unmet::and_parts` | `choice.rs:170-175` | not recursive: children or itself |
| `all_texts`' `push` | `labels.rs:1031-1040` | deleted: `parts` = `children` |
| `Not(Not)` guard in `negated` | `labels.rs:92` | deleted |
| `Not(Not)` guard in `all_texts`' sort | `labels.rs:1042` | deleted |
| "Two negations cancel" arm | `labels.rs:1088-1089` | deleted |
| `Not(Not)` guard in `unmet_negates` | `ranks.rs:69` | deleted |
| `unmet_for` | `labels.rs:319-353` | builds with `Unmet::all`/`any` |
| `Entry::issue` | `walk/judge.rs:301` | `Unmet::all(unmet)` |
| `unmet_text` `AtLeast` | `labels.rs:945-959` | item 3: `met == 0` reads "{needed} of: …" |

- Items 1 and 2 need no labels code of their own. With no `Any` in an `Any`, `group_texts`' `Any(_)` arm (`labels.rs:1013`) and `nested_unmet_text` (`:985`) only bracket an "or" inside an "and" or an "n of". With no `AtLeast(1)`, "1 more of: X" cannot arise.
- MCP `tree.rs::needs` (`:271-287`) keeps calling `and_parts` (R absorbs it in T5).
- Item 5 is kept: `all_texts` sorts refusals last and `unmet_reason` puts them first.

### T2: callers move to the view

| Caller | Now | Calls instead |
|---|---|---|
| `Prerequisite::leaves` | `prerequisite.rs:174-189` | `polar_leaves().map(..)`: one stack walk in the crate |
| `unmet_leaves`, `prerequisite_leaves` | `ranks.rs:106-128` | deleted. Callers use `unmet.leaves().filter(|l| !l.refused).map(|l| l.condition)`, then `names_pick` as before |
| MCP `leaves` | `apps/mcp/src/validate/tree.rs:260-267` | deleted. `Node::needs` (`:145`) reads `unmet.leaves().filter(|l| !l.refused)` into `is_named_by(data, leaf.condition)` |
| `Repeat::derive` | `parameter.rs:289-306` | `prerequisite.reading().parts()`: a refused leaf equal to `own.leaf()` → `Once`; a refused `Rank { fact == own }` → `Times`. The `Not(Any(members))` arm goes, since De Morgan puts each member among the parts |
| `Named::walk` | `write/prerequisites.rs:464-496` | `Named::of` folds `tree.polar_leaves()`: a leaf with `fact()` goes to accepted or refused by `refused`; a refused one without a fact is keyed by its `Debug`, as today |
| `leaf_at` | `write/prerequisites.rs:505-534` | deleted → `leaf.fact().map(|f| DataAt::from(&f))` |
| `carrier_guard::walk` | `write/carrier_guard.rs:141-166` | matches `Reading`: `Any` widens with its needed leaves' facts, `All`/`AtLeast` descend, refused leaves are skipped. The `AtLeast(0 | 1)` arm and the `Not(_)` arm go. `use super::prerequisites::leaf_at` (`:35`) goes |
| `condition_names` | `check.rs:1253-1290` | shrinks: the ten Fact-bearing arms become `condition.fact()` then `names(at, (&fact).into())`. `ClassSpellLevel`, `SkillRanks`, `NoOptionFrom`/`PickedThrough`, `DeityOffers` and the values keep their arms |
| check.rs `fact_at` | `check.rs:2594-2607` | moved to `From<&Fact> for DataAt` |
| game-check `fact_at` | `write/carriers.rs:708-722` | `Fact::try_from(at).ok()` |

**What T2 leaves alone, and why.**
- **`needs_unheld`** (`slots.rs:51-69`). It runs per option in a filtered pool, where `reading()` would allocate on every call, and it is already correct: it reads a `Not` subtree as never needing an option, which matches the view on every shape the data holds. The deletion test gains nothing.
- **`is_alignment`, `states_alignment`, `without_alignment`** (`prerequisite.rs:214-259`). They need the empty-group case and the stored `Not`.
- **Game-check's `canon`/`group`** (`areas/prerequisites.rs:301-352`). This departs from `overall.md` §1. `canon` normalises `Node` trees after the GUID mapping:
  - The game side is built raw from the dump.
  - `planner_side` (`:674`) turns a whole-selection `Any` into one leaf and maps distinct keys to one GUID, which makes new nests and duplicates.
  - The alignment mask needs flattened groups.

  Deletion test: remove `canon` and its flattening comes back at both sides. It earns its keep. Its input from the planner side is now canonical, which costs nothing.

## 3. Tickets

### T1: one normal form for every prerequisite, at load

- **Problem.**
  - Each reader re-derives the tree's shape:
    - `and_parts`, `all_texts` and three `Not(Not)` guards in labels, and one in ranks;
    - `carrier_guard`'s `AtLeast(0 | 1)`;
    - game-check's `canon`.
  - Fixing one place leaves the others. 329 fixed "and inside and" in three places and left "or inside or" bracketed.
  - `AtLeast(1, [x])` reads "Needs 1 more of: Kinetic Healer" (308f).
  - "N more of" with none held (330.3).
- **Change.**
  - `canonicalize` normalises (N1–N5), and `compile_prerequisites` applies it to every entry.
  - Add `is_canonical`, and the `Unmet::all`/`any` constructors used by `check_in`, `unmet_for` and `Entry::issue`.
  - Delete the flattening and the guards (§2).
  - Item 3 in `unmet_text`.
- **Done when:**
  - Every tree in the checked data is canonical (a real-data test).
  - The form proof's `compare`, with `FORM_PROOF_OFFERS=1` and T0's `FORM_PROOF_REASONS=1`, reports 0 status changes, and every changed reason is one of items 1–3, listed in the Answer by count.
  - The plain game check prints the same report, byte for byte, with exit 0.
  - The exact texts in §4 pass.
- **Files:**
  - `libs/rules/src/prerequisite.rs`, `component.rs`, `choice.rs`, `state.rs`;
  - `libs/rules/src/engine/labels.rs`, `engine/ranks.rs`, `engine/walk/judge.rs`;
  - `libs/rules/tests/engine/prerequisites.rs`, `labels.rs`;
  - `libs/seeds/tests/game_data/wild_talents.rs`, `barbarian_witch_shaman_prerequisites.rs`, and a new `canonical_trees.rs` with its `mod` line;
  - `apps/mcp/tests/server/wording.rs`.
- **Checks:**
  - `cargo test -p rules -p seeds -p corpus -p mcp -p game-check`. game-check's code is not touched, but its writer calls `canonicalize`.
  - `cargo run -p seeds --bin check-data`.
  - `cargo build -p rules --target wasm32-unknown-unknown`.
  - The game check run.
  - The form proof: record on the base in the lane before the first edit, compare on the tip.
  - `cargo fmt --all --check`.
- **Glossary:** a "normal form" sentence for **Prerequisite**, through `domain-modeling`, once the other session's `GLOSSARY.md` edits are committed.
- **[Otherwise]** Without items 1–2, N1 and N2 shrink (§1). Without item 3, drop the `labels.rs:945` change.
- Model: Opus to build.

### T2: one polarity-aware view, with each leaf's Fact

- **Problem.**
  - Five walkers each decide polarity and the leaf-to-entry mapping on their own:
    - `Named::walk` toggles polarity;
    - `ranks.rs` and `carrier_guard` drop a `Not` subtree;
    - MCP returns the `Not` node itself;
    - `Repeat::derive` matches `Not(Any)`.
  - There are four leaf↔entry tables: `leaf_at`, `condition_names`, and the two `fact_at`s.
- **Change.** Add `Leaf`, `Reading`, `polar_leaves`, `reading`, `fact`, `Unmet::leaves`, and the `From`/`TryFrom` pair. Move the callers in §2.
- **Done when:**
  - `leaf_at`, both `fact_at`s, `unmet_leaves`/`prerequisite_leaves` and MCP `leaves` are gone, and no function outside `prerequisite.rs` matches `Prerequisite::Not` to decide polarity (`grep`).
  - The form proof (`FORM_PROOF_OFFERS=1`, `FORM_PROOF_REASONS=1`) shows 0 differences.
  - The game check's report is byte-identical.
  - `cargo test -p game-check`'s writer fixtures pass unchanged.
- **Files:**
  - `libs/rules/src/prerequisite.rs`, `choice.rs`, `component.rs`, `check.rs`, `parameter.rs`;
  - `libs/rules/src/engine/ranks.rs`;
  - `apps/mcp/src/validate/tree.rs`;
  - `tools/game-check/src/write/prerequisites.rs`, `carrier_guard.rs`, `carriers.rs`;
  - `libs/rules/tests/engine/prerequisites.rs`.
- **Checks:**
  - `cargo test -p rules -p seeds -p corpus -p mcp -p game-check`.
  - The rules wasm build.
  - `cargo run -p seeds --bin check-data`.
  - The game check run.
  - The form proof: form-only.
  - fmt.
- **Order:** after T1 (they share `prerequisite.rs`). It unblocks N (T6), which reads `Reading` and `Leaf::threshold`/`value` for "raises value" and "Any → All", and G (T4), which reads `Leaf::fact` for "satisfies".
- Model: Sonnet to build (the interface is fixed here and the proof is mechanical), Opus to review.

## 4. Tests

### New, at the interface

`libs/rules/tests/engine/prerequisites.rs` (T1):
- `an_and_inside_an_and_is_one_and`;
- `an_or_inside_an_or_is_one_or`;
- `at_least_one_is_an_or_and_a_lone_alternative_is_itself`;
- `a_double_negation_is_its_condition`, including `Not(Any([Not(a), b]))`, which stays as is;
- `a_group_of_one_is_its_child`;
- `at_least_two_keeps_its_count`;
- `empty_groups_and_never_stay_as_they_are`;
- `normalising_twice_changes_nothing` (`is_canonical`);
- `a_normal_tree_is_met_as_its_raw_tree_is`. It evaluates hand trees raw and canonical against a few test-world states through `CharacterState::check`, and asserts the same `is_ok()`. This pins that the form is behaviour-free.

Through the test world's engine (T1):
- `a_failing_or_inside_an_or_reads_as_one_or`. Evaluate `Any([All([X, Any([A, B])]), C])` with `X` held, and get one flat `Unmet::Any` of three.
- `a_class_and_its_archetype_failing_read_as_one_and`. `Entry::issue` through a class option's reason.

`labels.rs` (T1):
- `none_held_reads_n_of`. `Unmet::AtLeast { needed: 2, met: 0, .. }` reads "Needs 2 of: dodge, toughness". The `met: 1` case at `:693` stays "1 more of".

seeds `canonical_trees.rs` (T1):
- `every_checked_prerequisite_is_canonical`.

`prerequisites.rs` (T2):
- `a_leaf_under_one_not_is_refused_and_under_two_is_needed`;
- `the_reading_pushes_a_not_onto_its_leaves` (`Not(Any)` gives an `All` of refused leaves, `Not(All)` an `Any`, `Not(AtLeast(2, 3))` an `AtLeast(2, ..)`);
- `a_reading_of_a_not_inside_an_all_merges_into_it`;
- `each_fact_leaf_names_its_fact_and_fact_leaf_round_trips` (`Fact::leaf(f).fact() == Some(f)` for every kind);
- `a_leaf_says_its_threshold_and_value`;
- `the_unmet_leaves_carry_polarity`;
- `fact_and_data_at_convert_both_ways`.

### Existing tests

- **Survive unchanged:**
  - `prerequisites.rs`'s four `leaves` tests, which pin the T2 reimplementation;
  - `labels.rs` apart from the two below;
  - `repeats.rs`;
  - seeds `repeats_from_the_header`, `negated_prerequisites`;
  - every game-check fixture. "a drop under two nots" yields the same narrowing: the old tree's `x` is accepted either way;
  - MCP `validate.rs` roots;
  - the corpus self-checks.
- **Move:**
  - `labels.rs:845` `an_and_inside_an_and_reads_its_parts_once_and_its_refusals_last`. Its tree half canonicalises the tree first, and the flattening it pinned moves to `an_and_inside_an_and_is_one_and`. Its `Unmet` half comes from evaluation instead of a hand-built nested `Unmet`. The refusals-last assertions stay (item 5).
  - `labels.rs:532-536` ("Two negations cancel" on a hand `Not(Not)`) moves to `a_double_negation_is_its_condition`.
- **Deleted:** none.

seeds expectations hand-built in non-normal shape fail and are rewritten to the normal form, with their citations unchanged. Those built through `canonical()` pass as they are. `barbarian_witch_shaman_prerequisites.rs:281` passes as is, but its expected value is rewritten to `has_option(kinetic-healer)` so the test asserts what its comment says (CODING_STANDARDS:29).

### Exact texts that change

- **Item 1.**
  - `apps/mcp/tests/server/wording.rs:99`: "Needs the Fey Tricks selection or the Rogue's Rogue Talents selection or the Rogue Talent selection or the Dark Lurker's Rogue Talents selection".
  - `:218`: "the Cleric's Domain selection or the Inquisitor's Domain selection or the Domain (Sacred Huntsmaster) selection or the Domain (Living Deity) selection or the Domain (Divine Hunter) selection".
  - The order is the canonical sort of the merged children, not a game fact (329). The builder pins the order the build prints.
- **Item 2.** `libs/seeds/tests/game_data/wild_talents.rs:312`: "Needs 1 more of: Kinetic Healer" becomes "Needs Kinetic Healer". 330.3's "Needs 1 more of: Bomb, Arcane Bombs" becomes "Needs Bomb or Arcane Bombs" (in some order).
- **Item 3.** 330.3's "(2 more of: Diva Style, Feint)" with none held becomes "(2 of: Diva Style, Feint)". The builder finds and pins a real-data case through `options`.

## 5. Risks and unknowns

1. **Unverified point 1 (the browser's JSON carries the compiled tree): settled, yes** (see the header). The JSON's hash, and so its URL, changes once. That is harmless: the browser cache is keyed by that URL.
2. **Unverified point 2 (the writer's compile): settled, it does not use `compile_prerequisites`.** P reaches it through `canonicalize`. If the normal form lived only in `compile_prerequisites`, the writer's `before == after` would compare raw against normal and rewrite unchanged entries. The fixture `an_equal_compiled_tree_is_not_rewritten` guards this; the builder runs `--write --dry-run` before and after and expects the same change list.
3. **Seeds trees compared without `canonicalize`.** How many fail is unknown. The builder runs `cargo test -p seeds` after the first edit and rewrites each to the normal form, citations unchanged. CODING_STANDARDS:31 is not crossed: the tree means the same.
4. **Reasons that move beyond items 1–3:**
   - `one_case_reason` may now match an `All` that `Entry::issue` used to nest;
   - `unmet_for`'s collapse now flattens.

   The reasons record lists each change. A change outside items 1–3 goes to the main session before merge.
5. **The view's De Morgan reads `Not(Any([Not(a), b]))` as needing `a`.** Today `ranks.rs` and `carrier_guard` skip `a`. No `.ron` holds such a shape: no `Not(HasNot|Not|NoClassLevel|NoArchetype|NotProficient)`; the six `Not(All|Any)` files hold no inner `Not`. A seeds test `no_tree_nests_a_not_under_a_not_group` would pin that. The byte-identical game check and the form proof settle the rest.
6. **The check after normalising.** `checked()` runs `check()` before compiling, on files already canonical from `seeds::files`, so its rules see normal hand trees. `StoryNotRequired` and `EmptyPrerequisiteGroup` recurse into children, so flattening cannot hide a case. The browser's second `check()` sees normal trees too. The builder confirms this with check-data.
7. **A label caller with its own tree.** Only tests and `search`/`describe` (data trees) call the label functions. A future caller with a raw tree reads brackets, not a wrong rule.
8. **Departure from the frame.** Game-check's `canon` stays whole (§2), and `needs_unheld` stays on the raw tree. The main session decides whether to accept these, or to measure `reading()` on the pool path before moving `needs_unheld`.
