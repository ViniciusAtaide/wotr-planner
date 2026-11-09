# Rank branches by a number, and list what raises it

Type: design
Status: resolved
Blocked by: none ([119](119-caster-line.md), done; the caster line is the number this run needed; the attack line from [114](114-attack-line-and-damage.md) already exists)
Map: [WotR Build Planner](../map.md)

## Question

The user's goal in both agent runs was a maximum: "maximum attack damage" (2026-11-08) and "the maximum chance to debuff the maximum amount of enemies" (2026-11-09). `compare` is facts only and "doesn't say which Build is better" ([110](110-mcp-compare-and-instructions.md)), so every choice between candidates in the Max DC run was the agent's judgement:
- Arcane bloodline against Crossblooded (Arcane + Fey) against Seeker;
- Azata (Favorable Magic, Zippy Magic) against Aeon (the Save DC gaze);
- which school Expanded Arsenal should copy to.

The agent also found the items that raise a DC by memory (see [123](123-search-by-effect.md)). Once the planner computes a number, it could answer both questions itself without judging anything: a number is a fact.

## Options

- **A. Nothing:** `compare` gives facts; the agent ranks.
- **B. `compare` orders by a named number:** `compare {builds, by: {caster_line: {school: "enchantment", subschool: "compulsion"}}}` (or `attack_line: {weapon}`) gives each Build's number at its last level and orders the summaries by it. Still facts, and still nothing about what the number leaves out.
- **C. B plus `improve`:** for a Build, a choice address and a number, try every open option of that choice and return them ordered by the change they make to the number ("slot.mythic.4: Spell Focus (Mythic) +1 compulsion DC; Expanded Arsenal +0; ..."). The engine walks one Build per option, as `options` already checks each one.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B with 119, then C. C is the one that would have found School Power and Fey without memory, and it reuses `options`' per-option walk.
2. **Ties and conditions:** Favorable Magic changes no DC, but changes the odds. *Recommend:* the numbers stay the unconditional lines'; anything conditional (gazes, roll twice) stays in summaries and the agent's words, and `improve` says `no_change` rather than hiding it.
3. **Cost:** a slot with ~60 open options means ~60 walks. *Recommend:* measure on real data before deciding a limit; the walks' cost is known from ticket 83.

## Done when

The user decides. If B: `compare` with `by` orders the run's three bloodline branches by their compulsion DC, with a server test. If C: `improve` on the run's MR 4 mythic feat slot lists Spell Focus (Mythic) first for enchantment DC, with a server test. `scripts/check.sh` passes.

## Choices settled

