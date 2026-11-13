# The base classes' stepped features are granted once

Type: design
Status: resolved
Blocked by: none
Model: the user decided; Sonnet to build (data against the guide, as tickets 177 and 182), Opus to review the data and the archetype table.
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 182](182-still-more-scaling-features.md) left this open as a question for the user ("The user decides whether the sheet should show these steps at all … before any is built"). A QA session through the MCP server (2026-11-13, rerun on `main` at `ce9cc3b`) shows what leaving it open costs: tickets 170, 177 and 182 stepped the archetypes' features, so where an archetype replaces a base feature, `compare` reports the archetype gaining a step at each level and the base class gaining nothing there.

| Archetype | Its stepped replacement | The base feature it replaces, granted once |
| --- | --- | --- |
| Hospitaler (paladin) | Smite Evil (Hospitaler) at 1, 7, 13, 19 | Smite Evil at 1 (the summary: "+1 per 3 levels") |
| Two-Handed Fighter | Strong Grip at 2, 6, 10, 14, 18; Two-Handed Weapon Training at 5, 9, 13, 17 | Bravery at 2, Armor Training at 3, Weapon Training at 5 |
| Invulnerable Rager (barbarian) | Extreme Endurance at 3, 6, 9, 12, 15, 18 | Danger Sense at 3 |

The rogue's Danger Sense is the same (at 3 only; its summary: "+1 every 3 levels (max +6)"). The base features already stepped (the rogue's Sneak Attack, the ranger's Favored Enemy, the barbarian's Damage Reduction, the slayer's Studied Target) line up with their replacements, so the sheet is inconsistent between features of the same shape.

182's Answer lists the base features found and left: Danger Sense (barbarian and rogue), the cavalier's Banner, the druid's and the warpriest's Wild Shape (Wolf) uses, Channel Energy, the paladin's and the ranger's stepped features, the fighter's Bravery, Weapon Training and Armor Training, and the archetypes' copies of them (Wild Shape (Wolf) in four druid archetypes and the Feral Champion). Its notes stand: the Dragonheir Scion's table has no Bravery, the Defender of the Hearth's Weapon Training at 9 is its delayed first grant, and Advanced Weapon Training reads Weapon Training with `count: 1`.

Options:
- **A.** Step the base features, each at the levels its guide sentence gives, with 182's rules: every archetype that loses, replaces or delays one gets its own rows (`assert_gains` builds every archetype of a class), and what reads each count is checked first. The sheet and `compare` then agree feature for feature.
- **B.** Step none, and unstep the archetypes' replacements of base features, so both sides show one grant. Undoes part of 170, 177 and 182.
- **C.** Leave it. The `wotr-build` skill then says that `compare` reports an archetype's stepped replacement as a gain at each step even where the base class gains the same.

**Recommendation: A.** The archetype side is built and reviewed; B throws that away, and C leaves `compare` misleading for every archetype that replaces a stepped base feature.

## Done when

The user has chosen. For A: each base feature stepped as its guide sentence says or struck with the reason, the archetype table in the Answer, the snapshot diff showing only the new grants, and `scripts/check.sh` passing.

## Answer

Done 2026-11-13 (branch `ticket/188-base-classes-stepped-features`). Data and tests only: no engine, schema or codec change.

**The user's decision** (2026-11-13, "do as the recommended for all questions"): option A in full; the cavalier's Tactician and Challenge now, with Banner; the four archetype inferences as recommended (below); the second tier is [ticket 194](194-more-base-class-stepped-features.md).

### What was stepped (the guide's sentence, on the owner's own level scale)

