# Ticket 306, stage S2: decisions on the slice research (2026-11-16)

The slices and their edit lists are in [slices.md](slices.md). Its §0 holds the decisions the design settles: the writer writes membership only, all or nothing per selection; selection keys need no aliases; Magical Tail is a remap; `once_per_option` is not generalised. These are the main session's answers to its §3 questions:

1. **The family slices make their splits by hand,** then run the writer limited with `--only`. New entries need names and summaries in our own words, aliases depend on the slot, and rows aren't the writer's until S5. A split table in the writer would cost about four more slices for no gain.
2. **306c commits S2's base links,** about 160 KB, for the links test.
3. **The gap tickets are filed before 306e,** so each line S2 moves out of `known.txt` can name its ticket:

| Gap | Ticket |
| --- | --- |
| G1, weapon categories in prerequisites | 313 |
| G2, a feature's rank as a stat | 314 |
| G3, parametrised feats one per value | 315 |
| G4, sub-skills | 312 |
| G5, planner choices the game lacks | 316 |
| G6, the spellbook kinds | 317 |
| G7, the components import | 318 |
| G8, archetype rows as adds and removes | 319 |
| G9, steps on the sheet | 320 |

**Builds whose pick is refused.** About 194 of the corpus's 2,000 open Builds will open with a pick refused, because the game's lists don't offer it. Picks the game's lists no longer offer are applied as the game has them (map Notes, 234; design §5). That counts 66 talents and 132 picks from the general and per-class feat lists; the feat-list picks weren't in the design's first count. Each refusal is listed by Build in its slice's Answer. No gold build changes are expected.

**A renamed or new selection key keeps its row's slot order** (main session, 2026-11-16, from 306e). `normalise` in `libs/rules/src/game_data.rs` sorts each row's `selections` by key, and slot ids follow that order. So a key that sorts elsewhere moves the row's slots, and a base link's picks stop opening. 306e's skald list first sorted after `skald/battle-scion/…` and broke 7 Battle Scion links; it's named `rogue/skald-talents` so it sorts where `rogue/rogue-talents` did. Every family slice checks its new keys sort where the keys they replace did. The link census catches a slip as a refusal that isn't toward the game. 306h meets the same sort for `skald/rage-power`.

**An option moved into a wider list keeps its owner's gating** (main session, 2026-11-17, from 306i's rescue). When a family slice moves options from a gated selection into a wider list, each moved option carries over the gate as a `Requires`, cited by its blueprint, as the game's options carry it in their prerequisites. Otherwise the census shows offers moving away from the game. 306i's six greater exploits needed the game's `ArcanistGreaterExploitsFeature` c7536b93f17c70d4fa3a8cf9aa76bfb7.

**A slice that gives a class its own entry updates the corpus generator in the same diff** (main session, 2026-11-17, from the rescue retros of 306j and 306k: a cause seen twice). When a split gives a class its own Extra feat or list, the generators under `tools/corpus/src/builds/` still name the old entry, and the regenerated corpus loses Builds or holds ones that no longer open. Change the generator so the Build count stays the same, and state the count before and after in the Answer. Ticket 306af makes `compare` name such Builds.

**A Proof counts refusals in link lines,** as `306-refused.txt` and `compare` count them: each Build's sample and `full` copies, and each `/ filled` or `/ legend` variant. A family of Builds is several lines.
