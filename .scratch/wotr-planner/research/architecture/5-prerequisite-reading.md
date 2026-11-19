# Candidate 5: one normalised reading of a Prerequisite — findings

(Explorer report, read-only, main at 790eff76, 2026-11-18.)

## 1. Current state

**Types.** `Prerequisite` (`libs/rules/src/prerequisite.rs:23-148`): four groups (`All`, `Any`, `AtLeast(u8, ..)`, `Not(Box)`), 34 leaf kinds. Compiled at check time from `Condition` (`component.rs:1167` `compile_requires`, `:1240` `Condition::compile`), the source of non-normal shapes: `ArchetypeLevel` → `All([Archetype, ClassLevel])` inside the outer `All` (`:1275-1286`; 58 `.ron` files); an alignment or armour list → an `Any` that can sit inside the entry's `Any` (`:1251`, `:1333-1344`); `AtLeast((*n).max(1), ..)` keeps `AtLeast(1, [x])` (`:1264`); `Not(All([]))` = "never" (`:1309`); `HasNot` and `Condition::Not` can stack `Not` on `Not`. `canonicalize` (`prerequisite.rs:268`) only sorts; runs in `compile_prerequisites` (`component.rs:1192`) and again in `Catalog::new` (`engine/catalog.rs:85`, `data.canonical()`). `Unmet` (`choice.rs:139-165`): `Condition{needs, has}`, `All`, `Any`, `AtLeast{needed, met, failed}`, `MetLater`; no `Not` variant — a failing `Not` becomes `Condition{needs: Not(..)}` cloning the whole subtree (`state.rs:704`, `:786`).

**Walkers (21).**
- Evaluation: 1. `CharacterState::check_in` (`state.rs:653-791`): `All`, `AtLeast` evaluate every child (to list reasons); `Any` stops at the first that holds; `Not` a flag; nothing flattened. Callers `check`, `check_for`, `check_feat_for`, `check_option` (`:596-651`), the walk's `judge.rs:274`, `slots.rs:90`.
- Generic leaf walks: 2. `Prerequisite::leaves` (`prerequisite.rs:174`): every group and every `Not`, polarity ignored; used by `stories`, `collect_feats` (`cursor.rs:432`), `names_alignment`, `components.rs:204` (`reads_spellbooks`), `cursor.rs:763` (`spellcasting_could_meet`), `slots.rs:203`. 3. `is_alignment`, `states_alignment`, `without_alignment` (`prerequisite.rs:214-259`): `without_alignment` looks only at a top-level `All`; used by `catalog.rs:85-93`, `:597` `entry_prerequisites`. 4. `Checker::prerequisite`, `condition_names` (`check.rs:1164-1290`): the story-among-alternatives check is top-level only (`:1175`).
- Engine: 5. `Catalog::needs_unheld` (`slots.rs:51-69`): own All/Any/AtLeast semantics, `Not` as false. 6. `Engine::unmet_negates` (`ranks.rs:66-86`): walks `Unmet` then the `Prerequisite` inside; special-cases `Not(Not)`; counts `NoOtherOptionOfGroup`/`NoOptionFrom` as negations. 7. `unmet_leaves`, `prerequisite_leaves` (`ranks.rs:106-128`): drop every `Not`, including `Not(Not(x))` (a positive need). 8. `Repeat::derive` (`parameter.rs:274-307`): only the top-level `All`'s `Not` children, matching `own.leaf()` (`component.rs:329`), a `Rank` of itself, or an `Any` containing it.
- Wording (`engine/labels.rs`), two parallel renderers: 9. over `Prerequisite`: `prerequisite_text_within` (`:1048`); `all_texts` (`:1030`, flattens a nested `All`, refusals last, recognises `Not(Not)` again at `:1042`); `group_texts` (`:1005`, brackets an inner `Any`/`AtLeast`, `told_apart`); `held_text` (`:821`, special `Any`-of-held at `:862-890`); `alternatives_noun` (`:1245`). 10. over `Unmet`: `unmet_reason` (`:678`, refusals first); `one_case_reason` (`:777`); `unmet_text` (`:927`); `unmet_all_texts` (`:966`); `nested_unmet_text` (`:977`); `negated` (`:87`, a third `Not(Not)` check); `Unmet::and_parts` (`choice.rs:170`). 11. `unmet_for`, `within_reach` (`labels.rs:316-391`): collapse an `Any` to its reachable child, which can put an `All` inside an `All` — one reason `and_parts` exists.
- MCP: 12. `leaves` (`apps/mcp/src/validate/tree.rs:260`, over `Unmet`): returns the `Not` node itself as a leaf, so `Given::is_named_by` (`:171`) never matches it. 13. `needs` (`tree.rs:271`): `and_parts` → `issue_reason` → strip `"Needs "`. 14. `options.rs:179` calls `unmet_negates`; `search.rs:78,86` call `short_prerequisite_text`/`prerequisite_text`.
- Game-check: 15. `Named::walk` (`tools/game-check/src/write/prerequisites.rs:473`): tracks polarity correctly (accepted and refused facts), via `leaf_at` (`:505`, leaf → `DataAt`). 16. `carrier_guard::walk` (`write/carrier_guard.rs:141`): required facts with `Any` alternatives; `AtLeast(0|1)` as `Any`; skips `Not`. 17. `areas/prerequisites.rs`: own `Node`/`Leaf` over GUIDs (`:34`, `:86`), `planner_side` (`:674`), `leaves` (`:287`), `canon`/`group` (`:301-352`) — **already the proposed normal form**: same-kind groups flattened, `Not(Not)` cancelled, `AtLeast≤1` as `Any`, a group of one replaced by its child, empty groups dropped, children sorted and deduplicated, alignments folded into one mask. Private to game-check, keyed by GUIDs.

