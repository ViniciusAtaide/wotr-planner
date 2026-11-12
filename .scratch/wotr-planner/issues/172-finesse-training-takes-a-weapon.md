# Finesse Training takes a weapon

Type: task
Status: resolved
Blocked by: 160
Model: Opus to build; review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists "Finesse Training - Light Shield" at the Enforcer's level 18 as a known gap. The planner grants the rogue's Finesse Training with no weapon choice, by ticket [54](54-mercies-and-enforcer-exploits.md)'s rule that a choice which gates no pick stays a label.

The user, 2026-11-11: "make finesse training take a weapon". This reverses ticket 54's rule for Finesse Training only.

## Build

AFK. Game data and whatever the engine needs for a feature that takes a value; no schema change expected, but check.

- Find how the guide's rogue page (`guide-cache/rogue.html`) words Finesse Training: at which rogue levels a weapon is chosen, and from which weapons. The data follows the guide.
- Give Finesse Training a weapon value at each level the guide gives a choice, the way other parameterised picks take a weapon (`Parameter::Weapon`). Every archetype that keeps Finesse Training gets it.
- An old link, where Finesse Training has no weapon, must still open; its weapon shows as a choice still to make.
- The number it changes: Finesse Training lets Dexterity replace Strength on damage with the chosen weapon. Say in the Answer whether the attack line counts it now or it is left for the attack-line tickets (139, 140).
- Remove the Finesse Training entry from the gold test's known gaps and give the Enforcer the Light Shield. Regenerate the snapshots and review the diff: only Finesse Training's lines change.

## Done when

- "Finesse Training - Light Shield" agrees in the gold test, with its known-gap entry deleted.
- A real-data test pins the weapon choice at each level the guide gives one.
- An old link with Finesse Training still opens.
- `scripts/check.sh` passes.

## Answer

Done 2026-11-12 (branch `ticket-172`). Finesse Training is a pick: a weapon at rogue 3, 11 and 19, from the guide's list, once per weapon. This reverses ticket [54](54-mercies-and-enforcer-exploits.md)'s "choices that gate no pick stay labels" for Finesse Training only (the user, 2026-11-11). One small engine change (an option counted once per value of its own) and one migration; the link format is unchanged.

**The guide** (`guide-cache/rogue.html`, `rogue#finesse-training`, the only saved page that describes it; `items` and `character-creation` only name it):

> At 3rd level, a Rogue may select a finessable weapon that she has proficiency in. When she attacks with this weapon, she adds her Dexterity modifier instead of her Strength modifier to the damage roll. She can select a second weapon at 11th level and a third at 19th level.
>
> The weapons selectable are: Bite, Claw, Dagger, Dueling Sword, Elven Curve Blade, Estoc, Handaxe, Kama, Kukri, Light Hammer, Light Mace, Light Pick, Light Shield, Nunchaku, Punching Dagger, Rapier, Sai, Shortsword, Sickle, Starknife, Unarmed.

Every rogue table but the Rowdy's lists "Finesse Training" at 3, 11 and 19; the Rowdy "Loses finesse training + Evasion". The list includes the Light Shield, so the Enforcer's pick needs no decision: it is `light-shield-bash` ("Light Shield Bash", martial, light, finessable) in `weapons.ron`. Every name on the list is a weapon there: "Handaxe", "Shortsword" and "Unarmed" are the Hand Axe, the Short Sword and the Unarmed Strike.

**The mechanism.** The data already has "a selection whose one option takes a value" (the Zen Archer's Way of the Bow, the shaman's Battle Master: `parameter: Weapon`, `values: [...]`). So `rogue/finesse-training` is a selection in place of the feature, and its one option `rogue/finesse-training/weapon` ("Finesse Training") takes `parameter: Weapon` with the guide's 21 `values`. A pick reads "Finesse Training (Light Shield Bash)". The new part is the repeat. "A second weapon" means another weapon, but an option's `PerValue` counted only a chosen feat's values (ticket 47), and the state held no value of an option's own. `Repeat::Times(3)` would have offered the dagger again at 11. So the engine changed in four small places:

