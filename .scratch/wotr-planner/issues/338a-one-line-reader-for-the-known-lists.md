# One line reader and one reconcile for the game check's lists

Type: build (tooling)
Status: open
Blocked by: 337b
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)
Design: [design-L.md, T7a](../research/architecture/design-L.md) (interface §1, what goes §2, tests §4, risks §5), in the frame of [overall.md](../research/architecture/overall.md); decisions 3 and 7 in [decisions.md](../research/architecture/decisions.md).

## Problem

The game check's four lists each have their own parser and their own idea of a stale line. Every list's error says `known.txt` (`lib.rs:127`), and the same owner is recorded twice: by `owner()` and by hand.

## Change

- Add `src/ledger/` with the line reader, the reconcile, `KnownDifferences` and `WriteRegistry` (design-L §1), run over today's four files.
- Delete `owner()`. `user-facts.txt` is read and never written, so its `held` lines stay byte-identical. A stale `held` line still fails only under `--write` (decision 3).

## Done when

- `cargo test -p game-check` passes, with each new test from design-L §4 in `--list`.
- The plain check's stdout is byte-identical to the base's, compared with `cmp`.
- The output of `--write <every area> --dry-run` is byte-identical to the base's.
- Exit codes 0, 1 and 2 keep today's meanings.
- A deliberately broken line in each of the four files prints that file's own name.

## Files

- new `tools/game-check/src/ledger/{mod,lines,reconcile,known,registry}.rs`
- `tools/game-check/src/{lib,report,main}.rs` and `src/write/{mod,tables,prerequisites}.rs`
- deleted `src/known.rs` and `src/write/facts.rs`
- `tools/game-check/tests/fixture/{known,write,prerequisites_write,support,pins,determinism}.rs`

## Checks

- `cargo test -p game-check`
- `cargo fmt --all --check`
- `scripts/test-modules.sh`
- the game check, on the base and the tip
