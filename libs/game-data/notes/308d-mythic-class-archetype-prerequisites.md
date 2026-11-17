# 308d: the mythic abilities', classes' and archetypes' prerequisites, as the game's

## What the data says now

- **22 entries** hold the game's prerequisites, written by `game-check --write prerequisites --only <24 entries>` (308a's
  writer; 308g's narrowing allowlist): 10 mythic abilities, one mythic feat, 7 prestige classes, the oracle and three
  archetypes. The writer leaves an entry untouched when its compiled tree already equals the game's (Abundant Smite).
  Defensive Study and the Stalwart Defender were written, then put back as they were (below).
- **What moved, by pattern** (each toward the game):
  - The prestige classes written name no character level (none of the game's classes holds a
    PrerequisiteCharacterLevel). The Duelist needs Dodge (DuelistClass 4e0ea99612ae87a499c7fb0588e31828), the Mystic
    Theurge 3 ranks in Knowledge (Arcana) and Lore (Religion), not 5, and not the Magic Deceiver
    (MysticTheurgeClass 0920ea7e4fd7a404282e3d8b0ac41838). The Signifer's feats are an `AtLeast(1, …)`
    (PrerequisiteFeaturesFromList) and the Winter Witch's the Winter Child's Ice in the Blood, the frost spirit, the
    winter patron or the Winter Child's domain.
  - The Magic Deceiver refuses Loremaster and Mystic Theurge levels (MagicDeceiver 5c77110cd0414e7eb4c2e485659c9a46);
    the oracle refuses Apsu (OracleClass 20ce9bf8af32bee4c8557a045ab499b1); the Dark Elementalist is evil and the
    Rageshaper anything but lawful (their PrerequisiteAlignment).
  - Class levels: Any Means Necessary needs Hellknight 7, not 1 (FreeFromEvil a0a814897cd33ed4ea6ffca4b0e1197a);
    Kinetic Overcharge and Over-Infused Blasts kineticist 1 (KineticOvercharge 4236ca275c6fa9b4a9945b2645262616,
    DoubleKineticistInfusions 7410cc35ae6aa9d4a82f1d23c081a1ac). Over-Infused Blasts had none, so a non-kineticist
    could take it; Witch Wandering Hex needs witch 1, a Sylvan Trickster rogue 2 or a Hexcrafter magus 4, not a hex.
  - Selections in place of their options: Second Mystery, Second Bloodrager Bloodline, Beneficial Curse and Impossible
    Domain need the selection held (whatever is picked in it), and only the game's: a Reformed Fiend's or a Mixed-Blood
    Rager's bloodline, an Enlightened Philosopher's or a Divine Herbalist's mystery, and a Living Deity's curse are
    other blueprints, so they're refused. Beneficial Curse has no PrerequisiteNoFeature of itself, so it repeats while a
    curse is left (its `known.txt` line from 303 went).
  - Impossible Domain (ExtraDomain 213a8480d22206b45acbfa0619ca5aaf): DomainsSelection, DivineHunterDomainsSelection or
    FakeDivineSparkSelection. A Pentamic Faith Hellknight and a Faith Hunter are refused, and a Living Deity before
    arcanist 20 (the user, 2026-11-17: "Yes, as the game"; `research/308/decisions.md` item 1).
  - Limitless Rage counts the Elemental Rampager's
    rampage and the Rageshaper's devastating form beside the three rages (LimitlessRage
    5cb58e6e406525342842a073fb70d068). Weapon Focus (mythic) accepts the rogue talent's Weapon Focus (one blueprint,
    WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e).
- **The engine reads a shared own NoFeature as once** (`rules::Repeat::derive`): Second Mystery's and Second Bloodrager
  Bloodline's own PrerequisiteNoFeature names a blueprint each shares with a selection of the planner's
  (`oracle/second-mystery`, `bloodrager/second-bloodline`), so it is written `Not(Any(itself, the selection))`, and the
  header still reads it as taken once, and its reason names each held entry once ("Not with Second Mystery").
