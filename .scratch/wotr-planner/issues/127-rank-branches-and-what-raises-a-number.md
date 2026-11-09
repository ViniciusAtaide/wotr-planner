# Rank branches by a number, and list what raises it

Type: design
Status: open
Blocked by: [119](119-caster-line.md) (the caster line is the number this run needed; the attack line from [114](114-attack-line-and-damage.md) already exists)
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
