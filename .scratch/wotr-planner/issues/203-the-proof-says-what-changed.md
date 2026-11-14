# The proof says where a Build differs, not what

Type: tooling
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the retrospective of the queue of tickets 188 to 193 (2026-11-14; user: "do every recommendation"). A proof record holds hashes for most fields (`tools/proof/src/fingerprint.rs`), so `compare.txt` names the Builds, slots and fields that differ and nothing else. Attributing that queue's 2,161 differing Builds took an Opus agent about 294,000 tokens and three throwaway tools, deleted afterwards: a dump of every option in clear, a per-merge step comparison, and a record of the old crates on the new data to tell a code cause from a data cause. Each earlier proof entry in the map was attributed by hand the same way.

## Build

In `tools/proof` and `scripts/proof.sh`, nothing in `libs/` or `apps/`:

- **`proof dump <game-data dir> [--builds <file> | --links <file>] [--only <substring>] [--threads N]`**: the same Builds as `record`, written in clear, one fact per line as `<build>\t<path>\t<value>`: every slot option's status, open flag and reason text; the class, path and switch pickers the record hashes; `removable`; the sheet's feature lines per level; totals. One path names one fact, so two dumps line up by `(build, path)`. The paths follow the snapshot files' (`libs/seeds/tests/game_data/snapshots.rs`) where both print the same thing.
- **`proof explain <a.dump> <b.dump>`**: the differences, grouped as the snapshot check groups its message: by path with levels, ranks and indexes wildcarded and by the change (`old → new`), each group with its count of Builds and lines and three example Builds, largest group first; a slot or a Build only on one side is its own group. A long value is cut at a stated width in the group line and given in full once.
- **`scripts/proof.sh … -- --explain`** (a record option the script takes out before calling `record`): after the comparison, when it isn't empty, both harnesses dump only the differing Builds and `explain.txt` is written beside `compare.txt`. In `both` mode it also records side A's crates on side B's game data and says in `explain.txt`'s first lines whether the differences come from the data, the code, or both (the three comparisons' counts).
- The script's header and `tools/proof/src/lib.rs`'s docs say when to reach for it.

The agent who attributed the last proof left its scripts and outputs in that session's scratchpad (`attrib/` beside the proof's output folder, if it still exists): `pdiff_all.txt` shows the per-option detail that was needed, `final.py` and `classify.py` the grouping. They are a reference for the shape, not code to keep.

Stretch, only if the rest is done and it is cheap: `scripts/proof.sh steps <out> <ref>`, a data record at each first-parent merge between `<ref>` and `HEAD`, each compared with the one before, so a difference is tied to the merge that made it.

## Done when

Run on the last queue (`scripts/proof.sh both <out> ce9cc3b -- --explain`, side B at `8118dd5` or later), `explain.txt` shows the Extra Rogue Talent group as the largest, with its old and new reason texts, and says the differences come from the data; a test in `tools/proof` pins `explain`'s grouping on two small hand-made dumps; `scripts/check.sh` passes. The main session runs the proof itself; the ticket's agent tests with `--only`.
