# An MCP server for building with the rules engine (plan)

Type: design
Status: resolved (built in tickets 105–111; the user's own run of the prompt is left, ticket 111)
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user wants to ask an agent for a Build in plain words, for example: "a BFT main character, buffs and summoning, Lich or Aeon (compare), summons lasting 24h through caster level 25 and Extend, buffed with the mass animal buffs". The agent should build it through an MCP server, and it must not be able to produce an impossible Build. This ticket plans that server. Spells and companions stay out of scope (map, Out of scope).

## 1. What the engine can check for that prompt

| Part | Engine today | Notes |
| --- | --- | --- |
| "Main char" | n/a | A Build is one character. Companions are out of scope. |
| "BFT" | Not a class key | By initials it's **Brown-Fur Transmuter** (`arcanist/brown-fur-transmuter`: Powerful Change at 3, Shared Transmutation at 9), which fits "buffs". "BT" would be **Beast Tamer** (`bard/beast-tamer`: Summon Nature's Ally features, Augment Summoning granted at 2). The agent asks the user, and `search` matches initials. |
| Class, archetype, levels, prestige entry | **Full** | `class_options` / `advances_options`: entry prerequisites, table caps, alignment, faith. |
| Summon and buff feats | **Full** | Spell Focus (Conjuration) → Augment Summoning → Superior Summoning, Metamagic (Extend Spell), exploits, slots, order, repeats. |
| Mythic picks | **Full** | Enduring Spells, Greater Enduring Spells, Ascendant Summons, Spell Focus (Mythic), and path picks (Lich powers, Aeon gazes). It checks only the prerequisites the data holds. |
| Lich or Aeon | **Partly** | The engine checks the path at MR 3, the first ascension at MR 1 (a separate choice) and each path's picks, and lists each path's features by rank (Lich's Summon Spectre…, Aeon's golems, Summoner's Gaze, Caster Level gaze). Which path is *better* is not modelled. |
| "Focus on buffs and summoning" | **None** | It's a goal, not a rule. The engine checks picks; it doesn't rank them. |
| 24h summons: CL 25 + Extend | **None** | Spells, durations and metamagic effects aren't modelled. `CharacterState::caster_level` counts class levels only, for prerequisites; mythic, gear and buff CL bonuses aren't in it. |
| Mass animal buffs | **None** | These are spells (out of scope). The engine gives only spell-level access ("arcane 9th from level 17"). |

**The honesty boundary** (the tool descriptions and the server's `instructions` say it):
- **The agent may claim as fact:** every choice in the Build is legal under the planner's rules, at the level it's taken; the link opens in the planner; the features, numbers (BAB, saves, scores, ranks, class CL, spell-level access) and path features the sheet lists.
- **The agent is reasoning outside the validator when it talks about:** what a feature or spell does, spell availability, durations, CL arithmetic, item or buff effects, summons' stats, and "which is better". It must label all of that as its own game knowledge. The data holds names, prerequisites and guide anchors, and almost no summaries (only one non-empty summary in `libs/game-data`). The agent gives the user guide links and never fetches them (robots.txt).

## 2. Architecture

- **Where:** a new `apps/mcp` package, binary `wotr-mcp`, as a workspace member. **Rust**, so it calls `rules` directly: the same engine the browser runs as wasm, with no wasm bridge and no second copy of the rules.
- **SDK:** the official `rmcp` **3.9.0** (crates.io, 2026-11-05; MSRV 1.88, edition 2024, so the pinned nightly builds it), with features `server`, `macros` and `transport-io`. Tools are declared with `#[tool_router]` / `#[tool]`, and arguments are `Parameters<T: JsonSchema + Deserialize>`. It implements MCP spec 2026-07-28. Sources: github.com/modelcontextprotocol/rust-sdk and crates.io.
- **Transport:** **stdio** only, launched by Claude Code or Desktop. HTTP isn't worth it: the site is static, with nowhere to host a server. Because the server holds no state between calls (below), Streamable HTTP can be added later without a redesign.
- **Game data:** the `.ron` files read at startup with `seeds::read_game_data_files` + `GameData::check` (`--data <dir>`, default `seeds::game_data_dir()`). Rejected: SQLite (must be seeded first, async, a second copy); the site's JSON as default (network, and the site lags the checkout); embedding at build time (suits a distributed binary nobody needs yet). The server prints the hash the site names (`game-data/<hash>.json`), so drift from the deployed site is visible.
- The server keeps `GameData` for search and `Arc<Engine>` for the rules. Tool calls run on `spawn_blocking`.
- `rules` types aren't serde (`Edit`, `Sheet`, `CharacterState`), so the crate defines its own JSON shapes. `rules` doesn't change.

## 3. Tools

**State: the Build code is the handle.** Every tool takes `build` (the link code, `#b=` stripped) and returns the new code. Server-side sessions were rejected: the code already is the export, branching is free (keep two codes), nothing is lost on a restart, and no store is needed. Codes are short (ticket 46).

Every response carries `{ build, link, todo }`. `link` is `https://viniciusataide.github.io/wotr-planner/#b=<code>`. `todo` lists the open choices: empty slots, unchosen start fields, an unchosen path once MR 3 is reached, and unspent skill points.

**Choice addresses** are strings the sheet prints and the agent copies:
- `start.race|faith|alignment|background|racial-ability`, `start.score.int`
- `level.4.class` (one past the end adds a level), `level.4.increase`, `level.4.skill.arcana`
- `mythic.first-ascension|path|switch`
- `slot.feat.3.general.0`, `slot.selection.1.0`, `slot.mythic.5`, …: every `SlotId`

**Values** are a key (`"metamagic-extend-spell"`), `{key, parameter}` (`{"key":"spell-focus","parameter":"conjuration"}`), `{class, archetype?, advances?}` for a class, or a number for a score or ranks.

| Tool | Input → output | Engine calls |
| --- | --- | --- |
| `search` | `{query, kinds?, limit?}` → `[{kind, key, name, parent?, category?, prerequisite, guide_url}]`, matching names, keys and initials | `GameData` lists, `prerequisite_text`, label fns |
| `describe` | `{key, kind}` → the item, plus class/archetype rows by level or path features by rank and its selections | `effective_row`, `path_rank_features`, `path_summary` |
| `new_build` | `{name?, link?}` → envelope. A `link` is opened only through `open_link` | `open_link`, `encode_build` |
| `options` | `{build, choice, query?, only_open=true, values=false}` → `[{value, name, status: "ok" \| {issue, reason}}]`, plus a `closed` count | `slot_entries`, `class_options`, `advances_options`, `race/faith/alignment/background/racial_ability_options`, `ability_increase_options`, `point_buy_steps`, `skill_steps`, `path/first_ascension/switch_options`, `issue_text` |
| `choose` | `{build, steps:[{choice, value}]}` → envelope + `{applied, rejected?: {step, issue, reason}}`. It stops at the first rejection and returns the code as it stood | the option's own `edit` → `Engine::apply` → `is_possible` |
| `withdraw` | `{build, choice}` → envelope, or `not_removable` | `withdrawal` |
| `sheet` | `{build, removable=false}` → per level: class, class level, BAB, saves, scores, ranks, feats, features, picks by address; per rank: status, path, picks, features; class CL, spell-level access, daily-choice notes | `evaluate`, labels, `removable` |
| `compare` | `{builds:[a,b,…]}` → each sheet's summary, plus the features, picks and numbers that differ, by level or rank | `evaluate` ×n |

**Issue codes:** the engine's own (`unmet`, `already_held`, `max_times`, `class_level_cap`, `not_from_path`, `deity_alignment`, `needs_faith`, `deity_domain`, `breaks_later`), each with `issue_text`, plus the server's (`not_an_option` with the closest names, `no_such_choice`, `not_removable`, `stale_link`).

**Comparing Lich and Aeon:** build to MR 2 once. Branch with `choose mythic.path` to `lich` and to `aeon`, fill each branch, then call `compare`. The agent states the engine's facts and labels its verdict as judgment.

## 4. Resources and prompts

- **The server's `instructions`:** the workflow (resolve names → ask about ambiguities → start → levels → mythic → links) and the honesty boundary. Every client sees these.
- **One resource:** `wotr://glossary` (GLOSSARY.md). It earns its place because the tools speak in Slot, Withdrawal and Path switch.
- **No prompt yet:** the instructions cover the workflow (question 4).

## 5. The guarantee

- **No input Build is trusted.** Every incoming code goes through `open_link`, which decodes it and runs `is_possible`. An impossible code is `stale_link` before any work.
- **No edit comes from the agent.** `choose` recomputes the choice's options, finds the offered option matching the value, requires `Ok` (prerequisites and `breaks_later`), applies *that option's* `Edit` with `Engine::apply`, and then checks `is_possible` again as a final gate. Steppers move one step at a time, and every step is checked the same way.
- `withdraw` applies only `Engine::withdrawal`'s edit, and only when it's removable.
- **Gaps:** a hand-made code that happens to be possible is accepted (fine: it's a possible Build); a stale code gets no reason (ticket 40); a refused withdrawal doesn't name its dependant (`sheet removable=true` shows the leaves); local data and the site can drift, so a link valid here may be stale there (the hash shows it); and the guarantee reaches only as far as the data.

## 6. Testing

- **Through MCP:** an rmcp client over `tokio::io::duplex` on the small test world (moved into `rules` behind a `test-world` feature, a move only). Cases: initials search; statuses and reasons; valid `choose` → new code; invalid → `unmet`, code unchanged; unoffered value → `not_an_option`; a batch stops at the first failure; a forged impossible code → `stale_link`; `withdraw` refused on a non-leaf.
- **Property test:** 1,000 random `choose` walks over offered values; every returned code passes `open_link`.
- **End to end, real data** (`apps/mcp/tests`): "BFT" finds Brown-Fur Transmuter; a 20-level BFT arcanist with Spell Focus (Conjuration), Augment Summoning, Superior Summoning and Extend Spell (Augment Summoning first → `unmet`); Lich and Aeon branches to MR 10 with Enduring and Greater Enduring Spells; both links open; `compare` lists the path differences.
- **By hand:** run the user's prompt in Claude Code; the transcript's claims stay inside the boundary and the link opens on the site.

## 7. Slices

Each runs one at a time, in order.

- **105 Test world behind a feature.** The world moves into `rules` behind `test-world`, and the engine tests use it from there. *Done when:* the same tests pass and `scripts/check.sh` passes.
- **106 Scaffold `apps/mcp`.** It loads the RON with `check`, prints the hash, and offers `search` and `describe` over stdio. *Done when:* the tests pass, `claude mcp add` connects, and "BFT" finds the archetype.
- **107 Codes, addresses, `new_build`, `sheet`.** It adds the address grammar both ways, the envelope and `todo`. *Done when:* round-trip tests of every `SlotId` pass and `sheet` works on the world.
- **108 `options`.** Every picker and stepper, with filters and issue codes. *Done when:* the status and reason tests pass.
- **109 `choose` and `withdraw`.** *Done when:* the rejection, batch and stale-link tests pass, and so does the property test.
- **110 `compare`, instructions, glossary resource.** *Done when:* the compare tests pass and the boundary text is reviewed by the user.
- **111 End to end on real data, plus a manual agent run.** *Done when:* the scenario above passes, and the user has run the prompt.
- **After each slice:** run `scripts/check.sh`, and add a line to the map's Decisions.

## Open questions (with recommendations)

1. **What does "BFT" mean?** *Recommend:* Brown-Fur Transmuter (arcanist). The agent still asks whenever a name is ambiguous.
2. **Stateless codes or server sessions?** *Recommend:* stateless codes.
3. **Data source?** *Recommend:* RON at startup. Add a `--check-site` hash comparison only if drift bites.
4. **A "make a build" prompt?** *Recommend:* not yet. Add it if agents skip steps.
5. **HTTP transport?** *Recommend:* no; stdio only.
6. **Should a locked withdrawal name what depends on it?** *Recommend:* not now. If it's needed later, it's an engine ticket.
7. **Bring spells into scope** (so "mass animal buffs" can be checked)? *Recommend:* no. The boundary covers it.
8. **Check in a project `.mcp.json`?** *Recommend:* yes, launching `cargo run -p mcp --release`.

## Settled

The user approved the plan (2026-11-08): "if the plan is solid … start implementing it with the recommended approach and deploy it". All eight recommendations above are taken. "Deploy" for a stdio server means it's on `main`, pushed, and launchable from the checked-in `.mcp.json`; GitHub Pages can't host it.

Order: 106 runs first, because 105 moves the test world and would collide with ticket 103, which is still in progress. 105 follows once 103 lands, then 107–111 in order.
