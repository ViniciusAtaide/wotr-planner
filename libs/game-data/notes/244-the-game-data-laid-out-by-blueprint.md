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
- **Each entry's text:** a summary true for every holder, and the guide link of the most general section. A leading "From Nth level," is dropped where the holders reach it at different levels.
  - The collapse first kept the base class's summary, which was false for some holders. The review rewrote each group whose copies' summaries differed in substance: a number that differs by holder reads "as the class's rows give it", or is named per holder where short. See "Summaries rewritten".
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
- A name told apart reads its holder (review). For a Build, the class taken whose rows have reached the feature (its archetype when only the archetype's rows grant it), else the path or an option held that grants it: a Rogue 4's Uncanny Dodge is "(Rogue)", a Barbarian 2's "(Barbarian)". Without a Build (search, describe, a prerequisite's text), the one holder there is, a class with its archetypes counting as the class: "Damage Reduction (Barbarian)". A feature several classes, paths or options grant is named alone ("Uncanny Dodge"), never for a holder the reader may not have.
  - A prerequisite's text names no Build's holder: any holder's copy meets it.
  - A selection's holder in a told-apart option ("Arcane (Sorcerer Bloodline)") still reads its first holder; no selection was collapsed, so no case is known to read wrong.

## Behaviour toward the game

- A prerequisite reads Sneak Attack summed across classes, as the game's one `SneakAttack` 9b9eac6709e1c084cb18c3a366e0ec87 (`Ranks` 20, `AddStatBonus` SneakAttack 1 per rank). The Arcane Trickster's `PrerequisiteFullStatValue` SneakAttack 2 (ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3) is met by a rogue 1 and Vivisectionist 1.
  - Test: `collapsed_features::the_arcane_trickster_reads_sneak_attack_summed_across_classes`.
  - The corpus has no Build this changes.
- A Possessed Oracle's Powerless Prophecy grants the one Uncanny Dodge (`UncannyDodgeChecker`). With a class that grants it too, the planner counts a second rank. The game's checker gives Improved Uncanny Dodge then, which the planner doesn't model (an inference from the blueprint's name, not read in the code).

## Summaries rewritten (review)

Of the 68 collapsed groups, 30 had copies whose summaries differed. Rewritten, each checked against its blueprint:
- **`sneak-attack`** (SneakAttack 9b9eac6709e1c084cb18c3a366e0ec87, `Ranks` 20): 1d6 per rank, the ranks from each class's rows and summed; the Rogue's every odd level named.
- **`armor-training`** (ArmorTraining 3c380607706f209499d951b29d3c44f3, `Ranks` 5, `MaxDexBonusIncrease` and `ArmorCheckPenaltyIncrease` 1 per rank): per rank, at most 5; heavy armour's speed from rank 2 (`ArmorSpeedPenaltyRemoval.OnTurnOn`, `GetRank() >= 2`). The fighter's "every 4 levels" and "heavy from 7th" were false for the Hellknight (2, 5, 8).
- **`channel-positive-energy`**: one game feature, ChannelEnergyFeature a79013ff4bcd4864cb669622a29ddafb (positive channel, `PrerequisiteFeature` ChannelPositiveAllowed). AngelfireApostleArchetype 857bc9fadf70f294795a9cba974a48b8 and PriestOfBalanceArchetype a4560e3fb5d247d68fb1a2738fcc0855 both add it at 1; the Priest also adds ChannelNegativeFeature 3adb2c906e031ee41a01bfc1d5fb7eea, the planner's `cleric/priest-of-balance/channel-negative-energy`. "Only positive, never negative" was false for the Priest. Guide link: `cleric#channel-energy`.
- **`elemental-overflow`**: kept collapsed. ElementalOverflowBonusFeature 2496916d8465dbb4b9ddeafdf28c67d8 is one blueprint: ElementalOverflowProgression 86beb0391653faf43aec60d5ec05b538 and the Psychokineticist's MentalOverflowProgression 42b8ce9dfb087c74db531bad426ad2f3 both grant it at 3. Only their 6th-level effects differ (ElementalOverflowEffectFeature 74e4520801f18ae42af3acf261ac7e90, MentalOverflowEffectFeature 5441bfcff6f1c4846ae103ca1e500619), which the planner doesn't model; the summary names both. The Psychokineticist's loss stays gone.
- **`bomb`** (AlchemistBombsFeature, damage by rank): the dice as the class's rows give them, the Alchemist's and the Reanimator's named.
- **`smite-evil`** (SmiteEvilFeature: the ability and SmiteEvilResource): uses as the class's rows give them, the Paladin's and the Hospitaler's named.
- **`studied-target`** (SlayerStudyTargetFeature, `Ranks` 10): rising as the class's rows give it, the Slayer's steps named; "slayer ability DCs" became "class ability DCs" for the Divine Scion.
- **`metakinesis-empowered`, `-maximized`, `-quicken`**: "accept Burn" was false for the Overwhelming Soul, who can't; it now says the Soul cuts the cost away with Gather Power.
- **`wild-shape-wolf`**: the druid's "1/day plus 1 every two levels (8 at 18th)" was false for the Feyspeaker (wolf from 6th); the uses now "rise with level".
- **`advanced-talents`, `major-hexes`, `grand-hexes`**: "can choose advanced rogue talents" or "hexes" now says the pick it opens per holder, with the Vivisectionist's three discoveries and the Sylvan Trickster's Fey Tricks.
- **`heavenly-fire`**: the guide link is the bloodline's section, `character-creation#heavenly-fire`, not the Empyreal Sorcerer's.

Left as they were, true for every holder: `arcane-reservoir` (wording only), `banner`, `cavalier-mount`, `celestial-ascension`, `evasion`, `improved-evasion`, `improved-uncanny-dodge`, `quarry`, `resist-natures-lure`, `skald-bardic-knowledge`, `lore-master`, `well-versed`, `studied-target-swift`, `uncanny-dodge`, `uncanny-dodge-talent`. The other 7 groups whose texts differed were never collapsed (the Stigmata steps, the school powers, Elemental Rampager, the combined wild shapes).

## Not done here (ticket 244's Answer lists them)

- The selections the game keeps per class.
- The splits of Blessing and Second Blessing and of the Aeon's gazes.
- The 11 selection groups that differ, the 3 ranger archery feat-list pairs, and the 2 option pairs.
- The oracle mystery and revelation copies.
- The planner-made entries turned into rules.
- The domains are ticket 294's.
