# Gaps from the second martial run

Type: design
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user asked again (2026-11-09) for "the strongest martial build possible, maximum attack damage", this time from scratch. The agent built a Human Two-Handed Fighter 20 with a falchion and branched the Mythic path at MR 3: Demon against Trickster (Perception II opens the Trickster critical chain: Improved Improved Critical, Improved Improved Improved Critical and its multiplier step). Both Builds are legal and full. These are what the planner couldn't settle, or got wrong. The Human skill point it found is fixed in [130](130-skilled-race-skill-points.md).

### 1. The attack line can't tell the two branches apart

`compare` with `weapon: falchion` gave both Builds the same line: +29/+24/+19/+14, 2d4+14, Power Attack −6/+18, **15–20/×2**. The choice between them rested on the agent's own arithmetic (about 1,000 damage a round each on paper; Demon ahead on accuracy).

- **Wrong today:** the Trickster Build holds all three Trickster critical feats, ordinary feats like Improved Critical, which the line counts. Its range should read 13–20 and its multiplier ×3. `attack_leaves_out` doesn't name them, so the line states a wrong fact.
- **Left out, as documented:** what decides a martial build's damage is mostly unconditional: Two-Handed Weapon Training, Backswing, Greater Power Attack and Weapon Mastery (class features), and Power Attack, Improved Critical, Weapon Focus and Weapon Specialization (Mythic).

Options:
- **A.** Count the three Trickster critical feats in the line, as Improved Critical is counted.
- **B.** A, plus the unconditional weapon class features and the weapon mythic feats, each as its own `counts` row.
- **C.** [113](113-effects-as-data.md)'s option C, a modifier model.

*Recommend:* A now, since the line states a wrong fact. B as its own design, sharing [119](119-caster-line.md) and [120](120-initiative-and-skill-totals.md)'s per-item bonus shape.

### 2. Intelligence's skill points

The guide's character creation page (its Intelligence entry): Intelligence "grants bonus skills equal to one point per two character levels for each point of Intelligence bonus". So Int 12 gives a point on odd levels, Int 14 one every level, and Int 18 two. The guide adds that Skilled "is equivalent to +4 Intelligence". The engine adds the whole Int modifier every level, which is twice the guide's figure. The user's skill table ([research/skills-from-user.md](../research/skills-from-user.md)) gives class points only, so it doesn't settle this.

Options:
- **A.** Keep the whole modifier.
- **B.** The guide's rule: half the Int modifier per level, accumulated, so +1 Int gives a point at levels 1, 3, 5 and so on.

*Recommend:* B, since the guide wins where the game and the guide disagree. It changes every Build's skill points, so it's the user's call.

### 3. The Titan Fighter: Weapon Training or not?

The guide contradicts itself on `fighter#titan-fighter`. The intro says it "does not receive Armor Training or Weapon Training", but its level table lists Weapon Training at 5 and Weapon Mastery at 20. The data's rows keep the base fighter's Weapon Training, following the table, while its summary follows the intro.

*Recommend:* the user says which wins. If it's the table, the summary drops "or Weapon Training". If it's the intro, the archetype removes the row at 5.

**Decided and built (the user, 2026-11-09):** "Incredible Heft doesn't count as Weapon Training, so they cannot get it, for example for Gloves of Dueling." The intro wins: the Titan Fighter's empty `(level: 5)` row takes the base fighter's Weapon Training away, so Advanced Weapon Training stays closed to it. Weapon Mastery at 20 stays, as the table lists it and the user didn't question it. Its summary was already right. The test is `class_rules::the_titan_fighter_has_no_weapon_training`. Behaviour proof (`scripts/proof.sh data`, 2,162 Builds): only the 22 Titan Fighter Builds differ. Their sheets lose Weapon Training, and their feat slots close Advanced Weapon Training ("Needs Weapon Training (Fighter)", checked through the rebuilt `wotr-mcp`). `scripts/check.sh`: ALL-OK.

