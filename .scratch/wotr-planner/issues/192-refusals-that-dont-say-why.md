# Refusals and labels from the MCP QA session

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build; Opus to review the wording.
Map: [WotR Build Planner](../map.md)

## Problems

Found by a QA session through the MCP server (2026-11-13); the first four were rechecked on `main` at `ce9cc3b`. Each reason is correct but leaves out what the reader needs, or reads oddly.

1. **A lowered score doesn't name what it breaks.** A sorcerer with Dodge held through a Belt of Physical Perfection +2, Dexterity lowered from 11 to 10: "At Dexterity 11 → 10: Would break a later choice, which must be withdrawn first …" (`BREAKS_LATER`, `apps/mcp/src/options.rs`). The gear refusal for the same Build names it ("Dodge needs Belt of Physical Perfection +2"). Name the first choices it would break, as ticket 171's locked rank moves do.
2. **"Already taken"** for an option taken once per value (`Issue::AlreadyHeld`, `libs/rules/src/engine/labels.rs`): Finesse Training (Rapier) chosen again at rogue 19 says only "Already taken". Say where: "Finesse Training (Rapier) is taken at level 3".
3. **A rank level below the rank under it** (`choose` `mythic.4.level` 12 with MR 3 at 13): "`12` isn't an option for `mythic.4.level`, and no offered value is close to it. `options` lists what it takes, with `values: true` for a parameterised feat's values." (`apps/mcp/src/choose.rs`). 13 is offered and one away, the cause (MR 3 is taken at 13) is missing, and the parameterised-feat hint doesn't apply to a level.
4. **Withdrawing a rank already at its earliest level:** "The Build holds nothing at `mythic.4.level`" (`apps/mcp/src/withdraw.rs`). MR 4 is held, at level 11; say it is already at the earliest level it comes at.
5. **"Combat Style Feat feat"**, the ranger's slot label at 2, 6, 10, 14 and 18 (`format!("{} feat", …)` in `labels.rs`, on the selection "Combat Style Feat").
6. **"(no Bloodline yet, no Bloodline yet)"** at the head of the Dragon Disciple's prerequisite in `search` ("one of 11 alternatives ((no Bloodline yet, no Bloodline yet), Draconic (Black), …)"), from `labels.rs`'s `no {} yet`. Say what the alternative is (a sorcerer or a magus with no bloodline chosen yet, if that's it) once.
7. **"Needs Perception II"** for Stat Focus at level 13 when Perception II is held, but at MR 4 moved to level 14. The same-level case already says where the pick is ("… in Mythic rank 4's Improved Mythic Trick slot, which comes after this slot in level 11's pick order"); the later-rank case should say the rank and its level.
8. **"Stat Focus (AC) needs MR 3 earlier than level 13"** on `options` for `mythic.3.level`: Stat Focus needs Perception II in MR 4, and moving MR 3 breaks it only by taking MR 4 along. Name MR 4 and that it moves with MR 3.

Items 7 and 8 come from a subagent's report and were not rechecked on `ce9cc3b`: reproduce them first (an Empyreal Sorcerer 20, Trickster, Perception I at MR 3 and Perception II at MR 4).

## Done when

Each reason or label says what is held, where, and why, without the stray hint; tests pin the new texts; the snapshot check's reason lines move only for these; `scripts/check.sh` passes.
