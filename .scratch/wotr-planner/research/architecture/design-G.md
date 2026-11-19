# G: Grant index — detailed design

(Explorer, read-only, main at 790eff76, 2026-11-18. Fits `overall.md` §1 G, tickets T3 and T4. Findings: `4-grant-index.md`. Marks **[if the user …]** say where an answer other than the architect's recommendation changes this design.)

## 1. Interface

### Static half: `libs/rules/src/grants.rs`, `pub use grants::*`

```rust
/// Every grant in the game data, by granting entry and by fact. Built once; Build-independent; no IO (wasm-safe).
pub struct GrantIndex { /* sites: Vec<Site>, by_entry: FxHashMap<DataAt, Range<u32>>,
                         by_fact: FxHashMap<Fact, Vec<(u32, u16)>>, offered_in: FxHashMap<Fact, Vec<FeatureSelectionKey>> */ }

/// One place the data gives facts together: carrier_guard's "site".
pub struct Site {
    pub by: DataAt,                 // the granting entry
    pub class: Option<ClassKey>,    // a class row's class, an archetype row's base class
    pub when: When,
    pub grants: Vec<Grant>,
}

pub enum When {
    Always,                          // an option's feats or chosen feat, a feat's or mythic feat's selection,
                                     // a mythic ability's features/selection, a background's feats, a deity's
                                     // favoured weapons, a Path switch's features
    ClassRow { level: u8 },          // class or archetype level row
    OptionRow { level: u8 },         // option level row (option level)
    Rank { rank: u8 },               // a path's rank row
    Component(GrantRule),            // the component's own rule
}
pub enum GrantRule { Held, AtClassLevel { level: u8, before: bool }, IfHas { has: Fact, not: bool }, OnApply }

pub struct Grant {
    pub fact: Fact,
    pub value: GrantValue,
    pub opens_slot: bool,            // a row's selection, a feat's or mythic feat's `selection`, an ability's `selection`
}
pub enum GrantValue { Every, Fixed(Parameter), Picked }   // feats / feats_for, GrantWithValue, favoured / chosen feats

pub enum Satisfies { Yes, No }

impl GrantIndex {
    pub fn new(data: &GameData) -> Self;
    /// What an entry gives, in `When` then row order. Empty for an entry that gives nothing or that the data lacks.
    pub fn gives(&self, by: &DataAt) -> &[Site];
    /// Every site that grants `fact`, with the grant, ordered by (`by`, `when`).
    pub fn given_by(&self, fact: &Fact) -> impl Iterator<Item = (&Site, &Grant)>;
    /// The selections whose lists offer `fact` as a pick (own and shared options, feats, mythic abilities).
    pub fn offered_in(&self, fact: &Fact) -> &[FeatureSelectionKey];
}

impl Grant {
    /// Whether this grant meets P's leaf: never a refused leaf; the same Fact; a value the leaf asks for that the
    /// grant gives (`Every`; `Fixed` equal; `Picked` with `picked` equal, or `None` = "can, by some pick").
    pub fn satisfies(&self, leaf: &Leaf<'_>, picked: Option<&Parameter>) -> Satisfies;
}
```

`Leaf` is P's (overall §3): polarity, value, `fact: Option<Fact>`. A leaf with no Fact (BAB, a score, skill ranks) is `No`. A Feature leaf's `count` is not read: one grant adds one rank, and a site can only contribute. "Is it met" stays the walk's question. **[If T2's leaf carries no value, `satisfies` takes `(&Fact, Option<&Parameter>, refused: bool)`; T4 waits for T2 either way.]**

Invariants:
- Rows-only callers can still filter: every row grant of a class, archetype, path or option row is a `Site` with `When::ClassRow`, `OptionRow` or `Rank`.
- Sites are sorted by key, so `new(shuffled) == new(canonical)`. That keeps game-check's report byte-identical, and `Catalog::new`'s `canonical()` needs no ordering role here.
- `new` never panics on a key the data lacks (`check.rs` builds one before `checked()`).
- The index says "can give". Time-dependent application stays the walk's (overall §6): option levels, archetype replacement, `replaced_by`, `only_without_option`, a switch's drops.

Error modes: none. Lookups return empty.

