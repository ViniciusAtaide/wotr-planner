# Gear in the MCP, the skill and the plan; the Enforcer gets its belt

Type: task
Status: done
Blocked by: 160, 165, 166
Model: Opus to build; Sonnet for the "fresh agent" test in Done when (a weaker model succeeding from the instructions alone is the stronger proof); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The fourth gear ticket from the equipment plan ([163](163-equipment.md), user, 2026-11-11): the MCP, the wotr-build skill, the map, and the gold build that started it ([160](160-gold-builds.md)).

Decided in 163:

- **Gear is something the user states, never something the planner assumes.** `compare` and `improve` never add gear by themselves.
- **Both numbers:** `sheet` gives each number with gear and without when they differ. `compare` and `improve` rank by the naked number unless the request asks for gear to count.
- **Addresses:** `gear.<slot>`, with the existing `options`, `choose` and `withdraw`.
- **`search`** finds items by name and by bonus.
- **Other effects** come in a field of their own, left out under `compact`, and never affect a number or a ranking.
- **The Enforcer** wears the weakest item that makes its level 19 legal: a belt with a Dex +2 enhancement bonus. The write-up names no items.

## Build

AFK. `apps/mcp`, `.claude/skills/wotr-build/`, the map, and one gold test. No engine or schema change.

- **`options` and `choose`:** gear addresses arrive with the grammar (166). `options` on a gear slot lists its items with summary and other effects. `todo` never lists an empty gear slot: gear is optional.
- **An item a pick needs can't be removed or replaced** (user, 2026-11-12): `options` shows such an item's alternatives as closed with the reason, and `choose` and `withdraw` refuse with it. The MCP tests' "withdraw the belt and see the pick refused" becomes: withdrawing the belt is refused, naming the pick; after the pick is withdrawn, the belt comes off.
- **`options` statuses:** `Open` gains a second variant for an option open only because of gear, naming the item (`apps/mcp/src/shapes.rs`; today it is the single `"ok"`).
- **`sheet`:** the worn items with their summaries; each number with gear, and a `naked` block with the numbers gear changes; the list of gear-dependent picks, each with its item. The scores' doc ("feature bonuses included, gear never") changes to match.
- **`compare`:** shows both numbers; its differences and `by` rank on the naked ones by default, and on the geared ones with a new argument. Say which was used in the result.
- **`improve`:** ranks naked by default, the same argument switches it. It never proposes an item.
- **`search`:** `kinds: ["gear"]`, matching the name, the slot and the summary's words ("Dex", "enhancement").
- **The server's instructions and tool descriptions** (`apps/mcp/src/server.rs`): "The numbers are naked: no gear…" becomes: a Build has no gear unless the user asks for it; how to add it; both numbers; which one ranks; what gear still leaves out.
- **The wotr-build skill** (`SKILL.md`, `martial.md`): the same change. "Add gear only if the user asks" stays, now with the addresses to do it. A build that relies on gear says so in its report, naming the picks.
- **The Enforcer:** in the gold test, wear the Dex +2 belt, with a comment that the write-up names none and this is the smallest assumption that makes level 19 legal. Remove the Greater Two-Weapon Fighting entry from 160's known gaps; the test then proves it. Regenerate the Enforcer's snapshot and show the diff.
- **The map:**
  - rewrite Out of scope's "Gear and companions" entry: companions stay out; gear is in, as 163 decided, with what the second stage still holds back;
  - a Decisions line for the gear tickets;
  - ticket [16](16-bonus-feats-ignore-prerequisites.md) D2 and ticket [21](21-schema-questions-real-data.md) R3 said "not gear": leave resolved tickets as they are, and name them in the Decisions line as reversed.
- **MCP tests** in `apps/mcp/tests/server/`: choose a belt, see a locked feat open with the gear status, `sheet` shows both numbers and the dependent pick, withdraw the belt and see the pick refused; `compare` ranks naked by default and geared when asked.

## Done when

- Each MCP test above is listed by `cargo test -p mcp -- --list` and passes.
- The gold test passes with the Greater Two-Weapon Fighting gap removed.
- A fresh agent, given only the MCP's instructions, builds a character that needs a belt for a feat and reports the dependence, without reading the source.
- The map's Out of scope entry, the server's instructions and the skill say the same thing about gear.
- `scripts/check.sh` passes.