| Feature | Steps at | The page and heading |
| --- | --- | --- |
| Barbarian Danger Sense | 3, 6, 9, 12, 15, 18 | `barbarian`, Danger Sense: +1 every 3 levels, +6 at 18 |
| Rogue Danger Sense | 3, 6, 9, 12, 15, 18 | `rogue`, Danger Sense: the same |
| Fighter Bravery | 2, 6, 10, 14, 18 | `fighter`, Bravery: +1 every 4 levels, +5 at 18 |
| Fighter Armor Training | 3, 7, 11, 15 | Armor Training: every 4 levels, -4/+4 at 15 |
| Fighter Weapon Training | 5, 9, 13, 17 | Weapon Training: a new group every 4 levels up to 17 |
| Armiger, Ardent | 2, 6, 10, 14, 18 | `fighter`, Ardent: +1 every 4 levels, +5 at 18 |
| Aldori Defender, Defensive Parry | 3, 7, 11, 15, 19 | Defensive Parry: +1 every 4 levels after 3, +5 at 19 |
| Cavalier Banner | 5, 10, 15, 20 | `cavalier`, Banner: +1 at 10 and every 5 levels after |
| Cavalier Tactician | 1, 5, 10, 15, 20 | Tactician: an extra use at 5 and every 5 levels, 5 at 20 |
| Cavalier Challenge | 1, 4, 7, 10, 13, 16, 19 | Challenge: an extra use every 3 levels beyond 1st, 7 at 19 |
| Paladin Smite Evil | 1, 4, 7, 10, 13, 16, 19 | `paladin`, Smite Evil: another use at 4 and every 3 levels, 7 at 19 |
| Druid Wild Shape (Wolf) and each archetype's copy | 4, 6, 8, ..., 18 | `druid`, Wild Shape: an extra use every 2 levels above 4, 8 at 18 |
| Feral Champion Wild Shape (Wolf) | 7, 11, 15, 19 | `warpriest`, Wild Shape: one more every 4 levels, 4 at 19 |
| Warpriest Channel Energy | 4, 5, 8, 11, 14, 17, 20 | `warpriest`, Channel Energy: 1d6 plus 1d6 per 3 levels above 2nd, 7d6 at 20 |
| Warpriest Fervor | 2, 5, 8, 11, 14, 17, 20 | Fervor: the same per 3 more levels, 7d6 at 20 |

The class tables list each of these once, at its first level, so table and sentence do not conflict; the sentence gives the steps.

**The four inferences** (the guide gives no levels; the user took the recommended option):
- the Defender of the Hearth's Weapon Training, first at 9 (its delayed grant), steps at 13 and 17 with the class's rows (count 3 at 17);
- the Standard Bearer's own Banner (granted at 1) at 1, 10, 15 and 20;
- the Feyspeaker's Wild Shape (Wolf), which comes two levels later, at each even level from 6 to 18 (7 grants);
- the Shieldbearer's own Channel Energy on the warpriest's steps.

**Struck, with the reason:** the cleric's Channel Energy, the paladin's Lay on Hands and Channel Positive Energy (the guide gives a formula: no named levels or cap level, the rule 170, 177 and 182 used; 11 grants for the cleric); the ranger's Favored Enemy and Favored Terrain (already stepped in the base rows: 1, 5, 10, 15, 20 and 3, 8, 13, 18).

**Where the guide's archetype table decided against the plan:** nowhere. Each archetype's keep or lose was read from the guide's own table (the archetype headings of `barbarian`, `rogue`, `fighter`, `cavalier`, `paladin`, `druid`, `warpriest`) and agreed with the data-derived plan.

**Row mechanics found while building:** a row that is the base row is refused ("leave the level out"), so the Defender of the Hearth's row at 9 (its first Weapon Training) was deleted: the class's row at 9 now carries it. A new row for an archetype that loses a feature carries what the base row gives there, minus the stepped features.

### What reads each count

Nothing above 1. The Advanced Weapon Training prerequisite (`fighter/weapon-training`, `count: 1`), Natural Spell, Raking Claws and Shifter's Rush (the seven Wild Shape (Wolf) keys, `count: 1`), Extra Channel and Selective Channel (`count: 1`), the mythic Abundant Smite (`paladin/smite-evil`, `count: 1`); `bonuses.ron` and `interactions.ron` name none of the keys; none has `ability_bonuses`; the engine's other readers ask only that a feature is held (a count above 0). No number or pick moves. `class_rules.rs`'s Titan test (no Weapon Training at any level) passes with the Titan's new row at 13.

### The archetype table (what the data gives now)

Levels are where the archetype holds the feature at the end of its level 20 Build (`assert_gains`). "Loses" means none of the class's steps.

**Barbarian Danger Sense.** The class and Beastskin Berserker, Instinctual Warrior, Mad Dog, Pack Rager: keep, 3, 6, 9, 12, 15, 18. Armored Hulk, Flesheater, Invulnerable Rager: lose. New rows: Flesheater 12 (its rage power) and 15.

