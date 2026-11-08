# MCP server: Build codes, choice addresses, `new_build` and `sheet`

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Slice 107 of the [MCP server plan](104-mcp-server-plan.md): the Build code as the handle every Build tool takes, the envelope every such tool returns (`{build, link, todo}`), the choice address grammar both ways, and the first two Build tools, `new_build` and `sheet`. *Done when:* round-trip tests of every `SlotId` (and every other choice kind) pass and `sheet` works on the test world.

## Choices settled

1. **Opening a code** (`code.rs`): only through `Engine::open_link`, which decodes with the game data's keys and runs `is_possible`. Anything else (impossible, undecodable, empty) is `stale_link`, with one reason, as ticket 40 has it for the web app. `build` and `new_build`'s `link` take a bare code or a full link: the text after the last `#b=` (or a leading `b=`) is the code.
2. **The envelope:** `build` is `encode_build` of the Build, `link` is `https://viniciusataide.github.io/wotr-planner/#b=<code>`, `todo` lists `{choice, label}` in Build order:
   - the Build start's unchosen race, racial ability (only when the race chooses one), faith, alignment and background;
   - per level: its unchosen ability increase (every 4th level), unspent skill points (`level.N.skill`, while some skill is still below the character level), and its empty slots;
   - per reached rank: the first ascension at MR 1 while neither it nor a path is chosen (a Build without one takes its path's), the path at MR 3, and the rank's empty slots, except those a Path switch dropped.
   - Not listed: unspent point-buy points and the Path switch, which are optional. The ability increase isn't a slot but is an open pick like one, so it's listed.
3. **The address grammar** (`address.rs`, `mcp::Address`, `Display` + `FromStr`), refined from the plan's so every engine `Choice` has one (ticket 109's `withdraw` and `sheet removable` need them):
   - `start.race|subrace|faith|alignment|background|racial-ability`, `start.score.<str|dex|con|int|wis|cha>` (a full ability name reads too).
   - `level.N` (the level itself, `Choice::CharacterLevel`: withdrawing it removes the last level), `level.N.class`, `level.N.increase`, `level.N.skill` (the level's skill points as a whole, `Choice::SkillPoints`: what `todo` points at for unspent points), `level.N.skill.<skill>`.
   - `class.<class>.archetype`, `class.<class>.advances`: the engine keeps these on the class, not a level (`Choice::Archetype`/`Advances`), so their addresses do too.
   - `mythic.first-ascension|path|switch`.
   - Slots: `slot.feat.N.<race|class|general>.I`, `slot.selection.N.I`, `slot.option-feat.N.I`, `slot.option-selection.N.I`, `slot.feat-selection.N.I`, `slot.mythic.R`, `slot.mythic-extra.R`, `slot.path-feat.R.I`, `slot.path-selection.R.I`, `slot.mythic-selection.R.I`. Levels and ranks count from 1, indexes from 0.
   - Additions to the plan's grammar, and why: `start.subrace` (the engine's `Choice::Subrace`, withdrawn alone), `level.N`, `level.N.skill`, and the two `class.…` forms. `start.racial-ability` is the one address with no engine `Choice` (the engine keeps it with the race).
4. **Values** (`shapes::ChoiceValue`, written by `values.rs`), defined here because the sheet prints what each choice holds in the same form `options` (108) offers and `choose` (109) takes: a key, `{key, parameter}`, `{class, archetype?, advances?}` or a number. A parameter is the skill's or weapon's key, or a school, energy or armour type in lower case (`conjuration`); an alignment is its name in kebab case (`true-neutral`); a faith is a deity key or `atheism`; the race picker's value is the subrace's key when it has one (`half-elf/dual-heritage`, which starts with the race's); an ability is `str`…`cha`. A class's value carries `advances` only where the engine asks it (`Engine::asks_advances`: a spellcasting prestige class at its first level). `ChoiceValue::matches` compares a given value with an offered one (case-insensitive keys, advances as a set) for 109.
5. **Errors** (`error.rs`): a refusal is an error result whose text is `<code>: <reason>`, so an agent can branch on the code and read the reason. Structured error content was rejected: the tool's `outputSchema` describes the success shape, and a strict client may check error content against it.
6. **`new_build {name?, link?}`:** an empty Build, or the link's Build through the checked open; a `name` is set with `Edit::SetName` through `Engine::apply`. Returns the envelope alone.
7. **`sheet {build, removable?}`**, all from `Engine::evaluate`'s `Sheet` and the engine's labels:
   - `start`: each Build start choice as a pick line `{choice, label, value, name, removable?}` (open ones with `null`s), plus `point_buy_spent` of `point_buy_budget` (`PointBuy::cost`).
   - `levels`: class (and archetype), class level, BAB, saves and scores at the level's end, skill points (available, spent), total ranks by skill, the feats and features the level adds, Mythic ranks reached, and `picks`: class, increase, ranks added, then the slots in check order.
   - "What the level adds" is the one thing worked out here: the level's end state against the one before (a feat or a feat's value first held; a feature held more times). The engine has no per-level list (ticket 93 declined one for the web app), but its states say it exactly.
   - `ranks`: all ten, with status (`reached`, `partly_dropped`, `dropped`, `not_reached`), path, slots, `Engine::rank_features` and the dropped slots.
   - At the last level: `caster_level`, `spell_levels {arcane, divine}`, `class_skills`, `level_cap`, and `notes` (the background's note, `favoured_weapons_note`, `daily_choices`), as the web app's summary shows them.
   - `removable: true` runs `Engine::removable` once: each held pick line gets `removable: true|false`, and a top-level `removable` lists every withdrawable choice by address (including `level.N` and the `class.…` choices, which have no line).
8. **No `rules` change.** Every accessor needed was public.

## Progress log

- 2026-11-08: Fast-forwarded the worktree to `main` (9f5ee6f). Read the plan, the 105 and 106 tickets and the engine's `Sheet`, `Choice`, `held_choices`, pickers and labels.
- Wrote the address module, then its round-trip tests (`tests/server/address.rs`) with the Build tests (`builds.rs`, `sheet.rs`); the tests and the code were written together rather than strictly red first. One expectation was wrong on the first run: the test world's `build()` point buy costs 18 points, not the 21 its doc comment says (left as it is; not this ticket's).
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. `new_build` and `sheet` are served, every Build-taking tool opens its code only through `open_link` (`stale_link` otherwise) and returns `{build, link, todo}`, and `mcp::Address` writes and reads every choice's address, with round-trip tests for every kind. 28 server tests pass (13 new).
