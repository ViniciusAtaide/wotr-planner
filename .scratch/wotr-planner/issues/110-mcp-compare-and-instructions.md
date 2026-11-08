# MCP server: `compare`, the finished instructions, the glossary resource

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 110 of the [MCP server plan](104-mcp-server-plan.md): `compare {builds: [code, …]}`, so an agent can set two branches of one Build side by side (a Lich branch and an Aeon branch) and report the facts; the server's `INSTRUCTIONS` finished; and the `wotr://glossary` resource. *Done when:* the compare tests pass and the user has reviewed the boundary text.

## Choices settled

1. **`compare`'s input:** `{builds: [code, …]}`, 2 to 4 codes (links work too). Each is opened only through `open_link`, like every Build tool's code. Fewer or more is `invalid_arguments`, a new server code. A code that doesn't open is `stale_link`, and its reason names the code: "builds[1]: The Build code doesn't open …".
2. **Everything is read from the sheets.** `compare` evaluates each Build once and writes its sheet with `sheet`'s own code (`sheet::sheet_of`, split out of `sheet` for this), then diffs those sheets. So a difference `compare` reports is one the two `sheet` results show, in the same names and addresses.
3. **The result** (`shapes::CompareResult`). Every per-Build list follows the order of `builds`.
   - `builds`: a summary per Build at its last level. It has `build`, `link`, `name?`, `open_choices` (its `todo`'s length), `character_level`, `classes` (`{class, archetype?, levels, advances?}`), `bab`, `fort`, `reflex`, `will`, `scores`, `caster_level` (class levels only, as `sheet`), `spell_levels`, `mythic_rank`, `first_ascension?`, `path?`, `switch?` and `level_cap`.
   - `start`: the Build start's choices that differ, as `PickDiff {choice, label, names}`, with one name per Build (`null` where it's open or the Build lacks it).
   - `levels`: per character level where anything differs, `{level, numbers?, picks?, feats?, features?}`.
     - `numbers` holds `{what, values}` for `class_level`, `bab`, the saves and the six scores, with `null` where a Build lacks the level.
     - `picks` holds the level's choices by address (class, increase, ranks added, slots).
     - `feats` and `features` hold one list per Build: what it gains at that level that some other Build doesn't.
   - `ranks`: per Mythic rank where anything differs, `{rank, status?, path?, picks?}`. `status` and `path` appear only when they differ.
   - `path_features`: per rank, `{rank, only}`, where `only` holds one list per Build of the path features its sheet lists at that rank and some other Build's doesn't.
