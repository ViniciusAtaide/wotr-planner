# 308f: the barbarian's, witch's, shaman's, kineticist's, aeon's and Hellknight's option prerequisites, as the game's

## What the data says now

- **69 options** hold the game's prerequisites, written by `game-check --write prerequisites --only 'selection option
  barbarian/*,selection option witch/*,selection option shaman/*,selection option kineticist/*,selection option
  aeon/*,selection option hellknight/*,selection option loremaster/*,selection option angel/*,selection option
  azata/*,selection option demon/*,selection option lich/*,selection option trickster/*,selection option
  gold-dragon/*'` (308a's writer, 308g's allowlist, 308h's carrier guard). The run wrote 75; six were put back
  (below). The other families named (Loremaster, angel, azata, demon, lich, trickster, Gold Dragon) had nothing the
  writer could write.
- **What moved, by pattern** (each toward the game):
  - The rage powers' level gates read barbarian, skald or shifter levels or a Primalist bloodrager's (their
    PrerequisiteClassLevel and PrerequisiteArchetypeLevel, Group Any; RenewedVigorFeature
    aad496a7c9bee224cbd01a3cf8e42061 and the rest). Protect Vitals and Increased Damage Reduction name no shifter, and
    Increased Damage Reduction refuses the Inciter (PrerequisiteNoArchetype ProvocateurArchetype
    a1f5ab7eb78942f6889304acbd92e945). Come and Get Me needs 12 (it had no gate).
  - **The totems.** The plain and Greater totems need the one before them and their level (6, 10), and lost the 72
    `HasNot`s the planner gave them: their blueprints hold no PrerequisiteNoFeature (BeastTotemFeature
    c085888db293f2741b881cc989a2ab14, BeastTotemGreaterFeature 602d2684add008b4ba748af95c3f8b2e and the other six). The
    game keeps one line per character with the Lesser totems: each refuses the other three Lessers
    (BeastTotemLesserFeature d99dfc9238a8a6646b32be09057c1729's PrerequisiteNoFeature of FiendTotemLesserFeature,
    CelestialTotemLesserFeature and DaemonTotemLesserFeature, and SpiritTotemLesserFeature
    bd63d3ea0e030cf4c930db9ba5eaf931, which no entry maps). The four Lessers stay as they were (307, below), with the
    planner's refusal of the other lines' nine powers, so the legal picks are unchanged: a second line's Lesser is
    refused, and its plain and Greater powers need that Lesser. **Inference:** no planner row grants a Lesser outside
    a pick, so a held plain or Greater totem means its Lesser is held. The Celestial, Daemon and Fiend plain and
    Greater totems had no level gate and no Lesser before; the Fiend Greater accepts the Demon Dancer's Fiend Totem
    feature (one blueprint, FiendTotemFeature ce449404eeb4a7c499fbe0248056174f).
  - The Instinctual Warrior's Purity of Body needs barbarian 5 and Diamond Soul 14 (they asked 8 and 16); Deflect
    Arrows needs Dex 13 and barbarian 4 beside Improved Unarmed Strike (RageDeflectArrowsFeature
    d971bbb15f354621ac37fd51d0ab9044).
  - Infusions: Kinetic Blade and Extended Range name no level; Extended Range, Spindle, Wall and Exploding Arrows need
    the element selection (ElementalFocusSelection 1f3a15a3ae8a5524ab8b97f469bf4e3d: the kineticist's or the Blood
    Kineticist's), which the Kinetic Knight doesn't hold (KineticKnightArchetype 7d61d9b2250260a45b18c5634524a8fb
    removes it at 1); Exploding Arrows needs 3, Rending Arrows 11; Grappling one of Cloud, Deadly Earth or Wall.
  - Wild talents: Skill Focus, Skilled Kineticist and Elemental Whispers name no level (the slot's level gates them);
    Flame Shield needs kineticist 10 and Searing Flesh, not the fire element; Jagged Flesh Flesh of Stone, not the earth
    element; Healing Burst and Kinetic Revivification Kinetic Healer, not the water element (a list of one, which the
    reason reads "Needs 1 more of: Kinetic Healer").
  - The witch's and the shaman's Iceplant refuse each other; Intimidating Display refuses Dazzling Display.
  - The Prophet of Pestilence's plague mutations need the archetype's shaman level (PrerequisiteArchetypeLevel
    ProphetOfPestilenceArchetype bcd758a75fb54651a7f668fe2661a307), and those naming several damages need one of them,
    not all.
  - The Aeon's nine later gazes name no rank: the rank-5 and rank-8 gaze lists gate them.
  - Pentamic Faith refuses itself and the Hellknight's domain selection of its blueprint (`hellknight/pentamic-faith`),
    and needs the Order of the Godclaw.

## Put back: six entries the run wrote (decision mine)

- **The Elemental Witch's acid, electricity and fire rays** (ElementalWitchAcidRayFeature
  8006c9d043724f34b2745b3e5a5c2ef1 and the two others) need WitchElementsPatronProgression
  67e85f52b1f020847aaa738d8999d4cd. The game's Elemental Witch picks it in ElementalWitchPatronSelection
  3172b6960c774e19ad029c5e4a96d3e4; the planner holds that selection as a feature
  (`witch/elemental-witch/elemental-patron`) and has no patron slot for the archetype, so `witch/patron/elements` is
  never held and the written rays were refused to every Elemental Witch
  (`importer_lists::the_elemental_witch_picks_one_of_four_rays`). They stay 278's (its rows). The writer's guards
  don't see it: an added requirement isn't a narrowing, and the patron is a pick, not a grant. So a run naming them
  writes them again.
- **The air, earth and fire elements** (ElementalFocusAir 2bd0d44953a536f489082534c48f8e31 and the two others) refuse
  the Blood Kineticist (PrerequisiteNoArchetype BloodKineticistArchetype 365b50dba54efb74fa24c07e9b7a838c). The
  planner's Expanded Element offers the same entries for the game's second and third elements (SecondaryElementAir
  659c39542b728c04b83e969c834782a9, …), which refuse no archetype, so the written elements refused a Blood
  Kineticist's expanded air at 15, which the game allows (the corpus's `kineticist kineticist/blood-kineticist`,
  `talent blood`, `t59 blood-kineticist 20 / filled`). Their allowlist lines are gone, so the writer leaves them whole;
  `known.txt` names 306t (the second elements).

## Left whole, with their owner

`tools/game-check/known.txt` names each.

| Owner | Entries |
| --- | --- |
| 307 | the four Lesser totems (SpiritTotemLesserFeature), Life-Bonding Friendship (LifeBondingFriendshipFeature 5b63a6690ebbaa4459df3e04fe094cbd) |
| 270 | 22 infusions and Fire's Fury: a blast the vocabulary can't hold (307's decision 3) |
| 321 | 17 shaman hexes: a spirit's wandering feature (307's decision 3) |
| 306t | 11 wild talents: a second or third element, or the Kinetic Knight's focus; the three elements above |
| 283 | the Hexcrafter's 13 major and grand hexes (its rows) |
| 278 | Battle Master (the Wildland Shaman's rows); the Elemental Witch's three rays |
| 266 | the Gold Dragon's four breaths (hand prerequisites), the Lich's two skeletal upgrades (mythic class levels) |
| 287 | the Loremaster's combat feat (LoremasterTricksterCombatFeatSelection, 306o; a condition tree, 308) |
| 308 | Expanded Defense (PrerequisiteSelectionPossible; the check doesn't read it, so no line) |

## Corpus

- Three generators follow the game, the Build count held at 2,157: `talent fire 20` takes Flame Shield at 14 and Skill
  Focus at 6 (they swapped); `t65 celestial barbarian 6` takes Celestial Totem at 6, not 4; `t66 each power` and `t66
  deflect arrows only, extra rage power` have Dex 13.
- 63 links of 306's base are refused, each for a pick above (`tools/corpus/tests/links/306-refused.txt`, six blocks).

## Tests

`libs/seeds/tests/game_data/barbarian_witch_shaman_prerequisites.rs`: one test per pattern above. Un-ignored:
`instinctual_rage_powers::the_instinctual_warriors_monk_powers_need_the_levels_and_dex_their_blueprints_name`,
`skald_rage_powers::a_rage_powers_level_gate_reads_barbarian_skald_primalist_and_shifter_levels`,
`wild_talents::flame_shield_needs_kineticist_10_and_searing_flesh`,
`infusions::a_kinetic_knight_is_refused_the_infusions_that_need_the_elemental_focus_selection`. The totems' exclusion
is `report_gaps::the_totem_rage_powers_are_exclusive_one_line_per_character`.
