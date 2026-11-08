# MCP server: `choose` and `withdraw`

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 109 of the [MCP server plan](104-mcp-server-plan.md): the two tools that change a Build. `choose {build, steps: [{choice, value}]}` and `withdraw {build, choice}`, built so that the plan's section 5, "The guarantee", holds: no input Build is trusted, no edit comes from the agent, and an impossible Build never comes back. *Done when:* the rejection, batch and stale-link tests pass, and so does the property test.

## Choices settled

1. **`choose`'s result:** the envelope plus `applied` (the steps applied, from the first) and, when a step is refused, `rejected: {step, choice, issue, reason}`. A refusal is a successful result with `rejected`, not an error, because the steps before it did apply. `step` is the index in `steps`, from 0, so it always equals `applied`. The code is the Build as the last applied step left it. Only a code that doesn't open (`stale_link`) or arguments that don't parse fail the whole call.
2. **One step** (`choose.rs`, `take`):
   1. parse the address, else `no_such_choice`;
   2. recompute the offers with values, `options::offers(…, values: true)`, so every offer carries its edit (a `no_such_choice` from it is the step's rejection);
   3. find the offer whose value matches (`ChoiceValue::matches`), else `not_an_option` with up to 5 closest offered values;
   4. an offer the Build already holds (`current`) is a no-op, counted as applied;
   5. require `Ok`, else the engine's issue code and reason (the offer's own reason for a slot, `issue_text` otherwise, the server's wording for `breaks_later`);
   6. apply **that offer's own `Edit`** with `Engine::apply`;
   7. **the final gate** (`code::reopen`): encode the result and open the code again with `Engine::open_link`. It decodes and runs `is_possible`, so what comes back is exactly what the code holds and is possible. If it fails, the step is refused as `internal`, naming the step, and nothing is applied. The walk afterwards continues from the decoded Build.
3. **Steppers.** `start.score.<ability>` and `level.N.skill.<skill>` with a number move to it one stepper step at a time, recomputing the offers after each, and each step goes through the same open check and the same final gate. A step the stepper doesn't offer is `not_an_option` with the stepper's limits (7 to 18 within the 25-point budget, with the points spent; skill points and the character-level cap). A step that isn't `Ok` (`breaks_later`) gives the engine's issue. Either way the reason names the step ("At Intelligence 16 → 17: …"), and **the whole move is rolled back**: the step's code is as it stood before it, as for any refused step. The current value is a no-op, counted as applied.
4. **Prestige classes.** A class value without `advances`, where the class picker offers that class only with `advances`, is `not_an_option`: "asks which classes its levels advance: give `advances` too, `[]` for none", listing the offered answers (`["wizard"] (Eldritch Knight, advancing Wizard)`, `[] (…, advancing nothing)`).
5. **One leniency in matching:** a class already taken with an archetype is offered at later levels only with it (`{class: "arcanist", archetype: "arcanist/brown-fur-transmuter"}`), so `{class: "arcanist"}` alone names that offer when it's the class's only one. A class not yet taken is offered without an archetype too, and that offer matches first. It's still the offer's own edit.
6. **Closest values** (`not_offered`): offers ranked by the words they share with the given value (prefixes count), then edit distance between the value texts, then open before closed, then the picker's order. Each is written `Name (`value as JSON`)`, with ", not open now" on a closed one, so the agent can copy the value. The reason also suggests `options` (with `values: true` for parameterised feats). A value of the wrong shape (a key where a class object is wanted) gets the right shape back this way.
7. **`withdraw`** (`withdraw.rs`) uses only `Engine::withdrawal(&build, &address.choice())`, and only when that says removable; then it applies that edit and passes the same final gate. Its result is the envelope alone.
   - `not_removable` when something depends on it: "withdraw what depends on it first. `sheet` with `removable: true` lists what can be withdrawn now." (The plan's open question 6: the dependant isn't named.)
   - `start.racial-ability`: refused (`not_removable`), since the engine keeps it with the race. Change the race, or choose another ability.
   - `level.N.class`: refused (`not_removable`) as settled. To change a level's class, `choose` another one; to drop the last level, withdraw `level.<last>`.
   - A choice the Build doesn't hold: `no_such_choice`. `level.N` for a level that isn't the last says only the last can be withdrawn.
8. **`sheet {removable: true}` now matches `withdraw`:** `level.N.class` isn't in the top-level `removable` list, and its pick line has no `removable` mark. Before this, the sheet listed choices `withdraw` refuses.
9. **A fix to `options` (108), found by the property test:** the engine's class picker offers `level.21.class` on a 20-level Build without a Legend switch, and a 21-level Build isn't possible. The web app hides the add-level control past `sheet.level_cap` (`libs/ui/src/grid.rs`), so the engine leaves that check to its caller. `options` (and so `choose`) now answers `no_such_choice` for a level past the cap: "Level 20 is the last level a Build can have". The final gate had already refused the step as `internal`, so no impossible Build came back even before the fix.
10. **Annotations:** both tools are `read_only_hint: true`, like `new_build`. They change nothing outside the call: a Build is a code in and a code out.
11. **Errors** (`error.rs`): `not_an_option`, `not_removable` and `internal` join `stale_link` and `no_such_choice`.
12. **The instructions** say that `choose` and `withdraw` exist, and that every choice in a Build these tools returned is legal. The old "once the building tools exist" is gone. Ticket 110 finishes the text.
13. `Offer::edit`'s `#[expect(dead_code)]` is gone. **No `rules` change.**

## Progress log

- 2026-11-08: Fast-forwarded the worktree to `main` (576ea52). Read the plan, tickets 105 to 108, and the engine's `withdrawal`, `held_choices`, `apply`, `open_link` and steppers.
- Red first: wrote `tests/server/choose.rs` and `withdraw.rs`. All 17 failed with "tool not found".
- Wrote `choose.rs`, `withdraw.rs`, the shapes and the errors. 16 of 17 passed. The real-data test failed: `{class: "arcanist"}` at level 2 didn't match, because a class taken with an archetype is offered only with it. That led to settled choice 5.
- Wrote the property test (`tests/server/walks.rs`). It passed at once from empty Builds, but only 4 of 15,000 codes reached a Mythic path. Each walk now starts with 0 to 12 random levels. That run failed at `level.21.class`, with the final gate refusing the step as `internal`, which led to settled choice 9. Then 500 walks of 20 steps took about 52s, with good coverage (below).
- The smoke test showed the sheet listing `level.N.class` as removable while `withdraw` refuses it, which led to settled choice 8.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.
- Smoke test of the release binary over JSON-RPC: `initialize`, `tools/list` (7 tools), `new_build`, then a 7-step `choose`. It stopped at step 6, Augment Summoning `unmet`, "Needs Spell Focus (Conjuration)". Then an 11-step batch: race, racial ability, faith, alignment, Int 17, BFT 1, Spell Focus (Conjuration), arcanist 2 and 3, Augment Summoning at 3, then Int 19, which was refused as "At Intelligence 18 → 19: there is no such step …". `withdraw` refused Spell Focus (`not_removable`) and `level.3.class`, and took `level.3` back. `sheet` showed the picks.

## Answer

Done 2026-11-08. `choose` applies steps in order, stopping at the first refused one. Each step takes only an offered, open option, applies that offer's own engine edit, and passes the final gate (the new code reopened through `open_link`). Steppers move one checked step at a time. `withdraw` applies only the engine's Withdrawal, and only when it's removable. It refuses `level.N.class` and `start.racial-ability` and says what to do instead. 62 server tests pass (19 new). Among them is the property test: 500 random walks of 20 steps over the test world (seed `0x5EED0109`). The walks made about 10,300 applied choices, 250 refused closed options, 400 applied and 80 refused score moves, and 1,000 withdrawals, with 527 codes holding a Mythic path, and every returned code opened. It found one gap, in `options` (settled choice 9), which is now fixed.

For 110 and 111:
- `compare` can take `choose`'s codes as they come. Branching is two `choose` calls from one code (`mythic.path` `lich` and `aeon`).
- A level past the cap is `no_such_choice`. A Legend switch raises the cap to 40, and `sheet.level_cap` shows it.
- The instructions should tell agents to batch steps, to read `rejected`, and to use `options` with `values: true` for parameterised feats.
