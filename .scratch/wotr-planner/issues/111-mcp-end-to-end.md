# MCP server: end to end on real data, and a manual agent run

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 111 of the [MCP server plan](104-mcp-server-plan.md), the last one. Does the server carry the user's example prompt from start to finish on the real `libs/game-data`, the way an agent would drive it? The prompt: "a BFT main character, buffs and summoning, Lich or Aeon (compare), summons lasting 24h through caster level 25 and Extend, buffed with the mass animal buffs". *Done when:* the scenario passes as a test, the binary has been run as a scripted agent would run it, and the user has a guide for running the prompt in Claude Code (the user's own run closes the plan).

## Choices settled

1. **The test** is `apps/mcp/tests/server/end_to_end.rs`, `the_users_prompt_bft_summoner_lich_or_aeon`. It uses the real data and only the MCP interface: an rmcp client, as in the other server tests. A small `Agent` helper holds one Build's code and replaces it after every `choose`. Its methods:
   - `choose` (every step must apply) and `try_choose`;
   - `options` (always `values: true`) and `value` (the open option a query names);
   - `take` (the first preferred open option, else the first open one) and `fill` (`take` on every `todo` entry under an address prefix).
2. **Every pick goes through `choose`.** Values come from `options` wherever the test would otherwise hard-code an object or an unsure key:
   - the background, `{class, archetype}` for the Brown-Fur Transmuter, and `{key: "spell-focus", parameter: "conjuration"}`;
   - Metamagic (Extend Spell), Enduring Spells, Spell Focus (Mythic) (Conjuration), Greater Enduring Spells and Ascendant Summons;
   - the remaining feats, the exploits and the path picks.

   A few plain keys the agent already has from `search` or the prompt are given directly: `human`, `int`, `atheism`, `true-neutral`, `augment-summoning`, `superior-summoning`, the paths.
3. **The scenario's choices:**
   - **Build start:** human (+2 Int), atheist, true neutral, the Arcane background. The point buy is Str 8, Dex 14, Con 14, Int 18, which spends all 25 points; the steps go Str down first so the budget allows the rest.
   - **Feats:** Spell Focus (Conjuration) goes in the human's level-1 bonus feat, which is checked before the general feat, and Augment Summoning in the general feat. That way both summoning feats come at level 1. Augment Summoning in the bonus feat first is `unmet`, "Needs Spell Focus (Conjuration)", and the code is unchanged. Superior Summoning comes at 3 and Extend at 5. The rest come from a preference list through `options`.
   - **Levels:** 2 to 20 go in one batch. All five increases go to Int, for Int 25 at 20.
   - **Exploits** are filled, Greater Exploits included.
   - **Skill points:** one `choose` per level, with a rank per skill in turn. Each batch stops at a `not_an_option` when the level's points run out, which shows reading `rejected`.
   - **Mythic, shared:** the first ascension is the Lich's (Danse Macabre), so both branches share MR 1–2. Then Enduring Spells at MR 1 and Spell Focus (Mythic) (Conjuration) at MR 2.
   - **Mythic, per branch:** branch at `mythic.path` into `lich` and `aeon`. Each branch takes Greater Enduring Spells at MR 3 and Ascendant Summons at MR 5, plus its path picks: Deadly Magic first for the Lich, Summoner's Gaze where it opens (MR 5) for the Aeon. The rest is filled until `todo` is empty.
4. **What it asserts:**
   - "BFT" finds the archetype first.
   - Augment Summoning before Spell Focus is `unmet`.
   - The Build has 20 levels and Int 25, and only Mythic choices are open before the Mythic ranks.
   - Both branches end with an empty `todo`.
   - `compare` shows Lich and Aeon at MR 10 with the shared first ascension, the same classes and numbers, no level differing in numbers or picks, `slot.path-selection.3.0` as "Lich Powers / Enforcing Gaze", and path features differing from MR 3 on: Summon Fallen against Summon Axiomite at 3, and Summon Ravener Dragon at 9.
   - Both final codes open again through `new_build {link}`, and their `link`s are the planner's URL.