- `CharacterState::option_values`, with `has_option_for`: the values an option is held for when the value is the option's own;
- the walk holds and releases them (`Held::OptionValue`);
- the judge's `PerValue` checks them for an option with no chosen feat;
- `GameData::check` accepts `PerValue` on an option with a `parameter` (`DataIssue::PerValueOptionWithoutValue`, renamed from `PerValueWithoutChosenFeat`).

The seed's matching CHECK needed the migration `20261112094713_172-option-per-value.sql`, which recreates `selection_option` with the CHECK widened. No `Build`, codec or link change was needed.

**Classes and archetypes.** Only `classes/rogue.ron` names Finesse Training (`rg finesse-training libs/game-data`; no other class, prestige class or path page grants it). The base rows 3, 11 and 19 hold the selection. So do the archetype rows that replace them and keep it: the Eldritch Scoundrel's 3, 11 and 19, the Master of All's 3, 11 and 19, and the Knife Master's and the Thug's 3. The Dark Lurker, the Sylvan Trickster and the Underground Chemist have no row there and take the base rows. **The check for archetypes that lose it:** an archetype row replaces the base row whole, so an archetype loses the selection only by having its own row at that level. The Rowdy has its own rows at 3, 11 and 19 without it, so it gains no slot. The test builds every rogue archetype at 20 and asserts the levels: [3, 11, 19] for the seven that keep it and none for the Rowdy. No archetype falls back to a base row it shouldn't take.

**Older links.** Before this ticket a Build held Finesse Training as a feature, with no slot. Now its rogue 3, 11 and 19 have an empty "Finesse Training" slot. The link text is the same (the codec stores picks, and there is none). The Build opens and is possible, and each weapon is a choice still to make. The MCP lists it in `todo` as "Level 3: Finesse Training", and the web app shows an empty chip. The real-data test opens a pinned rogue 20 link (`AaXEsQkAAAgDwdha60AO5N6OIcQFhBzPZ08p4TgFnoxf`) and the bench's real `multi_prestige20` link, whose rogue 3 is at level 5. `V1_LINK`, `V1_GEAR_LINK` and `version_1_body_layout_is_frozen` are untouched. No pick was "left open": the Weapon kind is not `may_stay_open`, so a pick must name its weapon, and an older Build has no pick at all.

**The attack line** doesn't count Finesse Training. It is a class feature, and `ATTACK_LINE_LEAVES_OUT` leaves class features out ("class features (Weapon Training, sneak attack, …)"). Counting it would mean Dexterity replacing Strength on damage for a weapon the state holds the option for: a few lines now that `option_values` exists. But Fencing Grace and Slashing Grace, the feats that do the same, aren't counted either, and counting one source of Dex to damage without the others would make the line inconsistent. So it stays with the attack-line tickets: [139](139-attack-line-weapon-features-and-mythic-feats.md) (weapon class features) should name it beside Weapon Training when it is designed. This ticket doesn't edit 139.

**Not checked:** the guide's "that she has proficiency in". No weapon value checks proficiency (Weapon Focus's neither), and the list holds weapons a rogue isn't proficient in without a feat (the exotic ones, the martial Light Shield). Ticket 155 (proficiency in the attack line) is the place for it.

**The web app and the MCP** needed no change. The level's chip is labelled "Finesse Training". Its picker shows one entry, which opens the 21 weapons (`Engine::slot_entries`: one entry, noun "a weapon", 21 options; pinned in the real-data test). The grid shows a filled slot as "Finesse Training (Light Shield Bash)". In the MCP, `options` with `values: true` lists the 21 as `{key: "rogue/finesse-training/weapon", parameter: …}`, and `choose` takes one. At 11 the weapon taken at 3 is `already_held` ("Already taken"). The bare key is refused with `needs_parameter`. `apps/mcp/tests/server/finesse_training.rs` pins these.