Cost:
- Building is one linear pass over about 3,300 level rows, about 3,900 entries and 5 grant components (counted in `libs/game-data/`). That makes about 15k `Grant`s at roughly 80 bytes each, about 1.5 MB including hash maps.
- Reads are `O(1)` plus the answer.
- **Wasm startup:** `Engine::new` already runs `holders()` (`catalog.rs:999`). The browser logs `timed("engine", …)` (`libs/ui/src/planner.rs:281`).
- Measure in a release `cargo leptos` build on a quiet machine: the median of 5 page loads' "engine" console line on base and tip, alternating.
- Natively, an ignored seeds test `grant_index_build_time` prints `Instant` medians of 20 builds.
- Budget: under 10 ms native and under 5% of the "engine" line in wasm. Report both in the Answer.

### Build-aware half: on `Engine` (`libs/rules/src/engine/grants.rs`)

```rust
impl Engine {
    pub fn grants(&self) -> &GrantIndex;
    /// What a pick would give, the pick's own fact first, slots it opens included; static (no walk).
    pub fn pick_gives(&self, pick: &Pick) -> Gives;
    /// Which entry gives `fact` to the Build at `state`: the first class taken whose effective rows up to its level
    /// grant it (its archetype when the class's own rows don't), else a path or option held whose rows grant it.
    pub fn grantor_in(&self, state: &CharacterState, fact: &Fact) -> Option<DataAt>;
    /// Whether the Build's classes could give `fact`: some non-class site, no site at all, or a class or archetype
    /// site of a class (archetype) the Build has.
    pub fn in_reach(&self, state: &CharacterState, fact: &Fact) -> bool;
    /// The declared pick that opens `slot`, for a slot a pick opens: an Extra feat's `FeatSelection`, a rank's
    /// `MythicExtra` / `MythicSelection`. Read from the declared picks, no walk.
    pub fn opened_by(&self, build: &Build, slot: &SlotId) -> Option<SlotId>;
}

pub struct Gives { pub own: Fact, pub picked: Option<Parameter>, pub sites: Vec<&'e Site> /* by lifetime */ }
impl Gives {
    pub fn satisfies(&self, leaf: &Leaf<'_>) -> bool;        // own fact as a Grant(Every|picked) or any site's grant
    pub fn opens(&self) -> impl Iterator<Item = &FeatureSelectionKey>;
}
```

How `opened_by` resolves a slot:
- `FeatSelection { level, .. }`: the declared picks at `level`, and at the Mythic ranks taken at `level` (`rank_level`), whose `pick_gives(..).opens()` includes the selection that offers the slot's declared pick (`selection_of`). If several match, the first in `SlotId` order wins.
- `MythicExtra { rank }` and `MythicSelection { rank, .. }`: `Mythic { rank }`.
- Any other slot: `None`.
- Cost: `O(picks at that level)`.

`grantor_in` and `in_reach` keep today's answers exactly: they read row sites only (`When::ClassRow`, `OptionRow`, `Rank`), as `GrantedBy` did. **[Widening them to mythic-ability, switch and component sites is more like the game, but it changes reason texts: some "or"s keep alternatives they trim today. Raise it as wording item 6. If the user says yes, T4 drops the row filter and the reasons record lists the changes; otherwise the filter stays.]**

**`Slot` does not gain its granting pick.** 330.2's slot is *unreached*: its feat was refused, so the walk never makes it. `opened_by` answers from the declared Build instead. A reached slot already sits where its opener was held. Add `Slot::opened_by` only when a reader of a reached slot needs it (for example, withdraw naming the feat a slot dies with). That would change `Pending::Feat(FeatKey)` (`walk.rs:307`) to carry the opener's `SlotId`, plus `hold.rs:72`.

## 2. Behind the seam