## 2. The concept and disagreements

Every walker re-derives: same-kind flattening, `AtLeast(1)` as `Any`, double-negation cancelling, leaf polarity, leaf → fact, "one text per distinct leaf".

- `All` in `All` flattened by `all_texts`, `and_parts`, `canon`, `Named`; not by `Repeat::derive`, `without_alignment`, `one_case_reason`, the story check.
- `Any` in `Any` flattened only by game-check's `canon`; 329's fixed text still reads "the Fey Tricks selection or (the Rogue's Rogue Talents selection or the Rogue Talent selection) or …" (329 Answer line 31).
- `AtLeast(1)` as `Any`: `canon`, `carrier_guard` yes; labels no ("Needs 1 more of: Kinetic Healer", pinned in 308f line 139). "N more of" (`labels.rs:955`) ignores `met == 0` (330 item 3). Prerequisite text says "2 of:" (`:1083`), unmet text "N more of:" for the same group.
- `Not(Not(x))`: cancelled by labels (three guards), `unmet_negates`, `Named`, `canon`; treated as "not a need" by `ranks.rs:125`; walked through ignoring polarity by `leaves()`.
- Polarity: `leaves()` ignores; `Named` tracks; `ranks.rs` and `carrier_guard` drop every `Not` subtree; MCP keeps the `Not` node.
- Leaf → fact: four tables: `check.rs:1253` `condition_names`; game-check `leaf_at` (omits `ClassSpellLevel`, `NoOptionFrom`, `PickedThrough`, `SkillRanks`); `Fact::leaf` (inverse); MCP `is_named_by` / `ranks.rs:132` `names_pick`.
- `told_apart`: applied to `Any` in both renderers; `Unmet::AtLeast` text skips it (`:950`), `Prerequisite::AtLeast` text uses it.
- Refusal order: reasons put refusals first (ticket 65); prerequisite text last (329). Both deliberate; two orders for one concept.
- Empty groups: evaluation `All([])` true, `Any([])` false; `canon` turns `Not(All([]))` ("never") into nothing; `check.rs:1243` flags empty groups.

## 3. Tests today

- `leaves`: `libs/rules/tests/engine/prerequisites.rs`, 4 tests.
- Labels: `libs/rules/tests/engine/labels.rs`, ~26 tests (`:422`, `:543` negation; `:659` groups; `:322` told apart; `:362` short text; `:845` and-inside-and; `:890` `unmet_negates`). Many build `Unmet` by hand (`needs` helper `:13`), testing past the evaluator with shapes `check_in` may never produce.
- MCP: real-data exact texts in `apps/mcp/tests/server/wording.rs` (4, ticket 329); tree placement only via `validate.rs`'s `roots`; `tree.rs` unit tests only `echoes`.
- Evaluation: mostly indirect (pickers, agreement sweeps, seeds `negated_prerequisites.rs`, `repeats_from_the_header.rs`, `prerequisites_as_components.rs`), a few direct `state.check` (`held_selections.rs:46-52`, `seeds/.../follow_ups.rs:435`).
- `Repeat::derive`: `libs/rules/tests/engine/repeats.rs`, 7 tests.
- Game-check: `Named` and narrowings fixture `prerequisites_write.rs:838-931` (incl. "a drop under two nots", "a drop from an any inside an all"); `canon` `areas.rs:41-152`; `carrier_guard::walk` only through the command.
- Untested: `needs_unheld` alone; `ranks.rs`'s `Not(Not)`; `AtLeast(1)` vs `Any` agreement across walkers; `Any`-in-`Any` text; `Repeat::derive` on a nested `All`. The form proof records statuses (`tools/corpus/tests/form_proof.rs:1-40`), never reason texts.

