# Favorite Metamagic picks its metamagic, and the other in-game choices with no pick

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Max DC run (2026-11-09, see [119](119-caster-line.md)) Favorite Metamagic ("The chosen metamagic costs one spell level less") was taken at MR 3 and the Build asked nothing more. `libs/game-data/mythic/abilities.ron` gives it no selection and no parameter, so the link can't say which metamagic, and the agent had to tell the user "choose Persistent in the game". Second Bloodline, beside it in the same file, does open its pick (`selections: ["sorcerer/bloodline"]`), so the gap is per item.

## Scope

1. **Favorite Metamagic** takes its metamagic, from the guide's list: a parameter of the metamagic feats (or a selection of them), each held once. Whether it needs the metamagic feat taken is the guide's call; check the saved page.
2. **An audit** of every feat, mythic ability, mythic feat and option whose summary says "chosen", "choose", "pick" or "of your choice" and that has neither a parameter nor a selection, listed in the ticket with what the guide says each chooses. Each one the guide backs gets its pick; the rest are listed as checked.
3. A link with Favorite Metamagic and no metamagic (every older one) still opens, with the pick open in `todo`.

## Open questions (with recommendations)

1. **Parameter or selection?** *Recommend:* a parameter over the metamagic feats, as mythic feats already take one (ticket 32), so `options {values: true}` lists them and the link format already holds it. A selection would need option entries that duplicate the feats.
2. **Repeatable?** The guide may let it be taken once per metamagic. *Recommend:* follow the saved page; if it says nothing, once (the run found it "already held" at MR 7).

## Done when

Favorite Metamagic opens a metamagic choice in `todo` and `options`; the audit's list is in this ticket; each fixed item has a real-data test in `libs/seeds/tests/game_data/` and a note in `libs/game-data/README.md`; an older link still opens; `scripts/proof.sh` shows no other change; `scripts/check.sh --schema` passes.

## Choices settled

