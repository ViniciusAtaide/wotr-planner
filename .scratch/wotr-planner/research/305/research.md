# Ticket 305 research: one held-facts map (2026-11-16)

A read-only Opus agent wrote this on base `1387bbeb`, with ticket 302's branch in view. Line numbers are the base's; tickets 302–304 have since moved some. The decisions on it are in [decisions.md](decisions.md).

## 1. The maps today (`libs/rules/src/state.rs`, about lines 70–114)

**The maps:**
- `feats: BTreeMap<FeatKey,u8>`, with `feat_values: BTreeMap<FeatKey, BTreeMap<Parameter,u8>>`.
- `features: BTreeMap<FeatureKey,u8>`.
- `selection_options: BTreeMap<SelectionOptionKey,u8>`, with `option_values`. The caches `options_through`, `option_groups` and `option_selections` sit beside it.
- `mythic_abilities: BTreeSet` (presence only), with `mythic_ability_values` (a set) and `mythic_ability_open_holds` (a count).
- `mythic_feats: BTreeSet`, with `mythic_feat_values` (a set).
- `selections: BTreeMap<FeatureSelectionKey,u8>`.

**Writers:** only `Walker::hold`/`release` and `add_feature` (`hold.rs`). They are called from:
- picks (`hold.rs`);
- class rows (`level.rs`);
- option rows (`options.rs`);
- path ranks (`ascension.rs`);
- components' `Grant` (`components.rs`, `held_of`).

**Readers:**
- `check_in` in `state.rs`;
- `Fact::rank` in `component.rs`, and through it `components.rs` and `level.rs` (`NoSelectionIf`);
- `judge.rs`, for repeats;
- `slots.rs`;
- `labels.rs` (`option_count`);
- `engine.rs`, `attack.rs`, `caster.rs`;
- `catalog.rs`, for proficiency;
- `bonuses.rs`, `hold.rs`, `ascension.rs`, `level.rs`, `options.rs`;
- the MCP `sheet.rs` (`new_feats`/`new_features`, which diff the states before and after a level);
- the tests (`gold.rs`, `support.rs` and struct literals).

The UI and the codec never read them.

## 2. The game's model (WrathDecompiled 6ce4a6b)

- **One list keyed by blueprint and parameter:** `FeatureCollection : UnitLogicCollection<Feature>`, reached as `UnitProgressionData.Features`.
- **Adding:** `FeatureCollection.PrepareFactForAttach` makes a parametrised feature with a new `Param` a new instance; otherwise the first instance of that blueprint gets `AddRank`.
- **The rank cap:** `Feature.AddRank` stops at `Blueprint.Ranks`.
- **Removing:** `PrepareFactForDetach` removes the fact when its rank is below 2, otherwise calls `RemoveRank`.
- **Reads:**
  - `GetRank(blueprint)` ignores the parameter.
  - `UnitHelper.GetFeature(feature, param)` matches both.
  - `EntityFactsProcessor.HasFact(blueprint)` matches the blueprint only, and `PrerequisiteFeature.CheckInternal` uses `HasFact`.
- **Source and level:** `Feature.SetSource` keeps `m_Source`, `SourceLevel` and, when `Ranks > 1`, a per-rank `m_RankToSource`.
- **Selections are facts:**
  - `LevelUpHelper.AddFeaturesFromProgression` adds a granted selection itself (`unit.AddFact(item)`).
  - `SelectFeature.Apply` adds the picked item and records the pick in `UnitProgressionData.AddSelection` (`m_Selections`), which is the game's form of `options_through`.

## 3. The proposed map

```rust
pub held: BTreeMap<Fact, HeldFact>   // Fact: component.rs's enum, the grantable kinds
pub struct HeldFact { holds: u8, values: BTreeMap<Parameter, u8> }
```

- **`values`** mirrors the game's instance per `Param`.
- **The rank arithmetic** stays one function per kind: a count for feats, features, options and selections; presence for the mythic entries.
- **The views** (`feats()`, `features()`, `options()`, `mythic_*()`) are `range` scans over one variant. `Fact`'s derived `Ord` (variant, then key) iterates each kind in today's order, so the sheet, the gold tests, `attack.rs`'s "first in key order" and `caster.rs` stay byte-identical.
- **Entries with zero holds are removed,** so `CharacterState: Eq` holds unchanged.
- **Source and level aren't needed,** since no reader reads them.

## 4. Could any answer change?

**Keyed by `Fact`:** no. Every reader gets the same answer by construction.

**Keyed by GUID, answers would change.** 90 game blueprints are held under two or more planner keys:
- the Stunning Fist feat and the monk's Stunning Fist feature;
- Wings, both a feat and the angel feature;
- the rogue-talent Weapon Focus option and the Weapon Focus feat (1e1f627d…);
- 39 feature/option pairs;
- features repeated across archetypes;
- 302's selections that carry their feat's GUID.

Those are planner splits for 293 (stage 308).

**Holding the selections of feats as facts,** as the game does, changes no offer, number or `Requires` result:
- no data names a selection in a condition;
- no `NoSelectionIf` is in the data;
- none of the 83 feat selections has components;
- `selections` is read only by `P::Selection` and `Fact::rank(Selection)`.

## 5. Planned edits and tests

**Edits:**
- Add `HeldFact` and the map, with its views. `check_in` and `Fact::rank` read the map.
- `hold`/`release` write one entry.
- Drop the `offers_feats` filters at the row-hold sites, and hold the feat selections of path ranks, of the race and subrace, and of the general slot.
- Port every reader.

**Tests:**
- each view equals the old map's iteration;
- a held selection of feats is a fact (`Prerequisite::Selection("feats")` holds at level 1, citing BasicFeatSelection and `AddFeaturesFromProgression`);
- a mythic feat held twice keeps rank 1;
- the form proof shows 0 differences;
- `corpus --test links` passes;
- the scoped checks pass.

## 6. Files

**The rules crate (`libs/rules/src/`):**
- `state.rs`, `component.rs`, `engine.rs`;
- `engine/`: `attack.rs`, `caster.rs`, `catalog.rs`, `slots.rs`, `labels.rs`;
- `engine/walk/`: `hold.rs`, `level.rs`, `options.rs`, `ascension.rs`, `judge.rs`, `components.rs`, `bonuses.rs`.

**Elsewhere:**
- `apps/mcp/src/sheet.rs`;
- the tests in rules and seeds;
- the data README and the 301 note's "305" lines.
