# The Extra feats' caps

Type: build (engine rule and data)
Status: resolved
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

## Answer

**Built** (2026-11-15), data only: the planner's form for a cap, `Repeat::Times(n)`, fits each (no new form). Data note: `libs/game-data/notes/250-the-extra-feats-caps.md`, which lists each cap with its blueprint.

- **`Times(20)`**, the blueprint's `Ranks`: Extra Ki (`ExtraKi` 231a2a603d0b437e939553e6da3e7247), Extra Reservoir (`ExtraReservoir` 29afc53efa4d469b9476760fa09576de); were `Unlimited`.
- **`Times(10)`**, the blueprint's `Ranks`: Extra Rage (`ExtraRage` 1a54bbbafab728348a015cf9ffcf50a7), Extra Channel (`ExtraChannel` cd9f19775bd9d3343a31a065e93f0c47), Extra Lay on Hands (`ExtraLayOnHands` a2b2f20dfb4d3ed40b9198e22be82030), Extra Performance (`ExtraPerformance` 0d3651b2cb0d89448b112e23214e744e), Extra Bombs (`ExtraBombs` 54c57ce67fa1d9044b1b3edc459e05e2), Extra Bloodrage (`ExtraBloodRage` 7624c6e46551720459cf5292da67d2f9), Extra Arcane Pool (`ExtraArcanePool` 42f96fc8d6c80784194262e51b0a1d25); were `Unlimited`. A feature that isn't a selection stops at its `Ranks` (`BlueprintFeatureSelection.CanSelect`, WrathDecompiled 6ce4a6b). Extra Channel's variants are one planner feat until 244's pool entries.
- **`Times(3)`, already so:** the planner's eight Extra feats that are selections (Extra Arcana, Arcanist Exploit, Discovery, Hex, Mercy, Rogue Talent, Slayer Talent, Wild Talent), each capped by a `PrerequisiteCondition` that is `Not` of `ContextConditionBuffRank` on its own fact at rank 3 (`ContextConditionBuffRank.CheckCondition`: the fact's rank at least the value). The game's other two of the ten (Extra Hex (Shaman), the Vivisectionist's Extra Discovery) wait on the split 264 planned, now 293.
- **Uncapped:** Extra Rage Power stays `Unlimited` (`ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a has no condition); Favorite Metamagic repeats once per metamagic (249). Extra Bane Weapon stays `Once` (`ExtraBane` 756dc2f7340b0b34285a1dc367ff7359, `Ranks` 1).

**Decisions.** Ours: `Times(n)` is the form; 242's "medium confidence" on Favorite Metamagic is settled by its blueprint (no `PrerequisiteCondition`, read here).

**Tests** (real data, `libs/seeds/tests/game_data/parameterised_feats.rs`): `the_extra_feats_repeat_up_to_their_blueprints_ranks` (off `#[ignore]`) and `the_extra_talent_feats_stop_at_3_and_extra_rage_power_doesnt` (each `Times(3)` with its blueprint and GUID, Extra Rage Power `Unlimited`, Extra Bane Weapon `Once`, and a witch 7 refused a fourth Extra Hex with `MaxTimes { times: 3 }`). `feats_repeat_and_take_values_as_the_guide_says`'s count of repeating feats is unchanged (35): the nine went from `Unlimited` to `Times(n)`.

## Done when

- A real-data test asserts each cap with its blueprint and GUID, and that a fourth Extra Hex is refused.
- The scoped checks pass.