Every recommendation taken while the user slept, 2026-11-09 (the user's brief: "take the recommended answers"). Questions 1–3:

1. **B, then C**, recommended answer, taken while the user slept, 2026-11-09: `compare` takes `by`, and a new tool `improve` ranks one choice's open options.
2. **The numbers stay the unconditional lines'**, recommended answer, taken while the user slept, 2026-11-09: `by` reads the attack line or the caster line ([114](114-attack-line-and-damage.md), [119](119-caster-line.md)) as they are; an option whose number doesn't move goes in `improve`'s `no_change` (Favorable Magic, the superpower, is there for the compulsion DC), and both results carry the line's `leaves_out`.
3. **Measured, no limit**, recommended answer, taken while the user slept, 2026-11-09: see the progress log's timings. The worst slot of the run's Build (its level-19 general feat slot, 241 open values) costs 36 ms in release, about 0.15 ms per option walked: [83](83-index-the-engines-hot-lookups.md)'s class picker (one recheck walk per candidate, ~240) costs 7–8 ms, and `improve`'s walk is a full `evaluate` (a sheet) where the picker's is a check without one. No limit is set; `query` narrows the options tried when an agent wants fewer.

Settled while building (the smallest choice consistent with the map's Notes, taken while the user slept, 2026-11-09):

4. **The DC's spell level** (the orchestrator's brief: recommended, the highest castable, or the heightened row when present): the highest spell level castable at the last level, which is the heightened row's DC when the Build holds Heighten Spell (119 settled that the two are equal). A Build that casts one level less shows one less: that is part of the fact.
5. **The numbers by name:** `by: {"caster_line": {school, subschool?, energy?, metamagic?, number?}}` with `number` `dc` (default), `spell_penetration` or `concentration`; `by: {"attack_line": {weapon, number?}}` with `number` `attack` (default: the first attack's bonus) or `damage` (the bonus each hit adds to the dice; the dice are the weapon's, the same in every Build; Power Attack not taken). Refused as `caster` and `weapon` are (`invalid_arguments`).
6. **`compare` with `by`** puts the Builds in the number's order (highest first, a Build without the line last, ties in the order given) *before* the rest is computed, so every per-Build list follows that order; each summary gets `number` and `given` (its place in `builds`), and the result `by` (`number` in words, `order`, `leaves_out`). `by` doesn't add the line itself to the summaries: `caster`/`weapon` still do.
7. **`improve`'s baseline is the Build as it is:** `change` = the number with the option taken minus the number now, so on a filled slot the held option is `no_change` and the others show what swapping gains or loses (on the run's Build MR 2's Spell Focus (Mythic) is `no_change` and every other mythic feat −1). Each open option (as `options` lists them with `values: true`, so a parameterised feat once per value) is applied by the engine's own edit and evaluated; closed ones are counted (`closed`), not tried. `options` (those that change it, highest number first, with `value`, `name`, `number`, `change`), `no_change` and `no_line` (the option's Build has no such line) list names only, to keep the response short (the level-19 slot has 241). `query` filters as `options`' does.
8. **The test branches.** The run's own three candidates (Arcane, Crossblooded Arcane + Fey, Seeker) differ at level 1 (the bloodline or the archetype), and every other bloodline at `slot.selection.1.0` is `breaks_later` on the run's Build (School Power is Arcane's). So the server test branches its MR 1 Second Bloodline instead: Fey (DC 34, the run), Infernal and Celestial (32 each: Infernal's arcana is for charms), given Infernal, Celestial, Fey and returned Fey, Infernal, Celestial. The run's MR 4 slot holds Expanded Arsenal (Conjuration) and MR 2 Spell Focus (Mythic) (Enchantment), so the `improve` test first withdraws MR 2: then MR 4 lists Spell Focus (Mythic) (Enchantment) first, 33 → 34.
9. **The instructions:** "Report its differences as facts, without ranking them" now ends "by your own judgement", and a paragraph "Ranking by a number" says a ranking by a number the planner computes is a fact, names `by` and `improve`, and asks the agent to say what the number leaves out and to label "better overall" as judgement. GLOSSARY: **Ranking number**.

## Progress log

- 2026-11-09: read the brief, the map's Notes, this ticket, 119, 114's MCP side (`attack.rs`, `caster.rs`, `compare.rs`, `options.rs`, `choose.rs`) and 83's timings.
- Built `apps/mcp/src/rank.rs`: `NumberArgs` (the `by` argument), `Number` (read at a Build's last level from `Engine::caster_line`/`Engine::attack_line`, described in words, with its line's `leaves_out`), `compare`'s order, and `improve`. `compare.rs` reorders the Builds by the number; `shapes.rs` gains `CompareArgs::by`, `CompareResult::by`, `BuildSummary::{number, given}`; `server.rs` the `improve` tool, `compare`'s description and the instructions.
- Timings, release build, the real data, the run's Build, `by` the enchantment compulsion DC (each the second of two calls through the MCP client):

  | choice | options tried | closed | ms |
  | --- | --- | --- | --- |
  | `slot.feat.19.general.0` | 241 | 655 | 35.7 |
  | `level.20.class` | 165 | 62 | 27.4 |
  | `start.race` | 43 | 5 | 9.1 |
  | `slot.mythic.4` | 23 | 136 | 5.1 |
  | `slot.mythic.2` | 23 | 136 | 5.2 |
  | `level.12.increase` | 6 | 0 | 1.3 |
  | `slot.feat.7.class.0` | 1 | 37 | 0.7 |

  For scale, `options` on `slot.feat.19.general.0` with `values: true` is 4.0 ms. No limit needed (choice 3).
- Server tests `apps/mcp/tests/server/ranking.rs` (9): `compare` by the compulsion DC orders three bloodline branches; without `by` nothing changes; `improve` on MR 4 lists Spell Focus (Mythic) first; a held pick's replacements show their loss; Favorable Magic is `no_change`; spell penetration and `query`; the attack line on the test world (`improve` and `compare`); refused arguments; the instructions. `listing.rs` lists `improve`. `cargo test -p mcp --test server`: 120 passed.
- Not in the web app: the ticket is about the MCP tools; `Engine` needed no change, so no behaviour proof (nothing in `rules` or `libs/game-data/` changed) and no migration or query change.
- `cargo fmt --all --check` and `scripts/check.sh`: ALL-OK (fmt, clippy, wasm clippy, the workspace tests, the rules wasm build, the seed check).

## Answer

Done 2026-11-09 (B, then C). `compare` takes `by`, a number of the caster line (`dc` of the highest spell level castable, `spell_penetration`, `concentration`) or of the attack line (`attack`, the first attack's bonus; `damage`, the bonus per hit), and orders the Builds by it at their last level, highest first, each summary with its `number` and `given` place, the result with `by` (the number in words, the order, what it leaves out). The new tool `improve` takes a Build, a choice's address and the same `by`, takes every option open there by the engine's own edit and returns those that change the number, highest first with their change, then `no_change` and `no_line` by name, `closed` and `leaves_out`. On the Max DC run's Build, `compare` by the compulsion DC puts the Fey branch (34) before Infernal and Celestial (32), and `improve` on MR 4 (MR 2's Spell Focus (Mythic) withdrawn) lists Spell Focus (Mythic) (Enchantment) first, +1. A slot of 241 open options costs 36 ms (release), so no limit. The instructions say a ranking by a number the planner computes is a fact, and "better overall" stays the agent's judgement.
