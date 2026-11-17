# 308c: the feats' prerequisites, as the game's

## What the data says now

- **70 feats** hold the game's prerequisites, written by `game-check --write prerequisites --only 'feat *'` (308a's writer):
  one `Requires` per game component (`PrerequisiteFeature`, `PrerequisiteStatValue`, `PrerequisiteClassLevel`, …), its
  `group` from the component's `Group`, a blueprint several entries share as an `Any` of `Has` each, then the Extra
  feats' cap. The writer leaves a feat untouched when its compiled tree already equals the game's.
- **Two hand-written** (decision 2 of `.scratch/wotr-planner/research/308/decisions.md`, the main session, 2026-11-17):
  - `extra-arcana`: ExtraArcanaSelection 00727883edf145e2a6bce9ad176ecfd8 needs MagusArcanaSelection
    e9dc4dfc73eaaf94aae27e0ed6cc9ada (`magus/magus-arcana`), HexcrafterMagusHexArcanaSelection
    ad6b9cecb5286d841a66e23cea3ef7bf (`magus/hexcrafter/hex-arcana`) or EldritchMagusArcanaSelection
    d4b54d9db4932454ab2899f931c2042c, Group Any.
  - `extra-hex-shaman`: ExtraShamanHexSelection d0b4c8245d504b8c9c6d3fccc1f8c5b6 needs ShamanHexSelection
    4223fe18c75d4d14787af196a04e14e7 (`shaman/hex`) or WinterWitchShamanHexSelection 4e760c5034fafb2438993d8a192150b9.

  **Inference:** the Eldritch Scion's and the Winter Witch's selections are left out. Neither has an entry, and no Build
  can hold either today: the Winter Witch's rows grant no hex selection, and the Eldritch Scion holds
  `magus/magus-arcana`, which the list names too. So leaving them out refuses nothing a Build holds. Each `known.txt`
  line narrows to the member it leaves out, `-> 306o`. 306aa's `extra-hex` (the witch's) is narrowed the same way.
- **What moved, by pattern** (each toward the game):
  - Repeated tier leaves: Cleave's chain, Bull Rush, Sunder Armor and their Greater forms need Strength 13 themselves.
    The manoeuvre feats need Intelligence 13 beside `Any(Combat Expertise, Kinetic Warrior)`.
  - Races: Ironguts is a dwarf's or a half-orc's, Murmurs of Earth an oread's, Blood Feaster a dhampir's.
  - Base attack bonus: Ranged Feint needs BAB 2. Bashing Finish adds Shield Bash, Two-Weapon Fighting and Shield
    Proficiency to its BAB 11 and Shield Master.
  - The Extra feats name the selection, held from its level whatever is picked in it, not its options:
    - Extra Rage Power: the barbarian's, the Instinctual Warrior's or the skald's. A Battle Scion with none is refused.
    - Extra Arcanist Exploit, Extra Mercy, Extra Discovery (Vivisectionist), Extra Wild Talent (`kineticist/wild-talents`,
      not kineticist 2).
    - Extra Bombs and Extra Lay On Hands name the Arcane Bomber's bombs and the Divine Herbalist's Healer's Way.
  - Widenings:
    - Dazzling Display, Shatter Defenses and both Penetrating Strikes accept the rogue talent's Weapon Focus (one
      blueprint, WeaponFocus 1e1f627d26ad36f43bbd26cc2bf8ac7e).
    - Arcane Strike is `CasterType(Arcane)` (PrerequisiteCasterType), so a bloodrager 1 qualifies.
    - Natural Charmer and Perfect Strike read the full score (PrerequisiteFullStatValue).
  - Removals that loosen:
    - Rapid Shot has no prerequisite but Intelligence 3, which is omitted.
    - Manyshot needs Rapid Shot and BAB 6 only.
    - Critical Mastery counts two of five critical feats, not Flaying or Stunning Critical.
    - Pummeling Style names the monk's flurry only, which the Sohei, Zen Archer and Quarterstaff Master remove.
    - Advanced Weapon Training needs the fighter's Weapon Training and fighter 5, not the Sohei's or the Disciple of the
      Pike's.
  - Proficiency: Light Armor and Shield Proficiency refuse themselves (their own `PrerequisiteNotProficient`), and Spell
    Chain needs Spell Penetration (SpellChain d8d641edc9a74a64809f56feff9495b6).

## Left whole (the writer's gaps), with their owner

`tools/game-check/known.txt`, section "263 and 264", names each feat's owner. 263 and 264 are superseded by 293.

| Owner | Feats |
| --- | --- |
| 307 | carriers: Diehard, Discordant Voice, Lingering Performance, Extra Performance, Extra Ki, Extra Slayer Talent, Natural Spell, Raking Claws, Shifter's Edge, Selective Channel, Energized Wild Shape, Frightful Shape, Shifter's Rush, Improved Elven Immunities, Steel Soul, Wings, Spell Specialization (and 317) |
| 313 | weapon categories: Fencing Grace, Slashing Grace, Point-Blank Master |
| 314 | Accomplished Sneak Attacker |
| 315 | the Trickster's critical feats (hand trees) |
| 174 | the three barding feats, Boon Companion |
| 306x | Disruptive |
| 308 | Extended Aspects (a condition tree, decision 3) |
| 267, 275 | Extra Channel, Favored Enemy Spellcasting (narrowings whose owners were already named by 308a) |

The allowlist `tools/game-check/prerequisite-narrowing.txt` lost every `feat` line: the entries it let through are
written, so the lines matched no narrowing.

## Not modelled

- The Hexcrafter gets the magus arcana at 3 in the game, but `archetypes/magus/hexcrafter.ron` row 3 lacks it (283).
  So a Hexcrafter 5 still can't take Extra Arcana; `a_hexcrafter_5_meets_extra_arcana` waits on 283.

## Tests

`libs/seeds/tests/game_data/feat_prerequisites.rs` covers one test per pattern above. Also:
- `class_skills::advanced_weapon_training_needs_the_fighters_weapon_training`
- `skald_rage_powers::extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection`
- `skald_rage_powers::a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot`
- `wild_talents::extra_wild_talent_needs_the_wild_talent_selection`
