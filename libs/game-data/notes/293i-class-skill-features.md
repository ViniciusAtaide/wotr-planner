# 293i: the class skills the game gives through a feature

**What the data says now.** A class skill the game gives an archetype through its own feature is held through that
feature, as a `ClassSkill` component, not folded into the archetype's `class_skills` list (the game's shape; user's
standing rule on skills, decision 1 of `research/293/decisions.md`).

- Components added to existing features: `witch/witch-of-the-veil/secrecy` (Stealth, Trickery:
  `WitchOfTheVeilSecrecyFeature` 8c189560d7f94dd688f0dc29f6963610), `oracle/seeker/seekers-trapfinding` and
  `sorcerer/seeker-sorcerer/seekers-trapfinding` (Trickery: `SeekerTrapfinding` cb3abfaa682b67c48953dfa200aab325,
  `SorcererSeekerTrapfinding` b1c4f14bc87534e4da7ecd8e0b0658d2; the game's `Thievery` is the planner's Trickery).
- Four new features, each granted at level 1 of its archetype: `alchemist/metamorph/stealth`
  (`MetamorphClassSkills` 1d42191a101f2fb42b6e87b7bfc99a5e), `monk/sohei/lore-nature` (`SoheiLoreNatureFeature`
  d59fe46a1aa1ace4e987d843c8e309ff), `sorcerer/empyreal-sorcerer/lore-religion` (`BloodlineCelestialClassSkill`
  b0520c35f31ba544189d3293ed14dd67, level 1 of `EmpyrealBloodlineProgression` 8a95d80a3162d274896d50c2f18bb6b1) and
  `sorcerer/sylvan-sorcerer/lore-nature` (`BloodlineFeyClassSkill` 243f474ce797acb4086c2dbc58660c4a, level 1 of
  `SylvanBloodlineProgression` a46d4bd93601427409d034a997673ece). Names and summaries are the planner's own.
- Lists: Metamorph lost Stealth and Sohei lost Lore (Nature) (the game's `ReplaceClassSkills` lists lack them); the
  Empyreal's and Sylvan's own `class_skills` went (the game keeps the sorcerer's list, `ReplaceClassSkills: false`).
- The Animal domain's Lore (Nature) moved to a new row 1 (`AnimalDomainBaseFeature` d577aba79b5727a4ab74627c4c6ba23c
  at level 1 of `AnimalDomainProgression` 23d2f87aa54c89f418e68e790dba11e0).
- The Witch of the Veil's and the oracle Seeker's own lists still hold their feature's skill (their lists are
  262's); the Wind Whisperer needs no feature (its mystery's row gives the skills).

**Not modelled.** The four class-table lines for Metamorph, Sohei, Empyreal and Sylvan under 262 in
`tools/game-check/known.txt` are gone: their lists now agree with the game.

**Test.** `an_archetypes_own_feature_holds_the_class_skill_its_list_lacks` in
`libs/seeds/tests/game_data/class_skills.rs`.