- **Older links:** `support::without_mythic_abilities_the_game_refuses_since_308d` drops exactly Over-Infused Blasts
  without kineticist 1 and Impossible Domain without a domain selection from an older link's Build, for
  `first_ascension` and `ranks_and_pools` (`targeted living-deity-8`'s Impossible Domain at arcanist 6).

## Left whole, with their owner

`tools/game-check/known.txt` names each. 265's mythic lines are superseded by 293.

| Owner | Entries |
| --- | --- |
| 307 | Abundant Bane, Abundant Ki Pool, Domain Zealot, Everlasting Judgement, Destiny Beyond Birth, Defensive Study, Stalwart Defender (carriers) |
| 306x | Bloodline Ascendance, Second Bloodline (the Seeker's bloodline; their Sage, Sylvan and Empyreal ones are 316's) |
| 306p | School Tolerance |
| 306o | the magus (EldritchScionClass) |
| 308 | Second Spirit (a condition tree, decision 3) |
| 315 | the nine Mythic Armor Focus feats, Loremaster, Student of War (per-value features) |
| 314 | Mythic Sneak Attack |
| 313 | Aldori Swordlord (a weapon proficiency) |
| 174 | Mythical Beast |
| 265 | Greater Enduring Spells, held for ticket 208 (another session's) |
| 267 | Hag of Gyronna, Mantis Zealot, Feral Champion, Angelfire Apostle, Priest of Balance, cleric, inquisitor, warpriest, paladin |
| 287 | Arcane Trickster, Dragon Disciple (hand prerequisites) |

**The Hag of Gyronna stays as it was** (decision mine, 308d): DarkSisterArchetype 411fa458481e44f0855d47a19358874b names
only an alignment, but every other deity's feature and AtheismFeature refuse it (PrerequisiteNoArchetype; AsmodeusFeature
a3a5ccc9c670e6f4ca4a686d23b89900), which the planner's deities don't hold until 267. Writing it would have let a Hag of
Gyronna follow any deity. Its two allowlist lines are removed, so the writer leaves it whole (`-> 267`).

**Defensive Study and the Stalwart Defender stay as they were** (308d's review and fix round): the game's prerequisite
names a blueprint another of the game's features grants, which the planner's entry doesn't (307's carriers), so writing
it refused a character the game allows:

- DefensiveStudy ec041312142ba1f48a0044521f68fc0a names SlayerStudyTargetFeature 09bdd9445ac38044389476689ae8d5a1, which
  SanctifiedSlayerStudyTargetFeature 6d9d3fbc30564e64d966dba27cd6357a (AddFacts) and TandemExecutionerStudyTargetFeature
  3dac0ae65dec4b619c035df07751b59d (AddFeatureOnClassLevel HunterClass 1) grant. The base's `Any` of the four studied
  targets is the game's in effect. Its four allowlist lines stay out: their evidence missed the grants.
- StalwartDefenderClass d5917881586ff1d4d96d5b7cebda9464 names Endurance 54ee847996c25cd4ba8773d7b8555174, which
  UnstoppableDwarf 109dca82e31a4b1093c501f8914ca1a8 grants (AddFacts); `subraces/dwarf/unstoppable-dwarf.ron` doesn't.
  The writer doesn't count an added requirement as a narrowing, so a run naming `class stalwart-defender` writes it
  again: leave it out until 307.

## Tests

- `libs/seeds/tests/game_data/mythic_class_prerequisites.rs`: one test per pattern above.
- `option_follow_ups::a_hellknight_3_with_pentamic_faiths_domain_cant_take_impossible_domain` (un-ignored),
  `a_living_deity_below_arcanist_20_cant_take_impossible_domain`, `a_living_deity_with_impossible_domain_holds_a_third_domain`
  (its MR 1 now at 20).
- `repeats_from_the_header::{the_mythic_entries_repeat_as_their_headers_say, the_selections_with_their_own_no_feature_are_taken_once}`,
  `mythic::beneficial_curse_gives_a_slot_of_the_oracles_curses`,
  `negated_prerequisites::every_negated_prerequisite_in_the_data_reads_as_what_rules_the_option_out` (the deity and
  the `Any` kinds), and the rules tests `repeats::a_selection_not_of_an_any_naming_itself_is_taken_once` and
  `labels::a_negated_condition_says_what_rules_the_option_out`.
