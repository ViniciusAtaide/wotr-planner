# 306k: the Aeon's three gaze lists, the Elemental Witch's rays, the Nine-Tailed Heir's bloodline, the Vivisectionist's Extra Discovery, the angel's greater list

## What the data says now

- **Aeon gaze:** `aeon/enforcing-gaze` (`AeonGazeFirstSelection` 7e468a52…) lists the game's 8 and is the picker of ranks 3 and 4. New
  `aeon/enforcing-gaze-2` (`AeonGazeSecondSelection` 7d498e03…, 16: the first's 8 plus HP, concentration, summoner's gaze, relativity of
  space, primordial magic, normalize size, area effects and power of law) serves ranks 5 to 7 and `aeon/enforcing-gaze-3`
  (`AeonGazeThirdSelection` 0bec49f6…, 17: the second's 16 and the temporal jump) ranks 8 to 10 (`AeonProgression` 34b9484b… grants them at its
  levels 1 and 2, 3 to 5 and 6 to 8; Aeon level 1 is mythic rank 3). Every option keeps its key; the writer moved the ones the first list
  lacks into `-2`'s `options` (first in key order whose list holds them) and the temporal jump into `-3`'s. The options' own
  `MythicLevel(5)`/`MythicLevel(8)` `Requires` stay (266's `known.txt` lines); the tier lists now make them redundant.
  Each rank row holds one selection, so no slot moves. `mythic/paths/devil.ron` drops all three selections when an Aeon switches.
- **Elemental Witch:** the archetype's row 1 names the new selection `witch/elemental-witch/energy` (`ElementaWitchEnergySelection`
  9ec1bf20…) in place of the feature `witch/elemental-witch/elemental-ray`; its four options (`witch/elemental-witch/energy/{acid, cold,
  electricity, fire}`, the `ElementalWitch*RayFeature` blueprints) each grant that feature on their row 1, so the sheet keeps it. Row 11
  still grants the feature (S5). The selection names `option_levels: ["witch"]`, as the game's rays are plain features with no
  progression. The row had no selection (the archetype removes the witch's hex and patron), so the new one is its slot 0.
- **Nine-Tailed Heir:** new `sorcerer/nine-tailed-heir/bloodline` (`NineTailedHeirBloodlineSelection` 7c813fb4…, the sorcerer's 21 shared) on
  row 1 in place of `sorcerer/bloodline`. The row's selections stay in the order they sorted in (`magic-feats`, then the bloodline), so the
  slot ids stay. The options stay owned by `sorcerer/bloodline`, so `holds_option_of("sorcerer/bloodline")` (Dragon Disciple's
  prerequisite) is unchanged. `archetype_losses.ron` says the heir's `sorcerer/bloodline` is replaced by it.
- **Vivisectionist's Extra Discovery:** new feat `extra-discovery-vivisectionist` (`ExtraVivsectionistDiscoverySelection` 10287e7b…, up to 3
  times) with the selection `alchemist/vivisectionist/extra-discovery` (the game's 25, `needs_slot_of` the medical discovery,
  `option_levels: ["alchemist"]`); it is in the general and dragon feat lists. `alchemist/extra-discovery` lost the medical discovery from
  `needs_slot_of` and holds the game's 28 (`ExtraDiscoverySelection` 537965879…); it still reaches an Underground Chemist until 264. The feat
  `extra-discovery`'s prerequisite is left as it was (264's `known.txt` line), so a Vivisectionist is still offered it and gets no slot.
- **Angel:** `angel/greater-sword-of-heaven` lists the game's 12: its 3, eight of the improved list's (shared) and the new option
  `angel/greater-sword-of-heaven/anti-demon` (`AngelSwordAntiDemonFeature` 249ff73e…). None of the moved options has a gate to carry: the game's
  only prerequisite among the 12 is Heavenly Host's Depersonalize, which the option already holds.

## Written by the writer, and by hand

`game-check --write option-lists --only aeon/enforcing-gaze,aeon/enforcing-gaze-2,aeon/enforcing-gaze-3,witch/elemental-witch/energy,
sorcerer/nine-tailed-heir/bloodline,alchemist/vivisectionist/extra-discovery,angel/greater-sword-of-heaven,alchemist/extra-discovery`
wrote nine fields in seven files (the two new Aeon lists, the first's cut, the heir's 21, both Extra Discovery lists, the angel's 8 shared).
The writer skips a selection with no list, so the heir's, the Vivisectionist's and the alchemist's Extra Discovery were seeded with one shared
option first. The Elemental Witch's four options were written by hand and matched the game's list as it stood.

## Left in `known.txt`

- 264: `extra-discovery-vivisectionist` (the game needs `VivsectionistDiscoverySelection`; the planner's names the class level and archetype).
- 278: the prerequisites of the Elemental Witch's acid, electricity and fire rays (the game: `WitchElementsPatronProgression`; the Elemental
  Witch's patron is a plain feature in the planner, so nothing can name it yet).
- Removed: 293's `aeon/enforcing-gaze` and the mythic ranks line, 266's `angel/greater-sword-of-heaven`. The slice expected a 263 pool line for
  `ExtraVivsectionistDiscoverySelection`; `known.txt` held none.

## Tests

`importer_lists::the_aeons_three_gaze_lists_are_the_games`, `importer_lists::the_elemental_witch_picks_one_of_four_rays`,
`importer_lists::the_heir_the_vivisectionists_extra_discovery_and_the_angel_have_the_games_lists`.
