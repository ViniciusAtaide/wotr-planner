# The six deepenings: overall architecture

(Architect, read-only, main at 790eff76, 2026-11-18. Frame for each explorer's detailed design; signatures are theirs. Verified for this frame: the crate graph; `compile_prerequisites` runs inside `GameData::checked`; the walk's `failures` never yields `Unmet::MetLater` (only `Slot::try_pick` does); the form proof records status kinds, never reason texts; `rules::agreement` sits behind the `testing` feature; the snapshot check was removed (map, 235). Unverified: whether the browser's JSON carries the compiled tree; whether the writer's before/after compile goes through `compile_prerequisites`.)

## 1. The modules

Six candidates, six modules, but not one-for-one: candidate 1 splits (its pure tree logic is module P's view), candidate 3 splits by direction, and candidate 4's "satisfies" seam joins P and G.

- **P: Prerequisite reading** (`rules`, `prerequisite.rs`). *What does this Prerequisite ask, in one shape?* Two halves behind one seam: a normal-form invariant on the existing enum (flatten All-in-All and Any-in-Any, `AtLeast(1)` as Any, cancel `Not(Not)`, a group of one as its child), applied once at load; and a read-only polarity-aware view whose leaves carry their **Fact**. Absorbs the flattening in `and_parts` and `all_texts`, the three `Not(Not)` guards, `leaves`/`prerequisite_leaves`/`unmet_leaves`, game-check's `leaf_at`, `Named::walk` and `carrier_guard::walk`, MCP `tree.rs::leaves`, `check.rs`'s `condition_names`. Game-check's `areas/prerequisites.rs` `canon` keeps only GUID and alignment-mask folding.
- **G: Grant index** (`rules`, new `grants`). *Which sites give this Fact, and what does this entry give?* A static, Build-independent half over `&GameData` (every grant kind: rows, feats, `feats_for`, chosen feats, components, a feat's and mythic feat's `selection`, mythic abilities, switch features, backgrounds, deities' favoured weapons with their values), and a Build-aware reader on `Engine` (who holds it in this Build, what a pick gives including the slots it opens). It also answers "does this site satisfy this leaf", reading P's leaf Fact. Absorbs `GrantedBy`/`holders`, `held_holder`, `within_reach`'s Feature arm, game-check's `sites`, `Grants::of`/`carry` and its `favoured_weapon_feats` copy, MCP `pick_gives` and `Given::is_named_by`'s scans, `check.rs:1925`, seeds `support::grants`/`class_rows`/`all_*_grants`. Two adapters (the engine's `Catalog`, game-check's dump-free `GameData`): a real seam.
- **R: Refusal** (`rules`, engine). *Why can't this Pick or value be taken at this Address in this Build, and what would let it in?* Takes the Build, the Address, the pick or value and an `Issue` from whichever source (picker or walk); returns the issue worded with its trim and held-where, its needs parts, the later choices it breaks named, and a lazy later-rank answer (rank, level, ranks that move) with its cache inside. Absorbs MCP `issue_reason`, the reason block duplicated in `options.rs`/`choose.rs`, `Wording::trimmed`/`unmet`, `later_rank_level`/`later_rank_clause`/`LaterLevels`, `tree.rs::needs`, picker.rs's later-rank code (via 199). Four adapters stay outside: web (" · "), MCP `options`/`choose` (`Fix::Choose`), `validate` (`Fix::Declare`), `withdraw`. Names no tool (CODING_STANDARDS:9).
- **N: Narrowing check** (`game-check`, new `qualification` in `write/`). *Does this rewrite refuse a character the game allows?* One definition over P's view and G's index, run once per plan, recording typed **Narrowings** (drops, adds Not, adds requirement, raises value, Any→All, carrier, stand-in) with their evidence (blueprint name and GUID). `stale` and `audit` read the plan's record instead of re-planning. Absorbs `narrowings_of`, `Named`, `carrier_guard::gaps`, the string matching in `stale`/`audit`, and every `owner()` guess (owner moves to L). Absorbs ticket 324.
- **L: Ledgers** (`game-check`, new `ledger`). Split by direction, on one shared line reader and two-sided reconcile (file name in errors, line numbers, `#` rule, ` -> ` split, duplicates, stale lines). **Known differences** (*is this finding decided, and which ticket removes it?*): `known.txt` with carrier hits as findings of their own area, so `carriers-known.txt`, `with_carriers` and `carriers_known.rs` go and 307c's list is one more area. **Write registry** (*may the writer make this change?*): Free, Held, Applied(ticket), Allowed(evidence), over `user-facts.txt` (byte-identical) and the narrowing allowlist, keyed by N's Narrowing.
- **X: Expected refusals** (`rules`, `#[cfg(feature = "testing")]`, beside `agreement`). *Is each Failure of this old Build one the game is known to refuse, by which rule?* Rule rows (entry, slot kind, issue shape, citation, ticket, optional dependent pick) given as text by the caller (rules has no IO); answers excused, per-rule hits, unexcused. Absorbs `support.rs` helpers A–D and their ordered chains, `links.rs`'s Battle Prowess, skald and rogue-talent lists, and `306-refused.txt` (a Build is refused-as-expected iff every Failure is excused). Rows in one file per ticket so data tickets share none. One adapter (the engine); its seam is rows vs matcher.

## 2. The layering

```
 rules ─────────────────────────────────────────────────────────────
   P  prerequisite reading  (normal form at load + view, leaf→Fact)
   ▲            ▲                    ▲                 ▲
   │            G  grant index ──────┤                 │
   │            ▲  (static | Engine) │                 │
   │            │                    │                 │
   R  refusal ──┘   (reads Issue/Unmet from walk & pickers)
   ▲                                                   │
   Offer (200) embeds R                    X expected refusals [testing]
   ▲            ▲                          (reads Engine::failures, P)
 ui (web)   mcp (options, choose,            ▲              ▲
             validate, withdraw)          seeds tests   corpus tests
 game-check (no Engine; GameData only) ─────────────────────────────
   N narrowing check ──reads──▶ P view, G static half
   ▲
   L ledgers (known differences | write registry) keyed by N's Narrowing
```

Arrows point at what is read. Nothing in `rules` reads up; game-check reads only P and G's static half.

## 3. Shared vocabulary

- **Normal-form Prerequisite** (P): the existing enum under the invariant; every tree after `checked()` holds it, and `check_in` therefore yields a normal `Unmet`.
- **Reading / Leaf** (P): a leaf with its polarity (needed or refused), its value and its **Fact** (the type exists in `component.rs`; P owns the leaf↔Fact mapping, `Fact::leaf` included).
- **Site** (G): where a grant comes from (a row at a level, a component, a favoured-weapon rule, a granted selection slot), with its granting entry.
- **Refusal** (R): the engine's worded answer for one Pick at one Address. New glossary term, proposed through `domain-modeling`; never named "Offer".
- **Narrowing** (N): one typed way a rewrite refuses more, with evidence and no owner. Distinct from R's Refusal: different crate, different question.
- **Ledger line / Decision** (L): a key, a decision (ticket, held, applied, allowed with evidence), its file and line.
- **Excuse rule** (X): entry, slot kind, issue shape, citation, ticket. Its issue shape reads P's normal form, so a rule never hand-matches a nested `Unmet`.

## 4. Cross-cutting decisions

- **Where the normal form applies:** at load, in `compile_prerequisites` (inside `GameData::checked`), so the engine, game-check and seeds see one tree; the view is derived on demand (cached in `Catalog` where hot). Reason: one invariant, zero per-call cost; `Catalog::new`'s second canonicalise stays harmless. `unmet_for`'s collapse re-normalises its result.
- **Relative vs absolute:** N's writer guard is relative (a carrier or stand-in gap counts only when the old tree didn't already have it); the plain check's audit stays absolute, listed in L's known differences with owners. Reason: the guard prevents regressions, the ledger tracks the backlog (308h's 17 "holds" hits are backlog, not regressions).
- **`validate` asks the walk**, and R words what the walk returns. Reason: a Declared Build is one walk per call (CODING_STANDARDS:17); R is issue-source-agnostic, so one wording holds anyway, and R's lazy later-rank answer closes 330.1. `MetLater` stays picker-only (out, §6).
- **The empty narrowing allowlist survives** as the write registry's Allowed decision, evidence required. Reason: 324's added-requirement and stand-in cases need it; its staleness comes from the one plan, so the second `plan` per check goes.
- **One ledger, which lists:** `known.txt` + `carriers-known.txt` (+307c) become one; `user-facts.txt` + the allowlist share the write registry. Apart: `gold::KNOWN_GAPS` (guarded, the user's), `agreement` `KNOWN` (engine vs engine), `option_lists::EXCEPTIONS` (another concept), and X's rows (another crate). Game-check's exit codes stay 0/1/2 with today's meanings.
- **For the user: `held` user-fact staleness in the plain check.** (a) As today: stale only under `--write` (exit 2). (b) The plain check exits 1 on a `held` line no difference matches. Recommend (a): (b) adds a failure on a guarded file each clearing needs a `User approved:` line.
- **R and 199–202: build on, and insert.** 199 lands as written; R's ticket follows it and precedes 200, whose Offer then carries a Refusal for its status/reason/like/opens fields. **For the user**, since 199–202 belong to another session: (a) insert R between 199 and 200 (recommended: 200 lifts finished reasons instead of moving the duplicated block); (b) R after 202 (200 moves the block as is, R reworks it inside `rules`); (c) R first, 199 rewritten on it. Also for that session: 199's and 200's "snapshot check" lines read as the form proof (`FORM_PROOF_OFFERS=1` plus the reasons record below).
- **For the user: wording changes.** Each changes text a player or agent reads:
  1. Any-in-Any loses its brackets ("the Fey Tricks selection or the Rogue's Rogue Talents selection or …"). Recommend yes; it falls out of P.
  2. `AtLeast(1)` reads as an Any: "Needs 1 more of: Kinetic Healer" becomes "Needs Kinetic Healer". Recommend yes.
  3. "N more of" with none held reads "N of" (330.3). Recommend yes (already ticketed).
  4. The MCP entry line in a rank's slot gains 171's hint ("for Longsword"), against 199's "MCP output unchanged". Recommend yes, in R's ticket.
  5. Refusal order (reasons refusals-first, prerequisite text refusals-last): recommend keep both (65 and 329 chose them).
  **Proof:** a new opt-in reasons record in the form proof (`FORM_PROOF_REASONS=1`: each non-ok option's reason and each Failure's text, streamed) so a form-only ticket proves no reason moved and a wording ticket's `compare` lists each changed reason, grouped; each intended change also gets an exact-text test.
- **Owners live only in L.** N records evidence; an unlisted Narrowing or regenerated carrier line prints no guessed ticket and is printed for the main session to assign (324's last item).

## 5. Order of work

Lane A is `rules`/`mcp`/`ui`; lane B is `tools/` and tests. At most two builders.

| # | Lands | Unblocks | Lane | After | Kind |
|---|---|---|---|---|---|
| T0 | Form proof's reasons record (`tools/corpus/tests/form_proof.rs`) | proving every wording step | B | — | tooling |
| T1 | P's normal form at load; 330.3; the wording the user accepts | T2, X | A | T0, user's wording answer | statuses form-only (`FORM_PROOF_OFFERS=1`); texts change only as accepted |
| T2 | P's view and leaf→Fact; game-check `Named`/`leaf_at`/walk and MCP `leaves` adopt it | N, T4 | A | T1 | form-only (form proof; game-check report byte-identical) |
| T3 | G's static half; `Catalog` and game-check `sites` read it | N, T4 | B | T0; beside T1 only if T1 leaves `catalog.rs` alone | engine form-only; carrier audit may gain hits (listed) |
| T4 | G's Engine half (what a pick gives, slots included; holder in this Build); closes 330.2 | R | A | T2, T3 | behaviour: `validate`'s tree |
| — | 199 (other session) | R | A | — | as its ticket |
| T5 | R; closes 330.1 and the duplicated block | 200 | A | T1, T4, 199 | wording form-only by the reasons record, except accepted item 4 |
| T6 | N in game-check; absorbs 324 | T7 | B | T2, T3 | behaviour of the tool (new Narrowing kinds; game check rerun) |
| T7 | L: one known-differences ledger, the write registry | 307c | B | T6 | tool output form; exit codes unchanged |
| T8 | X in `rules` (testing); helpers A–D, `links.rs` lists and `306-refused.txt` go | — | B | T1, T7 (lane free) | test-only: every corpus link's fate unchanged, every rule hit |
| — | 200 → 201 → 202 (other session) | — | A | T5 | as their tickets |

T1 and T2 share `prerequisite.rs`, T4 and T5 share `labels.rs`/MCP `validate/`, T6 and T7 share `write/` and `report.rs`: each pair runs in sequence.

## 6. What stays out

- **N (cand. 1):** a structural diff seam in `rules` (one adapter: hypothetical); diff by evaluation (C) in the writer's loop (too heavy; the form proof's census narrowing stays the oracle).
- **R (cand. 2):** `validate` asking the picker (B: a picker check per failing slot, unmeasured) and so `MetLater` in `validate`: later, with a measured cost; an MCP-only `refusal.rs` (C: contradicts 199/200).
- **L (cand. 3):** a full generic `Ledger<Key, Decision>` (B: moves Facts' and Narrowings' complexity into generics, risks the guarded `held` lines); a cross-crate core shared with `KNOWN_GAPS`, `agreement`, `EXCEPTIONS` and X (one small reconcile duplicated is cheaper than a seam across `rules` and `game-check`).
- **G (cand. 4):** companions and pets; time-dependent application (option levels, skipped rows, switch drops) stays in the walk.
- **P (cand. 5):** `GameData::check` refusing non-normal hand trees (normalising them is enough); De Morgan in the stored tree (only the view pushes polarity, so "not with A or B" keeps its wording); a separate normalised `Unmet` (C: falls out of the invariant).
- **X (cand. 6):** per-Build rows (S1: ~460 Builds × picks, tied to generator names) and the hybrid (S3: keeps one shared file); guarding rows (mechanical checks instead: cited, GUID is the entry's or its slot kind's, hit, exact shape).
- **Elsewhere:** 330 items 4–9; making the per-picker calls private (200's later ticket).
