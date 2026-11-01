# Hand-fix the extracted mythic data

Type: task
Status: open
Blocked by: 10
Map: [WotR Build Planner](../map.md)

## Question

Stage of the split extraction (user, 2026-11-01). Resolve the `## Mythic` section of `game-data/EXTRACTION_REPORT.md` by editing the RON files directly. AFK.

- **Files you may edit:** `game-data/mythic/**` and `game-data/progression.ron` only. Plus the report's `## Mythic` section.
- **Focus:** Mythic abilities and feats and their prerequisites, each path's rank table and path picks (Aeon gazes, Trickster tiers), path switch rules (kept ranks, level cap, further ascensions) for Legend, Gold Dragon, Swarm That Walks and Devil, and the ascension table 6/7/9/11/14/15/16/17×2/19.

**Shared rules for the hand-fix stages** (these four run in parallel, one agent each):
- **Your area:** fix only this stage's section of `game-data/EXTRACTION_REPORT.md`, and only this stage's files (below). Other agents are editing the other files at the same time.
- **Source:** the saved pages in `guide-cache/`. Never fetch GameFAQs or any site.
- **Copyright:** commit facts only (names, numbers, prerequisites, keys, anchors). A `summary` stays empty unless it's one short line in your own words.
- **Validating:** use the database-free check command from ticket 10's Answer. Never seed, reset or migrate the dev database; the main session seeds once all stages are done. No schema or `rules` type changes. Anything that needs one becomes a proposed follow-up in your Answer.
- **Unclear items:** anything you can't resolve accurately from the guide stays in the report, with a note on why. Don't guess.
- **Done when:** your section of the report holds only unresolvable items, the check command passes, `cargo test --workspace` passes, and there are no commits.
- **Recording:** append an `## Answer` with fixed and remaining counts and the follow-ups, set `Status: resolved`, and add one line at the end of map.md's Decisions so far using Edit. Keep a dated `## Progress log`, because runs can be cut off.
