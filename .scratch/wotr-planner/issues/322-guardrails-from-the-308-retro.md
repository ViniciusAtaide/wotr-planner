# Guardrails from the 308 retro: the carrier audit fails the check, `compare` sizes a narrowing, guarded files need the user

Type: build (tooling)
Status: open
Blocked by: 308h
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Problem

The retro of 2026-11-17's queue (user: "apply 1 2 4 5 8") found two classes of mistake that only reviews caught:

1. **Writes that refuse characters the game allows.** The game check compares blueprints by GUID, so it can't see who qualifies; the form proof's `prerequisite` census row attributes every line of a Build holding the entry (`regenerates: true`), so it can't tell a move toward the game from one away. 308a's first review found four such writes, 308d's two (Defensive Study, the Stalwart Defender). 308g and 308h guard new writes, but nothing fails when later data reopens a carrier gap, and a reviewer can't see a narrowing's size.
2. **Builders changing expectations instead of stopping.** 306aa swapped a corpus pick to keep a Build closed, 306s let 11 keys through in any slot, 308c weakened an exact reason to `.is_err()`, 308e reverted entries to make the Enforcer gold Build pass. The judgement half is now in `CODING_STANDARDS.md` ("An expectation moves only toward the game"); the mechanical half is here.

## Change

1. **The carrier audit fails the game check.** The plain game check runs `prerequisites::audit` (308h's `--carrier-audit`) and compares its hits with a committed list, `tools/game-check/carriers-known.txt` (one hit per line, each `-> <ticket>`, 307 for a carrier), the way it compares findings with `known.txt`: a hit not listed fails it, and a listed line no hit matches is stale and fails it. Seed the list from the audit at this ticket's base.
2. **`compare` sizes a prerequisite change.** With `FORM_PROOF_CENSUS`, `compare` prints, for each entry the log wrote, the option statuses that went to `Unmet` or `BreaksLater` in Builds whose link isn't refused, and splits all status changes into: options the log names, Builds whose links are refused, and the rest (named, at most `FORM_PROOF_SHOWN`). This folds in 306af's open part (a rename read as lost and new is out of scope here).
3. **Guarded files need the user.** A shell script `scripts/guarded-files.sh [<base>]` fails when the diff (the staged diff with no argument; `<base>..HEAD` with one) changes an expected value or `KNOWN_GAPS` in `libs/seeds/tests/game_data/gold.rs`, or adds, removes or edits a `held` line in `tools/game-check/user-facts.txt`, unless a ticket file under `.scratch/wotr-planner/issues/` in the same diff has a line starting `User approved:`. Wire it into `.githooks/pre-commit`, `scripts/queue-checks.sh` (with the queue's base, from an env var or argument it documents) and `.github/workflows/check.yml` (against the merge base).

## Done when

Tests for each: the check fails on an unlisted carrier hit and on a stale line; `compare`'s split on a fixture; the script on a staged gold change with and without the approval line (a shell test the pre-commit runs, or a `tools/` test). `cargo test -p game-check -p corpus` pass, the game check exits 0 at the tip with the seeded list, `bash -n` on each script, and `scripts/queue-checks.sh`'s new step runs.
