# Build the option families where their owners are built

Type: task
Status: open
Blocked by: 68
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 68](68-build-picks-where-owned.md)'s follow-up (user, 2026-11-03: "we should patch the extraction data directly in code and not in runtime"; its recommendations taken, 2026-11-02). Ticket 68 left the largest label-then-replace passes in `libs/extract/src/build.rs`:

- `option_levels` and `option_families` (tickets 30–31, about 4,400 lines). They replace cells in several classes' rows in force at once, and they depend on which rows are sparse.
- `Picks::late`: the wild talents' and infusions' option prerequisites name element options that `option_families` builds, so today they're set right after it.
- The follow-ups that edit what the families build: `follow_ups`, `option_follow_ups`, `second_follow_ups`, the backgrounds, `option_class_skills`, `report_gaps` and `talent_follow_ups`.

Move each to where its owner is built, through the `Picks` registry or an owner-stage provider, so no label is created and then replaced. Where the sparse-rows order really matters, say how the new shape handles it.

## Build

AFK; the only agent in the repo. A refactor of `libs/extract` only: no change to `rules`, the schema, `libs/game-data/` or the link format. Use ticket 68's `Picks` registry (`libs/extract/src/picks.rs`) and its provider shape (`provide` reading the guide with its assertions, `check` read-only at the end). Keep every guide-sentence assertion, moved to where its fact is built. The user is asleep and said to take every recommendation (2026-11-02): where a pass can't sensibly move, keep it with a comment giving the reason, record your recommendation, and go on.

**Done when:**
- `libs/game-data/` is byte-identical (`git diff HEAD -- libs/game-data` empty).
- A fresh extraction into the scratchpad, report included, is byte-identical (`diff -r`) to one made with HEAD's extractor before the change. Re-check after each step.
- `cargo test --workspace -- --list` is unchanged apart from moved names, each explained in the Answer.
- No label-then-replace pass is left for what this ticket covers; any pass kept is listed in the Answer with its reason.
- The lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
