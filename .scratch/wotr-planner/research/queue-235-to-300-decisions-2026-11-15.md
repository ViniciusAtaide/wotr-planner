# The queue of tickets 235–300: decisions, lane plans and follow-ups (2026-11-15)

The main session's running record of the queue that took the game as the source of truth. The "Follow-ups" and "Gap candidates" sections are not ticketed yet: they become tickets at the queue's end, or the importer (293) settles them.

User, 2026-11-15:
- 262: apply the game's class skills (13 rows) and the Defender of the Hearth's skill points (3), changing research/skills-from-user.md's table where it differs.
- 276: apply the game's rule for Improved Uncanny Dodge (Eldritch Scoundrel, bard, slayer or skald level 12), reversing ticket 190's choice of 2026-11-13.
- Build all of 247–291 in this queue, in order, then 244, then 245.
- Earlier: the seven facts of 242's research are applied (see the tickets' "decided" lines).
- 309 game-check findings filed under 244 by the filer: accepted (main session), 244's Done-when updated.

Lane plan:
- Engine lane (rules files, one after another): 254 → 247 → 249 → 250 → 248 → 251 → 252 → 253.
- Data / cross-cutting lane: 256 (tools), 261, 262, 267, 270, 290, 291, then 255, 257, 258, 259, 260 as their files allow.
- Class-file tickets 263–289 after their blockers, two at a time, no two on one file.
- Then 244, then 245.

User, 2026-11-15 (later), "let's do 1 2 3 4":
1. Lighter reviews: the main session reviews small or mechanical diffs itself; an engine ticket gets one Sonnet review.
2. Engine rules bundled: 247+249+250 (a selection held from its level, taken again, the Extra caps), 248+251+253 (no order in a level, proficiency where prerequisites are ignored, archetype level offsets), 255+257+258+259+260 (engine and data) — each bundle one builder, one branch per ticket inside it, merged together.
3. 244 starts as soon as 254 merges, in lane-game, beside the engine bundles in lane-tests; engine bundles merge to main first, 244 merges main into its branch before it finishes.
4. The queue stops after 293 (and the queue-end checks): the gap tickets 293 leaves, the model parts of 267/270/288/289, and 245 wait for a next queue.

Follow-ups to file at the queue's end (from 252):
- Feature bonuses still count toward ability prerequisites; by `Modifier.IsPermanent` an untyped one (Legend's +4, LegendFeature 7f99961610390044391f300c7ef5e0c8; the Swarm's) wouldn't; Racial ones (Dragon Disciple) do.
- Perfect Mind's skill floor shows as "Ranks" in the totals rather than its own modifier; it also feeds the class-skill +3 and ten-rank thresholds.
- Decision (main session, map Notes rule): Perfect Mind's BAB is 15 + the Mythic rank as the code reads, the summary says where the text differs.

Gap candidates for 293's list (from the 247–250 review):
- Other mythic abilities offered by MythicAbilitySelection are selections themselves (ExtraDomain, SecondBloodline, SecondMystery, …): the game's CanSelect lets them repeat; the planner holds them once.
- extra-hex is one planner feat capped at 3 across witch and shaman hexes; the game has two (ExtraShamanHexSelection), each capped at 3; ExtraVivsectionistDiscoverySelection is also missing as its own feat.

The component vocabulary (design in $G/vocab/design.md; user, 2026-11-15):
- Adopt the staged plan: 295 (core) → 296 (prerequisites as components) → 297 (numbers) → 298 (selections as the game has them) → 293 (the importer; the only stage that changes values). 296–298 prove no behaviour moves.
- Entries: every reachable feature becomes an entry (the user, against the recommendation of "only those named or carrying a vocabulary component"): about 3,500 more. 295 measures the browser JSON's growth first and reports it.
- Compile components at load into the engine's current structures (recommended, taken).
- The other open questions take the design's recommendations: unify the held-fact maps in 298; AddFeatureIfHasFact follows the code (one-way); no runtime markers in the data; build only the property getters the planner's components use; Skill Focus in the game's selection shape with a link alias.
- The third engine bundle (255, 257–260) dissolves into the vocabulary and the importer; 258 becomes the generic Spell Focus kind (+ ExpandedArsenal).

From 248/251/253's fix round:
- 251 makes 829 of 2,164 corpus Builds impossible (the generator takes weapon feats in weapons the class isn't proficient with). Game-correct, but it narrows the self-checks' coverage: 245 (the corpus generator's quirks) should regenerate those Builds with proficient weapons.
- The game's RangerStyleArcherySelection10 also offers Toughness, Dodge, Iron Will, Lightning Reflexes and Great Fortitude, which ranger/bonus-feats-10b lacks: for 293.
