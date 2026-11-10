# Rogue Secret and Wizard Feat waive their options' prerequisites

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Two Loremaster secrets choose from another class's list with its prerequisites waived:

- **Rogue Secret:** "any Rogue talent without having to meet the prerequisites", Advanced Talents included. Ticket [125](125-favorite-metamagic-picks-its-metamagic.md)'s audit found it backed by the guide but not built.
- **Wizard Feat** (`loremaster/secret/wizard-feat`): "A Wizard bonus feat with prerequisites waived; Metamagic (Perfectly Ordinary Spell) isn't offered."

A selection slot can't waive its options' prerequisites today. Feat slots can: since ticket [16](16-bonus-feats-ignore-prerequisites.md), a Bonus-list feat slot skips the feat's prerequisites but still checks its list and that the feat isn't already held.

Options:
- **A.** Ticket 16's rule for selection slots too: a slot that waives prerequisites still checks its list, already-held and repeats. Rogue Secret gives a slot of the rogue talents (Advanced Talents included); Wizard Feat gives a slot of the wizard's bonus feat list, less Perfectly Ordinary Spell.
- **B.** Leave both unbuilt, and record Rogue Secret as checked in `pick_values`' audit list.

*Recommend:* A.

Questions for the build: does "without having to meet the prerequisites" also waive the rogue level that Advanced Talents need? The guide's "Advanced Talents included" says yes. Does Wizard Feat take a Bonus-list feat slot, which already waives prerequisites, or a selection slot?

## Done when

A Loremaster with Rogue Secret picks any rogue talent, Advanced Talents included, whatever the Build holds. A Loremaster with Wizard Feat picks any wizard bonus feat but Perfectly Ordinary Spell, prerequisites waived. Each pick is still refused when the Build already holds it. Real-data tests cover both. The audit list drops Rogue Secret. `scripts/proof.sh` shows that only Builds holding these secrets differ. `scripts/check.sh` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): A. Rogue Secret also waives the rogue level that Advanced Talents need, as the guide says "Advanced Talents included".