### 4. `search` caps `limit` at 100 without saying so

A search with `limit: 400` returned 100 of 261 feats. `total` was right, but no `note` said the limit was capped, and the tool description doesn't give the cap.

*Recommend:* state the cap in the description, and add a `note` when `limit` exceeds it.

### 5. The MCP server wasn't running when the session started

`.mcp.json` connects to `http://localhost:3902/mcp` ([129](129-mcp-over-http.md)). Nothing was listening, so the client marked `wotr-planner` failed (ECONNREFUSED). The agent started `WOTR_MCP_PORT=3902 target/release/wotr-mcp` by hand and called it with `curl`, since the client doesn't redial a project server mid-session.

Options:
- **A.** Document it: start the dev server or `wotr-mcp` before a session.
- **B.** A `SessionStart` hook that starts `target/release/wotr-mcp` on 3902 when nothing is listening.
- **C.** Make stdio the default again, keeping HTTP as an option.

*Recommend:* B.

### 6. Smaller cases for open tickets

- **[126](126-refusals-name-the-pick-order.md):** a level-11 general feat can't use Perception II, taken at MR 4, which also lands at level 11. The refusal said "Needs Perception II", which the Build already held, so the agent had to infer the pick order.
- **[125](125-favorite-metamagic-picks-its-metamagic.md):** Stat Focus takes no parameter, though its summary says "a stat of your choice". It's the same shape as Favorite Metamagic.

## Done when

The user has answered 2 and 5 (3 is answered and built). A is built for 1, so the Trickster Build's line reads 13–20/×3, with a real-data test. 4's cap is stated. `scripts/check.sh` passes.

## Choices settled

1. **Item 1, option A** (recommended answer, taken while the user slept, 2026-11-09): the attack line counts the Trickster's three critical feats as it counts Improved Critical, for the weapon they're held for. Improved Improved Critical and Improved Improved Improved Critical each widen the threat range by one more roll *after* Improved Critical's doubling (the guide's trickster page: "+1", "an additional +1"), and Improved Improved Improved Critical Improved adds 1 to the multiplier. A falchion (18–20/×2) with all four reads 13–20/×3. They add nothing to attack or damage, so they get no `counts` row; `critical`'s description names them. **Option B is not built:** the unconditional weapon class features (Two-Handed Weapon Training, Backswing, Greater Power Attack, Weapon Mastery) and the weapon mythic feats (Power Attack, Improved Critical, Weapon Focus, Weapon Specialization (Mythic)) in the line, each as its own `counts` row, stay a later design sharing [119](119-caster-line.md) and [120](120-initiative-and-skill-totals.md)'s per-item bonus shape. `attack_leaves_out` still names them.
2. **Item 2, option B** (recommended answer, taken while the user slept, 2026-11-09): Intelligence gives half its modifier in skill points a level, accumulated: ⌈L·m/2⌉ − ⌈(L−1)·m/2⌉ at character level L with modifier m, so an odd modifier's half point comes at odd character levels (Int 12: 1, 0, 1, 0, …; Int 14: 1; Int 16: 2, 1, 2, 1, …; Int 18: 2), and a penalty's at even ones (Int 8: 0, −1, 0, −1, …). The modifier is the one at that level, as before, so an increase or Legend's +4 counts from its level. The class's points plus Intelligence's stay at least 1, then the race's are added (ticket 130 unchanged).
   - **Skilled stays at 1:** under the half rule +4 Intelligence is +2 to the modifier, a point a level, which is exactly Skilled's point. So the guide's "equivalent to +4 Intelligence" now holds (under the old whole-modifier rule +4 Int was 2 points a level). Tested: a human fighter with Int 10 has a dwarf fighter's points with Int 14.
   - **Old links:** no new code. A link whose level spends more points than the new rule gives isn't possible, so `Engine::open_link` opens it as a stale link (ticket 40: a blank Build and the one generic error). The impact is measured below. **The user should confirm this:** it changes every Build with Int 12 or more (fewer points at even levels, or every level), and gives Builds with Int 9 or less a point back at odd levels.
   - Its own commit, `Ticket 131 item 2: …`, so it can be reverted alone.