| Today | Fate | Callers then call |
|---|---|---|
| `catalog.rs:1061-1142` `GrantedBy`, `Holders`, `holders()` | T3: deleted | `catalog.rs` derives `Vec<GrantedBy>` from `GrantIndex` row sites. The type stays until T4 so `labels.rs` is untouched. T4: deleted |
| `catalog.rs:777-798` `feature_holders`, `selection_holders` | T3: body shrinks to a lookup over the index's projection. T4: deleted | `labels.rs` → `Engine::grantor_in` / `grants()` |
| `catalog.rs:971-1059` `FeatureFacts.holders`, `selection_holders`; `shared_selection_names` (1009-1023) | T3: the two maps go. `shared_selection_names` reads "selections a row site opens" from the index | — |
| `catalog.rs:743-765` `favoured_weapon_feats` | T3: becomes `index.gives(&DataAt::Deity(d))` | `hold.rs:391-401` (unchanged signature) |
| `labels.rs:222-249` `held_holder` | T4: moves into `Engine::grantor_in` | `feature_name_held` (208-220) |
| `labels.rs:251-304` `sole_holder`, `selection_owner`, `feature_holders`, `holders` | T4: shrink to filters over `given_by` | — |
| `labels.rs:375-383` `within_reach`'s Feature arm | T4: `self.in_reach(state, &Fact::Feature(..))`. The ClassLevel and Archetype arms stay: they aren't grants | `unmet_for` |
| game-check `carrier_guard.rs:183-323` `sites` | T3: deleted | `index.gives(at)`, each site's facts |
| `carrier_guard.rs:41-73` `Grants::of` | T3: `sites` and `picks` become `GrantIndex::new(ctx.planner.data)` and `offered_in`. `granters` (the dump side) stays | `prerequisites.rs` `plan` / `audit` |
| `carrier_guard.rs:81-91` `carry` | T3: same rule over `index.gives` / `given_by` | `gaps` (until N absorbs it, T6) |
| `carrier_guard.rs:325-339` `favoured_weapon_feats` copy | T3: deleted | the deity's `Always` site, now with values |
| MCP `tree.rs:231-257` `pick_gives` | T4: deleted | `engine.pick_gives` |
| MCP `tree.rs:154-229` `Given`, `is_named_by` | T4: `Given` keeps `Class`, `Archetype`, `Score` (choices, not grants) and gains `Pick(Gives)`. The two row scans (189-196, 204-218) become `gives.satisfies(leaf)` and `index.gives(Class)` plus the class's archetypes | `Node::needs` |
| MCP `tree.rs:259-267` `leaves` | T2 (P) | — |
| MCP `wording.rs:98-123` `reached_through` | T4: gains `FeatSelection` → `engine.opened_by`. Its Mythic arms call `opened_by` too | `Wording::node` |
| `check.rs:1925-1945` counts-as scan | T3: `index.given_by(&Fact::Feature(f))` with a site of `class == Some(other)` | — |
| seeds `support.rs:940-1040` `grants`, `class_rows`, `all_class_grants`, `all_path_grants` | T3: same signatures, bodies over `GrantIndex` (the transitive closure stays in `support`) | seeds tests unchanged |

Deletion test: without G, the six copies above come back. With G, every "who grants" question has one place to fix.

## 3. Tickets

### T3: The grant index's static half

- **Problem.** Four crates each answer "what gives this fact" (findings §1–2). The carrier guard copies the favoured-weapon rule without values (308h, 324). Switch features and a feat's selection are in none.
- **Change.** Add `rules::grants` (§1 static). Build it in `Catalog::new`. Have `catalog.rs`, `check.rs:1925`, game-check's `carrier_guard` and seeds `support` read it, as in §2's T3 rows. `labels.rs` is untouched.
- **Done when:**
  - Every grant kind in §1's `When` has a site, pinned by tests.
  - The engine's statuses and reasons are unchanged, shown by the form proof.
  - The game check exits 0.
  - The carrier audit's before and after are diffed, and each changed hit is explained in the Answer: a new site that now carries a fact (a hit gone), or a new site that breaks "every site gives both" (a hit added).
  - The wasm and native costs are measured.
- **Files:**
  - `libs/rules/src/{grants.rs, lib.rs, engine/catalog.rs, check.rs}`
  - `libs/rules/tests/engine/{grants.rs, main.rs}`
  - `libs/seeds/tests/game_data/{support.rs, grant_index.rs, main.rs}`
  - `tools/game-check/src/write/carrier_guard.rs`, `tools/game-check/tests/fixture/prerequisites_write.rs`
  - `tools/game-check/carriers-known.txt` (if hits move)
