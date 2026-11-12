# Mythic picks can come after the levels: Mythic Shield Focus at the Enforcer's rank 10

Type: task
Status: resolved
Blocked by: 160, 166, 167, 168
Model: Fable to build (it changes the order the walk checks picks in); review on Opus.
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists the Enforcer's Mythic Shield Focus at Mythic rank 10 as a known gap. The planner places rank 10 at character level 19 (ticket [04](04-progression-model.md)'s rank table), and checks a level's picks before the ranks that come at it. The Enforcer takes Shield Focus at level 20, so "Shield Focus (Mythic)" at rank 10 is locked: "Needs Shield Focus".

The user, 2026-11-11: "you can select mythic feats after leveling all base levels so it's perfectly fine to level mythic shield focus mythic 10 at level 20."

That reverses the map decision for this case. Two ways to build it, for the user to choose before this starts:

- **A. Rank 10 moves to character level 20.** Only the rank table changes. It fixes this build, and no other rank.
- **B. Any rank may be taken later than its table level.** A rank's picks are checked at the level the user places it, anywhere from its table level to the last level. This matches "after leveling all base levels" for every rank, and it changes the Build, its link and the level screen.

*Recommend:* A if the game always gives rank 10 at the very end; B if the user wants ranks placed freely, as a larger ticket.

**Decided** (user, 2026-11-11): B. Any rank may be taken later than its table level.

## Build

AFK. `libs/rules`, the web app and the MCP. It runs after the gear tickets (166, 167, 168), since it edits the Build, the link, the level screen and the MCP as they do.

- Read ticket 04's Answer and the map's Notes on Mythic rank timing first: they record why each rank has its level.
- **The model:** each rank has a character level it is taken at. It defaults to the rank table's level, which stays the earliest allowed, and the user may move it later, up to the Build's last level. Ranks keep their order: a rank is never placed before the rank below it. A rank's picks are checked at the level it is placed, after that level's own picks, as today.
- **The Build and the link:** a rank's placement is stored only when it differs from the table, in a trailing optional part of the link (ticket 91's pattern, and 166's for gear). Every existing link opens unchanged and writes the link it always did.
- **The web app:** on the level screen, a rank shows at the level it is placed, with a control to move it later or back. The summary shows each rank's level.
- **The MCP:** `sheet`'s `character_level` per rank is the placed level; an address moves a rank (`mythic.R.level`), offered by `options`. The server's instructions say a pick that "comes after this slot" can also be fixed by taking the rank later.
- If the design needs a decision this ticket doesn't settle (a rank's features that the level's own picks depend on, a path switch at a moved rank), stop and put it to the user rather than choosing.
- Prove what changes with the snapshot check: the ranks' `character_level` lines and whatever opens because of them, and nothing else.
- With nothing moved, the snapshots don't change at all.
- Remove the Mythic Shield Focus entry from the gold test's known gaps: place the Enforcer's rank 10 at level 20 and fill its Mythic feat slot. The test then proves it.
- Old links must still open: a Build whose rank picks were legal stays legal.

## Done when

- The Enforcer's Mythic Shield Focus at rank 10 agrees in the gold test, with its known-gap entry deleted.
- The snapshot diff is reviewed and holds only the intended change.
- `scripts/check.sh` passes.

## Answer

**Built**, 2026-11-12 on branch `ticket-171`, from `main` at `27d6d81`. Not merged. Option B: any Mythic rank may be taken later than its table level. Ticket [04](04-progression-model.md)'s rank table changes from each rank's fixed level to its earliest level.

### The model

