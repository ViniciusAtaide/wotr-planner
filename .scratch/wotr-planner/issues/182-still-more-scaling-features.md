# More scaling features are still granted once, not at each step

Type: bugfix
Status: done
Blocked by: [177](177-more-scaling-features.md)
Model: Sonnet to build (data against the guide, as tickets 170 and 177); Opus to review the data and the archetype table.
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 177](177-more-scaling-features.md) stepped the features ticket 170 found. Its work turned up more of the same shape, each granted once in the data where the guide's text names steps. The review of 177 read the sentences of the first six below and the notes on what stepping each would touch; the rows of the last four were **not checked** by it. The same rules apply as in 177: the guide's sentence for each, quoted in the Answer; the steps on the owner's own level scale; every archetype or sharer that loses, replaces or delays the feature listed with what it holds at the steps (`assert_gains` in `scaling_features.rs` builds every archetype of a class); what reads each count checked first; no feature stepped without a guide sentence that gives the levels.

## Build

Each feature: the guide's sentence, then data rows at the step levels, a test, a README note, the snapshots reviewed. An archetype row replaces the base row's `features`, `feats`, `feat_slots` and `selections` in full, so a new row must also carry what the base row gives there that the archetype keeps.

- **The Herald of the Horn's Horn Call** (skald, `skald/herald-of-the-horn/horn-call`): "Sonic spell DCs +1 at 7th level, +2 at 13th and +3 at 19th". It has a row at 7; new archetype rows at 13 and 19 must carry what the skald's base rows give there if the archetype keeps it: Raging Song (Swift Action) at 13 and Damage Reduction at 19.
- **The Reformed Fiend's Hatred Against Evil** (bloodrager): steps at 5, 10, 15 and 20 (+1 attack and damage against evil while raging, +4 at 20). It has a row at 5 and 10; a new row at 15 must keep the base row's bloodline feat slot and one at 20 Mighty Bloodrage. Its Damage Reduction (Reformed Fiend) ("DR 3/evil at 7th level, +3 more at 10th and every three levels after") is the same kind: check it with this one.
- **The Flesheater's Devour Essence** ("+1d8 at 8th, 13th and 18th level") and **Power from Flesh** (9, 18; read the sentence first), barbarian. The Flesheater's rows at 8 and 18 already exist (rage powers lost at 8, kept at 18): carry what they hold.
- **The Fiendflesh Shifter's Fiendish Resilience**: the natural armour at 4, 12 and 20 and the resistance at 8 and 16 ("+1 natural armour (+1 at 4th, 12th, 20th) and fire and electricity resistance 5 (+10 at 8th, 16th)"). Two series on one feature: the count cannot say which, so decide whether it is one feature stepped at 4, 8, 12, 16, 20 or two.
- **The Fake Celebrity's Inhibit Greatness** (arcanist, 17: "-4 at 17th level"), an option row at 17 beside Inhibit Courage's.
- **The Overwhelming Soul's Mental Prowess** (kineticist, uses at 10, 14 and 18: "+1 use at 10th, 14th, 18th"), a row at each, carrying the base row's wild talent at 10, 14 and 18.
- Not checked by the review (read the sentences, then the rows): any others a sweep of the data's summaries for "every", "rising" and "+1 more at" finds.

**A separate question for the user inside this ticket: the base classes' stepped features.** The fighter's Bravery (+1 every four levels, max +5 at 18) and Armor Training, Weapon Training and the like are granted once at their first level in the class's table, as 170's features were. Stepping them would touch every archetype with a row at a step level, so the cost is far larger and each archetype that loses or delays one needs its own row. Notes from the review: the Dragonheir Scion has its own rows at 6, 10, 14 and 18 and its table has no Bravery, so stepping Bravery on the fighter's rows would have to leave it out there; the Defender of the Hearth's Weapon Training at 9 is its delayed first grant, not a second one; Advanced Weapon Training reads Weapon Training with `count: 1`. The user decides whether the sheet should show these steps at all (the level-up of the game names them for some classes and not for others) before any is built.

**Done when:** each feature is stepped as the guide says or struck with the reason, the archetype table is in the Answer, the snapshot diff shows only the new grants, and `scripts/check.sh` passes.

## Answer

Done 2026-11-13 (branch `ticket-182`). Data only: no engine, schema or migration change. Every feature below is granted again on the row of each step level, on the owner's own level scale (the archetype's class level), at the levels the guide's sentence gives. The question for the user (the base classes' stepped features) is not built and stays open below.

### The ticket's Build list