5. **Where the data lacks what the prompt implies, the test asserts it's absent and says why:**
   - "caster level 25": the sheet's caster level is class levels only, so both branches show 20.
   - "the mass animal buffs": spells are out of scope, so `search` finds nothing for "mass animal growth" or "animal growth".
   - "summons lasting 24h" and what Extend or Enduring Spells do: durations aren't modelled.

   All three are the agent's own game knowledge under the instructions' honesty boundary.
6. **A data gap seen, not fixed here:** `options` offers Greater Enduring Spells at MR 1 to a Build without Enduring Spells. The data gives Greater Enduring Spells no prerequisite (`libs/game-data/mythic/abilities.ron`). If the guide gives it one, that's a data ticket: edit the `.ron`, add a README note and a real-data test. The scenario takes Enduring Spells first anyway.
7. **The manual run** used a scratchpad Python script (about 150 lines, not checked in). It spoke newline-delimited JSON-RPC to `target/release/wotr-mcp`: `initialize`, `notifications/initialized`, `tools/list`, `resources/list` and `resources/read`, then 180 `tools/call`s for the same scenario. The condensed transcript is below.

## Manual run guide (Claude Code)

1. **Build the server once first**, from the repo root: `SQLX_OFFLINE=true cargo build -p mcp --release`. `.mcp.json` launches it with `cargo run -q -p mcp --release`. A cold release build can take longer than Claude Code waits for a server to start (its `MCP_TIMEOUT`), so the first launch could fail on a clean checkout. Once it's built, `cargo run` starts in about a second.
2. **Start Claude Code at the repo root** (`claude`). The project's `.mcp.json` is checked in, so Claude Code asks once before it starts a project server. It names `wotr-planner` from `.mcp.json` and offers to use this server (or all of this project's servers), or to continue without it. Approve it. The choice is remembered per project; `claude mcp reset-project-choices` asks again.
3. **Check that it connected:** `/mcp` lists `wotr-planner` as connected, with 8 tools (`search`, `describe`, `new_build`, `sheet`, `options`, `choose`, `withdraw`, `compare`) and one resource, `wotr://glossary`. The tools show as `mcp__wotr-planner__<tool>`. Claude Code may ask for permission the first time each one runs; they're all read-only (they return codes and change nothing).
4. **The prompt to try**, as the user wrote it:

   > a BFT main character, buffs and summoning, Lich or Aeon (compare), summons lasting 24h through caster level 25 and Extend, buffed with the mass animal buffs

   **What to expect:**
   - The agent should `search` "BFT" and ask whether it means the Brown-Fur Transmuter. "BT" would be the Beast Tamer.
   - It should build in `choose` batches, branch at the Mythic path, and `compare`.
   - It should give two planner links.
   - It should label the 24h duration, the CL 25 arithmetic, Extend's and Enduring Spells' effects and the mass animal buffs as its own game knowledge, not as checked facts.
   - It shouldn't fetch any guide link.
5. **Check the links** open on the site: `https://viniciusataide.github.io/wotr-planner/#b=…`. The server's stderr names the data file the local data matches (`game-data/a7ba1f835d463455.json` today). If the site serves a different one, a link made here may be stale there.

## Manual run transcript (condensed)