- **Checks:**
  - `cargo test -p rules -p seeds -p corpus -p game-check`
  - `cargo build -p rules --target wasm32-unknown-unknown`
  - `cargo fmt --all --check`
  - **Form proof** on its base, in its lane, before the first edit: `FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1` (T0's reasons record), then `compare` on the tip. Expect 0 differences.
  - The game check.
  - `--carrier-audit` on base and tip, diffed.
  - New or gone hits: `carriers-known.txt` is regenerated with its header command. The builder lists each new line in the Answer with no ticket assigned, and the main session sets each line's ticket (overall: owners live in L). **[If T7 lands before T3, the hits go to `known.txt`'s carrier area instead.]**
- **Beside T1:** yes, if T1's diff leaves `catalog.rs` and `check.rs` alone. T1's normal form lives in `component.rs`/`prerequisite.rs`, and the second `canonical()` at `catalog.rs:85` "stays harmless" (overall §4). `labels.rs` is shared with T1, so T3 keeps the `feature_holders` / `selection_holders` signatures and does not touch `labels.rs`. Before spawning, run `git diff --stat main...<T1 branch>` and confirm neither file is in it. Both lanes touch `seeds` support only if T1 does; check the same way. **[If T1's builder needs `check.rs` for `condition_names`, the `check.rs:1925` move goes to T4.]**
- **Model:** Opus to build: cross-crate, with judgement on the audit's hits.

### T4: The grant index's Engine half; closes 330 item 2

- **Problem.** `validate` places an Extra feat's slot under its level's class, so a refused Extra feat's slot pick is a separate root labelled "Level 3: slot" (330.2). `pick_gives` omits a feat's `selection`, an ability's features and option rows' chosen feats. `held_holder`, `within_reach` and `is_named_by` each re-scan rows.
- **Change.** Add §1's Engine half. Make `labels.rs`, MCP `tree.rs` and `wording.rs` read it, as in §2's T4 rows. Delete `GrantedBy`.
- **Done when:**
  - Instinctual Warrior 3 with `extra-rage-power` refused and a pick at `slot.feat-selection.3.0`: `validate` nests that slot's line under the feat's line, with no root of its own.
  - Its label names the selection the feat would open, not "slot". The exact text gets an exact-text test; label style is `placed_label`'s.
  - The MCP `validate` tests pass unchanged.
  - The form proof (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`) shows statuses unchanged, and reasons unchanged unless wording item 6 is accepted. In that case `compare` lists each change, grouped.
- **Files:**
  - `libs/rules/src/engine/{grants.rs, labels.rs, catalog.rs}`, `libs/rules/src/engine.rs` (module)
  - `apps/mcp/src/validate/{tree.rs, wording.rs, declare.rs}`
  - `apps/mcp/tests/server/validate.rs`
  - `libs/rules/tests/engine/grants.rs`
  - `libs/seeds/tests/game_data/grant_index.rs`
- **Checks:**
  - `cargo test -p rules -p mcp -p seeds -p corpus`
  - the rules wasm build
  - fmt
  - the form proof as above
  - No game check: no data or game-check diff.
- **After** T2 and T3, and before T5 (R), which shares `labels.rs` and `validate/`. **[If the user picks (c) for R, R first: T4 follows R and puts `opened_by`'s placement in R's module instead of `wording.rs`. If (b): no change for T4.]**
- **Model:** Opus to build.

## 4. Tests

New, at the interface (`libs/rules/tests/engine/grants.rs`, test world):

- `every_grant_kind_is_a_site`: one entry per `When`/`GrantValue` kind gives its site:
  - a class-row feat, `feats_for` (Fixed), an option's chosen feat (Picked);
  - `GrantAtClassLevel`, a feat's selection (`opens_slot`), a mythic ability's features;
  - a switch's features, a background's feats;
  - a deity's unarmed strike (Improved Unarmed Strike) and an exotic weapon (Exotic Weapon Proficiency, Fixed to that weapon).
- `given_by_is_the_inverse_of_gives`.
- `the_index_ignores_the_datas_order`: shuffled data gives an equal index.
- `an_unknown_key_gives_nothing`.
- `a_refused_leaf_is_never_satisfied`; `a_grant_for_one_value_meets_only_that_value`; `a_grant_for_every_value_meets_any`; `a_picked_value_meets_the_picks_value`.
- `row_sites_are_the_old_holders`: T3 only. It pins the projection against the `GrantedBy` lists and is deleted with `GrantedBy` in T4.
- T4: `an_extra_feat_gives_its_selections_slot`; `opened_by_names_the_extra_feat_at_the_slots_level`; `opened_by_names_the_first_of_two_extra_feats`; `a_ranks_extra_slot_is_opened_by_its_general_pick`.

Real data (`libs/seeds/tests/game_data/grant_index.rs`):

- `the_index_agrees_with_the_walk`, a self-check:
  - Over the gold Builds and the seeds' Build fixtures, every Feature, Feat, Option and Selection the walk's last state holds has a site in `given_by` whose `by` the Build holds. That means: a class or archetype taken; an option, feat, ability, mythic feat or path held; the background, the deity, the switch; or the fact was a pick.
  - Its long run over the corpus's ~2,000 Builds goes in `tools/corpus/tests/self_checks.rs` under `--ignored`.
  - A miss is an index bug, never a test exception.
- Game-cited:
  - `irori_gives_improved_unarmed_strike` (IroriFeature 23a77a5985de08349820429ce1b5a234, from 308h).
  - `extra_rage_power_opens_a_rage_power_slot` (ExtraRagePower; the builder cites the name and GUID from `game-check show`).
  - `legends_switch_grants_its_features` (the switch blueprint, cited by the builder).
- `grant_index_build_time`, ignored: the timing in §1.

MCP: `a_refused_extra_feat_holds_its_slots_pick` (`validate.rs`): 330.2's exact tree and label.

game-check fixtures: `an_exotic_favoured_weapon_gives_its_proficiency_for_that_weapon`; `a_switch_feature_carries_its_fact`.

What happens to existing tests:
- **Survive unchanged:** `collapsed_features.rs` (58-120); the picker-reason tests (`skald_rage_powers.rs`, `ki_powers.rs`, `instinctual_rage_powers.rs`, `wild_talents.rs`); `validate.rs` 57-540; the carrier tests in `prerequisites_write.rs` (1088-1304); `tree.rs`'s `echoes` tests; every seeds caller of `support::grants`, `class_rows` and `all_*_grants`.
- **Move:** none.
- **Deleted:** only T3's transitional `row_sites_are_the_old_holders`.

## 5. Risks and unknowns

1. **"Can give" against "gives"**: for example, `option_row` (Bloodline Ascendance), lists-only arcana, and `only_without_option`. The index over-approximates by design. The agreement test checks only that every held fact has an indexed grantor. Settle: the builder runs it. A held fact with no site is a missing grant kind and gets added. Over-approximation is documented, not tested.
2. **`Grant::opens_slot` and whether the walk holds `Fact::Selection` for a feat's selection** (`level.rs:377`, `ascension.rs:210`). Settle: read both and pin the answer in `every_grant_kind_is_a_site`. The agreement test catches a mismatch.
3. **Carrier audit drift.** New sites (switch features, a feat's selection, values) can add hits as well as remove them through `carry`'s "every site gives both" rule. Settle: diff the audit, explain each hit in the Answer, and have the main session assign tickets.
4. **`opened_by` ambiguity**: two Extra feats at one level opening the same selection. The walk's order (band deferral) can differ from `SlotId` order. Settle: a test pins first-in-`SlotId`-order. If the corpus has such a Build, check `validate`'s tree by hand.
5. **Wasm startup**: unmeasured (§1 budget). Settle: measure in T3. If it is over budget, build `offered_in` lazily or drop `by_fact` for facts nothing reads.
6. **P's `Leaf` shape** is not built yet. T4's `satisfies` waits for T2. If T2 changes the leaf's Fact mapping (for example `FeatIfValue`), `satisfies` follows it.
7. **`check.rs` builds the index on unchecked data.** Settle: `an_unknown_key_gives_nothing`, plus the existing `check` tests on bad data.
8. **game-check determinism.** Settle: `tools/game-check/tests/determinism.rs` passes, and the report is byte-identical where the audit's hits didn't change.
9. **A new open decision for the user: wording item 6** (widening `in_reach` / `grantor_in`, §1). Until answered, T4 keeps today's row filter.