**Rogue Danger Sense.** The class and Dark Lurker, Eldritch Scoundrel, Master of All, Rowdy, Sylvan Trickster, Underground Chemist: keep, 3, 6, 9, 12, 15, 18. Knife Master (Blade Sense) and Thug (Brutal Beating): lose. New rows: Thug 6, 12, 18 (rogue talents) and 9, 15 (Sneak Attack).

**Fighter Bravery** (2, 6, 10, 14, 18): the class and Aldori Defender, Defender of the Hearth, Mutation Warrior, Titan Fighter keep. Armiger (Ardent), Dragonheir Scion (Fearful Might), Tower Shield Specialist (Burst Barrier), Two-Handed Fighter (Strong Grip) lose. New rows: Armiger 6, 14, 18 (combat feat slot, Ardent).

**Fighter Armor Training** (3, 7, 11, 15): the class and Armiger, Defender of the Hearth, Tower Shield Specialist keep. Aldori Defender, Dragonheir Scion, Mutation Warrior, Titan Fighter, Two-Handed Fighter lose. New rows: Aldori Defender 7, 11, 15 (Defensive Parry) and 19 (Armor Mastery, Defensive Parry); Dragonheir Scion 11.

**Fighter Weapon Training** (5, 9, 13, 17): the class and Aldori Defender, Armiger, Dragonheir Scion, Mutation Warrior keep. Defender of the Hearth: 9, 13, 17 (inference). Titan Fighter (no Weapon Training, ticket 131), Tower Shield Specialist, Two-Handed Fighter lose. New rows: Titan Fighter 13, Tower Shield Specialist 17.

**Armiger's Ardent** 2, 6, 10, 14, 18 and **Aldori Defender's Defensive Parry** 3, 7, 11, 15, 19: the owner only.

**Cavalier Banner** (5, 10, 15, 20): the class and Beast Rider, Cavalier of the Paw, Fearsome Leader, Gendarme, Knight of the Wall keep. Disciple of the Pike, Ghost Rider lose; the Standard Bearer has its own (1, 10, 15, 20, inference). **Tactician** (1, 5, 10, 15, 20): the class and Beast Rider, Cavalier of the Paw, Disciple of the Pike, Standard Bearer keep; Fearsome Leader, Gendarme, Ghost Rider, Knight of the Wall lose. **Challenge** (1, 4, 7, 10, 13, 16, 19): the class and every archetype. New rows: Disciple of the Pike 10, 15; Fearsome Leader 5, 10, 20; Gendarme 5, 10; Ghost Rider 15, 20 (Supreme Charge); Knight of the Wall 5, 10, 15, 20; Standard Bearer 10.

**Paladin Smite Evil** (1, 4, 7, 10, 13, 16, 19): the class and Divine Guardian, Divine Hunter, Warrior of the Holy Light keep. Divine Scion, Hospitaler (its own, 1, 7, 13, 19), Martyr, Stonelord, Tortured Crusader lose. New rows: Divine Scion 4 (Channel Positive Energy), 13, 16, 19; Hospitaler 10, 16; Martyr 4 (Channel Positive Energy), 19; Stonelord 7, 13, 19; Tortured Crusader 7, 10, 13, 19.

**Druid Wild Shape (Wolf)**: the class 4, 6, ..., 18. No archetype holds the class's key: Drovier and Winter Child have no Wild Shape; Blight Druid, Defender of the True World, Elemental Rampager and Primal Druid hold their own copy at 4, 6, ..., 18; Feyspeaker's own at 6, 8, ..., 18. New rows: Blight Druid 14, 16, 18; Defender 14, 16; Elemental Rampager 14, 18; Primal Druid 14, 18; Drovier 16, 18; Winter Child 14, 18.

**Warpriest Channel Energy** (4, 5, 8, 11, 14, 17, 20): the class and Feral Champion keep. Champion of the Faith, Cult Leader, Disenchanter, Mantis Zealot, Proclaimer replace it; the Shieldbearer's own at the same levels (inference). **Fervor** (2, 5, 8, 11, 14, 17, 20): the class and every archetype. **Feral Champion Wild Shape (Wolf)** 7, 11, 15, 19. New rows: Champion 5, 11, 14, 17; Cult Leader 5, 8 (Sacred Weapon), 11, 14, 17, 20 (Aspect of War, Sacred Weapon); Disenchanter 5, 11, 14, 17; Mantis Zealot 5, 14, 17; Shieldbearer 5, 11, 14, 17; Feral Champion 15 (combat feat).

