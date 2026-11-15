# Clean up the logic left from the guide and the proof

Type: build
Status: open. Queued last, after every other ticket of this queue (user, 2026-11-15).
Blocked by: [the game data laid out by blueprint](244-the-game-data-laid-out-by-blueprint.md) (it rewrites the data folder and the corpus's Builds read the data)
Model: Opus to build: it changes the corpus the self-checks run on.
Map: [WotR Build Planner](../map.md)

## Problem

Ticket 234 replaced the guide with the game as the source of truth for rules. 235 removed the proof, the two-sided record that compared the engine with an earlier commit. The rank and context splits ticket and the layout ticket remove the planner's old data shapes. Three pieces of the old logic are covered by none of them (user, 2026-11-15: "Yes, file it").

## Change

1. **The corpus generator's compatibility quirks** (`tools/corpus/src/generate.rs:3-4, 64, 188, 221`; `builds/classes.rs:74`; `builds/selections.rs:60`).
   - 235 kept each quirk, because it decides which Builds the corpus holds (its Answer: 2,163 Builds, names hash `50a7a1336a9f14d0`, Debug hash `0f53b5145c6d5a56`). The quirks existed so that records from two commits could be compared, and the proof that compared them is gone.
   - Examples: Builds that name no class choice, rank slots never filled, a pass that reads the sheet once before it fills.
   - **Change:** remove each quirk whose only reason was comparison. Where a quirk still covers something the self-checks need (a Build with an unfilled slot, say), keep that Build on purpose, as a named case.
   - The corpus changes. The Answer gives its new size and hashes, and what each removed quirk took out or added. The self-checks' long runs pass on the new corpus.
2. **The `allows` battery's dropped edits.** 235 dropped the first-ascension, withdrawal, `RemoveLevel` and `SetName` edits, because nothing asserted them (235's Answer, item 3).
   - **Change:** decide per edit whether an invariant holds and is worth checking. Examples: each first-ascension option's status agrees with `allows` of its edit; a withdrawal's `allows` agrees with `removable`.
   - Add the ones that hold to `tools/corpus/tests/self_checks.rs`. List the rest in the Answer with the reason not to check them.
3. **The guide-era leftovers in the data folder and its tooling:**
   - `libs/game-data/EXTRACTION_REPORT.md`, the extractor's report, at 0 items since the hand-fix stages;
   - the README's extraction history (`libs/game-data/README.md`);
   - any code, script or doc still describing the guide as the source of the data rather than of summaries and links;
   - `libs/game-data/interactions.ron`'s entries that read "the guide doesn't say …", which `describe` serves to agents (ticket 240 left them as data);
   - the doc comments in `libs/rules` and `apps/mcp/src/shapes.rs` that say "what the guide says…" of a fact.
   - **Change:** delete what only records the extraction; the History in the map and the tickets keep it. Reword what remains so it describes the game as the source of truth. Search with `git grep -n -i -E 'extract|the guide (is|wins|says)'` outside `.scratch/`.

## Done when

- No quirk is left whose only reason is a cross-commit comparison. The corpus's new size and hashes are in the Answer, and the self-checks' long runs pass on it.
- Each dropped battery edit is checked by a self-check or named in the Answer with the reason it isn't.
- No file outside `.scratch/` and `libs/game-data/notes/` describes the guide as the data's source.
- `scripts/check.sh` passes.
