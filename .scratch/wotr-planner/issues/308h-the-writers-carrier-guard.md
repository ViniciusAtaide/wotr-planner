# Importer S4, 308h: the writer's carrier guard

Type: build (the importer, stage S4: tooling)
Status: resolved
Blocked by: 308d
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

308d's review and fix round (2026-11-17) found the prerequisites writer (`tools/game-check/src/write/prerequisites.rs`) wrote two trees that refuse characters the game allows, through a gap its narrowing guard (308a, 308g) doesn't see:

- **Defensive Study** (DefensiveStudy ec041312142ba1f48a0044521f68fc0a) names SlayerStudyTargetFeature 09bdd944…, which SanctifiedSlayerStudyTargetFeature 6d9d3fbc… (`AddFacts`) and TandemExecutionerStudyTargetFeature 3dac0ae6… (`AddFeatureOnClassLevel`) grant in the game. The planner's two studied-target features don't grant `studied-target`.
- **The Stalwart Defender** (StalwartDefenderClass d5917881…) needs Endurance, which UnstoppableDwarf 109dca82… grants through `AddFacts`. The planner's subrace doesn't. This one was an *added requirement*, which the guard doesn't count as a narrowing, so a later `--only` naming the class writes it again.

Both are 307's missing carriers. Until 307 lands, an entry whose game tree requires a fact some game blueprint grants through a component, where the planner's entry for that granting blueprint doesn't grant the fact, must stay unwritten.

## Change

- **The carrier guard.** For each fact a new tree requires (in `Has`, in an `All` or an `Any`), find the game blueprints that grant its blueprint through a component (`AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureOnApply`, progression level entries, and the kinds `game-check granted-by` follows). For each such grantor a planner entry maps, if that entry doesn't grant the fact (its rows, its components), the entry is a gap: `<fact> granted by <grantor> <GUID>, which <planner entry> doesn't grant -> 307`, left whole. Unmapped grantors are the existing gap path.
- **An audit of what's written.** A mode (or a dry-run report) that runs the guard over every entry as the data holds it, so the entries 308c and 308d already wrote are checked too. Each hit names the entry, the fact and the carrier.
- Tests for both carriers above as fixture cases, and one where the planner's grantor does grant the fact (written).

## Done when

The audit lists Defensive Study and the Stalwart Defender (already reverted, so as "left whole") and every other hit; each hit in an entry 308c or 308d wrote is reported, then reverted to its base tree or confirmed toward the game in the Answer with evidence; a dry run over 308e's and 308f's entries shows which the guard now leaves whole; `cargo test -p game-check` passes, the game check exits 0.

## Answer (2026-11-17)

**Built** (`tools/game-check/`; no data changed):

- **The guard**, `src/write/carrier_guard.rs`. For each fact a tree requires (in an `All`, an `Any` or an `AtLeast`; not under a `Not`), it climbs the game's component grants up to each blueprint an entry maps, past those none does. For each entry of such a grantor that a character holds, it asks whether the entry gives the fact (an entry of its blueprint). If not, the gap reads `<fact> granted by <grantor> <GUID>, which <entry> doesn't grant -> 307`. The prerequisites writer's `plan` runs it on every rewrite. Any carrier gap leaves the entry whole, whatever `prerequisite-narrowing.txt` allows. The rewrite's narrowings still count toward `stale`, so no allowlist line goes stale because of the guard.
- **One walk, two users.** `granted-by`'s breadth-first climb is now `lookup::chains_up`, generic over the node type. `Library::grant_chains` uses it with every reference and stops at the granting kinds. The guard uses it with the carriers' grant vocabulary and stops at a blueprint an entry maps. The grant edges are `carriers::granters`, which the carriers area's `candidates` now builds through too, unchanged.
- **The audit**, `prerequisites::audit` and `game-check --carrier-audit [--data] [--binz]`. Each line is one hit: `<entry> | holds: <gap>` for the tree the data holds (a hand prerequisite, or the compiled `Requires`), and `<entry> | left whole: <gap>` for the game's tree that the writer now refuses. The last line is the count. It exits 0.

**Decisions (mine):**

