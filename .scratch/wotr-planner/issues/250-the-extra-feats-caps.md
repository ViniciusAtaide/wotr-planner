# The Extra feats' caps

Type: build (engine rule and data)
Status: open
Blocked by: [249](249-selections-and-parametrised-feats-are-taken-again.md)
Model: Opus to build: the caps come from the game's conditions and ranks, read per feat.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15).
- **The ten Extra-talent selections** (Extra Rogue Talent, Extra Slayer Talent, Extra Mercy, Extra Arcanist Exploit, Extra Wild Talent, Extra Hex, …) are selections, so [249](249-selections-and-parametrised-feats-are-taken-again.md)'s rule lets them repeat; each caps itself at 3 with a `PrerequisiteCondition` on its own rank (`Blueprints/Classes/Prerequisites/PrerequisiteCondition.cs`, `UnitLogic/Mechanics/Conditions/ContextConditionBuffRank.cs`, WrathDecompiled 6ce4a6b). Extra Rage Power (`ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a) and Favorite Metamagic (`FavoriteMetamagicSelection` 503fb196aa222b24cb6cfdc9a284e838) have no such condition (242: medium confidence on Favorite Metamagic).
- **The parametrised and ranked Extra feats** stop at their blueprint's `Ranks`: Extra Ki (`ExtraKi` 231a2a603d0b437e939553e6da3e7247) and Extra Reservoir 20; Extra Rage, Extra Channel (all its variants), Extra Lay on Hands, Extra Performance, Extra Bombs, Extra Bloodrage and Extra Arcane Pool 10. The planner has them unlimited. No Build of 20 levels reaches these caps, so only the data changes.
- The game check doesn't compare `PrerequisiteCondition` (63 counted as not compared), so no finding stands for this.

## Change

- Each Extra feat's cap is the game's: `Times(3)` for the ten with the condition, none for Extra Rage Power and Favorite Metamagic, the blueprint's `Ranks` for the others. The planner's form for a cap is read first; a new one only if none fits.
- A data note records each cap with its blueprint.

## Tests 242 marks as waiting on this ticket

- `parameterised_feats.rs`: `feats_repeat_and_take_values_as_the_guide_says` (Extra Ki `Times(20)`, the ten-rank feats, and the count of repeating feats that follows).

## Done when

- A real-data test asserts each cap with its blueprint and GUID, and that a fourth Extra Hex is refused.
- The scoped checks pass.
