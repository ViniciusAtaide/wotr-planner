# Ticket 244: the game data laid out by blueprint

## The layout

- One file per entry, in a folder by kind, named by its key: `features/sneak-attack.ron`, `archetypes/rogue/thug.ron`, `options/rogue/rogue-talents/fast-stealth.ron`. `libs/game-data/README.md`, Layout, lists the folders.
- No entry holds another. A class's, archetype's, path's and option's rows name the features and selections they grant by key, and a selection's `options` are the keys of the options it defines (each option is one selection's own; `shared_options` are others').
- An archetype names its `class`, a subrace its `race`.
- `planner/<folder>/` holds the 149 entries with no game blueprint (ticket 237's Answer lists them), each as `(why: "...", entry: ...)`: the reason, then the entry.
  - The 64 bloodline-spell placeholders say "the held bloodline's spell at this spell level". The game has one blueprint per bloodline. Turning them into that rule is an engine change; it is not done here.
- The move was made by a script kept outside the repo. Each entry keeps its text and comments.
  - The files' header comments are now in the README.
  - The Crusader's bonus-feat lists each carry the note that stood over them.
- The spells' "page and level heading" lived only in their old file paths (`spells/<page>/level-<n>.ron`). It went with them. The guide link still names the page.

## One entry per game feature

Since ticket 244 a key says nothing of an owner. Keys made before keep their prefix; a collapsed one has none.

- **What was collapsed:** 68 groups of features that copied one blueprint, 351 copies in all. Each group is now one entry with an owner-less key: `sneak-attack`, `uncanny-dodge`, `improved-uncanny-dodge`, `evasion`, the casters' cantrips, the magus's arcane weapon and armour steps, the shifter's claws, the kineticist's metakinesis, the skald's borrowed bard features, the paths' `hard-to-kill` and `bypass-epic-damage-reduction`, and others.
  - Names a class shares keep its distinction: `wizard-cantrips`, `witch-cantrips`, `shaman-orisons`.
  - The `UncannyDodge` talent copy is `uncanny-dodge-talent`, apart from `UncannyDodgeChecker`'s `uncanny-dodge`.
- **Each entry's text:** it keeps the base class's summary and guide link. A leading "From Nth level," is dropped where the holders reach it at different levels.
  - The copies' archetype-specific words went: the Hospitaler's fewer smites, the Reanimator's 1d4 bombs, the Angelfire Apostle's positive-only channel.
  - In the game those are other features (`SmiteEvilAdditionalUse` and the like), which the class tickets model.
  - The rows still grant the steps where they did.
- **References:** every reference follows the new key. A prerequisite's repeated child drops, so the Arcane Trickster asks `AtLeast(2, [Accomplished Sneak Attacker, Sneak Attack, Sneak Attack 2])`.
- **Archetype rows:** 149 archetype rows became their class's rows and went.
- **`archetype_losses.ron`:** of its losses "replaced by its own copy", 184 went, and the Psychokineticist's loss of Elemental Overflow with them. 7 now name the levels the archetype doesn't have the feature at (`at`), because the archetype gets the one feature at other levels.
- **Left apart, two entries holding one blueprint:**
  - **The Martyr's Stigmata steps,** the game's `BardMovePerformance` and `BardSwiftPerformance`. Extra Performance, Lingering Performance and Discordant Voice name the bard's as a stand-in for `BardicPerformanceResourcePrerequisite` (tickets 263 and 264). Collapsed first, a Martyr from 7 would take Extra Performance and Discordant Voice, which the game refuses.
  - **One game feature the planner splits into its powers, granted apart:** `EnchantmentSchoolBaseFeature`, `NecromancySchoolBaseFeature`, `ElemenetalRampagerSelection`.
  - **The combined wild shapes,** which hold one of two blueprints: Leopard with Small Elemental, Smilodon, Bear.
- **Options and features that share a blueprint** (decision 8, 39 pairs) stay two entries. The engine keeps options and features as two kinds, apart in picks, links, bonuses and prerequisites, and merging them is more than a small change.

## Who holds what

The sheet's holder of a feature or a selection is what grants it by its rows, not its key's prefix (decision 7). The holders are classes, archetypes, paths and options, in that order (`Catalog::feature_holders`).

- A picker's reason keeps the alternatives that one of the character's classes or archetypes grants.
- A name told apart reads its first holder: "Animal Companion (Cavalier)".

## Behaviour toward the game

- A prerequisite reads Sneak Attack summed across classes, as the game's one `SneakAttack` 9b9eac6709e1c084cb18c3a366e0ec87 (`Ranks` 20, `AddStatBonus` SneakAttack 1 per rank). The Arcane Trickster's `PrerequisiteFullStatValue` SneakAttack 2 (ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3) is met by a rogue 1 and Vivisectionist 1.
  - Test: `collapsed_features::the_arcane_trickster_reads_sneak_attack_summed_across_classes`.
  - The corpus has no Build this changes.
- A Possessed Oracle's Powerless Prophecy grants the one Uncanny Dodge (`UncannyDodgeChecker`). With a class that grants it too, the planner counts a second rank. The game's checker gives Improved Uncanny Dodge then, which the planner doesn't model (an inference from the blueprint's name, not read in the code).

## Not done here (ticket 244's Answer lists them)

- The selections the game keeps per class.
- The splits of Blessing and Second Blessing and of the Aeon's gazes.
- The 11 selection groups that differ, the 3 ranger archery feat-list pairs, and the 2 option pairs.
- The oracle mystery and revelation copies.
- The planner-made entries turned into rules.
- The domains are ticket 294's.
