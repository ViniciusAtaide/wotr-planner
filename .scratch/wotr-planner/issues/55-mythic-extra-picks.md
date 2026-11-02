# Mythic Extra Feat and Extra Mythic Ability give their pick

Type: task
Status: open
Blocked by: 54
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 49](49-extra-feats-extra-picks.md)'s decision 4 (user, 2026-11-02): Mythic Extra Feat ("You can select any normal feat") and Extra Mythic Ability ("You can select a mythic ability") give one more pick each time they're taken: a feat slot and a mythic ability slot, at the level (Mythic rank) they're taken. Today they give nothing but themselves.

Check both against their guide entries and the saved mythic pages: whether each repeats, whether the feat slot takes any feat whose prerequisites are met (or ignores them), and whether the mythic ability slot offers what a Mythic rank's ability slot offers (path abilities included or not). Look for other mythic feats or abilities that give a pick the same way and include them if the guide states it plainly. If the guide doesn't settle one, write a question with a recommendation and go on.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Prefer reusing ticket 49's machinery (`Feat::extra_pick_of`, `SlotId::FeatSelection`) if a feat slot and a mythic ability slot fit it; otherwise say what was added and why. Same data method as tickets 43–45 and 49, with before/after proof (only Builds taking these picks differ). Ticket 48's picker shortcut must stay exact (add these feats to the always-walked ones if their slot shifts later indices).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: Mythic Extra Feat taken at a Mythic rank gives one feat slot at that rank's level, offering what a normal feat slot offers there; Extra Mythic Ability gives one mythic ability slot; removing either is refused while its slot holds a pick and frees once it's cleared; the sheet labels the new slots (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
