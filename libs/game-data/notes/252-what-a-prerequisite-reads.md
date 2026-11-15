# 252: what a prerequisite reads

## What the data says now

- `gold-dragon/perfect-mind` (`mythic/paths/gold-dragon.ron`, blueprint `DragonLevel2Immunities` d1f01519ffbe9144dac703b579a0c073, given at `GoldenDragonProgression` a6fbca43902c6194c947546e89af64bd's level 2):
  - `bab_at_least_mythic_rank_plus: Some(15)`, replacing `bab_floor: Some(15)`. The BAB itself is at least 15 plus the Mythic rank while the feature is held: the sheet, the attack line and every prerequisite read it (24 for a wizard 17 at MR 9).
  - `skill_ranks_at_least_mythic_rank: true` is unchanged in the file but now means totals only: each skill's total counts at least the Mythic rank for its ranks, and a skill-rank prerequisite reads the ranks spent.
  - Its summary says "BAB at least 15 + MR (the game's code; its text says 15)" and "totals counting ranks at least MR".
- No gear row changed. What changed for gear is the rule (`rules::BonusType::counts_toward_prerequisites`): an ability prerequisite counts an item's enhancement (and racial) bonus and no other.

## The game

- **BAB:** `DragonLevel2BABOverride` f4af14e9f4408d44c830d891fee8f4a5 adds an UntypedStackable `BaseAttackBonus` modifier of −(BAB − 15 − rank), at least 0; its rank is a `ContextRankConfig` on `MythicLevel`, and `RecalculateOnStatChange` on BAB reapplies it.
  - `AddContextStatBonus.OnTurnOn` adds Value × Multiplier, at least Minimal.
  - `PropertyCalculatorComponentHelper.GetPropertyValue` evaluates the property in the fact's own context, so the rank getter reads the Mythic rank.
  - `EntityFact.Reapply` takes the modifier off before working it out again, so the BAB it reads is the BAB without it.
  - `ModifiableValue.CalculatePermanentValue` counts every modifier toward BAB, and `PrerequisiteStatValue.GetStatValue` reads that.
  - So the BAB is max(BAB, 15 + MR) (WrathDecompiled 6ce4a6b). The guide and the game's own text both say "at least 15". Following the map's rule (the code's behaviour is modelled where it differs from the text), the data follows the code. This reading of the code is the builder's (ticket 252), and the code shows it through the property and rank chain above.
- **Skills:** `GoldenDragonSharedSkillUMD` 4e0b919e4fbd85142bce959fae129d1a puts `AddGoldenDragonSkillBonus` on every skill. Its `OnTurnOn` adds a modifier of max(MR − BaseValue, 0) to the total, and `PrerequisiteStatValue.GetStatValue` reads a skill's `BaseValue`, its ranks.
- **Ability scores:** `PrerequisiteStatValue.GetStatValue` reads an attribute's `PermanentValue`. `ModifiableValueAttributeStat.CalculatePermanentValue` applies only the modifiers that `Modifier.IsPermanent` accepts: a racial one, or an Enhancement one whose `ItemSource` is set. `AddStatBonusEquipment.OnTurnOn` adds an item's bonus through `AddItemModifierUnique`, with the item as its source.
- **Tomes and manuals (the ticket's open question):** a manual's ability, for example `ManualOfGainfulExercisePlus2_Ability` a8b94294f28dec848b11c2651dcbabe0, runs `ContextActionAddFeature` with a permanent feature (`ManualOfGainfulExercisePlus2_Feature` e5ed74f06e40f994da1727662f9c9169). That feature's `AddStatBonus` is `Inherent`, and `AddStatBonus.OnTurnOn` adds it as a modifier (`AddModifierUnique`), not to the base value. `IsPermanent` refuses `Inherent`, so a tome or manual counts in every number but not toward an ability prerequisite. 242's gold row ("a tome's inherent bonus too") was wrong, and its R3 was right.

## Not modelled

- The features' own ability bonuses (`ability_bonuses`) still count toward prerequisites as before. In the game, the Dragon Disciple's +2s are Racial and count. Legend's +4 (`LegendFeature` 7f99961610390044391f300c7ef5e0c8) and the Swarm's bonuses (`EyeOfTheSwarmFeature` 73c9302686b957d4dae298061eb9ffe1, `TheSwarm` 704d7f26933c3784ea28949e0c3511c0) are untyped (`Descriptor: None`) and, by the same `IsPermanent`, would not count. That is left for a follow-up ticket: this one's change is the gear's.
- A level's skill points still read the scores without gear (the user's rule, ticket 163).

## Tests

- `follow_ups.rs`: `perfect_mind_raises_the_bab_itself_to_15_plus_the_mythic_rank`, `perfect_minds_skill_floor_raises_totals_and_meets_no_skill_rank_prerequisite`.
- `gear_engine.rs`: `greater_two_weapon_fighting_at_dex_17_needs_the_belt_of_incredible_dexterity_2` (an enhancement belt counts), `a_manuals_inherent_dexterity_doesnt_meet_greater_two_weapon_fightings_dexterity`, `dravens_hats_profane_intelligence_doesnt_meet_combat_expertise_and_an_enhancement_headband_does`.