1. **Grant edges are component grants, not every reference.** The edges are `AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`, a progression's `LevelEntries` and an archetype's `AddFeatures`: 295's vocabulary, as `carriers::grants` reads it. `granted-by` also climbs selection lists, but a listed blueprint is a pick, and the planner's selections already model picks. The walk is `granted-by`'s; only its edges differ.
2. **A class's own progression reaches no class.** A class names its progression with `m_Progression`, which grants nothing. Its rows belong to the progression rows area, as in the carriers area (an inference, as there).
3. **What "gives the fact" covers.** Three findings from the first audit (292 hits, 176 after this) led to the last three rules below:
   - **Own rows and grants.** The entry's own rows or grant components give it. That means a class's or archetype's level rows; an option's feats, chosen feat and rows; a mythic ability's features and selection; a path's ranks; a background's feats; and `Grant`, `GrantAtClassLevel`, `GrantIf`, `GrantOnApply` and `GrantWithValue`. This covers Dodge from Fluid Nature: the Duelist has no hit, because `oracle/revelations/waves/fluid-nature` gives `dodge` in its level-5 row.
   - **Rows that give both.** No selection offers the entry as a pick, and every planner row or grant that gives the entry gives the fact too. The rogue's class row gives `rogue/rogue-proficiencies` and Armor Proficiency (Light Armor) together at level 1, where the game's RogueProficiencies 33e2a7e4ad9daa54eaf808e1483bb43c gives the second through `AddFacts`.
   - **A deity's favoured weapons.** A deity gives its favoured weapons' feats, as the engine holds them (`libs/rules/src/engine/catalog.rs` `favoured_weapon_feats`). IroriFeature 23a77a5985de08349820429ce1b5a234 grants ImprovedUnarmedStrike through `AddFeatureOnClassLevel`, which the engine holds for Irori's favoured unarmed strike.
   - **An alternative in the same `Any`.** The tree accepts the grantor's entry beside the fact in one `Any`. This is Defensive Study's base shape.
4. **The owner is always 307.**

**The audit over all the data: 176 hits.** 132 are `holds` and 44 are `left whole`.

- **Left whole, 44 lines in 41 entries:**
  - **Defensive Study:** TandemExecutionerStudyTargetFeature 3dac0ae65dec4b619c035df07751b59d and SanctifiedSlayerStudyTargetFeature 6d9d3fbc30564e64d966dba27cd6357a.
  - **The Stalwart Defender:** Endurance from UnstoppableDwarf 109dca82e31a4b1093c501f8914ca1a8. It also lists light armour from RogueProficiencies and medium armour from ClericProficiencies 8c971173613282844888dc20d572cfc9. Those two come from planner rows the game removes: EldritchScoundrelArchetype and HeraldCallerArchetype each list the feature under `RemoveFeatures` at level 1, and their rows are known under 276 and 267.
  - **Favored Enemy Spellcasting:** SableMarineArchetype 475117e837fa461589005a5339f653a0.
  - **13 bomb discoveries:** ExplosiveExpert 8a551fcca93446f0adfbe5e57e5bc636 grants `bomb`.
  - **11 rogue talents:** TaintedSneakAttackFeature e6ce101a94ac9034b8b55c546e74b9dd grants `sneak-attack`.
  - **The shaman's Battle Master hex:** WildlandShamanArchetype bb14b164c2ce4e2bb05434a3218ff73d.
  - **13 witch hexes:** HexcrafterArchetype 79ccf7a306a5d5547bebd97299f6fc89 grants `major-hexes` and `grand-hexes`.
