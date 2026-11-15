# Inquisitor rows

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: the judgement's uses and the archetypes' rows read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `archetype_losses.ron`) differ on `inquisitor.ron`:
- **Judgment** (`JudgmentFeature` 981def910b98200499c0c8f85a78bde8) is a feature, not a choice: the planner's `inquisitor/judgment` selection names it. Its uses at 4, 7, 10, 13, 16, 19 are `JudgmentAdditionalUse` (ee50875819478774b8968701893b52f5), for the class and each archetype that keeps judgement (`InquisitorProgression` 4e945c2fe5e252f4ea61eee7fb560017); the Faith Hunter's own judgement and smite uses (`FaithHunterSwornEnemySmiteExtraUse` 2fab74210ab73ba47b8bad213becce4f).
- **Detect Magic** at 1 for the class and its archetypes.
- **Archetypes:** the Living Grimoire (`LivingGrimoireArchetype` b68b52e5e802426d989a3d4f8c986761) removes Cunning Initiative (`CunningInitiative` 6be8b4031d8b9fc4f879b72b5428f1e0) at 2, and its Sacred Word's `Plus2` to `Plus5` and Blessed Script are the game's steps; the Tactical Leader (`TacticalLeaderArchetype` 639b74fd2f48d474e965c596b1649095) keeps Exploit Weakness (`ExploitWeakness` 374a73288a36e2d4f9e54c75d2e6e573) at 14, and its Swift Tactician rows are the game's; the Sanctified Slayer's talents are `SanctifiedSlayerSlayerTalentFeature` (db3767ba22ce84942af56525cb9f9ce6) and its studied target repeats ([254](254-a-features-steps-follow-the-class-level.md)); the Monster Tactician's favoured-enemy rank-ups; the Sacred Huntsmaster's (`SacredHuntsmasterArchetype` 46eb929c8b6d7164188eb4d9bcd0a012) domain selection offers Animal and Scalykind.

## Findings (the game check, `known.txt` section 284)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / in game only | 49 |
| progression rows / a further rank in planner only | 15 |
| progression rows / in planner only | 5 |
| option lists / in game only | 2 |
| option lists / a choice the game doesn't offer | 1 |
| **all** | **72** |

The 10 entries: `archetype inquisitor/faith-hunter`, `archetype inquisitor/judge`, `archetype inquisitor/living-grimoire`, `archetype inquisitor/monster-tactician`, `archetype inquisitor/sacred-huntsmaster`, `archetype inquisitor/sanctified-slayer`, `archetype inquisitor/tactical-leader`, `class inquisitor`, `selection inquisitor/judgment`, `selection inquisitor/sacred-huntsmaster/domain`.

The report lists each finding under its group, marked `[known: 284]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows become the game's with [254](254-a-features-steps-follow-the-class-level.md)'s model; `archetype_losses.ron`'s living-grimoire and tactical-leader entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `scaling_features.rs`: `the_inquisitor_archetypes_step_the_features_the_guide_gives_steps_to`.
- `archetype_losses.rs` (living-grimoire, tactical-leader).
- `option_families.rs`: `an_inquisitors_domain_runs_on_inquisitor_levels` (re-read with the Sacred Huntsmaster's list).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 284 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