```
> initialize -> wotr-planner 0.1.0 (WotR Build Planner); capabilities ['resources', 'tools']
  instructions: 2569 chars
> tools/list -> choose, compare, describe, new_build, options, search, sheet, withdraw
> resources/list -> wotr://glossary (text/markdown, 25218 bytes); resources/read -> '# WotR Build Planner …'
> search 'BFT' -> archetype arcanist/brown-fur-transmuter (Brown-Fur Transmuter)
> search 'BT'  -> archetype bard/beast-tamer; option barbarian/rage-power/beast-totem; feat back-to-back
> new_build {name: 'BFT summoner'}
> choose [human, +2 int, atheism, true-neutral, arcane, Str 8, Int 18, Dex 14, Con 14] -> applied 9
> options level.1.class 'brown-fur' -> {"class": "arcanist", "archetype": "arcanist/brown-fur-transmuter"}; choose it
> choose slot.feat.1.race.0=augment-summoning -> applied 0, rejected unmet: Needs Spell Focus (Conjuration)
> choose [slot.feat.1.race.0={key: spell-focus, parameter: conjuration}, slot.feat.1.general.0=augment-summoning] -> applied 2
> choose level.2..20.class={class: arcanist} -> applied 19
> choose [slot.feat.3=superior-summoning, slot.feat.5=metamagic-extend-spell, level.4/8/12/16/20.increase=int] -> applied 7
> remaining feats -> 7 Greater Spell Focus (Conjuration); 9 Spell Penetration; 11 Greater Spell Penetration;
                     13 Metamagic (Quicken Spell); 15 Toughness; 17 Metamagic (Persistent Spell); 19 Alertness
> exploits -> 1 Potent Magic; 5 Swift Consume; 7 Spell Resistance; 11 Arcane Barrier + Greater Spell Resistance;
              13 Acid Jet; 15 Armored Mask; 17 Dimensional Slide; 19 Energy Shield
> level.N.skill batches -> ranks per level 7 ×7, 8 ×8, 9 ×5; each batch stops at not_an_option (points spent)
> options mythic.first-ascension -> aeon (Force Reality), angel, azata, demon, lich (Danse Macabre), trickster
> choose [first-ascension=lich, slot.mythic.1=enduring-spells, slot.mythic.2=spell-focus-mythic(conjuration)] -> applied 3
  trunk todo: mythic.path, slot.mythic.3 … slot.mythic.10
> [lich] choose [mythic.path=lich, slot.mythic.3=greater-enduring-spells, slot.mythic.5=ascendant-summons] -> applied 3
  [lich] path picks: Deadly Magic, Skeletal Assassin (3); Skeletal Barbarian I (4); Death of Elements (6);
         Skeletal Bard I (8); Death Rush (9); Skeletal Cleric I (10); todo now: []
> [aeon] same three steps -> applied 3
  [aeon] gazes: Attack (3); Attacks of Opportunity (4); Summoner's Gaze (5); Area Effects (6); Caster Level (7);
         Combat Maneuvers (8); Concentration (9); Damage (10); todo now: []
> compare [lich, aeon]
  both: Arcanist (Brown-Fur Transmuter) 20; BAB 10, F/R/W 6/6/12, Int 25, CL 20, arcane 9, MR 10,
        first ascension Lich, cap 20, 0 open choices
  start diffs: none; level diffs: none
  MR 3–10: path Lich | Aeon; each rank's path pick, e.g.
    slot.path-selection.3.0 [Lich Powers / Enforcing Gaze]: Deadly Magic | Enforcing Gaze - Attack
    slot.path-selection.3.1 [Skeletal Champion]: Skeletal Assassin | (none)
    slot.path-selection.5.0 [Enforcing Gaze]: (none) | Enforcing Gaze - Summoner's Gaze
  path features, Lich only | Aeon only:
    MR 3  Summon Fallen | Aeon Immunities, Summon Axiomite
    MR 4  Lord Beyond the Grave, Summon Plague Beast Mastodon | Aeon Bane, Summon Stone Golem
    MR 5  Dark Rites, Summon Plague Beast Mastodons | Aeon Immunities, Mythic Suppression, Summon Iron Golem
    MR 6  Summon Ecorche | Enforcing Gaze - Extra Gaze, Improved Aeon's Bane, Summon Gold Golem
    MR 7  Summon Nightshade Nightcrawler | Aeon Companion Immunities, Aeon Immunities, Mythic Suppression, Summon Gold Golems
    MR 8  Summon Nightshade Nightcrawler and Ecorche | Enforcing Gaze - Perpetual, Summon Adamantine Golem
    MR 9  Summon Ravener Dragon | Aeon Companion Immunities, Greater Aeon's Bane, Mythic Suppression, Summon Adamantine Golems
    MR 10 (none) | Aeon Immunities, Enforcing Gaze - Extra Gaze
> new_build {link: <lich link>} -> opens; new_build {link: <aeon link>} -> opens
> search 'mass animal growth' -> 0 matches; search 'animal growth' -> 0 matches
(180 tool calls, 0.6 s in all)
stderr: wotr-mcp: game data from …/libs/game-data, which the site serves as game-data/a7ba1f835d463455.json
```