- **Holds, in the entries 308c and 308d wrote: 17 lines in 13 entries.** None is reverted. In each, the tree before 308c (`73bafe1d`) or before 308d (`a165d7a2`) already required the fact, so the write refuses no holder of the grantor that the base didn't:
  - **Trip, from ShamanHexStoneStabilityFeature 05b6b99b0f394284899eccd4728031b7.** Agile Maneuvers' base `Any` already named Trip; 308c only added Disarm, a widening. Greater Trip's and Pummeling Bully's bases both had `Has(trip)`.
  - **Dazzling Display, from ShamanHexIntimidatingDisplayFeature 2eacbef4be7c6434e9cfaa9632fdc429.** Shatter Defenses' base had `Has(dazzling-display)`.
  - **Medium armour, from ClericProficiencies (the Herald Caller's row).** Armor Proficiency (Heavy Armor)'s base had `Has(armor-proficiency-medium-armor)`. HeraldCallerArchetype removes ClericProficiencies at level 1, so no Herald Caller holds it in the game.
  - **Light armour, from RogueProficiencies (the Eldritch Scoundrel's row).** Armor Proficiency (Medium Armor)'s base had `Has(armor-proficiency-light-armor)`. EldritchScoundrelArchetype removes RogueProficiencies at level 1.
  - **Shield proficiency, from ClericProficiencies and CultLeaderWeaponProficiencies c12afa02355cd8142981980d37a77556.** These hits are on Bashing Finish, Shield Master, Greater Shield Focus and Tower Shield Proficiency. The Herald Caller's grantor is the removed one above. The Cult Leader's is real: its `AddFacts` names ShieldsProficiency cb8686e7357a68c42bdd9d4e65334633, which the planner's row lacks. That gap is 307's; the Cult Leader's rows are known under 268. 308c's added `Proficient([Shield])` refuses no Cult Leader the base didn't:
    - Tower Shield Proficiency's base had `Has(shield-proficiency)`.
    - Shield Master and Bashing Finish need Shield Bash, and Greater Shield Focus needs Shield Focus. Both of those feats require `Has(shield-proficiency)`, before and after.
    - An inference: no planner entry grants Shield Bash or Shield Focus outright.
  - **`bomb`, from ExplosiveExpert.** Extra Bombs' base had `Has(bomb)`; 308c widened it to `AtLeast(1, bomb, arcane bombs)`.
  - **The rage power selection, from ShifterWolverineProgression 4fc5f64d13ce4c0e8258a83ff63c50af.** Its `LevelEntries` give RagePowerSelection twice at level 8. The planner's `shifter/shifter-aspect/wolverine-aspect` has no rows, so a wolverine shifter holds no rage power either way, and Extra Rage Power's base `Any` of rage powers refused it too.
  - **Heavy armour, from PurifierCelestialArmorFeature 7dc8d7dede2704640956f7bc4102760a (`AddFeatureOnClassLevel` 11).** The Hellknight's (308d) base had `Has(armor-proficiency-heavy-armor)`.
- **No revert**, so `306-refused.txt`, `known.txt` and the allowlist are unchanged, and `cargo test -p seeds -p corpus` wasn't run (no data diff).
- **The other 115 `holds` lines** are in entries 308c and 308d didn't write. They are 307's carriers, or a slice's to meet when it writes them:
  - the bloodline selections' `AddFeatureIfHasFact` (EldritchScionBloodlineSelection 94c29f69cdc34594a6a4677441ed7375 and BloodOfDragonsSelection da48f9d7f697ae44ca891bfc50727988, on the Seeker's and Sage's bloodlines: 306x, 316);
  - the Hexcrafter's hexes, Tainted Sneak Attack, Explosives Expert;
  - the Wind Whisperer's, Wildland Shaman's and Kinetic Knight's grants;
  - the Sacred Huntsmaster's and Sable Marine's favored enemies;
  - HeraldCallerAugmentSummoning 1ad9828c09fd28c4293c7b12845e6d20;
  - the shaman hexes' Trip and Dazzling Display.

**Dry runs (`--write prerequisites --dry-run`).** slices.md abbreviates the `--only` lists, so each family is written out as `selection option <family>/*`:

- **308e** (rogue, slayer, alchemist, magus, arcanist, ranger, hunter, paladin, monk, stalwart-defender): 59 fields to write, in 59 files. The guard alone leaves 24 whole that the writer would have written: 13 bomb discoveries and 11 rogue talents (Confounding Blades, Crippling Strike, Dispelling Attack, Emboldening Strike, the three Focusing Attacks, Petrifying Strike, Slow Reactions, Weakening Wound, Wearying Strike).
- **308f** (barbarian, witch, shaman, kineticist, aeon, hellknight, loremaster, angel, azata, demon, lich, trickster, gold-dragon): 75 fields to write, in 75 files. The guard hits 14 entries: the 13 witch hexes and Battle Master. Each is also held by another gap, so the guard leaves none whole that the writer would have written.

**Tests** (`tools/game-check/tests/fixture/prerequisites_write.rs`, on the writer's own world):

- `a_fact_an_add_facts_carrier_grants_and_its_planner_entry_doesnt_leaves_the_entry_whole`
- `a_fact_an_add_feature_on_class_level_carrier_grants_and_its_planner_entry_doesnt_leaves_the_entry_whole`
- `a_fact_each_planner_grantor_grants_is_written` (directly, and through a blueprint no entry maps)
- `a_grantor_the_same_any_accepts_is_no_gap`
- `a_grantor_every_planner_row_gives_beside_the_fact_is_no_gap`
- `a_deitys_favoured_unarmed_strike_gives_improved_unarmed_strike`
- `the_carrier_audit_lists_what_the_data_holds_and_what_the_writer_leaves_whole`

The first two and the audit's failed before the guard: the writer wrote both carried trees, and the audit was empty. The rest pin the false-positive rules. Citations: `Blueprints/Classes/Prerequisites/PrerequisiteFeature.cs` `CheckInternal`, `UnitLogic/FactLogic/AddFacts.cs` `OnActivate`, and `Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs` `Apply` (WrathDecompiled 6ce4a6b), plus the blueprints named above.

**Checks:**

- `cargo test -p game-check`: 165 passed.
- `cargo fmt --all --check`: clean.
- `scripts/test-modules.sh`: OK.
- The game check: exit 0, 3,572 differences, all known, no stale `known.txt` or allowlist line.
