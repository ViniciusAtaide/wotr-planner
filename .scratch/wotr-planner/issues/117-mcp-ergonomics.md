# MCP ergonomics from the Demon Greatsword run

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The Demon Greatsword run (2026-11-08, see [112](112-effect-summaries.md)) took more calls than it needed. Some of them went to facts the server already gives.

| What happened | Cause |
| --- | --- |
| 45 one-rank skill steps to fill levels 6–20 | `level.N.skill.<skill>` takes one skill per step |
| A batch refused: alignment before faith | The `todo` order was right; the instructions don't say that faith comes first |
| A batch refused: scores assumed to start at 7 | `sheet` gives `point_buy_spent` and `point_buy_budget`, and the stepper names its points; the instructions don't say scores start at 10 |
| Not knowing which character level MR 4 falls at | `sheet` gives each rank's `character_level`; the agent never called it |
| Improved Critical (Mythic) without its weapon: `not_an_option`, with Sai, Bite and Bomb as the "closest" | A parameterised key without its parameter gets the general error |
| `compare` on the same code twice, just to get a one-Build summary | `compare` takes 2 to 4 codes |

## Scope

1. **Skill plans:** `level.N.skill` also takes a map, `{"athletics": 1, "perception": 1, "mobility": 1}`. Each rank is still one checked step, and the first refused rank stops the step as today.
2. **A missing parameter:** a parameterised key given bare is answered `needs_parameter`, with what it takes ("a weapon") and its open values.
3. **One-Build summary:** `compare` takes 1 to 4 codes; with one it returns the summary and no differences.
4. **Instructions, a "Before you plan" paragraph:**
   - scores start at 10 with a 25-point budget, and `sheet` shows what's spent;
   - the faith comes before the alignment;
   - `sheet` gives each Mythic rank's character level;
   - branch class splits and `compare` them before committing to one ([115](115-plan-ahead-class-splits.md), option A).

## Open questions (with recommendations)

1. **A level range for skills** (`level.6..20.skill`)? *Recommend:* not yet. With item 1, a 20-level plan is 20 steps, and a range adds an address form to the grammar.

## Done when

Server tests cover the skill map (with a refusal partway through), `needs_parameter`, and a one-code `compare`. The user reviews the instructions text, as in ticket 110. `scripts/check.sh` passes.
