# N: Narrowing check, detailed design

(Explorer, read-only, main at 790eff76, 2026-11-18. Fits `overall.md`. It reads P's view (T2) and G's static half (T3), and hands owners and the allowlist to L (T7). Paths are under `tools/game-check/` unless I say otherwise.)

## 1. Interface

There is one new module, `src/write/qualification.rs`. The prerequisites writer, `stale` and `audit` read it through one record.

```rust
/// Built once per plan. Cost: one pass over the dump, which yields both `granters` (today's `carriers::granters`)
/// and `listed_in` (the game selections whose `m_AllFeatures` names each blueprint); G's static index over the
/// `GameData`; and the stand-in table (each selection with `shared_options`, its game list read once).
pub struct Qualification<'a> { /* ctx, G index, granters, listed_in, stand_ins */ }

impl<'a> Qualification<'a> {
    pub fn new(ctx: &'a Ctx<'a>) -> Qualification<'a>;
    /// Every way `after` refuses a character `before` let in, or that the game lets in. Relative throughout:
    /// a carrier, pick or stand-in gap that `before` already had is not reported.
    pub fn narrowings(&self, entry: &DataAt, before: Option<&Prerequisite>, after: Option<&Prerequisite>)
        -> Vec<Narrowing>;
    /// The audit's absolute question: each carrier gap of one tree.
    pub fn carrier_gaps(&self, entry: &DataAt, tree: Option<&Prerequisite>) -> Vec<CarrierGap>;
}

/// The tree-only half: no `Ctx`, no dump. `narrowings` calls it and adds the evidence and the four
/// data-bound kinds. Public so that tests reach it without a world.
pub fn diff(before: Option<&Prerequisite>, after: Option<&Prerequisite>) -> Vec<TreeNarrowing>;

pub struct TreeNarrowing { pub kind: TreeKind, pub subject: Subject }
pub enum TreeKind { Drops, AddsNot, AddsNeed, RaisesValue { from: u8, to: u8 }, Tightens { was: u8, now: u8 } }

pub struct Narrowing { pub kind: Kind, pub subject: Subject, pub evidence: Evidence }
pub enum Kind {
    Drops,                                      // today's `drops`
    AddsNot,                                    // today's `adds Not`; the own-blueprint exemption stays
    AddsRequirement,                            // an added need no planner site, pick or choice can meet (324 a)
    RaisesValue { from: u8, to: u8 },           // a needed threshold up, or a refused one down
    Tightens { was: u8, now: u8 },              // Any→All or AtLeast raised: slack over the shared members
    Carrier(CarrierGap),                        // relative carrier gap (308h)
    StandIn { selection: FeatureSelectionKey }, // the entry stands for another blueprint in this selection (324 b)
}
/// A leaf with its threshold zeroed: a fact with its `Parameter` (`feat weapon-focus (longsword)`), or a leaf that
/// is no fact (an `Alignment`, an `AbilityScore` of 0). Ord by its display text, so today's report order holds.
pub enum Subject { Fact { at: DataAt, value: Option<Parameter> }, Leaf(Prerequisite) }
pub enum Evidence {
    Blueprint { name: String, guid: BlueprintGuid }, NotInDump(BlueprintGuid), PlannerOnly, NoFact,
    Unoffered { selection: BlueprintGuid, name: String, planner: Option<DataAt> }, // a game slot the planner lacks
    Shared { selection_blueprint: Option<BlueprintGuid>, own: BlueprintGuid },   // a stand-in
}
pub struct CarrierGap { pub fact: DataAt, pub grantor: BlueprintGuid, pub grantor_name: String, pub by: DataAt }
```

**`Narrowing::said()`** is the key L uses: `<kind> <subject>`. The forms are `drops feat x`, `adds Not feat x`, `adds feature x`, `raises class-level cleric to 5`, `tightens feat a, feat b to 2 of 2`, `carrier <gap>` and `stand-in in selection kineticist/expanded-element`. `Display` adds ` (<evidence>)` and **never an owner**. `CarrierGap`'s `Display` is today's text without ` -> 307`.

**The record**, in `prerequisites.rs`:

```rust
pub enum Scope<'s> { All, Only(&'s BTreeSet<String>) }
pub struct Plan { pub changes: Vec<Change>, pub entries: Vec<Planned> }   // entry order
pub struct Planned {
    pub at: DataAt,
    pub outcome: Outcome,              // Written | Equal | NoBlueprint | LeftWhole(Vec<Hold>)
    pub held_gaps: Vec<CarrierGap>,    // absolute, the tree the data holds
    pub game_gaps: Vec<CarrierGap>,    // absolute, the game's tree; empty when `target` gapped
    pub narrowings: Vec<Narrowing>,    // every one the rewrite makes, allowed or not
}
pub enum Hold { Hand, Gap { what: String, ticket: &'static str }, Narrowing(Narrowing) }
pub fn plan(ctx: &Ctx, scope: Scope, allowed: &Narrowings) -> Plan;
pub struct FullPlan(Plan);                           // only from `Scope::All`
impl FullPlan {
    pub fn stale(&self, allowed: &Narrowings) -> Vec<String>;  // allowlist lines no `narrowings` matches
    pub fn audit(&self) -> Vec<String>;   // `<at> | holds: <gap>` for held_gaps; `| left whole:` for LeftWhole
}
```

**Invariants**
- `FullPlan` exists only for `Scope::All`, so `stale` and `audit` can never read an `--only` plan.
- An entry is written iff `target` has no gap, the compiled trees differ, and each of its `narrowings` is allowed.
- `Carrier` and `StandIn` are never allowed by a `drops`/`adds Not` line; each needs a line of its own kind.
- `held_gaps` is computed for every entry, rewritten or not, because the audit needs it.

**Error modes.** None at runtime. `Narrowings::parse` refuses an unknown kind; the file and line rules are today's.

**Determinism**
- Entries come in `entries()` order (`prerequisites.rs:79`).
- Narrowings come in `Kind` order, then `Subject` text, with carrier gaps in tree order as today (`carrier_guard.rs:130`).
- `audit` is byte-identical to today's for the same data, except that ` -> 307` is gone and the relative change in §2 applies.

**Cost**
- One `plan` per check: `main.rs:225` and `:229` collapse into one call.
- `Planner::entries_of` (`planner.rs:42`) is a linear scan. It becomes a `BTreeMap<&BlueprintGuid, Vec<DataAt>>` built in `Planner::new` and returns a slice. It is called per fact, per chain and per entry (`prerequisites.rs:558`, `:834`, `:914`; `carrier_guard.rs:118`).
- The builder times the plain check (stderr prints it) before and after: medians of alternating runs on a quiet machine.

## 2. Behind the seam