### Old against new

Before editing, every class and every archetype at level 20 (222 characters, made possible with the first race and alignment that allow each, projected as the snapshot check does, with every slot's open options) was dumped; again after. **346 lines differ, all added `level.N.feature/<key>` lines, none removed or changed** (no feat, feat slot, selection, open option, spell level or label moved): barbarian Danger Sense 25, rogue Danger Sense 35, fighter Bravery 20, Armor Training 12, Weapon Training 17, Ardent 4, Defensive Parry 4, Banner 18, Tactician 20, Challenge 54, the Standard Bearer's Banner 3, Smite Evil 24, the class's Wild Shape (Wolf) 7 and the archetypes' copies 7, 7, 7, 6, 7 (Blight Druid, Defender, Rampager, Feyspeaker, Primal Druid), Channel Energy 12, the Shieldbearer's 6, Fervor 48, the Feral Champion's 3.

### Snapshot check

Before regenerating (`SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`), 21 of 44 Builds differ, 189 added lines:

```text
the engine's output differs from the snapshots in 21 of 44 Builds

The same, levels and ranks wildcarded:
  level.*.feature/fighter/bravery: + Bravery in 14 Builds, 56 lines
  level.*.feature/fighter/armor-training: + Armor Training (Fighter) in 14 Builds, 42 lines
  level.*.feature/fighter/weapon-training: + Weapon Training (Fighter) in 14 Builds, 42 lines
  level.*.feature/druid/wild-shape-wolf: + Wild Shape (Wolf) (Druid) in 1 Build, 7 lines
  level.*.feature/cavalier/challenge: + Challenge in 1 Build, 6 lines
  level.*.feature/paladin/smite-evil: + Smite Evil in 1 Build, 6 lines
  level.*.feature/rogue/danger-sense: + Danger Sense (Rogue) in 2 Builds, 6 lines
  level.*.feature/warpriest/channel-energy: + Channel Energy (Warpriest) in 1 Build, 6 lines
  level.*.feature/warpriest/fervor: + Fervor in 1 Build, 6 lines
  level.*.feature/barbarian/danger-sense: + Danger Sense (Barbarian) in 1 Build, 5 lines
  level.*.feature/cavalier/tactician: + Tactician (Cavalier) in 1 Build, 4 lines
  level.*.feature/cavalier/banner: + Banner in 1 Build, 3 lines
```

Every changed line is an added grant of a feature in scope. The fighter's 14 Builds are `class-fighter`, `bench-fighter20`, `bench-fighter20_nopath`, `bench-legend40` and the ten `path-*`. The gold builds pass with no known gap added.

### Tests (`scaling_features.rs`)

`the_barbarians_danger_sense_steps_every_three_levels`, `the_rogues_danger_sense_steps_every_three_levels`, `the_fighters_bravery_armor_training_and_weapon_training_step`, `the_armigers_ardent_and_the_aldoris_defensive_parry_step`, `the_cavaliers_banner_tactician_and_challenge_step`, `the_standard_bearers_own_banner_steps_at_10_15_and_20`, `the_paladins_smite_evil_steps_every_three_levels_after_4`, `the_druids_wild_shape_wolf_steps_every_two_levels_and_in_each_archetypes_own_copy`, `the_warpriests_channel_energy_and_fervor_step`, `the_feral_champions_wild_shape_wolf_steps_every_four_levels`. Each builds the class and every archetype (`assert_gains`), so a leak shows.

### Not in this ticket

The second tier (bard Inspire Courage and Inspire Competence, alchemist Poison Resistance, monk AC Bonus and Fast Movement, kineticist Infusion Specialization, shifter Defensive Instinct, skald Inspired Rage, Slayer's Advance, the paladin's Divine Weapon Bond, All Is Darkness and Stonestrike) is [ticket 194](194-more-base-class-stepped-features.md).

### `scripts/check.sh` (after `git merge main`, which brought 192's note; `cargo fmt --all --check` passes)

```text
fmt OK (1s)
clippy OK (7s)
wasm-clippy OK (4s)
test OK (190s)
rules-wasm OK (4s)
data-check OK (4s)
ALL-OK
```

### Simplify

Run on the whole diff (data rows and ten tests): nothing to change. The tests use the existing `assert_gains`; the rows hold nothing derivable. The temporary dump test and the visibility changes it needed were removed before the commit.
