# 308e: the rogue's, slayer's, alchemist's, magus's, arcanist's and small families' option prerequisites, as the game's

## What the data says now

- **54 options** hold the game's prerequisites, written by `game-check --write prerequisites --only 'selection option
  rogue/*,selection option slayer/*,selection option alchemist/*,selection option magus/*,selection option
  arcanist/*,selection option ranger/*,selection option hunter/*,selection option paladin/*,selection option
  monk/*,selection option stalwart-defender/*'` (308a's writer, 308g's allowlist, 308h's carrier guard). The run wrote
  59; the five `ranger/combat-style-feats-10/*` were put back (below).
- **What moved, by pattern** (each toward the game):
  - Uncanny Dodge (UncannyDodgeTalent ca5274d057152fa45b7527cad0927840) is a rogue talent of an Eldritch Scoundrel
    rogue, a bard, a skald, a slayer or a Feyform Shifter at 4; Improved Uncanny Dodge (ImprovedUncannyDodgeTalent
    e821c61b2711cea4cb993725b910e7e8) of an Eldritch Scoundrel rogue, a bard, a skald or a slayer at 12, with no
    Uncanny Dodge first. The user applied the game's in 276 (`tools/game-check/user-facts.txt`); the two summaries say
    so now.
  - The slayer's combat style talents are the game's RangerStyleSelection2, 6 and 10: the 2nd-level one only refuses
    itself and the ranger's style (no slayer level, no upper level), the 6th and 10th need the style before and ranger,
    slayer or Sanctified Slayer 6 or 10. A ranger therefore can't take the slayer's 2nd-level talent.
  - The Bloodseeker's nocturnal abilities need their slayer level (6 to 16); the magus arcana named by level need it,
    and Bane, Devoted, Enduring and Ghost Blade refuse the Armored Battlemage.
  - Archetypes refused: Cognatogen (Incense Synthesizer), Fast Bombs (Metamorph, Vivisectionist), Preserve Organs
    (Metamorph), Potent Magic (Phantasmal Mage), Mystery Seeker (Master of All), Survivalist (Master of All, Sylvan
    Trickster).
  - The mutagen discoveries accept the Mutation Warrior (its mutagen and fighter levels); Nauseating Flesh needs level 12
    of an alchemist, a Mutation Warrior or an Underground Chemist; Mummification an Underground Chemist's rogue 10,
    not its discovery feature.
  - Blinding Strike refuses Blinding Critical; Improved Evasion needs Evasion (the class feature or the rogue talent,
    one blueprint); Swift Consume needs arcanist 1, so the Arcane Enforcer is refused it.
  - Fatigued, Shaken and Sickened mercies and Sudden Speed have no prerequisite (their slots' levels gate them).
  - The Stalwart Defender's Fearless Defense and Increased Damage Reduction accept the Stonelord paladin 8 and 12.
    Clear Mind is the rage power's blueprint (ClearMindFeature 4a236ae23d5a4f76b3616ddfa337e724: barbarian, skald or
    shifter 8, or a Primalist bloodrager 8), which StalwartDefenderDefensivePowerSelection
    2cd91c501bda80b47ac2df0d51b02973 doesn't list (its `option lists` line is 287's). Writing it narrows a planner-only
    option toward "not offered" (decision mine).
  - The otherworldly companions name their alignments; the ranger's 6th-level style lists accept every planner entry of
    the 2nd-level blueprint (the Stormwalker's and the Nomad's bonus feats, the nested list).

## Left whole, with their owner

`tools/game-check/known.txt` names each.

| Owner | Entries |
| --- | --- |
| 307 | the 13 bomb discoveries (ExplosiveExpert 8a551fcca93446f0adfbe5e57e5bc636 grants the bomb feature, `explosives-expert` doesn't); 11 rogue talents that need Sneak Attack (TaintedSneakAttackFeature e6ce101a94ac9034b8b55c546e74b9dd grants it, `lich/lich-powers/tainted-sneak-attack` doesn't) |
| 313 | the rogue talent Weapon Focus (a weapon category) |
| 266 | Holy Water Jet (the mythic classes) |
| 267 | the Razmiri Infiltrator (deity features no entry maps), the Living Deity (its hand `Atheist` stands for the game's list of every deity's feature) |
| 293 | the ranger's 10th-level style lists (below) |

**The ranger's 10th-level style lists stay as they were** (308e, after the gold Enforcer became impossible).
RangerStyleMenacingSelection10 7d5bd07830a5e2c478852bf8b860db2c and the other four need the 6th-level list
(RangerStyleMenacingSelection6 a9998f05a0472bb4f8950304fe26ec0c), which the game holds once picked at 6. The planner
nests that list in `ranger/combat-style-feats-6` and stores only the feat (304's note), so neither the option
`ranger/combat-style-feats-6/menacing` nor the list `ranger/combat-style-feat/menacing-feats-6` is ever held, and the
written tree refused every 10th-level style feat the game allows. 304 left "opens the 10th-level style lists with the
6th-level one" to 293 (its decision Q4), so the lines stay 293's. Their five allowlist lines are gone, so the writer
leaves them whole as a narrowing (it prints `-> 308`).

## Gold

The Enforcer takes Uncanny Dodge as its slayer talent at 8 (SlayerTalentSelection6 43d1b15873e926848be2abf0ea3ad9a8
lists UncannyDodgeTalent, open from slayer 4), and `KNOWN_GAPS` loses its 173 entry.

## Tests

`libs/seeds/tests/game_data/rogue_slayer_alchemist_prerequisites.rs` covers one test per pattern above. Un-ignored:
`borrowed_selections::{the_vivisectionists_advanced_talents_meet_every_advanced_talent,
a_rogue_10_feyform_shifter_is_offered_the_advanced_talents, a_sanctified_slayer_8_may_take_the_first_combat_style_feat,
an_eldritch_scoundrel_2_isnt_offered_uncanny_dodge, an_eldritch_scoundrel_takes_improved_uncanny_dodge_from_rogue_12,
the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents, a_scoundrel_2_with_bard_2_isnt_offered_uncanny_dodge}`,
`follow_ups::a_slayer_may_take_the_first_combat_style_feat_at_6`,
`negated_prerequisites::a_slayer_6_who_skipped_it_takes_the_2nd_level_combat_style`,
`option_follow_ups::a_ranger_10_slayer_2_cant_take_the_2nd_level_combat_style`.
