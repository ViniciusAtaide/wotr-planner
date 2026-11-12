# Gear in the MCP, the skill and the plan; the Enforcer gets its belt

Type: task
Status: open
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
