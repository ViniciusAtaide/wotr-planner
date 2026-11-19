# Candidate 4: one grant index — findings

(Explorer report, read-only, main at 790eff76, 2026-11-18.)

## 1. Current state: who answers "who grants X" / "what does X give"

**A. Engine, reverse, static.** `libs/rules/src/engine/catalog.rs:1061-1142`: `GrantedBy {Class, Archetype, Path, Option}`, built by `holders()` in `FeatureFacts::new` (971-1059) at `Catalog::new`. Covers row `features` and row `selections` only (class, archetype, path-rank, option level rows). Not: feats, `feats_for`, `chosen_feats`, components (`Grant*`), mythic abilities' `features`/`selection`, a feat's or mythic feat's `selection`, path-switch `features`, backgrounds, deities (doc 777-778: "Empty for a feature only a mythic ability, a path switch or a selection gives."). Reads class rows raw (no archetype replacement). Interface `feature_holders` / `selection_holders` (779-798), `pub(super)`.

**B. Engine labels, Build-aware readers of A** (`libs/rules/src/engine/labels.rs`): `held_holder` (222-249) re-walks `catalog.rows(class, archetype)` up to the taken level (archetype replacement applied), then falls back to path or options held; `sole_holder` (251-277), `selection_owner` (279-285); `within_reach` (358-391) for `reason_for`/`unmet_for` (306-356) reads `Prerequisite::Feature` holders against the Build's classes and archetypes; path and option holders count as always in reach, as does a feature with no holder (so a component-only feature is always in reach). Callers: `feature_name_for`, `feature_name_apart`, `prerequisite_text`, every picker reason, MCP `Wording::trimmed` (`apps/mcp/src/validate/wording.rs:141-152`).

**C. Engine, forward, during the walk** (time-dependent, the real application): `hold.rs:203-208`, `303-310` via `Hold::favoured_weapon_feats` (391-401) → `Catalog::favoured_weapon_feats` (catalog.rs:743-765); option rows `walk/options.rs:280-310`; a feat's `selection` slot `Pending::Feat` (`walk.rs:301-308`) → `SlotId::FeatSelection` (`walk/level.rs:524-535`). The granting feat is not recorded on the `Slot` (`engine.rs:428-433`).

**D. game-check, forward per entry** (`tools/game-check/src/write/carrier_guard.rs`): `sites` (183-323) lists what each planner entry gives as sets of `Fact`: class/archetype rows (features, feats, `feats_for`, selections); an option's `feats`, `chosen_feat`, its rows' feats, `chosen_feats`, selections; a mythic ability's features and selection; path ranks; a background's feats; a deity's favoured-weapon feats; every grant component via `carriers::named` (`carriers.rs:541-548`). Omits path-switch features, a feat's or mythic feat's `selection`, `replaced_by`. `Grants::of` / `carry` (39-92): a fact is carried if every site giving the entry gives the fact and no selection offers the entry as a pick. `favoured_weapon_feats` (325-339) is a hand copy of the engine's that drops the weapon `Parameter` (324 lists it). Callers: `prerequisites.rs` `plan` and `audit` (`--carrier-audit`); audit hits feed `carriers-known.txt`.

**E. MCP validate, forward per pick** (`apps/mcp/src/validate/tree.rs`): `Given` (154-229), `pick_gives` (231-257): `Pick::Feat` only the feat (not its `selection`); `Pick::SelectionOption` the option, its `chosen_feat`, its `feats`, its rows' `feats` (not `chosen_feats`, not nested selections); mythic pick only itself. `is_named_by` makes two ad hoc reverse scans: `Given::Class` × `Feature` over the class rows or the rows of *any* of its archetypes, whichever the Build takes (205-217); `Given::SelectionOption` × `Feature` over the option's rows (189-196). Callers: `Wording::gives` (`wording.rs:76-96`), `declare.rs:601-610`. `reached_through` (`wording.rs:98-123`) maps any slot with a level to `Address::Class(level)`, so a `FeatSelection` slot never sits under the Extra feat that grants it.

**F. Further copies:** `libs/rules/src/check.rs:1925-1945` (`CountsTowardUngrantedFeature`), the same scan as E's `Given::Class` arm; `libs/seeds/tests/game_data/support.rs:940-1040` `grants()` (rows → selections → options' rows, transitive), `class_rows`, `all_class_grants`, `all_path_grants` (incl. switch features), used across seeds tests (summaries, class_skills, athletics_iii, gold.rs:1620).

None models companions or pets.

## 2. The concept

"Which entries' sites (a row at a level, a grant component, a held-fact rule) give fact F; conversely, what does entry E give?"

| Variant | Where | Grant kinds counted |
|---|---|---|
| Reverse, rows only | A | features, selections in class/archetype/path/option rows |
| Reverse, Build-filtered, archetype-aware | B | A's, plus effective rows |
| Forward, every kind | D `sites` | rows, feats, components, background, deity |
| Forward per pick | E `pick_gives` | feat or option, partial feats |
| Class-or-archetype existence | E, F `check.rs` | raw rows |
| Transitive through selections | F seeds | rows, recursive |
| Granted by anything at all | 324 (open) | rows, grants or selections; not built |