The two final links:
- Lich: `https://viniciusataide.github.io/wotr-planner/#b=Ad1QPUjDQBi9Lxdpk2oEFUpRVATBHzJUiiBuEa2LRjAullgculj7Q9TBTFaxm4Obi06i1klQkDh1cXNowTpZUBC1CeggTloxJTWmFH_G0jvufY_3vXfHfVmQD8f8ZLPH9ZJ618ZjseO5qTdpWlbye-7cQ12l68K8o2paXHK7ArsZfPXqu1yEuwPbsprmxL4nvm2CYpmZYCKhPTp34lvOzo9ndNY90NK_vjLYdZ26ie4nIxthttejdDRKm3xmkh0eaiUYv2uNdt-PcgHxaNanRZXz-riO_MUtTqveHlUW8rvZVa94EjzN6S9CLTcitC8shUKRcEACRNYA2BmmgaabkA2MVU7In1vfHjt8Sb-yf_soMMW_qMWr6_qvkAMQQhgQoReiBDAQ5aIFgDJuKGSNisDgJYcGAjCGAiITiqjPCxdzYMmY-xM`
- Aeon: `https://viniciusataide.github.io/wotr-planner/#b=Ad2PS0gCURSG75k7pTPaBBWIFRVBUMgsDOlBraYHLSqDdJOYtHCR5YOpFrlKJGlTELSIoKBNZYsIatG0mk17g2wVERG9BmoRrbLoypQpkrUU7-V-5-c__z1wLiG4O-CiqyzGl1hCGQyFDjz2N3EkKL1vmR_uSvLdt03qCqYlyOt52I3j8-JNehpudjSzj6eCs_nJWjvE8NzoRDSq3Bs2IquGho9ndNzUXt0yP9fReBG7CmzL_kUfb7JI9eXisjU-zPd01VCcyxhmzbd9gtu5P-ZQAtJJaYRw_KhyRUaJtmvTYXhvqV_uXvCstXbq7a-OM9ALvba6qRmv1-9zi4DoIgAtx5WxbAXSgHqyBf176yejhW8rp_p3joGU-ZdMyxKfbIV0gBDCgChSqAxgoLLNNACjTkj-VSsCVWc8FijAGJJEKXyR7EhU7vsJ`

## Progress log

- 2026-11-08: Built on 110 (025405a). Explored the real data through the release binary with a scratch script, as an agent would. That turned up the facts above:
  - MR 1 comes at level 6 and MR 10 at 19.
  - A human arcanist with Int 20 gets 7 skill points.
  - The BFT has no exploit at 3, where Powerful Change replaces it.
  - Greater Enduring Spells is open without Enduring Spells.
  - No spell matches a search.
- Wrote the test. Its first run failed in the test's own helper: "enduring spells" also matches Greater Enduring Spells, so `value` now prefers the option whose name is the query. Then it passed (about 1.6 s in a debug build).
- Ran the same scenario against the release binary over stdio (transcript above).
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK. `--full`: ALL-OK.

## Answer

Done 2026-11-08. The user's prompt runs end to end on the real data through the MCP interface. The steps:
1. "BFT" resolves to the Brown-Fur Transmuter.
2. A legal 20-level BFT arcanist is built with the summoning feats; Augment Summoning taken first is refused as `unmet`.
3. MR 10 is reached and the Build branches into Lich and Aeon. Each branch has Enduring and Greater Enduring Spells, Ascendant Summons and its path picks, and nothing left open.
4. `compare` lists the path differences rank by rank, and both links open.

The prompt's spell, duration and caster-level parts are absent by design, and the test says so. 72 server tests pass (1 new). The plan's last step is the user's own run in Claude Code, with the guide above.