| Feature | Steps at | Guide's sentence (`guide-cache/`, the saved page named) |
| --- | --- | --- |
| Herald of the Horn, Horn Call | 7, 13, 19 | `skald`: "At 7th level, the DC of any spells with the sonic descriptor increases by +1. The DC increase is +2 at 13th level and +3 at 19th level." |
| Reformed Fiend, Hatred Against Evil | 5, 10, 15, 20 | `bloodrager`: "At 5th level, a Reformed Fiend gains +1 to attack and damage rolls against evil creatures while raging. This bonus increases by +1 every 5 levels (+4 at 20th level)." |
| Reformed Fiend, Damage Reduction (Reformed Fiend) | 7, 10, 13, 16, 19 (already so) | "At 7th level, a Reformed Fiend gains DR 3/evil. At 10th level and every three levels thereafter, his DR increases by +3." Checked, no change: its rows already carry it at each. |
| Flesheater, Devour Essence | 2, 8, 13, 18 | `barbarian`: "At 8th level and every 5 additional levels, the amount of HP restored increases by +1d8 (max 4d8 at 18th level)." |
| Flesheater, Power from Flesh | 2, 9, 18 | "At 9th and 18th levels, the bonus against edible enemies increases by +1." (read first, as the ticket asked: it has steps) |
| Fiendflesh Shifter, Fiendish Resilience | 2, 4, 8, 12, 16, 20 | `shifter`: "At 2nd level ... the shifter gains a +1 natural armour bonus to AC and 5 resistance to electricity and fire. At 4th, 12th and 20th level, the natural armour bonus increases by 1. At 8th and 16th level, the shifter's resistance to electricity and fire increases by 10." |
| Fake Celebrity, Inhibit Greatness | 12, 17 | `arcanist`: "From 12th level, a Fake Celebrity can use her performance to inflict a -2 penalty to attack rolls, a -1 penalty to saving throws and a -3 penalty to caster level checks to overcome spell resistance. At 17th level, the caster level penalty increases to -4." |
| Overwhelming Soul, Mental Prowess | 1 (granted), 10, 14, 18 | `kineticist`: "From 6th level, once per day, she can reduce the burn cost of any Wild Talent by 1. She can use this ability an additional time at 10th, 14th and 18th levels." |

**Fiendish Resilience: one feature, stepped at 2, 4, 8, 12, 16 and 20, not two.** The guide gives one ability with two series (natural armour at 2, 4, 12 and 20; the resistance at 2, 8 and 16), and the count cannot say which a step raised. Each of the five later steps improves the ability, so a count of one feature is the number of improvements it has had; nothing reads it (below), and each series' value follows from the owner's level when a number is wanted. Two features would split what the guide names once, with names the game does not use, and would show two "Fiendish Resilience" lines at level 2. If a later ticket computes armour or resistance, it reads the level, not the count.

**Mental Prowess:** the data grants it at 1 (the class's table lists it at level 1; its text says "From 6th level, once per day"). The new rows at 10, 14 and 18 carry the base row's wild talent; the count at 18 is 4 (the grant and three more uses). The review notes that the sentence's own first use comes at 6, which has no step here: the grant at 1 stands for the burn rule the class's table gives at level 1, and the steps are the sentence's "additional time at 10th, 14th and 18th levels". Left as built; a step at 6 would also follow from the sentence if the user wants the first use shown.

**Rows the build list's new archetype rows carry.** The Herald's rows at 13 and 19 carry the skald's Raging Song (Swift Action) and Damage Reduction (its table keeps both; the row at 13 and 19 of the base has nothing else). The Reformed Fiend's row at 15 keeps the bloodline feat slot (`OptionList(selection: "bloodrager/bloodline")`) and its row at 20 Mighty Bloodrage; its row at 10 already held the spell and DR. The Flesheater's row at 8 holds Devour Essence and no rage power (its table has none at 8), its row at 13 only Devour Essence, its row at 9 only Power from Flesh, and its new row at 18 keeps the rage power its table gives there (it had fallen back to the base row, so the Flesheater's rage powers are at 4, 10, 12, 16, 18 and 20, as before). The Overwhelming Soul's rows at 10, 14 and 18 keep the wild talent. The Fiendflesh Shifter's rows at 8, 12 and 16 are new and hold the feature alone (the base rows are empty there); its rows at 4 and 20 already existed.

### The sweep for others

Method: every feature of the class and prestige files whose summary says "every", "rising", "increases" or names two or more levels, granted in one row only, was read against the guide's sentence. The new rows follow the same rule as ticket 177: a level where the archetype has a row gets the feature added; a level where it has none gets a new row that copies what the base row gives there (features, feats, feats for a value, feat slots, selections), since the archetype's own table had no row there. The archetype therefore keeps every grant it held, and the only change to a sheet is the added feature (checked by the test below: each owner gains the feature at exactly these levels, and the class and every other archetype of the class at none).