3. **Item 3: the user's own answer, not the orchestrator's default.** The orchestrator's default was the smallest change (keep the table's rows, fix the summary). But the main checkout's working tree already held the user's decision (2026-11-09, quoted above, uncommitted): the intro wins, no Weapon Training at 5. It is brought over verbatim (`fighter.ron`'s empty `(level: 5)` row, the README note, `class_rules::the_titan_fighter_has_no_weapon_training`, and the paragraph above), as its own commit, so the main checkout's uncommitted copy of those four files can be dropped when this branch merges.
4. **Item 5, option B** (recommended answer, taken while the user slept, 2026-11-09): a `SessionStart` hook in the checked-in `.claude/settings.json` (new; `.claude/settings.local.json` untouched), matcher `startup|resume`, runs `scripts/start-mcp.sh`. When nothing listens on 127.0.0.1:3902 (`WOTR_MCP_PORT` overrides the port) it starts `target/release/wotr-mcp` with `WOTR_MCP_PORT` set, detached (`nohup`, stdin from `/dev/null`, output to `target/wotr-mcp.log`), and waits up to about 5 s for it to listen. A lock directory (`$TMPDIR/wotr-mcp-<port>.lock`, `mkdir` being atomic, taken over after a minute) and a second check under it keep two sessions from starting two servers. Without the binary it prints a hint to run `scripts/build-mcp.sh` and exits 0; it always exits 0. In a worktree session it starts that worktree's binary, which serves that worktree's game data. `scripts/build-mcp.sh`'s header mentions it.
5. **Items 4 and 6:** item 4 (`search`'s limit cap) and item 6's first bullet (ticket 126's pick-order refusal) are built by two other agents, who add their own Progress log lines here. Item 6's second bullet (Stat Focus's missing parameter) goes to [125](125-favorite-metamagic-picks-its-metamagic.md) later.

## Progress log

- 2026-11-09: item 6's second bullet built in [125](125-favorite-metamagic-picks-its-metamagic.md): Stat Focus takes a stat, one of the twelve the guide lists, once per stat; a link made before it holds one still opens, with the stat open in `todo`.
- 2026-11-09 (item 6, first bullet, in [126](126-refusals-name-the-pick-order.md)): a Mythic rank's picks come after its character level's slots, and the refusal now says so: Improved Improved Critical in the level-11 general slot with Perception II at MR 4 reads "Needs Perception II: Perception II is in Mythic rank 4's Improved Mythic Trick slot, which comes after this slot in level 11's pick order" (real-data test in `libs/seeds/tests/game_data/pick_order.rs`).
- 2026-11-09 (item 3): brought the user's uncommitted change over from the main checkout's working tree; `class_rules` 9 passed. The worktree's dev database was older than ticket 130's migration, so it was rebuilt with CLAUDE.md's four steps (no `.sqlx` change).
- 2026-11-09 (item 1), test first: `attack_line::the_trickster_critical_feats_add_to_the_threat_range_and_the_multiplier_after_improved_critical` (rules; a greatsword: 17–20/×2, 16–20/×2, 15–20/×2, 15–20/×3, and nothing for feats held for another weapon) failed, then passed with `THREAT_RANGE_FEATS` and `CRITICAL_MULTIPLIER_FEAT` in `libs/rules/src/engine/attack.rs`. The real-data test `weapons::a_trickster_falchions_critical_feats_make_it_13_20_x3` checks the four feats are weapon-parameterised and the falchion reads 13–20/×3. `GLOSSARY.md`'s Attack line and the MCP `critical` description name the feats.
- 2026-11-09 (item 5): hand tests of `scripts/start-mcp.sh` (the hook's own command, `"$CLAUDE_PROJECT_DIR"/scripts/start-mcp.sh`):
  - on 3902, where a `wotr-mcp` started by hand already listened: "already listens", nothing started;
  - on a free port 3999: started and listening in about 1 s; run again: "already listens", still one listener (`lsof`);
  - two at once on a free port 3998: one started it, the other said another session was starting it; one listener;
  - the binary moved away: the build hint, exit 0;
  - a stale binary (the worktree's clone of an older build, which can't read ticket 130's data): it starts, the binary exits, and the script says it isn't listening yet and names the log; exit 0.
  - An MCP `initialize` POST to the started server answered 200. The test servers were stopped and the lock removed itself.
  - Not tested: whether Claude Code dials `.mcp.json`'s servers only after the SessionStart hooks end (the script waits for the port so it's ready if it does). A session that still finds it refused is fixed by `/mcp` reconnect.
- 2026-11-09 (item 2), test first: `derived::intelligence_gives_half_its_modifier_a_level_the_odd_half_point_at_odd_levels` (Int 10 to 18 and 8, and odd character levels across a rogue/fighter) and `derived::the_humans_skilled_is_worth_four_points_of_intelligence` failed, then passed with `intelligence_skill_points` in `libs/rules/src/engine/walk/level.rs`. Four tests assumed the whole modifier and were updated: `derived::skill_ranks_accumulate_…` (fighter at level 2: 2, not 3), `derived::a_race_or_its_subrace_adds_skill_points_every_level` (a human fighter's level 2: 3), `level_pickers::no_skill_steps_up_once_the_levels_points_are_spent` (2 points at level 2), `path_rules::the_legend_switch_raises_every_score_by_4_from_the_switch_on` (Int 13 → 17 is a point a level, not 2); and `apps/mcp`'s `end_to_end::the_users_prompt_bft_summoner_lich_or_aeon` (a human arcanist with Int 20–25 spends at least 5 a level, not 7).
- **Item 2's impact on existing links** (measured 2026-11-09):
  - **Which links stop opening.** At character level L with Int modifier m, the old rule gave m points and the new gives ⌈m/2⌉ at odd levels and ⌊m/2⌋ at even ones. So Int 12–13 (m = 1) loses a point at even levels only, and Int 14 or more loses ⌊m/2⌋ or ⌈m/2⌉ at every level. A link whose level spent more ranks than the new count is impossible, and `Engine::open_link` opens it as a stale link (ticket 40: a blank Build and the generic error; nothing is repaired). In short: **any link of a character with Int 12–13 that spent all its points at some even level, or with Int 14 or more that spent all its points at any level, stops opening.** That is most planned casters and every agent-made Build with Int 12 or more (the MCP agents spend every point). Int 10–11 links are untouched; Int 8–9 ones gain a point at odd levels, and none breaks.
  - **The proof's Builds.** The 2,162 Builds the harness exported under the old rule (each level's points all spent; `proof-13/builds.tsv`, recorded on side B of the item 1 and 3 proof) were checked with the new engine (`Engine::is_possible`, a scratch test, not committed). 2,002 were possible under the old rule, 1,191 of them spending skill ranks. Under the new rule **127 of the 2,002 stop opening, and none starts**: 126 of the harness's "full" Builds, whose fill raised Intelligence (120 through ability increases, 4 through Legend's +4, 2 through a tiefling's), and the bench Build `multi_prestige20` (a human with Int 16, which spent all its points at every level). The harness's point buy is 10 in everything, so its count understates what real links see: the harness's Builds all keep Int 10–11 unless their fill raises it.
  - **The bench links** (`tools/proof/bench-links.tsv`): `fighter20`, `fighter20_nopath`, `legend40`, `start` and `blank` still open. `multi_prestige20` overspent each even level by one, and the harness asserts every bench link opens, so `scripts/proof.sh` would stop on it: it loses one rank at each even level (Athletics at 2 and 4, Mobility at 6, 18 and 20, Lore (Religion) at 8 to 16, its least-held skills), and opens again. It is part of item 2's commit.
- **Behaviour proofs** (`scripts/proof.sh`, each with its own `CARGO_TARGET_DIR` in the session's scratch folder, since other agents were running proofs on the machine; the machine's load average was over 100, so each record took about 30 minutes):
  - Items 1 and 3, `proof.sh both <out> dd436d3`: 2,162 Builds on both sides, **22 differ, all Titan Fighter Builds** (`fighter/titan-fighter`, the `t61 titan …` and `t61 ranger 2 titan 6 …` Builds and their full versions), each in its sheet and its feat slots' options and reasons from level 5 on (Weapon Training gone, so Advanced Weapon Training closed). Nothing else differs, and no Build's possibility changes. The fingerprint holds no attack line, so item 1 shows only in its tests.
  - Item 2, `proof.sh rules <out> 42a8302` (the commit before it): 2,162 Builds, exported under the new rule, so both sides find the same 2,002 possible. **482 differ**, all through the skill points: 461 in their sheet (the points available), 211 in `allows` (an edit's possibility: raising or lowering Int, a rank more), 106 in the class picker (a class with fewer points at a level whose ranks it would overspend), 3 in what's removable. 376 are Legend Builds (Legend's +4 takes Int 10 to 14, a modifier of 2: 1 point a level now, 2 before), the rest Builds whose fill or bench link raised Int, and 21 Int 10–11 full Builds whose `allows` alone changed (lowering Int to 9 now costs points at even levels only).
- Checks: `scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, the workspace tests, the rules wasm build, the seed's `--check-only`), run after items 1, 2, 3 and 5; after the bench-link change `cargo test -p proof` passed again. Test counts: rules 502 (was 499: +1 attack line, +2 skill points), `game_data` 266 (+1 falchion; the Titan Fighter's test came with item 3), `apps/mcp` server 91 (unchanged). No migration and no query change (`.sqlx` unchanged); the worktree's database was only rebuilt to catch up with ticket 130's migration.

## Answer

**Item 2 is held back, not on `main` (the orchestrator, 2026-11-09):** it was built and merged, then reverted on `main` in the next commit ("Hold back ticket 131 item 2"), because it makes most existing caster links stop opening and the ticket itself calls it the user's call. The work is kept whole on branch `held/131-int-skill-points` (commit `beec821`, with the bench link change); to take it, `git revert` the hold-back commit. Until then the engine gives the whole Int modifier, as before.

Done 2026-11-09 for items 1, 2, 3 and 5; items 4 and 6 are covered elsewhere (choice 5).

1. **The attack line counts the Trickster critical chain** (A): the run's Trickster falchion Build now reads 13–20/×3 where the Demon one reads 15–20/×2, so `compare` with `weapon: falchion` tells them apart. B (class features and mythic weapon feats in the line) is a later design with 119/120's per-item bonus shape.
2. **Intelligence gives half its modifier in skill points** (B, the guide's rule), the odd half point at odd character levels; Skilled's point is the guide's "+4 Intelligence". It changes every Build with Int 12 or more: of the proof's 2,002 possible Builds, 127 stop opening (126 whose fill raised Int, and the bench `multi_prestige20`); a real link with Int 12–13 that spent all its points at an even level, or with Int 14+ at any level, opens as a stale link. Its own commit, for the user to confirm or revert.
3. **The Titan Fighter has no Weapon Training**: the user's own answer (the intro wins), found uncommitted in the main checkout and brought over verbatim.
4. **The MCP server starts with the session**: the checked-in `SessionStart` hook runs `scripts/start-mcp.sh`, which starts `target/release/wotr-mcp` on 3902 in the background unless something already listens there, never twice, and only hints when the binary isn't built.
