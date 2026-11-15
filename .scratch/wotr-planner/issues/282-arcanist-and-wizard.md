# Arcanist and wizard

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [254](254-a-features-steps-follow-the-class-level.md), [260](260-greater-exploits-opens-the-exploit-picks.md) (it changes `arcanist.ron` and `wizard.ron` first)
Model: Opus to build: the Magic Deceiver's ways and the wizard's schools read from their blueprints.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `follow_ups.rs`, `archetype_losses.ron`; half B `mercies_and_exploits.rs`, `scaling_features.rs`) differ on `arcanist.ron` and `wizard.ron`:
- **The Magic Deceiver** (`MagicDeceiver` 5c77110cd0414e7eb4c2e485659c9a46): its ways' prerequisites exclude the deities the game names (the Living Deity's `MagicDeceiverWayOfLivingGodProgression` 464ffaf88140474a949618bec7955e78 and the others: `not <Deity>Feature` leaves the planner lacks); the archetype refuses Loremaster and Mystic Theurge levels; the Razmiri Infiltrator (`MagicDeceiverWayOfRazmiriInfiltratorProgression` cd96109ca30b45dcb6d7390945c8487d) is any non-good alignment (LN and CN too) and needs a deity (not the Lich's, not Camelia's Green Faith); the Living Deity's curse (`MysteryGiftFeatureCurseSelection` 4e7265c0ae1345db90d3375f4ced94cc) offers the nine `<Curse>CurseNoProgression` features, not the full curses; Inhibit Courage and Greatness's steps ([254](254-a-features-steps-follow-the-class-level.md)'s pass).
- **Exploits:** Holy Water Jet (`ArcanistExploitHolyWaterJetFeature` 3ff84c64d01075d4cbc3dc274c48a4bf) has no alignment prerequisite (it is refused to an Aeon, Demon, Devil, Lich, Trickster or Swarm-That-Walks mythic); Swift Consume (`ArcanistExploitSwiftConsumeFeature` eaec53cbf34042f4e9f0d5451f9529d1) needs arcanist level 1, so an Enforcer can't take it; the exploit selection's 22 options include the greater exploits ([260](260-greater-exploits-opens-the-exploit-picks.md)); the White Mage (`WhiteMageArchetype` 5f2fa2e0d1cf1ca428af7e24c443d868) removes the exploit at 1; Potent Magic's prerequisite.
- **The wizard** (`WizardProgression` 02f3049806dbf62459259ea8cae8f715): Detect Magic and cantrips at 1; the opposition school selection (`OppositionSchoolSelection` 6c29030e9fea36949877c43a6f94ff31) as a pick; the Arcane Bond selection; the Thassilonian Specialist's schools (`ThassilonianSchoolSelection` f431178ec0e2b4946a34ab504bb46285) are its own seven features, not the specialisation progressions; the Elemental Specialist's school selection (`SpecialistSchoolSelection` 5f838049069f1ac4d804ce0862ab5110) offers the game's eight; the Cruromancer's Blood Infusion (`BloodInfusionFeature` 9677fd6917ce46ae9fbd34a97af0849d) and school are features, not choices; the Arcane Bomber's and Thassilonian's cantrips rows; the Shadowcaster's Umbral Mind and the Spell Master's Focused Spells repeat ([254](254-a-features-steps-follow-the-class-level.md)).

## Findings (the game check, `known.txt` section 282)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner | 56 |
| option lists / in game only | 30 |
| prerequisites / missing in planner, a blueprint it doesn't map | 24 |
| progression rows / in game only | 18 |
| option lists / in planner only | 16 |
| progression rows / a further rank in planner only | 6 |
| progression rows / in planner only | 5 |
| option lists / a choice the game doesn't offer | 2 |
| prerequisites / not in game | 1 |
| prerequisites / value differs | 1 |
| **all** | **159** |

The 30 entries: `archetype arcanist/magic-deceiver`, `archetype arcanist/white-mage`, `archetype wizard/arcane-bomber`, `archetype wizard/cruromancer`, `archetype wizard/elemental-specialist`, `archetype wizard/exploiter-wizard`, `archetype wizard/scroll-savant`, `archetype wizard/shadowcaster`, `archetype wizard/spell-master`, `archetype wizard/thassilonian-specialist`, `class wizard`, `selection arcanist/arcanist-exploits`, `selection arcanist/magic-deceiver/ways-of-the-deceiver/living-deity/curse`, `selection option arcanist/arcanist-exploits/holy-water-jet`, `selection option arcanist/arcanist-exploits/potent-magic`, `selection option arcanist/arcanist-exploits/swift-consume`, `selection option arcanist/magic-deceiver/ways-of-the-deceiver/living-deity`, `selection option arcanist/magic-deceiver/ways-of-the-deceiver/razmiri-infiltrator`, `selection option wizard/school/abjurer`, `selection option wizard/school/conjurer`, `selection option wizard/school/diviner`, `selection option wizard/school/enchanter`, `selection option wizard/school/illusionist`, `selection option wizard/school/invoker`, `selection option wizard/school/necromancer`, `selection option wizard/school/transmuter`, `selection wizard/cruromancer/blood-infusion`, `selection wizard/cruromancer/school`, `selection wizard/elemental-specialist/school`, `selection wizard/thassilonian-specialist/school`.

The report lists each finding under its group, marked `[known: 282]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The prerequisites, lists and rows become the game's; `archetype_losses.ron`'s white-mage entry follows; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `mercies_and_exploits.rs`: `an_arcane_enforcer_picks_an_arcanist_exploit_at_1_4_7_and_every_3_levels` and `arcanist_options` users (Holy Water Jet open, Swift Consume closed to the Enforcer; 22 in the selection).
- `follow_ups.rs`: `the_razmiri_infiltrator_is_true_neutral_or_evil` (LN and CN too; a deity).
- `scaling_features.rs`: `the_fake_celebritys_inhibit_courage_steps`, `the_fake_celebritys_inhibit_greatness_steps_at_17` ([254](254-a-features-steps-follow-the-class-level.md)'s pass).
- `archetype_losses.rs` (white-mage).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 282 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
