# 306h: the skald's and the Instinctual Warrior's rage powers are their own lists

## What the data says now

- New selections `barbarian/skald-rage-power` (`SkaldRagePowerSelection` 2476514e31791394fa140f1a07941c96, 33 offers) and
  `barbarian/instinctual-warrior/rage-power` (`InstinctualWarriorRagePowerSelection`
  609f0e5336084442a0dafa3abd4d31c5, 38). `barbarian/rage-power` keeps the barbarian's 33 (`RagePowerSelection`
  28710502f46848d48b3f0d6132817c4e); the five `*-instinctual-warrior` options moved to the Instinctual Warrior's list as
  its own options (keys unchanged), and its other 33 are shared with the barbarian's.
- Rows: `classes/skald.ron` 3 to 18, `archetypes/skald/demon-dancer.ron` 12 and `archetypes/skald/inciter.ron` 6, 12, 18
  name the skald's list; `archetypes/barbarian/instinctual-warrior.ron` 4 to 20 name the Instinctual Warrior's.
- **The skald's key is `barbarian/skald-rage-power`, not `skald/rage-power`:** a row's selections sort by key and slot ids
  follow, and the skald's rows at 12 (and the Demon Dancer's) hold `rogue/skald-talents` too. `skald/rage-power` would
  sort after it and move both slots, so a base link's picks there would stop opening. `barbarian/skald-rage-power`
  sorts, as `barbarian/rage-power` did, before `rogue/skald-talents`. The Instinctual Warrior's rows hold one selection each.
- Extra Rage Power (`barbarian/extra-rage-power`, `ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a) lists the
  game's 38 as shared options and is named "Rage Power", the label of its slot. Its `needs_slot_of` names the barbarian's,
  the skald's, the Instinctual Warrior's and (until 264) the Battle Scion's selection.
- `barbarian/pack-rager/bonus-feats` (`PackRagerTeamworkFeatSelection` 94ebbd6472c19fa4ea7196eaff11a740) gained Shake It Off, the game's thirteenth.
- The Battle Scion's selection (`BattleProwessSelection` 29b480a26a88f9e47a10d8c9fab84ee6) shares the game's 21 rage powers
  (hand edit: the list also holds the twelve teamwork feats, which the writer can't express beside options).

## Written by the writer, and by hand

`game-check --write option-lists --only barbarian/rage-power,barbarian/skald-rage-power,barbarian/instinctual-warrior/rage-power,barbarian/extra-rage-power,barbarian/pack-rager/bonus-feats`
wrote the lists. The writer skips a selection with no list, so the three new/empty ones were seeded with one shared
option first (`barbarian/rage-power/clear-mind`, which two options hold, so the writer needs one the selection offers
already to choose between them), as 306g did.

## Not modelled

- The Battle Scion's twelve teamwork feats stand as one option, `teamwork-feat`: the game check still lists them as
  "in game only" under 286.
- Extra Rage Power's prerequisites and its Battle Scion link (264): the test that closes it to a Battle Scion stays ignored.

## Tests

`importer_lists::the_skald_and_the_instinctual_warrior_have_their_own_rage_powers`; un-ignored
`extra_picks::the_barbarians_rage_power_slot_offers_33_and_extra_rage_power_38` and
`skald_rage_powers::the_battle_scions_selection_offers_21_rage_powers`.
