# 302: nested selections

**What the data says now.** A feat, a mythic feat or a mythic ability that grants a pick names the selection it
grants (`selection`), as the game's Extra feats and mythic abilities are themselves `BlueprintFeatureSelection`s: a
pick of it opens one slot of that selection (`SelectFeature.Apply` adds a picked selection to the level-up's
selections, same source and level; WrathDecompiled 6ce4a6b). `Feat::extra_pick_of`, `MythicFeat::extra_pick` (and
`MythicExtraPick`) and `MythicAbility::selections` are gone. Every slot id, its numbering and every pick are as before.

- **The Extra feats** each grant a new selection carrying the feat's own blueprint (one GUID on two entries, as
  `oracle/second-mystery` and `cleric/impossible-domain` already had). Each names, in `needs_slot_of`, the selections
  its pick stood for: the planner's rule until 293, that the slot comes only to a character who had a slot of one of
  them first.

  | Feat | Selection | Game blueprint | Form |
  | --- | --- | --- | --- |
  | `extra-arcanist-exploit` | `arcanist/extra-arcanist-exploit` | `ExtraArcanistExploitSelection` 0a7065e13f4449fa81bfb33199bf7f6a | own list |
  | `extra-mercy` | `paladin/extra-mercy` | `ExtraMercySelection` 8a49abed5be9473da1e1fd1e2457562e | own list |
  | `extra-slayer-talent` | `slayer/extra-slayer-talent` | `ExtraSlayerTalentSelection` 53f7237f5b1447bb851ba68045a00e41 | own list |
  | `extra-wild-talent` | `kineticist/extra-wild-talent` | `ExtraWildTalentSelection` 0f73730cf1b44ee882671e55d5f6e471 | own list |
  | `extra-rage-power` | `barbarian/extra-rage-power` | `ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a | first had |
  | `extra-rogue-talent` | `rogue/extra-rogue-talent` | `ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12 | first had |
  | `extra-hex` | `witch/extra-hex` | `ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35 | first had |
  | `extra-arcana` | `magus/extra-arcana` | `ExtraArcanaSelection` 00727883edf145e2a6bce9ad176ecfd8 | first had |
  | `extra-discovery` | `alchemist/extra-discovery` | `ExtraDiscoverySelection` 537965879fc24ad3948aaffa7a1a3a66 | first had |

  An **own list** shares every option of the one selection the feat stood for, with that selection's name and header
  (`option_levels`, `band`), so an option taken through it has the level and the slot the label it had. A **first had**
  selection lists nothing: its slot is a slot of the first selection of its `needs_slot_of` the character had a slot
  of, as `extra_pick_of` gave; the game check skips it (293 writes the game's list). Extra Hex's names
  `witch/wandering-hex` too, so a character whose first hex slot was Witch Wandering Hex's keeps a hex slot.
- **The mythic abilities** grant one selection each: `oracle/beneficial-curse` (`BeneficialCurse`
  2dda67424ee8e0b4d83ef01a73ca6bff), `sorcerer/second-bloodline` (`SecondBloodline` 3cf2ab2c320b73347a7c21cf0d0995bd),
  `bloodrager/second-bloodline` (`SecondBloodragerBloodline` b7f62628915bdb14d8888c25da3fac56), `shaman/second-spirit`
  (`SecondSpirit` 2faa80662a56ab644aec2f875a68597f), each sharing today's list (the oracle's curses, the bloodlines, the
  spirits) with that selection's name and option levels, and the band of its blueprint's group
  (`SelectFeature.CalculatePriority`, WrathDecompiled 6ce4a6b: Group None, Features, but SecondSpirit's ShamanSpirit,
  Path; a rank's slots are filled in order, so the band moves no pick); `second-mystery` and `impossible-domain` keep naming `oracle/second-mystery`
  and `cleric/impossible-domain`. Witch Wandering Hex's is `planner/selections/witch/wandering-hex.ron`, a pick of a
  witch hex, with its reason: the game's `WitchWanderingHex` fb8490d1721bd5a4883c3b6559310351 is a plain feature
  (ticket 259).
- **The mythic feats**: Extra Feat grants `feats` (`BasicFeatSelection`; the game's `ExtraFeatMythicFeat`
  e10c4f18a6c8b4342afe6954bde0587b is a selection of its own whose list is 293's); Extra Mythic Ability grants
  `mythic/extra-mythic-ability` ("Mythic ability", band `MythicAbility`, `ExtraMythicAbilityMythicFeat`
  8a6a511c55e67d04db328cc49aaad2b8), listing every mythic ability, the 63 a rank's general pick offers (its
  `mythic_abilities`; the game's list is 293's). Its slot is a mythic ability slot of that selection.

**How.** A script outside the repo (`<scratchpad>/game/302/rewrite.py`, `python3 -I rewrite.py <data dir>
[<kind-map.tsv>]`), from the data at `1387bbeb`, not the game: it moves the 18 fields (9 feats, 7 mythic abilities,
2 mythic feats), writes the 15 new selections, copying each header from the selection it stands for (the mythic
abilities' bands set to the game's) and refusing a header field it doesn't know, and run again moves nothing. The seeds fixture's Second Bloodline was edited by hand.

**Decisions.**
- The main session (`game/302/decisions.md`): own lists where the game's equals the one selection's (Q1); new
  selections with the ability's blueprint and a copied header, Wandering Hex a planner one (Q2); `option_lists` stays
  until 304 (Q3); Extra Mythic Ability's own selection with today's list (Q4); `once_per_option` stays (Q5); refusing a
  selection with nothing left to pick is 303's (Q6).
- Here: Extra Rage Power is a first-had list, not an own list. Its pick stood for two selections, and a Battle Scion
  skald's slot was the Battle Prowess selection, which offers a teamwork option the barbarian's doesn't: an own list
  would change that offer (the game's `ExtraRagePowerSelection` lists the barbarian's 38 and closes the feat to a
  Battle Scion, ticket 264).
- Here: the own lists keep `needs_slot_of`, the planner's rule that a feat gives its slot only after a slot of its
  selection. The game gives the slot whenever the feat is taken; without the rule a Bloodseeker, Vanguard or Deliverer
  slayer taking Extra Slayer Talent before its first talent slot would gain a slot it lacks today. For mercy,
  arcanist exploit and wild talent the rule never binds (inference: their prerequisites need an option only that
  selection offers, or kineticist 2, which every kineticist archetype reaches with its wild talent slot).
- Here: the keys (`<class>/extra-<name>`, `<class>/second-…`, `mythic/extra-mythic-ability`) and Extra Feat naming
  `feats` rather than a copy of its 261.

**What it does not model.** The game's own lists of the Extra feats and mythic abilities where they differ from the
selection each stands for (293); Witch Wandering Hex's daily ability (259); refusing a granted selection with nothing
left to pick (303).

**Tests.** `libs/seeds/tests/game_data/nested_selections.rs`: each Extra feat grants a selection with its blueprint;
the own lists list the selection they stand for, with its header; the first-had ones name the selections they reach;
the mythic abilities name their selections; Extra Mythic Ability's lists every mythic ability and Extra Feat's is
`feats`; the chain Extra Mythic Ability, Second Bloodline, a bloodline at one rank; the category selections list the
feats of their category (301's review).
