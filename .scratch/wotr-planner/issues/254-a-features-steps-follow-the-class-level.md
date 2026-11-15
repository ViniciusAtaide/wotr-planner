# A feature's steps follow the class level, not repeated grants

Type: design (to decide in the ticket, recommendation below), then build (engine rule)
Status: open
Blocked by: none
Model: Opus to build: a change of model the class tickets then apply.
Map: [WotR Build Planner](../map.md)

## Problem

The planner shows a feature that improves with level by granting it again at each step (a rank, since [243](243-ranked-features-and-filtered-pools.md)). The game rarely does that:
- **It grants the feature once and scales it by class level** (`UnitLogic/Mechanics/Components/ContextRankConfig.cs`; `Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs`, WrathDecompiled 6ce4a6b), or **grants a separate extra-use feature** at each step (`SmiteEvilAdditionalUse` 0f5c99ffb9c084545bbbe960b825d137, `JudgmentAdditionalUse`, the warpriest's `SacredWeaponEnchantPlus2` to `Plus5`, …).
- **The game check's group "progression rows / a further rank in planner only"** (557 findings, filed under the class tickets 268–287) is mostly this shape: the cavalier's challenge, tactician and banner, the warpriest's fervor, sacred weapon and armour, smite evil, the druids' wild shape, and the archetypes' own scaling features.
- **The opposite group, "a further rank in game only"** (160), is where the game grants a feature again and the planner once: bomb dice (`AlchemistBombsFeature` c59b2f256f5a70a4d896568658315b7d), Infusion Specialization, Inspire Courage and Competence, Poison Resistance. Those are ranks the planner should grant (243's model), class by class.
- **The engine rule the code contradicts** (242's research, rule R-rank): a step on class level follows the class level whether or not a later row is traded away. The Primalist (`PrimalistTakeBloodlinePower4` 2140040bf367e8b4a9c6a632820becbe, …) who takes rage powers at 8 keeps the resistance taken at 4, and it still rises at 8 with bloodrager level, since the step is the buff's rank on bloodrager level, not a row (`BloodragerInfernalBloodline` 9aef64b53406f114cb43f898a3aec01e).
- **Steps the blueprints don't show** (242's research, part 1): 17 scaling features have no step in their blueprints (Greenrager's Furious Summoning, Banner of Solace, Primal Size, Shield Block, Drunken Resilience, Bones and Soul of Stone, Touch of Flame, Wind Treader, Shadows, Caustic Ridicule, Prompted Backstabbing, Abyssal Bloodrage, Adaptive Physiology, Inhibit Courage and Greatness, Battle Spirit). One pass over the code they use comes before any class ticket drops or keeps their steps.

## Change

- **Decide the model** (recommended): a feature granted once may carry its steps, the levels where its numbers rise, read from the game (its `ContextRankConfig` or `AddFeatureOnClassLevel`) and computed from the class level the game reads, never from rows. A row grants a feature again only where the game does (a rank, 243). An extra-use feature the game grants is its own entry. The sheet and the MCP's gained features show steps as they show ranks today.
- **The engine** computes a step from the class level the game names, whatever rows an archetype or a Primalist trades away.
- **The code pass** over the 17 features above, each recorded as "steps at …" or "no step" with its code citation, for the class tickets to apply.
- **No class's data** changes here but one worked example (the cavalier's challenge, say) to prove the model; the class tickets 268–287 convert their own findings.

## Tests 242 marks as waiting on this rule

- `scaling_features.rs`: `a_primalists_rage_powers_replace_the_row_and_its_step_with_it` (rage powers at 4: never a resistance; at 8: the resistance taken at 4 still rises at 8).
- The other `scaling_features.rs` tests wait on their class tickets, each listed there.

## Done when

- The model is recorded in the Answer, with who decided it.
- The Primalist test holds the game's answer; a fixture test shows a step computed from class level that survives a traded-away row.
- The 17 features' steps are recorded, each with its citation.
- The scoped checks pass.