## 4. Damage from the scatter

- 329: the doubled level clause fixed in `and_parts`, `all_texts`, MCP `needs` (Answer line 34); left `Any` in `Any` bracketed.
- 308d: "Not with (A or A)" fixed in `held_text` only (308d line 52).
- 192 item 6: "(no Bloodline yet, no Bloodline yet)" in three texts (`search`, `describe`, the reason).
- 330 item 3: "N more of" when none held; 308f's "1 more of: Kinetic Healer".
- Ticket 77 (2026-11-04) merged the plain leaf walks into `leaves()` and deliberately left the structural walkers alone — those later diverged.

## 5. Constraints

- wasm: `rules` has no IO; a normaliser in `prerequisite.rs` fits.
- Hot path: `check_in` runs per option in `options` and the sweeps, already cloning a `Prerequisite` per failing leaf; normalising per call adds allocation, at load costs nothing per call. Load already canonicalises twice.
- Serialised form: `.ron` holds `Requires` components plus hand trees for planner entries and the 296 leftovers (`README.md:899-905`). Normalising in `compile_prerequisites` changes the compiled tree (seen by the browser's JSON, MCP, game-check's before/after equality `write/prerequisites.rs:395`), not the `.ron` files; in `Catalog::new` only the engine. Statuses shouldn't move (every rewrite preserves evaluation); `FORM_PROOF_OFFERS=1` would show it; reason texts are not proven by it.
- No GUID tables in `rules`; game-check's `canon` works over GUIDs.

## 6. Overlap

- (1): `narrowings_of` with `Named::walk` / `leaf_at` (`write/prerequisites.rs:549`, `:473`, `:505`), `carrier_guard::walk` — both need a polarity-aware leaf set; `before == after` (`:396`) becomes structural.
- (2): the `Unmet` side — `unmet_reason`, `unmet_text`, `held_text`, `negated`, `one_case_reason`, `and_parts`, `unmet_for` / `within_reach`, `Engine::unmet_negates`, MCP `needs` / `leaves`. The largest shared surface: a refusal's wording is a reading of a normalised `Unmet`.
- (3): indirect; `carriers-known.txt` is regenerated from `carrier_guard::walk`.
- (4): leaf → fact (`leaf_at`, `check.rs` `condition_names`, `Fact::leaf`, `Given::is_named_by`, `names_pick`); `within_reach` / `feature_holders`; `carrier_guard`'s alternatives.
- (6): exact reasons ("Needs 1 more of: Kinetic Healer", `negated_prerequisites` kinds) pinned there move with any wording change.

## 7. Shapes (uncommitted)

- **A. A normal-form invariant on the existing enum:** `Prerequisite::normalize()` (or a stronger `canonicalize`): flatten `All` in `All` and `Any` in `Any`, `AtLeast(1)` as `Any`, cancel `Not(Not)`, a group of one → its child; once at load (`compile_prerequisites` or `Catalog::new`); `GameData::check` could refuse a non-normal hand tree. One invariant: highest leverage; `check_in` keeps its input's shape, so `Unmet` comes out normal too. Deletion test: flattening returns in `and_parts`, `all_texts`, three `Not(Not)` guards, `carrier_guard`'s `AtLeast(0|1)`, `ranks.rs`. Cost: test world and `labels.rs` tests build trees by hand (normalise them or use a constructor); `unmet_for`'s collapse can still nest groups unless it re-normalises. Exact-text tests through `Engine` and MCP survive.
- **B. A read-only `Reading<'p>` view:** `And`, `Or`, `AtLeast(n, ..)`, `Leaf { leaf: &Prerequisite, refused: bool, fact: Option<Fact> }` (polarity pushed to leaves, fact resolved); cached per entry at Catalog load, on demand in game-check; evaluation stays on the raw tree. `Named`, `carrier_guard`, `ranks.rs`, MCP `leaves`, `Repeat::derive`, `needs_unheld` read it. Deeper; hides `leaf_at` and the fact tables; adds a type beside `Unmet`; De Morgan for `Not(Any)`/`Not(All)` must keep "not with A or B" wording. Owner wording (`selection_owner`, `feature_name_apart`) needs the Catalog, stays in labels.
- **C. A normalised `Unmet` from evaluation plus one reading:** `Unmet::parts() -> (refusals, needs)`; labels' `unmet_*` and MCP `needs` / `leaves` read it; merges with candidate 2. Fixes the 329/330 class at the wording seam without changing the compiled tree; nothing for game-check or `Repeat`.

A and B combine: A as invariant, B as the polarity-aware view; game-check's `canon` then gets an already-normal tree and keeps only GUID and alignment-mask folding.
