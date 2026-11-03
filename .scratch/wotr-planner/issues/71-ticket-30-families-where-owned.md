# Build ticket 30's option families where their owners are built

Type: task
Status: open
Blocked by: 70
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 69](69-option-families-where-owned.md)'s decisions (its recommendations taken, user, 2026-11-02). Ticket 69 kept `option_levels::apply` (ticket 30's seven families, the Primalist, `other_classes`, Second Spirit's line, Disruptive) because each archetype's decisions are read off its rows in force, known only once the rows are sparse. Record each placeholder cell as a marker when the rows are read, compare markers in `progressions::sparse_archetype_rows`, and decide each archetype right after its rows are made sparse, so nothing is labelled and then replaced. Fold in the follow-ups that edit what these families build (the rest of `follow_ups`, `option_follow_ups`, `second_follow_ups`, `option_class_skills`, `report_gaps`' scoundrel rows and `talent_follow_ups`), the Living Deity and Impossible Domain that ticket 70 kept (the Living Deity's rows are the Magic Deceiver's ways; Impossible Domain's prerequisite names the Living Deity's way, so it must exist before dangling references are dropped), and add the hooks ticket 69 found missing: skills, Perfect Mind, the Sword Saint's and half-elf's lists, Animal Focus, Champion of the Faith. If the shape allows, read the shaman hexes' "Wind" line as the shaman's Wind spirit directly and drop `Pick::SelectionKeepingFeature` (the data already says the spirit; the extraction must stay byte-identical).

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