**Snapshot check**, before regenerating (every changed line is Finesse Training's: its feature lines go, and its slot, options and the Enforcer's pick come in; three files changed: `class-rogue.txt`, `gold-enforcer.txt`, `bench-multi_prestige20.txt`):

```text
the engine's output differs from the snapshots in 3 of 44 Builds

Changes, each with the Builds it's in:
  - level.11.feature/rogue/finesse-training = Finesse Training in 1 Build: class-rogue
  - level.18.feature/rogue/finesse-training = Finesse Training in 1 Build: gold-enforcer
  - level.19.feature/rogue/finesse-training = Finesse Training in 1 Build: class-rogue
  - level.3.feature/rogue/finesse-training = Finesse Training in 1 Build: class-rogue
  - level.5.feature/rogue/finesse-training = Finesse Training in 1 Build: bench-multi_prestige20
  + slot.selection.11.0 = Finesse Training: - in 1 Build: class-rogue
  + slot.selection.11.0.open/rogue/finesse-training/weapon(bite) in 1 Build: class-rogue
  … (one line per weapon and level)
  + slot.selection.18.0 = Finesse Training: rogue/finesse-training/weapon(light-shield-bash) "Finesse Training (Light Shield Bash)" in 1 Build: gold-enforcer
  + slot.selection.19.0 = Finesse Training: - in 1 Build: class-rogue
  + slot.selection.3.0 = Finesse Training: - in 1 Build: class-rogue
  … and 13 more

The same, levels and ranks wildcarded:
  level.*.feature/rogue/finesse-training: - Finesse Training in 3 Builds, 5 lines
  slot.selection.*.0: + Finesse Training: - in 2 Builds, 4 lines
  slot.selection.*.0.open/rogue/finesse-training/weapon(bite): + in 1 Build, 3 slots
  … the same for each of the 21 weapons: + in 1 Build, 3 slots
  slot.selection.*.0: + Finesse Training: rogue/finesse-training/weapon(light-shield-bash) "Finesse Training (Light Shield Bash)" in 1 Build, 1 line
```

**Gold test:** the `Reopened(172)` entry is deleted. The Enforcer takes `rogue/finesse-training/weapon` for `light-shield-bash` in `selection(18, 0)`: character level 18 is its rogue (Sylvan Trickster) level 3, its levels 16 to 19 being rogue. `VALUES` reads the write-up's "Light Shield" as "Light Shield Bash". The line "Finesse Training - Light Shield" now agrees. Three known gaps are left: Mythic Shield Focus (171), the Elk companion (174) and the Enforcer's Uncanny Dodge (173).

**Tests:**

- `libs/seeds/tests/game_data/finesse_training.rs` (5 tests):
  - the option's list is the guide's 21, `PerValue`, and no feature is left;
  - a rogue 20 has the slot at 3, 11 and 19, labelled and offering all 21, with the picker's one entry; three weapons filled is possible, and the pick's name;
  - a weapon taken at 3 is "Already taken" at 11, and the longsword is refused;
  - every archetype's levels, with the Rowdy's none, and only the rogue naming the selection;
  - the older links.
- `parameterised_feats.rs`'s repeat table gains the option.
- `rules`: `an_option_once_per_value_repeats_for_another_value_of_its_own` (the test world's Battle Master once per weapon) and the check test reworded.
- `seeds/tests/database.rs`: the CHECK accepts `per_value` on an option with a parameter.
- MCP: `finesse_training.rs` (3 tests).

**For the queue's proof:** the proof's sheet hash leaves out an empty `option_values` (`tools/proof/src/fingerprint.rs`), as it does a state's empty faith. A Build that picks an option with a value of its own still differs there: the shaman's (and the test world's slayer's) Battle Master, besides every rogue Build's new Finesse Training slots. That difference is the new field, not a change in what is allowed.
