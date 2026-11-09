# Refusals name the pick order within a level

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Max DC run (2026-11-09, see [119](119-caster-line.md)) one `choose` batch put Spell Focus (Enchantment) in `slot.feat.1.general.0` and Greater Spell Focus (Enchantment) in `slot.feat.1.class.0`, the sorcerer's level-1 Magic feat. The second was refused:

> `unmet`: "Needs Spell Focus in the same school"

Spell Focus *was* held at level 1. The refusal is right: ticket 32 set the pick order within a level (class selections, class feat slots, race slots, then the general slot), so the class slot is checked before the general slot's pick exists. But nothing in the reason says so, and the agent guessed the cause and worked around it by moving Greater Spell Focus to level 3. The fix it missed was to swap the two slots.

## Scope

1. **The reason:** when an `unmet` prerequisite is met by a pick in a later slot of the same level, the reason says so and names that slot: "Needs Spell Focus in the same school: Spell Focus (Enchantment) is in slot.feat.1.general.0, which comes after this slot in the level's pick order; swap them." In the engine's reason text, so the web app's picker says it too.
2. **The instructions:** one bullet in "Before you plan": within a level, class selections and class feat slots come before race slots and the general slot, so a feat that needs another taken at the same level goes in the later slot.

## Open questions (with recommendations)

1. **Only feats?** *Recommend:* any prerequisite leaf a later same-level pick would meet (a feat, a class feature from a later selection), found by checking the level as if that slot came first; no new rule.

## Done when

A rules test (the sorcerer's level-1 case) and a server test give the new reason; the instructions bullet is in, for the user's review as in ticket 117; `scripts/proof.sh` shows only reason text changed; `scripts/check.sh` passes.

## Choices settled

Each is the recommended answer (or, where the ticket gave none, the smallest choice consistent with the map's Notes), taken while the user slept, 2026-11-09.

1. **Only feats?** No: any pick in a later slot of the same character level that turns the refusal into an open option (a feat, a Feature selection option, a mythic ability or feat), found by checking the pick against the state just before and just after each later pick of the level, in walk order. No new rule: the pick stays refused.
2. **Mythic ranks count as later in the level** (ticket 131, item 6): a Mythic rank's slots are filled after its character level's own slots, so a level-11 general feat that needs Perception II taken at MR 4 (also level 11) names MR 4's slot.
3. **Where the note lives:** in the `Unmet` the slot's check gave, wrapped as `Unmet::MetLater { unmet, later }` (the outermost layer only), so the issue stays `unmet` for the MCP server, the web app's picker and the proof harness's generator, and the reason text is the engine's (`Engine::issue_text`).
4. **How it names the slot:** the way the sheet labels it, not by an MCP address: "level 1's Feat slot", "Mythic rank 4's Improved Mythic Trick slot". The instructions bullet gives the MCP addresses in pick order.
5. **"swap them" only when a swap works:** the reason says so only when this slot takes the later pick now and the later slot takes this pick after it (prerequisites included). A Mythic rank's Feature selection doesn't take a feat, so the Perception II reason doesn't say it.
6. **Only a later pick, not a grant between picks:** when the option opens before the next later pick (a Mythic rank's row features, say), the reason stays plain.

## Progress log

- 2026-11-09: Tests first. `libs/rules/tests/engine/pick_order.rs` (test world, 3 tests): a human fighter's level-1 Combat feat slot refusing Cleave while Power Attack is in the general slot (the reason names it and says to swap; swapped, Cleave is open); Great Cleave with Cleave later still gives the plain reason (BAB +4 is missing too); Crane Style in the level-11 general slot with Dodge in the Trickster's MR 4 bonus feat slot (named, no swap). `libs/seeds/tests/game_data/pick_order.rs` (real data, 2 tests): the sorcerer's level-1 case ("Needs Spell Focus in the same school: Spell Focus (Enchantment) is in level 1's Feat slot, which comes after this slot in the level's pick order; swap them"; swapped, the Build is possible) and the Perception II case ("Needs Perception II: Perception II is in Mythic rank 4's Improved Mythic Trick slot, which comes after this slot in level 11's pick order"). `apps/mcp/tests/server/choose.rs` gains one (`rejected` gives the new reason; the swapped batch applies both steps), `listing.rs` one (the bullet, in pick order).
- Built it: a walker probe `Probe::Later(slot)` that walks to the end of the slot's character level and keeps each later pick with the state either side (`walk.rs`, `walk/hold.rs`); `Engine::options` asks it lazily, once per slot, for the Build without the slot's own pick, and only when another pick sits at that character level (`pickers.rs`); `Unmet::MetLater` and `LaterPick` (`choice.rs`, `LaterPick` boxed for clippy's `result_large_err`); the text in `labels.rs`. One existing test changed its expected status: `options_see_earlier_picks_of_the_same_level_but_not_later_ones` (Power Attack in the race slot, Cleave in the class slot) now gets the wrapped reason.
- The instructions gain one "Before you plan" bullet (`apps/mcp/src/server.rs`); `GLOSSARY.md`'s Slot entry gains the Mythic ranks at the end of the level's order and the reason.
- The worktree's database was stale (no ticket 130 column), so it was rebuilt with the repo's first three steps; no migration or query changed.
- `scripts/proof.sh rules` against `main` (dd436d3), 2,162 Builds: 615 differ, every one only in `slots`, and in each of its 1,183 slots only in `options` and `reasons`: the option statuses' Debug text (which now holds the `MetLater` layer) and the lock-reason texts. No sheet, `possible`, `removable`, `allows`, open count, class, path or switch picker changed. The slots: 510 path selections (a Mythic rank's later selection at the same level), 414 race slots, 144 class feat slots, 114 general slots (a later Mythic rank) and one class selection. Side B recorded in 1,718 s against side A's 2,245 s on a loaded machine, so the extra walk per slot costs nothing visible.
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm clippy, tests, rules wasm, seed check). `cargo fmt --all --check` passes. Tests: rules engine 502 (3 new), real data 266 (2 new), MCP server 93 (2 new).

## Answer

Done 2026-11-09. When a prerequisite fails in a slot but a pick in a later slot of the same character level would meet it, the picker's reason says so and names that slot, and says to swap the two picks when each slot takes the other's. It's the engine's reason text, so the web app's picker, `options` and `choose`'s `rejected` all say it, and the issue stays `unmet`. A Mythic rank's picks count as later than its level's slots, which covers ticket 131's Perception II case:

> Needs Spell Focus in the same school: Spell Focus (Enchantment) is in level 1's Feat slot, which comes after this slot in the level's pick order; swap them

> Needs Perception II: Perception II is in Mythic rank 4's Improved Mythic Trick slot, which comes after this slot in level 11's pick order

The new "Before you plan" bullet, for the user's review:

> - Within a level, picks are checked in order: class selections and class feat slots (`slot.selection.N.I`, `slot.feat.N.class.I`), race slots, the general slot (`slot.feat.N.general.0`), the slots options and feats give, then each Mythic rank that comes at that level (`slot.mythic.R`, `slot.path-feat.R.I`, `slot.path-selection.R.I`). A pick that needs another taken at the same level goes in the later slot: Spell Focus in the sorcerer's level-1 class slot and Greater Spell Focus in the general slot, not the reverse. A refusal that says the prerequisite's pick "comes after this slot" names that pick's slot: swap the two picks, or take this one at a later level.