Each the recommended answer (or, where none fit, the smallest choice consistent with the map's Notes), taken while the user slept, 2026-11-09.

1. **Parameter, not selection** (question 1). Mythic abilities couldn't take a parameter, so the model grew one: `MythicAbility::parameter` and `Pick::MythicAbility { ability, parameter }` (as ticket 32 gave mythic feats). `options {values: true}` lists one option per value; the sheet names it "Favorite Metamagic (Persistent Spell)".
2. **The metamagic values are every metamagic feat.** The saved guide page lists none ("the selected metamagic type"), so a `Metamagic` value is any feat whose key starts `metamagic-` (`Feat::is_metamagic`, a rule in Rust: no derived column). That's the guide's twelve "Metamagic (…)" feats, Completely Normal Spell included. Having the feat taken isn't required: the guide doesn't say so.
3. **Once** (question 2). `guide-cache/mythic-paths.html` says nothing of taking it again, so an ability is held once whatever its value (a second Favorite Metamagic is "already held"). No `repeat` on mythic abilities.
4. **Older links keep opening** (scope 3). The link layout changed only by new tags (pick `4`: a mythic ability with a value; parameters `6` metamagic, `7` stat, `8` element), as ticket 91 added the first ascension: a valueless mythic ability is still tag `2`, so every link made before this reads as it did and the frozen `V1_LINK` is unchanged. Tickets 32, 46 and 55 replaced version 1 because no link had shipped yet; links have shipped now (ticket 100), so no version 2 and no regenerated vector. A link newer than this doesn't open in an older planner.
5. **A value may stay open, only for the new kinds.** A Build with Favorite Metamagic and no metamagic is possible (the ability is held), and the MCP server's `todo` lists the slot ("MR 1: Favorite Metamagic: choose a metamagic"); the sheet shows "Favorite Metamagic (?)". `choose` at that slot with `{key, parameter}` fills it. `ParameterKind::may_stay_open` is true for `Metamagic`, `Stat` and `Element` only, so Weapon Focus without a weapon is still refused. A new pick by key alone is still `needs_parameter` in `choose`.
6. **Stat Focus** (ticket 131 item 6): `parameter: Stat`, the guide's twelve (AC, attack, attacks of opportunity, CMB, CMD, damage, HP, initiative, Fortitude, Reflex, Will, speed; the summary now lists the saves as the guide does). It repeats once per stat: the user's decision 27-1 makes every parameterised feat `PerValue`, which wins over "once when the guide is silent".
7. **The audit's fixes**: of the items it found, the guide backs a value list for Favorite Metamagic, Stat Focus and Ascendant Element ("acid, cold, electricity, fire, sonic or negative energy": `parameter: Element`, held once). Those three are fixed; the rest are listed below as checked.
8. **Schema**: a new migration, `20261109013047_125-pick-values.sql`. It recreates `feat` with `parameter_kind` widened by `metamagic`, `stat` and `element` (the database is built from scratch, so recreating an empty table is safe; every foreign key to `feat` still resolves), and adds `mythic_ability.parameter`. `mythic_feat.parameter` and `selection_option.parameter` keep the first six kinds until one needs another. Only picks name the new values: the data check refuses one in a prerequisite, an option's `values` or `feats_for` (`DataIssue::PickOnlyValue`), so no value columns were added.

## The audit

Every feat, mythic ability, mythic feat and selection option whose summary says "chosen", "choose", "choice" or "pick" and that had no pick (no parameter, selection, chosen feat, pool, rows with slots or selections, or extra pick). The real-data test `pick_values::every_item_whose_summary_chooses_has_its_pick_or_was_checked` pins the list. Feature and selection summaries (a selection describes its own pick; a feature isn't picked) were left out.

| Item | What the guide says it chooses | Outcome |
| --- | --- | --- |
| Favorite Metamagic (mythic ability) | "the selected metamagic type" | **Fixed**: a metamagic feat, held once. |
| Ascendant Element (mythic ability) | "The chosen element (acid, cold, electricity, fire, sonic or negative energy)" | **Fixed**: an element, held once. |
| Stat Focus (feat) | one of "AC, Attack, Attacks of Opportunity, CMB, CMD, Damage, HP, Initiative, Fortitude, Reflex, Will or Speed" | **Fixed**: a stat, once per stat. |
| Advanced Weapon Training (feat) | one of seven Weapon Training options (Armed Bravery, Combat Maneuver Defense, Effortless Dual Wielding, Fighter's Finesse, Fighter's Reflexes, Fighter's Tactics, Trained Initiative) | Checked, backed, **not built**: the options aren't in the data (the fighter's own Weapon Training picks no groups or options either) and a feat can't grant a selection slot yet (`extra_pick_of` gives one only of a selection held before). It needs the seven options in `fighter.ron`, which ticket 131 was editing at the same time. A follow-up. |
| Lore (Religion) II and III (Trickster tricks) | "You can select two domains", then "two additional domains" | Checked, backed, **not built**: a domain's abilities need an option level, and no class levels or rule give one through a Trickster trick. A follow-up, needing that decision. |
| Rogue Secret (Loremaster secret) | "any Rogue talent without having to meet the prerequisites", Advanced Talents included | Checked, backed, **not built**: a selection slot can't waive its options' prerequisites. A follow-up. (Wizard Feat beside it is the same shape, but its summary doesn't say "choose".) |
| Energized Wild Shape (feat) | "a selected energy type" | Checked: the guide names no list, so there's no value list to back. |
| Spell Specialization (feat) | "Select one spell" (changeable as caster level rises) | Checked: a spell isn't a value yet (no spell catalogue: ticket 121). |
| Metamagic (Selective Spell) (feat) | creatures of your choice left out of the area | Checked: chosen in play, not in the Build. |
| Increased Damage Reduction (Stalwart Defender power) | "can be picked twice" | Checked: a repeat (`Times(2)`), not a choice. |

## Progress log

- 2026-11-09: rules. `ParameterKind` gains `Metamagic`, `Stat` and `Element` (`Stat`, `Element` enums in the guide's order; `ParameterKind::may_stay_open`), `Parameter` the matching values; `MythicAbility::parameter`; `Pick::MythicAbility { ability, parameter }` (`Pick::mythic_ability`, `mythic_ability_for`); the picker lists a parameterised ability once per value; `check_pick` takes a fitting value, or none for the new kinds; `Engine::value_left_open`; labels ("Persistent Spell" for Metamagic (Persistent Spell)); the data check (`fits`, `PickOnlyValue`). Codec: pick `4`, parameters `6`–`8`, documented; a new frozen-layout unit test for them; the proptest round trip covers them; `V1_LINK` unchanged and still passing. The tests were written alongside the change, not red first.
  - `libs/rules/tests/engine/pick_values.rs`, 14 tests on the test world with two metamagic feats, Favorite Metamagic and Ascendant Element added and its Stat Focus taking a stat.
- Seeds: the migration, the loader and the seed write `mythic_ability.parameter`; the value columns refuse the new kinds (unreachable after the check). Dev database rebuilt with CLAUDE.md's four steps; `.sqlx` regenerated (two queries changed).
- Data: `parameter: Metamagic` on Favorite Metamagic, `parameter: Element` on Ascendant Element, `parameter: Stat, repeat: PerValue` on Stat Focus (summary lists the guide's saves). `libs/seeds/tests/game_data/pick_values.rs`, 7 real-data tests (the twelve metamagic feats, each item's values and statuses, a valueless older link opening, held once, the audit). Three older tests updated: 16 parameterised feats (was 15), 35 repeating feats (was 34), and Extra Mythic Ability's slot offers one entry per ability (Favorite Metamagic and Ascendant Element now list one option per value). `libs/game-data/README.md` and `GLOSSARY.md` (**Parameter**, **Parameterised feat**, **Pick**).
- MCP: `todo` lists a filled slot whose pick's value is open; values print stats and elements with dashes (`attacks-of-opportunity`, `negative-energy`), a metamagic by its feat's key; the instructions mention parameterised mythic abilities and open values. `apps/mcp/tests/server/pick_values.rs`, 3 server tests on the real data. UI: a chip whose pick's value is open shows "(?)" and says what to choose in its title.
- Proof harness (`tools/proof`): it compiles against both pick shapes (a mythic ability pick made through serde; Witch Wandering Hex by its key); JSON Builds read a valueless mythic ability in either shape; every hashed Debug text writes a valueless mythic ability as before (`as_before_ticket_125`), so records compare; and `record --fold-values <key>,…` folds an item's per-value options into one.
