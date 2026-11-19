# Candidate 1: one qualification diff behind the prerequisites writer — findings

(Explorer report, read-only, main at 790eff76, 2026-11-18. Paths under `tools/game-check/` unless said otherwise. The narrowing allowlist has zero entries today, only its header: 308c–f used up and removed every line.)

## 1. Current state

**`src/write/prerequisites.rs`**
- `plan` (353–453) is the real module. Over `entries` (79–142): skips an entry with no blueprint or JSON; lists a hand prerequisite; `target` builds the game's `Requires` (596–633) with `Gap`s from `condition` (636–822); compiles before and after (`compiler`, 338); then runs two guards on different footings: `narrowings_of(before, after)` (relative) and `carrier_guard::gaps(after)` (absolute, new tree only).
- `plan` returns `Plan {changes, left, made, carried, grants}` (325–335). Three wrappers each re-run the whole plan: `changes` (315), the writer, from `write/mod.rs:778`; `stale` (259–277), only to read `made`, from `main.rs:225`; `audit` (283–312), with `Narrowings::default()` to read `carried`, then `carrier_guard::gaps` again per held tree, from `main.rs:158` (`--carrier-audit`) and `main.rs:229` (plain check).
- `narrowings_of` (549–581): "drops X" = a fact the old tree accepts that the new tree doesn't name at all; "adds Not X" = a newly refused fact, except entries of the entry's own GUID.
- `Named::walk` (464–501) tracks polarity only; All, Any, AtLeast flattened alike; an accepted non-fact leaf is ignored (493).
- `leaf_at` (505–534) names a fact by entry and drops its value, level and rank; also used at `carrier_guard.rs:35`.
- `Narrowing {said, blueprint, owner}` (538); `blueprint_of` (584); `owner` (1013–1063) guesses the owning ticket from the asset name.
- `Narrowings` (166–253) parses the allowlist, keyed by strings `(entry, "drops …")`.

**`src/write/carrier_guard.rs`**: `Grants {granters, sites, picks}` (41–92), `Grants::carry` (81); `gaps` (96–137): per required fact, `lookup::chains_up` to a mapped grantor; a planner entry of that grantor not carrying the fact is a gap, always `-> 307`; `walk` (141–166): Any and AtLeast(0|1) widen, anything under `Not` skipped; `sites` (183–323) re-reads each planner row kind; `favoured_weapon_feats` (327) mirrors `libs/rules/src/engine/catalog.rs:746`.

**`src/carriers_known.rs`**: `KnownCarriers::parse`, `compare` (87), `regenerate` (110); a hit matches on its text before the last ` -> ` (`split`, 45).

**Wiring**: `report.rs:269` `with_stale_narrowings`, `report.rs:286` `with_carriers`; `main.rs:217–238` plain check runs `stale` then `audit` (plans twice); `write/mod.rs:675` `run` allows no narrowing (fixtures); `run_with` (696) takes the list, passed at `main.rs:303`.

## 2. The concept and its variants

Shared question: does this tree refuse a character the game allows? Each guard answers part, with its own idea of:

**"Narrower"**
- Narrowing guard: relative to the old tree, by fact name only. Blind to an added requirement (324: Elemental Witch's rays, Stalwart Defender), a raised value or level (`ClassLevel` 3→5, a higher stat), Any → All or a raised AtLeast count, a dropped accepted non-fact leaf. A refused non-fact leaf is keyed by its `Debug` text, value included (491): polarities asymmetric.
- Carrier guard: absolute on the new tree; leaves an entry whole even when the old tree had the same gap (308h found 17 such `holds` hits, "no regression").
- `Gap`: "the writer can't say it", no comparison.
- 324's stand-in case (an entry in `shared_options` standing for another blueprint): absolute too.

**"Owner"**: a narrowing `owner()` falling back to "308" (567); an unmapped blueprint `owner()` → "307" (1007); `entry_of` `owner()` → "308" (922); a carrier gap always "307" (127); a `Gap` hard-coded per kind (308/313/314/317/174); `carriers-known.txt` and `known.txt` overridden by hand (308g kept five blocked entries' hand owners in `known.txt` because `owner()` names none).

**"Evidence"**: a narrowing gives the fact's blueprint name and GUID, or "no blueprint"; a carrier gap the grantor's name and GUID plus the planner entry; the allowlist a free-text reason nothing verifies (308d found four lines with false evidence, "other blueprints").

Everything leaves as a string (`Unexpressed.what`, `Plan.made`); `stale`, `audit`, `KnownCarriers` match on those strings.

## 3. Tests today

All in `tests/fixture/prerequisites_write.rs` on one hand-built world (95–446), through `run`/`run_with` → `Outcome.unexpressed` strings and file contents, `prerequisites::stale`, `prerequisites::audit`, `Narrowings::parse`, `KnownCarriers::*`.
- 308a: `a_rewrite_that_drops_an_old_alternative…`, `…adds_a_negation…`, `an_allowlisted_narrowing_is_written`, `a_no_feature_on_a_shared_guid_becomes_a_not_of_each_entry` (1048).
- 308g: `a_blocked_narrowing_names_its_facts_owner`, `an_allowlist_line_that_matches_no_narrowing_fails_the_check` (995), `the_allowlist_parser_refuses_a_malformed_line` (957), `a_shared_own_refusal…` (947), `a_drop_under_two_nots…`, `a_drop_from_an_any_inside_an_all…`.
- 308h: carrier tests (1088–1164, 1291, 1304).
- 322: carrier-list tests (1192–1285); `command.rs:62–77` checks only `--known-format` usage.
- Untested: Any → All, a raised value, an added `Has`, a stand-in (324's list); the carrier guard on a gap the old tree already had; `owner()` beyond one domain case; the plain check's combined wiring (`main.rs:217–238`); determinism of `audit`/`stale` (`determinism.rs` has neither); allowlist evidence. Every narrowing test needs the whole dump-backed world though the diff is pure tree logic.

## 4. Damage from the scatter

- 308a first review: four writes refused characters the game allows → the narrowing guard (808ff242).
- 308g (926e841e): stale allowlist lines never reported; a blocked narrowing printed `-> 308` instead of its owner; an unreachable parser branch.
- 308d (5b1878f3): Defensive Study and the Stalwart Defender written then reverted by hand; Endurance "an added requirement, which the guard doesn't count" (308d lines 24, 42–43); four allowlist lines with false evidence.
- 308h (cd16bad8): first audit 292 hits; three rules added to remove false positives (rows giving both, deities, `Any` alternatives) → 176: the definition still settling.
- 322 (0b9c8c1a, e6c1e211): a third list and comparison added to the check.
- 308f (4af12b11): Elemental Witch's rays and air/earth/fire elements put back by hand; no guard saw them (308f lines 31–38).
- 308's "308 left": a full dry run still writes the three rays; only never omitting `--only` stops it (`issues/308-importer-prerequisites.md:29–34`).
- 324 open; also notes regenerating the carrier list gives any new hit `-> 307` unchecked.

## 5. Constraints

- Crates: game-check depends on `rules`, `seeds`, `serde_json` etc.; nothing depends on game-check. Owner and evidence need the dump (`Ctx::name`, `owner`), so stay in game-check. A pure tree diff could live in `rules` but must build for wasm32 and know nothing of blueprints or tickets.
- Performance: the plain check plans twice, each run may build `Grants::of` (scans the dump, `carriers::granters`, `write/carriers.rs:294`); `Planner::entries_of` (`planner.rs:42`) is a linear scan called per fact, chain and entry. A shared module should plan once and serve all three readers.
- Formats (fixed, committed, matched on exact strings, asserted by fixtures): allowlist `<entry> / drops|adds Not <fact> [-> reason]` and `<entry> -> reason`; `carriers-known.txt` `<entry> | holds|left whole: <gap> -> <ticket>`.
- Determinism: entry order, BTreeMaps, order-preserving dedupe in `gaps` (130); the carrier list is regenerated and diffed.

## 6. Overlap

- (2): the engine's `allows` answers the same question dynamically; form_proof's `StatusSplit` and census narrowing (322 item 2, `tools/corpus/tests/form_proof.rs`) already measure "who newly refuses" over ~2,156 Builds — the semantic oracle for this static diff. Shared: `rules::Prerequisite`, `Condition`, `Fact`, `DataAt`.
- (3): `Narrowings`, `KnownCarriers`, `Known` (`src/known.rs`), `Facts` (`write/facts.rs`) share one shape (parse, line numbers, `-> <ticket>`, stale lines); this module's answers are what the ledger lists; its record type should be the ledger's key.
- (4): `carrier_guard::Grants` and `sites` re-read rows as `engine/catalog.rs` `holders`/`GrantedBy` (1061–1094) does; `favoured_weapon_feats` copied; 324's "no row, grant or selection gives it" needs exactly this index; `Grants.picks` already includes `shared_options` (`carrier_guard.rs:57`).
- (5): `Named::walk` (polarity), `carrier_guard::walk` (alternatives), `leaf_at`, `Prerequisite::leaves` and `canonicalize` (`libs/rules/src/prerequisite.rs:174, 268`), MCP `Given::is_named_by` (`apps/mcp/src/validate/tree.rs:156–171`). A leaf view with polarity, alternatives and value would serve both guards.
- (6): the seeds tests that caught what the guards missed: 308d's `mythic_class_prerequisites.rs`, 308f's `importer_lists::the_elemental_witch_picks_one_of_four_rays`.

## 7. Shapes (uncommitted)

- **A. A `qualification` module in game-check:** `Qualification::of(ctx, grants, entry, before, after) -> Vec<Refusal>`, `Refusal { kind, fact, evidence, owner }`, kind ∈ drops, adds Not, adds requirement, raises value, Any → All, carrier, stand-in. `plan` writes when the list is empty or allowlisted; `stale` and `audit` read answers recorded on one `Plan` (no re-planning). Deletion test: `narrowings_of`, `Named`, `carrier_guard::gaps`/`walk`, `owner`, the string matching in `stale` and `audit` collapse into one place. World-based writer, stale, audit, carrier-list tests survive if strings stay; narrowing-shape tests could move to the module's interface.
- **B. Two layers:** a pure diff over (5)'s normalised reading, `diff(old, new) -> Vec<Change>`, structural, no dump, could sit in `rules`; a game-check adapter adds evidence, owner, carrier and stand-in checks (absolute, dump-bound). Two seams; 308g's narrowing-shape tests (`nested-not`, `all-any`, `wide-feat`, `deny-feat`) become tree-only unit tests, 324's value and Any → All cases cheap to pin. Only one adapter (game-check), so the `rules` seam is hypothetical unless (2) or (5) adopt it.
- **C. Semantic by evaluation:** evaluate old and new trees against a set of characters as form_proof does. Exact incl. value changes, too heavy for the writer's loop; keep as a test oracle beside A or B.

Whatever the shape, settle §2's mismatch first: does a carrier or stand-in refusal count only when the old tree didn't already make it (relative), or always (absolute)?