**Stepped** (one feature of one archetype each; the steps include the level it is first granted at where the data already had it; guide sentence shortened to the step clause):

| Class | Archetype: feature | Steps at | Guide (the step sentence) |
| --- | --- | --- | --- |
| Barbarian | Armored Hulk: Resilience of Steel | 3, 6, 9, 12, 15, 18 | "This increases by +1 every three additional levels (+6 at 18th level)." |
| Barbarian | Invulnerable Rager: Extreme Endurance | 3, 6, 9, 12, 15, 18 | "This increases by 1 point every three levels (maximum 6 at 18th level)." |
| Bard | Archaeologist: Archaeologist's Luck | 1, 5, 11, 17 | "The bonus increases to +2 at 5th level, +3 at 11th level and +4 at 17th level." |
| Bard | Archaeologist: Danger Sense | 3, 6, 9, 12, 15, 18 | "These bonuses increase by +1 every 3 levels (maximum +6 at 18th level)." |
| Bard | Chelaxian Diva: Deadly Vibrato | 6, 14, 20 | "At 14th level, the damage increases to 2d8 and at 20th level it increases to 3d8." |
| Bard | Chelaxian Diva: Devastating Aria | 2, 7, 11, 15, 19 | "This damage increases by +1d6 at 7th level and every four additional class levels (max 5d6 at 19th level)." |
| Bard | Dirge Bard: Haunting Refrain | 5, 10, 15, 20 | "This increases by +1 every five subsequent levels (maximum +5 at 20th level)." |
| Bard | Flame Dancer: Fire Dance | 3, 6, 11 | "At 6th level this increases to 20 and at 11th level, it increases to 30." |
| Bard | Thundercaller: Thunder Call | 3, 7, 11, 15, 19 | "The damage increases to 3d8 at 7th level and by 2d8 every four additional class levels (maximum 9d8 at 19th level)." |
| Bard | Tranquil Whisperer: Inspire Tranquility | 1, 5, 11, 17 | "At 5th level and every six class levels thereafter, the bonus increases by +1 (maximum +4 at 17th level)." |
| Bloodrager | Greenrager: Furious Summoning | 9, 11, 20 | "At 9th level, creatures summoned by a Greenrager's summon allies spells gain a +4 morale bonus to Strength and Constitution. At 11th level, this bonus increases to +6 and at 20th level, it increases to +8." |
| Bloodrager | Hag Riven: Scarred Hide | 7, 10, 13, 16, 19 | "This increases by +1 every 3 additional class levels (maximum +5 at 19th level)." |
| Cavalier | Disciple Of The Pike: Bigger They Are | 1, 6, 12 | "At 6th level, the bonus increases to +3 when the creature is three size categories larger. Finally, At 12th level, the bonus increases to +4 when the creature is four size categories larger." |
| Cavalier | Disciple Of The Pike: Weapon Training (Spears / Polearms) | 5, 9, 13, 17 | "Weapon Training gives a +1 bonus to attack and damage rolls, increasing by an additional +1 every four levels (max +4 at 17th level)." |
| Cavalier | Fearsome Leader: Fearmonger | 3, 6, 9, 12, 15, 18 | "This increases by +1 for every three levels thereafter (max +6 at 18th level)." |
| Cavalier | Standard Bearer: Banner of Solace | 11, 15, 19 | "While the temporary HP last, allies gain a +2 morale bonus to damage rolls, increasing to +3 at 15th level and +4 at 19th." |
| Druid | Elemental Rampager: Claws | 1, 9, 15 | "At 9th level, the damage increases to 1d8 and finally at 15th level, the damage increases to 1d10." |
| Druid | Elemental Rampager: Rampage | 1, 6, 11, 16 | "The attack and damage bonuses increase by +1 at 6th level and every five levels thereafter (max +5 at 16th level)." |
| Druid | Primal Druid: Primal Size | 1, 8, 16 | "At 8th level, ... a +4 size bonus to Strength and Constitution. At 16th level, her size increases by two categories and she gains a +6 bonus to Strength and Constitution." |
| Druid | Winter Child: Arctic Native | 4, 8, 12, 16, 20 | "The cold resistance increases by +5 every four additional class levels (maximum 25 at 20th level)." |
| Fighter | Defender Of The Hearth: Shield Block | 1, 5, 10, 15, 20 | "At 5th level and every 5 subsequent levels, the number of attacks affected increases by 1 (max 5 at 20th level)." |
| Fighter | Tower Shield Specialist: Burst Barrier | 2, 6, 10, 14, 18 | "This increases by +1 four every additional class levels (maximum +5 at 18th level)." |
| Fighter | Two Handed Fighter: Strong Grip | 2, 6, 10, 14, 18 | "This increases by +1 for every four additional levels (maximum +5 at 18th level)." |
| Fighter | Two Handed Fighter: Two-Handed Weapon Training | 5, 9, 13, 17 | "The bonuses increase by +1 for every four additional class levels (maximum +4 at 17th level)." |
| Hunter | Colluding Scoundrel: Backstabber | 8, 15 | "This increases to 3d6 points at 15th level." |
| Hunter | Colluding Scoundrel: Scapegoat | 1, 8, 15 | "The attack penalty increases to -4 at 8th level and -6 at 15th level." |
| Hunter | Forester: Tactician | 3, 7, 12, 17 | "She can use this ability once per day, gaining an additional use at 7th, 12th and 17th levels." |
| Hunter | Wandering Marksman: Planned Distraction | 3, 12, 18 | "This increases to +2 at 12th level and +3 at 18th level." |
| Inquisitor | Faith Hunter: Sworn Enemy - Smite | 6, 10, 14, 18 | "At 10th level and every four subsequent levels, the Faith Hunter gains an additional daily use of this ability (max 4 at 18th level)." |
| Inquisitor | Living Grimoire: Blessed Script | 5, 10, 16 | "At 10th and 16th levels, this total increases by an additional +1." |
| Inquisitor | Tactical Leader: Swift Tactician | 3, 6, 9, 12, 15, 18 | "The ability can be used once per day plus one additional time per day at 6th, 9th, 15th and 18th levels. At 12th level, the Tactician ability becomes a swift action." |
| Kineticist | Kinetic Knight: Knight's Resolve | 3, 5, 7, 9, 11, 13, 15, 17, 19 | "She gains one additional usage per day for every two class levels beyond third for a maximum of 9 times at level 19." |
| Magus | Bladebound: Black Blade | 1, 5, 9, 13, 17 | "At 5th level and every four subsequent levels, the blade gains an additional +1 enhancement bonus (maximum +5 at 17th level)." |
| Magus | Bladebound: Sharpening the Black Blade | 3, 6, 9, 12, 15, 18 | "For every three additional levels, the damage bonus increases by +1 (maximum +6 at 18th level)." |
| Monk | Drunken Master: Drunken Courage | 7, 11 | "From 7th level ... immune to fear ... From 11th level, she also gains a +4 alchemical bonus to saves against mind-affecting and compulsion effects." |
| Monk | Drunken Master: Drunken Ki | 3, 5, 7, 8, 9, 11, 13, 15, 17, 19 | "The size of the pool is two points + 1 for each additional odd-numbered level (maximum 10 points at 19th level). At 8th level, this becomes a swift action." |
| Monk | Drunken Master: Drunken Resilience | 9, 15, 19 | "At 15th level, the DR increases to DR 5/- and at 19th level, it becomes DR 10/-." |
| Monk | Sensei: Mystic Powers | 6, 10 | "At 10th level, she may do the same for all allies within 30' without having to expend extra ki points." |
| Monk | Sensei: Mystic Wisdom | 10, 14 | "At 14th level, she may expend 2 points to grant an ability to all allies within 30' or else provide Improved Evasion to a single ally." |
| Monk | Sohei: Ki Weapon | 4, 8, 12, 16, 20 | "This bonus increases by +1 for every four additional levels (max +5 at 20th level)." |
| Monk | Student Of Stone: Bones of Stone | 7, 10, 15 | "At 10th level, this becomes DR 2/chaotic. Finally, at 15th level, this becomes DR 5/chaotic." |
| Monk | Student Of Stone: Soul of Stone | 12, 16 | "At 16th level, the range of this ability increases to 30'." |
| Oracle | Enlightened Philosopher: Mental Acuity | 7, 10, 13, 16, 19 | "At 7th level, you gain a +1 inherent bonus to Intelligence, increasing by an additional +1 every three additional class levels (maximum +5 at 19th level)." |
| Oracle | Lone Strider: Fade from Memory | 7, 14 | "At 14th level, this increases to 50% concealment." |
| Oracle | Lone Strider: Recluse's Stride | 1, 5, 10 | "At 5th level, you gain a +20 AC bonus against the first attack of opportunity in a round. At 10th level, you can teleport a medium distance as a move action ..." |
| Oracle | Wind Whisperer: Haste of the Wind | 7, 13, 19 | "He gains a second use of this ability at 13th level and a third at 19th." |
| Paladin | Hospitaler: Smite Evil (Hospitaler) | 1, 7, 13, 19 | "A Hospitaler gains fewer uses of Smite Evil, receiving additional uses at 7th, 13th and 19th levels." |
| Paladin | Stonelord: Stoneblood | 3, 9, 15 | "At 9th level, this chance increases to 50% and she becomes immune to petrification. At 15th level, this chance increases to 75% and she becomes immune to bleed." |
| Paladin | Tortured Crusader: Second Chance | 2, 8, 15 | "From 8th level, she restores 1d8 damage per two class levels and at 15th level, the healing is maximised." |
| Paladin | Warrior Of The Holy Light: Power of Faith | 4, 8, 12, 16, 20 | "At 4th level, the Warrior gains an additional use of Lay on Hands and gains one additional use every four subsequent levels." |
| Ranger | Flamewarden: Touch of Flame | 4, 12 | "At 12th level, weapons gain the flaming burst ability instead." |
| Ranger | Stormwalker: Wind Treader | 4, 12 | "At 12th level, weapons gain the shocking burst ability instead." |
| Rogue | Knife Master: Blade Sense | 3, 6, 9, 12, 15, 18 | "This bonus increases by +1 every three additional levels (maximum +6 at 18th level)." |
| Rogue | Sylvan Trickster: Fey Resistance | 8, 11, 14, 17, 20 | "This increases by +2 every three additional levels (max DR 10/cold iron at 20th level)." |
| Shaman | Shadow Shaman: Shadows | 4, 9, 14 | "At 9th level, he gains the benefits of Displacement instead and at 14th level, he receives the effect of Greater Invisibility ." |
| Shaman | Spirit Hunter: Spirit Weapon Enchantment | 1, 5, 9, 13, 17 | "For every four levels above 1st, the weapon gains another +1 enhancement bonus (maximum +5 at 17th level). From 5th level, he can add any of Flaming, Frost, Shock, Keen and Speed." |
| Shifter | Child Of The Manticore: Manticore Aspect | 1, 5, 10, 15, 20 | "At 5th level, when making the first attack in a round, the shifter throws two spikes rather than one. At 10th level, the inherent bonus to Strength and Dexterity increases to +4." |
| Shifter | Child Of The Manticore: Manticore Shape | 4, 9, 14 | "At 9th level, he gains immunity to ground-based effects and a +4 bonus to damage with natural attacks. At 14th level, he gains the improved natural attack ability with spike attacks." |
| Shifter | Griffonheart Shifter: Griffon Form | 4, 9, 14 | "At 9th level, the shifter gains the pounce ability and improved natural attack with his bite ability. Finally, at 14th level, the shifter gains the rake ability." |
| Shifter | Wild Effigy: Heart of Earth | 4, 9, 15 | "At 9th level, this chance increases to 50% and he becomes immune to petrification. At 15th level, this chance increases to 75% and he becomes immune to bleed." |
| Skald | Court Poet: Insightful Contemplation | 1, 4, 8, 12, 16, 20 | "At 4th level and every four levels thereafter, the Will save bonus increases by +1 (max +6 at 20th level). At 8th level the attribute bonuses increase to +4." |
| Skald | Inciter: Caustic Ridicule | 10, 15 | "At 15th level, this penalty increases to -2." |
| Skald | Inciter: Prompted By Backstabbing | 8, 13, 18 | "This increases by +1d6 at 13th level and 18th level." |
| Slayer | Executioner: Assassinate | 10, 14, 19 | "At 14th level, when an Executioner assassinates a target, enemies within 30' become demoralised. At 19th level, an enemy surviving an assassinate attempt takes 2d6 bleed damage." |
| Slayer | Stygian Slayer: Invisibiliy | 4, 8, 12, 16, 20 | "He gains an additional use at 8th level and every 4 additional class levels (max 5 times at 20th level)." |
| Slayer | Spawn Slayer: Studied Spawn | 5, 10, 15, 20 | "At 10th level, if the target is Huge, the bonuses increase by +2. At 15th level, the bonuses are Gargantuan: +3 and at 20th level the bonuses are Colossal: +4." |
| Warpriest | Champion Of The Faith: Smite | 4, 8, 12, 16, 20 | "The Warpriest gains an additional use of Smite every four levels beyond 4th (maximum 5/day at 20th level)." |
| Warpriest | Disenchanter: Mystic Interference | 4, 8, 12, 16, 20 | "At 8th level and every four levels thereafter, the bonus increases by +1 (maximum +5 at 20th level)." |
| Warpriest | Mantis Zealot: Mantis Swarm | 4, 11, 18 | "This increases by an additional +1d6 at 11th and 18th levels." |
| Warpriest | Mantis Zealot: Fading | 7, 10, 13, 16, 19 | "This chance increeases by +10% at 10th level and every three subsequent levels (max 60% at 19th level)." |
| Warpriest | Proclaimer: Cleanser of Evil | 2, 5, 7, 8, 11, 13, 14, 17, 19, 20 | "At 5th level and every three levels thereafter, the damage increases by +1d6 (max 7d6 damage at 20th level). At 7th level, the radius increases to 10' and by +5' every six levels (max 20' at 19th level)." |
| Witch | Hag Of Gyronna: Sunder Hope | 8, 14, 20 | "She can use this ability twice per day from 14th level and a 3rd time at 20th level." |
| Witch | Hagbound: Hunched Muscle | 2, 8, 14 | "This increases to +4 at 8th level and +6 at 14th level." |
| Witch | Witch Of The Veil: Shrouded Step | 2, 8, 16 | "At 8th level, she can teleport 30' and then become invisible. At 16th level, the effect is Greater Invisibility ." |
| Wizard | Shadowcaster: Umbral Mind | 10, 12, 14, 16, 18 | "This increases by an additional +1 every two class levels (maximum +5 at 18th level)." |
| Wizard | Spell Master: Focused Spells | 1, 8, 16 | "He can do this twice per day at 8th level and three times per day at 16th level." |
| Wizard | Spell Master: Item Bond Specialization | 5, 10, 15, 20 | "At 5th level and every 5 subsequent levels, the Spell Master gains an additional use of his Bonded Object to restore a spell." |