- **Stored:** `Build::rank_levels`, a map of rank to character level, holding a rank only when it is taken later than the rank table's level. Empty for every Build that moves no rank.
- **Derived:** `Engine::rank_level(build, rank)` is the level a rank is taken at: its stored level or else the table's, and never before the rank below it. `RankSheet::character_level` is that level; no sheet or state type gained a field.
- **Bounded:** `Engine::apply` keeps a stored level only when it is later than the table's and no later than the Build's last level, and only for a rank the ascensions get to. The walk fails a hand-made level that is none (`Choice::RankLevel`: `NotAllowed` at or before the table's level, `NotReached` past the last level or for a rank that never comes), so such a link opens nothing.
- **Judged:** the walk ascends each rank at the level it is taken at, after that level's own picks, as before. A move is an edit like any other (`Edit::RankLevel`): the cursor walks the edited Build from the earliest level the rank can come at, and a move that would refuse a choice is `BreaksLater`.
- **The picker:** `Engine::rank_level_options(build, rank)` lists the levels from the earliest the rank comes at (the table's, or the rank below's when that is later) to the last, each with its edit, whether it is held, the ranks above that move with it, and for a locked one the choices it would refuse and a reason naming them. Its withdrawal (`Choice::RankLevel`) takes the rank as early as it comes again.

### What "later" must not break

- **Order.** Moving a rank later takes the ranks above it along; it is never refused for their sake. A rank with a level of its own keeps it while the rank below is no later, and follows when it is ({MR 3: 12, MR 4: 13}, then MR 3 to 15: MR 4 comes at 15; MR 3 back to 12: MR 4 is at 13 again, and the link is the one it was). The picker says which ranks a level moves ("MR 4 moves to level 12"), later or back. Two ranks at one level come in rank order.
- **The upper bound, and removing levels.** Only the last level can be removed, so a level is never removed from under a moved rank; removing the level a rank was moved to is allowed, as removing the level of any rank is today. It takes the rank's picks (and the first ascension, path or switch chosen at it), and the rank's level goes too, so the rank comes back at its earliest level, reached and empty. That removal is now walked from the earliest level the returning ranks come at, and is refused (not removable) if a choice there would fail with what the ranks give; no data reaches that today.
- **Features held from the rank's level.** Nothing new was invented. A level's pick between the two levels that needs what the rank gives fails in the walk of the moved Build, so the move is locked and names it: in the test world, a Legend's +4 Dexterity that Dodge at level 19 needs locks MR 8 at levels 19 and 20 ("dodge needs MR 8 earlier than level 19"); a level's skill points and a level past the cap are named the same way. Moving a rank back is locked by the rank's own pick ("Shield Focus (Mythic) needs MR 10 later than level 19").
- **The first ascension.** MR 1 and 2 move like any rank; their rows are held from the level they are taken at, and MR 1 at the last level takes MR 2 and 3 (and the path choice) along. A Build without a first ascension still takes its MR 3 path's rows at MR 1 and 2.
- **Path switches.** MR 8 moves like any rank and the switch happens where it is taken. What a switch keeps and drops is by rank, never by level, so a Devil, Gold Dragon or Swarm switch at a moved MR 8 gives the same statuses, dropped slots and final state as at level 17 (a test compares them). The old path holds until the switch's level.
- **Legend.** Its switch can be taken at any level from 17 to 20: the levels past 20 need the cap it raises, so a later level is locked, naming level 21. MR 9 and 10 never come, so they have no level picker, and switching to Legend drops a level they were placed at. A rank below MR 8 moved past level 20 would take the switch along, so it is locked the same way.
- **The two cases to consult on.** Neither needed a rule: (a) a level's own picks that depend on a moved rank are covered by the refusal above; (b) a switch at a moved rank is settled by the existing rules, which are all by rank. No question is left open for the design.

### The link

- A last section after the gear: `3, varint count (at least 1), count x (u8 rank, u8 level)`, in rank order, each rank once, written only when a rank is moved. It names no key.
- The three trailing sections (first ascension `1`, gear `2`, rank levels `3`) are each there or not on their own, always in tag order. The decoder refuses a section before one with a lower tag, an empty section, a rank out of order or twice, a rank or level of 0, an unknown tag and bytes left over.
- `version_1_body_layout_is_frozen`, `V1_LINK` and `V1_GEAR_LINK` are untouched. `V1_RANK_LEVELS_LINK` is pinned beside them (the gear Build with MR 1 at level 3 and MR 10 at level 20); without its rank levels it writes `V1_GEAR_LINK`, and without its gear `V1_LINK`.
- Fuzz, as ticket 166's review did, over all eight combinations of the three sections: 1,424 truncations (each either refused or exactly the Build with whole sections cut off) and 2,048 trailing bytes (all refused), no panic. The round-trip proptest takes any rank levels.

### The text export

A rank taken later says so on its own line, "MR 10 (level 20, taken later than level 19)", with or without a pick, and so does a rank it takes along. A Build that moves no rank gives the text it always did (the v1 byte-for-byte test is unchanged).

### The MCP

- **Address:** `mythic.R.level`. `options` lists the levels (`{"value": 20, "name": "Level 20", "status": "ok"}`; "Level 12, which moves MR 4 to level 12"; a locked one `{issue: "breaks_later", reason: "toughness-mythic needs MR 4 later than level 11"}`); `choose` and `validate` take the number; `withdraw` takes the rank as early as it comes, or refuses with `not_removable` naming the pick.
- **`sheet`:** a rank's `character_level` is the level it is taken at, with `table_level` beside it only for a rank taken later. `compare` gives a rank's `character_level` per Build where they differ.
- **A rank's pick that needs a later level's pick** says which level of the rank opens it: "Needs Shield Focus. MR 10 taken at level 20 opens it: `choose` `mythic.10.level` 20 first" (`Engine::rank_level_that_opens`).
- **Wording:** the instructions (3,852 of 4,000 characters) and the build guide say a rank can be taken later; the `options` and `withdraw` descriptions name the address. A test pins the phrases in both texts.

### The web app

- **Levels:** each rank has its own badge on the row of the level it is taken at ("✦ MR 10", with "from level 19" and a dashed border when it is taken later), and beside it a small round control (⇅, the remove control's language, with a thumb-sized tap target) that opens the rank's level picker.
- **The picker** ("Character level", "Mythic rank 10"): a row per level, the held one ticked, "The earliest: the rank table's level" and "MR 4 moves to level 12" under a level's name, a locked level behind "Show N locked" with its reason, and Remove while the rank can come back.
- **Mythic track:** each rank's row gives its level, "from 19" for a rank taken later, and the same control. A rank's slot picker adds "MR 10 taken at level 20 opens it" to a locked pick's reason.
- **Summary:** "Mythic ranks by level: MR 1 at 6 · … · MR 10 at 20 (from 19)".
- Checked in a release build on the Fold's cover (412x960) and inner (720x864) screens with touch, and on desktop (1440x900), by a script that asserts each step (below).

### Behaviour checks

- **Snapshots:** with no rank moved, every file is untouched. With the Enforcer's MR 10 at level 20 the check's grouped message was:

  ```text
  the engine's output differs from the snapshots in 1 of 44 Builds

  Changes, each with the Builds it's in:
    rank.10.character-level: 19 → 20 in 1 Build: gold-enforcer
    slot.mythic.10: Mythic feat: - → Mythic feat: shield-focus-mythic "Shield Focus (Mythic)" in 1 Build: gold-enforcer
  ```

  Only `gold-enforcer.txt` was regenerated, by those two lines.
- **Gold:** the Enforcer takes MR 10 at level 20 with Shield Focus (Mythic); its `Reopened(171)` entry is deleted and the test passes with two gaps left (the Elk companion, 174, and Uncanny Dodge, 173).
- **Agreement:** `rules::agreement` judges each reached rank's level picker too; no disagreement on the test world's sample, the real data's, or seven Builds with moved ranks.
- **The proof:** not run (the queue's). No type it hashes changed: `Sheet`, `RankSheet`, `LevelSheet` and `CharacterState` have the fields they had, no option's status or reason changed for a Build that moves no rank, and the harness calls nothing new, so it needs no shim.

### Judgement calls

- **Pushing, not refusing,** for the ranks above a moved rank: moving MR 1 to the last level would otherwise take ten moves from the top down.
- **A rank's own level is kept while the rank below is later** rather than dropped, so moving the lower rank back restores the Build and its link exactly. Two Builds can so differ in a stored level that has no effect; `validate` accepts such a level and `sheet` gives the level the rank is really taken at.
- **A slot a rank's pick gives at its level** (Extra Feat's feat with an extra pick: a `FeatSelection` slot at the rank's level) is keyed by the level, so the move is locked while that slot is filled, naming its pick; cleared, the rank moves and the slot comes at its new level. Carrying the pick along was the alternative, and would have been a new rule.
- **The reason for a locked move** names the first three choices it would refuse and counts the rest.
- **The clause in the instructions.** The ticket words it as a pick whose prerequisite "comes after this slot". The engine says that only of a later slot at the same level, which taking the rank later never fixes; the case it fixes (a rank's pick that needs a later level's pick) was refused as plain "Needs Shield Focus". So the refusal now names the level of the rank that opens the pick, and the instructions say "a Mythic rank's pick whose prerequisite comes after this slot, at a later level". The hint is beside the engine's reason (in the MCP's and the web app's wording), not in the option's status, so no status the proof hashes changed for a Build that moves no rank.
- **The summary line and the move controls** are new on every Build with a rank, moved or not; nothing else on such a Build's screens changed.

### Done when

- The Enforcer's Mythic Shield Focus at MR 10 agrees in the gold test, its known-gap entry deleted: `gold::gold_builds_gain_what_the_write_ups_list` passes, and `gold::the_enforcers_mythic_shield_focus_needs_mr_10_taken_at_level_20` pins it on the real data.
- The snapshot diff holds only the intended change: the two lines above.
- `scripts/check.sh --full`: ALL-OK.

