# MCP server: `options`

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 108 of the [MCP server plan](104-mcp-server-plan.md): `options {build, choice, query?, only_open = true, values = false}`, over every picker and stepper the engine has, with the engine's issues as codes and its reasons. *Done when:* the status and reason tests pass.

## Choices settled

1. **Result:** `{options: [{value, name, status, category?, takes?, current?}], closed, choice}` plus the envelope. `status` is `"ok"` or `{issue, reason}`. `closed` counts the options `only_open` hid (after the query); with `only_open: false` it's 0. `choice` echoes the address as the server writes it. `current: true` marks what the Build holds now.
2. **Every picker and stepper, from the engine** (`options.rs`, `offers`):
   - a slot: `Engine::slot_entries`. With `values: false` a parameterised entry is one line, `value` its key, `takes` its noun ("a weapon"), open when any value is, and its issue chosen as the engine chooses its one reason (the first failing value's that isn't already holding it); with `values: true`, one line per value, `{key, parameter}`, each with its own status and the entry's reason for it;
   - `level.N.class`: `class_options`; a spellcasting prestige class first taken there (`asks_advances`) is offered once per `advances_options` answer, `{class, archetype?, advances: [...]}` (`[]` for none), and if no answer keeps the Build possible, once as `breaks_later`;
   - `start.race` (`race_options`: races and subraces), `start.faith`, `start.alignment`, `start.background`, `start.racial-ability` (none for a race without the choice: `no_such_choice`);
   - `level.N.increase`: `ability_increase_options`;
   - steppers: `start.score.<ability>` (`point_buy_steps`) and `level.N.skill.<skill>` (`skill_steps`) give the step down and the step up that exist, valued by the score or the ranks they'd set; `level.N.skill` gives each skill's step up, valued by the skill's key (what `todo` points at for unspent points);
   - `mythic.first-ascension|path|switch`: `first_ascension_options`, `path_options`, `switch_options`, only once the Build reaches MR 1, 3 or 8 (the engine's pickers don't check the rank themselves; the web app shows them only on reached ranks).
   - `start.subrace`, `level.N`, `class.<class>.archetype|advances` exist for `sheet`'s marks and 109's `withdraw`; they have no picker of their own, so `options` answers `no_such_choice` naming the picker that sets them.
3. **Issue codes:** each `Issue` variant in snake case (`unmet`, `already_held`, `max_times`, `not_allowed`, `class_level_cap`, `not_from_path`, `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`), with `Engine::issue_text` as the reason. `breaks_later` has no engine text (the web app greys it out), so the server words it: "Would break a later choice, which must be withdrawn first (`sheet` with `removable: true` shows what can come off now)". The server's own codes: `no_such_choice` (not an address, or a choice the Build lacks, with what the Build has) and `stale_link`.
4. **Values** are 107's `ChoiceValue` (`shapes.rs`), unchanged: what `options` lists is exactly what the sheet prints and what 109's `choose` takes, and `ChoiceValue::matches` compares them.
5. **Each offer keeps its engine edit** (`Offer::edit`), so 109's `choose` recomputes the offers, finds the one whose value matches, requires `Ok`, and applies *that* edit. A parameterised entry listed without values has no edit: `choose` needs the `{key, parameter}`.
6. **The query** keeps the options whose name or value text holds every word of it, case-insensitively.
7. **No `rules` change.**

## Progress log

- 2026-11-08: Built on 107 (013bd0b). Wrote `offers` over the engine's pickers, the tool, and `tests/server/options.rs` (15 tests, one on real data), written with the code rather than strictly red first. All passed on the first run, apart from a clippy lint in the test helper.
- Smoke test: the release binary, piped `initialize`, `notifications/initialized`, `new_build {name}`, then `options` on `level.1.class` with and without `query: "arcanist"`. Arcanist and its six archetypes come back `ok` ("Brown-Fur Transmuter (Arcanist)", …), and the full picker lists 140 open and 95 closed.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. `options` covers every engine picker and stepper, lists the open options by default and counts the rest, gives each closed option its engine issue code and reason, and filters by query; `values: true` lists a parameterised feat's values. 43 server tests pass (15 new). 109 should find its offer with `options::offers` and `ChoiceValue::matches`, and apply the offer's own edit.