**Not stepped, with the reason:**

- Hag-Riven's Sorcerous Claws: the guide's own numbers disagree ("This increases by +1 every 3 additional class levels (maximum +4 at 13th level)": from 5th, every three levels gives +4 at 14th, not 13th).
- Drunken Master's Drunken Power: "+1d8 every 3 additional levels (max +5d8 at 18th level)": from 5th, every three levels gives +5d8 at 17th, not 18th.
- Mantis Zealot's Red Shroud: "The AC bonus increases by +1 every four levels up to a maximum of +6 at 20th level": the sentence names no level. Read as 4, 8, 12, 16 and 20 it fits the +6 at 20th (read as every four levels after 1st it gives +5 at 17th instead), but either reading is an inference, not the guide's levels. (The review corrected the reason first given here, which counted from 1st only.)
- **Features whose steps the data already grants as features of their own** (struck by the review, 2026-11-13; they were stepped at first): the same reason as the magus's Arcane Weapon and Arcane Armor below, since stepping the feature as well counts each step twice. Each stays granted once at its first level, pinned in `features_whose_steps_are_features_of_their_own_are_granted_once`:
  - Phantasmal Mage, Arcane Metamagic: its "at levels 5, 11 and 17" are Arcane Metamagic I, II and III, on the rows at 5, 11 and 17.
  - Beast Tamer, Beast Call: each step ("At 3rd level and every two levels thereafter, the power increases by 1 level") is Summon Nature's Ally II to IX, on the rows at 3 to 17.
  - Hag-Riven, Claws of the Hag: the steps at 2, 5, 9, 13 and 17 are Claws of the Hag (Magic), (1d6), (Rend), (Critical) and (Weakness), as the guide's table names them. It is also a list of separate abilities, not one number.
  - Elemental Rampager, Elemental Rampage: the sentence quoted for it ("This increases to 2d6 at 11th level and 3d6 at 20th level") is Elemental Damage's, a feature already granted at 4, 11 and 20; Elemental Rampage's own ("along with abilities at 11th and 20th level") is Elemental Ability, granted at 11 and 20.
  - Winter Child, Call a Feast: its 10th and 15th-level gains are Call a Feast - Cover of Snow and - Biting Frost, on those rows (the guide's table names both).
  - Armored Battlemage, Arcane Pool (Armored Battlemage): "+2 at 5th level and every four levels thereafter to +5 at 17th level" is Arcane Armor +2 to +5, on the rows at 5, 9, 13 and 17.
  - Spell Dancer, Spell Dance: the guide's table names Spell Dance (II), (III) and (IV) at 5, 9 and 13, and their summaries carry the +20', +30', +40' rises. The text's "each four subsequent levels" would also give a rise at 17, which the table does not name: left, with the table.
  - Possessed Oracle, Powerless Prophecy: it is the archetype's one curse, and its 5th, 10th and 15th-level gains are the Curse 2, 3 and 4 already on those rows.
  - Purifier, Celestial Armor: its 11th-level gain is "Armor Proficiency (Heavy) as a bonus feat", which the row at 11 already grants as a feat. Its Armor Training "as a Fighter four levels lower" rises with the fighter's Armor Training, which is the base classes' question.
- Features whose summary names several levels because they are lists of separate abilities, not a number that rises: the bloodline spells (7, 10, 13, 16), Bottled Ally, Fey Magic, Summoning Rager, Shadow Spells, Dual-Cursed Oracle's bonus spells, Versatile Healing Channel (a list of spells), the magus's Arcane Weapon and Arcane Armor and the transmuter's Physical Enhancement (already one feature per step), the shifter's claws, spikes and slam (the same), the Reanimator's Reanimation features.
- Counter Curse, Overwhelming Spell and Spontaneous Healing: no level steps (the rise is per spell level sacrificed).
- Marked "no step by class level in the guide's sentence": Bomb Reanimator ("skip the damage increases"), Animal Companion of the Mad Dog, Wandering Spirit 2 (two grants).

**Found and left** (not stepped here; each needs more than data rows, or is the user's question):

- **Option rows**: the bloodlines' abilities that rise (the sorcerer's Abyssal Claws, Arcane Combat Casting Adept, Infernal Hellfire, Serpentine Fang and Undead Grasp of the Dead; the bloodrager's Abyssal Claws, Hag Fortitude, Hag Transformation and Serpentine Fang), the domains' powers (the cleric's and the inquisitor's: the resistances 10 and 20, Chaos Blade and the other 1/day-plus-one powers, Aura of Protection, Body of Ice), the shaman spirits' (Shard Soul, Spirit of Nature, Body of Earth), the warpriest's blessings and the wizard's schools (Resistance, Elemental Wall, Universalist's Metamagic Master). Their rows sit in the option, which several selections share on different level scales (the Seeker, the Eldritch Scion, the second spirit, the cleric and the inquisitor's domains), so each needs the sharer table 177 built; one ticket for the lot is the right size.
- **Class features of the base classes** (the user's question below, and the same shape): Danger Sense (barbarian), the cavalier's Banner, the druid's and the warpriest's Wild Shape (Wolf) uses, Channel Energy, the paladin's and the ranger's, Fighter Weapon Training and Armor Training and the like, and the copies of them in archetypes that replace them (Wild Shape (Wolf) in four druid archetypes and the Feral Champion).
- The guide's table puts the Mantis Zealot's Mantis Swarm at 4 and its text says "From 5th level"; the data follows the table (4) and steps at the sentence's later levels (11 and 18), so its first grant is unchanged.

### What reads each count

Nothing. A search of `libs/`, `apps/` and `tools/` for every key above outside the class files finds only: the Advanced Weapon Training prerequisite (`Feature(..., count: 1)`, which holds at 1 or more) naming the Disciple of the Pike's Weapon Training (Spears / Polearms); the mythic ability Abundant Smite's, naming the Hospitaler's Smite Evil (also `count: 1`; the review found it, the first search missed `mythic/`); the Extra Arcane Pool and Abundant Arcane Pool prerequisites and `bonuses.ron`'s Possessed Oracle initiative bonus, naming features the review unstepped (the Armored Battlemage's Arcane Pool, `count: 1`; Powerless Prophecy, held or not); the engine's other readers of a count (`bonuses`, which asks that a feature is held, a count above 0; the options' "held" check, the same; the Winter Witch's `counts_as`, which counts only the witch's hexes; the MCP sheet's new features, which lists a step as a gain at its level, as intended), which asks only that the feature is held (a count above 0). The only `ability_bonuses` is the sorcerer's Strength of the Abyss, stepped by ticket 170. So no number or pick moves.

### The archetype table

`assert_gains` builds every archetype of the class (and the class) as a possible Build and asserts what each gains the feature at. In every case below the owner gains the feature at the levels listed and **the class and every other archetype of that class gain it at none** (an archetype that does not have it is not given a step it never had; an archetype that replaces a base feature by one of its own does not share it). The owners and levels are the tables above and these:

| Feature | Owner | Held at the steps | Other archetypes and the class |
| --- | --- | --- | --- |
| Horn Call | Herald of the Horn | 7, 13, 19 | none (the Herald also keeps Raging Song (Swift Action) at 13 and Damage Reduction at 9, 14, 19, as the skald's rows; asserted) |
| Hatred Against Evil | Reformed Fiend (Abyssal or Infernal) | 5, 10, 15, 20 | none; Mighty Bloodrage at 20 and the bloodline feat slots at 6, 9, 12, 15, 18 kept (asserted) |
| Devour Essence; Power from Flesh | Flesheater | 2, 8, 13, 18; 2, 9, 18 | none; rage power slots at 4, 10, 12, 16, 18, 20 (asserted) |
| Fiendish Resilience | Fiendflesh Shifter | 2, 4, 8, 12, 16, 20 | none |
| Inhibit Greatness | Magic Deceiver, Fake Celebrity way | 12, 17 | the other five ways and the class: none |
| Mental Prowess | Overwhelming Soul | 1, 10, 14, 18 | none; wild talent slots as the base's (asserted) |
| the sweep's features | each owner above | its steps | none |

**Snapshot check** (`SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`, then `git status`): no snapshot file changed. The check's message is empty ("the engine's output matches the snapshots") before and after regenerating: every corpus Build is a base class, a prestige class or a path with no archetype, and none holds a feature stepped here, so every changed line is in a new test, not a snapshot. The gold builds (`gold.rs`) pass with no known gap added or removed.

**`scripts/check.sh`:** ```text
fmt OK (1s)
clippy OK (2s)
wasm-clippy OK (35s)
test OK (424s)
rules-wasm OK (10s)
seed-check OK (1s)
ALL-OK
```
(No `--schema`: no migration or query touched.)

**Tests** (`libs/seeds/tests/game_data/scaling_features.rs`): one test each for the Herald, the Reformed Fiend, the Flesheater, the Fiendflesh Shifter, the Fake Celebrity and the Overwhelming Soul; one test per class for the sweep (the table above, asserted for the class and every one of its archetypes); and `features_whose_steps_are_features_of_their_own_are_granted_once` for the nine the review unstepped (each granted once, at its first level, for its owner and no other archetype; the step features at their levels).

### Review (Opus, 2026-11-13)

- **Every stepped feature read against the guide's own sentence** (the saved pages, converted to text): the levels of the Build list's seven and of the sweep's features match the sentence on the owner's level scale, with the first grant at the level of the archetype's table where the sentence gives none (Resilience of Steel, Beast Call, Inspire Tranquility, Heart of Earth, Smite Evil (Hospitaler), Recluse's Stride, Bigger They Are and the like). On the builder's uncertain cases: Mantis Swarm (table 4, text "From 5th") keeps its first grant at 4 and steps at 11 and 18: sound. Heart of Earth: the sentence gives 9 and 15 and no first level; the table's 4 stands: sound. Mental Prowess: see its note above. Hospitaler's Smite Evil: "additional uses at 7th, 13th and 19th levels", granted at 1 by the table: sound. Recluse's Stride: 5 and 10 from the sentence: sound. Spirit Weapon Enchantment: "for every four levels above 1st" is 5, 9, 13 and 17: sound. Claws of the Hag, Arcane Metamagic and Beast Call: unstepped, with six more of the same kind (the struck list above).
- **Nine features unstepped**, each already stepped by features of its own (the struck list above). Their rows are back as on `65104fc`, except where another stepped feature of the same archetype shares the row.
- **No leak: every class and archetype built at 20 on `65104fc` and on this branch, then diffed.** 222 pick-less Builds (each class and each of its archetypes, made possible with the first race and alignment that allow it), projected as the snapshot check does, with every slot's open options. After the review's fixes, 66 Builds differ, in 256 lines, every one a `level.N.feature/<key>` line adding a stepped feature at one of its step levels: 83 features (the 92 the pick-less Builds reach, less the nine unstepped; Inhibit Greatness is an option row and is pinned by its own test). No feat, feat slot, selection, open option, number or other feature changed.
- **Readers of the counts:** the mythic abilities Abundant Smite and Abundant Arcane Pool read two of the features (`count: 1`); added to "What reads each count" above. Nothing else.
- **The struck list:** the Red Shroud's reason was wrong in its arithmetic (corrected above); it stays struck, since the sentence names no level. The other reasons hold.
- Snapshots: none changed. The gold builds pass.

### The question for the user, unchanged

The base classes' stepped features (the fighter's Bravery, Armor Training and Weapon Training, Danger Sense, Banner and the like) are granted once at their first level, as ticket 170's features were. Stepping them would touch every archetype with a row at a step level, so the cost is far larger and each archetype that loses or delays one needs its own row. Not built; the notes in the ticket's Build section still stand. The sweep above found more of the same (the druid's and the warpriest's Wild Shape (Wolf) uses, Channel Energy), which belong to the same decision.
