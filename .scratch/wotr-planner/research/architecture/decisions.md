# The six deepenings: decisions

The main session's answers to the open questions in `overall.md` and the six designs (2026-11-19). The user was asked about items 1–3 and declined to answer; these are the architect's recommendations, taken by the main session, and the user may overrule any of them before its ticket is built.

1. **Wording a player sees (P, R).** Accepted:
   - an or inside an or loses its brackets;
   - `AtLeast(1, [x])` reads as a need ("Needs Kinetic Healer");
   - "N more of" with none held reads "N of" (330 item 3);
   - the MCP entry line in a Mythic rank's slot gains ticket 171's hint.

   Kept: two refusal orders, refusals first in a reason and last in prerequisite text (tickets 65, 329).
2. **R against 199–202 (another session's tickets).** R comes after 199 and before 200: tickets 336a and 336b are blocked by 199. The other session is told so through its tickets' `Blocked by` only when it commits them; this queue does not edit 199–202.
3. **A stale `held` line in `user-facts.txt`** fails only under `--write`, with exit 2, as today (design-L option a).
4. **Where the normal form lives (P).** `Prerequisite::canonicalize`, which every load path already calls, including the writer's before/after compile (design-P settled both of the architect's unverified points). game-check's `canon` stays whole: it normalises after the GUID mapping. `needs_unheld` stays on the raw tree (per-option hot path, already correct).
5. **G's "wording item 6"** (count mythic-ability, switch and component grants as holders in reasons) is out: T4 keeps today's row-only `grantor_in`/`in_reach`. A later ticket may widen it.
6. **324's scope.** N closes the three Elemental Witch rays and the kineticist's air, earth and fire elements. Greater Enduring Spells needing Enduring Spells is not a narrowing; it is ticket 208's (design-N risk 5).
7. **Owners.** N records evidence, never a ticket; 337a removes the guessed owners but for the "no form" gaps; 338a deletes `owner()`. A new carrier line is written `-> ?` and fails the check until the main session sets its ticket.
8. **X's order.** 339 is blocked by 332 (normal form), not 333: X uses the existing `Fact::leaf`. It shares `libs/seeds/tests/game_data/support.rs` with 334, so it runs after it.

## Tickets

| Ticket | Design | Model | Blocked by | Lane |
|---|---|---|---|---|
| 331 form proof records reasons | design-X §3 (T0) | Sonnet | — | tools |
| 332 one normal form | design-P T1 | Opus | 331 | rules |
| 333 the polarity-aware view | design-P T2 | Sonnet, Opus review | 332 | rules |
| 334 grant index, static half | design-G T3 | Opus | 331; beside 332 only if 332 leaves `catalog.rs`, `check.rs` and seeds `support.rs` alone | tools |
| 335 grant index, Engine half (330.2) | design-G T4 | Opus | 333, 334 | rules |
| 336a refusal, picker side | design-R T5a | Opus | 199, 335 | rules |
| 336b refusal, `validate` (330.1) | design-R T5b | Opus | 336a | rules |
| 337a one plan, relative carrier guard | design-N T6a | Opus | 333, 334 | tools |
| 337b the four new narrowing kinds (324) | design-N T6b | Opus | 337a | tools |
| 338a one line reader and reconcile | design-L T7a | Sonnet | 337b | tools |
| 338b carrier hits fold into `known.txt` | design-L T7b | Opus | 338a | tools |
| 339 expected refusals as rows | design-X T8 | Opus | 332, 334 | tools |
