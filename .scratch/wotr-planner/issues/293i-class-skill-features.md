# Importer S1, 293i: the features' ClassSkill that 262 needs

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293f
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

The builder wrote this section's results under "Done when" and left Status open. The main session restored the Answer from its report and the review (2026-11-16).

**Built:** the game's shape for the class skills an archetype's list lacks (decision 1).
- **`ClassSkill` components on three existing features:**
  - `witch/witch-of-the-veil/secrecy` (8c189560d7f94dd688f0dc29f6963610);
  - `oracle/seeker/seekers-trapfinding` (cb3abfaa682b67c48953dfa200aab325);
  - `sorcerer/seeker-sorcerer/seekers-trapfinding` (Trickery; b1c4f14bc87534e4da7ecd8e0b0658d2).
- **Four new features on row 1:**
  - `alchemist/metamorph/stealth`: MetamorphClassSkills 1d42191a101f2fb42b6e87b7bfc99a5e;
  - `monk/sohei/lore-nature`: SoheiLoreNatureFeature d59fe46a1aa1ace4e987d843c8e309ff;
  - `sorcerer/empyreal-sorcerer/lore-religion`: BloodlineCelestialClassSkill b0520c35f31ba544189d3293ed14dd67;
  - `sorcerer/sylvan-sorcerer/lore-nature`: BloodlineFeyClassSkill 243f474ce797acb4086c2dbc58660c4a.
- **The archetypes' own lists changed to match:**
  - the Metamorph's list lost Stealth, and the Sohei's lost Lore (Nature);
  - the Empyreal and Sylvan Sorcerers lost their own `class_skills`, so the sorcerer's list applies (`ReplaceClassSkills: false` in the game).
- **The Animal domain's Lore (Nature) moved to its row 1.**
- **The data note** is `libs/game-data/notes/293i-class-skill-features.md`.

**The game check:** 4,556 → 4,548 differences, exit 0 (the reviewer rebuilt `game-check` and diffed it against the base). Eight findings went:
- 4 class-table "class skills differ" lines (13 → 9), whose `known.txt` lines are removed;
- 4 progression-rows "in game only" lines (`MetamorphClassSkills`, `SoheiLoreNatureFeature`, `BloodlineCelestialClassSkill`, `BloodlineFeyClassSkill`, each at level 1).

The builder's report said the row findings stayed. They did not: the slice's count of 4 row findings was right.

**Tests:**
- new `class_skills::an_archetypes_own_feature_holds_the_class_skill_its_list_lacks`, over 7 archetypes, each citing its blueprint;
- three tests un-ignored, each citing the game:
  - `a_seeker_sorcerer_holds_trickery_and_its_bloodlines_class_skill`;
  - `the_animal_domains_class_skill_is_on_its_first_row`;
  - `the_animal_domains_lore_nature_comes_with_the_domain_at_its_first_level`.

**Left for 262:** the oracle Seeker's and Witch of the Veil's own lists still hold their feature's skill. These are the user's facts, and they stay among the 9 known "class skills differ" lines under 262.

Form proof (base 17d92338 recorded before the first edit): `builds.tsv`, `slots.txt` and `sheets.txt` are identical; 2,161
Builds compared, 2,161 old links reopen, 0 offer differences. `numbers.txt` differs in 20 Builds (82 lines), every one a
`Class skill=3` line toward the game: Trickery for the Seeker Sorcerer's Builds (`SorcererSeekerTrapfinding`
b1c4f14bc87534e4da7ecd8e0b0658d2, which the list lacked) and Lore (Nature) from level 1 instead of 4 for Animal-domain
Builds (`AnimalDomainBaseFeature` d577aba79b5727a4ab74627c4c6ba23c at level 1 of `AnimalDomainProgression`
23d2f87aa54c89f418e68e790dba11e0). The planner's own Seeker (oracle) and Witch of the Veil lists still hold their skill.
Scoped checks: `cargo test -p seeds -p corpus` passes, `check-data` and `cargo fmt --all --check` pass.