4. **"Differs" for lists means "not every Build has it".** For two Builds that is the two sides of the symmetric difference. For three or four, an item held by only some Builds is listed under each Build that holds it. Items compare by key and name, so the Lich's Hard to Kill and the Aeon's (`lich/hard-to-kill`, `aeon/hard-to-kill`) are different features. They show as "Hard to Kill (Lich)" and "Hard to Kill (Aeon)" (`feature_name_apart`), as the sheet shows them. When the branches share a first ascension, MR 1–2 come from the same path and those features drop out of the comparison.
5. **Path features aren't listed twice.** A level's `features` on the sheet include the path features its Mythic ranks give. `compare` leaves those (every key in the Build's rank features) to `path_features`, so a Lich-or-Aeon comparison shows them once, by rank.
6. **A pick whose label differs** between Builds (`slot.path-selection.3.0` is "Lich Powers" on one branch and "Enforcing Gaze" on the other) gets each label once, joined by " / ".
7. **No ranking, and no verdict.** The tool's description and the result's doc say it gives facts only.
8. **`INSTRUCTIONS`, finished** (`server.rs`; the full text is in the Answer). The text is short and imperative. It covers:
   - the six-step workflow, with "BFT" as the example of asking;
   - batching and reading `rejected`;
   - values: `options {values: true}`, `advances`, and a number for a stepper;
   - branching, then `compare`;
   - the honesty boundary from the plan's section 1, as three bullets;
   - a pointer to the glossary.

   The phrases the old test looks for ("legal", "own game knowledge", "never fetch") stay. A new test checks that the workflow steps come in order and that the batching, values, branching and glossary phrases are there.
9. **The resource `wotr://glossary`** serves `GLOSSARY.md` as `text/markdown`.
   - The text is embedded with `include_str!("../../../GLOSSARY.md")` (`mcp::GLOSSARY`), so the binary needs no file at run time. A test checks that it equals the file.
   - The capabilities list resources (`enable_resources`).
   - `resources/list` gives the one resource, with its title, description, MIME type and size. `resources/read` of any other URI is a `resource_not_found` protocol error, as MCP has it for resources.
   - No resource templates, no subscriptions: the text changes only when the binary is rebuilt.
10. **No `rules` change.**

## Progress log

- 2026-11-08: Fast-forwarded the worktree to `main` (d5f6c23). Read the plan, tickets 105 to 109, and rmcp 3.9.0's resource API (`ServerHandler::list_resources`/`read_resource`, `Resource::new`, `ResourceContents::text`, `ReadResourceResponse`).
- Wrote `compare.rs`, its shapes, and `tests/server/compare.rs` (6 tests on the test world) with the code. Two test expectations were wrong at first: the test world's feats are named by their keys ("power-attack"), and a path row's features are in key order. The first run also showed the path features listed twice (under the levels and under `path_features`), which led to settled choice 5. Branches with both MR 3 slots open showed no pick difference (both `null`), so the branch test now fills them, which led to settled choice 6.
- Rewrote `INSTRUCTIONS` and added the glossary resource. `tests/server/listing.rs` gains three tests: the workflow, the resource listed and read (equal to `GLOSSARY.md`), and an unknown URI refused.
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. `compare` takes 2 to 4 codes, opens each checked, and returns each Build's summary plus only what differs: the Build start's choices, by level the numbers, choices, feats and features, by rank the status, path and choices, and `path_features`, by rank, the path features each Build has that another lacks. All of it comes from the same sheets `sheet` returns. The server lists and serves `wotr://glossary`, embedded at build time. 71 server tests pass (9 new).

The finished instructions, for the user's review:

> wotr-planner builds Pathfinder: Wrath of the Righteous characters with the rules engine of a build planner. Every Build its tools return is one the planner's rules allow.
>
> Workflow:
> 1. Resolve every name the user gives with `search` (names, keys and initials: "BFT" finds the Brown-Fur Transmuter). If a name could mean more than one item, ask the user which one before you build.
> 2. Start with `new_build`. Every Build tool takes the Build's code (`build`) and returns a new code, the planner `link` and `todo`, the choices still open, each by its address.
> 3. Fill the Build start: race, racial ability, faith, alignment, background, point-buy scores.
> 4. Add levels with `level.N.class` (N one past the last level adds one) and fill each level's open choices: feats, Feature selections, ability increases, skill points.
> 5. Fill the Mythic ranks once their levels are reached: the first ascension at MR 1, the path at MR 3, and each rank's picks.
> 6. Give the user the final `link`.
>
> Batching: send many steps in one `choose`. It stops at the first refused step: read `rejected` (its issue and reason), fix that step, and resend the rest from the code it returned.
>
> Values: take them from `options`, which lists what a choice can take now and why the rest is closed. A parameterised feat needs `{key, parameter}`: list them with `options {values: true}`. A spellcasting prestige class needs `advances`, the classes it advances (`[]` for none). A point-buy score or a skill's ranks take the number to reach.
>
> Branching: for alternatives (Lich or Aeon), build the shared part once, `choose` each alternative from that same code, fill each branch, then `compare` the codes. Report its differences as facts, without ranking them.
>
> The honesty boundary:
> - State as fact only what the tools return: every choice in the Build is legal under the planner's rules at the level it is taken; the link opens in the planner; and the features, numbers (BAB, saves, scores, skill ranks, caster level from class levels, spell-level access) and path features the sheet lists.
> - Label everything else as your own game knowledge: what a feature, feat or spell does, spell availability, durations, caster level arithmetic, item and buff effects, summons' stats, and which choice is better. The planner checks none of it: its data holds names, prerequisites and guide links, and almost no descriptions.
> - Guide links (`guide_url`) are for the user to read: never fetch them yourself.
>
> The resource `wotr://glossary` defines the planner's terms (Build start, Slot, Withdrawal, Path switch, …).