## Answer

**Built**, 2026-11-12 on branch `ticket-168`, from `main` at `5b5b5dc`. Not merged. No engine or schema change: everything is the engine API of ticket 166, called from `apps/mcp`.

### What was built

- **`apps/mcp/src/gear.rs`**, the server's gear: a place's or a usable item's offers from `Engine::gear_options` and `Engine::usable_options`, the reason a needed item can't come off, an item's addresses, the gear lines and gear-dependent lines of `sheet`, and the sheet `compare` and `improve` count.
- **`options` and `choose`** on `gear.<place>` (`gear.belt`, …, `gear.ring.0`, `gear.ring.1`, `gear.body`, `gear.weapon`): "None" (value `none`) and every item the place takes, each with its summary, its `other_effects` (left out under `compact`) and the engine's verdict. On `gear.usable.<item key>`: `none` and the item. An address that names a worn item, or no item, is `no_such_choice` saying where it goes. A text starting with `gear` that isn't an address gets the list of gear addresses. `todo` lists no gear: gear is optional. `choose`'s result says when a step moved a ring from the other ring place (`moved`: `{step, choice, item, name, from}`); `none` at an empty place is a no-op, applied as choosing any value the Build already holds is (counted in `applied`, the code unchanged).
- **A step that leaves a choice allowed only because of gear is applied and says so:** `choose`'s result has `leans_on_gear` (`{step, choice, leaning}`, `leaning` as `sheet`'s `gear_dependent`), from `Engine::newly_gear_dependent` of each edit the step applies (a point-buy move is several edits), keeping those still leaning when the step is done; `withdraw`'s result has `leans_on_gear` too (`withdraw` now returns `WithdrawResult`, the envelope plus that field).
- **Statuses:** `"ok"`; `{"with_gear": "With Belt of Dexterity +2"}`, an option open only because of the Build's gear (a slot's `SlotOption::with_gear`, worded by `Engine::gear_note`); `{"leans_on_gear": "Greater Two-Weapon Fighting would then be allowed only because of the gear"}`, an open option of the other pickers whose `leans_on_gear` isn't empty (point buy, race, faith, alignment, background, racial ability, ability increase, class); or `{issue, reason}`.
- **An item a pick needs can't be withdrawn, or replaced by an item that doesn't keep the pick** (the user, 2026-11-12): `options` shows "None" and every item that would refuse the pick as `breaks_later`, with the engine's reason ("Greater Two-Weapon Fighting needs Belt of Dexterity +2"), and an item that keeps it (a stronger belt) open; `choose` refuses the locked ones with it; `withdraw` answers `not_removable: Greater Two-Weapon Fighting needs Belt of Dexterity +2, so `gear.belt` can't be withdrawn now, nor replaced by an item that doesn't keep that choice: withdraw the choice first, or `choose` an item that keeps it …`.
- **`sheet`:** `gear` (each item held: `choice`, `label`, `value`, `name`, `summary`, `other_effects`, and `removable` with `removable: true`); `gear_dependent` (each choice only gear allows: `choice`, `label`, `name`, `needs`, `items`); `with_gear` on that choice's pick line; `naked`, the numbers without the gear only where they differ (`levels`: each level's `scores`, `attack`, `caster`; the last level's `totals`). Every other number counts the gear. Each level's `scores` doc now says so.
- **`compare`:** `with_gear` (default false). The summaries' numbers (scores, attack line, caster line, `by`'s `number`), the level differences' numbers and `by`'s order are naked unless `with_gear: true`; the result's `numbers` is `"naked"` or `"with_gear"`. Each summary also has `gear` (its items), `gear_dependent`, and the other numbers where its gear changes them: `with_gear` beside naked ones, `naked` beside geared ones. The result's `gear` lists the gear places whose item differs. The choices, feats and features compared are each Build's own.
- **`improve`:** `with_gear` (default false) and `numbers` in the result. The naked number is the Build's with no gear at all, so an option open only because of gear changes nothing. A gear address is `invalid_arguments`: it never proposes an item.
- **`search` and `describe`:** kind `gear`. A hit has `category` (its slot), `summary` (its bonuses), `prerequisite: null`, a `guide_url` when the item has one (the 22 generic items have none), `other_effects` and `places` (both left out under `compact`). Gear is found only when `kinds` names `gear`: an unfiltered search finds none, as before this ticket, and `describe`'s closest names for a missing gear key search gear only. An item matches by name, and by its slot and its abilities' full names ("belt", "Dexterity") as more words of its summary, which match alone only when they hold every query word, so a staff isn't a match for "weapon focus".
- **`validate`:** `"gear.<place>": "<item key>"` (or `none`) and `"gear.usable.<key>": "<key>"`. The walk refuses an unknown key (`unknown_key`, with the closest items) and an item at a place that doesn't take it (`not_allowed`, naming its places). A usable address given another key is `not_an_option`. Replaces ticket 166's `no_such_choice` arms and its test.
- **The instructions** (3,393 characters, under the 4,000 the test holds), every tool description and the wotr-build skill (`SKILL.md`: a Gear section, the honesty boundary, ranking, the report; `martial.md`: the worksheet's line and its Gear section) say: a Build has no gear unless the user asks for it; how to add it; that it counts for prerequisites and every number, never skill points; `with_gear`, `gear_dependent` and the lock; both numbers and which one ranks; and what gear still leaves out. A Build that relies on gear names each pick and its item in its report.
- **The map:** Out of scope's "Gear and companions" is rewritten (gear in as 163 decided, what 169 still holds back; companions out until 174), a Decisions line names 16's D2 and 21's R3 as reversed, and the gear note's "Nothing is built yet" sentence now says 168 rewrote the entry.
- **The gold test:** the Enforcer wears the Belt of Incredible Dexterity +2 (a comment says the write-up names no item and this is the user's choice of the smallest assumption that makes level 19 legal), its level-19 Feat slot takes Greater Two-Weapon Fighting, and the `Gear(168)` entry is gone, with the `Why::Gear` kind it was the only one of. Nine known gaps remain.

### Tests

`apps/mcp/tests/server/gear.rs`, on the test world with a feat needing Dex 19 (and on the real data for `search`):
`choosing_a_belt_opens_a_locked_feat_with_the_gear_status`, `sheet_shows_both_numbers_and_the_dependent_pick`, `withdrawing_the_belt_is_refused_naming_the_pick_until_the_pick_is_withdrawn`, `a_usable_item_is_used_and_stopped_at_its_own_address`, `a_step_that_would_lean_on_the_gear_says_so`, `compare_ranks_naked_by_default_and_with_gear_when_asked`, `improve_ranks_naked_by_default_and_never_proposes_gear`, `search_finds_gear_by_name_place_and_bonus`, `the_instructions_and_the_guide_say_gear_comes_only_when_asked`, and from the review `an_unfiltered_search_finds_no_gear`, `choosing_a_worn_ring_at_the_other_ring_place_says_it_moved`, `none_at_an_empty_place_is_a_no_op_like_any_held_value`, `a_step_that_leaves_a_pick_leaning_on_the_gear_is_applied_and_says_so`. In `validate.rs`, `declared_gear_is_held` and `declared_gear_that_doesnt_fit_fails` replace `a_declared_gear_address_is_no_choice_yet`; `attack_line.rs`'s instructions test now reads `no_tool_takes_assume_and_gear_counts_only_when_held`.

### Done when

- The MCP tests are listed by `cargo test -p mcp -- --list` and pass (166 in `server`).
- The gold test passes with the Greater Two-Weapon Fighting gap removed: `gold_builds_gain_what_the_write_ups_list` with nine gaps.
- The snapshot check passes with every file untouched but `snapshots/gold-enforcer.txt` (`git status`). Before regenerating, its grouped message was:

  ```
  the engine's output differs from the snapshots in 1 of 44 Builds
  …
  The same, levels and ranks wildcarded:
    level.*.scores.dex: 16 → 18 in 1 Build, 11 lines
    level.*.scores.dex: 17 → 19 in 1 Build, 9 lines
    level.*.feat/greater-two-weapon-fighting: + Greater Two-Weapon Fighting in 1 Build, 1 line
    slot.feat.*.general.0: Feat: - → Feat: greater-two-weapon-fighting "Greater Two-Weapon Fighting" in 1 Build, 1 line
    totals.initiative: 17: Dexterity +3, … → 18: Dexterity +4, … in 1 Build, 1 line
    totals.reflex: 16: Base save +13, Dexterity +3 → 17: Base save +13, Dexterity +4 in 1 Build, 1 line
    totals.skill.mobility: 13: … Dexterity +3 … → 14: … Dexterity +4 … in 1 Build, 1 line
    totals.skill.stealth: 13: … Dexterity +3 … → 14: … Dexterity +4 … in 1 Build, 1 line
    totals.skill.trickery: 6: … Dexterity +3 … → 7: … Dexterity +4 … in 1 Build, 1 line
  ```

  The belt's Dex at every level, the feat at 19, and the Dex totals. No skill points moved, and the snapshot projection has no gear line of its own.
- The map's Out of scope entry, the instructions and the skill say the same about gear (above; `the_instructions_and_the_guide_say_gear_comes_only_when_asked` holds the shared phrases).
- **Left to the main session:** a fresh agent (Sonnet), given only the MCP's instructions, builds a character that needs a belt for a feat and reports the dependence.
- `scripts/check.sh`: ALL-OK.

### Review fixes (2026-11-12)

An independent review drove a server built from this branch on the real data and found no wrong answer and no panic. Four low-severity changes:

1. **The lock's wording:** "can't be withdrawn or replaced" was too strong, since an item that keeps the pick (a Dex +6 belt for a Dex +2 one) is open. The instructions, the `options` and `withdraw` descriptions, `withdraw`'s error, the shapes' docs, `SKILL.md` and the map's Out of scope entry and Decisions line now say: it can't be withdrawn, or replaced by an item that doesn't keep the pick.
2. **Unfiltered `search` returned gear** ("belt": 28 hits; "dex": three items in the top ten). Gear now comes only when `kinds` names `gear`, said in `search`'s description, the instructions and the skill.
3. **`choose` moved a ring silently:** its result now has `moved`. `none` at an empty place was already handled as every no-op is (applied, the code unchanged, as choosing the race the Build holds); a test pins that it matches.
4. **`choose` and `withdraw` never warned of new gear dependence:** both now return `leans_on_gear`, the step applied all the same; the tool descriptions and one clause of the instructions' gear paragraph say so.

### Judgement calls

- **Two open statuses, not one:** `with_gear` for an option open only because of gear (what the ticket asks), and `leans_on_gear` for an option that is open but would leave another choice leaning on the gear, which 166's pickers report and an agent can't see otherwise.
- **`compare`'s summary numbers follow `numbers`** (naked by default) and the other set sits beside them only where the gear changes it, so a field never changes meaning with the flag. The BAB, base saves and caster level (class levels) are the Build's own, which gear doesn't change. The attack line's weapon is the Build's own Weapon Focus weapon either way.
- **The naked numbers are the engine's** (`Engine::naked_sheet`): the Build with no gear, so a pick only gear allows counts for nothing in them (`improve` says so).
- **A usable item's address** offers `none` and the item, mirroring a place, so `choose` and `withdraw` work on it as on a belt.
- **`search`'s slot and ability words** match alone only when they hold every query word, so adding gear doesn't add noise to an existing feat search (`search::a_category_filters_the_matches_and_needs_no_query` held).
- **`prerequisite: null`** for a gear hit: an item needs nothing.
- **`leans_on_gear` in `choose`** gathers `newly_gear_dependent` over every edit a step applies, then keeps the choices still gear-dependent in the Build the step made, worded by `gear_dependents`; a Build without gear pays nothing (the engine returns at once).
- **The README's gold section** said gear was one of four gap kinds and used Greater Two-Weapon Fighting's refusal as its example; it now has three kinds and Mythic Shield Focus's refusal.

### Edits outside `apps/mcp`, the skill, the map and the gold test

- `libs/game-data/README.md`, "The gold standard": the Enforcer's belt, three gap kinds, and the example refusal (above).
