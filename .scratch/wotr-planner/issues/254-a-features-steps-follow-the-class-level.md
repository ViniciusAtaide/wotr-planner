# A feature's steps follow the class level, not repeated grants

Type: design (to decide in the ticket, recommendation below), then build (engine rule)
Status: resolved
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

## Answer

**Built** (2026-11-15). The model is the recommendation above, taken by the main session under the user's instruction of 2026-11-15 to build tickets 247–291 in order.

**The model.**
- **Data.** A feature granted once may carry `steps: (on: Classes([...]) | CharacterLevel, at: [levels])` (`Feature::steps`, `FeatureSteps`, `StepLevel`): the levels where its numbers rise, on the level the game reads. A row grants a feature again only where the game does (a feature rank, 243). An extra-use feature the game grants is its own entry, as before. The field sits on the feature, so it moves with it into 244's per-blueprint files. One `steps.at` list means "some number of the feature rises here", so a feature whose several numbers rise at different levels lists every level (the class tickets will meet this shape).
- **Engine.** `Engine::feature_step(state, feature)` is the grants plus `Engine::steps_reached`: the step levels at or below that level (the classes' levels summed, or the character level), counted only while the feature is held. It is computed when asked, from the state's class levels and the catalog, so no walk hook, no state field and no cost on the walk. A row traded away after the grant takes no step with it; a grant traded away gives none; a step level passed before the grant counts from the grant on (the game reads the level whenever the feature is held). `feature_count` stays the grants (the feature rank), which prerequisites read.
- **Check.** `GameData::check` refuses steps naming an unknown class (`UnknownKey`), no class or no level (`StepsWithoutLevels`) and a level outside 1–40.
- **Sheet.** The MCP's gained features (`Gained::step`) list a feature on the level its step rises with no grant, with `step` (Infernal Resistance at bloodrager 5: step 2, at 8: step 3), as they list a rank; `step` is set only when the feature was already held before the level, so a feature granted after its step levels shows as gained, with no `step`. The web app lists no gained features, so it is unchanged.

**Decisions.**
- The main session: the recommended model (above).
- Ours: steps computed on demand by the engine rather than stored in the state (the simplify review: the stored copy needed three refresh hooks on the walk's hot path); a `StepLevel::CharacterLevel` beside classes, since the code pass found three features on the character level; a step limited to an archetype's levels (`MaxClassLevelWithArchetype`) written on the class, the feature being the archetype's own; `CountsToward::Feature`'s doc says a stepped feature rises on its steps' classes, not by reaching rows early.

**The worked example** (`classes/bloodrager.ron`, note `libs/game-data/notes/254-a-features-steps-follow-the-class-level.md`): Infernal Resistance is granted on the Infernal bloodline's row 4 only, `steps` on bloodrager at 5 and 8 (the poison-save bonus and the resistance rise apart); Abyssal Bloodrage on the Abyssal row 12 only, `steps` on bloodrager at 16 and 20. Their other holders rise at the same levels as before; the Primalist now gets the game's answer.

**Tests.**
- `scaling_features.rs` `a_primalists_rage_powers_replace_the_row_and_its_step_with_it` holds the game's answer (no `#[ignore]` on this branch): Infernal Resistance `[4, 5, 8]` with nothing traded, none traded at 4, `[4, 5, 8]` traded at 8, none traded at 4 and 8; Abyssal Bloodrage `[12, 16, 20]`, none traded at 12, `[12, 16, 20]` traded at 16 or at 20, its rank 1 and step 3 at 20. Game: `PrimalistTakeBloodlinePower4` 2140040bf367e8b4a9c6a632820becbe adds `BloodragerInfernalResistance` ac77b22f74aed034ca2b16ba6a4bfde9, which `BloodragerInfernalBloodline` 9aef64b53406f114cb43f898a3aec01e grants at 4 only; `BloodragerInfernalResistanceBuff` 66c973ccad1293345ae6c5bfeced4a7b has two `ContextRankConfig`s on `BloodragerClass` level: the resistance table rises at 8, and the second (`DamageDice`, feeding `SavingThrowBonusAgainstDescriptor`, the poison-save bonus) has the custom table {4 -> 2, 8 -> 4}, which rises at 5 (`ApplyProgression` takes the first entry whose value is at least the level) (`ContextRankConfig.GetBaseValue` ClassLevel, `ApplyProgression` Custom, WrathDecompiled 6ce4a6b). `BloodragerAbyssalBloodline` 55d5bbf4b5ae1744ab26c71be98067f9 grants `BloodragerAbyssalBloodrage` 29fc83f824111bb41829957486752c93 at 12 only (`PrimalistTakeBloodlinePower12` 57bb4dc36611c7444817c13135ec58b4 for a Primalist); `BloodragerAbyssalBloodrageBonusProperty` 415f71e5a47f4cccb3dbd10bd7a0f8f8 multiplies a `ClassLevelGetter` on `BloodragerClass` (1 at 12–15, 2 at 16–19, 3 at 20) by the feature's rank (`FactRankGetter`, `PropertySettings.ApplyProgression`). Its `gains` helper reads `Engine::feature_step`, so every other step test reads grants and steps alike, with unchanged values.
- Fixture tests, `libs/rules/tests/engine/feature_steps.rs`: `a_feature_granted_once_rises_at_its_steps_with_the_class_level`, `a_step_survives_a_traded_away_row_and_never_comes_without_the_grant` (the test world's invented Primalist trades the bloodline's row at 9 and keeps the resistances' step at sorcerer 9; traded at 3, no step), `a_step_reads_the_level_its_steps_name` (one class, the character level, two classes summed), `a_feature_granted_after_a_step_level_holds_that_step_at_once`, `steps_name_a_known_class_and_a_level_to_rise_at`.
- MCP: `gained.rs` `real_data_the_sheet_says_the_step_a_level_raises_a_feature_to` (Infernal Resistance: level 4 listed, level 5 with `step: 2`, level 8 with `step: 3`) and `real_data_a_feature_granted_after_its_step_level_is_gained_with_no_step` (the Second Bloodrager Bloodline at MR 1 grants it at 6, after its step at 5: listed with no `step`, then `step: 3` at 8), cited as above.
- The game check reports the same findings before and after and exits 0: it reads no `steps` and compares no option rows (a steps area is not built here).

**The code pass over the 17 features** (blueprints at 2.7.0x; code at WrathDecompiled 6ce4a6b; a custom table's entry is the top of its band, `ContextRankConfig.ApplyProgression` and `PropertySettings.ApplyProgression`; a class feature's ability has the character level as caster level, `RuleCalculateAbilityParams.OnTrigger`; the unit property `Level` is the character level, `UnitPropertyExtension.GetInt`). None of the 17 is granted again anywhere (no rank), though three carry `Ranks` 3.

| Feature | Steps | Citation |
| --- | --- | --- |
| Furious Summoning (Greenrager) | no step: granted at bloodrager 9, +4 always (planner 11, 20 wrong) | `GreenragerFuriousSummoningBuff` 75f8c80d609d0af4ea57f1aecec87fb5's rank is on caster level, custom (4 to 9, 6 to 11, 8 after), but `OnSpawnBuff.OnEventDidTrigger` adds it with no ability context, so its caster level is the default 1 (an inference from the code path: `BuffCollection.AddBuff`, `AbilityParams`) |
| Banner of Solace (Standard Bearer) | steps on the character level: temp HP every 2 levels, damage at 15 and 19 (granted at cavalier 11) | `StandardBearerBannerOfSolaceBuff` 0ce31d4fea05f214e852e6d5717c6206, through `StandardBearerBannerOfSolaceAbility` fc3b21de968dfc048afb1531c6eca7a7: caster level, Div2 and DelayedStartPlusDivStep (start 7, step 4) |
| Primal Size (Primal Druid) | no step: separate features at 8 and 16 | `PrimalDruidArchetype` c1c86e2997fd4257a13ef5601b5dc6dd grants `PrimalDruidEnlarge8` baacd973524742669326169bf0f22610 and `PrimalDruidEnlarge16` eb56443bad3545f2a8a0f03cd563b0df, which `EnlargePerson` c60969e7f264e6d4b84a1499fdcf9039 checks |
| Shield Block (Defender of the Hearth) | steps at fighter 5, 10, 15, 20 | `ShieldBlockEffectBuff` 4ed87a04c7864760a418f2609b3099b1: `ContextConditionCharacterClass` on `FighterClass` (`CheckCondition`) |
| Drunken Resilience (Drunken Master) | no step: DR 2 always (planner 15, 19 wrong) | `DrunkenResilienceFeature` 4db82a5a352543b3842f63e2583a85f1 adds `DamageReductionDrunkenMaster` 187ddd9028a1413fa256fdc9f46bf7d5 once (`AddFacts`); its rank (`FeatureListRanks` of itself) stays 1 |
| Bones of Stone (Student of Stone) | steps on the character level at 10 and 15 (granted at monk 7) | `BonesOfStoneAbility` 282411eed5f0fbe4eb4c1b01cc80c0a5 (the `Level` property against 10), `BonesOfStoneBuff2` 0f156dc63e8437440972d730ef86693b (caster level, custom: 5 from 15) |
| Soul of Stone (Student of Stone) | step on the character level at 16 (granted at monk 12) | `SoulOfStoneAbility` b53ee7514922ab84581063ab3bb4ed2a (the `Level` property against 16), `SoulOfStoneBuff2` e7e04a7eed1f9ba4dbf34598d86b968c |
| Touch of Flame (Flamewarden) | step at ranger 12 (granted at 4) | `FlamewardenWeaponAbility` 2d62167ffe9c12446bd1c402ea356812: ClassLevel `RangerClass` against 12 (`ContextConditionSharedValueHigher.CheckCondition`) |
| Wind Treader (Stormwalker) | step at ranger 12 (granted at 4) | `StormwalkerWeaponAbility` 9437fe797361e4e41b37655a5597f0c9, as Touch of Flame |
| Shadows (Shadow Shaman) | steps at shaman 9 and 14 (granted at 4, once) | `ShadowShamanShadowsAbility` 085b4d4d0c5747bb8b1173e914c24d20: ClassLevel `ShamanClass` against 9 and 14 |
| Caustic Ridicule (Provocateur) | step at skald 15, with the archetype (granted at 10) | `InsultEffectBuff` 96b188c4d5cd4fa08c196068edc2fd1f: `MaxClassLevelWithArchetype` [`SkaldClass`, `ProvocateurArchetype`], custom (1 from 10, 2 from 15) |
| Prompted Backstabbing (Provocateur) | no step: separate features at 8, 13, 18 | `ProvocateurArchetype` a1f5ab7eb78942f6889304acbd92e945 grants `RagingSongSneakAttack1d6Feature` bf8fa773a8654957ade9f7fe419dbd95, `2d6` 4d1febea156c48cf8655e6cb47f94a30, `3d6` 80b37123c484432086d5645575c8a15c, which `InspiredRageEffectBuff` 75b3978757908d24aaaecaf2dc209b89 checks |
| Abyssal Bloodrage | steps at bloodrager 16 and 20 (granted at 12): built here | as in Tests |
| Adaptive Physiology (Metamorph) | no step: 25% fortification always (planner 6, 18 wrong) | `AdaptivePhysiologyFeature` cec88d5f7ebda294c93b95cd7360f504: `AddFortification` times its rank (`OnTurnOn`), granted once, so rank 1 |
| Inhibit Courage (Fake Celebrity) | steps at arcanist 5, 11, 17, with the Magic Deceiver (granted at 1) | `FakeInspireCourageEffectBuff` 4ef4e115938c42ad9e2141c8d7e1bbaa: `MaxClassLevelWithArchetype` [`ArcanistClass`, `MagicDeceiver`], StartPlusDivStep (start −1, step 6) |
| Inhibit Greatness (Fake Celebrity) | step at arcanist 17 (granted at 12) | `FakeInspireGreatnessEffectBuff` ae2139ca23764c7c8bc980bd8158617b: the same rank config, into its spell-penetration penalty |
| Battle Spirit (shaman) | steps at shaman 8 and 16 (granted with the spirit) | `ShamanBattleSpiritEffectBuff` 7a76add83f3397c4eacc59ebd8b804d8: ClassLevel `ShamanClass`, OnePlusDivStep step 8 |

The pass was a research subagent's, spot-checked in the blueprints here for Furious Summoning, Drunken Resilience and Battle Spirit; the Furious Summoning verdict rests on the code path to a default caster level, an inference. The class tickets apply it: a step goes to `steps`, "no step" drops the planner's extra rows, and a separate feature is its own entry.

**Not done here.** No other class's data changes (268–287). The game check has no steps area. 242 half B (lane-tests) splits this Primalist test and adds an ignored `a_primalists_traded_power_never_steps_and_a_kept_one_steps_with_the_class_level` that waits on this ticket: on merging both, its expected values are the ones this branch's test holds, and its `#[ignore]` goes.

## Done when

- The model is recorded in the Answer, with who decided it.
- The Primalist test holds the game's answer; a fixture test shows a step computed from class level that survives a traded-away row.
- The 17 features' steps are recorded, each with its citation.
- The scoped checks pass.

**Review fixes.** (1) Infernal Resistance also rises at bloodrager 5: `BloodragerInfernalResistanceBuff` 66c973ccad1293345ae6c5bfeced4a7b has a second `ContextRankConfig` on `BloodragerClass` level (`DamageDice`, feeding `SavingThrowBonusAgainstDescriptor`, the poison-save bonus) with the custom table {4 -> 2, 8 -> 4}, and `ContextRankConfig.ApplyProgression` (WrathDecompiled 6ce4a6b) takes the first entry whose `BaseValue` is at least the level, so 2 up to level 4 and 4 from 5; the resistance's own table rises at 8. `bloodrager.ron` has `steps: at: [5, 8]`; the Primalist tests, `the_bloodragers_infernal_resistance_and_abyssal_bloodrage_step` and the MCP test hold `[4, 5, 8]` (the note says the summary's "+4 at 8th" is the guide's). (2) `new_features` sets `step` only when the feature was already held before the level, so a feature granted after its step levels is gained with no `step`; `Gained::step`'s doc says so; `real_data_a_feature_granted_after_its_step_level_is_gained_with_no_step` pins it (it failed with `step: 2` at level 6 before the fix).
