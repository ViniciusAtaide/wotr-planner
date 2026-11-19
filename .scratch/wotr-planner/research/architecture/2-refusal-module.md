# Candidate 2: one refusal module for a Pick at an Address — findings

(Explorer report, read-only, main at 790eff76, 2026-11-18. GLOSSARY.md's uncommitted **Offer** entry, lines 326–328, was read, not touched.)

## 1. Current state

**Engine (`libs/rules`)**
- `issue_text` (`engine/labels.rs:639`): one line for an `Issue`; `None` for `BreaksLater` (the web app greys that option with no reason; glossary **Removable**).
- `reason_for` (`labels.rs:309`): trims an "or" to the alternatives the character's classes can reach, via `within_reach` (`labels.rs:389`).
- `held_where` (`engine/pickers.rs:223`): where a pick is already held.
- `slot_entries_in` (`pickers.rs:163–200`) puts these together for each `EntryOption.reason`, and each `SlotEntry.reason` via `shown()` (`engine.rs:516`).
- Later-rank functions in `engine/ranks.rs`: ticket 171 `rank_level_that_opens` (387, one walk per level tried); ticket 192 `later_rank_pick` (491, no walk), `later_rank_text` (526), `later_rank_levels` (549, two walks per level), `level_that_lets_in` (578); `Engine::moves_text`. None assembles a reason; each front end calls them.

**MCP: `issue_reason` call sites (15 calls, 4 files)**
- `issue_reason` (`apps/mcp/src/options.rs:277`) = `issue_text` with `BREAKS_LATER` (118) as fallback.
- Calls: `options.rs:990`; `choose.rs:242`; `validate/tree.rs:277` (`needs`); `validate/wording.rs:134, 161, 163, 192, 198, 306, 317, 428, 441, 467, 553, 556`.
- `issue_code` (`options.rs:256`) used at `options.rs:983`, `choose.rs:233`, `tree.rs:57`, `validate/mod.rs:103`.

**MCP: assembling the reason**
- `Offer` (`options.rs:24–58`): `status`, `reason`, `like`, `opens`, `gear`.
- `slot_offers` (626–738): per value, `later_rank_level` (744, ticket 171, eager) else `later_rank_clause` (790, ticket 192); per entry of values, only `later_rank_clause`.
- The reason block is copied verbatim at `options.rs:984–998` and `choose.rs:234–250` (`apply_offer`): `offer.reason`, else `Breaks::reason(edit|like)` (193), else `issue_reason`, then `LaterLevels::tail(Fix::Choose)` (828).
- `Fix` (759) is `Choose`/`Declare`; `opens_text` (766) adds the action clause.
- `withdraw.rs:89–113` words `not_removable` from `Breaks::refusals`, `cause`, `names`, with its own text.

**MCP `validate` (`validate/wording.rs`)**: words from the walk's `Failure`, never a picker. `reason` (127) branches on the issue; `trimmed` (141) reruns `reason_for` on the sheet at the slot's level; `unmet` (158) calls `later_rank_clause` and `LaterLevels::tail(Fix::Declare)` on the Build without the failing pick; `already_held` (189) calls `held_where`. `unmet` never calls `rank_level_that_opens`: the cause of 330 item 1. `tree.rs:271` `needs` splits the `Unmet` with `and_parts`, words each part via `issue_reason`, strips "Needs ". `echoes` (341) drops a reason repeating `needs`.

**Web app (`libs/ui/src/picker.rs`)**: `reason` (449) = `issue_text` (older pickers). `SlotPicker` (481) takes `slot_entries_in`, runs `later_rank_levels` (617–684) with its own `opens_text` and `joined` (687); computes 171 and 192 eagerly for every locked option.

## 2. The question

"Why can't this Pick (or value) be taken at this Address in this Build, and what would let it in?"

| Variant | Engine text | Held where | `reason_for` trim | 171 hint | 192 clause + level | Breaks names choices | Action clause |
|---|---|---|---|---|---|---|---|
| web slot picker | yes | yes | yes | yes, also on entry "for X" | yes, " · " | no (grey) | none |
| web older pickers | `issue_text` | – | – | – | – | no | none |
| MCP `options` value | yes | yes | yes | yes | yes | yes / `BREAKS_LATER` | `choose … first` |
| MCP `options` entry (values=false) | yes | yes | yes | **no** | yes | via `like` | `choose` |
| MCP `choose` | as `options` values=true | | | yes | yes | yes | `choose` |
| MCP `validate` | walk issue | yes | re-trimmed | **no** (330.1) | yes, only when the pick is the only failure | n/a | `declare …` |
| MCP `withdraw` | – | – | – | – | – | own text | `withdraw X first` |

Drift today: web "…opens it, moving MR 5…" vs MCP "…, which moves MR 5…" (ticket 199).

## 3. Tests today

- rules: `tests/engine/labels.rs` (`issue_text`, incl. 329's `an_and_inside_an_and_…`); `slot_entries.rs` (held where, entry reasons); `rank_levels.rs:620–956` (later-rank functions one by one, never a whole reason).
- seeds: ~26 direct `issue_text` calls plus helpers (`support.rs:724` `lock_reason`, `:771`) in `game_data/*`.
- MCP: `rank_levels.rs` (26; 171 pinned only through `choose`, line 83; 192 through `options`+`choose` 743 and `validate` 784); `choose.rs` (19, incl. Breaks texts 589, 627, 694, 724); `options.rs` (6); `validate.rs` (8); `withdraw.rs:177`; `wording.rs` (329) asserts one string across `validate`, `options`, `choose` — the only "one wording" check; `tree.rs:353–380` unit-tests `echoes`.
- Untested: web later-rank texts (picker.rs tests only `lift`, 1369–1380); `validate`'s 171 hint; the MCP entry line lacking 171; `BREAKS_LATER` as last fallback. Self-checks (`tools/corpus/tests/self_checks.rs`, `rules/src/agreement.rs:235–451`) check statuses, never reasons.

## 4. Damage from the scatter

- 330 item 1: `validate` drops "MR 9 taken at level 20 opens it…" (`later_rank_level` private to `options.rs`).
- 199: web/MCP drift "moving" vs "which moves"; five engine calls and a per-rank cache repeated per front end; MCP doc comment "as the web app says it" (`options.rs:788`).
- 192 fix rounds: d0097f67 (validate trims an or as the picker does; Clear Mind "Barbarian level 8 … or Skald level 8" only in validate); 9ce1f809 (one later-rank note for options and validate); 192 items 119, 124, 125; 8118dd5c produced `unreached.rs`, the model CODING_STANDARDS:8 names.
- 329 item 2: the doubled level clause fixed in three readers (`unmet_reason`, `all_texts`, MCP `tree.rs::needs`).
- The `options`/`choose` reason block duplicated verbatim.

## 5. Constraints

- Dependencies: `ui` (→ `frontend`, wasm) and `mcp` depend on `rules`; `rules` depends on no app crate and builds for wasm32-unknown-unknown (std-only plus base64/miniz/serde/rustc-hash).
- CODING_STANDARDS: line 9 `rules` names no tool and no address (so ": `choose` `mythic.4.level` 12 first", "declare …", "`slot.x` holds now" (`Breaks::reason`, `options.rs:235`) and `BREAKS_LATER`'s `sheet` hint stay in MCP; the engine returns rank, level, moves as data); line 7 a reason is true for this Build; line 8 one wording per refusal; line 10 one referent per pronoun; line 12 each fact once (`echoes`); line 16 a status is behaviour (reasons off the status path); line 17 walk per call, not per option.
- Performance: `rank_level_that_opens` runs eagerly per locked Unmet option in a rank's slot in MCP (`options.rs:696`, even for rows `only_open` later drops) and web (`picker.rs:641`). The 192 level is lazy in MCP (`LaterLevels`), eager in web. `choose` rebuilds every offer with values per step and per stepper step (`choose.rs:216, 289`); `missing_parameter` again (404). Sweeps use statuses only, so a lazy refusal module costs them nothing; eager reasons in `slot_entries` would charge the seeds tests.

## 6. Overlap

- (4) grant index: `reason_for` → `within_reach` decides which alternatives a reason keeps; `ranks.rs` `rank_picks_meeting` / `names_pick` (132) answer "which held pick meets this leaf", the same question as MCP `Given::is_named_by` / `pick_gives` (`tree.rs:171, 232`).
- (5) normalised reading: `Unmet::and_parts` (`choice.rs`), `unmet_reason` (`labels.rs:678`), MCP `tree.rs::leaves` (260) and `needs` (271), `ranks.rs::unmet_leaves` (106) — `leaves` and `unmet_leaves` are duplicates. A refusal's `needs` list is a reading of the `Unmet` tree.
- (6) expected refusals: `support.rs` `lock_reason` and siblings plus ~26 `issue_text` calls become callers of the refusal interface.
- (1), (3): no shared types; game-check calls none of these.
- Offer (tickets 200/201): the refusal is the `status`/`reason`/`like`/`opens` slice of `Offer`.

## 7. Shapes (uncommitted)

- **A. `Refusal` value in `rules` beside `Offer`.** `Engine::refusal(build, sheet, address, pick|value, issue) -> Refusal` holding the issue worded with `reason_for` + `held_where`, `needs` parts, the named broken choices, a lazy `later` with `.opens()` keeping the per-rank cache inside and returning `{rank, level, moves}`, the engine's text. Each front end keeps a small adapter (web " · ", MCP `Fix::{Choose, Declare}`, `withdraw`) — three adapters, a real seam. Deletion test: removing it brings back `issue_reason`, the duplicated block, `trimmed`, `later_rank_clause` and picker.rs 617–695 in three places. Tests: MCP texts and `rank_levels.rs` move into rules tests through `refusal`; the per-tool triplets shrink to one adapter test each. 199 is its first step; 200's `Offer` embeds `Refusal`; unlike 199/200, `validate`'s Failure path calls it too; closes 330.1.
- **B. `validate` asks the picker** (`Engine::offers` of 200, or `offer_for(build_without_pick, address, value)`). One code path; costs a picker check per failing slot; the picker's issue may differ from the walk's (try_pick adds `MetLater`; `failures` may not — unverified). Subsumes more of 200; needs a measured cost (CODING_STANDARDS:17).
- **C. MCP-only `refusal.rs`** on the `unreached.rs` model: fixes 330.1 and the duplication fast; web stays separate until 199; conflicts with 199/200's "the engine words it". A stopgap.

Open: should the MCP entry line in a rank's slot gain the 171 hint (199 item 1's "for Longsword" implies yes, against 199's "MCP output unchanged")? Does `validate` get the swap hint, i.e. does the walk produce `MetLater`? Unverified.
