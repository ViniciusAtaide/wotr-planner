# The MCP server's real-data tests after the 308 queue

Type: build (tests, maybe data)
Status: resolved
Blocked by: 308f
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

`scripts/queue-checks.sh` at `main` 0a10cf89 (the 308 queue's close, 2026-11-17) fails in `cargo test -p mcp --test server`, which no ticket's scoped checks ran:

- `compact::real_data_a_long_prerequisite_is_shortened_in_search_and_whole_in_describe` (`apps/mcp/tests/server/compact.rs` ~54): Witch Wandering Hex's prerequisite no longer reads as "one of N hexes (Air Barrier, …)"; it is now "(Magus level 4, the Hexcrafter archetype) or (Rogue level 2, the Sylvan Trickster archetype) or Witch level 1" (308d wrote the game's).
- `end_to_end::the_users_prompt_bft_summoner_lich_or_aeon` (`end_to_end.rs` ~296): after the agent fills feats from its lists, `slot.feat-selection.15.0`, `.17.0`, `.19.0`, `mythic.first-ascension` and the mythic slots are still open.

## Change

Find which merge of the queue (`2e044a3c..0a10cf89`) moved each, and whether the move is toward the game (cite the blueprint). Toward the game: change the test so it still tests what it says (a long prerequisite that is still long; a fill list the game's lists offer), per `CODING_STANDARDS.md` "An expectation moves only toward the game". Away from the game: fix the data or the engine.

## Done when

`cargo test -p mcp` passes, each change explained in the Answer with its cause and the game's evidence, and `scripts/queue-checks.sh 2e044a3c` gets past the workspace tests.

## Answer

Built on base 0a10cf89 (Opus builder; the fix rule is the ticket's and the main session's brief, the choice of replacements the builder's, as marked). Both moves are toward the game, so only the tests changed: no data, no engine.

**1. The long prerequisite.** Moved by **308d** (merge ec5c83e3, commit 5b1878f3), which wrote `mythic/abilities/witch-wandering-hex.ron` from the game: WitchWanderingHex fb8490d1721bd5a4883c3b6559310351 has three `Any` prerequisites, PrerequisiteClassLevel WitchClass 1, and PrerequisiteArchetypeLevel SylvanTricksterArchetype (rogue) 2 and HexcrafterArchetype (magus) 4. The planner's 27 hexes were its own. Its text is now short, so it no longer tests shortening.
- **The replacement (builder):** the Loremaster. The builder listed every real prerequisite `search` shortens (a temporary print in `SearchIndex::new`, removed). There are ten: Dragon Disciple, Loremaster, Boon Companion, Shifter's Edge, Domain Zealot, Everlasting Judgement, Mythical Beast, and the Chaos, Evil, Good and Law domains. Each is listed in `tools/game-check/known.txt`. Most of the long groups are the planner's expansion of what the game holds as one or two facts: Domain Zealot (DomainMastery 2de64f6a1f2baee4f9b7e52e3f046ec5 needs one of five features, DomainsSelection among them) and Everlasting Judgement (EverlastingJudgement 4a6dc772c9a7fe742a65820007107f03 needs JudgmentFeature or JudgmentFaithHunterFeature). Those groups would shrink when 307 closes the gap. The Loremaster's long group is a list the game itself holds: LoremasterClass 4a7c05adfbaf05446a6bf664d28fb103 has PrerequisiteFeaturesFromList, Amount 1, of ten metamagic feats (EmpowerSpellFeat, ExtendSpellFeat, HeightenSpellFeat, MaximizeSpellFeat, QuickenSpellFeat, ReachSpellFeat, PersistentSpellFeat, BolsteredSpellFeat, SelectiveSpellFeat, CompletelyNormalSpellFeat). The planner's twelve add Intensified and Piercing Spell (game check: `class loremaster`, `-> 315`). With 315's fix the group still has ten alternatives, more than `MOST_ALTERNATIVES` (6), so the test asserts no count.
- **The test** (`compact.rs`) searches `Loremaster` among classes and asserts four things. The short text starts `(one of `. It holds ` metamagics (Metamagic (Bolster Spell), Metamagic (Completely Normal Spell), ` and `, …)), `, so the later groups stay whole. `describe` starts `(Metamagic (Bolster Spell) or Metamagic (Completely Normal Spell) or `. The whole text names Metamagic (Selective Spell), and the short one doesn't. All four metamagics named are in the game's list. The ratio `whole > 4 × short` went: it measured 49 hexes, and the Loremaster's other groups are kept whole. The Superior Summoning half is unchanged.
- The examples of a shortened text in `apps/mcp/src/search.rs` (`LONG_PREREQUISITE`'s doc), `shapes.rs` (`Hit::prerequisite`, the schema an agent reads) and `server.rs` (`search`'s description) said "one of 49 hexes (Air Barrier, …)", a text the server no longer gives. They now quote the Loremaster's (builder). `libs/rules/src/engine/labels.rs:978` keeps the hexes as an illustration of the form; it is outside this lane.

**2. The end-to-end prompt.** Moved by **308c** (merge a165d7a2, commit ae5ea9fb), which wrote `feats/extra-arcanist-exploit.ron` from the game. Extra Arcanist Exploit needed one of the 16 arcanist exploits picked; it now needs the selection, `Has(Selection("arcanist/arcanist-exploits"))` or the Exploiter's. The game: ExtraArcanistExploitSelection 0a7065e13f4449fa81bfb33199bf7f6a has PrerequisiteFeature ArcanistExploitSelection b8bf3d5023f2d8c428fdf6438cecaea7 or ExploiterExploitSelection 2ba8a0040e0149e9ae9bfcb01a8ff01d (Group Any), and a PrerequisiteCondition that its own rank is under 3.
- **Why the slots stayed open:** the test fills the feats before the exploits. Before 308c the feat was closed then, so levels 15, 17 and 19 took Toughness, Metamagic (Persistent Spell) and Alertness. Now Extra Arcanist Exploit, fifth in `FEATS`, is open, and it is taken at 15, 17 and 19 (its three ranks). Each one opens its own slot (306aa), `slot.feat-selection.N.0`. The agent's fills, by `slot.feat.` and `slot.selection.`, never reached those slots. Every other pick in `FEATS` and `EXPLOITS` is taken as before. Each of the three slots offered the open exploits (Flame Arc, Force Strike, Ice Missile, …) and nothing was refused. The mythic choices were open only because the final assertion failed first: it allows `slot.mythic.` and `mythic.`.
- **Proof of the cause:** with only the base's `extra-arcanist-exploit.ron` line put back (temporarily, then restored), the unchanged test passes, with the three levels taking Toughness, Persistent Spell and Alertness.
- **The fix (builder):** the agent fills `slot.feat-selection.` from `EXPLOITS` after `slot.selection.`, from what `options` offers there: the Extra selection's list, the game's exploits (306aa). The final assertion is unchanged.

**Tests** (each in `cargo test -p mcp --test server -- --list`): `compact::real_data_a_long_prerequisite_is_shortened_in_search_and_whole_in_describe`, `end_to_end::the_users_prompt_bft_summoner_lich_or_aeon`. No real-data test in `libs/seeds` was added: no fact changed.

**Checks** (once, after the last edit):
- `cargo test -p mcp`: `test result: ok. 246 passed; 0 failed` (server), with the unit tests' 8 passed.
- `cargo fmt --all --check`: no output, exit 0.
- `scripts/test-modules.sh`: `test-modules OK`.
- `bash scripts/guarded-files.sh 0a10cf89`: run on the commit, since it reads `<base>..HEAD`. It touches nothing guarded.
- `scripts/queue-checks.sh 2e044a3c` is the main session's.
