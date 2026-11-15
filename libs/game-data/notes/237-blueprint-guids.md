# Ticket 237: a blueprint GUID on every game-data entry

## What the data says now

Every entry of the 18 kinds that can have a game blueprint (races, subraces, weapons, feats, feat lists, classes and prestige classes, archetypes, features, selections, selection options, mythic abilities, feats and paths, backgrounds, deities, gear, spells, spell lists) holds a `blueprint: "<32 hex digits>"` right after its `key` when the game has one blueprint for it: 4,309 entries. Skills hold none (the game's skills are a stat, not a blueprint). 175 entries of the 18 kinds are left empty (the 11 skills have no field), listed with their reasons in ticket 237's Answer; `GameData::blueprints` walks them all.

The field is a citation and the key of the game check (ticket 238). It is never serialised to the browser, and no rule reads it.

## What it rests on

The game's blueprints, BubblePrintsData `2.7.0x_0`. Each GUID was matched by a script kept outside the repo, then reviewed by hand: an entry granted at a level is confirmed by the same level of its granter's `LevelEntries` or `AddFeatures`; an option by its selection's `m_AllFeatures`; a feat, spell, item and the like by a unique name in its kind. Where names matched but structure did not, the structure won. A grant through a component counts as a granter too: the Razmiri mask's eighth step is `RazmiryInfiltratorMaskFeature9`, which `RazmiryInfiltratorMaskFeature` grants by `AddFeatureOnClassLevel(ArcanistClass, 8)`; each Dragonblood colour's breath is `DragonbloodShifter<Colour>BreathWeaponFeature`, which `DragonbloodShifter<Colour>Buff` grants by `AddFeatureOnClassLevel(ShifterClass, 10)`; Change Shape II is `TransmutationSchoolChangeShapeFeature`, which `TransmutationSchoolChangeShapeFeatureAdd` grants by `AddFeatureOnClassLevel(WizardClass, 12)`.

## Decisions

- The main session, 2026-11-14: the field is `BlueprintGuid` beside the key newtypes, `Option`, after `key`, on the 18 kinds; `GameData::check` refuses anything but 32 lowercase hex digits (`MalformedBlueprint`).
- The user's direction via the main session, 2026-11-15: any number of entries may hold one GUID for now. There is no sharing check and no list of allowed sharings; the planner's rank and context splits get their own ticket (`.scratch/wotr-planner/issues/243-ranked-features-and-filtered-pools.md`), and the copies of one game object feed a later design ticket on laying the data out by blueprint.
- The main session, 2026-11-14: an entry the game makes of several blueprints names one, the primary; the others are below for 238.
- Ours (inference, each marked so in the Answer): a mythic summon tier is its rank (`AeonSummonFeatureTier7` is the gold golems); Intense Spells and Summoner's Charm are carried by the school's own feature; the nature mystery's companion revelation is `OracleRevelationBondedMount`.

## One entry, several blueprints (the primary first)

- `alchemist/incense-synthesizer/incense-fog` (a selection): `IncenseSynthesizerIncenseFogSelection`, whose options are the ten incenses; `IncenseFogFeature`, the fog itself, which `IncenseSynthesizerArchetype` adds at 1.
- `cleric/domain`: `DomainsSelection`; the game's second pick is `SecondDomainsSelection`, the separatist's `SecondDomainsSeparatistSelection`.
- `warpriest/blessings`: `BlessingSelection`; the second pick is `SecondBlessingSelection`.
- `aeon/enforcing-gaze`: `AeonGazeFirstSelection`; then `AeonGazeSecondSelection` and `AeonGazeThirdSelection`.
- `magical-tail` (feat): `MagicalTail1`; `MagicalTail2` to `8`, one per pick.
- `advanced-weapon-training`: `AdvancedWeaponTraining1`; `2` to `4`.
- `extra-channel`: `ExtraChannel` (the cleric's); the game has one per channelling class. `extra-hex`: `ExtraWitchHexSelection`; the shaman's is another.
- `abundant-smite-smite-chaos`: `AbundantSmite`; `AbundantSmiteChaos` too.
- `magus/spell-dancer/spell-dance-ii`: `SpellDanceBlurFeature`; `SpellDanceHasteFeature` at the same level.
- `gold-dragon/perfect-mind`: `DragonLevel2Immunities`; nine more rank-2 features (the stat and BAB overrides, `GoldenDragonSharedSkillUMD`).
- The combined wild shapes: the first animal named (`WildShapeIIISmilodonFeature`, `WildShapeIVBearFeature`, `WildShapeIILeopardFeature`); the elemental and plant forms at the same level, and `WildShapeExtraUse`.
- `bloodrager/primalist/primal-choices` and its options: the 4th-level copies (`PrimalistLevel4Selection`, `PrimalistTakeBloodlinePower4`, `PrimalistTakeRagePowers4`); copies at 8, 12, 16 and 20.
- `paladin/warrior-of-the-holy-light/power-of-faith`: `PowerOfFaithTier1MainFeature`; tiers 2 to 5.
- `monk/sohei/weapon-training`: `SoheiWeaponTrainingSelection`; `SoheiWeaponRankUpTrainingSelection`.
- `aeon/aeon-immunities`: `AeonThirdLevelImmunities` (then fifth, seventh, tenth); `aeon/enforcing-gaze-extra-gaze`: `AeonGazeFeatureExtraGaze` (and `2`); `aeon/mythic-suppression`: `AeonDemythication` (upgrades 1 and 2).
- `druid/elemental-rampager/claws`: `ElementalRampagerClaws1d6`; `1d8` and `1d10`.
- `warpriest/shieldbearer/sacred-shield`: `SacredArmorShieldFeature`; `SacredArmorShieldEnchantPlus2` to `5`.
- `wizard/school/diviner/forewarned`: `DivinationSchoolBaseInitiativeFeature`; the 20th-level `DivinationSchoolForewarnedFeature`. `wizard/school/transmuter/physical-enhancement`: `TransmutationPhysicalEnhancementFeature`; the capstone `TransmutationPhysicalEnhancementCapstone`. `wizard/school/invoker/intense-spells`: `SpecializationSchoolEvocation`; `IntenseSpellsPenetration` is its own entry.
- `alchemist/metamorph/shapechanger-beast-shape-i`: `MetamorphBeastShapeIFeature`, with `MetamorphShapechangerFeature`; `…-iv-dragonkind-i`: `MetamorphBeastShapeIVFeature`, with `MetamorphFormOfTheDragonIFeature`.
- The kineticist's element feat lists: the unnumbered `WildTalentBonusFeat<Element>`; the numbered copies.
- Weapons: `dwarven-urgrosh` is `DwarvenUrgrosh` (and `DwarvenUrgroshSpearHead`), `gnome-hooked-hammer` `GnomeHookedHammerHead` (and `…Hook`); a natural weapon is its category's base type (`BiteType`), which the per-weapon feats name.
- Spells: `crusaders-edge` is `CrusadersEdgeCast` (on the lists; `CrusadersEdge` is its effect); `dragon-storm` `DragonUltimateApsu` (the corrupted list's is `DragonUltimateDahak`).

## Differences for 238

- `alchemist/incense-synthesizer/incense-fog`: the planner offers a pick at 1; the game adds the selection first at 3 (then 6, 9, 12, 15, 18).
- The Dragonblood breath: the game grants it at shifter 10 under the colour's form (`DragonbloodShifter<Colour>Buff`); the planner offers it as a Greater Wyrmshifter option at 20.
- `stalwart-defender/defensive-powers/clear-mind`, `immobile` and `mighty-resilience` are not among `StalwartDefenderDefensivePowerSelection`'s options (`clear-mind` names `ClearMindFeature`, the barbarian's rage power; the other two are empty).

## Not modelled

No sharing rule, and nothing reads the GUID yet: ticket 238 compares each entry with its blueprint.

## Tests

`libs/seeds/tests/game_data/blueprints.rs` (spot GUIDs cited by blueprint name, the empty count per kind); `libs/rules/tests/engine/check.rs` (`a_blueprint_is_32_lowercase_hex_digits`).
