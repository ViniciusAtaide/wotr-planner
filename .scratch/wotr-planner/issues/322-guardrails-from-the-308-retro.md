# Guardrails from the 308 retro: the carrier audit fails the check, `compare` sizes a narrowing, guarded files need the user

Type: build (tooling)
Status: resolved
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

## Answer

Built on base 04420d25 (Opus builder; the three changes and their shape are the ticket's and the main session's brief, the rest the builder's, as marked). No game fact changes, so no blueprint or code citation applies.

**1. The carrier audit fails the game check.**
- `tools/game-check/src/carriers_known.rs` (new): `KnownCarriers` (`embedded`, `parse`, `compare`, `regenerate`) and `CarrierCheck` (the hits no line lists, the lines no hit matches). A line is a hit as the audit prints it; it matches the hit whose text before its last ` -> ` it repeats, so its ticket is free (307 as the audit prints it, or a slice's). A line without ` -> <ticket>` or a hit listed twice is refused (builder).
- `Report::with_carriers` prints `Carrier hits (ticket 322): N, …` and, failing, `Carrier hits carriers-known.txt doesn't list:` and `carriers-known.txt lines that match no carrier hit:` (as `with_stale_narrowings` does). `main.rs`: the plain check runs `prerequisites::audit` and compares; `--carrier-audit --known-format` prints the list written again (its header, each hit with its listed ticket or its own owner), refused without `--carrier-audit`.
- `tools/game-check/carriers-known.txt`: seeded from the audit at the base, **176 hits**, each `-> 307` (no hit names another owner). Its header gives the regeneration command, through a `.new` file and `mv`, since redirecting onto the file would empty it before cargo embeds it (builder). The main session regenerates it at each data merge: 308e and 308f will move the hits.

**2. `compare` sizes a prerequisite change** (`tools/corpus/tests/form_proof.rs`, census path only). With `FORM_PROOF_CENSUS` and the offers recorded, `OfferChanges` carries a `StatusSplit`. It prints `census narrowing <entry>: N statuses to Unmet or BreaksLater in N Builds (<before> -> <now> N, …)` for each log entry whose options went from an open kind to a refusing one in Builds whose link isn't refused, then how many entries narrowed none. It splits every change into those on options the log names, those in Builds whose links are refused (`LinkCensus.refused` now holds the names), and the rest, each `ELSE <Build>: <key>: …` (the first `FORM_PROOF_SHOWN`). Builder's choices: a change between two refusing kinds isn't a narrowing; a class's or archetype's own entry matches no option (they are no picks). `compare` still fails on any offer change, as before. 306af is resolved; "its open part" read as the retro's sizing need (inference).

**3. Guarded files need the user.** `scripts/guarded-files.sh [<base>]`: the staged diff, or `<base>..HEAD`. Guarded lines of `gold.rs` (its header defines them): inside `fn jack()`/`fn enforcer()`; `const KNOWN_GAPS`, `NAMES` and `VALUES` (builder: NAMES and VALUES decide what each write-up name expects); and `assert!`/`assert_eq!`/`assert_ne!` statements. Blank and `//` lines aside, and a removed and an added line with the same code once a trailing comment is stripped cancel (builder: 242's citations on pick lines would otherwise flag). In `user-facts.txt`: any removed or added line ending `-> held`. Approval: an added line starting `User approved:` under `.scratch/wotr-planner/issues/`. Exit 0, 1 naming each line, 2 on a bad base. Wired into `.githooks/pre-commit` (before the Rust filter; the test too when the script changes), `scripts/queue-checks.sh` (base from the argument, else `QUEUE_BASE`, else the merge base with the upstream; header documents it; two steps) and `.github/workflows/check.yml` (full history; the merge base with the default branch). `gold/*.ron` and `source.txt` are not guarded (not in the ticket).
- History: from the 308 queue's base (`60659ba2^1`) to 04420d25 it passes. Older ranges flag only 293e's creation of `user-facts.txt` (2026-11-16). 308e isn't merged at the base, so no approval line was added: 308e's gold change is the main session's to approve with the user.

**Tests** (each in `--list`):
- game-check: `an_unlisted_carrier_hit_and_a_stale_carrier_line_fail_the_check`, `a_carrier_line_matches_its_hit_whatever_its_ticket_and_regenerating_keeps_the_ticket`, `a_carrier_line_without_a_ticket_or_listed_twice_is_refused`, `the_embedded_carrier_list_parses`, `command::known_format_goes_with_the_carrier_audit`.
- corpus: `pieces::the_census_splits_the_status_changes_and_sizes_each_logged_entrys_narrowing`.
- `scripts/guarded-files-test.sh` (13 cases on a throwaway repository): a staged gold pick without and with approval, an approval not at the line's start, a known gap, a number inside a multi-line assert, a comment and an unguarded line, a citation added to a pick, a non-held and a held user fact, a committed range without and with approval, a bad base.

**Checks:** `cargo test -p game-check` (170 passed), `cargo test -p corpus --test form_proof` (23 passed, 2 ignored), `cargo fmt --all --check`, `scripts/test-modules.sh` (OK), `bash -n` on the four scripts, the guard's test (OK), and the game check (exit 0: 3,572 differences all known; 176 carrier hits, each listed). `queue-checks.sh` as a whole is the main session's; its usage prints with `--help`.
