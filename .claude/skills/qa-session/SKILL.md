---
name: qa-session
description: Test the wotr-planner through its MCP tools as a careful agent would use them, and report what is wrong. Use after a queue of tickets is merged, or when asked for a QA session.
---

# A QA session through the MCP tools

You test and report. You edit no file and run no cargo command. Read the `wotr-build` skill first: the workflow, the addresses, the honesty boundary.

## Steps

1. **Is the server current?** It reads the game data once, at startup. Pick one fact from the newest data change and one from the newest code change, and check both through the tools. If the data is older than the tickets under test, stop and say so.
2. **Each ticket does what it says.** For every ticket named, read its file and `## Answer` in `.scratch/wotr-planner/issues/`, reproduce the original problem's steps, confirm the fix, then push at its edges: one level below, one above, a rank not reached, another class or archetype that must not be offered the thing, the same value through `options`, `choose`, `validate` and `withdraw`.
3. **A regression sweep.** Build four characters from scratch to level 20 with a Mythic path, each through `search`, `new_build`, `validate` of the whole map, `choose` for what is open, `sheet`: a martial, an arcane caster with a prestige class, a multiclass that borrows selections, and a Legend to level 40 or a Build that moves Mythic ranks later. Along the way use `options` with `only_open: false` and with `values: true`, `withdraw` on a choice something depends on, `compare` of two variants and `improve` by a number.
4. Done when every ticket has a pass or a fail for each thing checked, and all four Builds end with an empty `todo` or a reason they can't.

## What to look for

- A value one tool offers and another refuses.
- A refusal that is untrue, names the wrong pick, level or rank, or carries a hint that doesn't apply.
- A text that reads oddly: a doubled word, a raw key where a name belongs, "(s)", a pronoun with two possible referents.
- A `todo` entry nothing can fill.
- A `sheet` that disagrees with what was chosen; a number that is wrong against the guide's rule (check a few by hand).
- An error, a panic, a timeout, a slow call (say which and roughly how long).

## Report

1. The server check.
2. Per ticket: pass or fail for each thing checked.
3. The four Builds' links.
4. Every problem, most serious first: the exact steps (the Build's code, the tool, the arguments), the exact output, what you expected and why, and whether it is new or older than the tickets under test, with how you judged.
5. What you did not test.

Give exact inputs and outputs: the fixes are built from this report. Label what you inferred.