**How `diff` reads a tree (P's view, normal form)**

Each leaf gives `(subject, threshold, refused)`. If T2 exposes no threshold, N keeps a private `split_threshold`: `ClassLevel.level`, `Feature.count`, `Rank.rank`, `SkillRanks.ranks`, both ability scores, `BaseAttackBonus`, `ClassSpellLevel`, `SpellLevel`, `SpontaneousSpellLevel`, `CasterLevel`, `CharacterLevel`, `MythicRank`. Each needed leaf also gets its nearest non-All group: members, and n of k.

**The tree kinds**
- **Drops** follows today's rule for a fact subject: any needed fact the new tree doesn't name. For a non-fact subject, only an alternative counts (a member of a group with slack): dropping an `Alignment` from an Any narrows, dropping a stat from an All widens.
- **AddsNot**: today's rule, with the own-GUID exemption at `:556-578`.
- **AddsNeed**: a needed subject the old tree doesn't name. It is a candidate only; it becomes `AddsRequirement` or `StandIn` only through the data rules below.
- **RaisesValue**: the same subject and polarity in both trees, with a needed threshold up or a refused one down.
- **Tightens**: for a group, slack (k − n) counted over the members both trees need. It fires when slack falls. Any(A,B)→All(A,B) goes from 1 to 0; AtLeast 1→2 of {A,B,C} goes from 2 to 1. Any(A,B,C)→Any(A,B) is a Drops of C, not a tightening.

**The data rules in `narrowings`**
- **AddsRequirement** comes from an `AddsNeed` whose fact either:
  - has no G site, is offered by no planner selection, and is not chosen (`Fact::is_grantable`); or
  - is listed in a game selection (`listed_in`) whose planner entry offers it nowhere: the rays' shape. WitchElementsPatronProgression `67e85f52…` is `options/witch/patron/elements.ron`, offered by `witch/patron`. The game's Elemental Witch picks it through ElementalWitchPatronSelection `3172b696…`, which the planner maps to the **feature** `witch/elemental-witch/elemental-patron`, and that feature offers nothing.
  - 324's literal rule ("no row or slot grants it") misses this second case, because `witch/patron` does offer the option.
  - Relative: a pick gap counts only when the old tree's needs didn't already have it.
- **Carrier**: `carrier_gaps(after)` minus `carrier_gaps(before)`, matched on `(fact, grantor, by)`.
- **StandIn**: the entry is a `SelectionOption` in a selection's `shared_options`, and that selection's game list doesn't name the entry's blueprint (or the selection has no blueprint). Then each `AddsNot`, `AddsNeed` and `RaisesValue` the diff found also yields a `StandIn`. This is the kineticist's `kineticist/element/air`: ElementalFocusAir `2bd0d449…` is offered by `selections/kineticist/expanded-element.ron` for SecondaryElementAir `659c3954…`.

**What moves or goes, function by function**
- `prerequisites.rs`
  - Module doc `:14-22` is rewritten.
  - `Named` `:455-501` is deleted; P's view takes its place.
  - `leaf_at` `:503-534` is deleted, apart from its `Fact`→`DataAt` arms (`:520-531`), which move into `Subject`.
  - `Narrowing` `:536-542` and `narrowings_of` `:544-581` move into qualification, typed.
  - `blueprint_of` `:583-593` becomes `Evidence::of`.
  - `plan` `:353-453` shrinks: `:399-433` become one call to `q.narrowings` plus the allowlist filter.
  - `Plan` `:324-335` becomes the record above.
  - `stale` `:259-277` and `audit` `:279-312` become `FullPlan` methods; neither re-plans.
  - `changes` `:315-322` stays a wrapper.
  - `owner` `:1011-1063` is no longer called for narrowings (`:567`). It stays private for `Gap`'s `unmapped` (`:1004`) and `entry_of` (`:922`) until T7 moves them to L.
  - `Narrowings` `:162-253` stays; only its kind check at `:213` widens to the seven kinds.
- `carrier_guard.rs`
  - `gaps` `:96-137` becomes `Qualification::carrier_gaps` and returns `CarrierGap`.
  - `walk` `:141-166` is deleted; P's view provides the alternatives.
  - `Grants` `:41-92` keeps only `granters` plus G's `carry`, because T3 already moved `sites` (`:183-323`) and `favoured_weapon_feats` (`:327`).
  - What remains folds into `qualification.rs`.
- `carriers_known.rs`
  - `split` `:45` keeps its fallback, so the committed lines, whose hit is the text before ` -> `, still match unchanged.
  - `regenerate` `:110-124` writes a new hit **with no ticket** and also returns the new hits, which `main.rs:159` prints to stderr. The main session must assign each one: `parse` (`:65-68`) refuses a ticketless line, so the next check exits 2 until it is assigned. That is 324's last item, with no new mechanism.
- `main.rs:144-166` and `:217-238`: one `plan(ctx, Scope::All, &narrowings)`, then `.stale` and `.audit`.
- `planner.rs:42`: the index.
- `write/mod.rs:778` and `report.rs` are unchanged.

**Hand-off to L (T7)**
- The write registry is keyed by `(DataAt, Narrowing::said())`. `Allowed(evidence)` replaces `Narrowings`.
- `CarrierGap`'s `Display` becomes the known-differences finding text.
- `owner()` and `Gap`'s tickets become L's.
- N exports no ticket.

**Key and grammar changes**
- The allowlist accepts `<entry> / adds|raises|tightens|carrier|stand-in …` alongside `drops` and `adds Not`. The file is empty, so nothing migrates.
- The writer's `left unwritten` lines lose `-> <owner>`, e.g. `narrows: drops feat keen-feat (KeenFeat <guid>)`.
- Audit hits lose ` -> 307`.

**Committed files**
- `carriers-known.txt` lines still match. Its `left whole` lines for entries the relative guard now lets through go stale, so T6 regenerates the list (removals only).
- Fixture strings drop the owner suffix.

## 3. Tickets

T6 is too big for one builder. It splits in two, run in sequence on the same files.

**T6a: one plan and one record; the relative carrier guard; no owners.**
- **Problem:** the check plans twice. `stale` and `audit` re-plan. Narrowings and carrier gaps are strings with guessed owners. The carrier guard is absolute, so it holds entries whose old tree already had the gap (308h's 17).
- **Change:** §1 and §2 except the four new kinds: `qualification.rs` with `diff` (Drops and AddsNot over P's view), `carrier_gaps`, the relative `Carrier`, `Plan`/`FullPlan`, the planner index, owners off, and `regenerate` without tickets.
- **Done when:**
  - the plain check calls `plan` once;
  - the audit's output equals today's, less ` -> 307` and less the `left whole` lines the relative guard releases (each named in the Answer, with its held tree's gap);
  - a dry run lists those entries as written, and nothing else moves.
- **Files:** `src/write/{qualification.rs (new), prerequisites.rs, carrier_guard.rs (folded), mod.rs}`, `src/carriers_known.rs`, `src/main.rs`, `src/planner.rs`, `carriers-known.txt` (regenerated), `tests/fixture/prerequisites_write.rs`, `tests/fixture/qualification.rs` (new, with its `mod` line in `main.rs`).
- **Checks:**
  - `cargo test -p game-check`, with the new names in `--list`;
  - `cargo fmt --all --check`; `scripts/test-modules.sh`;
  - the game check rerun (exit 0, no stale line), timed before and after;
  - `--carrier-audit` diffed against the base's;
  - `--write prerequisites --dry-run` diffed against the base's.
- **Model:** Opus to build.

**T6b: the four new kinds (ticket 324).**
- **Problem:** 324's whole list. An added requirement no planner site or slot meets; a stand-in entry; a leaf's value; Any→All. Regenerating the carrier list gave new hits `-> 307` unchecked; T6a closes that last one.
- **Change:** `RaisesValue`, `Tightens`, `AddsRequirement` (both rules) and `StandIn`; the allowlist grammar; `listed_in` and `stand_ins` in `Qualification::new`.
- **Done when:**
  - a full `--write prerequisites --dry-run` writes neither the Elemental Witch's three rays (left whole, `adds … (Unoffered ElementalWitchPatronSelection 3172b696…)`) nor `kineticist/element/{air,earth,fire}`;
  - the elements stay whole even with a scratch allowlist that allows their `adds Not archetype kineticist/blood-kineticist`, held by `StandIn`;
  - Greater Enduring Spells: see risk 5;
  - each new kind has a fixture or tree test;
  - the Answer gives the dry run's count per kind over every reached rewrite, with a scratch allow-all list, so the cost to future slices is visible.
- **Files:** `src/write/{qualification.rs, prerequisites.rs}`, `prerequisite-narrowing.txt` (header only), and the two test files.
- **Checks:** as for T6a, plus 324's issue file set to resolved.
- **Model:** Opus to build.

## 4. Tests

**Tree-only, through `qualification::diff`** (new `tests/fixture/qualification.rs`, no world)
- `a_dropped_alternative_is_a_drop`: Any(A,B)→A.
- `a_drop_under_two_nots_is_a_drop`: moved from the fixture.
- `a_drop_from_an_any_inside_an_all_is_a_drop`: moved.
- `an_added_refusal_is_an_adds_not`.
- `an_added_need_is_a_candidate_not_a_narrowing`: the diff yields `AddsNeed`, and only the data rules turn it into a narrowing.
- `a_raised_class_level_is_a_raised_value`: cleric 3→5.
- `a_lowered_refused_level_is_a_raised_value`: Not(ClassLevel 6)→Not(4).
- `a_lowered_need_narrows_nothing`.
- `an_any_turned_all_tightens`.
- `an_at_least_raised_tightens`.
- `an_alternative_dropped_is_a_drop_not_a_tightening`.
- `weapon_focus_for_another_weapon_drops_the_first`: a `Parameter` is identity, not a threshold.
- `an_alignment_dropped_from_an_any_is_a_drop`.
- `a_stat_dropped_from_an_all_narrows_nothing`.
- `a_normal_form_rewrite_narrows_nothing`: Any(Any(A,B),C) against Any(A,B,C), and AtLeast(1) against Any.
- `the_diff_is_in_kind_then_subject_order`.

**World-based** (`prerequisites_write.rs`, new world entries numbered from 240)
- `an_added_requirement_no_planner_site_gives_leaves_the_entry_whole`: 324 (a), literal.
- `an_added_requirement_a_game_selection_offers_and_its_planner_entry_doesnt_leaves_the_entry_whole`: the rays' shape, a planner feature mapping a game selection.
- `an_added_requirement_a_row_gives_is_written`: Spell Chain's shape.
- `a_carrier_gap_the_old_tree_had_is_written_and_audited_as_holds`: relative.
- `a_stand_in_entrys_added_refusal_needs_a_stand_in_line`: allowing `adds Not` leaves it whole; allowing `stand-in` too writes it.
- `a_stand_in_entry_whose_rewrite_adds_nothing_is_written`.
- `one_full_plan_answers_stale_and_audit`: `FullPlan::stale` and `::audit` reproduce today's two fixture expectations.
- `a_blocked_narrowing_names_its_evidence_and_no_owner`.
- `regenerating_the_carrier_list_leaves_a_new_hit_without_a_ticket`.
- `the_allowlist_parser_takes_each_narrowing_kind`.

**Survive unchanged:**
- 308a's writer tests (`:579` to `:765`, the hand, equal, `--only`, held-fact and mythic tests);
- `an_entrys_non_requires_components_are_kept`; `a_no_feature_on_a_shared_guid…` (`:1048`);
- the parser refusals (`:957`), except that one message lists the kinds;
- `the_embedded_allowlist_parses`; the stale test (`:995`), through `FullPlan`;
- `:1122`, `:1135`, `:1291`, `:1304`; the carrier-list parse tests (`:1267`, `:1283`).

**Strings change** (` -> 307` or ` -> 308` dropped): `:1088`, `:1104`, `:1145`, `carrier_hits` (`:1168`), `:1192`, the 308a narrowing tests, and `:1242` (its regeneration half).

**Deleted:** `a_blocked_narrowing_names_its_facts_owner`, and the world's `domain-feat`/`narrow-domain` (216, 217) if nothing else reads them.

## 5. Risks and unknowns

1. **P's view may not carry a threshold or group shape.** The builder reads T2's API first. If it lacks them, `split_threshold` stays private to N, and P gains them only if a second reader appears.
2. **The writer's `compiler` (`:338`) calls `compile_requires` and `canonicalize`, not `compile_prerequisites`** (the architect's unverified point). Settle by reading T1's change. N normalises explicitly in `compiler`. Equality on the normal form may turn some changes into `Equal`, so T6a's dry-run diff names each.
3. **The pick-gap rule's noise** (copies of the domain lists, 294; archetype copies of a class's list, 293). Settle by T6b's per-kind counts. If one family floods, add an evidence-cited exemption only when the planner models that copy through `shared_options` and `option_levels`.
4. **The relative guard releases entries.** T6a names each one with its held gap. None is written in T6: they wait for the next data slice, and L tracks them as `holds`.
5. **Greater Enduring Spells is not a narrowing.** It adds a reachable need, and it belongs to ticket 208. 324's done-when ("none of the four held fields") cannot be met by N without breaking the definition. The main session either amends 324 to the three rays (recommended) or holds it in the registry under 208.
6. **A stand-in selection with no blueprint** (planner-only) counts as a stand-in. The builder lists such selections in the Answer.
7. **The owner-free lines** change what the main session reads in `left unwritten`. `known.txt`, the check's own list, doesn't move.
8. **`regenerate`'s ticketless line exits 2 by design.** The `.new && mv` recipe still guards the embed. The header says so.
9. **T6 and T7 share `write/` and `report.rs`,** so they run in sequence. T6 must keep `Narrowings` whole for T7 to replace.
