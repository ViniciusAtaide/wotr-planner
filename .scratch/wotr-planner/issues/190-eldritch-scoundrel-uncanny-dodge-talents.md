# The Eldritch Scoundrel can't take (Improved) Uncanny Dodge as a rogue talent

Type: bugfix
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. The guide's rogue page, under Rogue Talents: "An Eldritch Scoundrel can select (Improved) Uncanny Dodge as Rogue Talents." The Eldritch Scoundrel loses both features ("No Uncanny Dodge"), and the data's own summary of `rogue/rogue-talents` says it "can also take (Improved) Uncanny Dodge" (`libs/game-data/classes/rogue.ron`). But `options` on an Eldritch Scoundrel's talent slot (`slot.selection.8.0`, `only_open: false`) offers neither.

## Build

- Two options of `rogue/rogue-talents`, Uncanny Dodge and Improved Uncanny Dodge, each granting the rogue's feature (`rogue/uncanny-dodge`, `rogue/improved-uncanny-dodge`), open only to an Eldritch Scoundrel. The selection is shared with the bard and the skald (ticket [60](60-borrowed-class-selections.md)), so the prerequisite names the archetype.
- Whether Improved Uncanny Dodge needs Uncanny Dodge first: the guide's sentence doesn't say. Take the rogue's order (Uncanny Dodge at 4, Improved at 8) as the prerequisite only if the guide supports it; otherwise ask the user.
- The same shape as [173](173-uncanny-dodge-as-a-slayer-talent.md) (Uncanny Dodge as a slayer talent), but here the guide names it, so no decision is needed.

## Done when

An Eldritch Scoundrel is offered both at its talent slots (2, 8, 12, 16, 20) and no other class or archetype is; a real-data test pins it; `scripts/check.sh` passes.

## Answer

Done 2026-11-13.

1. **The data** (`libs/game-data/classes/rogue.ron`, `rogue/rogue-talents`): two options, `rogue/rogue-talents/uncanny-dodge` and `rogue/rogue-talents/improved-uncanny-dodge`, linking to `rogue#rogue-talents` (where the guide's sentence is). Each grants the rogue's own feature (`rogue/uncanny-dodge`, `rogue/improved-uncanny-dodge`) through a level-1 option row. Uncanny Dodge needs `Archetype("rogue/eldritch-scoundrel")`; Improved needs that and `Feature("rogue/uncanny-dodge")`. No Rust, schema or link change.
2. **The guide.** The Eldritch Scoundrel's table has talents at 2, 8, 12, 16 and 20 and no Uncanny Dodge or Improved row ("No Uncanny Dodge"); the data matched. The guide says nothing of an order.
3. **Readers and double grants.** Nothing reads the two rogue features (no prerequisite, bonus or interaction), and every class and archetype has its own Uncanny Dodge keys, so the rogue's can't be granted twice: the Eldritch Scoundrel's rows omit them, and each option can be taken once. A feature held twice would only count 2.
4. **Who is offered them.** Only an Eldritch Scoundrel. The bard and skald borrow the selection and see both locked. The slayer, Fey Tricks, Underground Chemist, Feyform Shifter and Vivisectionist list their shared keys one by one, so they gained nothing. An Eldritch Scoundrel with bard or skald levels may take them through those slots (`Archetype` is Build-wide): accepted, as for the Sanctified Slayer.
5. **Tests** (`libs/seeds/tests/game_data/borrowed_selections.rs`): `an_eldritch_scoundrel_is_offered_uncanny_dodge_at_each_talent_level`, `an_eldritch_scoundrel_takes_improved_uncanny_dodge_only_after_uncanny_dodge`, `the_scoundrels_talents_grant_the_rogues_features_once`, `no_other_class_or_archetype_is_offered_the_scoundrels_talents`, `a_scoundrel_with_bard_levels_may_take_them_through_the_bards_slots`; the two counts of the rogue talents there went from 36 to 38 (the shared lists stay 36). No snapshot changed.
6. **Decisions** (decided by the main session, 2026-11-13, under the user's "answer the easy ones"): the options grant the rogue's own feature keys; the multiclass edge is accepted; Improved needing Uncanny Dodge is built as option A, provisionally.

### Open for the user

Does Improved Uncanny Dodge need Uncanny Dodge first? The guide doesn't state it; the build takes the rogue's order (Uncanny Dodge at 4, Improved at 8) as `Feature("rogue/uncanny-dodge", count: 1)` on the Improved option. If the user says no, remove that condition and rewrite the test that pins the order.
