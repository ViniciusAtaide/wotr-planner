# Share selection options between selections

Type: task
Status: open
Blocked by: 24
Map: [WotR Build Planner](../map.md)

## Question

**User decision (2026-11-01, [ticket 21](21-schema-questions-real-data.md) S4, option (a)).** 47 options are copies of another selection's (36 rogue talents in `slayer/slayer-talents`, 11 witch hexes in `shaman/hex`), so a rogue/slayer can take one talent twice.

- **Rust:** `FeatureSelection.shared_options: BTreeSet<SelectionOptionKey>` (`#[serde(default)]`). An option keeps the key of the selection that defines it; the engine offers own ∪ shared, and already-held compares the one key.
- **Schema:** `selection_shared_option (selection_key, option_key, PRIMARY KEY (both), CHECK (NOT starts_with(option_key, selection_key || '/')))`.
- **Data:** delete the 47 copies and their prerequisite trees; the slayer shares the 36 rogue talents and the shaman the 11 witch hexes.
- **Class-dependent prerequisites:** where a copy named its own class's feature (the 16 advanced slayer talents need `slayer/advanced-talents`), the shared option's prerequisite accepts either class's feature with an `Any`. Accepted trade-off: a rogue 10 / slayer 2 can then take an advanced talent through the slayer slot.
- **A slot on two lists:** Fey Tricks (rogue talents + witch hexes) and Underground Chemist (rogue talents + alchemist discoveries) become their own selections sharing both lists, if the guide data has their slots; otherwise list them in the report.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB): data by one-off script, extractor emits the new shape, docs updated. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus a test that a rogue/slayer can't take one talent twice.
