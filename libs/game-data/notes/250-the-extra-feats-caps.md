# 250: the Extra feats' caps

**What the data says now** (`feats.ron`, each feat's `repeat`), the game's cap for each:

- **`Times(20)`, the blueprint's `Ranks`:** `extra-ki` (`ExtraKi` 231a2a603d0b437e939553e6da3e7247), `extra-reservoir`
  (`ExtraReservoir` 29afc53efa4d469b9476760fa09576de). Both were `Unlimited`.
- **`Times(10)`, the blueprint's `Ranks`:** `extra-rage` (`ExtraRage` 1a54bbbafab728348a015cf9ffcf50a7),
  `extra-channel` (`ExtraChannel` cd9f19775bd9d3343a31a065e93f0c47), `extra-lay-on-hands` (`ExtraLayOnHands`
  a2b2f20dfb4d3ed40b9198e22be82030), `extra-performance` (`ExtraPerformance` 0d3651b2cb0d89448b112e23214e744e),
  `extra-bombs` (`ExtraBombs` 54c57ce67fa1d9044b1b3edc459e05e2), `extra-bloodrage` (`ExtraBloodRage`
  7624c6e46551720459cf5292da67d2f9), `extra-arcane-pool` (`ExtraArcanePool` 42f96fc8d6c80784194262e51b0a1d25). All
  were `Unlimited`. The nine summaries say "up to 20 times" or "up to 10 times" in place of "can be taken more than once". A plain feature stops at its `Ranks` (`UnitLogic/Feature.cs` `AddRank`;
  `BlueprintFeatureSelection.CanSelect` refuses a non-selection feature held at its `Ranks`, WrathDecompiled 6ce4a6b).
- **`Times(3)`, unchanged:** the eight Extra feats that are selections, each capped by a `PrerequisiteCondition` that
  is `Not` of `ContextConditionBuffRank` on its own fact at rank 3 (`Blueprints/Classes/Prerequisites/
  PrerequisiteCondition.cs`; `ContextConditionBuffRank.CheckCondition` holds at a rank of at least the value):
  `extra-arcana` (`ExtraArcanaSelection` 00727883edf145e2a6bce9ad176ecfd8), `extra-arcanist-exploit`
  (`ExtraArcanistExploitSelection` 0a7065e13f4449fa81bfb33199bf7f6a), `extra-discovery` (`ExtraDiscoverySelection`
  537965879fc24ad3948aaffa7a1a3a66), `extra-hex` (`ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35),
  `extra-mercy` (`ExtraMercySelection` 8a49abed5be9473da1e1fd1e2457562e), `extra-rogue-talent`
  (`ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12), `extra-slayer-talent` (`ExtraSlayerTalentSelection`
  53f7237f5b1447bb851ba68045a00e41), `extra-wild-talent` (`ExtraWildTalentSelection`
  0f73730cf1b44ee882671e55d5f6e471). The game's other two of the ten (Extra Hex (Shaman), the Vivisectionist's Extra
  Discovery) are feats the planner doesn't have apart yet (264, now 293).
- **Uncapped:** `extra-rage-power` stays `Unlimited` (`ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a has
  no condition). Favorite Metamagic is a mythic ability held once per metamagic (ticket 249). `extra-bane-weapon`
  stays `Once` (`ExtraBane` 756dc2f7340b0b34285a1dc367ff7359, `Ranks` 1).

A Build meets a cap of 10 only by spending ten feats on one of them (an inference: the snapshot and corpus tests show
no Build that does). The game check doesn't compare `repeat` or
`PrerequisiteCondition`, so no finding stood for this.

**Tests.** `parameterised_feats.rs`: `the_extra_feats_repeat_up_to_their_blueprints_ranks`,
`the_extra_talent_feats_stop_at_3_and_extra_rage_power_doesnt` (a fourth Extra Hex is refused).