Disagreements: components (D yes; A, B, E no — after 307's carriers a component-only feature has no holder in A); a feat's `selection` (only C; E's omission is 330.2); mythic abilities (D yes; A, B, E no); path-switch features (only seeds `all_path_grants`); archetype replacement (only B; E's `Given::Class` matches archetypes the Build doesn't take); favoured weapons (C keeps the value, D drops it); `chosen_feats` on option rows (D yes, E no).

## 3. Tests today

- Holder names and `within_reach`: via `Engine::feature_name_for` / `feature_name_apart` / `prerequisite_text` in `libs/seeds/tests/game_data/collapsed_features.rs:58-120`; picker reasons in `skald_rage_powers.rs:389,735`, `ki_powers.rs:213,242`, `instinctual_rage_powers.rs:153`, `wild_talents.rs:280`. Through the engine's public interface; survive.
- MCP `Given` and tree placement: via `validate` in `apps/mcp/tests/server/validate.rs:95-180`, `501-540`.
- Carrier guard: game-check fixtures via writer and audit (`tools/game-check/tests/fixture/prerequisites_write.rs:1088-1110`, `1145-1300`); favoured copy 1301-1304.
- Untested: `pick_gives` for a feat with a `selection` (330.2); `Given::Class` via an archetype the Build lacks; `GrantedBy` directly; D vs C agreement; path-switch and mythic-ability grants in A and D.

## 4. Damage from the scatter

- 330.2 (open): a refused Extra feat's `slot.feat-selection.3.0` is a separate `not_reached` root: `pick_gives` (tree.rs:234) has no `feat.selection`, and `reached_through` (wording.rs:105-106) sends `FeatSelection` to `Class(level)`. The walk knows the granting feat (`Pending::Feat`) but doesn't expose it.
- 308h (`cd16bad8`): game-check rebuilt `sites` and copied `favoured_weapon_feats` (doc cites the engine's, carrier_guard.rs:16): a second source that can drift.
- 324 (open): needs "an entry no planner row, grant or selection gives", a fourth reverse question, plus the value-aware comparison D's copy dropped.
- 244 review (`3ea756b3`): first-holder-in-key-order named a Rogue's Uncanny Dodge after the Barbarian (collapsed_features.rs:66-69).
- 329: `shared_selection_names` (catalog.rs:991-993, 1009-1023) filters to "selections rows name" because an Extra feat's selection isn't in A.
- 189/192 (`d4474a1c`): `within_reach` treats any feature rows don't grant as always in reach.

## 5. Constraints

- `rules` depends on nothing in the workspace, builds for wasm32-unknown-unknown. `seeds`, `mcp`, `game-check`, `corpus` depend on `rules` and `seeds`; `ui` on `rules`. An index in `rules` over `&GameData` reaches every caller.
- game-check has no `Engine`: `Ctx.planner.data` is a `&GameData` (`tools/game-check/src/planner.rs`) from `check_game_data_files` (`main.rs:294`). An index only on `Catalog`/`Engine` wouldn't serve it.
- Loading: `Engine::new(CheckedGameData)` (`engine.rs:79-85`) builds `Catalog`, which runs `holders()` once — in the browser (`libs/ui/src/planner.rs:281`, `timed("engine")`), MCP `Planner::new` (`apps/mcp/src/planner.rs:37-50`), backend (`apps/backend/src/main.rs:24`), corpus.
- Cost: one linear pass over rows and components; wasm startup cost unmeasured.
- The rules crate names no tool: the index returns keys and sites; wording stays in `labels.rs` and MCP.
- Two kinds of fact: time-dependent application (C: option levels, skipped rows, switch drops, `GrantAtClassLevel` thresholds) stays in the walk; only static "can give" moves.

## 6. Overlap

- (1): `carrier_guard` is called from the writer's `plan`; shared `carrier_guard::walk` (required facts with `Any` alternatives) and `prerequisites::leaf_at` (`Prerequisite` → `DataAt`).
- (2): MCP `Node.gives`, `Node.under`, `pick_gives`, `Wording::reached_through`; 330.2 sits at the joint — the refusal module needs "what this pick would give, slots included".
- (3): `carriers-known.txt` is the carrier audit's output; a change in `sites` changes the ledger.
- (5): `within_reach` and `Given::is_named_by` are both "does grant G satisfy leaf L"; shared `Unmet`, MCP `leaves` (tree.rs:259-267), carrier_guard `walk` (141-166), `leaf_at`, `rules::Fact`, `DataAt`. The leaf → `Fact` mapping is the join.
- (6): seeds `support::grants`, `class_rows`, `all_*_grants` are test-side copies the index could replace.

## 7. Shapes (uncommitted)

- **(a) Static `rules::grants::GrantIndex::new(&GameData)`**, pure and Build-independent: `gives(&DataAt) -> &[Site]`, `given_by(&Fact) -> &[(DataAt, Site)]`; a `Site` is a row at a level / a component / a favoured-weapon rule / a granted selection slot. Behind the seam: every grant kind in D plus a feat's and mythic feat's `selection`, switch features, mythic abilities, values kept. `Catalog` owns one; game-check builds one from `GameData`. Shrinks `holders`, `sites`, both `favoured_weapon_feats`, `pick_gives`, `is_named_by`'s scans, check.rs's scan. Deletion test: five copies come back. Tests in §3 survive; new real-data tests at `GrantIndex`. Risk: B's archetype-aware effective rows stay Build-side, so "holder" keeps two layers.
- **(b) (a) plus a Build-aware reader on `Engine`:** `Engine::grantors(state, fact)` absorbs `held_holder` and `within_reach`'s Feature arm; `Engine::what_pick_gives(build, slot, pick)` returns facts plus the slots it opens (fixes 330.2 at the root); `Slot` could carry its granting pick. game-check uses only the static half; two adapters make it a real seam.
- **(c) A "satisfies" seam:** `GrantIndex::can_satisfy(&DataAt, leaf: &Prerequisite) -> bool`, absorbing `is_named_by`, `carry`, `within_reach`. Couples to (5)'s normalised leaf; build after or with it. Most leverage per interface, but the deletion test depends on (5).
