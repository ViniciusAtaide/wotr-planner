# The proof says where a Build differs, not what

Type: tooling
Status: resolved
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

## Answer

Built 2026-11-14 in the lane-tools worktree.

**What was built** (`tools/proof` and `scripts/proof.sh` only):

- **`proof dump`** (`tools/proof/src/dump.rs`): the facts behind every field of a record, one per line, `<build>\t<path>\t<value>`, sorted by Build then path (a path twice is numbered `#2`). Paths follow the snapshot files' dot form (`sheet.levels.3.state.bab`, `slot.Feat.1.General.0.option/power-attack`, `totals.5.…`, `caster.5.<spell>.…`, `classes.4.option/…`, `paths.option/…`, `switches.option/…`, `removable/<choice>`, `allows/<edit>`). A lock reason is its text. The sheet, totals and caster lines are the engine's Debug texts read into fields by a small reader (a map by its keys, a list by its elements' `id`, `rank`, `character_level`, `key` or `name`, so an insertion doesn't rename the rest), so the harness still compiles against an older ref's crates (checked: ref `ce9cc3b`'s crates build and agree with the working tree's harness on the old data). `--names <file>` keeps the Builds a file lists; `--only`, `--threads`, `--light`, `--fold-values`, `--links`, `--builds` are `record`'s. Builds go out in chunks, so memory is a chunk's lines.
- **`proof explain <a> <b> [--top N]`** (`tools/proof/src/explain.rs`): streams the two dumps, holds one Build's lines from each, and groups the differences by wildcarded path (every number between dots is `*`; an option's key keeps its numbers) and change (`old → new`, `+ value`, `- value`), with the Builds and lines counted and three example Builds, largest group first (by Builds, then lines). A slot only on one side is a group of its own (`slot only in A (kind)`), as is a Build only on one side. A changed `-hash` line (option order, lock-reason issues) is a group only when no other line of its slot or picker differs. A value over 100 characters is cut, a change of a list of alternatives (`Needs a or b or c`) says which alternatives were added or dropped, and every cut value is given in full at the end as `[n]`.
- **`scripts/proof.sh … -- --explain`**: taken out of the record options; after a non-empty comparison both harnesses dump the Builds `compare.txt` names (`differing.txt`, `A.dump`, `B.dump`) and `explain.txt` is written: first lines say where the differences come from, then `proof explain`. `data` and `rules` say so outright. `both` records side A's crates on B's game data (C: A->C is the data, C->B the code) and, where A's crates can't read B's data (an older crate rejects a field the new data has: it happens, `C.err`), side B's crates on A's data (D), and gives the three counts and a verdict. The usage is in the script's header and in the crate's docs.

**Fields not in clear:** the order of a picker's options and the issues' Debug texts are hashed lines (`order-hash`, `reason-hash`: a reorder, or an issue that changes without changing its text, shows as that line); everything else a record hashes has its facts. An option's *edit* isn't in a record's hash either.

**Decisions:** group order by Builds then lines, hash lines held back when explained, D as the fallback when C can't be read (all the builder's; the ticket text left them open).

**Tests:** `tools/proof/tests/explain.rs` (`differences_are_grouped_by_wildcarded_path_and_change_largest_first` pins the grouping on two small hand-made dumps, plus wildcards, the long-value cut, an agreeing pair, an out-of-order dump); `tools/proof/tests/dump.rs` (every kind of fact, sorted and unique, full against light, Builds by name); the reader's unit tests in `dump.rs`.

**Snapshot message:** none, the check was silent (no behaviour changed).

**Not verified here:** the end-to-end `scripts/proof.sh both <out> ce9cc3b -- --explain` (the main session runs the proof). What was run: `dump` and `explain` by hand on the old data against the current (`--only rogue`: the Extra Rogue Talent group is first, 68 of 71 Builds), the script's `where_from` function with the sample records (C unreadable, D chosen, "from the data"). The proof's size: about 1.5 MB of dump a Build, so a few GB a side when most Builds differ.
