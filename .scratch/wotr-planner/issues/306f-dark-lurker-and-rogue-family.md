# Importer S2, 306f: Dark Lurker, Evasion and the rogue family

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306e
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences named below. The note is `libs/game-data/notes/306f-dark-lurker-and-rogue-family.md`.

**What was built**
- New option `options/rogue/rogue-talents/evasion.ron` (`Evasion` 576933720c440aa4d8d42b0c54b77e80, granting the `evasion` feature) and new selection `selections/rogue/dark-lurker/rogue-talents.ron` (`DarkLurkerRogueTalentSelection` 1edbb68422c046f98a86f4726f6721b2): it owns Evasion and shares the rogue's 38 (`rogue/rogue-talents/skill-focus` named by hand, since the game's `SkillFocusSelection` c9629ef9… is held by two options; then the writer wrote the list). Names and summaries ours.
- `archetypes/rogue/dark-lurker.ron` gains rows 4, 6, 10, 12, 16, 18 and 20 naming it, each carrying the class row's features. The key `rogue/dark-lurker/rogue-talents` sits in rows holding no other selection, so no slot moves.
- Writer: `game-check --write option-lists --only rogue/dark-lurker/rogue-talents,rogue/rogue-talents,rogue/sylvan-trickster/fey-tricks,rogue/underground-chemist/discovery,shifter/feyform-shifter/rogue-talent,rogue/master-of-all/skill-focus`: Underground Chemist −17 (29 offers), Feyform Shifter +18 (the rogue's 38), Master of All's `ignores_prerequisites` true to false (the blueprint says false), Dark Lurker 39.
- `archetypes/rogue/underground-chemist.ron`: rows 12 to 20 removed (equal to the class's), so the discovery is at 10 only as the game has it.
- `archetype_losses.ron`: the Dark Lurker's selection loss is now a replacement by its own list (no `waits_on`); the chemist's loss at 4 and 10 no longer waits. `known.txt`: removed the lists of master-of-all, underground-chemist and feyform, and the Dark Lurker's rows line.

**Decisions:** Evasion carries no prerequisite (the blueprint's own `PrerequisiteNoFeature` of Evasion is a feature's own NoFeature, which the game check leaves out; main session's design). **Not done, deviates from the section:** `rogue/sylvan-trickster/fey-tricks` is not written: the game's 64 include `HexChannelerChannelFeature` b40316f05d4772e4894688e6743602bd, which 306j makes, so the writer leaves it whole; its `known.txt` line stays and `a_sylvan_tricksters_fey_tricks_are_the_games_64` stays ignored, now on 306j (the planner offers 62: it also lacks one rogue talent the game's list holds). The chemist's `progression rows` line (276) stays for its planner-only Discovery feature at 10.

**Tests:** `importer_lists::the_dark_lurker_offers_its_own_list_with_evasion` (39 offers, Evasion offered at 4, 6, 10, 12, 16, 18, 20, no slot at 2, 8, 14, none on a plain rogue; cited by the list's GUID and `DarkLurkerArchetype` 33eb0020…), `importer_lists::the_master_of_alls_skill_focus_checks_prerequisites` (`MasterOfAllSkillFocus` f2d2c170…); un-ignored `borrowed_selections::a_feyform_shifters_rogue_talent_is_the_rogues_selection` (`FeyformShifterShifterArchetype` 1cdfc7d3…, `RogueTalentSelection` c074a5d6…) and `data::an_underground_chemist_takes_a_discovery_at_10_only` (`UndergroundChemistSelection` c3d08fdb…, 29).

**Proof (offers, `FORM_PROOF_OFFERS=1`, base `aeab1bb5`, this lane):** links: 1,998 open, 1,998 the same, 0 through an heir, 0 refused, 0 changed (so no lines in `306-refused.txt`). Offers: 18,067,323 recorded, 18,067,927 now; only recorded 327, only now 931; 0 statuses changed. The 8 Builds that differ are exactly those holding a slot of a changed selection: `rogue rogue/dark-lurker` (plain, full, full/legend: the new slot at 20 and its fill) and `rogue rogue/underground-chemist`, its full and legend and the two `follow chemist trick once` Builds (rows 12 to 20 now the rogue's selection, picks the same). No slot-order refusal. Gold builds: none changed.
