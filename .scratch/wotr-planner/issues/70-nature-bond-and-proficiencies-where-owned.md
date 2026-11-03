# Build the nature bond and proficiencies where their owners are built

Type: task
Status: open
Blocked by: 69
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 68](68-build-picks-where-owned.md)'s follow-up (its recommendations taken, user, 2026-11-02). Two passes ticket 68 kept still patch finished data:

- **`class_rules::nature_bond`** is label-then-replace. The level-1 companion feature it removes is still what Boon Companion's and Mythical Beast's guide lines resolve to, and a later pass removes that feature. Build the nature bond's slot where the class is built, and resolve those two lines to what they mean without the intermediate feature.
- **`proficiencies`** drops a proficiency feature its feats cover, on the full rows, and its tests drive it on hand-made data. Build only the facts that survive, where the class is built. Keep or adapt its tests so the same cases are covered.

Also fold in any pass ticket 69 kept that this shape now allows.

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
