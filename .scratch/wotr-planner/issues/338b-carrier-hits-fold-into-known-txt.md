# The carrier hits fold into `known.txt`

Type: build (tooling)
Status: open
Blocked by: 338a
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-L.md, T7b](../research/architecture/design-L.md), in the frame of [overall.md](../research/architecture/overall.md); decisions in [decisions.md](../research/architecture/decisions.md).

## Problem

Ticket 322 gave the check a third comparison, through a list with its own grammar (`carriers-known.txt`). Ticket 307c would add a fifth list.

## Change

The carrier hits become findings of a new `Area::Carriers`, placed last in `Area::ALL`. `carriers-known.txt`'s lines move into `known.txt` in the same commit, inside a `# BEGIN carriers` / `# END carriers` block that `--known-format` regenerates without touching `known.txt`'s prose (design-L §2). `carriers-known.txt`, `carriers_known.rs` and `with_carriers` go.

## Done when

- The report matches the base's except for three changes:
  - one more Areas line;
  - the carrier section in its new form;
  - the Differences line's counts.

  The builder shows this as a `diff` of the two reports, listing every changed line.
- The check exits 0.
- Each of these gives exit 1: a planted unlisted hit, a stale block line, and a `?` line.
- `--known-format` round-trips the tip's `known.txt` byte for byte.
- `carriers-known.txt` is gone.

## Files

- `tools/game-check/src/areas/{mod,carriers}.rs`
- `tools/game-check/src/{report,main,lib}.rs`
- `tools/game-check/src/ledger/known.rs`
- `tools/game-check/known.txt`
- deleted: `tools/game-check/carriers-known.txt` and `src/carriers_known.rs`
- `tools/game-check/tests/fixture/{prerequisites_write,command,known}.rs`

The main session then edits two things: the carrier sentence in CLAUDE.md's "Behaviour checks" section, and ticket 307c, which then targets `known.txt`.

## Checks

- The same checks as 338a.
- `--carrier-audit --known-format`, diffed against the file.
